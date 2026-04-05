//! Repository pattern for database access (based on migrations schema)
//!
//! Provides traits and implementations for CRUD operations on blockchain entities.
//! Uses async/await with SQLx and connection pooling.

use async_trait::async_trait;
use sqlx::{PgPool, Postgres};
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
}

