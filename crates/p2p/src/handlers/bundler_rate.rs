use crate::protocol::PeerRequest;
use serde_json;
use tracing::warn;

/// BundlerRate 处理器（占位）
pub struct BundlerRateHandler {}

impl BundlerRateHandler {
    pub async fn handle(&self, _request: PeerRequest) -> serde_json::Value {
        warn!("BundlerRate not implemented");
        serde_json::json!({ "error": "NOT_IMPLEMENTED" })
    }
}