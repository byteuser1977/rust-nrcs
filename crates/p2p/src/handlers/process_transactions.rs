use crate::{peer::Peers, protocol::PeerRequest};
use serde_json;
use std::sync::Arc;
use tracing::{debug, warn};

/// ProcessTransactions 处理器
/// 请求：transactions 数组（每个交易 JSON）
/// 处理：验证交易并广播到内存池
/// 响应：空 JSON {} 或错误
pub struct ProcessTransactionsHandler {
    peers: Arc<Peers>,
}

impl ProcessTransactionsHandler {
    pub fn new(peers: Arc<Peers>) -> Self {
        Self { peers }
    }

    pub async fn handle(&self, request: PeerRequest, _peers: Arc<Peers>) -> serde_json::Value {
        debug!("Handling ProcessTransactions request");

        let transactions: Vec<serde_json::Value> = match request.get("transactions") {
            Some(txs) => txs,
            None => {
                warn!("ProcessTransactions missing 'transactions' field");
                return serde_json::json!({ "error": "MISSING_TRANSACTIONS" });
            }
        };

        debug!("Received {} transactions", transactions.len());

        // TODO: 验证每笔交易
        // 1. 签名验证
        // 2. 双花检查
        // 3. 余额检查
        // 4. 手续费检查

        // TODO: 添加到内存池并广播

        warn!("ProcessTransactions not fully implemented");

        // 成功返回空对象（与 Java 一致）
        serde_json::json!({})
    }
}