//! Discovery Daemon (发现守护进程)
//!
//! 对应 NRCS Java: getMorePeersThread (30秒间隔)
//!
//! 职责:
//! - 从连接的节点获取更多节点
//! - 向其他节点广播自己的节点列表
//! - 节点持久化

use crate::config::P2PConfig;
use crate::peer::{Peer, PeerState, Peers};
use crate::protocol::{PeerRequest, RequestType};
use orm::PeerRepository;
use orm::models::misc::PeerModel;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Discovery Daemon
///
/// 对应 NRCS Java: getMorePeersThread
pub struct DiscoveryDaemon {
    peers: Arc<Peers>,
    config: P2PConfig,
    peer_repo: Option<Arc<dyn PeerRepository>>,
    running: Arc<RwLock<bool>>,
}

impl DiscoveryDaemon {
    /// Create a new discovery daemon
    pub fn new(peers: Arc<Peers>, config: P2PConfig) -> Self {
        Self {
            peers,
            config,
            peer_repo: None,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Set peer repository for persistence
    pub fn set_peer_repo(&mut self, repo: Arc<dyn PeerRepository>) {
        self.peer_repo = Some(repo);
    }

    /// Start the discovery daemon
    pub async fn start(&self) {
        let mut running = self.running.write().await;
        if *running {
            warn!("Discovery daemon is already running");
            return;
        }
        *running = true;
        drop(running);

        info!("Discovery daemon started (interval: {}s)", self.config.discovery_daemon_interval_secs);

        let peers = Arc::clone(&self.peers);
        let config = self.config.clone();
        let running = Arc::clone(&self.running);
        let peer_repo = self.peer_repo.clone();

        tokio::spawn(async move {
            loop {
                if !*running.read().await {
                    break;
                }

                tokio::time::sleep(Duration::from_secs(config.discovery_daemon_interval_secs)).await;

                if let Err(e) = Self::discovery_loop(&peers, &config).await {
                    warn!("Discovery loop error: {}", e);
                }

                // Persist peers to database
                if let Some(ref repo) = peer_repo {
                    if let Err(e) = Self::persist_peers(&peers, repo).await {
                        warn!("Peer persistence error: {}", e);
                    }
                }
            }
            info!("Discovery daemon stopped");
        });
    }

    /// Stop the discovery daemon
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Discovery daemon stopping...");
    }

    /// Main discovery loop
    /// 
    /// 对应 NRCS Java: getMorePeersThread.run()
    async fn discovery_loop(peers: &Arc<Peers>, config: &P2PConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 检查是否有太多已知节点
        let count = peers.known_peers_count().await;
        if config.too_many_known_peers(count) {
            return Ok(());
        }

        // 1. 从连接的节点获取更多节点
        if let Some(peer) = peers.get_any_peer(PeerState::Connected, true).await {
            Self::request_peers_from_peer(peers, &peer, config).await?;
        }

        // 2. 向其他节点广播自己的节点列表
        Self::share_my_peers(peers, config).await?;

        Ok(())
    }

    /// Request peers from a connected peer（完整实现）
    ///
    /// 对应 NRCS Java: getMorePeersThread 中的 getPeers 请求
    async fn request_peers_from_peer(
        peers: &Arc<Peers>,
        peer: &Peer,
        _config: &P2PConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::websocket::WebsocketClient;

        // 1. 构建 getPeers 请求
        let request = PeerRequest::new(RequestType::GetPeers, 1);

        // 2. 发送请求
        debug!("[DiscoveryDaemon] Requesting peers from {}", peer.address);

        match WebsocketClient::send_request(peer.address, request).await {
            Ok(response) => {
                // 3. 解析返回的节点列表
                if let Some(peers_arr) = response.get("peers").and_then(|v| v.as_array()) {
                    let mut added = 0usize;

                    for peer_info in peers_arr {
                        // 提取地址信息（兼容 camelCase 和 snake_case）
                        let addr_str = peer_info.get("announcedAddress")
                            .or_else(|| peer_info.get("announced_address"))
                            .or_else(|| peer_info.get("address"))
                            .and_then(|v| v.as_str());

                        if let Some(addr_str) = addr_str {
                            if let Ok(addr) = addr_str.parse::<std::net::SocketAddr>() {
                                if !peers.contains_peer(&addr).await {
                                    if peers.is_blacklisted_addr(&addr).await {
                                        continue;
                                    }
                                    let mut new_peer = Peer::new(addr, false);
                                    new_peer.set_announced_address(addr_str.to_string());
                                    peers.register_peer(new_peer).await;
                                    added += 1;
                                }
                            }
                        }
                    }

                    if added > 0 {
                        debug!("[DiscoveryDaemon] Discovered {} new peers from {}", added, peer.address);
                    } else {
                        debug!("[DiscoveryDaemon] No new peers from {}", peer.address);
                    }
                }
            }
            Err(e) => {
                debug!("[DiscoveryDaemon] Failed to get peers from {}: {}", peer.address, e);
            }
        }

        Ok(())
    }

    /// Share my peers with other peers（完整实现）
    ///
    /// 对应 NRCS Java: getMorePeersThread 中的 addPeers 请求
    async fn share_my_peers(
        peers: &Arc<Peers>,
        config: &P2PConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::websocket::WebsocketClient;
        use serde_json::{Map as JsonMap, Value};

        // 1. 获取可分享的节点列表
        let all_peers = peers.get_known_peers().await;
        let mut shareable = Vec::new();

        for peer in all_peers {
            if peers.is_peer_blacklisted(&peer).await {
                continue;
            }
            if peer.announced_address.is_none() {
                continue;
            }
            if !peer.share_address {
                continue;
            }

            // 构建节点信息（只包含必要字段，与 Java 一致）
            let mut peer_info = JsonMap::new();

            // 解析 announcedAddress 为 address + port
            if let Some(ref addr_str) = peer.announced_address {
                if let Some((host, port)) = addr_str.rsplit_once(':') {
                    peer_info.insert("address".into(), Value::String(host.to_string()));
                    if let Ok(port_num) = port.parse::<i64>() {
                        peer_info.insert("port".into(), Value::Number(serde_json::Number::from(port_num)));
                    }
                } else {
                    peer_info.insert("address".into(), Value::String(addr_str.clone()));
                }
            }

            // 服务标志
            peer_info.insert("services".into(),
                           Value::Number(serde_json::Number::from(peer.services)));

            shareable.push(Value::Object(peer_info));
        }

        if shareable.is_empty() {
            return Ok(());
        }

        // 2. 选择几个已连接节点进行分享（最多 5 个）
        let connected = peers.get_public_peers(PeerState::Connected).await;
        let share_targets: Vec<&Peer> = connected.iter()
            .take(config.send_to_peers_limit.min(5))
            .collect();

        if share_targets.is_empty() {
            return Ok(());
        }

        // 3. 构建 addPeers 请求
        let shareable_len = shareable.len();
        let mut request = PeerRequest::new(RequestType::AddPeers, 1);
        request.set("peers", shareable);

        // 4. 并发发送给目标节点
        let share_targets_len = share_targets.len();

        for target in share_targets {
            let target_addr = target.address;
            let req_clone = request.clone();

            tokio::spawn(async move {
                match WebsocketClient::send_request(target_addr, req_clone).await {
                    Ok(resp) => {
                        if let Some(added) = resp.get("added").and_then(|v| v.as_i64()) {
                            debug!("[DiscoveryDaemon] Shared {} peers with {} (accepted {})",
                                   shareable_len, target_addr, added);
                        }
                    }
                    Err(e) => {
                        debug!("[DiscoveryDaemon] Failed to share peers with {}: {}", target_addr, e);
                    }
                }
            });
        }

        debug!("[DiscoveryDaemon] Sharing {} peers with {} targets", shareable_len, share_targets_len);
        Ok(())
    }

    /// Persist peers to database
    async fn persist_peers(peers: &Arc<Peers>, repo: &Arc<dyn PeerRepository>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let all_peers = peers.get_active_peers().await;

        if all_peers.is_empty() {
            return Ok(());
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i32;

        let mut saved = 0usize;
        for peer in &all_peers {
            let addr_str = peer.address.to_string();
            let model = PeerModel {
                address: addr_str,
                last_updated: Some(now),
                services: Some(peer.services as i64),
            };
            if repo.upsert(&model).await.is_ok() {
                saved += 1;
            }
        }

        if saved > 0 {
            debug!("[DiscoveryDaemon] Persisted {} peers to database", saved);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_discovery_daemon_creation() {
        // TODO: 添加测试
    }
}
