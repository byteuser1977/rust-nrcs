use crate::protocol::PeerRequest;
use serde_json;
use tracing::warn;

/// 未知请求处理器
pub struct UnknownHandler {}

impl UnknownHandler {
    pub fn handle(&self, request_type: &str) -> serde_json::Value {
        warn!("Unknown request type: {}", request_type);
        serde_json::json!({ "error": "NOT_IMPLEMENTED" })
    }
}