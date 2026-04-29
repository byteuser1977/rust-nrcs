//! Transaction Daemon (交易守护进程)
//!
//! 对应 NRCS Java: sendTransactionsThread (30秒间隔)
//!
//! 职责:
//! - 批量发送交易
//! - 广播交易到其他节点
//! - 从内存池获取未确认交易

use crate::config::P2PConfig;
use crate::peer::{PeerState, Peers};
use crate::protocol::{PeerRequest, RequestType};
use serde_json::{self, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Transaction Daemon
///
/// 对应 NRCS Java: sendTransactionsThread
pub struct TransactionDaemon {
    peers: Arc<Peers>,
    config: P2PConfig,
    running: Arc<RwLock<bool>>,
}

impl TransactionDaemon {
    /// Create a new transaction daemon
    pub fn new(peers: Arc<Peers>, config: P2PConfig) -> Self {
        Self {
            peers,
            config,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the transaction daemon
    pub async fn start(&self) {
        let mut running = self.running.write().await;
        if *running {
            warn!("Transaction daemon is already running");
            return;
        }
        *running = true;
        drop(running);

        info!("[Transaction] Daemon started (interval: {}s)",
              self.config.transaction_daemon_interval_secs);

        let peers = Arc::clone(&self.peers);
        let config = self.config.clone();
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            loop {
                if !*running.read().await {
                    break;
                }

                tokio::time::sleep(Duration::from_secs(config.transaction_daemon_interval_secs)).await;

                if let Err(e) = Self::transaction_loop(&peers, &config).await {
                    warn!("[Transaction] Loop error: {}", e);
                }
            }
            info!("[Transaction] Daemon stopped");
        });
    }

    /// Stop the transaction daemon
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("[Transaction] Daemon stopping...");
    }

    /// Main transaction loop
    ///
    /// 对应 NRCS Java: sendTransactionsThread.run()
    async fn transaction_loop(peers: &Arc<Peers>, config: &P2PConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 1. 获取未发送的交易（从内存池）
        let pending_transactions = Self::get_pending_transactions().await;

        if pending_transactions.is_empty() {
            debug!("[Transaction] No pending transactions to broadcast");
            return Ok(());
        }

        // 2. 限制批量大小（sendTransactionsBatchSize，默认 10）
        let batch_size = config.send_transactions_batch_size.min(pending_transactions.len());
        let batch: Vec<&Value> = pending_transactions.iter().take(batch_size).collect();

        debug!("[Transaction] Broadcasting {} transactions (batch size: {})",
               batch.len(), batch_size);

        // 3. 广播交易到其他节点
        Self::broadcast_transactions(peers, config, &batch).await?;

        Ok(())
    }

    /// Get pending transactions from mempool
    ///
    /// 对应 NRCS Java: 从 TransactionProcessor.getUnconfirmedTransactions() 获取
    ///
    /// 注意：当前实现返回空列表，实际集成时需要从 tx-engine 的内存池获取
    async fn get_pending_transactions() -> Vec<Value> {
        // TODO: 实际实现应该从以下来源获取：
        // 1. tx-engine 模块的内存池 (MemPool)
        // 2. 数据库中未确认的交易
        // 3. 本地创建但未广播的交易

        // 当前返回空列表作为占位符
        Vec::new()
    }

    /// Broadcast transactions to peers（完整实现）
    ///
    /// 对应 NRCS Java: 发送 processTransactions 请求
    async fn broadcast_transactions(
        peers: &Arc<Peers>,
        config: &P2PConfig,
        transactions: &[&Value],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::websocket::WebsocketClient;

        if transactions.is_empty() {
            return Ok(());
        }

        // 获取连接的节点
        let active_peers = peers.get_public_peers(PeerState::Connected).await;

        if active_peers.is_empty() {
            debug!("[Transaction] No active peers for broadcasting");
            return Ok(());
        }

        // 构建 processTransactions 请求
        let tx_jsons: Vec<Value> = transactions.iter().map(|t| (*t).clone()).collect();
        let mut request = PeerRequest::new(RequestType::ProcessTransactions, 1);
        request.set("transactions", tx_jsons);

        // 发送到部分节点（使用 sendToPeersLimit）
        let send_limit = config.send_to_peers_limit.min(active_peers.len());
        let mut success_count = 0usize;

        for peer in active_peers.iter().take(send_limit) {
            let peer_addr = peer.address;
            let req_clone = request.clone();

            tokio::spawn(async move {
                match WebsocketClient::send_request(peer_addr, req_clone).await {
                    Ok(response) => {
                        // 检查响应中 accepted 的数量
                        if let Some(accepted) = response.get("accepted").and_then(|v| v.as_i64()) {
                            debug!("[Transaction] Peer {} accepted {} transactions",
                                   peer_addr, accepted);
                        } else {
                            debug!("[Transaction] Successfully sent to {}", peer_addr);
                        }
                    }
                    Err(e) => {
                        debug!("[Transaction] Failed to send to {}: {}", peer_addr, e);
                    }
                }
            });

            success_count += 1;
        }

        info!("[Transaction] Broadcasted {} transactions to {} peers",
              transactions.len(), success_count);

        Ok(())
    }

    /// 手动触发交易广播（用于新交易入池时立即广播）
    pub async fn force_broadcast(
        &self,
        transactions: &[Value],
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        if transactions.is_empty() {
            return Ok(0);
        }

        let refs: Vec<&Value> = transactions.iter().collect();
        Self::broadcast_transactions(&self.peers, &self.config, &refs).await?;

        info!("[Transaction] Force broadcasted {} transactions", transactions.len());

        Ok(transactions.len())
    }
}

/// Get current timestamp in seconds
#[allow(dead_code)]
fn current_timestamp() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer::Peer;

    #[tokio::test]
    async fn test_transaction_daemon_creation() {
        let addr: std::net::SocketAddr = "127.0.0.1:17974".parse().unwrap();
        let my_peer = Peer::new(addr, false);
        let peers = Arc::new(Peers::new(my_peer));
        let config = P2PConfig::default();

        let daemon = TransactionDaemon::new(Arc::clone(&peers), config);
        assert!(!*daemon.running.read().await);
    }

    #[tokio::test]
    async fn test_transaction_daemon_start_stop() {
        let addr: std::net::SocketAddr = "127.0.0.1:17974".parse().unwrap();
        let my_peer = Peer::new(addr, false);
        let peers = Arc::new(Peers::new(my_peer));
        let config = P2PConfig::default();

        let daemon = TransactionDaemon::new(Arc::clone(&peers), config);

        daemon.start().await;
        assert!(*daemon.running.read().await);

        daemon.stop().await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[test]
    fn test_get_pending_transactions_empty() {
        // 测试默认情况下返回空列表
        let rt = tokio::runtime::Runtime::new().unwrap();
        let pending = rt.block_on(TransactionDaemon::get_pending_transactions());
        assert!(pending.is_empty());
    }

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts > 1700000000); // 2023 年以后的时间戳
    }
}
