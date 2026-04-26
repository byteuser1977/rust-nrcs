//! Repository pattern for database access (based on migrations schema)
//!
//! Provides traits and implementations for CRUD operations on blockchain entities.
//! Uses async/await with SQLx and connection pooling.

use async_trait::async_trait;
use thiserror::Error;

use crate::models::*;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("duplicate key: {0}")]
    DuplicateKey(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("blockchain error: {0}")]
    Blockchain(#[from] blockchain_types::BlockchainError),
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;

#[async_trait]
pub trait Repository<T>: Send + Sync {
    async fn insert(&self, item: &T) -> RepositoryResult<()>;
    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<T>>;
    async fn update(&self, item: &T) -> RepositoryResult<()>;
    async fn delete(&self, db_id: i64) -> RepositoryResult<()>;
    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<T>>;
    async fn count(&self) -> RepositoryResult<i64>;
}

#[async_trait]
pub trait BlockRepository: Repository<BlockModel> {
    async fn find_by_height(&self, height: i32) -> RepositoryResult<Option<BlockModel>>;
    async fn find_by_id_column(&self, id: i64) -> RepositoryResult<Option<BlockModel>>;
    async fn find_by_hash(&self, hash: &[u8]) -> RepositoryResult<Option<BlockModel>>;
    async fn find_latest(&self) -> RepositoryResult<Option<BlockModel>>;
    async fn find_range(&self, start_height: i32, end_height: i32) -> RepositoryResult<Vec<BlockModel>>;
    async fn find_by_generator(&self, generator_id: i64) -> RepositoryResult<Vec<BlockModel>>;
    async fn get_height(&self) -> RepositoryResult<i32>;
    async fn get_block_id_at_height(&self, height: i32) -> RepositoryResult<Option<i64>>;
    async fn has_block(&self, id: i64) -> RepositoryResult<bool>;
    async fn get_ids_after(&self, block_id: i64, limit: i32) -> RepositoryResult<Vec<i64>>;
    async fn update_next_block_id(&self, previous_block_id: i64, next_block_id: i64) -> RepositoryResult<()>;
}

#[async_trait]
pub trait TransactionRepository: Repository<TransactionModel> {
    async fn find_by_txid(&self, id: i64) -> RepositoryResult<Option<TransactionModel>>;
    async fn find_by_full_hash(&self, full_hash: &[u8]) -> RepositoryResult<Option<TransactionModel>>;
    async fn find_by_sender(&self, sender_id: i64, limit: i64) -> RepositoryResult<Vec<TransactionModel>>;
    async fn find_by_recipient(&self, recipient_id: i64, limit: i64) -> RepositoryResult<Vec<TransactionModel>>;
    async fn find_by_block(&self, block_id: i64) -> RepositoryResult<Vec<TransactionModel>>;
    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<TransactionModel>>;
    async fn find_unconfirmed(&self, limit: i64) -> RepositoryResult<Vec<TransactionModel>>;
}

#[async_trait]
pub trait AccountRepository: Repository<AccountModel> {
    async fn find_by_account_id(&self, id: i64) -> RepositoryResult<Option<AccountModel>>;
    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<AccountModel>>;
    async fn find_latest_by_id(&self, id: i64) -> RepositoryResult<Option<AccountModel>>;
    async fn find_by_address(&self, address: &str) -> RepositoryResult<Option<AccountModel>>;
    async fn update_balance(&self, account_id: i64, balance: i64, unconfirmed_balance: i64) -> RepositoryResult<()>;
}

#[async_trait]
pub trait AccountAssetRepository: Repository<AccountAssetModel> {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<AccountAssetModel>>;
    async fn find_by_asset(&self, asset_id: i64) -> RepositoryResult<Vec<AccountAssetModel>>;
    async fn find_by_account_and_asset(&self, account_id: i64, asset_id: i64) -> RepositoryResult<Option<AccountAssetModel>>;
    async fn update_quantity(&self, account_id: i64, asset_id: i64, quantity: i64, height: i32) -> RepositoryResult<()>;
    async fn increase_quantity(&self, account_id: i64, asset_id: i64, delta: i64) -> RepositoryResult<()>;
    async fn decrease_quantity(&self, account_id: i64, asset_id: i64, delta: i64) -> RepositoryResult<()>;
}

#[async_trait]
pub trait AssetRepository: Repository<AssetModel> {
    async fn find_by_asset_id(&self, id: i64) -> RepositoryResult<Option<AssetModel>>;
    async fn find_by_owner(&self, owner_id: i64) -> RepositoryResult<Vec<AssetModel>>;
    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<AssetModel>>;
    async fn find_tradable(&self, limit: i64) -> RepositoryResult<Vec<AssetModel>>;
}

