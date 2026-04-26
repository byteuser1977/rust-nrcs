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

#![allow(warnings)]

use serde::{Serialize, Deserialize, Serializer, Deserializer};
use std::ops::{Deref, DerefMut};

pub mod constants;
pub mod config;

pub use constants::*;

pub mod block;
pub mod transaction;
pub mod account;
pub mod asset;
pub mod currency;
pub mod order;
pub mod alias;
pub mod account_ext;

pub mod genesis;
pub mod sync;
pub mod fork;
pub mod validation;
pub mod processor;

pub mod prelude {
    pub use crate::block::*;
    pub use crate::transaction::*;
    pub use crate::account::*;
    pub use crate::asset::*;
    pub use crate::currency::*;
    pub use crate::order::*;
    pub use crate::alias::*;
    pub use crate::account_ext::*;
    pub use crate::genesis::*;
    pub use crate::constants::*;
    pub use crate::config::GlobalConfig;
    pub use crate::{
        Hash256, Hash512, PublicKey, SecretKey, Signature,
        Timestamp, Height, Amount, AccountId, AssetId, BlockId,
        TransactionId, TransactionType, TxReceipt,
        CurrencyId, TransferId, OrderId, AliasId, AliasOfferId,
        BlockchainError, Result,
    };
}

/// 区块链错误类型
#[derive(Debug, thiserror::Error)]
pub enum BlockchainError {
    #[error("invalid block hash: {0}")]
    InvalidHash(String),

    #[error("invalid block: {0}")]
    InvalidBlock(String),

    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    #[error("insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: u64, need: u64 },

    #[error("serialization error: {0}")]
    Serialization(Box<dyn std::error::Error + Send + Sync>),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("database error: {0}")]
    Database(String),
}

pub type Result<T> = std::result::Result<T, BlockchainError>;

mod hex_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
    
    pub fn serialize<const N: usize, S>(arr: &[u8; N], serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(arr))
    }
    
    pub fn deserialize<'de, const N: usize, D>(deserializer: D) -> std::result::Result<[u8; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        struct HexVisitor<const N: usize>;
        
        impl<'de, const N: usize> de::Visitor<'de> for HexVisitor<N> {
            type Value = [u8; N];
            
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(&format!("a hex string of {} bytes or an array", N))
            }
            
            fn visit_str<E>(self, value: &str) -> std::result::Result<[u8; N], E>
            where
                E: de::Error,
            {
                let bytes = hex::decode(value).map_err(|e| de::Error::custom(format!("invalid hex: {}", e)))?;
                if bytes.len() != N {
                    return Err(de::Error::custom(format!("expected {} bytes, got {}", N, bytes.len())));
                }
                let mut arr = [0u8; N];
                arr.copy_from_slice(&bytes);
                Ok(arr)
            }
            
            fn visit_seq<A>(self, mut seq: A) -> std::result::Result<[u8; N], A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut arr = [0u8; N];
                for i in 0..N {
                    arr[i] = seq.next_element()?
                        .ok_or_else(|| de::Error::invalid_length(i, &self))?;
                }
                Ok(arr)
            }
        }
        
        deserializer.deserialize_any(HexVisitor::<N>)
    }
}

/// 固定大小的 SHA-256 哈希（32 字节）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hash256(pub [u8; 32]);

impl Serialize for Hash256 {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        hex_serde::serialize::<32, S>(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for Hash256 {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Hash256(hex_serde::deserialize::<32, D>(deserializer)?))
    }
}

impl std::fmt::Display for Hash256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl std::hash::Hash for Hash256 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl AsRef<[u8]> for Hash256 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// 固定大小的 SHA-512 哈希（64 字节）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hash512(pub [u8; 64]);

impl Serialize for Hash512 {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        hex_serde::serialize::<64, S>(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for Hash512 {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Hash512(hex_serde::deserialize::<64, D>(deserializer)?))
    }
}

impl std::fmt::Display for Hash512 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl std::hash::Hash for Hash512 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl AsRef<[u8]> for Hash512 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// 签名类型（Ed25519 和 SM2 均为 64 字节）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signature(pub [u8; 64]);

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        hex_serde::serialize::<64, S>(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Signature(hex_serde::deserialize::<64, D>(deserializer)?))
    }
}

impl std::hash::Hash for Signature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for Hash256 {
    type Target = [u8; 32];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Hash256 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<[u8; 32]> for Hash256 {
    fn from(arr: [u8; 32]) -> Self {
        Self(arr)
    }
}

impl Deref for Hash512 {
    type Target = [u8; 64];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Hash512 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<[u8; 64]> for Hash512 {
    fn from(arr: [u8; 64]) -> Self {
        Self(arr)
    }
}

impl Deref for Signature {
    type Target = [u8; 64];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Signature {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<[u8; 64]> for Signature {
    fn from(arr: [u8; 64]) -> Self {
        Self(arr)
    }
}

/// 公钥类型（目前仅支持 Ed25519 32 字节）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PublicKey {
    /// Ed25519 公钥（32 字节）
    #[serde(with = "serde_big_array::BigArray")]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SecretKey {
    /// Ed25519 私钥（64 字节，包含 seed+public）
    #[serde(with = "serde_big_array::BigArray")]
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

/// 货币 ID
pub type CurrencyId = u64;
/// 转账 ID
pub type TransferId = u64;
/// 订单 ID
pub type OrderId = u64;
/// 别名 ID
pub type AliasId = u64;
/// 别名报价 ID
pub type AliasOfferId = u64;

pub use crate::transaction::TransactionType;

/// 交易收据（Transaction Receipt）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_type_conversion() {
        assert_eq!(u8::from(TransactionType::Payment), 0);
        assert_eq!(TransactionType::from_type(0), TransactionType::Payment);
        assert_eq!(TransactionType::from_type(255), TransactionType::Unknown);
    }
    
    #[test]
    fn test_constants_from_module() {
        use crate::constants::{BLOCK_VERSION, TRANSACTION_VERSION};
        assert_eq!(BLOCK_VERSION, 3);
        assert_eq!(TRANSACTION_VERSION, 1);
    }
}
