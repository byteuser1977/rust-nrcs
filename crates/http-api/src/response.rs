//! 统一响应格式定义

use serde::{Deserialize, Serialize};
use blockchain_types::*;

/// 账户详情响应
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountResponse {
    pub account_id: AccountId,
    pub address: Option<String>,
    pub balance: Amount,
    pub unconfirmed_balance: Amount,
    pub reserved_balance: Amount,
    pub guaranteed_balance: Amount,
    pub assets: Vec<AssetHolding>,
    pub properties: Vec<PropertyEntry>,
    pub current_height: Height,
    pub created_at: Timestamp,
}

/// 资产持仓
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetHolding {
    pub asset_id: AssetId,
    pub quantity: Amount,
}

/// 账户属性
#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyEntry {
    pub key: String,
    pub value: String,
}

impl From<Account> for AccountResponse {
    fn from(account: Account) -> Self {
        Self {
            account_id: account.id,
            address: account.address,
            balance: account.balance,
            unconfirmed_balance: account.unconfirmed_balance,
            reserved_balance: account.reserved_balance,
            guaranteed_balance: account.guaranteed_balance,
            assets: account.assets.iter().map(|(asset_id, quantity)| AssetHolding {
                asset_id: *asset_id,
                quantity: *quantity,
            }).collect(),
            properties: account.properties.iter().map(|(k, v)| PropertyEntry {
                key: k.clone(),
                value: v.clone(),
            }).collect(),
            current_height: account.current_height,
            created_at: account.created_at,
        }
    }
}

/// 交易详情响应
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub transaction_id: TransactionId,
    pub full_hash: String,
    pub type_id: u8,
    pub sender_id: AccountId,
    pub recipient_id: Option<AccountId>,
    pub amount: Amount,
    pub fee: Amount,
    pub height: Option<Height>,
    pub block_id: Option<BlockId>,
    pub timestamp: Timestamp,
    pub status: String, // "pending", "confirmed", "success", "failed"
    pub gas_used: Option<u64>,
}

/// 区块详情响应
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockResponse {
    pub height: Height,
    pub block_hash: String,
    pub previous_block_hash: String,
    pub payload_hash: String,
    pub generator_id: AccountId,
    pub nonce: u64,
    pub base_target: u64,
    pub total_amount: Amount,
    pub total_fee: Amount,
    pub transaction_count: usize,
    pub timestamp: Timestamp,
}

/// 节点信息响应
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeInfoResponse {
    pub version: String,
    pub chain_id: String,
    pub height: Height,
    pub syncing: bool,
    pub peer_count: usize,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub uptime_seconds: u64,
}

/// 提交交易响应
#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitTransactionResponse {
    pub transaction_id: TransactionId,
    pub full_hash: String,
    pub status: String,
    pub message: String,
}