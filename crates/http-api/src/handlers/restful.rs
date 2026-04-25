//! RESTful API Handlers
//!
//! RESTful 风格的 API 处理器

use axum::{
    extract::{Path, State},
    Json,
};
use std::time::Instant;
use blockchain_types::AccountId;
use crate::state::ApiState;
use crate::core::ApiError;

fn parse_account_id(account_str: &str) -> Result<AccountId, ApiError> {
    account_str
        .parse::<AccountId>()
        .map_err(|_| ApiError::InvalidParameter(format!("Invalid account ID: {}", account_str)))
}

pub async fn get_account(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let start_time = Instant::now();
    
    let account_id = parse_account_id(&id)?;
    
    let account = state.account_manager
        .get_account_info(account_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let processing_time = start_time.elapsed().as_millis() as u32;
    
    Ok(Json(serde_json::json!({
        "account": account.id.to_string(),
        "accountRS": account.address.unwrap_or_default(),
        "balanceNQT": account.balance.to_string(),
        "unconfirmedBalanceNQT": account.unconfirmed_balance.to_string(),
        "forgedBalanceNQT": "0",
        "guaranteedBalanceNQT": account.guaranteed_balance.to_string(),
        "requestProcessingTime": processing_time
    })))
}

pub async fn get_balance(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let start_time = Instant::now();
    
    let account_id = parse_account_id(&id)?;
    
    let balance = state.account_manager
        .get_balance(account_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let processing_time = start_time.elapsed().as_millis() as u32;
    
    Ok(Json(serde_json::json!({
        "balanceNQT": balance.to_string(),
        "unconfirmedBalanceNQT": balance.to_string(),
        "forgedBalanceNQT": "0",
        "guaranteedBalanceNQT": balance.to_string(),
        "requestProcessingTime": processing_time
    })))
}

pub async fn get_latest_block(
    State(state): State<ApiState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let start_time = Instant::now();
    
    let latest_block = state.block_repo
        .find_latest()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let processing_time = start_time.elapsed().as_millis() as u32;
    
    if let Some(block_model) = latest_block {
        let block = block_model.to_domain()
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        let block_hash = block.compute_hash()
            .unwrap_or(blockchain_types::Hash256([0u8; 32]));
        
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
    } else {
        Ok(Json(serde_json::json!({
            "error": "No blocks found",
            "requestProcessingTime": processing_time
        })))
    }
}

pub async fn get_block_by_height(
    State(state): State<ApiState>,
    Path(height): Path<u32>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let start_time = Instant::now();
    
    let block_model = state.block_repo
        .find_by_height(height as i32)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let processing_time = start_time.elapsed().as_millis() as u32;
    
    if let Some(block_model) = block_model {
        let block = block_model.to_domain()
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        let block_hash = block.compute_hash()
            .unwrap_or(blockchain_types::Hash256([0u8; 32]));
        
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
    } else {
        Err(ApiError::NotFound(format!("Block at height {} not found", height)))
    }
}
