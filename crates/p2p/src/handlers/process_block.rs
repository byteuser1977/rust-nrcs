//! ProcessBlock Handler
//!
//! 对应 Java: ProcessBlock.java
//!
//! 处理接收到的区块，验证并添加到区块链

use crate::{peer::Peers, protocol::PeerRequest};
use serde_json;
use std::sync::Arc;
use tracing::{debug, info, warn};
use super::BlockVerifier;
use blockchain_types::prelude::*;

/// ProcessBlock 处理器
/// 请求：完整的区块 JSON（同 Block.getJSONObject() 结构）
/// 处理：验证区块，然后异步提交到区块链处理器
/// 响应：空 JSON {}
pub struct ProcessBlockHandler {
    #[allow(dead_code)]
    peers: Arc<Peers>,
    verifier: Arc<dyn BlockVerifier>,
}

impl ProcessBlockHandler {
    pub fn new(peers: Arc<Peers>, verifier: Arc<dyn BlockVerifier>) -> Self {
        Self { peers, verifier }
    }

    pub async fn handle(&self, request: PeerRequest, _peers: Arc<Peers>) -> serde_json::Value {
        debug!("Handling ProcessBlock request");

        // 获取区块数据（可能在 "block" 字段中，或者直接在请求根级别）
        let block_json = match request.get::<serde_json::Value>("block") {
            Some(block) => block,
            None => {
                // 尝试直接从请求根级别获取区块字段
                let block_json = request.get::<serde_json::Value>("block");
                match block_json {
                    Some(b) => b,
                    None => {
                        // 如果没有 "block" 字段，检查是否有区块的其他字段
                        if request.get::<String>("version").is_some() {
                            // 整个请求就是区块数据
                            serde_json::to_value(&request).unwrap_or(serde_json::Value::Null)
                        } else {
                            warn!("ProcessBlock request missing 'block' field");
                            return serde_json::json!({ "error": "MISSING_BLOCK" });
                        }
                    }
                }
            }
        };

        // 解析区块
        let block: Block = match serde_json::from_value(block_json.clone()) {
            Ok(b) => b,
            Err(e) => {
                warn!("Failed to deserialize block: {}", e);
                return serde_json::json!({ "error": "INVALID_BLOCK_JSON", "details": e.to_string() });
            }
        };

        // 获取 previousBlock 字段（用于验证）
        let _previous_block_id_str: Option<String> = request.get("previousBlock")
            .or_else(|| block_json.get("previousBlock").and_then(|v| v.as_str().map(|s| s.to_string())));

        debug!("Received block: height={}, generator={:?}", block.height, block.generator_id);

        // 验证并处理区块
        match self.verifier.verify_and_process(block).await {
            Ok(_) => {
                info!("Block verified and processed successfully");
                serde_json::json!({})
            }
            Err(e) => {
                warn!("Block verification/processing failed: {}", e);
                serde_json::json!({ "error": "BLOCK_REJECTED", "reason": e.to_string() })
            }
        }
    }
}

/// 解析无符号长整型区块ID
#[allow(dead_code)]
fn parse_unsigned_long(s: &str) -> Option<i64> {
    s.parse::<u64>().ok().map(|v| v as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_unsigned_long() {
        assert_eq!(parse_unsigned_long("123"), Some(123));
        assert_eq!(parse_unsigned_long("18446744073709551615"), Some(-1)); // u64::MAX as i64
        assert_eq!(parse_unsigned_long("-1"), None);
        assert_eq!(parse_unsigned_long("invalid"), None);
    }
}
