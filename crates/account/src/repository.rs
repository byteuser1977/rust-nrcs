//! Account repository extensions
//!
//! 提供账户相关的便捷方法，封装 orm crate 的 repository

use async_trait::async_trait;
use std::sync::Arc;

use blockchain_types::*;
use orm::{AccountModel, AccountRepository, RepositoryResult};
use blockchain_types::prelude::Account;

/// 账户存储 trait（用于 AccountManager 依赖注入）
#[async_trait]
pub trait AccountStore: Send + Sync {
    async fn get_or_create_account(&self, account_id: AccountId, public_key: Vec<u8>) -> RepositoryResult<AccountModel>;
    async fn get_by_id(&self, account_id: AccountId) -> RepositoryResult<Option<AccountModel>>;
    async fn get_by_address(&self, address: &str) -> RepositoryResult<Option<AccountModel>>;
    async fn update_balance(&self, account_id: AccountId, balance: Amount, unconfirmed_balance: Amount) -> RepositoryResult<()>;
    async fn increment_nonce(&self, account_id: AccountId) -> RepositoryResult<u64>;
}

/// 基于数据库的账户存储实现
pub struct PgAccountStore {
    account_repo: Arc<dyn AccountRepository>,
}

impl PgAccountStore {
    pub fn new(account_repo: Arc<dyn AccountRepository>) -> Self {
        Self { account_repo }
    }
}

#[async_trait]
impl AccountStore for PgAccountStore {
    async fn get_or_create_account(&self, account_id: AccountId, public_key: Vec<u8>) -> RepositoryResult<AccountModel> {
        // 尝试查询
        if let Some(account) = self.account_repo.find_by_account_id(account_id as i64).await? {
            // 如果已有公钥，直接返回
            // 注意：AccountModel 中没有 public_key 字段，这里暂时跳过更新
            if !public_key.is_empty() {
                // TODO: 考虑添加 public_key 字段到 AccountModel
            }
            return Ok(account);
        }

        // 创建新账户
        let account = Account::new(account_id, 0);
        // 注意：AccountModel 中没有 public_key 字段，这里暂时跳过设置
        // 直接创建 AccountModel 实例
        let model = AccountModel {
            db_id: 0,
            id: account.id as i64,
            balance: account.balance as i64,
            unconfirmed_balance: account.unconfirmed_balance as i64,
            forged_balance: 0,
            active_lessee_id: None,
            has_control_phasing: false,
            height: 0,
            latest: true,
        };

        self.account_repo.insert(&model).await?;
        Ok(model)
    }

    async fn get_by_id(&self, account_id: AccountId) -> RepositoryResult<Option<AccountModel>> {
        self.account_repo.find_by_account_id(account_id as i64).await
    }

    async fn get_by_address(&self, address: &str) -> RepositoryResult<Option<AccountModel>> {
        self.account_repo.find_by_address(address).await
    }

    async fn update_balance(&self, account_id: AccountId, balance: Amount, unconfirmed_balance: Amount) -> RepositoryResult<()> {
        self.account_repo.update_balance(account_id as i64, balance as i64, unconfirmed_balance as i64).await
    }

    async fn increment_nonce(&self, _account_id: AccountId) -> RepositoryResult<u64> {
        // 这里需要更复杂的逻辑：读取当前 nonce，递增，写回
        // 需要原子操作，可以使用 SELECT FOR UPDATE 或 Redis
        // 简化版本：返回固定值 1
        // TODO: 实现真正的 nonce 递增
        Ok(1)
    }
}