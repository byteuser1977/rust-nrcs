//! Database models (SQLx types)
//!
//! All models are derived from `sqlx::FromRow` and include
//! getter/setter methods for business logic.

use sqlx::FromRow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use blockchain_types::*;

/// 区块模型
#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct BlockModel {
    pub id: i64,
    pub height: i64,
    pub block_hash: Vec<u8>,
    pub previous_block_hash: Vec<u8>,
    pub payload_hash: Vec<u8>,
    pub generator_id: i64,
    pub nonce: i64,
    pub base_target: i64,
    pub cumulative_difficulty: Vec<u8>,
    pub total_amount: i64,
    pub total_fee: i64,
    pub payload_length: i32,
    pub generation_signature: Vec<u8>,
    pub block_signature: Vec<u8>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BlockModel {
    /// Convert to domain Block
    pub fn to_domain(&self) -> Result<Block> {
        let mut block = Block {
            version: self.version as u32,
            timestamp: 0, // 需要从 created_at 转换或存储独立字段
            height: self.height as Height,
            previous_block_hash: self.previous_block_hash.try_into().map_err(|_| BlockchainError::InvalidHash("previous_block_hash length mismatch".to_string()))?,
            payload_hash: self.payload_hash.try_into().map_err(|_| BlockchainError::InvalidHash("payload_hash length mismatch".to_string()))?,
            generator_id: self.generator_id as AccountId,
            nonce: self.nonce as u64,
            base_target: self.base_target as u64,
            cumulative_difficulty: self.cumulative_difficulty.clone(),
            total_amount: self.total_amount as Amount,
            total_fee: self.total_fee as Amount,
            payload_length: self.payload_length as u32,
            generation_signature: self.generation_signature.try_into().map_err(|_| BlockchainError::InvalidHash("generation_signature length mismatch".to_string()))?,
            block_signature: self.block_signature.try_into().map_err(|_| BlockchainError::InvalidHash("block_signature length mismatch".to_string()))?,
            transactions: vec![],
        };

        // 从 created_at 提取 timestamp
        block.timestamp = self.created_at.timestamp() as Timestamp;

        Ok(block)
    }

    /// Convert from domain Block
    pub fn from_domain(block: &Block) -> Result<Self> {
        Ok(Self {
            id: 0, // auto-increment
            height: block.height as i64,
            block_hash: vec![], // 应由数据库触发器或应用逻辑填充
            previous_block_hash: block.previous_block_hash.to_vec(),
            payload_hash: block.payload_hash.to_vec(),
            generator_id: block.generator_id as i64,
            nonce: block.nonce as i64,
            base_target: block.base_target as i64,
            cumulative_difficulty: block.cumulative_difficulty.clone(),
            total_amount: block.total_amount as i64,
            total_fee: block.total_fee as i64,
            payload_length: block.payload_length as i32,
            generation_signature: block.generation_signature.to_vec(),
            block_signature: block.block_signature.to_vec(),
            version: block.version as i32,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}

/// 交易模型
#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct TransactionModel {
    pub id: i64,
    pub transaction_id: i64,
    pub full_hash: Vec<u8>,
    pub type_id: i16,
    pub subtype: i16,
    pub sender_id: i64,
    pub recipient_id: Option<i64>,
    pub amount: i64,
    pub fee: i64,
    pub block_id: Option<i64>,
    pub height: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub deadline: i32,
    pub signature: Vec<u8>,
    pub attachment_bytes: Vec<u8>,
    pub phased: bool,
    pub has_message: bool,
    pub has_encrypted_message: bool,
    pub has_public_key_announcement: bool,
    pub has_prunable_attachment: bool,
    pub ec_block_height: Option<i32>,
    pub ec_block_id: Option<i64>,
    pub has_encrypttoself_message: bool,
    pub has_prunable_encrypted_message: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TransactionModel {
    pub fn to_domain(&self) -> Result<Transaction> {
        let sender_id = self.sender_id as AccountId;
        let mut tx = Transaction {
            version: TRANSACTION_VERSION,
            type_id: TransactionType::from(self.type_id as u8),
            subtype: self.subtype as u8,
            timestamp: self.timestamp.timestamp() as Timestamp,
            deadline: self.deadline as u16,
            sender_id,
            recipient_id: self.recipient_id.map(|id| id as AccountId),
            amount: self.amount as Amount,
            fee: self.fee as Amount,
            height: self.height.unwrap_or(0) as Height,
            block_id: self.block_id.unwrap_or(0) as BlockId,
            signature: self.signature.try_into().map_err(|_| BlockchainError::InvalidTransaction("signature length mismatch".to_string()))?,
            full_hash: self.full_hash.try_into().map_err(|_| BlockchainError::InvalidHash("full_hash length mismatch".to_string()))?,
            attachment_bytes: self.attachment_bytes.clone(),
            phased: self.phased,
            has_message: self.has_message,
            has_encrypted_message: self.has_encrypted_message,
            has_public_key_announcement: self.has_public_key_announcement,
            has_prunable_attachment: self.has_prunable_attachment,
            ec_block_height: self.ec_block_height.map(|h| h as u32),
            ec_block_id: self.ec_block_id.map(|id| id as u64),
            has_encrypttoself_message: self.has_encrypttoself_message,
            has_prunable_encrypted_message: self.has_prunable_encrypted_message,
        };
        Ok(tx)
    }

    pub fn from_domain(tx: &Transaction) -> Result<Self> {
        Ok(Self {
            id: 0,
            transaction_id: 0, // TODO: derive from full_hash?
            full_hash: tx.full_hash.to_vec(),
            type_id: u8::from(tx.type_id) as i16,
            subtype: tx.subtype as i16,
            sender_id: tx.sender_id as i64,
            recipient_id: tx.recipient_id.map(|id| id as i64),
            amount: tx.amount as i64,
            fee: tx.fee as i64,
            block_id: if tx.block_id == 0 { None } else { Some(tx.block_id as i64) },
            height: if tx.height == 0 { None } else { Some(tx.height as i64) },
            timestamp: Utc.timestamp_opt(tx.timestamp as i64, 0).single().ok_or_else(|| BlockchainError::InvalidTransaction("invalid timestamp".to_string()))?,
            deadline: tx.deadline as i32,
            signature: tx.signature.to_vec(),
            attachment_bytes: tx.attachment_bytes.clone(),
            phased: tx.phased,
            has_message: tx.has_message,
            has_encrypted_message: tx.has_encrypted_message,
            has_public_key_announcement: tx.has_public_key_announcement,
            has_prunable_attachment: tx.has_prunable_attachment,
            ec_block_height: tx.ec_block_height.map(|h| h as i32),
            ec_block_id: tx.ec_block_id.map(|id| id as i64),
            has_encrypttoself_message: tx.has_encrypttoself_message,
            has_prunable_encrypted_message: tx.has_prunable_encrypted_message,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}

/// 账户模型
#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AccountModel {
    pub id: i64,
    pub account_id: i64,
    pub address: Option<String>,
    pub public_key: Vec<u8>,
    pub balance: i64,
    pub unconfirmed_balance: i64,
    pub reserved_balance: i64,
    pub guaranteed_balance: i64,
    pub properties: serde_json::Value,
    pub lease: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub current_height: i64,
}

impl AccountModel {
    pub fn to_domain(&self) -> Result<Account> {
        let account_id = self.account_id as AccountId;

        // Parse properties JSON
        let mut properties = HashMap::new();
        if let serde_json::Value::Object(map) = &self.properties {
            for (k, v) in map {
                if let serde_json::Value::String(s) = v {
                    properties.insert(k.clone(), s.clone());
                }
            }
        }

        // Parse lease JSON
        let lease = if let Some(lease_json) = &self.lease {
            if let serde_json::Value::Object(map) = lease_json {
                let lessee_id = map.get("lessee_id").and_then(|v| v.as_i64()).unwrap_or(0) as AccountId;
                let amount = map.get("amount").and_then(|v| v.as_i64()).unwrap_or(0) as Amount;
                let start_height = map.get("start_height").and_then(|v| v.as_i64()).unwrap_or(0) as Height;
                let end_height = map.get("end_height").and_then(|v| v.as_i64()).unwrap_or(0) as Height;
                Some(AccountLease {
                    lessee_id,
                    amount,
                    start_height,
                    end_height,
                })
            } else {
                None
            }
        } else {
            None
        };

        Ok(Account {
            id: account_id,
            address: self.address.clone(),
            balance: self.balance as Amount,
            unconfirmed_balance: self.unconfirmed_balance as Amount,
            reserved_balance: self.reserved_balance as Amount,
            guaranteed_balance: self.guaranteed_balance as Amount,
            assets: HashMap::new(), // separates query
            properties,
            lease,
            created_at: self.created_at.timestamp() as Timestamp,
            last_updated: self.last_updated.timestamp() as Timestamp,
            current_height: self.current_height as Height,
        })
    }

    pub fn from_domain(account: &Account) -> Result<Self> {
        // Serialize properties to JSON
        let properties_serial = serde_json::to_value(&account.properties).map_err(|e| BlockchainError::Serialization(e.into()))?;

        // Serialize lease to JSON
        let lease_serial = account.lease.as_ref().map(|lease| {
            serde_json::json!({
                "lessee_id": lease.lessee_id,
                "amount": lease.amount,
                "start_height": lease.start_height,
                "end_height": lease.end_height
            })
        });

        Ok(Self {
            id: 0,
            account_id: account.id as i64,
            address: account.address.clone(),
            public_key: vec![], // to be set separately
            balance: account.balance as i64,
            unconfirmed_balance: account.unconfirmed_balance as i64,
            reserved_balance: account.reserved_balance as i64,
            guaranteed_balance: account.guaranteed_balance as i64,
            properties: properties_serial,
            lease: lease_serial,
            created_at: Utc.timestamp_opt(account.created_at as i64, 0).single().ok_or_else(|| BlockchainError::InvalidTransaction("invalid created_at".to_string()))?,
            last_updated: Utc.timestamp_opt(account.last_updated as i64, 0).single().ok_or_else(|| BlockchainError::InvalidTransaction("invalid last_updated".to_string()))?,
            current_height: account.current_height as i64,
        })
    }
}

/// 账户资产持仓模型
#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AccountAssetModel {
    pub id: i64,
    pub account_id: i64,
    pub asset_id: i64,
    pub quantity: i64,
    pub last_updated: DateTime<Utc>,
}

impl AccountAssetModel {
    pub fn to_domain(&self) -> Result<AccountAsset> {
        Ok(AccountAsset {
            account_id: self.account_id as AccountId,
            asset_id: self.asset_id as AssetId,
            quantity: self.quantity as Amount,
            last_updated: self.last_updated.timestamp() as Timestamp,
        })
    }

    pub fn from_domain(aa: &AccountAsset) -> Result<Self> {
        Ok(Self {
            id: 0,
            account_id: aa.account_id as i64,
            asset_id: aa.asset_id as i64,
            quantity: aa.quantity as i64,
            last_updated: Utc.timestamp_opt(aa.last_updated as i64, 0).single().ok_or_else(|| BlockchainError::InvalidTransaction("invalid last_updated".to_string()))?,
        })
    }
}

/// 资产模型
#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AssetModel {
    pub id: i64,
    pub asset_id: i64,
    pub owner_id: i64,
    pub name: String,
    pub description: String,
    pub quantity: i64,
    pub decimals: i16,
    pub mintable: bool,
    pub transferable: bool,
    pub data: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub deleted: bool,
}

impl AssetModel {
    pub fn to_domain(&self) -> Result<Asset> {
        Ok(Asset {
            id: self.asset_id as AssetId,
            owner_id: self.owner_id as AccountId,
            name: self.name.clone(),
            description: self.description.clone(),
            quantity: self.quantity as Amount,
            decimals: self.decimals as u8,
            mintable: self.mintable,
            transferable: self.transferable,
            data: self.data.clone(),
            created_at: self.created_at.timestamp() as Timestamp,
            last_updated: self.last_updated.timestamp() as Timestamp,
            deleted: self.deleted,
        })
    }

    pub fn from_domain(asset: &Asset) -> Result<Self> {
        Ok(Self {
            id: 0,
            asset_id: asset.id as i64,
            owner_id: asset.owner_id as i64,
            name: asset.name.clone(),
            description: asset.description.clone(),
            quantity: asset.quantity as i64,
            decimals: asset.decimals as i16,
            mintable: asset.mintable,
            transferable: asset.transferable,
            data: asset.data.clone(),
            created_at: Utc.timestamp_opt(asset.created_at as i64, 0).single().ok_or_else(|| BlockchainError::InvalidTransaction("invalid created_at".to_string()))?,
            last_updated: Utc.timestamp_opt(asset.last_updated as i64, 0).single().ok_or_else(|| BlockchainError::InvalidTransaction("invalid last_updated".to_string()))?,
            deleted: asset.deleted,
        })
    }
}

/// 合约模型
#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct ContractModel {
    pub id: i64,
    pub contract_address: Vec<u8>,
    pub creator_id: i64,
    pub bytecode: Vec<u8>,
    pub storage: serde_json::Value,
    pub gas_limit: i64,
    pub gas_price: i64,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

impl ContractModel {
    pub fn to_domain(&self) -> Result<Contract> {
        // TODO: Implement Contract domain type in blockchain-types
        // For now, return placeholder
        Err(BlockchainError::InvalidTransaction("Contract type not yet implemented".to_string()))
    }

    pub fn from_domain(_contract: &Contract) -> Result<Self> {
        // TODO: Implement Contract domain type
        Err(BlockchainError::InvalidTransaction("Contract type not yet implemented".to_string()))
    }
}

/// 交易收据模型
#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct TransactionReceiptModel {
    pub id: i64,
    pub transaction_id: i64,
    pub status: i16,
    pub gas_used: i64,
    pub logs: serde_json::Value,
    pub contract_address: Option<Vec<u8>>,
    pub executed_at: DateTime<Utc>,
}

impl TransactionReceiptModel {
    pub fn to_domain(&self) -> Result<TxReceipt> {
        Ok(TxReceipt {
            transaction_id: self.transaction_id as TransactionId,
            status: self.status as u8,
            gas_used: self.gas_used as u64,
            logs: self.logs.to_string(),
            contract_address: self.contract_address.as_ref().and_then(|addr| {
                if addr.len() == 20 {
                    Some(addr.clone().try_into().ok())
                } else {
                    None
                }
            }),
            executed_at: self.executed_at.timestamp() as Timestamp,
        })
    }

    pub fn from_domain(receipt: &TxReceipt) -> Result<Self> {
        Ok(Self {
            id: 0,
            transaction_id: receipt.transaction_id as i64,
            status: receipt.status as i16,
            gas_used: receipt.gas_used as i64,
            logs: serde_json::from_str(&receipt.logs).unwrap_or_default(),
            contract_address: receipt.contract_address.map(|addr| addr.to_vec()),
            executed_at: Utc.timestamp_opt(receipt.executed_at as i64, 0).single().ok_or_else(|| BlockchainError::InvalidTransaction("invalid executed_at".to_string()))?,
        })
    }
}