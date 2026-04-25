//! GetNextBlockIds Handler
//!
//! 对应 Java: GetNextBlockIds.java
//!
//! 返回指定区块之后的区块ID列表

use crate::protocol::PeerRequest;
use serde_json;
use tracing::{debug, warn};
use orm::BlockRepository;
use std::sync::Arc;

/// 最大区块ID数量限制（与 Java 一致）
const MAX_BLOCK_IDS: i32 = 1440;

pub struct GetNextBlockIdsHandler {
    block_repo: Option<Arc<dyn BlockRepository>>,
}

impl GetNextBlockIdsHandler {
    pub fn new() -> Self {
        Self { block_repo: None }
    }

    pub fn with_block_repo(block_repo: Arc<dyn BlockRepository>) -> Self {
        Self { block_repo: Some(block_repo) }
    }

    pub async fn handle(&self, request: PeerRequest) -> serde_json::Value {
        debug!("Handling GetNextBlockIds request");

        if let Some(ref block_repo) = self.block_repo {
            self.handle_with_repo(request, block_repo).await
        } else {
            warn!("GetNextBlockIds: block repository not configured");
            serde_json::json!({ "nextBlockIds": [] })
        }
    }

    async fn handle_with_repo(&self, request: PeerRequest, block_repo: &Arc<dyn BlockRepository>) -> serde_json::Value {
        let block_id_str: Option<String> = request.get("blockId");
        let limit: i64 = request.get("limit").unwrap_or(0);

        // 与 Java 一致：如果 limit > 1440，返回错误
        if limit > MAX_BLOCK_IDS as i64 {
            return serde_json::json!({
                "error": "TOO_MANY_BLOCKS_REQUESTED",
                "errorDescription": "Maximum 1440 blocks can be requested at a time"
            });
        }

        // 与 Java 一致：limit <= 0 时使用默认值 1440
        let effective_limit = if limit <= 0 { MAX_BLOCK_IDS } else { limit as i32 };

        if let Some(block_id_str) = block_id_str {
            match self.parse_block_id(&block_id_str) {
                Ok(block_id) => {
                    match block_repo.get_ids_after(block_id, effective_limit).await {
                        Ok(ids) => {
                            debug!("Returning {} block ids after {}", ids.len(), block_id);
                            serde_json::json!({
                                "nextBlockIds": ids.iter()
                                    .map(|id| id.to_string())
                                    .collect::<Vec<String>>()
                            })
                        }
                        Err(e) => {
                            warn!("Failed to get block ids: {}", e);
                            serde_json::json!({ "nextBlockIds": [] })
                        }
                    }
                }
                Err(e) => {
                    warn!("Invalid blockId: {}", e);
                    serde_json::json!({ "error": "INVALID_BLOCK_ID" })
                }
            }
        } else {
            warn!("GetNextBlockIds missing blockId");
            serde_json::json!({ "error": "MISSING_BLOCK_ID" })
        }
    }

    /// 解析区块ID（支持无符号长整型字符串）
    /// 对应 Java: Convert.parseUnsignedLong
    fn parse_block_id(&self, id_str: &str) -> Result<i64, String> {
        // Java 使用 Long.parseUnsignedLong，支持大于 i64::MAX 的值
        // Rust 中我们需要处理这种情况
        if id_str.starts_with("-") {
            return Err("Block ID cannot be negative".to_string());
        }
        
        // 尝试解析为 u64，然后转换为 i64（保留位模式）
        id_str.parse::<u64>()
            .map(|v| v as i64)
            .map_err(|e| format!("Invalid block id: {}", e))
    }
}

impl Default for GetNextBlockIdsHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_creation() {
        let handler = GetNextBlockIdsHandler::new();
        assert!(handler.block_repo.is_none());
    }

    #[test]
    fn test_parse_block_id() {
        let handler = GetNextBlockIdsHandler::new();
        assert_eq!(handler.parse_block_id("123").unwrap(), 123);
        // 测试大数值（无符号长整型）
        assert!(handler.parse_block_id("18446744073709551615").is_ok()); // u64::MAX
        assert!(handler.parse_block_id("-1").is_err()); // 负数应该失败
    }

    #[test]
    fn test_limit_validation() {
        // limit > 1440 应该返回错误
        // limit <= 0 应该使用默认值 1440
    }
}
