//! 区块相关 DTO
//!
//! 与 Java APIGetBlock 等完全对齐

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiGetBlock {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator: Option<String>,
    
    #[serde(rename = "generatorRS", skip_serializing_if = "Option::is_none")]
    pub generator_rs: Option<String>,
    
    #[serde(rename = "generatorPublicKey", skip_serializing_if = "Option::is_none")]
    pub generator_public_key: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i32>,
    
    #[serde(rename = "numberOfTransactions", skip_serializing_if = "Option::is_none")]
    pub number_of_transactions: Option<i32>,
    
    #[serde(rename = "totalAmountNQT", skip_serializing_if = "Option::is_none")]
    pub total_amount_nqt: Option<String>,
    
    #[serde(rename = "totalFeeNQT", skip_serializing_if = "Option::is_none")]
    pub total_fee_nqt: Option<String>,
    
    #[serde(rename = "payloadLength", skip_serializing_if = "Option::is_none")]
    pub payload_length: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
    
    #[serde(rename = "baseTarget", skip_serializing_if = "Option::is_none")]
    pub base_target: Option<String>,
    
    #[serde(rename = "cumulativeDifficulty", skip_serializing_if = "Option::is_none")]
    pub cumulative_difficulty: Option<String>,
    
    #[serde(rename = "previousBlock", skip_serializing_if = "Option::is_none")]
    pub previous_block: Option<String>,
    
    #[serde(rename = "nextBlock", skip_serializing_if = "Option::is_none")]
    pub next_block: Option<String>,
    
    #[serde(rename = "payloadHash", skip_serializing_if = "Option::is_none")]
    pub payload_hash: Option<String>,
    
    #[serde(rename = "generationSignature", skip_serializing_if = "Option::is_none")]
    pub generation_signature: Option<String>,
    
    #[serde(rename = "previousBlockHash", skip_serializing_if = "Option::is_none")]
    pub previous_block_hash: Option<String>,
    
    #[serde(rename = "blockSignature", skip_serializing_if = "Option::is_none")]
    pub block_signature: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transactions: Option<Vec<ApiTransaction>>,
    
    #[serde(rename = "executedPhasedTransactions", skip_serializing_if = "Option::is_none")]
    pub executed_phased_transactions: Option<Vec<ApiTransaction>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiGetBlockId {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiGetBlocks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<ApiGetBlock>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiTransaction {
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
    pub type: Option<u8>,
    
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<u8>,
    
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
    
    #[serde(rename = "attachmentBytes", skip_serializing_if = "Option::is_none")]
    pub attachment_bytes: Option<String>,
    
    #[serde(rename = "ecBlockId", skip_serializing_if = "Option::is_none")]
    pub ec_block_id: Option<String>,
    
    #[serde(rename = "ecBlockHeight", skip_serializing_if = "Option::is_none")]
    pub ec_block_height: Option<i32>,
    
    #[serde(rename = "confirmations", skip_serializing_if = "Option::is_none")]
    pub confirmations: Option<i32>,
}
