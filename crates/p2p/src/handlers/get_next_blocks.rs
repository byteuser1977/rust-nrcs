//! GetNextBlocks Handler
//!
//! 对应 Java: GetNextBlocks.java
//!
//! 返回指定区块之后的区块列表

use crate::{peer::Peers, protocol::PeerRequest};
use serde_json;
use std::sync::Arc;
use tracing::{debug, warn};
use orm::BlockRepository;

/// 最大区块数量限制（与 Java 一致）
const MAX_BLOCKS: usize = 36;

/// TOO_MANY_BLOCKS_REQUESTED 错误响应
fn too_many_blocks_error() -> serde_json::Value {
    serde_json::json!({
        "error": "TOO_MANY_BLOCKS_REQUESTED",
        "errorDescription": "Maximum 36 blocks can be requested at a time"
    })
}

pub struct GetNextBlocksHandler {
    peers: Arc<Peers>,
    block_repo: Option<Arc<dyn BlockRepository>>,
}

impl GetNextBlocksHandler {
    pub fn new(peers: Arc<Peers>) -> Self {
        Self { peers, block_repo: None }
    }

    pub fn with_block_repo(peers: Arc<Peers>, block_repo: Arc<dyn BlockRepository>) -> Self {
        Self { peers, block_repo: Some(block_repo) }
    }

    pub async fn handle(&self, request: PeerRequest, _peers: Arc<Peers>) -> serde_json::Value {
        debug!("Handling GetNextBlocks request");

        let block_ids_opt: Option<Vec<String>> = request.get("blockIds");
        let block_id_opt: Option<String> = request.get("blockId");
        let limit: i64 = request.get("limit").unwrap_or(0);

        if let Some(ref block_repo) = self.block_repo {
            // 处理 blockIds 参数（请求特定区块）
            if let Some(ids) = block_ids_opt {
                if ids.len() > MAX_BLOCKS {
                    return too_many_blocks_error();
                }
                return self.handle_block_ids_request(block_repo, ids).await;
            }

            // 处理 blockId + limit 参数（请求后续区块）
            if let Some(ref block_id) = block_id_opt {
                let effective_limit = if limit <= 0 { MAX_BLOCKS as i64 } else { limit };
                if effective_limit > MAX_BLOCKS as i64 {
                    return too_many_blocks_error();
                }
                return self.handle_block_id_request(block_repo, block_id, effective_limit as usize).await;
            }

            warn!("GetNextBlocks missing both blockId and blockIds");
            return serde_json::json!({ "error": "MISSING_BLOCK_ID" });
        }

        warn!("GetNextBlocks: block repository not configured");
        serde_json::json!({ "nextBlocks": [] })
    }

    async fn handle_block_ids_request(&self, block_repo: &Arc<dyn BlockRepository>, ids: Vec<String>) -> serde_json::Value {
        debug!("Requested {} specific blocks", ids.len());
        let mut blocks = Vec::new();

        for id_str in ids {
            match self.parse_block_id(&id_str) {
                Ok(id) => {
                    match block_repo.find_by_id_column(id).await {
                        Ok(Some(model)) => {
                            blocks.push(self.block_to_json(&model));
                        }
                        Ok(None) => {
                            debug!("Block {} not found", id);
                        }
                        Err(e) => {
                            warn!("Error fetching block {}: {}", id, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Invalid block id '{}': {}", id_str, e);
                }
            }
        }

        serde_json::json!({ "nextBlocks": blocks })
    }

    async fn handle_block_id_request(&self, block_repo: &Arc<dyn BlockRepository>, block_id: &str, limit: usize) -> serde_json::Value {
        debug!("Requesting blocks after {} (limit: {})", block_id, limit);

        match self.parse_block_id(block_id) {
            Ok(start_id) => {
                match block_repo.find_by_id_column(start_id).await {
                    Ok(Some(start_block)) => {
                        let start_height = start_block.height;
                        let end_height = start_height + limit as i32;

                        match block_repo.find_range(start_height + 1, end_height).await {
                            Ok(models) => {
                                let blocks: Vec<serde_json::Value> = models.into_iter()
                                    .map(|m| self.block_to_json(&m))
                                    .collect();
                                serde_json::json!({ "nextBlocks": blocks })
                            }
                            Err(e) => {
                                warn!("Error fetching block range: {}", e);
                                serde_json::json!({ "nextBlocks": [] })
                            }
                        }
                    }
                    Ok(None) => {
                        warn!("Start block {} not found", block_id);
                        serde_json::json!({ "nextBlocks": [] })
                    }
                    Err(e) => {
                        warn!("Error fetching start block: {}", e);
                        serde_json::json!({ "nextBlocks": [] })
                    }
                }
            }
            Err(e) => {
                warn!("Invalid blockId format: {}", e);
                serde_json::json!({ "error": "INVALID_BLOCK_ID" })
            }
        }
    }

    /// 将区块模型转换为 JSON 格式
    /// 对应 Java: Block.getJSONObject()
    fn block_to_json(&self, model: &orm::models::BlockModel) -> serde_json::Value {
        let mut json = serde_json::Map::new();

        // 基本字段（与 Java 一致的驼峰命名）
        json.insert("version".to_string(), serde_json::Value::Number(model.version.into()));
        json.insert("timestamp".to_string(), serde_json::Value::Number(model.timestamp.into()));
        
        // previousBlock 使用无符号长整型字符串
        json.insert("previousBlock".to_string(), 
            model.previous_block_id
                .map(|id| format_unsigned_long(id))
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null)
        );

        // 金额字段（与 Java 一致的命名）
        json.insert("totalAmountNQT".to_string(), serde_json::Value::Number(model.total_amount.into()));
        json.insert("totalFeeNQT".to_string(), serde_json::Value::Number(model.total_fee.into()));
        json.insert("payloadLength".to_string(), serde_json::Value::Number(model.payload_length.into()));

        // 哈希字段（十六进制字符串）
        json.insert("payloadHash".to_string(), 
            serde_json::Value::String(hex::encode(&model.payload_hash)));
        
        // generatorPublicKey - 注意：这里需要从 generation_signature 或其他字段获取
        // Java 中 generatorPublicKey 是单独存储的
        json.insert("generatorPublicKey".to_string(), 
            serde_json::Value::String(hex::encode(&model.generation_signature)));

        json.insert("generationSignature".to_string(), 
            serde_json::Value::String(hex::encode(&model.generation_signature)));

        // previousBlockHash 仅在版本 > 1 时包含
        if model.version > 1 {
            json.insert("previousBlockHash".to_string(), 
                model.previous_block_hash
                    .as_ref()
                    .map(|h| serde_json::Value::String(hex::encode(h)))
                    .unwrap_or(serde_json::Value::String(String::new()))
            );
        }

        json.insert("blockSignature".to_string(), 
            serde_json::Value::String(hex::encode(&model.block_signature)));

        // 交易列表（暂时为空数组）
        json.insert("transactions".to_string(), serde_json::Value::Array(vec![]));

        serde_json::Value::Object(json)
    }

    /// 解析区块ID（支持无符号长整型字符串）
    fn parse_block_id(&self, id_str: &str) -> Result<i64, String> {
        if id_str.starts_with("-") {
            return Err("Block ID cannot be negative".to_string());
        }
        
        id_str.parse::<u64>()
            .map(|v| v as i64)
            .map_err(|e| format!("Invalid block id: {}", e))
    }
}

/// 将 i64 转换为无符号长整型字符串
/// 对应 Java: Long.toUnsignedString()
fn format_unsigned_long(value: i64) -> String {
    (value as u64).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_unsigned_long() {
        assert_eq!(format_unsigned_long(123), "123");
        assert_eq!(format_unsigned_long(-1), "18446744073709551615"); // u64::MAX
    }

    #[test]
    fn test_too_many_blocks_error() {
        let error = too_many_blocks_error();
        assert!(error.get("error").is_some());
    }
}
