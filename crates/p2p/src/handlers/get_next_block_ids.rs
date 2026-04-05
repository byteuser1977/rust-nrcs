use crate::protocol::PeerRequest;
use serde_json;
use tracing::warn;

/// GetNextBlockIds 处理器（占位）
pub struct GetNextBlockIdsHandler {}

impl GetNextBlockIdsHandler {
    pub async fn handle(&self, _request: PeerRequest) -> serde_json::Value {
        warn!("GetNextBlockIds not implemented");
        serde_json::json!({ "error": "NOT_IMPLEMENTED" })
    }
}