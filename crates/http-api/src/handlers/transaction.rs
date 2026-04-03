//! 交易相关处理器

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{error::ApiResult, response::{TransactionResponse, SubmitTransactionResponse}};
use blockchain_types::*;
use tx_engine::TransactionProcessor;

/// 提交交易请求
#[derive(Debug, Deserialize)]
pub struct SubmitTransactionRequest {
    pub type_id: u8,
    pub sender_id: AccountId,
    pub recipient_id: Option<AccountId>,
    pub amount: Amount,
    pub fee: Amount,
    pub deadline: u16,
    #[serde(with = "serde_bytes")]
    pub attachment_bytes: Vec<u8>,
}

/// 查询交易参数
#[derive(Debug, Deserialize)]
pub struct TransactionQuery {
    pub sender_id: Option<AccountId>,
    pub recipient_id: Option<AccountId>,
    pub limit: Option<u32>,
}

/// 提交交易（广播到网络）
pub async fn submit_transaction(
    State(state): State<ApiState>,
    Json(req): Json<SubmitTransactionRequest>,
) -> ApiResult<Json<SubmitTransactionResponse>> {
    // 构建交易（简化）
    let mut tx = Transaction::new(
        TransactionType::from(req.type_id),
        req.sender_id,
        req.recipient_id,
        req.amount,
        req.fee,
        chrono::Utc::now().timestamp() as u32,
        req.deadline,
    );
    tx.attachment_bytes = req.attachment_bytes;

    // 计算哈希
    tx.full_hash = tx.compute_hash()?;

    // 验证并执行（示例：直接执行）
    let receipt = state.tx_processor.execute(&tx).await?;

    Ok(Json(SubmitTransactionResponse {
        transaction_id: 0, // 实际应从 receipt 或交易中获取
        full_hash: hex::encode(tx.full_hash),
        status: format!("{:?}", receipt.status),
        message: "transaction submitted".to_string(),
    }))
}

/// 查询交易详情
pub async fn get_transaction(
    State(state): State<ApiState>,
    Path(tx_hash): Path<String>,
) -> ApiResult<Json<TransactionResponse>> {
    // 从哈希查找交易
    let hash_bytes = hex::decode(&tx_hash).map_err(|_| crate::error::ApiError::Validation("invalid hex hash".to_string()))?;
    let hash_arr: [u8; 32] = hash_bytes.try_into().map_err(|_| crate::error::ApiError::Validation("hash length mismatch".to_string()))?;

    // TODO: 从 repository 查询
    return Err(crate::error::ApiError::NotFound("transaction not found".to_string()));
}

/// 查询交易列表
pub async fn list_transactions(
    State(state): State<ApiState>,
    Query(query): Query<TransactionQuery>,
) -> ApiResult<Json<Vec<TransactionResponse>>> {
    let limit = query.limit.unwrap_or(50) as i64;

    // TODO: 根据 sender_id/recipient_id 筛选
    let mut txs = Vec::new();

    Ok(Json(txs))
}