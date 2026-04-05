use crate::protocol::PeerRequest;
use serde_json;
use tracing::{debug, warn};

/// GetTransactions / GetUnconfirmedTransactions 处理器
/// 响应：transactions 数组
pub struct GetTransactionsHandler {}

impl GetTransactionsHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn handle(&self, _request: PeerRequest) -> serde_json::Value {
        debug!("Handling GetTransactions request");

        // TODO: 从内存池或数据库获取交易

        warn!("GetTransactions not implemented");
        serde_json::json!({ "transactions": [] })
    }
}