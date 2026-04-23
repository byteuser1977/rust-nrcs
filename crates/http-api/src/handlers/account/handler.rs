//! Account API Handlers
//!
//! 账户相关的 API 处理器

use axum::{
    extract::State,
    Json,
};
use std::time::Instant;
use blockchain_types::AccountId;
use crate::handlers::account::{state::AccountApiState, dto::*};
use crate::core::ApiError;

/// 解析账户 ID
fn parse_account_id(account_str: &str) -> Result<AccountId, ApiError> {
    account_str
        .parse::<AccountId>()
        .map_err(|_| ApiError::InvalidParameter(format!("Invalid account ID: {}", account_str)))
}

/// 获取账户信息
pub async fn get_account(
    State(state): State<AccountApiState>,
    Json(req): Json<GetAccountRequest>,
) -> Result<Json<GetAccountResponse>, ApiError> {
    let start_time = Instant::now();
    
    let account_id = parse_account_id(&req.account)?;
    
    let account = state.account_manager
        .get_account_info(account_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let processing_time = start_time.elapsed().as_millis() as u32;
    
    Ok(Json(GetAccountResponse {
        account: account.id.to_string(),
        account_rs: account.address.unwrap_or_default(),
        balance_nqt: account.balance.to_string(),
        unconfirmed_balance_nqt: account.unconfirmed_balance.to_string(),
        forged_balance_nqt: "0".to_string(), // Account 没有 forged_balance 字段
        guaranteed_balance_nqt: account.guaranteed_balance.to_string(),
        public_key: None, // Account 没有 public_key 字段
        request_processing_time: processing_time,
    }))
}

/// 获取账户余额
pub async fn get_balance(
    State(state): State<AccountApiState>,
    Json(req): Json<GetBalanceRequest>,
) -> Result<Json<GetBalanceResponse>, ApiError> {
    let start_time = Instant::now();
    
    let account_id = parse_account_id(&req.account)?;
    
    let balance = state.account_manager
        .get_balance(account_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let processing_time = start_time.elapsed().as_millis() as u32;
    
    Ok(Json(GetBalanceResponse {
        balance_nqt: balance.to_string(),
        unconfirmed_balance_nqt: balance.to_string(),
        forged_balance_nqt: "0".to_string(),
        guaranteed_balance_nqt: balance.to_string(),
        request_processing_time: processing_time,
    }))
}

/// 获取账户公钥
pub async fn get_account_public_key(
    State(state): State<AccountApiState>,
    Json(req): Json<GetAccountRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let start_time = Instant::now();
    
    let account_id = parse_account_id(&req.account)?;
    
    let _account = state.account_manager
        .get_account_info(account_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let processing_time = start_time.elapsed().as_millis() as u32;
    
    // Account 没有 public_key 字段，返回空字符串
    Ok(Json(serde_json::json!({
        "publicKey": "",
        "requestProcessingTime": processing_time
    })))
}
