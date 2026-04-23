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
    running: Arc<RwLock<bool>>,
}

impl DiscoveryDaemon {
    /// Create a new discovery daemon
    pub fn new(peers: Arc<Peers>, config: P2PConfig) -> Self {
        Self {
            peers,
            config,
            running: Arc::new(RwLock::new(false)),
        }
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

        tokio::spawn(async move {
            loop {
                if !*running.read().await {
                    break;
                }

                tokio::time::sleep(Duration::from_secs(config.discovery_daemon_interval_secs)).await;
                
                if let Err(e) = Self::discovery_loop(&peers, &config).await {
                    warn!("Discovery loop error: {}", e);
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

    /// Request peers from a connected peer
    /// 
    /// 对应 NRCS Java: getMorePeersThread 中的 getPeers 请求
    async fn request_peers_from_peer(
        _peers: &Arc<Peers>,
        peer: &Peer,
        _config: &P2PConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 构建 getPeers 请求
        let _request = PeerRequest::new(RequestType::GetPeers, 1);

        // TODO: 发送请求并处理响应
        debug!("Requesting peers from: {}", peer.address);

        Ok(())
    }

    /// Share my peers with other peers
    /// 
    /// 对应 NRCS Java: getMorePeersThread 中的 addPeers 请求
    async fn share_my_peers(
        peers: &Arc<Peers>,
        _config: &P2PConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let all_peers = peers.get_known_peers().await;
        
        // 过滤可分享的节点（不能在 filter 中使用 await）
        let mut shareable_peers = Vec::new();
        for peer in all_peers {
            if peers.is_blacklisted_addr(&peer.address).await {
                continue;
            }
            if peer.announced_address.is_none() {
                continue;
            }
            if peer.state != PeerState::Connected {
                continue;
            }
            if !peer.share_address {
                continue;
            }
            shareable_peers.push(peer);
        }

        if shareable_peers.is_empty() {
            return Ok(());
        }

        // 构建 addPeers 请求
        let _request = PeerRequest::new(RequestType::AddPeers, 1);

        // TODO: 发送请求
        debug!("Sharing {} peers with other peers", shareable_peers.len());

        Ok(())
    }

    /// Update saved peers to database
    /// 
    /// 对应 NRCS Java: updateSavedPeers()
    async fn update_saved_peers(_peers: &Arc<Peers>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: 实现节点持久化
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_daemon_creation() {
        // TODO: 添加测试
    }
}
