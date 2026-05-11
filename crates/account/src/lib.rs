//! Account Management Module
//!
//! 负责：
//! - 账户创建（私钥生成、公钥哈希作为 ID）
//! - 余额管理（查询、转账、原子操作）
//! - Nonce 管理（防止重放攻击）
//! - 资产增发/回收（仅 admin）
//!
//! AccountService 对照 Java NRCS Account 类实现完整的账户服务抽象，
//! 提供双余额模型、资产/货币余额操作、担保余额机制、账户租赁、
//! 事件通知、分类账记录等功能。

pub mod manager;
pub mod repository;
pub mod crypto;
pub mod service;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountAsset {
    pub asset_id: blockchain_types::AssetId,
    pub quantity: blockchain_types::Amount,
    pub unconfirmed_quantity: blockchain_types::Amount,
}

pub use manager::{AccountManager, AccountConfig, AccountError, AccountResult, DatabaseAccountManager};
pub use repository::{AccountStore, PgAccountStore};
pub use crypto::{generate_keypair, AddressGenerator};
pub use service::{
    AccountService, DatabaseAccountService, AccountServiceError, AccountServiceResult,
    AccountEvent, AccountBalanceChange, AccountEventListener,
    LedgerHolding, LedgerEvent, LedgerEntry, AccountLeaseInfo,
};