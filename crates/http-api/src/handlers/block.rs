//! 区块相关处理器

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{error::ApiResult, response::BlockResponse, state::ApiState};
use blockchain_types::*;
use orm::BlockRepository;

/// 获取区块详情（根据高度）
pub async fn get_block_by_height(
    State(state): State<crate::state::ApiState>,
    Path(height): Path<u32>,
) -> ApiResult<Json<BlockResponse>> {
    let block_repo = state.block_repo.as_ref();
    let block_model = block_repo.find_by_height(height as i64).await?
        .ok_or_else(|| crate::error::ApiError::NotFound(format!("block at height {} not found", height)))?;

    let block = block_model.to_domain()?;

    // 转换为响应
    let response = BlockResponse {
        height: block.height,
        block_hash: hex::encode(compute_block_hash(&block)), // 需要计算或从 DB 存储
        previous_block_hash: hex::encode(block.previous_block_hash),
        payload_hash: hex::encode(block.payload_hash),
        generator_id: block.generator_id,
        nonce: block.nonce,
        base_target: block.base_target,
        total_amount: block.total_amount,
        total_fee: block.total_fee,
        transaction_count: block.transactions.len(),
        timestamp: block.timestamp,
    };

    Ok(Json(response))
}

/// 获取区块详情（根据哈希）
pub async fn get_block_by_hash(
    State(state): State<ApiState>,
    Path(hash): Path<String>,
) -> ApiResult<Json<BlockResponse>> {
    let hash_bytes = hex::decode(&hash).map_err(|_| crate::error::ApiError::Validation("invalid hex hash".to_string()))?;

    let block_repo = state.block_repo.as_ref();
    let block_model = block_repo.find_by_hash(&hash_bytes).await?
        .ok_or_else(|| crate::error::ApiError::NotFound(format!("block with hash {} not found", hash)))?;

    let block = block_model.to_domain()?;

    let response = BlockResponse {
        height: block.height,
        block_hash: hex::encode(compute_block_hash(&block)),
        previous_block_hash: hex::encode(block.previous_block_hash),
        payload_hash: hex::encode(block.payload_hash),
        generator_id: block.generator_id,
        nonce: block.nonce,
        base_target: block.base_target,
        total_amount: block.total_amount,
        total_fee: block.total_fee,
        transaction_count: block.transactions.len(),
        timestamp: block.timestamp,
    };

    Ok(Json(response))
}

/// 获取最新区块
pub async fn get_latest_block(
    State(state): State<ApiState>,
) -> ApiResult<Json<BlockResponse>> {
    let block_repo = state.block_repo.as_ref();
    let block_model = block_repo.find_latest().await?
        .ok_or_else(|| crate::error::ApiError::NotFound("no blocks found".to_string()))?;

    let block = block_model.to_domain()?;

    let response = BlockResponse {
        height: block.height,
        block_hash: hex::encode(compute_block_hash(&block)),
        previous_block_hash: hex::encode(block.previous_block_hash),
        payload_hash: hex::encode(block.payload_hash),
        generator_id: block.generator_id,
        nonce: block.nonce,
        base_target: block.base_target,
        total_amount: block.total_amount,
        total_fee: block.total_fee,
        transaction_count: block.transactions.len(),
        timestamp: block.timestamp,
    };

    Ok(Json(response))
}

/// 获取区块列表（分页）
pub async fn list_blocks(
    State(state): State<ApiState>,
    Query(query): Query<BlockQuery>,
) -> ApiResult<Json<Vec<BlockResponse>>> {
    let block_repo = state.block_repo.as_ref();

    // 简化：获取最近的 limit 个区块
    let limit = query.limit.unwrap_or(50) as usize;
    let offset = query.offset.unwrap_or(0) as i64;

    // TODO: 实现分页查询
    let blocks = vec![];

    Ok(Json(blocks))
}

/// 计算区块哈希（辅助函数）
fn compute_block_hash(block: &Block) -> Hash256 {
    // 应该使用 Block::compute_hash()
    // 这里由于 Block 结构体已有方法，直接调用
    block.compute_hash().unwrap_or([0u8; 32])
}