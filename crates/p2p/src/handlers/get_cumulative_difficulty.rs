use crate::protocol::PeerRequest;
use serde_json;
use tracing::warn;

/// GetCumulativeDifficulty 处理器（占位）
pub struct GetCumulativeDifficultyHandler {}

impl GetCumulativeDifficultyHandler {
    pub async fn handle(&self, _request: PeerRequest) -> serde_json::Value {
        warn!("GetCumulativeDifficulty not implemented");
        serde_json::json!({ "error": "NOT_IMPLEMENTED" })
    }
}