//! P2P Manager (P2P 管理器)
//!
//! 对应 NRCS Java: Peers.java 单例
//!
//! 职责:
//! - 管理所有节点
//! - 管理守护进程
//! - 提供统一的 P2P 接口
//! - 整合连接池、广播、持久化等模块

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
    config: Arc<P2PConfig>,
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
        let config = Arc::new(config);
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
        self.config.validate().map_err(P2PError::internal)?;

        // 加载种子节点到内存
        self.config.init_bootstrap_peers(&self.peers).await;

        // 创建守护进程
        self.connection_daemon = Some(ConnectionDaemon::new(
            Arc::clone(&self.peers),
            (*self.config).clone(),
        ));

        self.discovery_daemon = Some(DiscoveryDaemon::new(
            Arc::clone(&self.peers),
            (*self.config).clone(),
        ));

        self.unblacklist_daemon = Some(UnblacklistDaemon::new(
            Arc::clone(&self.peers),
            (*self.config).clone(),
        ));

        self.transaction_daemon = Some(TransactionDaemon::new(
            Arc::clone(&self.peers),
            (*self.config).clone(),
        ));

        info!("P2P manager initialized with {} bootstrap peers",
              self.peers.known_peers_count().await);
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
    /// 如果 blacklisting_enabled 为 false，则不执行黑名单操作
    pub async fn blacklist_peer(&self, addr: &std::net::SocketAddr, cause: String) {
        if !self.config.blacklisting_enabled {
            debug!("Blacklisting disabled, skipping blacklist for {}: {}", addr, cause);
            return;
        }

        if let Some(peer) = self.peers.get_peer(addr).await {
            let mut peer = peer.lock().await;
            let display_cause = if self.config.hide_error_details {
                cause.split(':').next().unwrap_or(&cause).to_string()
            } else {
                cause
            };
            peer.blacklist(display_cause);
        }
    }

    /// Blacklist a peer with exception-based exclusion rules
    ///
    /// 对应 NRCS Java: Peer.blacklist(Exception cause)
    /// 排除以下异常类型不拉黑:
    /// - 数据库相关异常（防止从零加载区块链时误拉黑）
    /// - EOF 解析错误
    pub async fn blacklist_peer_with_exception(&self, addr: &std::net::SocketAddr, error_type: &str, error_msg: &str) {
        if !self.config.blacklisting_enabled {
            return;
        }

        // 排除不拉黑的异常类型（对应 Java 中的排除规则）
        if error_type == "NotCurrentlyValid" || error_type == "BlockOutOfOrder" {
            return;
        }
        if error_type == "Database" {
            return;
        }
        if error_type == "Parse" && error_msg.contains("END_OF_FILE") {
            return;
        }

        let cause = if self.config.hide_error_details {
            error_type.to_string()
        } else {
            format!("{}: {}", error_type, error_msg)
        };

        self.blacklist_peer(addr, cause).await;
    }

    /// Unblacklist a peer
    ///
    /// 对应 NRCS Java: Peer.unBlacklist()
    pub async fn unblacklist_peer(&self, addr: &std::net::SocketAddr) {
        if let Some(peer) = self.peers.get_peer(addr).await {
            let mut peer = peer.lock().await;
            peer.un_blacklist();
        }
    }

    /// Connect peer (unblacklist first, then connect)
    ///
    /// 对应 NRCS Java: Peers.connectPeer(Peer peer)
    pub async fn connect_peer(&self, addr: &std::net::SocketAddr) -> Result<serde_json::Value, crate::error::P2PError> {
        // 先解除黑名单（对应 Java: peer.unBlacklist()）
        self.unblacklist_peer(addr).await;

        if let Some(peer_ref) = self.peers.get_peer(addr).await {
            let mut peer = peer_ref.lock().await;
            peer.connect(&self.config).await
        } else {
            Err(crate::error::P2PError::internal("Peer not found".to_string()))
        }
    }

    /// Check if peer is blacklisted
    pub async fn is_blacklisted(&self, addr: &str) -> bool {
        self.peers.is_blacklisted(addr).await
    }

    /// Check if peer address is blacklisted
    pub async fn is_blacklisted_addr(&self, addr: &std::net::SocketAddr) -> bool {
        self.peers.is_blacklisted_addr(addr).await
    }

    /// Manual blacklist a peer (API call)
    ///
    /// 对应 NRCS Java: BlacklistPeer.processRequest()
    pub async fn manual_blacklist(&self, addr: &std::net::SocketAddr) {
        self.blacklist_peer(addr, "Manual blacklist".to_string()).await;
    }

    /// Manual unblacklist a peer (API call)
    pub async fn manual_unblacklist(&self, addr: &std::net::SocketAddr) {
        self.unblacklist_peer(addr).await;
    }

    /// Broadcast transaction
    ///
    /// 对应 NRCS Java: Peers.broadcast(Transaction transaction)
    pub async fn broadcast_transactions(&self, transactions_json: &[serde_json::Value]) -> P2PResult<()> {
        use crate::broadcast::BroadcastManager;

        if transactions_json.is_empty() {
            return Ok(());
        }

        let manager = BroadcastManager::new(Arc::clone(&self.config));
        manager.broadcast_transactions(transactions_json, &self.peers).await;
        debug!("Broadcasting {} transactions", transactions_json.len());
        Ok(())
    }

    /// Send request to some peers
    ///
    /// 对应 NRCS Java: Peers.sendToSomePeers(JSONObject request)
    pub async fn send_to_some_peers(&self, request: &PeerRequest) -> P2PResult<Vec<crate::broadcast::BroadcastResult>> {
        use crate::broadcast::BroadcastManager;

        let manager = BroadcastManager::new(Arc::clone(&self.config));
        let results = manager.send_to_some_peers(request, &self.peers).await;
        debug!("Sent request to {} peers", results.len());
        Ok(results)
    }

    /// Broadcast block
    ///
    /// 对应 NRCS Java: Peers.broadcastBlock(IBlock block)
    pub async fn broadcast_block(
        &self,
        block_json: &serde_json::Value,
        previous_block_id: u64,
        timestamp: i64,
    ) -> P2PResult<()> {
        use crate::broadcast::BroadcastManager;

        let manager = BroadcastManager::new(Arc::clone(&self.config));
        manager.broadcast_block(block_json, previous_block_id, timestamp, &self.peers).await;
        info!("Broadcasting block (prev={})", previous_block_id);
        Ok(())
    }

    /// Get my peer info (完整版，包含所有字段)
    ///
    /// 对应 NRCS Java: Peers.getMyPeerInfoResponse()
    pub async fn get_my_peer_info_full(&self) -> serde_json::Value {
        self.peers.get_my_peer_info_full(&self.config).await
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
