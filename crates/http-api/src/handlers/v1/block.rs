//! 区块相关 API Handlers
//!
//! 与 Java 版本 GetBlock, GetBlocks 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct GetBlockHandler;

impl GetBlockHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBlockHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["block", "height", "timestamp", "includeTransactions", "includeExecutedPhased"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Blocks]
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let block_id = req.get_u64("block");
        let height = req.get_i32("height");
        let timestamp = req.get_i32("timestamp");
        let include_transactions = req.get_bool("includeTransactions");
        
        let block_model = if let Some(id) = block_id {
            state.block_repo
                .find_by_id_column(id as i64)
                .await
                .map_err(ApiError::Repository)?
        } else if let Some(h) = height {
            if h < 0 {
                return Err(ApiError::IncorrectHeight);
            }
            state.block_repo
                .find_by_height(h)
                .await
                .map_err(ApiError::Repository)?
        } else if let Some(ts) = timestamp {
            if ts < 0 {
                return Err(ApiError::IncorrectTimestamp);
            }
            state.block_repo
                .find_latest()
                .await
                .map_err(ApiError::Repository)?
        } else {
            state.block_repo
                .find_latest()
                .await
                .map_err(ApiError::Repository)?
        };
        
        let block = match block_model {
            Some(b) => b,
            None => return Err(ApiError::UnknownBlock),
        };
        
        let block_domain = block.to_domain().map_err(|e| ApiError::Internal(e.to_string()))?;
        
        let mut builder = RsRespBuilder::new();
        
        builder
            .insert("block", block.id.to_string())
            .insert("height", block.height)
            .insert("generator", block.generator_id.to_string())
            .insert("generatorRS", format_account_rs(block.generator_id as u64))
            .insert("timestamp", block.timestamp)
            .insert("numberOfTransactions", block_domain.transactions.len() as i32)
            .insert("totalAmountNQT", block.total_amount.to_string())
            .insert("totalFeeNQT", block.total_fee.to_string())
            .insert("payloadLength", block.payload_length)
            .insert("version", block.version)
            .insert("baseTarget", block.base_target.to_string())
            .insert("cumulativeDifficulty", hex::encode(&block.cumulative_difficulty));
        
        if let Some(prev_id) = block.previous_block_id {
            builder.insert("previousBlock", prev_id.to_string());
        }
        
        builder
            .insert("payloadHash", hex::encode(&block.payload_hash))
            .insert("generationSignature", hex::encode(&block.generation_signature))
            .insert("previousBlockHash", hex::encode(block.previous_block_hash.unwrap_or_default()))
            .insert("blockSignature", hex::encode(&block.block_signature));
        
        if include_transactions {
            let txs: Vec<serde_json::Value> = block_domain.transactions.iter()
                .map(|tx| json!({
                    "transaction": hex::encode(tx.full_hash.0),
                    "type": u8::from(tx.type_id),
                    "subtype": tx.subtype,
                    "timestamp": tx.timestamp,
                    "sender": tx.sender_id.to_string(),
                    "senderRS": format_account_rs(tx.sender_id),
                    "recipient": tx.recipient_id.map(|r| r.to_string()).unwrap_or_default(),
                    "recipientRS": tx.recipient_id.map(format_account_rs).unwrap_or_default(),
                    "amountNQT": tx.amount.to_string(),
                    "feeNQT": tx.fee.to_string()
                }))
                .collect();
            builder.insert("transactions", json!(txs));
        }
        
        Ok(builder.build())
    }
}

pub struct GetBlocksHandler;

impl GetBlocksHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBlocksHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex", "includeTransactions"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Blocks]
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let first_index = req.get_i32("firstIndex").unwrap_or(0);
        let last_index = req.get_i32("lastIndex").unwrap_or(99);
        
        let latest = state.block_repo
            .find_latest()
            .await
            .map_err(ApiError::Repository)?;
        
        let latest_height = latest.map(|b| b.height).unwrap_or(0);
        
        let start_height = (latest_height - first_index).max(0);
        let end_height = (latest_height - last_index).max(0);
        
        let blocks = if start_height >= end_height {
            state.block_repo
                .find_range(end_height, start_height)
                .await
                .map_err(ApiError::Repository)?
        } else {
            vec![]
        };
        
        let blocks_json: Vec<serde_json::Value> = blocks.iter()
            .map(|b| json!({
                "block": b.id.to_string(),
                "height": b.height,
                "generator": b.generator_id.to_string(),
                "generatorRS": format_account_rs(b.generator_id as u64),
                "timestamp": b.timestamp,
                "numberOfTransactions": 0,
                "totalAmountNQT": b.total_amount.to_string(),
                "totalFeeNQT": b.total_fee.to_string()
            }))
            .collect();
        
        let mut builder = RsRespBuilder::new();
        builder.insert("blocks", json!(blocks_json));
        
        Ok(builder.build())
    }
}

pub struct GetBlockIdHandler;

impl GetBlockIdHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBlockIdHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["height"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Blocks]
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let height = req.require_i32("height")?;
        
        if height < 0 {
            return Err(ApiError::IncorrectHeight);
        }
        
        let block = state.block_repo
            .find_by_height(height)
            .await
            .map_err(ApiError::Repository)?;
        
        let mut builder = RsRespBuilder::new();
        
        if let Some(b) = block {
            builder.insert("block", b.id.to_string());
        } else {
            return Err(ApiError::UnknownBlock);
        }
        
        Ok(builder.build())
    }
}

fn format_account_rs(account_id: u64) -> String {
    format!("NRCS-{}-{}-{}", 
        account_id % 10000,
        (account_id / 10000) % 10000,
        (account_id / 100000000) % 10000
    )
}
