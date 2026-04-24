//! GetMilestoneBlockIds Handler
//!
//! 对应 Java: GetMilestoneBlockIds.java
//!
//! 返回里程碑区块ID列表，用于区块链同步时找到共同区块

use crate::protocol::PeerRequest;
use serde_json;
use tracing::{debug, warn};
use orm::BlockRepository;
use std::sync::Arc;

pub struct GetMilestoneBlockIdsHandler {
    block_repo: Option<Arc<dyn BlockRepository>>,
}

impl GetMilestoneBlockIdsHandler {
    pub fn new() -> Self {
        Self { block_repo: None }
    }

    pub fn with_block_repo(block_repo: Arc<dyn BlockRepository>) -> Self {
        Self { block_repo: Some(block_repo) }
    }

    pub async fn handle(&self, request: PeerRequest) -> serde_json::Value {
        debug!("Handling GetMilestoneBlockIds request");

        if let Some(ref block_repo) = self.block_repo {
            self.handle_with_repo(request, block_repo).await
        } else {
            warn!("GetMilestoneBlockIds: block repository not configured");
            serde_json::json!({ "error": "NOT_CONFIGURED" })
        }
    }

    async fn handle_with_repo(&self, request: PeerRequest, block_repo: &Arc<dyn BlockRepository>) -> serde_json::Value {
        let last_block_id_str: Option<String> = request.get("lastBlockId");
        let last_milestone_block_id_str: Option<String> = request.get("lastMilestoneBlockId");

        let mut milestone_block_ids = Vec::new();
        let mut is_last = false;

        match block_repo.get_height().await {
            Ok(blockchain_height) => {
                if blockchain_height == 0 {
                    return serde_json::json!({ "milestoneBlockIds": [] });
                }

                if let Some(ref last_block_id_str) = last_block_id_str {
                    match self.parse_block_id(&last_block_id_str) {
                        Ok(last_block_id) => {
                            match block_repo.find_by_id_column(last_block_id).await {
                                Ok(Some(_block)) => {
                                    let my_last_block = block_repo.find_latest().await;
                                    if let Ok(Some(my_last)) = my_last_block {
                                        if my_last.id == last_block_id {
                                            milestone_block_ids.push(last_block_id_str.clone());
                                            is_last = true;
                                        } else if block_repo.has_block(last_block_id).await.unwrap_or(false) {
                                            milestone_block_ids.push(last_block_id_str.clone());
                                        }
                                    }
                                    if !milestone_block_ids.is_empty() {
                                        let mut response = serde_json::Map::new();
                                        response.insert("milestoneBlockIds".to_string(), 
                                            serde_json::Value::Array(
                                                milestone_block_ids.iter()
                                                    .map(|id| serde_json::Value::String(id.clone()))
                                                    .collect()
                                            )
                                        );
                                        if is_last {
                                            response.insert("last".to_string(), serde_json::Value::Bool(true));
                                        }
                                        return serde_json::Value::Object(response);
                                    }
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            warn!("Invalid lastBlockId: {}", e);
                        }
                    }
                }

                let (height, jump) = if let Some(last_milestone_str) = last_milestone_block_id_str {
                    match self.parse_block_id(&last_milestone_str) {
                        Ok(last_milestone_id) => {
                            match block_repo.find_by_id_column(last_milestone_id).await {
                                Ok(Some(block)) => {
                                    let height = block.height;
                                    let jump = std::cmp::min(1440, std::cmp::max(blockchain_height - height, 1));
                                    let new_height = std::cmp::max(height - jump, 0);
                                    (new_height, jump)
                                }
                                _ => {
                                    return serde_json::json!({ "error": "Don't have block" });
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Invalid lastMilestoneBlockId: {}", e);
                            return serde_json::json!({ "error": "INVALID_BLOCK_ID" });
                        }
                    }
                } else if last_block_id_str.is_some() {
                    (blockchain_height, 10)
                } else {
                    warn!("Old getMilestoneBlockIds request without lastBlockId");
                    return serde_json::json!({ 
                        "error": "Old getMilestoneBlockIds protocol not supported, please upgrade" 
                    });
                };

                let limit = 10;
                let mut current_height = height;
                let mut count = 0;

                while current_height > 0 && count < limit {
                    match block_repo.get_block_id_at_height(current_height).await {
                        Ok(Some(block_id)) => {
                            milestone_block_ids.push(format!("{}", block_id));
                        }
                        _ => break,
                    }
                    current_height = current_height.saturating_sub(jump);
                    count += 1;
                }

                let mut response = serde_json::Map::new();
                response.insert("milestoneBlockIds".to_string(), 
                    serde_json::Value::Array(
                        milestone_block_ids.into_iter()
                            .map(serde_json::Value::String)
                            .collect()
                    )
                );

                serde_json::Value::Object(response)
            }
            Err(e) => {
                warn!("Failed to get blockchain height: {}", e);
                serde_json::json!({ "error": "DATABASE_ERROR" })
            }
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

impl Default for GetMilestoneBlockIdsHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_creation() {
        let handler = GetMilestoneBlockIdsHandler::new();
        assert!(handler.block_repo.is_none());
    }

    #[test]
    fn test_parse_block_id_decimal() {
        let handler = GetMilestoneBlockIdsHandler::new();
        let result = handler.parse_block_id("123456789");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 123456789);
    }

    #[test]
    fn test_parse_block_id_hex() {
        let handler = GetMilestoneBlockIdsHandler::new();
        let result = handler.parse_block_id("0x75BCD15");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 123456789);
    }
}
