//! Transaction API DTOs
//!
//! 交易 API 的数据传输对象

use serde::{Deserialize, Serialize};

/// 发送转账请求
#[derive(Debug, Deserialize)]
pub struct SendMoneyRequest {
    pub recipient: String,
    #[serde(rename = "amountNQT")]
    pub amount_nqt: String,
    #[serde(rename = "feeNQT")]
    pub fee_nqt: String,
    pub deadline: u16,
    #[serde(rename = "secretPhrase")]
    pub secret_phrase: String,
    #[serde(rename = "referencedTransactionFullHash")]
    pub referenced_transaction_full_hash: Option<String>,
    pub message: Option<String>,
    #[serde(rename = "messageIsText")]
    pub message_is_text: Option<bool>,
}

/// 发送转账响应
#[derive(Debug, Serialize)]
pub struct SendMoneyResponse {
    pub transaction: String,
    #[serde(rename = "fullHash")]
    pub full_hash: String,
    #[serde(rename = "transactionBytes")]
    pub transaction_bytes: String,
    #[serde(rename = "signatureHash")]
    pub signature_hash: String,
    #[serde(rename = "requestProcessingTime")]
    pub request_processing_time: u32,
}

/// 获取交易请求
#[derive(Debug, Deserialize)]
pub struct GetTransactionRequest {
    pub transaction: String,
}

/// 获取未确认交易响应
#[derive(Debug, Serialize)]
pub struct GetUnconfirmedTransactionsResponse {
    #[serde(rename = "unconfirmedTransactions")]
    pub unconfirmed_transactions: Vec<serde_json::Value>,
    #[serde(rename = "requestProcessingTime")]
    pub request_processing_time: u32,
}
