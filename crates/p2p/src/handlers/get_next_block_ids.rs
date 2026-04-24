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
        let limit: i32 = request.get("limit").unwrap_or(1440);

        let limit = limit.min(1440).max(1);

        if let Some(block_id_str) = block_id_str {
            match self.parse_block_id(&block_id_str) {
                Ok(block_id) => {
                    match block_repo.get_ids_after(block_id, limit).await {
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

    fn parse_block_id(&self, id_str: &str) -> Result<i64, String> {
        if id_str.starts_with("0x") || id_str.starts_with("0X") {
            i64::from_str_radix(&id_str[2..], 16)
                .map_err(|e| format!("Invalid hex block id: {}", e))
        } else {
            id_str.parse::<i64>()
                .or_else(|_| u64::from_str_radix(id_str, 10).map(|v| v as i64))
                .map_err(|e| format!("Invalid block id: {}", e))
        }
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
        assert_eq!(handler.parse_block_id("0x7B").unwrap(), 123);
    }
}
