//! Transaction API Handlers
//!
//! 交易相关的 API 处理器

use axum::{
    extract::State,
    Json,
};
use std::time::Instant;
use blockchain_types::{AccountId, Amount, TransactionType};
use blockchain_types::prelude::*;
use crate::state::ApiState;
use crate::handlers::transaction::dto::*;
use crate::core::ApiError;

/// 获取当前时间戳
fn current_timestamp() -> Timestamp {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as Timestamp
}

/// 解析账户 ID
fn parse_account_id(account_str: &str) -> std::result::Result<AccountId, ApiError> {
    account_str
        .parse::<AccountId>()
        .map_err(|_| ApiError::InvalidParameter(format!("Invalid account ID: {}", account_str)))
}

/// 解析金额（NQT 单位）
fn parse_amount_nqt(amount_str: &str) -> std::result::Result<Amount, ApiError> {
    amount_str
        .parse::<Amount>()
        .map_err(|_| ApiError::InvalidParameter(format!("Invalid amount: {}", amount_str)))
}

/// 发送转账
pub async fn send_money(
    State(state): State<ApiState>,
    Json(req): Json<SendMoneyRequest>,
) -> std::result::Result<Json<SendMoneyResponse>, ApiError> {
    let start_time = Instant::now();
    
    let recipient = parse_account_id(&req.recipient)?;
    let amount = parse_amount_nqt(&req.amount_nqt)?;
    let fee = parse_amount_nqt(&req.fee_nqt)?;
    
    // 从 secret_phrase 派生发送者账户 ID
    let keypair = crypto::passphrase_to_keypair(&req.secret_phrase)
        .map_err(|e| ApiError::InvalidParameter(format!("Invalid secret phrase: {}", e)))?;
    
    let public_key = keypair.public_key();
    let pub_key_bytes = match public_key {
        crypto::PublicKey::Ed25519(bytes) => bytes,
        _ => return Err(ApiError::InvalidParameter("Unsupported key type".to_string())),
    };
    
    // 计算发送者账户 ID（公钥哈希的前 8 字节）
    let hash = crypto::sha256(&pub_key_bytes);
    let sender_id = u64::from_le_bytes([
        hash[0], hash[1], hash[2], hash[3],
        hash[4], hash[5], hash[6], hash[7],
    ]);
    
    // 构建交易
    let mut tx = Transaction::new(
        TransactionType::Payment,
        sender_id,
        Some(recipient),
        amount,
        fee,
        current_timestamp(),
        req.deadline,
    );
    
    // 签名交易
    let tx_bytes = tx.serialize_for_signing();
    let signature = keypair.sign(&tx_bytes);
    tx.signature = blockchain_types::Signature(signature);
    
    // 计算交易哈希
    tx.full_hash = tx.compute_hash()
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    // 执行交易
    let receipt = state.tx_processor.execute(&tx).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let processing_time = start_time.elapsed().as_millis() as u32;
    
    Ok(Json(SendMoneyResponse {
        transaction: receipt.transaction_id.to_string(),
        full_hash: hex::encode(tx.full_hash.0),
        transaction_bytes: hex::encode(&tx_bytes),
        signature_hash: hex::encode(tx.signature.0),
        request_processing_time: processing_time,
    }))
}

/// 获取交易信息
pub async fn get_transaction(
    State(state): State<ApiState>,
    Json(req): Json<GetTransactionRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let start_time = Instant::now();
    
    // 解析交易哈希
    let hash_bytes = hex::decode(&req.transaction)
        .map_err(|_| ApiError::InvalidParameter("Invalid transaction hash".to_string()))?;
    
    if hash_bytes.len() != 32 {
        return Err(ApiError::InvalidParameter("Transaction hash must be 32 bytes".to_string()));
    }
    
    let mut hash_arr = [0u8; 32];
    hash_arr.copy_from_slice(&hash_bytes);
    
    // 从 repository 查询交易
    let tx_model = state.tx_repo
        .find_by_full_hash(&hash_arr)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let processing_time = start_time.elapsed().as_millis() as u32;
    
    if let Some(tx_model) = tx_model {
        Ok(Json(serde_json::json!({
            "transaction": req.transaction,
            "sender": tx_model.sender_id,
            "recipient": tx_model.recipient_id,
            "amountNQT": tx_model.amount.to_string(),
            "feeNQT": tx_model.fee.to_string(),
            "timestamp": tx_model.timestamp,
            "height": tx_model.height,
            "requestProcessingTime": processing_time
        })))
    } else {
        Err(ApiError::NotFound(format!("Transaction {} not found", req.transaction)))
    }
}

/// 获取未确认交易列表
pub async fn get_unconfirmed_transactions(
    State(_state): State<ApiState>,
) -> std::result::Result<Json<GetUnconfirmedTransactionsResponse>, ApiError> {
    let start_time = Instant::now();
    
    // TODO: 从交易池获取未确认交易
    let unconfirmed_transactions = vec![];
    
    let processing_time = start_time.elapsed().as_millis() as u32;
    
    Ok(Json(GetUnconfirmedTransactionsResponse {
        unconfirmed_transactions,
        request_processing_time: processing_time,
    }))
}
