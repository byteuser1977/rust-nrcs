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

/// 最大返回的里程碑区块ID数量
const MAX_MILESTONE_IDS: usize = 10;

/// 最大跳跃距离
const MAX_JUMP: u32 = 1440;

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

        // 获取本地区块链高度
        let blockchain_height = match block_repo.get_height().await {
            Ok(h) => h,
            Err(e) => {
                warn!("Failed to get blockchain height: {}", e);
                return serde_json::json!({ "error": "DATABASE_ERROR" });
            }
        };

        if blockchain_height == 0 {
            return serde_json::json!({ "milestoneBlockIds": [] });
        }

        // 获取本地最后一个区块
        let my_last_block = match block_repo.find_latest().await {
            Ok(Some(b)) => b,
            _ => return serde_json::json!({ "milestoneBlockIds": [] }),
        };

        // 处理 lastBlockId 参数
        if let Some(ref last_block_id_str) = last_block_id_str {
            match self.parse_block_id(last_block_id_str) {
                Ok(last_block_id) => {
                    // 检查是否是本地的最后一个区块
                    if my_last_block.id == last_block_id {
                        let mut response = serde_json::Map::new();
                        response.insert("milestoneBlockIds".to_string(),
                            serde_json::json!([last_block_id_str.clone()])
                        );
                        response.insert("last".to_string(), serde_json::Value::Bool(true));
                        return serde_json::Value::Object(response);
                    }

                    // 检查本地是否有这个区块
                    if let Ok(true) = block_repo.has_block(last_block_id).await {
                        let mut response = serde_json::Map::new();
                        response.insert("milestoneBlockIds".to_string(),
                            serde_json::json!([last_block_id_str.clone()])
                        );
                        return serde_json::Value::Object(response);
                    }
                }
                Err(e) => {
                    warn!("Invalid lastBlockId: {}", e);
                }
            }
        }

        // 计算起始高度和跳跃距离
        let (start_height, jump): (i32, i32) = if let Some(ref last_milestone_str) = last_milestone_block_id_str {
            match self.parse_block_id(last_milestone_str) {
                Ok(last_milestone_id) => {
                    match block_repo.find_by_id_column(last_milestone_id).await {
                        Ok(Some(block)) => {
                            let height = block.height;
                            // 与 Java 一致: jump = min(1440, max(blockchainHeight - height, 1))
                            let jump = std::cmp::min(MAX_JUMP as i32, std::cmp::max(blockchain_height - height, 1));
                            // 新高度 = max(height - jump, 0)
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
            // 只有 lastBlockId，没有 lastMilestoneBlockId
            (blockchain_height, 10)
        } else {
            // 两个参数都没有，返回错误
            warn!("Old getMilestoneBlockIds request without lastBlockId");
            return serde_json::json!({
                "error": "Old getMilestoneBlockIds protocol not supported, please upgrade"
            });
        };

        // 收集里程碑区块ID
        let mut milestone_block_ids = Vec::new();
        let mut current_height = start_height;
        let mut count = 0;

        while current_height > 0 && count < MAX_MILESTONE_IDS {
            match block_repo.get_block_id_at_height(current_height).await {
                Ok(Some(block_id)) => {
                    milestone_block_ids.push(format!("{}", block_id));
                }
                _ => break,
            }
            current_height = std::cmp::max(current_height - jump, 0);
            count += 1;
        }

        // 如果高度为0，添加创世区块
        if current_height == 0 && count < MAX_MILESTONE_IDS {
            if let Ok(Some(block_id)) = block_repo.get_block_id_at_height(0).await {
                milestone_block_ids.push(format!("{}", block_id));
            }
        }

        serde_json::json!({
            "milestoneBlockIds": milestone_block_ids
        })
    }

    /// 解析区块ID（支持无符号长整型字符串）
    /// 对应 Java: Convert.parseUnsignedLong
    fn parse_block_id(&self, id_str: &str) -> Result<i64, String> {
        if id_str.starts_with("-") {
            return Err("Block ID cannot be negative".to_string());
        }
        
        id_str.parse::<u64>()
            .map(|v| v as i64)
            .map_err(|e| format!("Invalid block id: {}", e))
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
    fn test_parse_block_id_large() {
        let handler = GetMilestoneBlockIdsHandler::new();
        // 测试大数值（无符号长整型）
        let result = handler.parse_block_id("18446744073709551615"); // u64::MAX
        assert!(result.is_ok());
    }
}
