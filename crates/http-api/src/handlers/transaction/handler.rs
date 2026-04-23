//! Transaction API Handlers
//!
//! 交易相关的 API 处理器

use axum::{
    extract::State,
    Json,
};
use std::time::Instant;
use blockchain_types::{AccountId, Amount};
use crate::handlers::transaction::{state::TransactionApiState, dto::*};
use crate::core::ApiError;

/// 解析账户 ID
fn parse_account_id(account_str: &str) -> Result<AccountId, ApiError> {
    account_str
        .parse::<AccountId>()
        .map_err(|_| ApiError::InvalidParameter(format!("Invalid account ID: {}", account_str)))
}

/// 解析金额（NQT 单位）
fn parse_amount_nqt(amount_str: &str) -> Result<Amount, ApiError> {
    amount_str
        .parse::<Amount>()
        .map_err(|_| ApiError::InvalidParameter(format!("Invalid amount: {}", amount_str)))
}

/// 发送转账
pub async fn send_money(
    State(_state): State<TransactionApiState>,
    Json(req): Json<SendMoneyRequest>,
) -> Result<Json<SendMoneyResponse>, ApiError> {
    let start_time = Instant::now();
    
    let _recipient = parse_account_id(&req.recipient)?;
    let _amount = parse_amount_nqt(&req.amount_nqt)?;
    let _fee = parse_amount_nqt(&req.fee_nqt)?;
    
    // TODO: 构建交易
    // let sender_id = 0; // 从 secret_phrase 派生
    // let tx = Transaction::new(...);
    
    // TODO: 执行交易
    // let receipt = state.tx_processor.execute(&tx).await?;
    
    let processing_time = start_time.elapsed().as_millis() as u32;
    
    // 暂时返回模拟响应
    Ok(Json(SendMoneyResponse {
        transaction: "0".to_string(),
        full_hash: "0".to_string(),
        transaction_bytes: "0".to_string(),
        signature_hash: "0".to_string(),
        request_processing_time: processing_time,
    }))
}

/// 获取交易信息
pub async fn get_transaction(
    State(_state): State<TransactionApiState>,
    Json(req): Json<GetTransactionRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let start_time = Instant::now();
    
    // TODO: 从 repository 查询交易
    // let tx = state.tx_repo.find_by_hash(&hash_bytes).await?;
    
    let _processing_time = start_time.elapsed().as_millis() as u32;
    
    // 暂时返回错误
    Err(ApiError::NotFound(format!("Transaction {} not found", req.transaction)))
}

/// 获取未确认交易列表
pub async fn get_unconfirmed_transactions(
    State(_state): State<TransactionApiState>,
) -> Result<Json<GetUnconfirmedTransactionsResponse>, ApiError> {
    let start_time = Instant::now();
    
    // TODO: 从交易池获取未确认交易
    let unconfirmed_transactions = vec![];
    
    let processing_time = start_time.elapsed().as_millis() as u32;
    
    Ok(Json(GetUnconfirmedTransactionsResponse {
        unconfirmed_transactions,
        request_processing_time: processing_time,
    }))
}
