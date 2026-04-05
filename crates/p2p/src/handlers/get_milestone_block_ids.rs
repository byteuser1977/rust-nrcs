use crate::protocol::PeerRequest;
use serde_json;
use tracing::warn;

/// GetMilestoneBlockIds 处理器（占位）
pub struct GetMilestoneBlockIdsHandler {}

impl GetMilestoneBlockIdsHandler {
    pub async fn handle(&self, _request: PeerRequest) -> serde_json::Value {
        warn!("GetMilestoneBlockIds not implemented");
        serde_json::json!({ "error": "NOT_IMPLEMENTED" })
    }
}