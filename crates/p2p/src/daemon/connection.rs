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
        // TODO: 实现知名节点连接

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

        // 获取可连接的节点列表
        let all_peers = peers.get_known_peers().await;
        
        // 先过滤出可连接的节点（不能在 filter 中使用 await）
        let mut connectable = Vec::new();
        for peer in all_peers {
            if peers.is_blacklisted_addr(&peer.address).await {
                continue;
            }
            if peer.announced_address.is_none() {
                continue;
            }
            if peer.state == PeerState::Connected {
                continue;
            }
            if now - peer.last_updated <= 600 {
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

    /// Connect to a single peer
    async fn connect_peer(peers: &Arc<Peers>, peer: &Peer) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 使用 WebSocket 或 HTTP 连接
        if peers.is_blacklisted_addr(&peer.address).await {
            return Err("Peer is blacklisted".into());
        }

        // TODO: 实现实际的连接逻辑
        // 这里应该调用 websocket.rs 或 http.rs 中的连接方法
        
        debug!("Connecting to peer: {}", peer.address);
        Ok(())
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

    /// Cleanup inbound connections
    /// 
    /// 对应 NRCS Java: 清理 lastInboundRequest 过期的连接
    async fn cleanup_inbound_connections(_peers: &Arc<Peers>, _config: &P2PConfig, _now: i64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: 实现入站连接清理
        Ok(())
    }

    /// Prune known peers
    /// 
    /// 对应 NRCS Java: 删除过多的已知节点
    async fn prune_known_peers(peers: &Arc<Peers>, config: &P2PConfig, now: i64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let count = peers.known_peers_count().await;
        
        if config.too_many_known_peers(count) && Self::has_enough_connected_peers(peers, config).await {
            let all_peers = peers.get_known_peers().await;
            
            // 删除 lastUpdated > 24 小时的节点
            for peer in all_peers {
                if now - peer.last_updated > 24 * 3600 {
                    // TODO: 实现节点删除
                    debug!("Pruning old peer: {}", peer.address);
                }
            }
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
        self.next_u64() % 2 == 0
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
