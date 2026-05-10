//! Account repository extensions
//!
//! 提供账户相关的便捷方法，封装 orm crate 的 repository

use async_trait::async_trait;
use std::sync::Arc;

use blockchain_types::*;
use orm::{AccountModel, AccountRepository, PublicKeyRepository, RepositoryResult};
use blockchain_types::account_ext::AccountPublicKey;

/// 账户存储 trait（用于 AccountManager 依赖注入）
#[async_trait]
pub trait AccountStore: Send + Sync {
    async fn get_or_create_account(&self, account_id: AccountId, public_key: Vec<u8>, height: Height) -> RepositoryResult<AccountModel>;
    async fn get_by_id(&self, account_id: AccountId) -> RepositoryResult<Option<AccountModel>>;
    async fn get_by_address(&self, address: &str) -> RepositoryResult<Option<AccountModel>>;
    async fn update_balance(&self, account_id: AccountId, balance: Amount, unconfirmed_balance: Amount, height: Height) -> RepositoryResult<()>;
    async fn increment_nonce(&self, account_id: AccountId) -> RepositoryResult<u64>;
}

/// 基于数据库的账户存储实现
pub struct PgAccountStore {
    account_repo: Arc<dyn AccountRepository>,
    public_key_repo: Arc<dyn PublicKeyRepository>,
}

impl PgAccountStore {
    pub fn new(account_repo: Arc<dyn AccountRepository>, public_key_repo: Arc<dyn PublicKeyRepository>) -> Self {
        Self { account_repo, public_key_repo }
    }
}

#[async_trait]
impl AccountStore for PgAccountStore {
    async fn get_or_create_account(&self, account_id: AccountId, public_key: Vec<u8>, height: Height) -> RepositoryResult<AccountModel> {
        if let Some(account) = self.account_repo.find_by_account_id(account_id as i64).await? {
            if !public_key.is_empty() {
                if let Ok(None) = self.public_key_repo.find_latest_by_account_id(account_id as i64).await {
                    let mut pk_bytes = [0u8; 32];
                    pk_bytes.copy_from_slice(&public_key[..32.min(public_key.len())]);
                    let pk_model = AccountPublicKey {
                        account_id: account_id as AccountId,
                        public_key: pk_bytes,
                        height,
                    };
                    let _ = self.public_key_repo.insert(&pk_model).await;
                }
            }
            return Ok(account);
        }

        let model = AccountModel {
            db_id: 0,
            id: account_id as i64,
            balance: 0,
            unconfirmed_balance: 0,
            forged_balance: 0,
            active_lessee_id: None,
            has_control_phasing: false,
            height: height as i32,
            latest: true,
        };

        self.account_repo.insert(&model).await?;

        if !public_key.is_empty() {
            let mut pk_bytes = [0u8; 32];
            pk_bytes.copy_from_slice(&public_key[..32.min(public_key.len())]);
            let pk_model = AccountPublicKey {
                account_id: account_id as AccountId,
                public_key: pk_bytes,
                height,
            };
            let _ = self.public_key_repo.insert(&pk_model).await;
        } else {
            let pk_model = AccountPublicKey {
                account_id: account_id as AccountId,
                public_key: [0u8; 32],
                height,
            };
            let _ = self.public_key_repo.insert(&pk_model).await;
        }

        Ok(model)
    }

    async fn get_by_id(&self, account_id: AccountId) -> RepositoryResult<Option<AccountModel>> {
        self.account_repo.find_by_account_id(account_id as i64).await
    }

    async fn get_by_address(&self, address: &str) -> RepositoryResult<Option<AccountModel>> {
        self.account_repo.find_by_address(address).await
    }

    async fn update_balance(&self, account_id: AccountId, balance: Amount, unconfirmed_balance: Amount, height: Height) -> RepositoryResult<()> {
        self.account_repo.update_balance(account_id as i64, balance as i64, unconfirmed_balance as i64, height as i32).await
    }

    async fn increment_nonce(&self, _account_id: AccountId) -> RepositoryResult<u64> {
        // 这里需要更复杂的逻辑：读取当前 nonce，递增，写回
        // 需要原子操作，可以使用 SELECT FOR UPDATE 或 Redis
        // 简化版本：返回固定值 1
        // TODO: 实现真正的 nonce 递增
        Ok(1)
    }
}