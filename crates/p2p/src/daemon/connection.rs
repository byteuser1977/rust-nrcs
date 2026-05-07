//! Connection Daemon (连接守护进程)
//!
//! 对应 NRCS Java: peerConnectingThread (5秒间隔)
//!
//! 职责:
//! - 维护出站连接
//! - 自动重连断开的节点
//! - 管理入站连接
//! - 实施连接限制
//! - 连接知名节点

use crate::config::P2PConfig;
use crate::peer::{Peer, PeerState, Peers};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Connection Daemon
/// 
/// 对应 NRCS Java: peerConnectingThread
pub struct ConnectionDaemon {
    peers: Arc<Peers>,
    config: P2PConfig,
    running: Arc<RwLock<bool>>,
}

impl ConnectionDaemon {
    /// Create a new connection daemon
    pub fn new(peers: Arc<Peers>, config: P2PConfig) -> Self {
        Self {
            peers,
            config,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the connection daemon
    pub async fn start(&self) {
        let mut running = self.running.write().await;
        if *running {
            warn!("Connection daemon is already running");
            return;
        }
        *running = true;
        drop(running);

        info!("Connection daemon started (interval: {}s)", self.config.connection_daemon_interval_secs);
        
        let peers = Arc::clone(&self.peers);
        let config = self.config.clone();
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            loop {
                if !*running.read().await {
                    break;
                }

                tokio::time::sleep(Duration::from_secs(config.connection_daemon_interval_secs)).await;
                
                if let Err(e) = Self::connection_loop(&peers, &config).await {
                    warn!("Connection loop error: {}", e);
                }
            }
            info!("Connection daemon stopped");
        });
    }

    /// Stop the connection daemon
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Connection daemon stopping...");
    }

    /// Main connection loop
    /// 
    /// 对应 NRCS Java: peerConnectingThread.run()
    async fn connection_loop(peers: &Arc<Peers>, config: &P2PConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = current_timestamp();

        // 1. 连接新节点
        if !Self::has_enough_connected_peers(peers, config).await {
            Self::connect_to_new_peers(peers, config).await?;
        }

        // 2. 重连断开的节点
        Self::reconnect_stale_peers(peers, config, now).await?;

        // 3. 清理过时的入站连接
        Self::cleanup_inbound_connections(peers, config, now).await?;

        // 4. 清理过多已知节点
        Self::prune_known_peers(peers, config, now).await?;

        // 5. 连接知名节点
        Self::connect_well_known_peers(peers, config).await?;

        Ok(())
    }

    /// Check if we have enough connected peers
    async fn has_enough_connected_peers(peers: &Arc<Peers>, config: &P2PConfig) -> bool {
        let active_peers = peers.get_active_peers().await;
        config.has_enough_connected_peers(active_peers.len())
    }

    /// Connect to new peers
    /// 
    /// 对应 NRCS Java: peerConnectingThread 中的连接逻辑
    async fn connect_to_new_peers(peers: &Arc<Peers>, _config: &P2PConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = current_timestamp();

        let all_peers = peers.get_known_peers().await;
        
        let mut connectable = Vec::new();
        for peer in all_peers {
            if peers.is_peer_blacklisted(&peer).await {
                continue;
            }
            if peer.announced_address.is_none() {
                continue;
            }
            if peer.state == PeerState::Connected {
                continue;
            }
            if now - peer.last_connect_attempt <= 600 {
                continue;
            }
            connectable.push(peer);
        }

        // 分离 hallmarked 和非 hallmarked 节点
        let (hallmarked, non_hallmarked): (Vec<_>, Vec<_>) = connectable
            .into_iter()
            .partition(|p| {
                // 检查是否提供 HALLMARK 服务
                p.services & 0x01 != 0
            });

        if hallmarked.is_empty() && non_hallmarked.is_empty() {
            return Ok(());
        }

        // 随机选择最多 10 个节点进行连接
        let mut to_connect = Vec::new();
        let mut rng = SimpleRng::new(now as u64);
        
        for _ in 0..10 {
            let peer_list = if hallmarked.is_empty() {
                &non_hallmarked
            } else if non_hallmarked.is_empty() {
                &hallmarked
            } else {
                // 随机选择 hallmarked 或非 hallmarked
                if rng.next_bool() {
                    &hallmarked
                } else {
                    &non_hallmarked
                }
            };

            if !peer_list.is_empty() {
                let idx = rng.next_usize(peer_list.len());
                to_connect.push(peer_list[idx].clone());
            }
        }

        // 并发连接
        for peer in to_connect {
            let peers_clone = Arc::clone(peers);
            tokio::spawn(async move {
                if let Err(e) = Self::connect_peer(&peers_clone, &peer).await {
                    debug!("Failed to connect to peer {}: {}", peer.address, e);
                }
            });
        }

        Ok(())
    }

    /// Connect to a single peer（完整实现）
    ///
    /// 对应 Java: Peers.connectPeer(Peer peer)
    /// 连接前先解除黑名单（与 Java 一致）
    async fn connect_peer(peers: &Arc<Peers>, peer: &Peer) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::error::ErrorCode;

        // 1. 检查离线模式
        let config = P2PConfig::default();
        if config.offline_mode {
            debug!("[ConnectionDaemon] Offline mode, skipping connection to {}", peer.address);
            return Ok(());
        }

        // 2. 获取可变引用
        if let Some(peer_ref) = peers.get_peer(&peer.address).await {
            let mut p = peer_ref.lock().await;

            // 3. 连接前先解除黑名单（对应 Java: peer.unBlacklist()）
            if p.is_blacklisted() {
                p.un_blacklist();
                debug!("[ConnectionDaemon] Unblacklisted peer {} before connecting", peer.address);
            }

            // 4. 调用完整的连接握手流程
            match p.connect_with_peers(&config, Some(peers)).await {
                Ok(response) => {
                    debug!("[ConnectionDaemon] Successfully connected to {}: app={}, ver={}",
                          peer.address,
                          p.application.as_deref().unwrap_or("?"),
                          p.version.as_deref().unwrap_or("?"));

                    // 5. 注册活跃连接
                    peers.add_connection(peer.address).await;

                    debug!("[ConnectionDaemon] Connect response from {}: {:?}", peer.address, response);
                    Ok(())
                }
                Err(e) => {
                    warn!("[ConnectionDaemon] Failed to connect to peer {}: {}", peer.address, e);

                    // 6. 连接失败处理（根据错误类型决定是否加入黑名单）
                    let error_code = e.code;
                    if matches!(error_code, ErrorCode::ConnectionTimeout | ErrorCode::ReadTimeout) {
                        p.blacklist(format!("Connection failed (timeout): {}", e));
                    } else if matches!(error_code, ErrorCode::Blacklisted) {
                        // 已经在黑名单中，不需要再次操作
                    } else {
                        p.deactivate();
                    }
                    Err(format!("Connect failed: {}", e).into())
                }
            }
        } else {
            Err("Peer not found in registry".into())
        }
    }

    /// Reconnect stale peers
    /// 
    /// 对应 NRCS Java: 重连 lastUpdated > 3600 的节点
    async fn reconnect_stale_peers(peers: &Arc<Peers>, _config: &P2PConfig, now: i64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let all_peers = peers.get_known_peers().await;
        
        for peer in all_peers {
            if peer.state == PeerState::Connected
                && now - peer.last_updated > 3600
                && now - peer.last_updated > 600 {
                let peers_clone = Arc::clone(peers);
                tokio::spawn(async move {
                    if let Err(e) = Self::connect_peer(&peers_clone, &peer).await {
                        debug!("Failed to reconnect to peer {}: {}", peer.address, e);
                    }
                });
            }
        }

        Ok(())
    }

    /// Cleanup inbound connections（完整实现）
    ///
    /// 对应 NRCS Java: 清理 lastInboundRequest > 3600 秒的入站连接
    async fn cleanup_inbound_connections(peers: &Arc<Peers>, _config: &P2PConfig, now: i64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let active_peers = peers.get_active_peers().await;
        let mut cleaned = 0usize;

        for peer in active_peers {
            // 如果是入站连接且超过 3600 秒没有活动，断开
            if peer.is_inbound && (peer.last_inbound_request == 0 || now - peer.last_inbound_request > 3600) {
                peers.remove_connection(&peer.address).await;

                // 更新 peer 状态
                if let Some(peer_ref) = peers.get_peer(&peer.address).await {
                    let mut p = peer_ref.lock().await;
                    p.is_inbound = false;
                    p.fire_event(crate::peer::PeerEvent::RemoveInbound);
                    p.deactivate();
                }

                debug!("[ConnectionDaemon] Cleaned up stale inbound connection from {}", peer.address);
                cleaned += 1;
            }
        }

        if cleaned > 0 {
            debug!("[ConnectionDaemon] Cleaned up {} stale inbound connections", cleaned);
        }

        Ok(())
    }

    /// Prune known peers（完整实现）
    ///
    /// 对应 NRCS Java: 删除过多的已知节点
    async fn prune_known_peers(peers: &Arc<Peers>, config: &P2PConfig, now: i64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let count = peers.known_peers_count().await;

        if config.too_many_known_peers(count) && Self::has_enough_connected_peers(peers, config).await {
            let all_peers = peers.get_known_peers().await;
            let mut pruned_count = 0usize;

            // 删除 lastUpdated > 24 小时的非连接节点
            for peer in all_peers {
                if peer.state != PeerState::Connected && now - peer.last_updated > 24 * 3600 {
                    // 实现节点删除
                    debug!("[ConnectionDaemon] Pruning old peer: {}", peer.address);
                    peers.remove_peer(&peer.address).await;
                    pruned_count += 1;
                }
            }

            if pruned_count > 0 {
                debug!("[ConnectionDaemon] Pruned {} old peers", pruned_count);
            }
        }

        Ok(())
    }

    /// Connect to well-known peers（完整实现）
    ///
    /// 对应 NRCS Java: 连接 wellKnownPeers 配置中的节点
    async fn connect_well_known_peers(
        peers: &Arc<Peers>,
        config: &P2PConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 获取知名节点列表（从配置）
        let well_known_addrs = &config.well_known_peers;

        if well_known_addrs.is_empty() {
            return Ok(());
        }

        let mut connected_count = 0usize;

        for addr_str in well_known_addrs {
            // 解析地址
            match addr_str.parse::<std::net::SocketAddr>() {
                Ok(addr) => {
                    // 检查是否已存在
                    if !peers.contains_peer(&addr).await {
                    // 检查是否在黑名单中
                    if peers.is_blacklisted_addr(&addr).await {
                        debug!("[ConnectionDaemon] Well-known peer {} is blacklisted, skipping", addr);
                        continue;
                    }

                        // 创建并注册新节点
                        let mut new_peer = Peer::new(addr, false); // outbound connection
                        new_peer.services |= 0x01; // 标记为知名节点

                        peers.register_peer(new_peer).await;

                        debug!("[ConnectionDaemon] Registered well-known peer: {}", addr);

                        // 尝试连接
                        if let Some(peer_ref) = peers.get_peer(&addr).await {
                            let peer_snapshot = peer_ref.lock().await.clone();
                            drop(peer_ref);

                            if let Err(e) = Self::connect_peer(peers, &peer_snapshot).await {
                                debug!("[ConnectionDaemon] Failed to connect well-known peer {}: {}", addr, e);
                            } else {
                                connected_count += 1;
                            }
                        }
                    } else {
                        // 节点已存在，检查是否需要重连
                        if let Some(peer_ref) = peers.get_peer(&addr).await {
                            let peer = peer_ref.lock().await;
                            if peer.state != PeerState::Connected {
                                drop(peer);
                                let peer_snapshot = peer_ref.lock().await.clone();
                                drop(peer_ref);

                                if let Err(e) = Self::connect_peer(peers, &peer_snapshot).await {
                                    debug!("[ConnectionDaemon] Failed to reconnect well-known peer {}: {}", addr, e);
                                } else {
                                    connected_count += 1;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("[ConnectionDaemon] Invalid well-known peer address '{}': {}", addr_str, e);
                }
            }
        }

        if connected_count > 0 {
            debug!("[ConnectionDaemon] Connected to {}/{} well-known peers",
                   connected_count, well_known_addrs.len());
        }

        Ok(())
    }
}

/// Get current timestamp in seconds
fn current_timestamp() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Simple random number generator (避免依赖 rand crate)
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        // xorshift64
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    fn next_bool(&mut self) -> bool {
        self.next_u64().is_multiple_of(2)
    }

    fn next_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            return 0;
        }
        (self.next_u64() as usize) % max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts > 0);
    }
}
