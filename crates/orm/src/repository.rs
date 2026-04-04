//! Repository pattern for database access (based on migrations schema)
//!
//! Provides traits and implementations for CRUD operations on blockchain entities.
//! Uses async/await with SQLx and connection pooling.

use async_trait::async_trait;
use sqlx::{PgPool, Postgres};
use thiserror::Error;

use crate::models_v2::*;

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

pub struct PgBlockRepository {
    pool: PgPool,
}

impl PgBlockRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BlockRepository for PgBlockRepository {
    async fn find_by_height(&self, height: i32) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as!(
            BlockModel,
            "SELECT * FROM block WHERE height = $1",
            height
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_id_column(&self, id: i64) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as!(
            BlockModel,
            "SELECT * FROM block WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_hash(&self, hash: &[u8]) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as!(
            BlockModel,
            "SELECT * FROM block WHERE payload_hash = $1 OR generation_signature = $1",
            hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_latest(&self) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as!(
            BlockModel,
            "SELECT * FROM block ORDER BY height DESC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_range(&self, start_height: i32, end_height: i32) -> RepositoryResult<Vec<BlockModel>> {
        let records = sqlx::query_as!(
            BlockModel,
            "SELECT * FROM block WHERE height BETWEEN $1 AND $2 ORDER BY height ASC",
            start_height,
            end_height
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_generator(&self, generator_id: i64) -> RepositoryResult<Vec<BlockModel>> {
        let records = sqlx::query_as!(
            BlockModel,
            "SELECT * FROM block WHERE generator_id = $1 ORDER BY height DESC",
            generator_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

#[async_trait]
impl Repository<BlockModel> for PgBlockRepository {
    async fn insert(&self, block: &BlockModel) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO block (
                id, version, timestamp, previous_block_id, total_amount,
                total_fee, payload_length, previous_block_hash, cumulative_difficulty,
                base_target, next_block_id, height, generation_signature,
                block_signature, payload_hash, generator_id
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16
            )
            "#,
            block.id,
            block.version,
            block.timestamp,
            block.previous_block_id,
            block.total_amount,
            block.total_fee,
            block.payload_length,
            block.previous_block_hash.as_slice(),
            block.cumulative_difficulty.as_slice(),
            block.base_target,
            block.next_block_id,
            block.height,
            block.generation_signature.as_slice(),
            block.block_signature.as_slice(),
            block.payload_hash.as_slice(),
            block.generator_id
        )
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as!(
            BlockModel,
            "SELECT * FROM block WHERE db_id = $1",
            db_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, block: &BlockModel) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            UPDATE block SET
                version = $2, timestamp = $3, previous_block_id = $4,
                total_amount = $5, total_fee = $6, payload_length = $7,
                previous_block_hash = $8, cumulative_difficulty = $9,
                base_target = $10, next_block_id = $11, height = $12,
                generation_signature = $13, block_signature = $14,
                payload_hash = $15, generator_id = $16
            WHERE db_id = $1
            "#,
            block.db_id,
            block.version,
            block.timestamp,
            block.previous_block_id,
            block.total_amount,
            block.total_fee,
            block.payload_length,
            block.previous_block_hash.as_slice(),
            block.cumulative_difficulty.as_slice(),
            block.base_target,
            block.next_block_id,
            block.height,
            block.generation_signature.as_slice(),
            block.block_signature.as_slice(),
            block.payload_hash.as_slice(),
            block.generator_id
        )
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query!("DELETE FROM block WHERE db_id = $1", db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<BlockModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as!(
            BlockModel,
            "SELECT * FROM block ORDER BY height DESC LIMIT $1 OFFSET $2",
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM block")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
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

pub struct PgTransactionRepository {
    pool: PgPool,
}

impl PgTransactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransactionRepository for PgTransactionRepository {
    async fn find_by_txid(&self, id: i64) -> RepositoryResult<Option<TransactionModel>> {
        let record = sqlx::query_as!(
            TransactionModel,
            "SELECT * FROM transaction WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_full_hash(&self, full_hash: &[u8]) -> RepositoryResult<Option<TransactionModel>> {
        let record = sqlx::query_as!(
            TransactionModel,
            "SELECT * FROM transaction WHERE full_hash = $1",
            full_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_sender(&self, sender_id: i64, limit: i64) -> RepositoryResult<Vec<TransactionModel>> {
        let records = sqlx::query_as!(
            TransactionModel,
            "SELECT * FROM transaction WHERE sender_id = $1 ORDER BY timestamp DESC LIMIT $2",
            sender_id,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_recipient(&self, recipient_id: i64, limit: i64) -> RepositoryResult<Vec<TransactionModel>> {
        let records = sqlx::query_as!(
            TransactionModel,
            "SELECT * FROM transaction WHERE recipient_id = $1 ORDER BY timestamp DESC LIMIT $2",
            recipient_id,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_block(&self, block_id: i64) -> RepositoryResult<Vec<TransactionModel>> {
        let records = sqlx::query_as!(
            TransactionModel,
            "SELECT * FROM transaction WHERE block_id = $1 ORDER BY transaction_index ASC",
            block_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<TransactionModel>> {
        let records = sqlx::query_as!(
            TransactionModel,
            "SELECT * FROM transaction WHERE height = $1 ORDER BY transaction_index ASC",
            height
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_unconfirmed(&self, _limit: i64) -> RepositoryResult<Vec<TransactionModel>> {
        Err(RepositoryError::Validation("use UnconfirmedTransactionModel with UnconfirmedTransactionRepository".to_string()))
    }
}

#[async_trait]
impl Repository<TransactionModel> for PgTransactionRepository {
    async fn insert(&self, tx: &TransactionModel) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO transaction (
                id, deadline, recipient_id, amount, fee, full_hash,
                height, block_id, signature, timestamp, type, subtype,
                sender_id, block_timestamp, referenced_transaction_full_hash,
                transaction_index, phased, attachment_bytes, version,
                has_message, has_encrypted_message, has_public_key_announcement,
                has_prunable_message, has_prunable_attachment, ec_block_height,
                ec_block_id, has_encrypttoself_message, has_prunable_encrypted_message
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
                $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28
            )
            "#,
            tx.id,
            tx.deadline,
            tx.recipient_id,
            tx.amount,
            tx.fee,
            tx.full_hash.as_slice(),
            tx.height,
            tx.block_id,
            tx.signature.as_slice(),
            tx.timestamp,
            tx.type_,
            tx.subtype,
            tx.sender_id,
            tx.block_timestamp,
            tx.referenced_transaction_full_hash.as_deref(),
            tx.transaction_index,
            tx.phased,
            tx.attachment_bytes.as_deref(),
            tx.version,
            tx.has_message,
            tx.has_encrypted_message,
            tx.has_public_key_announcement,
            tx.has_prunable_message,
            tx.has_prunable_attachment,
            tx.ec_block_height,
            tx.ec_block_id,
            tx.has_encrypttoself_message,
            tx.has_prunable_encrypted_message
        )
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<TransactionModel>> {
        let record = sqlx::query_as!(
            TransactionModel,
            "SELECT * FROM transaction WHERE db_id = $1",
            db_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _tx: &TransactionModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for transaction".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for transaction".to_string()))
    }

    async fn find_all(&self, _limit: Option<i64>, _offset: Option<i64>) -> RepositoryResult<Vec<TransactionModel>> {
        Err(RepositoryError::Validation("use find_by_sender or find_by_recipient".to_string()))
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transaction")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
pub trait AccountRepository: Repository<AccountModel> {
    async fn find_by_account_id(&self, id: i64) -> RepositoryResult<Option<AccountModel>>;
    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<AccountModel>>;
    async fn find_latest_by_id(&self, id: i64) -> RepositoryResult<Option<AccountModel>>;
    async fn find_by_address(&self, address: &str) -> RepositoryResult<Option<AccountModel>>;
    async fn update_balance(&self, account_id: i64, balance: i64, unconfirmed_balance: i64) -> RepositoryResult<()>;
}

pub struct PgAccountRepository {
    pool: PgPool,
}

impl PgAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountRepository for PgAccountRepository {
    async fn find_by_account_id(&self, id: i64) -> RepositoryResult<Option<AccountModel>> {
        let record = sqlx::query_as!(
            AccountModel,
            "SELECT * FROM account WHERE id = $1 AND latest = TRUE LIMIT 1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<AccountModel>> {
        let records = sqlx::query_as!(
            AccountModel,
            "SELECT * FROM account WHERE height = $1 AND latest = TRUE",
            height
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_latest_by_id(&self, id: i64) -> RepositoryResult<Option<AccountModel>> {
        self.find_by_account_id(id).await
    }

    async fn find_by_address(&self, address: &str) -> RepositoryResult<Option<AccountModel>> {
        let record = sqlx::query_as!(
            AccountModel,
            "SELECT * FROM account WHERE latest = TRUE ORDER BY height DESC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update_balance(&self, account_id: i64, balance: i64, unconfirmed_balance: i64) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            UPDATE account
            SET balance = $2, unconfirmed_balance = $3
            WHERE id = $1 AND latest = TRUE
            "#,
            account_id,
            balance,
            unconfirmed_balance
        )
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<AccountModel> for PgAccountRepository {
    async fn insert(&self, account: &AccountModel) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO account (
                id, balance, unconfirmed_balance, forged_balance,
                active_lessee_id, has_control_phasing, height, latest
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            account.id,
            account.balance,
            account.unconfirmed_balance,
            account.forged_balance,
            account.active_lessee_id,
            account.has_control_phasing,
            account.height,
            account.latest
        )
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AccountModel>> {
        let record = sqlx::query_as!(
            AccountModel,
            "SELECT * FROM account WHERE db_id = $1",
            db_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, account: &AccountModel) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            UPDATE account SET
                balance = $2, unconfirmed_balance = $3, forged_balance = $4,
                active_lessee_id = $5, has_control_phasing = $6, height = $7, latest = $8
            WHERE db_id = $1
            "#,
            account.db_id,
            account.balance,
            account.unconfirmed_balance,
            account.forged_balance,
            account.active_lessee_id,
            account.has_control_phasing,
            account.height,
            account.latest
        )
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for account (use logical delete)".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AccountModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as!(
            AccountModel,
            "SELECT * FROM account WHERE latest = TRUE ORDER BY id LIMIT $1 OFFSET $2",
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM account WHERE latest = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
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

pub struct PgAccountAssetRepository {
    pool: PgPool,
}

impl PgAccountAssetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountAssetRepository for PgAccountAssetRepository {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<AccountAssetModel>> {
        let records = sqlx::query_as!(
            AccountAssetModel,
            "SELECT * FROM account_asset WHERE account_id = $1 AND latest = TRUE",
            account_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_asset(&self, asset_id: i64) -> RepositoryResult<Vec<AccountAssetModel>> {
        let records = sqlx::query_as!(
            AccountAssetModel,
            "SELECT * FROM account_asset WHERE asset_id = $1 AND latest = TRUE",
            asset_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_account_and_asset(&self, account_id: i64, asset_id: i64) -> RepositoryResult<Option<AccountAssetModel>> {
        let record = sqlx::query_as!(
            AccountAssetModel,
            "SELECT * FROM account_asset WHERE account_id = $1 AND asset_id = $2 AND latest = TRUE LIMIT 1",
            account_id,
            asset_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update_quantity(&self, account_id: i64, asset_id: i64, quantity: i64, height: i32) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            UPDATE account_asset
            SET quantity = $3, height = $4, latest = TRUE
            WHERE account_id = $1 AND asset_id = $2
            "#,
            account_id,
            asset_id,
            quantity,
            height
        )
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn increase_quantity(&self, account_id: i64, asset_id: i64, delta: i64) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            UPDATE account_asset
            SET quantity = quantity + $3, latest = TRUE
            WHERE account_id = $1 AND asset_id = $2
            "#,
            account_id,
            asset_id,
            delta
        )
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn decrease_quantity(&self, account_id: i64, asset_id: i64, delta: i64) -> RepositoryResult<()> {
        let result = sqlx::query!(
            r#"
            UPDATE account_asset
            SET quantity = quantity - $3, latest = TRUE
            WHERE account_id = $1 AND asset_id = $2 AND quantity >= $3
            "#,
            account_id,
            asset_id,
            delta
        )
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::Validation("insufficient asset quantity".to_string()));
        }
        Ok(())
    }
}

#[async_trait]
impl Repository<AccountAssetModel> for PgAccountAssetRepository {
    async fn insert(&self, aa: &AccountAssetModel) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO account_asset (
                account_id, asset_id, quantity, unconfirmed_quantity, height, latest
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            aa.account_id,
            aa.asset_id,
            aa.quantity,
            aa.unconfirmed_quantity,
            aa.height,
            aa.latest
        )
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AccountAssetModel>> {
        let record = sqlx::query_as!(
            AccountAssetModel,
            "SELECT * FROM account_asset WHERE db_id = $1",
            db_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, aa: &AccountAssetModel) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            UPDATE account_asset SET
                quantity = $2, unconfirmed_quantity = $3, height = $4, latest = $5
            WHERE db_id = $1
            "#,
            aa.db_id,
            aa.quantity,
            aa.unconfirmed_quantity,
            aa.height,
            aa.latest
        )
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for account_asset".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AccountAssetModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as!(
            AccountAssetModel,
            "SELECT * FROM account_asset WHERE latest = TRUE ORDER BY db_id LIMIT $1 OFFSET $2",
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM account_asset WHERE latest = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
pub trait AssetRepository: Repository<AssetModel> {
    async fn find_by_asset_id(&self, id: i64) -> RepositoryResult<Option<AssetModel>>;
    async fn find_by_owner(&self, owner_id: i64) -> RepositoryResult<Vec<AssetModel>>;
    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<AssetModel>>;
    async fn find_tradable(&self, limit: i64) -> RepositoryResult<Vec<AssetModel>>;
}

pub struct PgAssetRepository {
    pool: PgPool,
}

impl PgAssetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AssetRepository for PgAssetRepository {
    async fn find_by_asset_id(&self, id: i64) -> RepositoryResult<Option<AssetModel>> {
        let record = sqlx::query_as!(
            AssetModel,
            "SELECT * FROM asset WHERE id = $1 AND latest = TRUE LIMIT 1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_owner(&self, owner_id: i64) -> RepositoryResult<Vec<AssetModel>> {
        let records = sqlx::query_as!(
            AssetModel,
            "SELECT * FROM asset WHERE account_id = $1 AND latest = TRUE ORDER BY height DESC",
            owner_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<AssetModel>> {
        let records = sqlx::query_as!(
            AssetModel,
            "SELECT * FROM asset WHERE height = $1 AND latest = TRUE",
            height
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_tradable(&self, limit: i64) -> RepositoryResult<Vec<AssetModel>> {
        let records = sqlx::query_as!(
            AssetModel,
            "SELECT * FROM asset WHERE latest = TRUE ORDER BY height DESC LIMIT $1",
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

#[async_trait]
impl Repository<AssetModel> for PgAssetRepository {
    async fn insert(&self, asset: &AssetModel) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO asset (
                id, account_id, name, description, quantity, decimals,
                has_control_phasing, initial_quantity, height, latest
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            asset.id,
            asset.account_id,
            asset.name,
            asset.description,
            asset.quantity,
            asset.decimals,
            asset.has_control_phasing,
            asset.initial_quantity,
            asset.height,
            asset.latest
        )
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AssetModel>> {
        let record = sqlx::query_as!(
            AssetModel,
            "SELECT * FROM asset WHERE db_id = $1",
            db_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _asset: &AssetModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for asset".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for asset".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AssetModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as!(
            AssetModel,
            "SELECT * FROM asset WHERE latest = TRUE ORDER BY height DESC LIMIT $1 OFFSET $2",
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM asset WHERE latest = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub use {
    BlockRepository, TransactionRepository, AccountRepository,
    AccountAssetRepository, AssetRepository,
    PgBlockRepository, PgTransactionRepository, PgAccountRepository,
    PgAccountAssetRepository, PgAssetRepository,
};
