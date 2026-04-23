//! P2P Manager (P2P 管理器)
//!
//! 对应 NRCS Java: Peers.java 单例
//!
//! 职责:
//! - 管理所有节点
//! - 管理守护进程
//! - 提供统一的 P2P 接口

use crate::config::P2PConfig;
use crate::daemon::{ConnectionDaemon, DiscoveryDaemon, TransactionDaemon, UnblacklistDaemon};
use crate::error::{P2PError, P2PResult};
use crate::peer::{Peer, PeerState, Peers};
use crate::protocol::PeerRequest;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// P2P Manager
/// 
/// 对应 NRCS Java: Peers.java
pub struct P2PManager {
    /// 配置
    config: P2PConfig,
    /// 节点管理器
    peers: Arc<Peers>,
    /// 连接守护进程
    connection_daemon: Option<ConnectionDaemon>,
    /// 发现守护进程
    discovery_daemon: Option<DiscoveryDaemon>,
    /// 黑名单守护进程
    unblacklist_daemon: Option<UnblacklistDaemon>,
    /// 交易守护进程
    transaction_daemon: Option<TransactionDaemon>,
    /// 运行状态
    running: Arc<RwLock<bool>>,
}

impl P2PManager {
    /// Create a new P2P manager
    pub fn new(config: P2PConfig) -> Self {
        let my_peer = Peer::new(config.listen_addr, false);
        let peers = Arc::new(Peers::new(my_peer));

        Self {
            config,
            peers,
            connection_daemon: None,
            discovery_daemon: None,
            unblacklist_daemon: None,
            transaction_daemon: None,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize P2P manager
    /// 
    /// 对应 NRCS Java: Peers.init()
    pub async fn init(&mut self) -> P2PResult<()> {
        info!("Initializing P2P manager...");

        // 验证配置
        self.config.validate().map_err(|e| P2PError::internal(e))?;

        // 创建守护进程
        self.connection_daemon = Some(ConnectionDaemon::new(
            Arc::clone(&self.peers),
            self.config.clone(),
        ));

        self.discovery_daemon = Some(DiscoveryDaemon::new(
            Arc::clone(&self.peers),
            self.config.clone(),
        ));

        self.unblacklist_daemon = Some(UnblacklistDaemon::new(
            Arc::clone(&self.peers),
            self.config.clone(),
        ));

        self.transaction_daemon = Some(TransactionDaemon::new(
            Arc::clone(&self.peers),
            self.config.clone(),
        ));

        info!("P2P manager initialized");
        Ok(())
    }

    /// Start P2P manager
    pub async fn start(&mut self) -> P2PResult<()> {
        let mut running = self.running.write().await;
        if *running {
            warn!("P2P manager is already running");
            return Ok(());
        }
        *running = true;
        drop(running);

        info!("Starting P2P manager...");

        // 启动守护进程
        if let Some(ref daemon) = self.connection_daemon {
            daemon.start().await;
        }

        if let Some(ref daemon) = self.discovery_daemon {
            daemon.start().await;
        }

        if let Some(ref daemon) = self.unblacklist_daemon {
            daemon.start().await;
        }

        if let Some(ref daemon) = self.transaction_daemon {
            daemon.start().await;
        }

        info!("P2P manager started");
        Ok(())
    }

    /// Stop P2P manager
    pub async fn stop(&mut self) {
        let mut running = self.running.write().await;
        *running = false;
        drop(running);

        info!("Stopping P2P manager...");

        // 停止守护进程
        if let Some(ref daemon) = self.connection_daemon {
            daemon.stop().await;
        }

        if let Some(ref daemon) = self.discovery_daemon {
            daemon.stop().await;
        }

        if let Some(ref daemon) = self.unblacklist_daemon {
            daemon.stop().await;
        }

        if let Some(ref daemon) = self.transaction_daemon {
            daemon.stop().await;
        }

        info!("P2P manager stopped");
    }

    /// Get peers manager
    pub fn peers(&self) -> Arc<Peers> {
        Arc::clone(&self.peers)
    }

    /// Get config
    pub fn config(&self) -> &P2PConfig {
        &self.config
    }

    /// Add a peer
    /// 
    /// 对应 NRCS Java: Peers.addPeer(Peer peer)
    pub async fn add_peer(&self, peer: Peer) -> bool {
        self.peers.register_peer(peer).await;
        true
    }

    /// Remove a peer
    /// 
    /// 对应 NRCS Java: Peers.removePeer(Peer peer)
    pub async fn remove_peer(&self, addr: &std::net::SocketAddr) {
        self.peers.remove_peer(addr).await;
    }

    /// Get all peers
    /// 
    /// 对应 NRCS Java: Peers.getPeers(Predicate)
    pub async fn get_peers(&self) -> Vec<Peer> {
        self.peers.get_known_peers().await
    }

    /// Get active peers
    pub async fn get_active_peers(&self) -> Vec<Peer> {
        self.peers.get_active_peers().await
    }

    /// Get any peer with specified state
    /// 
    /// 对应 NRCS Java: Peers.getAnyPeer(PeerState state, boolean applyHallmark)
    pub async fn get_any_peer(&self, state: PeerState, prefer_hallmarked: bool) -> Option<Peer> {
        self.peers.get_any_peer(state, prefer_hallmarked).await
    }

    /// Blacklist a peer
    /// 
    /// 对应 NRCS Java: Peers.blacklist(Peer peer, String cause)
    pub async fn blacklist_peer(&self, addr: &std::net::SocketAddr, cause: String) {
        if let Some(peer) = self.peers.get_peer(addr).await {
            let mut peer = peer.lock().await;
            peer.blacklist(cause);
        }
    }

    /// Check if peer is blacklisted
    pub async fn is_blacklisted(&self, addr: &str) -> bool {
        self.peers.is_blacklisted(addr).await
    }

    /// Broadcast transaction
    /// 
    /// 对应 NRCS Java: Peers.broadcast(Transaction transaction)
    pub async fn broadcast_transaction(&self, _transaction: &[u8]) -> P2PResult<()> {
        // TODO: 实现交易广播
        debug!("Broadcasting transaction");
        Ok(())
    }

    /// Send request to some peers
    /// 
    /// 对应 NRCS Java: Peers.sendToSomePeers(JSONObject request)
    pub async fn send_to_some_peers(&self, _request: &PeerRequest) -> P2PResult<()> {
        // TODO: 实现请求发送
        debug!("Sending request to some peers");
        Ok(())
    }

    /// Get my peer info
    /// 
    /// 对应 NRCS Java: Peers.getMyPeerInfo()
    pub async fn get_my_peer_info(&self) -> serde_json::Value {
        self.peers.get_my_peer_info().await
    }

    /// Get connected peers count
    pub async fn connected_peers_count(&self) -> usize {
        let peers = self.peers.get_active_peers().await;
        peers.iter().filter(|p| p.state == PeerState::Connected).count()
    }

    /// Get known peers count
    pub async fn known_peers_count(&self) -> usize {
        self.peers.known_peers_count().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_p2p_manager_creation() {
        let config = P2PConfig::default();
        let manager = P2PManager::new(config);
        // known_peers_count 只计算已知的外部节点，不包括自己节点
        // 自己节点信息存储在 my_peer_info 中
        assert_eq!(manager.known_peers_count().await, 0);
    }

    #[tokio::test]
    async fn test_p2p_manager_init() {
        let config = P2PConfig::default();
        let mut manager = P2PManager::new(config);
        assert!(manager.init().await.is_ok());
    }
}
