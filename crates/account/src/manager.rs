//! Account Manager
//!
//! 核心账户管理服务：
//! - 创建账户（生成密钥、存储）
//! - 查询余额、资产
//! - 原子转账操作
//! - Nonce 管理（防重放）
//! - 资产增发/回收（admin）

use async_trait::async_trait;
use std::sync::Arc;
use thiserror::Error;

use blockchain_types::*;
use orm::{AccountRepository, AccountAssetRepository, RepositoryError};

use crate::{AccountStore, crypto::AddressGenerator};

/// 账户管理器配置
#[derive(Debug, Clone)]
pub struct AccountConfig {
    /// 是否启用地址生成（true=Base58 地址，false=仅用 account_id）
    pub enable_address: bool,
    /// 初始余额（新账户）
    pub initial_balance: Amount,
    /// Admin 账户 ID（用于资产增发/回收）
    pub admin_account_id: Option<AccountId>,
}

impl Default for AccountConfig {
    fn default() -> Self {
        Self {
            enable_address: true,
            initial_balance: 0,
            admin_account_id: None,
        }
    }
}

/// 账户错误类型
#[derive(Debug, Error)]
pub enum AccountError {
    #[error("account not found: {0}")]
    NotFound(AccountId),

    #[error("insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: Amount, need: Amount },

    #[error("invalid operation: {0}")]
    InvalidOperation(String),

    #[error("duplicate account: {0}")]
    DuplicateAccount(AccountId),

    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type AccountResult<T> = Result<T, AccountError>;

/// AccountManager trait
///
/// 提供账户管理的基本操作
#[async_trait]
pub trait AccountManager: Send + Sync {
    /// 创建新账户（生成密钥对）
    async fn create_account(&self, initial_balance: Option<Amount>) -> AccountResult<(Keypair, AccountId, String)>;

    /// 创建新账户（仅注册，不生成密钥）
    async fn register_account(&self, account_id: AccountId, public_key: Vec<u8>) -> AccountResult<()>;

    /// 获取账户余额
    async fn get_balance(&self, account_id: AccountId) -> AccountResult<Amount>;

    /// 获取账户完整信息（余额、资产）
    async fn get_account_info(&self, account_id: AccountId) -> AccountResult<Account>;

    /// 向账户转账（原子操作）
    async fn transfer(&self, from: AccountId, to: AccountId, amount: Amount) -> AccountResult<()>;

    /// 增加余额（ Credit ）
    async fn credit(&self, account_id: AccountId, amount: Amount) -> AccountResult<()>;

    /// 减少余额（Debit）
    async fn debit(&self, account_id: AccountId, amount: Amount) -> AccountResult<()>;

    /// 获取并递增发送者 nonce
    async fn get_and_increment_nonce(&self, sender_id: AccountId) -> AccountResult<u64>;

    /// 查询当前 nonce（不递增）
    async fn current_nonce(&self, account_id: AccountId) -> AccountResult<u64>;

    /// 增发资产（仅 admin）
    async fn mint_asset(&self, asset_id: AssetId, to: AccountId, amount: Amount) -> AccountResult<()>;

    /// 回收资产（仅 admin）
    async fn burn_asset(&self, asset_id: AssetId, from: AccountId, amount: Amount) -> AccountResult<()>;
}

/// Database-backed AccountManager implementation
pub struct DatabaseAccountManager {
    store: Arc<dyn AccountStore>,
    account_repo: Arc<dyn AccountRepository>,
    account_asset_repo: Arc<dyn AccountAssetRepository>,
    config: AccountConfig,
}

impl DatabaseAccountManager {
    pub fn new(
        store: Arc<dyn AccountStore>,
        account_repo: Arc<dyn AccountRepository>,
        account_asset_repo: Arc<dyn AccountAssetRepository>,
        config: AccountConfig,
    ) -> Self {
        Self {
            store,
            account_repo,
            account_asset_repo,
            config,
        }
    }

    /// 获取账户领域对象（带余额、资产等完整状态）
    async fn get_account_domain(&self, account_id: AccountId) -> AccountResult<Account> {
        // 获取数据库模型
        let model = self.store.get_by_id(account_id).await?
            .ok_or_else(|| AccountError::NotFound(account_id))?;

        // 转换为领域对象
        let mut account = model.to_domain().map_err(|e| AccountError::Repository(RepositoryError::Blockchain(e)))?;

        // 加载资产持仓
        let asset_models = self.account_asset_repo.find_by_account(account_id as i64).await?;
        for am in asset_models {
            let aa = am.to_domain().map_err(|e| AccountError::Repository(RepositoryError::Blockchain(e)))?;
            account.assets.insert(aa.asset_id, aa.quantity);
        }

        Ok(account)
    }
}

#[async_trait]
impl AccountManager for DatabaseAccountManager {
    async fn create_account(&self, initial_balance: Option<Amount>) -> AccountResult<(Keypair, AccountId, String)> {
        let (kp, account_id, address) = {
            let generator = ();
            generator.generate_account()
        };

        // 存储到数据库
        let public_key_bytes = kp.public.as_bytes().to_vec();
        self.store.get_or_create_account(account_id, public_key_bytes).await?;

        // 如果配置了初始余额，进行 credit
        if let Some(balance) = initial_balance {
            self.credit(account_id, balance).await?;
        }

        Ok((kp, account_id, address))
    }

    async fn register_account(&self, account_id: AccountId, public_key: Vec<u8>) -> AccountResult<()> {
        self.store.get_or_create_account(account_id, public_key).await?;
        Ok(())
    }

    async fn get_balance(&self, account_id: AccountId) -> AccountResult<Amount> {
        let account = self.get_account_domain(account_id).await?;
        Ok(account.balance)
    }

    async fn get_account_info(&self, account_id: AccountId) -> AccountResult<Account> {
        self.get_account_domain(account_id).await
    }

    async fn transfer(&self, from: AccountId, to: AccountId, amount: Amount) -> AccountResult<()> {
        if amount == 0 {
            return Err(AccountError::InvalidOperation("transfer amount cannot be zero".to_string()));
        }
        if from == to {
            return Err(AccountError::InvalidOperation("cannot transfer to self".to_string()));
        }

        // 查询发送方账户
        let from_account = self.get_account_domain(from).await?;
        if !from_account.has_balance(amount) {
            return Err(AccountError::InsufficientBalance {
                have: from_account.effective_balance(),
                need: amount,
            });
        }

        // 查询接收方账户
        let to_account = self.get_account_domain(to).await?;

        // 执行转账（都在一个事务中）
        // TODO: 实现事务边界
        let new_from_balance = from_account.balance.saturating_sub(amount);
        let new_from_unconfirmed = from_account.unconfirmed_balance.saturating_sub(amount);
        let new_to_balance = to_account.balance.saturating_add(amount);
        let new_to_unconfirmed = to_account.unconfirmed_balance.saturating_add(amount);

        self.store.update_balance(from, new_from_balance, new_from_unconfirmed).await?;
        self.store.update_balance(to, new_to_balance, new_to_unconfirmed).await?;

        Ok(())
    }

    async fn credit(&self, account_id: AccountId, amount: Amount) -> AccountResult<()> {
        if amount == 0 {
            return Ok(());
        }

        let account = self.get_account_domain(account_id).await?;
        let new_balance = account.balance.saturating_add(amount);
        let new_unconfirmed = account.unconfirmed_balance.saturating_add(amount);
        self.store.update_balance(account_id, new_balance, new_unconfirmed).await?;
        Ok(())
    }

    async fn debit(&self, account_id: AccountId, amount: Amount) -> AccountResult<()> {
        if amount == 0 {
            return Ok(());
        }

        let account = self.get_account_domain(account_id).await?;
        if !account.has_balance(amount) {
            return Err(AccountError::InsufficientBalance {
                have: account.effective_balance(),
                need: amount,
            });
        }

        let new_balance = account.balance.saturating_sub(amount);
        let new_unconfirmed = account.unconfirmed_balance.saturating_sub(amount);
        self.store.update_balance(account_id, new_balance, new_unconfirmed).await?;
        Ok(())
    }

    async fn get_and_increment_nonce(&self, sender_id: AccountId) -> AccountResult<u64> {
        self.store.increment_nonce(sender_id).await
    }

    async fn current_nonce(&self, account_id: AccountId) -> AccountResult<u64> {
        // TODO: 实现从数据库或缓存查询
        Ok(0) // placeholder
    }

    async fn mint_asset(&self, asset_id: AssetId, to: AccountId, amount: Amount) -> AccountResult<()> {
        // 检查是否为 admin
        if let Some(admin_id) = self.config.admin_account_id {
            if admin_id != to {
                return Err(AccountError::InvalidOperation("only admin can mint assets".to_string()));
            }
        }

        // 增加资产持仓
        self.account_asset_repo.increase_quantity(to as i64, asset_id as i64, amount as i64).await?;
        Ok(())
    }

    async fn burn_asset(&self, asset_id: AssetId, from: AccountId, amount: Amount) -> AccountResult<()> {
        // 检查是否为 admin
        if let Some(admin_id) = self.config.admin_account_id {
            if admin_id != from {
                return Err(AccountError::InvalidOperation("only admin can burn assets".to_string()));
            }
        }

        // 减少资产持仓
        self.account_asset_repo.decrease_quantity(from as i64, asset_id as i64, amount as i64).await?;
        Ok(())
    }
}