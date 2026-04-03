//! # NRCS Blockchain Core Types
//!
//! 核心区块链数据结构定义，包括：
//! - `Block`: 区块结构
//! - `Transaction`: 交易结构
//! - `Account`: 账户结构
//! - `Asset`: 资产结构
//!
//! 所有结构体都实现了 `serde` 的序列化/反序列化，支持 bincode（网络传输）和 JSON（API/日志）。
//!
//! ## 设计原则
//! - 使用固定大小数组（`[u8; 32]`）存储哈希，避免动态分配
//! - 金额使用 `u64`（单位：NQT，10^-8 精度），便于计算但注意溢出
//! - 时间戳使用 `u32`（Unix 秒），与 Java 原版兼容
//! - 所有字段均使用驼峰命名（snake_case），Rust 惯例

use serde::Serialize;

pub mod block;
pub mod transaction;
pub mod account;
pub mod asset;

pub mod prelude {
    pub use crate::block::*;
    pub use crate::transaction::*;
    pub use crate::account::*;
    pub use crate::asset::*;
}

/// 区块链错误类型
#[derive(Debug, thiserror::Error)]
pub enum BlockchainError {
    #[error("invalid block hash: {0}")]
    InvalidHash(String),

    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: u64, need: u64 },

    #[error("serialization error: {0}")]
    Serialization(Box<dyn std::error::Error + Send + Sync>),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, BlockchainError>;

/// 固定大小的 SHA-256 哈希（32 字节）
pub type Hash256 = [u8; 32];
/// 固定大小的 SHA-512 哈希（64 字节）
pub type Hash512 = [u8; 64];

/// 公钥类型（目前仅支持 Ed25519 32 字节）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PublicKey {
    /// Ed25519 公钥（32 字节）
    Ed25519([u8; 32]),
}

impl PublicKey {
    /// 获取公钥长度（字节）
    pub fn len(&self) -> usize {
        match self {
            PublicKey::Ed25519(bytes) => bytes.len(),
        }
    }

    /// 是否为 Ed25519 公钥
    pub fn is_ed25519(&self) -> bool {
        matches!(self, PublicKey::Ed25519(_))
    }

    /// 转换为字节 slice
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            PublicKey::Ed25519(bytes) => bytes,
        }
    }

}

/// 私钥类型（目前仅支持 Ed25519 64 字节）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SecretKey {
    /// Ed25519 私钥（64 字节，包含 seed+public）
    Ed25519([u8; 64]),
}

impl SecretKey {
    /// 获取私钥长度
    pub fn len(&self) -> usize {
        match self {
            SecretKey::Ed25519(bytes) => bytes.len(),
        }
    }

    /// 转换为字节 slice
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            SecretKey::Ed25519(bytes) => bytes,
        }
    }

}

/// 签名类型（Ed25519 和 SM2 均为 64 字节）
pub type Signature = [u8; 64];

/// 时间戳（Unix 时间戳，秒）
pub type Timestamp = u32;
/// 区块高度
pub type Height = u32;
/// NQT 金额单位（1 NRC = 10^8 NQT）
pub type Amount = u64;
/// 账户 ID（公钥哈希）
pub type AccountId = u64;
/// 资产 ID
pub type AssetId = u64;
/// 区块 ID
pub type BlockId = u64;
/// 交易 ID
pub type TransactionId = u64;

/// 交易类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
// #[repr(u8)]  // temporarily removed to avoid compiler ICE
pub enum TransactionType {
    /// 常规支付（0）
    Payment = 0,
    /// 资产转移（1）
    AssetTransfer = 1,
    /// 资产发行（2）
    AssetIssuance = 2,
    /// 智能合约调用（3）
    ContractInvocation = 3,
    /// 合约部署（4）
    ContractDeployment = 4,
    /// 租赁出块权（5）
    Lease = 5,
    /// 设置账户属性（6）
    SetProperty = 6,
    /// 其他类型
    Custom(u8),
}

impl From<u8> for TransactionType {
    fn from(value: u8) -> Self {
        match value {
            0 => TransactionType::Payment,
            1 => TransactionType::AssetTransfer,
            2 => TransactionType::AssetIssuance,
            3 => TransactionType::ContractInvocation,
            4 => TransactionType::ContractDeployment,
            5 => TransactionType::Lease,
            6 => TransactionType::SetProperty,
            _ => TransactionType::Custom(value),
        }
    }
}

impl From<TransactionType> for u8 {
    fn from(ty: TransactionType) -> Self {
        match ty {
            TransactionType::Payment => 0,
            TransactionType::AssetTransfer => 1,
            TransactionType::AssetIssuance => 2,
            TransactionType::ContractInvocation => 3,
            TransactionType::ContractDeployment => 4,
            TransactionType::Lease => 5,
            TransactionType::SetProperty => 6,
            TransactionType::Custom(v) => v,
        }
    }
}

/// 交易收据（Transaction Receipt）
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TxReceipt {
    /// 交易 ID
    pub transaction_id: TransactionId,
    /// 状态：0=pending, 1=success, 2=failed
    pub status: u8,
    /// 消耗的 Gas 数量
    pub gas_used: u64,
    /// 执行日志（JSON 数组字符串）
    pub logs: String,
    /// 合约地址（如果是合约调用）
    pub contract_address: Option<[u8; 20]>,
    /// 执行完成时间戳
    pub executed_at: Timestamp,
}

/// 区块版本
pub const BLOCK_VERSION: u32 = 1;

/// 交易版本
pub const TRANSACTION_VERSION: u8 = 1;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_block_serialization() {
        let block = Block {
            version: BLOCK_VERSION,
            timestamp: 1_704_000_000,
            height: 100,
            previous_block_hash: [0u8; 32],
            payload_hash: [1u8; 32],
            generator_id: 1234567890,
            nonce: 0,
            base_target: 1_000_000,
            cumulative_difficulty: Default::default(),
            total_amount: 1_000_000_000,
            total_fee: 500_000,
            payload_length: 0,
            generation_signature: [2u8; 64],
            block_signature: [3u8; 64],
            transactions: vec![],
        };

        let json = serde_json::to_string(&block).unwrap();
        let decoded: Block = serde_json::from_str(&json).unwrap();
        assert_eq!(block.version, decoded.version);
        assert_eq!(block.height, decoded.height);
    }

    #[test]
    fn test_transaction_type_conversion() {
        assert_eq!(u8::from(TransactionType::Payment), 0);
        assert_eq!(TransactionType::from(0), TransactionType::Payment);
        assert_eq!(TransactionType::from(255), TransactionType::Custom(255));
    }
}
