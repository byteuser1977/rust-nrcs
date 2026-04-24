use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::models::*;
use super::traits::*;
use super::public_key::PublicKeyRepository;
use blockchain_types::account_ext::AccountPublicKey;
use blockchain_types::AccountId;

pub struct SqliteBlockRepository {
    pool: SqlitePool,
}

impl SqliteBlockRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BlockRepository for SqliteBlockRepository {
    async fn find_by_height(&self, height: i32) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as::<_, BlockModel>(
            "SELECT * FROM block WHERE height = ?"
        )
        .bind(height)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_id_column(&self, id: i64) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as::<_, BlockModel>(
            "SELECT * FROM block WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_hash(&self, hash: &[u8]) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as::<_, BlockModel>(
            "SELECT * FROM block WHERE payload_hash = ? OR generation_signature = ?"
        )
        .bind(hash)
        .bind(hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_latest(&self) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as::<_, BlockModel>(
            "SELECT * FROM block ORDER BY height DESC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_range(&self, start_height: i32, end_height: i32) -> RepositoryResult<Vec<BlockModel>> {
        let records = sqlx::query_as::<_, BlockModel>(
            "SELECT * FROM block WHERE height BETWEEN ? AND ? ORDER BY height ASC"
        )
        .bind(start_height)
        .bind(end_height)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_generator(&self, generator_id: i64) -> RepositoryResult<Vec<BlockModel>> {
        let records = sqlx::query_as::<_, BlockModel>(
            "SELECT * FROM block WHERE generator_id = ? ORDER BY height DESC"
        )
        .bind(generator_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn get_height(&self) -> RepositoryResult<i32> {
        let record = sqlx::query_as::<_, BlockModel>(
            "SELECT * FROM block ORDER BY height DESC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record.map(|b| b.height).unwrap_or(0))
    }

    async fn get_block_id_at_height(&self, height: i32) -> RepositoryResult<Option<i64>> {
        let record = sqlx::query_as::<_, BlockModel>(
            "SELECT * FROM block WHERE height = ?"
        )
        .bind(height)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record.map(|b| b.id))
    }

    async fn has_block(&self, id: i64) -> RepositoryResult<bool> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM block WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(count.0 > 0)
    }

    async fn get_ids_after(&self, block_id: i64, limit: i32) -> RepositoryResult<Vec<i64>> {
        let block = self.find_by_id_column(block_id).await?;
        if block.is_none() {
            return Ok(Vec::new());
        }
        let height = block.unwrap().height;
        
        let records = sqlx::query_as::<_, BlockModel>(
            "SELECT * FROM block WHERE height > ? ORDER BY height ASC LIMIT ?"
        )
        .bind(height)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        
        Ok(records.into_iter().map(|b| b.id).collect())
    }
}

#[async_trait]
impl Repository<BlockModel> for SqliteBlockRepository {
    async fn insert(&self, block: &BlockModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO block (
                id, version, timestamp, previous_block_id, total_amount,
                total_fee, payload_length, previous_block_hash, cumulative_difficulty,
                base_target, next_block_id, height, generation_signature,
                block_signature, payload_hash, generator_id
            ) VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
            )
            "#,
        )
        .bind(block.id)
        .bind(block.version)
        .bind(block.timestamp)
        .bind(block.previous_block_id)
        .bind(block.total_amount)
        .bind(block.total_fee)
        .bind(block.payload_length)
        .bind(block.previous_block_hash.as_deref())
        .bind(block.cumulative_difficulty.as_slice())
        .bind(block.base_target)
        .bind(block.next_block_id)
        .bind(block.height)
        .bind(block.generation_signature.as_slice())
        .bind(block.block_signature.as_slice())
        .bind(block.payload_hash.as_slice())
        .bind(block.generator_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as::<_, BlockModel>(
            "SELECT * FROM block WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, block: &BlockModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE block SET
                version = ?, timestamp = ?, previous_block_id = ?,
                total_amount = ?, total_fee = ?, payload_length = ?,
                previous_block_hash = ?, cumulative_difficulty = ?,
                base_target = ?, next_block_id = ?, height = ?,
                generation_signature = ?, block_signature = ?,
                payload_hash = ?, generator_id = ?
            WHERE db_id = ?
            "#,
        )
        .bind(block.version)
        .bind(block.timestamp)
        .bind(block.previous_block_id)
        .bind(block.total_amount)
        .bind(block.total_fee)
        .bind(block.payload_length)
        .bind(block.previous_block_hash.as_deref())
        .bind(block.cumulative_difficulty.as_slice())
        .bind(block.base_target)
        .bind(block.next_block_id)
        .bind(block.height)
        .bind(block.generation_signature.as_slice())
        .bind(block.block_signature.as_slice())
        .bind(block.payload_hash.as_slice())
        .bind(block.generator_id)
        .bind(block.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM block WHERE db_id = ?")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<BlockModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, BlockModel>(
            "SELECT * FROM block ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
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

pub struct SqliteTransactionRepository {
    pool: SqlitePool,
}

impl SqliteTransactionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransactionRepository for SqliteTransactionRepository {
    async fn find_by_txid(&self, id: i64) -> RepositoryResult<Option<TransactionModel>> {
        let record = sqlx::query_as::<_, TransactionModel>(
            "SELECT * FROM transaction WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_full_hash(&self, full_hash: &[u8]) -> RepositoryResult<Option<TransactionModel>> {
        let record = sqlx::query_as::<_, TransactionModel>(
            "SELECT * FROM transaction WHERE full_hash = ?"
        )
        .bind(full_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_sender(&self, sender_id: i64, limit: i64) -> RepositoryResult<Vec<TransactionModel>> {
        let records = sqlx::query_as::<_, TransactionModel>(
            "SELECT * FROM transaction WHERE sender_id = ? ORDER BY timestamp DESC LIMIT ?"
        )
        .bind(sender_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_recipient(&self, recipient_id: i64, limit: i64) -> RepositoryResult<Vec<TransactionModel>> {
        let records = sqlx::query_as::<_, TransactionModel>(
            "SELECT * FROM transaction WHERE recipient_id = ? ORDER BY timestamp DESC LIMIT ?"
        )
        .bind(recipient_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_block(&self, block_id: i64) -> RepositoryResult<Vec<TransactionModel>> {
        let records = sqlx::query_as::<_, TransactionModel>(
            "SELECT * FROM transaction WHERE block_id = ? ORDER BY transaction_index ASC"
        )
        .bind(block_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<TransactionModel>> {
        let records = sqlx::query_as::<_, TransactionModel>(
            "SELECT * FROM transaction WHERE height = ? ORDER BY transaction_index ASC"
        )
        .bind(height)
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
impl Repository<TransactionModel> for SqliteTransactionRepository {
    async fn insert(&self, tx: &TransactionModel) -> RepositoryResult<()> {
        sqlx::query(
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
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
            )
            "#,
        )
        .bind(tx.id)
        .bind(tx.deadline)
        .bind(tx.recipient_id)
        .bind(tx.amount)
        .bind(tx.fee)
        .bind(tx.full_hash.as_slice())
        .bind(tx.height)
        .bind(tx.block_id)
        .bind(tx.signature.as_slice())
        .bind(tx.timestamp)
        .bind(tx.r#type)
        .bind(tx.subtype)
        .bind(tx.sender_id)
        .bind(tx.block_timestamp)
        .bind(tx.referenced_transaction_full_hash.as_deref())
        .bind(tx.transaction_index)
        .bind(tx.phased)
        .bind(tx.attachment_bytes.as_deref())
        .bind(tx.version)
        .bind(tx.has_message)
        .bind(tx.has_encrypted_message)
        .bind(tx.has_public_key_announcement)
        .bind(tx.has_prunable_message)
        .bind(tx.has_prunable_attachment)
        .bind(tx.ec_block_height)
        .bind(tx.ec_block_id)
        .bind(tx.has_encrypttoself_message)
        .bind(tx.has_prunable_encrypted_message)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<TransactionModel>> {
        let record = sqlx::query_as::<_, TransactionModel>(
            "SELECT * FROM transaction WHERE db_id = ?"
        )
        .bind(db_id)
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

pub struct SqliteAccountRepository {
    pool: SqlitePool,
}

impl SqliteAccountRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountRepository for SqliteAccountRepository {
    async fn find_by_account_id(&self, id: i64) -> RepositoryResult<Option<AccountModel>> {
        let record = sqlx::query_as::<_, AccountModel>(
            "SELECT * FROM account WHERE id = ? AND latest = 1 LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<AccountModel>> {
        let records = sqlx::query_as::<_, AccountModel>(
            "SELECT * FROM account WHERE height = ? AND latest = 1"
        )
        .bind(height)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_latest_by_id(&self, id: i64) -> RepositoryResult<Option<AccountModel>> {
        self.find_by_account_id(id).await
    }

    async fn find_by_address(&self, _address: &str) -> RepositoryResult<Option<AccountModel>> {
        let record = sqlx::query_as::<_, AccountModel>(
            "SELECT * FROM account WHERE latest = 1 ORDER BY height DESC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update_balance(&self, account_id: i64, balance: i64, unconfirmed_balance: i64) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account
            SET balance = ?, unconfirmed_balance = ?
            WHERE id = ? AND latest = 1
            "#,
        )
        .bind(balance)
        .bind(unconfirmed_balance)
        .bind(account_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<AccountModel> for SqliteAccountRepository {
    async fn insert(&self, account: &AccountModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO account (
                id, balance, unconfirmed_balance, forged_balance,
                active_lessee_id, has_control_phasing, height, latest
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(account.id)
        .bind(account.balance)
        .bind(account.unconfirmed_balance)
        .bind(account.forged_balance)
        .bind(account.active_lessee_id)
        .bind(account.has_control_phasing)
        .bind(account.height)
        .bind(account.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AccountModel>> {
        let record = sqlx::query_as::<_, AccountModel>(
            "SELECT * FROM account WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, account: &AccountModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account SET
                balance = ?, unconfirmed_balance = ?, forged_balance = ?,
                active_lessee_id = ?, has_control_phasing = ?, height = ?, latest = ?
            WHERE db_id = ?
            "#,
        )
        .bind(account.balance)
        .bind(account.unconfirmed_balance)
        .bind(account.forged_balance)
        .bind(account.active_lessee_id)
        .bind(account.has_control_phasing)
        .bind(account.height)
        .bind(account.latest)
        .bind(account.db_id)
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
        let records = sqlx::query_as::<_, AccountModel>(
            "SELECT * FROM account WHERE latest = 1 ORDER BY id LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM account WHERE latest = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteAccountAssetRepository {
    pool: SqlitePool,
}

impl SqliteAccountAssetRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountAssetRepository for SqliteAccountAssetRepository {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<AccountAssetModel>> {
        let records = sqlx::query_as::<_, AccountAssetModel>(
            "SELECT * FROM account_asset WHERE account_id = ? AND latest = 1"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_asset(&self, asset_id: i64) -> RepositoryResult<Vec<AccountAssetModel>> {
        let records = sqlx::query_as::<_, AccountAssetModel>(
            "SELECT * FROM account_asset WHERE asset_id = ? AND latest = 1"
        )
        .bind(asset_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_account_and_asset(&self, account_id: i64, asset_id: i64) -> RepositoryResult<Option<AccountAssetModel>> {
        let record = sqlx::query_as::<_, AccountAssetModel>(
            "SELECT * FROM account_asset WHERE account_id = ? AND asset_id = ? AND latest = 1 LIMIT 1"
        )
        .bind(account_id)
        .bind(asset_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update_quantity(&self, account_id: i64, asset_id: i64, quantity: i64, height: i32) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account_asset
            SET quantity = ?, height = ?, latest = 1
            WHERE account_id = ? AND asset_id = ?
            "#,
        )
        .bind(quantity)
        .bind(height)
        .bind(account_id)
        .bind(asset_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn increase_quantity(&self, account_id: i64, asset_id: i64, delta: i64) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account_asset
            SET quantity = quantity + ?, latest = 1
            WHERE account_id = ? AND asset_id = ?
            "#,
        )
        .bind(delta)
        .bind(account_id)
        .bind(asset_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn decrease_quantity(&self, account_id: i64, asset_id: i64, delta: i64) -> RepositoryResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE account_asset
            SET quantity = quantity - ?, latest = 1
            WHERE account_id = ? AND asset_id = ? AND quantity >= ?
            "#,
        )
        .bind(delta)
        .bind(account_id)
        .bind(asset_id)
        .bind(delta)
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
impl Repository<AccountAssetModel> for SqliteAccountAssetRepository {
    async fn insert(&self, aa: &AccountAssetModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO account_asset (
                account_id, asset_id, quantity, unconfirmed_quantity, height, latest
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(aa.account_id)
        .bind(aa.asset_id)
        .bind(aa.quantity)
        .bind(aa.unconfirmed_quantity)
        .bind(aa.height)
        .bind(aa.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AccountAssetModel>> {
        let record = sqlx::query_as::<_, AccountAssetModel>(
            "SELECT * FROM account_asset WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, aa: &AccountAssetModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account_asset SET
                quantity = ?, unconfirmed_quantity = ?, height = ?, latest = ?
            WHERE db_id = ?
            "#,
        )
        .bind(aa.quantity)
        .bind(aa.unconfirmed_quantity)
        .bind(aa.height)
        .bind(aa.latest)
        .bind(aa.db_id)
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
        let records = sqlx::query_as::<_, AccountAssetModel>(
            "SELECT * FROM account_asset WHERE latest = 1 ORDER BY db_id LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM account_asset WHERE latest = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteAssetRepository {
    pool: SqlitePool,
}

impl SqliteAssetRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AssetRepository for SqliteAssetRepository {
    async fn find_by_asset_id(&self, id: i64) -> RepositoryResult<Option<AssetModel>> {
        let record = sqlx::query_as::<_, AssetModel>(
            "SELECT * FROM asset WHERE id = ? AND latest = 1 LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_owner(&self, owner_id: i64) -> RepositoryResult<Vec<AssetModel>> {
        let records = sqlx::query_as::<_, AssetModel>(
            "SELECT * FROM asset WHERE account_id = ? AND latest = 1 ORDER BY height DESC"
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<AssetModel>> {
        let records = sqlx::query_as::<_, AssetModel>(
            "SELECT * FROM asset WHERE height = ? AND latest = 1"
        )
        .bind(height)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_tradable(&self, limit: i64) -> RepositoryResult<Vec<AssetModel>> {
        let records = sqlx::query_as::<_, AssetModel>(
            "SELECT * FROM asset WHERE latest = 1 ORDER BY height DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

#[async_trait]
impl Repository<AssetModel> for SqliteAssetRepository {
    async fn insert(&self, asset: &AssetModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO asset (
                id, account_id, name, description, quantity, decimals,
                has_control_phasing, initial_quantity, height, latest
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(asset.id)
        .bind(asset.account_id)
        .bind(&asset.name)
        .bind(&asset.description)
        .bind(asset.quantity)
        .bind(asset.decimals)
        .bind(asset.has_control_phasing)
        .bind(asset.initial_quantity)
        .bind(asset.height)
        .bind(asset.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AssetModel>> {
        let record = sqlx::query_as::<_, AssetModel>(
            "SELECT * FROM asset WHERE db_id = ?"
        )
        .bind(db_id)
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
        let records = sqlx::query_as::<_, AssetModel>(
            "SELECT * FROM asset WHERE latest = 1 ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM asset WHERE latest = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqlitePublicKeyRepository {
    pool: SqlitePool,
}

impl SqlitePublicKeyRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PublicKeyRepository for SqlitePublicKeyRepository {
    async fn find_latest_by_account_id(&self, account_id: i64) -> RepositoryResult<Option<AccountPublicKey>> {
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            "SELECT public_key FROM public_key WHERE account_id = ? AND latest = 1 ORDER BY height DESC LIMIT 1"
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((pk_vec,)) = row {
            if let Ok(bytes) = pk_vec.try_into() {
                let pk = AccountPublicKey {
                    account_id: account_id as AccountId,
                    public_key: bytes,
                    height: 0,
                };
                return Ok(Some(pk));
            }
        }
        Ok(None)
    }
}
