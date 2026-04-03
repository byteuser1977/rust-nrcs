//! 账户相关处理器

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{error::ApiResult, response::{AccountResponse, SubmitTransactionResponse}, state::ApiState};
use blockchain_types::*;

/// 查询账户参数
#[derive(Debug, Deserialize)]
pub struct AccountQuery {
    pub address: Option<String>,
    pub account_id: Option<u64>,
}

/// 创建账户请求
#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    #[serde(default)]
    pub initial_balance: Amount,
}

/// 转账请求
#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub from: AccountId,
    pub to: AccountId,
    pub amount: Amount,
}

/// 获取账户信息（根据地址或ID）
pub async fn get_account(
    State(state): State<crate::state::ApiState>,
    Path(account_id): Path<u64>,
) -> ApiResult<Json<AccountResponse>> {
    let account = state.account_manager.get_account_info(account_id).await?;
    Ok(Json(account.into()))
}

/// 创建新账户
pub async fn create_account(
    State(state): State<ApiState>,
    Json(req): Json<CreateAccountRequest>,
) -> ApiResult<Json<SubmitTransactionResponse>> {
    let (kp, account_id, address) = state.account_manager.create_account(Some(req.initial_balance)).await?;

    Ok(Json(SubmitTransactionResponse {
        transaction_id: account_id, // 简化：使用 account_id 作为标识
        full_hash: hex::encode(account_id.to_be_bytes()), // 占位
        status: "success".to_string(),
        message: format!("account created: {}", address),
    }))
}

/// 账户间转账
pub async fn transfer(
    State(state): State<ApiState>,
    Json(req): Json<TransferRequest>,
) -> ApiResult<Json<SubmitTransactionResponse>> {
    state.account_manager.transfer(req.from, req.to, req.amount).await?;

    // 构建一个模拟的交易响应（真实场景应创建并提交交易）
    Ok(Json(SubmitTransactionResponse {
        transaction_id: 0, // TODO: 生成交易 ID
        full_hash: "".to_string(),
        status: "pending".to_string(),
        message: "transfer submitted".to_string(),
    }))
}

/// 获取账户余额
pub async fn get_balance(
    State(state): State<crate::ApiState>,
    Path(account_id): Path<u64>,
) -> ApiResult<Json<serde_json::Value>> {
    let balance = state.account_manager.get_balance(account_id).await?;
    Ok(Json(serde_json::json!({
        "account_id": account_id,
        "balance": balance,
    })))
}