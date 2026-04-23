//! Transaction Daemon (交易守护进程)
//!
//! 对应 NRCS Java: sendTransactionsThread (30秒间隔)
//!
//! 职责:
//! - 批量发送交易
//! - 广播交易到其他节点

use crate::config::P2PConfig;
use crate::peer::Peers;
use crate::protocol::{PeerRequest, RequestType};
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

        info!("Transaction daemon started (interval: {}s)", self.config.transaction_daemon_interval_secs);
        
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
                    warn!("Transaction loop error: {}", e);
                }
            }
            info!("Transaction daemon stopped");
        });
    }

    /// Stop the transaction daemon
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Transaction daemon stopping...");
    }

    /// Main transaction loop
    /// 
    /// 对应 NRCS Java: sendTransactionsThread.run()
    async fn transaction_loop(peers: &Arc<Peers>, config: &P2PConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 1. 获取未发送的交易
        // TODO: 从交易池获取未发送的交易

        // 2. 批量发送交易
        // sendTransactionsBatchSize = 10
        let batch_size = config.send_transactions_batch_size;
        
        // 3. 广播交易到其他节点
        Self::broadcast_transactions(peers, config, batch_size).await?;

        Ok(())
    }

    /// Broadcast transactions to peers
    /// 
    /// 对应 NRCS Java: 发送 processTransactions 请求
    async fn broadcast_transactions(
        peers: &Arc<Peers>,
        config: &P2PConfig,
        _batch_size: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 获取连接的节点
        let active_peers = peers.get_active_peers().await;
        
        if active_peers.is_empty() {
            return Ok(());
        }

        // 构建 processTransactions 请求
        let _request = PeerRequest::new(RequestType::ProcessTransactions, 1);

        // 发送到部分节点
        let send_limit = config.send_to_peers_limit.min(active_peers.len());
        
        for peer in active_peers.iter().take(send_limit) {
            // TODO: 发送交易到节点
            debug!("Broadcasting transactions to peer: {}", peer.address);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_daemon_creation() {
        // TODO: 添加测试
    }
}
