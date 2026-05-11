//! Block API Handlers
//!
//! 区块相关的 API 处理器

use axum::{
    extract::State,
    Json,
};
use std::time::Instant;
use blockchain_types::Hash256;
use crate::handlers::block::{state::BlockApiState, dto::*};
use crate::core::ApiError;

/// 获取区块信息
pub async fn get_block(
    State(state): State<BlockApiState>,
    Json(req): Json<GetBlockRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let start_time = Instant::now();
    
    let block_model = if let Some(height) = req.height {
        // 按高度查询
        state.block_repo
            .find_by_height(height as i32)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?
    } else if let Some(block_id) = req.block {
        // 按区块 ID 查询
        let block_id_num = block_id.parse::<u64>()
            .map_err(|_| ApiError::InvalidParameter("Invalid block ID".to_string()))?;
        state.block_repo
            .find_by_id(block_id_num as i64)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?
    } else {
        return Err(ApiError::InvalidParameter("Missing height or block parameter".to_string()));
    };
    
    let block_model = block_model
        .ok_or_else(|| ApiError::NotFound("Block not found".to_string()))?;
    
    let block = block_model.to_domain()
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    // 计算区块哈希作为 ID
    let block_hash = block.compute_hash()
        .unwrap_or(Hash256([0u8; 32]));
    
    let processing_time = start_time.elapsed().as_millis() as u32;
    
    Ok(Json(serde_json::json!({
        "block": hex::encode(block_hash),
        "height": block.height,
        "generator": block.get_generator_id().to_string(),
        "timestamp": block.timestamp,
        "numberOfTransactions": block.transactions.len(),
        "totalAmountNQT": block.total_amount.to_string(),
        "totalFeeNQT": block.total_fee.to_string(),
        "payloadLength": block.payload_length,
        "version": block.version,
        "baseTarget": block.base_target,
        "previousBlock": hex::encode(block.previous_block_hash),
        "payloadHash": hex::encode(block.payload_hash),
        "generationSignature": hex::encode(block.generation_signature),
        "blockSignature": hex::encode(block.block_signature),
        "requestProcessingTime": processing_time
    })))
}

/// 获取区块列表
pub async fn get_blocks(
    State(_state): State<BlockApiState>,
    Json(req): Json<GetBlocksRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let start_time = Instant::now();
    
    let first_index = req.first_index.unwrap_or(0) as i32;
    let last_index = req.last_index.unwrap_or(99) as i32;

    let block_models = _state.block_repo
        .find_range(first_index, last_index)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let blocks: Vec<serde_json::Value> = block_models.iter()
        .filter_map(|m| {
            m.to_domain().ok().map(|b| {
                let hash = b.compute_hash().unwrap_or(Hash256([0u8; 32]));
                serde_json::json!({
                    "block": b.id.map(|id| id.to_string()).unwrap_or_default(),
                    "height": b.height,
                    "generator": b.generator_id.map(|id| id.to_string()).unwrap_or_default(),
                    "timestamp": b.timestamp,
                    "numberOfTransactions": b.transactions.len(),
                    "totalAmountNQT": b.total_amount.to_string(),
                    "totalFeeNQT": b.total_fee.to_string(),
                    "payloadLength": b.payload_length,
                    "blockSignature": hex::encode(b.block_signature.0),
                    "previousBlock": b.previous_block_id.map(|id| id.to_string()),
                    "fullHash": hex::encode(hash.0),
                })
            })
        })
        .collect();
    
    let processing_time = start_time.elapsed().as_millis() as u32;
    
    Ok(Json(serde_json::json!({
        "blocks": blocks,
        "requestProcessingTime": processing_time
    })))
}

/// 获取区块链状态
pub async fn get_blockchain_status(
    State(state): State<BlockApiState>,
) -> Result<Json<GetBlockchainStatusResponse>, ApiError> {
    let start_time = Instant::now();
    
    // 获取最新区块
    let latest_block = state.block_repo
        .find_latest()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let (last_block, number_of_blocks) = if let Some(block_model) = latest_block {
        let block = block_model.to_domain()
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        let block_hash = block.compute_hash()
            .unwrap_or(Hash256([0u8; 32]));
        (hex::encode(block_hash), block.height + 1)
    } else {
        ("0".to_string(), 0)
    };
    
    let processing_time = start_time.elapsed().as_millis() as u32;
    
    Ok(Json(GetBlockchainStatusResponse {
        application: "NRCS".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        time: chrono::Utc::now().timestamp() as u64,
        last_block,
        last_blockchain_feeder: None,
        last_blockchain_feeder_height: None,
        is_scanning: false,
        available_peers: true,
        number_of_blocks,
        is_testnet: false,
        request_processing_time: processing_time,
    }))
}
