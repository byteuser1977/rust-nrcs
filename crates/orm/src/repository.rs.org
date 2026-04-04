//! Repository pattern for database access
//!
//! Provides traits and implementations for CRUD operations on blockchain entities.
//! Uses async/await with SQLx and connection pooling (bb8).

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use thiserror::Error;
use std::sync::Arc;

use crate::models::*;
use blockchain_types::*;

/// Repository errors
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
    Blockchain(#[from] BlockchainError),
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// Common repository trait
#[async_trait]
pub trait Repository<T>: Send + Sync {
    /// Insert a new record
    async fn insert(&self, item: &T) -> RepositoryResult<()>;

    /// Insert multiple records
    async fn insert_many(&self, items: &[T]) -> RepositoryResult<()>;

    /// Find by ID
    async fn find_by_id(&self, id: i64) -> RepositoryResult<Option<T>>;

    /// Update an existing record
    async fn update(&self, item: &T) -> RepositoryResult<()>;

    /// Delete by ID
    async fn delete(&self, id: i64) -> RepositoryResult<()>;

    /// Find all with pagination
    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<T>>;

    /// Count total records
    async fn count(&self) -> RepositoryResult<i64>;
}

/// Block repository
#[async_trait]
pub trait BlockRepository: Repository<BlockModel> {
    async fn find_by_height(&self, height: i64) -> RepositoryResult<Option<BlockModel>>;
    async fn find_by_hash(&self, hash: &[u8]) -> RepositoryResult<Option<BlockModel>>;
    async fn find_latest(&self) -> RepositoryResult<Option<BlockModel>>;
    async fn find_range(&self, start_height: i64, end_height: i64) -> RepositoryResult<Vec<BlockModel>>;
}

/// Transaction repository
#[async_trait]
pub trait TransactionRepository: Repository<TransactionModel> {
    async fn find_by_txid(&self, txid: &[u8]) -> RepositoryResult<Option<TransactionModel>>;
    async fn find_by_sender(&self, sender_id: i64, limit: i64) -> RepositoryResult<Vec<TransactionModel>>;
    async fn find_by_recipient(&self, recipient_id: i64, limit: i64) -> RepositoryResult<Vec<TransactionModel>>;
    async fn find_by_block(&self, block_id: i64) -> RepositoryResult<Vec<TransactionModel>>;
    async fn find_unconfirmed(&self, limit: i64) -> RepositoryResult<Vec<TransactionModel>>;
}

/// Account repository
#[async_trait]
pub trait AccountRepository: Repository<AccountModel> {
    async fn find_by_account_id(&self, account_id: i64) -> RepositoryResult<Option<AccountModel>>;
    async fn find_by_address(&self, address: &str) -> RepositoryResult<Option<AccountModel>>;
    async fn update_balance(&self, account_id: i64, balance: i64, unconfirmed_balance: i64) -> RepositoryResult<()>;
}

/// Asset repository
#[async_trait]
pub trait AssetRepository: Repository<AssetModel> {
    async fn find_by_asset_id(&self, asset_id: i64) -> RepositoryResult<Option<AssetModel>>;
    async fn find_by_owner(&self, owner_id: i64) -> RepositoryResult<Vec<AssetModel>>;
    async fn find_tradable(&self, limit: i64) -> RepositoryResult<Vec<AssetModel>>;
}

/// AccountAsset repository
#[async_trait]
pub trait AccountAssetRepository: Repository<AccountAssetModel> {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<AccountAssetModel>>;
    async fn find_by_asset(&self, asset_id: i64) -> RepositoryResult<Vec<AccountAssetModel>>;
    async fn increase_quantity(&self, account_id: i64, asset_id: i64, delta: i64) -> RepositoryResult<()>;
    async fn decrease_quantity(&self, account_id: i64, asset_id: i64, delta: i64) -> RepositoryResult<()>;
}

/// Contract repository
#[async_trait]
pub trait ContractRepository: Repository<ContractModel> {
    async fn find_by_address(&self, address: &[u8]) -> RepositoryResult<Option<ContractModel>>;
    async fn find_by_creator(&self, creator_id: i64) -> RepositoryResult<Vec<ContractModel>>;
}

/// Transaction receipt repository
#[async_trait]
pub trait TransactionReceiptRepository: Repository<TransactionReceiptModel> {
    async fn find_by_transaction(&self, txid: i64) -> RepositoryResult<Option<TransactionReceiptModel>>;
    async fn insert_with_status(&self, txid: i64, status: u8, gas_used: u64, logs: &str, contract_address: Option<&[u8]>) -> RepositoryResult<()>;
}

// ========== Implementations ==========

/// PostgreSQL implementation using SQLx
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
    async fn insert(&self, block: &BlockModel) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO blocks (
                height, block_hash, previous_block_hash, payload_hash,
                generator_id, nonce, base_target, cumulative_difficulty,
                total_amount, total_fee, payload_length, generation_signature,
                block_signature, version
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14
            )
            "#,
            block.height,
            block.block_hash.as_slice(),
            block.previous_block_hash.as_slice(),
            block.payload_hash.as_slice(),
            block.generator_id,
            block.nonce,
            block.base_target,
            block.cumulative_difficulty.as_slice(),
            block.total_amount,
            block.total_fee,
            block.payload_length,
            block.generation_signature.as_slice(),
            block.block_signature.as_slice(),
            block.version
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(())
    }

    async fn find_by_height(&self, height: i64) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as!(
            BlockModel,
            "SELECT * FROM blocks WHERE height = $1",
            height
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(record)
    }

    async fn find_by_hash(&self, hash: &[u8]) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as!(
            BlockModel,
            "SELECT * FROM blocks WHERE block_hash = $1",
            hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(record)
    }

    async fn find_latest(&self) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as!(
            BlockModel,
            "SELECT * FROM blocks ORDER BY height DESC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(record)
    }

    async fn find_range(&self, start_height: i64, end_height: i64) -> RepositoryResult<Vec<BlockModel>> {
        let records = sqlx::query_as!(
            BlockModel,
            "SELECT * FROM blocks WHERE height BETWEEN $1 AND $2 ORDER BY height ASC",
            start_height,
            end_height
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(records)
    }

    // Implement basic Repository trait (stub)
    async fn update(&self, _item: &BlockModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("not implemented".to_string()))
    }

    async fn delete(&self, _id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("not implemented".to_string()))
    }

    async fn find_all(&self, _limit: Option<i64>, _offset: Option<i64>) -> RepositoryResult<Vec<BlockModel>> {
        Err(RepositoryError::Validation("use find_range instead".to_string()))
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM blocks")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepositoryError::DbError(e))?;
        Ok(count.0)
    }
}

/// Transaction repository implementation
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
    async fn insert(&self, tx: &TransactionModel) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO transactions (
                transaction_id, full_hash, type_id, subtype, sender_id,
                recipient_id, amount, fee, block_id, height, timestamp,
                deadline, signature, attachment_bytes, phased, has_message,
                has_encrypted_message, has_public_key_announcement,
                has_prunable_attachment, ec_block_height, ec_block_id,
                has_encrypttoself_message, has_prunable_encrypted_message
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23
            )
            "#,
            tx.transaction_id,
            tx.full_hash.as_slice(),
            tx.type_id,
            tx.subtype,
            tx.sender_id,
            tx.recipient_id,
            tx.amount,
            tx.fee,
            tx.block_id,
            tx.height,
            tx.timestamp,
            tx.deadline,
            tx.signature.as_slice(),
            tx.attachment_bytes.as_slice(),
            tx.phased,
            tx.has_message,
            tx.has_encrypted_message,
            tx.has_public_key_announcement,
            tx.has_prunable_attachment,
            tx.ec_block_height,
            tx.ec_block_id,
            tx.has_encrypttoself_message,
            tx.has_prunable_encrypted_message
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(())
    }

    async fn find_by_txid(&self, txid: &[u8]) -> RepositoryResult<Option<TransactionModel>> {
        let record = sqlx::query_as!(
            TransactionModel,
            "SELECT * FROM transactions WHERE full_hash = $1",
            txid
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(record)
    }

    async fn find_by_sender(&self, sender_id: i64, limit: i64) -> RepositoryResult<Vec<TransactionModel>> {
        let records = sqlx::query_as!(
            TransactionModel,
            "SELECT * FROM transactions WHERE sender_id = $1 ORDER BY timestamp DESC LIMIT $2",
            sender_id,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(records)
    }

    async fn find_by_recipient(&self, recipient_id: i64, limit: i64) -> RepositoryResult<Vec<TransactionModel>> {
        let records = sqlx::query_as!(
            TransactionModel,
            "SELECT * FROM transactions WHERE recipient_id = $1 ORDER BY timestamp DESC LIMIT $2",
            recipient_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(records)
    }

    async fn find_by_block(&self, block_id: i64) -> RepositoryResult<Vec<TransactionModel>> {
        let records = sqlx::query_as!(
            TransactionModel,
            "SELECT * FROM transactions WHERE block_id = $1",
            block_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(records)
    }

    async fn find_unconfirmed(&self, limit: i64) -> RepositoryResult<Vec<TransactionModel>> {
        let records = sqlx::query_as!(
            TransactionModel,
            "SELECT * FROM transactions WHERE block_id IS NULL ORDER BY fee DESC, timestamp ASC LIMIT $1",
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(records)
    }

    // Stubs
    async fn update(&self, _item: &TransactionModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("not implemented".to_string()))
    }

    async fn delete(&self, _id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("not implemented".to_string()))
    }

    async fn insert_many(&self, _items: &[TransactionModel]) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("batch insert not implemented".to_string()))
    }

    async fn find_by_id(&self, _id: i64) -> RepositoryResult<Option<TransactionModel>> {
        Err(RepositoryError::Validation("use find_by_txid instead".to_string()))
    }

    async fn find_all(&self, _limit: Option<i64>, _offset: Option<i64>) -> RepositoryResult<Vec<TransactionModel>> {
        Err(RepositoryError::Validation("use other find methods".to_string()))
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transactions")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepositoryError::DbError(e))?;
        Ok(count.0)
    }
}

/// Account repository implementation
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
    async fn insert(&self, account: &AccountModel) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO accounts (
                account_id, address, public_key, balance, unconfirmed_balance,
                reserved_balance, guaranteed_balance, properties, lease, current_height
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            account.account_id,
            account.address,
            account.public_key.as_slice(),
            account.balance,
            account.unconfirmed_balance,
            account.reserved_balance,
            account.guaranteed_balance,
            &account.properties,
            &account.lease,
            account.current_height
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(())
    }

    async fn find_by_account_id(&self, account_id: i64) -> RepositoryResult<Option<AccountModel>> {
        let record = sqlx::query_as!(
            AccountModel,
            "SELECT * FROM accounts WHERE account_id = $1",
            account_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(record)
    }

    async fn find_by_address(&self, address: &str) -> RepositoryResult<Option<AccountModel>> {
        let record = sqlx::query_as!(
            AccountModel,
            "SELECT * FROM accounts WHERE address = $1",
            address
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(record)
    }

    async fn update_balance(&self, account_id: i64, balance: i64, unconfirmed_balance: i64) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            UPDATE accounts
            SET balance = $1, unconfirmed_balance = $2, last_updated = NOW()
            WHERE account_id = $3
            "#,
            balance,
            unconfirmed_balance,
            account_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(())
    }

    // Stubs
    async fn find_by_id(&self, _id: i64) -> RepositoryResult<Option<AccountModel>> {
        Err(RepositoryError::Validation("use find_by_account_id instead".to_string()))
    }

    async fn update(&self, _item: &AccountModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("use update_balance instead".to_string()))
    }

    async fn delete(&self, _id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("not implemented".to_string()))
    }

    async fn insert_many(&self, _items: &[AccountModel]) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("batch insert not implemented".to_string()))
    }

    async fn find_all(&self, _limit: Option<i64>, _offset: Option<i64>) -> RepositoryResult<Vec<AccountModel>> {
        Err(RepositoryError::Validation("not implemented".to_string()))
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM accounts")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepositoryError::DbError(e))?;
        Ok(count.0)
    }
}

/// Asset repository implementation
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
    async fn insert(&self, asset: &AssetModel) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO assets (
                asset_id, owner_id, name, description, quantity,
                decimals, mintable, transferable, data
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            asset.asset_id,
            asset.owner_id,
            asset.name,
            asset.description,
            asset.quantity,
            asset.decimals,
            asset.mintable,
            asset.transferable,
            asset.data.as_slice()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(())
    }

    async fn find_by_asset_id(&self, asset_id: i64) -> RepositoryResult<Option<AssetModel>> {
        let record = sqlx::query_as!(
            AssetModel,
            "SELECT * FROM assets WHERE asset_id = $1",
            asset_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(record)
    }

    async fn find_by_owner(&self, owner_id: i64) -> RepositoryResult<Vec<AssetModel>> {
        let records = sqlx::query_as!(
            AssetModel,
            "SELECT * FROM assets WHERE owner_id = $1 ORDER BY created_at DESC",
            owner_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(records)
    }

    async fn find_tradable(&self, limit: i64) -> RepositoryResult<Vec<AssetModel>> {
        let records = sqlx::query_as!(
            AssetModel,
            "SELECT * FROM assets WHERE deleted = FALSE AND transferable = TRUE ORDER BY created_at DESC LIMIT $1",
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(records)
    }

    // Stubs
    async fn find_by_id(&self, _id: i64) -> RepositoryResult<Option<AssetModel>> {
        Err(RepositoryError::Validation("use find_by_asset_id instead".to_string()))
    }

    async fn update(&self, _item: &AssetModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("not implemented".to_string()))
    }

    async fn delete(&self, _id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("not implemented".to_string()))
    }

    async fn insert_many(&self, _items: &[AssetModel]) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("batch insert not implemented".to_string()))
    }

    async fn find_all(&self, _limit: Option<i64>, _offset: Option<i64>) -> RepositoryResult<Vec<AssetModel>> {
        Err(RepositoryError::Validation("not implemented".to_string()))
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM assets WHERE deleted = FALSE")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepositoryError::DbError(e))?;
        Ok(count.0)
    }
}

/// AccountAsset repository implementation
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
    async fn insert(&self, aa: &AccountAssetModel) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO account_assets (account_id, asset_id, quantity)
            VALUES ($1, $2, $3)
            ON CONFLICT (account_id, asset_id) DO UPDATE
            SET quantity = EXCLUDED.quantity, last_updated = NOW()
            "#,
            aa.account_id,
            aa.asset_id,
            aa.quantity
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(())
    }

    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<AccountAssetModel>> {
        let records = sqlx::query_as!(
            AccountAssetModel,
            "SELECT * FROM account_assets WHERE account_id = $1",
            account_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(records)
    }

    async fn find_by_asset(&self, asset_id: i64) -> RepositoryResult<Vec<AccountAssetModel>> {
        let records = sqlx::query_as!(
            AccountAssetModel,
            "SELECT * FROM account_assets WHERE asset_id = $1",
            asset_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(records)
    }

    async fn increase_quantity(&self, account_id: i64, asset_id: i64, delta: i64) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO account_assets (account_id, asset_id, quantity)
            VALUES ($1, $2, $3)
            ON CONFLICT (account_id, asset_id) DO UPDATE
            SET quantity = account_assets.quantity + EXCLUDED.quantity, last_updated = NOW()
            "#,
            account_id,
            asset_id,
            delta
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(())
    }

    async fn decrease_quantity(&self, account_id: i64, asset_id: i64, delta: i64) -> RepositoryResult<()> {
        let result = sqlx::query!(
            r#"
            UPDATE account_assets
            SET quantity = quantity - $1, last_updated = NOW()
            WHERE account_id = $2 AND asset_id = $3 AND quantity >= $1
            "#,
            delta,
            account_id,
            asset_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::Validation("insufficient asset quantity".to_string()));
        }
        Ok(())
    }

    // Stubs
    async fn find_by_id(&self, _id: i64) -> RepositoryResult<Option<AccountAssetModel>> {
        Err(RepositoryError::Validation("not implemented".to_string()))
    }

    async fn update(&self, _item: &AccountAssetModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("use increase_quantity/decrease_quantity instead".to_string()))
    }

    async fn delete(&self, _id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("not implemented".to_string()))
    }

    async fn insert_many(&self, _items: &[AccountAssetModel]) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("batch insert not implemented".to_string()))
    }

    async fn find_all(&self, _limit: Option<i64>, _offset: Option<i64>) -> RepositoryResult<Vec<AccountAssetModel>> {
        Err(RepositoryError::Validation("not implemented".to_string()))
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM account_assets")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepositoryError::DbError(e))?;
        Ok(count.0)
    }
}

/// TransactionReceipt repository implementation
pub struct PgTransactionReceiptRepository {
    pool: PgPool,
}

impl PgTransactionReceiptRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransactionReceiptRepository for PgTransactionReceiptRepository {
    async fn insert(&self, receipt: &TransactionReceiptModel) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO transaction_receipts (transaction_id, status, gas_used, logs, contract_address)
            VALUES ($1, $2, $3, $4::jsonb, $5)
            "#,
            receipt.transaction_id,
            receipt.status,
            receipt.gas_used,
            receipt.logs.as_str(),
            receipt.contract_address.as_deref()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(())
    }

    async fn insert_with_status(
        &self,
        txid: i64,
        status: u8,
        gas_used: u64,
        logs: &str,
        contract_address: Option<&[u8]>,
    ) -> RepositoryResult<()> {
        let receipt = TransactionReceiptModel {
            id: 0,
            transaction_id: txid,
            status: status as i16,
            gas_used: gas_used as i64,
            logs: serde_json::from_str(logs).unwrap_or_default(),
            contract_address: contract_address.map(|v| v.to_vec()),
            executed_at: Utc::now(),
        };
        self.insert(&receipt).await
    }

    async fn find_by_transaction(&self, txid: i64) -> RepositoryResult<Option<TransactionReceiptModel>> {
        let record = sqlx::query_as!(
            TransactionReceiptModel,
            "SELECT * FROM transaction_receipts WHERE transaction_id = $1",
            txid
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::DbError(e))?;
        Ok(record)
    }

    // Stubs
    async fn find_by_id(&self, _id: i64) -> RepositoryResult<Option<TransactionReceiptModel>> {
        Err(RepositoryError::Validation("use find_by_transaction instead".to_string()))
    }

    async fn update(&self, _item: &TransactionReceiptModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("not implemented".to_string()))
    }

    async fn delete(&self, _id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("not implemented".to_string()))
    }

    async fn insert_many(&self, _items: &[TransactionReceiptModel]) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("batch insert not implemented".to_string()))
    }

    async fn find_all(&self, _limit: Option<i64>, _offset: Option<i64>) -> RepositoryResult<Vec<TransactionReceiptModel>> {
        Err(RepositoryError::Validation("not implemented".to_string()))
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transaction_receipts")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepositoryError::DbError(e))?;
        Ok(count.0)
    }
}

// Re-export repository traits
pub use {
    BlockRepository, TransactionRepository, AccountRepository,
    AssetRepository, AccountAssetRepository, ContractRepository,
    TransactionReceiptRepository
};