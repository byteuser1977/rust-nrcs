//! Account Management Module
//!
//! 负责：
//! - 账户创建（私钥生成、公钥哈希作为 ID）
//! - 余额管理（查询、转账、原子操作）
//! - Nonce 管理（防止重放攻击）
//! - 资产增发/回收（仅 admin）

pub mod manager;
pub mod repository;
pub mod crypto;

// 定义 AccountAsset 结构体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountAsset {
    pub asset_id: blockchain_types::AssetId,
    pub quantity: blockchain_types::Amount,
    pub unconfirmed_quantity: blockchain_types::Amount,
}

pub use manager::{AccountManager, AccountConfig};
pub use repository::AccountStore;
pub use crypto::{generate_keypair, AddressGenerator};