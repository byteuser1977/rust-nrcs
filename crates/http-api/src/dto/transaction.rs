//! 交易相关 DTO
//!
//! 与 Java 版本完全对齐

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiGetTransaction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<String>,
    
    #[serde(rename = "senderRS", skip_serializing_if = "Option::is_none")]
    pub sender_rs: Option<String>,
    
    #[serde(rename = "senderPublicKey", skip_serializing_if = "Option::is_none")]
    pub sender_public_key: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient: Option<String>,
    
    #[serde(rename = "recipientRS", skip_serializing_if = "Option::is_none")]
    pub recipient_rs: Option<String>,
    
    #[serde(rename = "recipientPublicKey", skip_serializing_if = "Option::is_none")]
    pub recipient_public_key: Option<String>,
    
    #[serde(rename = "amountNQT", skip_serializing_if = "Option::is_none")]
    pub amount_nqt: Option<String>,
    
    #[serde(rename = "feeNQT", skip_serializing_if = "Option::is_none")]
    pub fee_nqt: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<u8>,
    
    #[serde(rename = "subtype", skip_serializing_if = "Option::is_none")]
    pub subtype: Option<u8>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block: Option<String>,
    
    #[serde(rename = "blockTimestamp", skip_serializing_if = "Option::is_none")]
    pub block_timestamp: Option<i32>,
    
    #[serde(rename = "fullHash", skip_serializing_if = "Option::is_none")]
    pub full_hash: Option<String>,
    
    #[serde(rename = "signatureHash", skip_serializing_if = "Option::is_none")]
    pub signature_hash: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<serde_json::Value>,
    
    #[serde(rename = "ecBlockId", skip_serializing_if = "Option::is_none")]
    pub ec_block_id: Option<String>,
    
    #[serde(rename = "ecBlockHeight", skip_serializing_if = "Option::is_none")]
    pub ec_block_height: Option<i32>,
    
    #[serde(rename = "confirmations", skip_serializing_if = "Option::is_none")]
    pub confirmations: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiGetTransactionBytes {
    #[serde(rename = "transactionBytes", skip_serializing_if = "Option::is_none")]
    pub transaction_bytes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiGetUnconfirmedTransactions {
    #[serde(rename = "unconfirmedTransactions", skip_serializing_if = "Option::is_none")]
    pub unconfirmed_transactions: Option<Vec<ApiGetTransaction>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiBroadcastTransaction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<String>,
    
    #[serde(rename = "fullHash", skip_serializing_if = "Option::is_none")]
    pub full_hash: Option<String>,
    
    #[serde(rename = "numberPeersSentTo", skip_serializing_if = "Option::is_none")]
    pub number_peers_sent_to: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiSendMoney {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<String>,
    
    #[serde(rename = "fullHash", skip_serializing_if = "Option::is_none")]
    pub full_hash: Option<String>,
    
    #[serde(rename = "transactionBytes", skip_serializing_if = "Option::is_none")]
    pub transaction_bytes: Option<String>,
    
    #[serde(rename = "signatureHash", skip_serializing_if = "Option::is_none")]
    pub signature_hash: Option<String>,
}

pub type ApiSendMessage = ApiSendMoney;
pub type ApiSendTransaction = ApiSendMoney;
