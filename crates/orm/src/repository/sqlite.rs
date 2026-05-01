use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::models::*;
use super::traits::*;
use super::public_key::PublicKeyRepository;
use blockchain_types::account_ext::AccountPublicKey;
use blockchain_types::{AccountId, Height};

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

    async fn update_next_block_id(&self, previous_block_id: i64, next_block_id: i64) -> RepositoryResult<()> {
        sqlx::query("UPDATE block SET next_block_id = ? WHERE id = ?")
            .bind(next_block_id)
            .bind(previous_block_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete_after_height(&self, height: i32) -> RepositoryResult<Vec<BlockModel>> {
        let blocks = self.find_blocks_after_height(height).await?;
        
        for block in &blocks {
            sqlx::query("DELETE FROM block WHERE id = ?")
                .bind(block.id)
                .execute(&self.pool)
                .await
                .map_err(RepositoryError::DbError)?;
        }
        
        if let Some(last_block) = blocks.last() {
            sqlx::query("UPDATE block SET next_block_id = NULL WHERE id = ?")
                .bind(last_block.previous_block_id)
                .execute(&self.pool)
                .await
                .map_err(RepositoryError::DbError)?;
        }
        
        Ok(blocks)
    }

    async fn find_blocks_after_height(&self, height: i32) -> RepositoryResult<Vec<BlockModel>> {
        let records = sqlx::query_as::<_, BlockModel>(
            "SELECT * FROM block WHERE height > ? ORDER BY height ASC"
        )
        .bind(height)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn delete_blocks_by_ids(&self, db_ids: &[i64]) -> RepositoryResult<()> {
        if db_ids.is_empty() {
            return Ok(());
        }

        for &db_id in db_ids {
            sqlx::query("DELETE FROM block WHERE db_id = ?")
                .bind(db_id)
                .execute(&self.pool)
                .await
                .map_err(RepositoryError::DbError)?;
        }
        Ok(())
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
            r#"SELECT * FROM "transaction" WHERE id = ?"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_full_hash(&self, full_hash: &[u8]) -> RepositoryResult<Option<TransactionModel>> {
        let record = sqlx::query_as::<_, TransactionModel>(
            r#"SELECT * FROM "transaction" WHERE full_hash = ?"#
        )
        .bind(full_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_sender(&self, sender_id: i64, limit: i64) -> RepositoryResult<Vec<TransactionModel>> {
        let records = sqlx::query_as::<_, TransactionModel>(
            r#"SELECT * FROM "transaction" WHERE sender_id = ? ORDER BY timestamp DESC LIMIT ?"#
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
            r#"SELECT * FROM "transaction" WHERE recipient_id = ? ORDER BY timestamp DESC LIMIT ?"#
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
            r#"SELECT * FROM "transaction" WHERE block_id = ? ORDER BY transaction_index ASC"#
        )
        .bind(block_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<TransactionModel>> {
        let records = sqlx::query_as::<_, TransactionModel>(
            r#"SELECT * FROM "transaction" WHERE height = ? ORDER BY transaction_index ASC"#
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

    async fn delete_transactions_by_ids(&self, db_ids: &[i64]) -> RepositoryResult<()> {
        if db_ids.is_empty() {
            return Ok(());
        }

        for &db_id in db_ids {
            sqlx::query(r#"DELETE FROM "transaction" WHERE db_id = ?"#)
                .bind(db_id)
                .execute(&self.pool)
                .await
                .map_err(RepositoryError::DbError)?;
        }
        Ok(())
    }
}

#[async_trait]
impl Repository<TransactionModel> for SqliteTransactionRepository {
    async fn insert(&self, tx: &TransactionModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO "transaction" (
                id, deadline, recipient_id, amount, fee, full_hash,
                height, block_id, signature, timestamp, type, subtype,
                sender_id, block_timestamp, referenced_transaction_full_hash,
                transaction_index, phased, attachment_bytes, version,
                has_message, has_encrypted_message, has_public_key_announcement,
                has_prunable_message, has_prunable_attachment, ec_block_height,
                ec_block_id, has_encrypttoself_message, has_prunable_encrypted_message
            ) VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
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
            r#"SELECT * FROM "transaction" WHERE db_id = ?"#
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
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM "transaction""#)
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

    async fn update_balance(&self, account_id: i64, balance: i64, unconfirmed_balance: i64, height: i32) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account
            SET balance = ?, unconfirmed_balance = ?, height = ?
            WHERE id = ? AND latest = 1
            "#,
        )
        .bind(balance)
        .bind(unconfirmed_balance)
        .bind(height)
        .bind(account_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn get_or_create(&self, account_id: i64) -> RepositoryResult<AccountModel> {
        if let Some(account) = self.find_by_account_id(account_id).await? {
            return Ok(account);
        }
        
        let account = AccountModel {
            db_id: 0,
            id: account_id,
            balance: 0,
            unconfirmed_balance: 0,
            forged_balance: 0,
            active_lessee_id: None,
            has_control_phasing: false,
            height: 0,
            latest: true,
        };
        
        self.insert(&account).await?;
        Ok(account)
    }

    async fn add_to_balance(&self, account_id: i64, amount: i64, height: i32) -> RepositoryResult<()> {
        // 对应 Java: Math.addExact() 溢出检查 + checkBalance() 负数检查
        // First check if the result would be negative
        if amount < 0 {
            let account = self.find_by_account_id(account_id).await?;
            if let Some(acc) = account {
                let new_balance = acc.balance.checked_add(amount)
                    .ok_or_else(|| RepositoryError::Validation(
                        format!("balance overflow for account {}", account_id)
                    ))?;
                if new_balance < 0 {
                    return Err(RepositoryError::Validation(
                        format!("insufficient balance for account {}: have {}, need {}",
                            account_id, acc.balance, -amount)
                    ));
                }
            }
        }

        let result = sqlx::query(
            r#"
            UPDATE account
            SET balance = balance + ?, height = ?
            WHERE id = ? AND latest = 1
            "#,
        )
        .bind(amount)
        .bind(height)
        .bind(account_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        if result.rows_affected() == 0 {
            let _account = self.get_or_create(account_id).await?;
            sqlx::query(
                r#"
                UPDATE account
                SET balance = balance + ?, height = ?
                WHERE id = ? AND latest = 1
                "#,
            )
            .bind(amount)
            .bind(height)
            .bind(account_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        }
        Ok(())
    }

    async fn add_to_unconfirmed_balance(&self, account_id: i64, amount: i64, height: i32) -> RepositoryResult<()> {
        // 对应 Java: Math.addExact() 溢出检查
        if amount < 0 {
            let account = self.find_by_account_id(account_id).await?;
            if let Some(acc) = account {
                let new_balance = acc.unconfirmed_balance.checked_add(amount)
                    .ok_or_else(|| RepositoryError::Validation(
                        format!("unconfirmed balance overflow for account {}", account_id)
                    ))?;
                if new_balance < 0 {
                    return Err(RepositoryError::Validation(
                        format!("insufficient unconfirmed balance for account {}: have {}, need {}",
                            account_id, acc.unconfirmed_balance, -amount)
                    ));
                }
            }
        }

        let result = sqlx::query(
            r#"
            UPDATE account
            SET unconfirmed_balance = unconfirmed_balance + ?, height = ?
            WHERE id = ? AND latest = 1
            "#,
        )
        .bind(amount)
        .bind(height)
        .bind(account_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        if result.rows_affected() == 0 {
            let _account = self.get_or_create(account_id).await?;
            sqlx::query(
                r#"
                UPDATE account
                SET unconfirmed_balance = unconfirmed_balance + ?, height = ?
                WHERE id = ? AND latest = 1
                "#,
            )
            .bind(amount)
            .bind(height)
            .bind(account_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        }
        Ok(())
    }

    async fn add_to_balance_and_unconfirmed(&self, account_id: i64, amount: i64, height: i32) -> RepositoryResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE account
            SET balance = balance + ?, unconfirmed_balance = unconfirmed_balance + ?, height = ?
            WHERE id = ? AND latest = 1
            "#,
        )
        .bind(amount)
        .bind(amount)
        .bind(height)
        .bind(account_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        if result.rows_affected() == 0 {
            let _account = self.get_or_create(account_id).await?;
            sqlx::query(
                r#"
                UPDATE account
                SET balance = balance + ?, unconfirmed_balance = unconfirmed_balance + ?, height = ?
                WHERE id = ? AND latest = 1
                "#,
            )
            .bind(amount)
            .bind(amount)
            .bind(height)
            .bind(account_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        }
        Ok(())
    }

    async fn get_account_count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM account WHERE latest = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }

    async fn add_to_forged_balance(&self, account_id: i64, amount: i64, height: i32) -> RepositoryResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE account
            SET forged_balance = forged_balance + ?, height = ?
            WHERE id = ? AND latest = 1
            "#,
        )
        .bind(amount)
        .bind(height)
        .bind(account_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        if result.rows_affected() == 0 {
            let _account = self.get_or_create(account_id).await?;
            sqlx::query(
                r#"
                UPDATE account
                SET forged_balance = forged_balance + ?, height = ?
                WHERE id = ? AND latest = 1
                "#,
            )
            .bind(amount)
            .bind(height)
            .bind(account_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        }
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

    async fn add_to_unconfirmed_quantity(&self, account_id: i64, asset_id: i64, delta: i64) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account_asset
            SET unconfirmed_quantity = unconfirmed_quantity + ?, latest = 1
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

    async fn increase_quantity(&self, asset_id: i64, delta: i64) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE asset
            SET quantity = quantity + ?, latest = 1
            WHERE id = ? AND latest = 1
            "#,
        )
        .bind(delta)
        .bind(asset_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn decrease_quantity(&self, asset_id: i64, delta: i64) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE asset
            SET quantity = quantity - ?, latest = 1
            WHERE id = ? AND latest = 1 AND quantity >= ?
            "#,
        )
        .bind(delta)
        .bind(asset_id)
        .bind(delta)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
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
        let row: Option<(Vec<u8>, i32)> = sqlx::query_as(
            "SELECT public_key, height FROM public_key WHERE account_id = ? AND latest = 1 ORDER BY height DESC LIMIT 1"
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((pk_vec, height)) = row {
            if let Ok(bytes) = pk_vec.try_into() {
                let pk = AccountPublicKey {
                    account_id: account_id as AccountId,
                    public_key: bytes,
                    height: height as Height,
                };
                return Ok(Some(pk));
            }
        }
        Ok(None)
    }

    async fn insert(&self, pk: &AccountPublicKey) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO public_key (account_id, public_key, height)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(pk.account_id as i64)
        .bind(pk.public_key.to_vec())
        .bind(pk.height as i64)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

pub struct SqliteAccountLedgerRepository {
    pool: SqlitePool,
}

impl SqliteAccountLedgerRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountLedgerRepository for SqliteAccountLedgerRepository {
    async fn find_by_account(&self, account_id: i64, limit: i64) -> RepositoryResult<Vec<AccountLedgerModel>> {
        let records = sqlx::query_as::<_, AccountLedgerModel>(
            "SELECT * FROM account_ledger WHERE account_id = ? ORDER BY height DESC LIMIT ?"
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_block(&self, block_id: i64) -> RepositoryResult<Vec<AccountLedgerModel>> {
        let records = sqlx::query_as::<_, AccountLedgerModel>(
            "SELECT * FROM account_ledger WHERE block_id = ? ORDER BY height DESC"
        )
        .bind(block_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

#[async_trait]
impl Repository<AccountLedgerModel> for SqliteAccountLedgerRepository {
    async fn insert(&self, ledger: &AccountLedgerModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO account_ledger (
                account_id, event_type, event_id, holding_type, holding_id,
                "CHANGE", balance, block_id, height, timestamp
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(ledger.account_id)
        .bind(ledger.event_type)
        .bind(ledger.event_id)
        .bind(ledger.holding_type)
        .bind(ledger.holding_id)
        .bind(ledger.change)
        .bind(ledger.balance)
        .bind(ledger.block_id)
        .bind(ledger.height)
        .bind(ledger.timestamp)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AccountLedgerModel>> {
        let record = sqlx::query_as::<_, AccountLedgerModel>(
            "SELECT * FROM account_ledger WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _ledger: &AccountLedgerModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for account_ledger".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for account_ledger".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AccountLedgerModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, AccountLedgerModel>(
            "SELECT * FROM account_ledger ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM account_ledger")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteAliasRepository {
    pool: SqlitePool,
}

impl SqliteAliasRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AliasRepository for SqliteAliasRepository {
    async fn find_by_alias_id(&self, id: i64) -> RepositoryResult<Option<AliasModel>> {
        let record = sqlx::query_as::<_, AliasModel>(
            "SELECT * FROM alias WHERE id = ? AND latest = 1 LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_name(&self, name: &str) -> RepositoryResult<Option<AliasModel>> {
        let name_lower = name.to_lowercase();
        let record = sqlx::query_as::<_, AliasModel>(
            "SELECT * FROM alias WHERE alias_name_lower = ? AND latest = 1 LIMIT 1"
        )
        .bind(&name_lower)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_owner(&self, account_id: i64) -> RepositoryResult<Vec<AliasModel>> {
        let records = sqlx::query_as::<_, AliasModel>(
            "SELECT * FROM alias WHERE account_id = ? AND latest = 1 ORDER BY alias_name_lower"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn update_owner(&self, alias_id: i64, new_owner_id: i64) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE alias SET account_id = ? WHERE id = ? AND latest = 1"
        )
        .bind(new_owner_id)
        .bind(alias_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn update_uri(&self, alias_id: i64, uri: &str) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE alias SET alias_uri = ? WHERE id = ? AND latest = 1"
        )
        .bind(uri)
        .bind(alias_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<AliasModel> for SqliteAliasRepository {
    async fn insert(&self, alias: &AliasModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO alias (
                id, account_id, alias_name, alias_name_lower, alias_uri,
                timestamp, height, latest
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(alias.id)
        .bind(alias.account_id)
        .bind(&alias.alias_name)
        .bind(&alias.alias_name_lower)
        .bind(&alias.alias_uri)
        .bind(alias.timestamp)
        .bind(alias.height)
        .bind(alias.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AliasModel>> {
        let record = sqlx::query_as::<_, AliasModel>(
            "SELECT * FROM alias WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, alias: &AliasModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE alias SET
                account_id = ?, alias_name = ?, alias_name_lower = ?,
                alias_uri = ?, timestamp = ?, height = ?, latest = ?
            WHERE db_id = ?
            "#,
        )
        .bind(alias.account_id)
        .bind(&alias.alias_name)
        .bind(&alias.alias_name_lower)
        .bind(&alias.alias_uri)
        .bind(alias.timestamp)
        .bind(alias.height)
        .bind(alias.latest)
        .bind(alias.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for alias (use logical delete)".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AliasModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, AliasModel>(
            "SELECT * FROM alias WHERE latest = 1 ORDER BY alias_name_lower LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alias WHERE latest = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteAliasOfferRepository {
    pool: SqlitePool,
}

impl SqliteAliasOfferRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AliasOfferRepository for SqliteAliasOfferRepository {
    async fn find_by_alias(&self, alias_id: i64) -> RepositoryResult<Option<AliasOfferModel>> {
        let record = sqlx::query_as::<_, AliasOfferModel>(
            "SELECT * FROM alias_offer WHERE id = ? AND latest = 1 LIMIT 1"
        )
        .bind(alias_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_buyer(&self, buyer_id: i64) -> RepositoryResult<Vec<AliasOfferModel>> {
        let records = sqlx::query_as::<_, AliasOfferModel>(
            "SELECT * FROM alias_offer WHERE buyer_id = ? AND latest = 1"
        )
        .bind(buyer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn update_price(&self, alias_id: i64, price: i64, buyer_id: Option<i64>) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE alias_offer SET price = ?, buyer_id = ? WHERE id = ? AND latest = 1"
        )
        .bind(price)
        .bind(buyer_id)
        .bind(alias_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete_by_alias(&self, alias_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM alias_offer WHERE id = ?")
            .bind(alias_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<AliasOfferModel> for SqliteAliasOfferRepository {
    async fn insert(&self, offer: &AliasOfferModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO alias_offer (id, price, buyer_id, height, latest)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(offer.id)
        .bind(offer.price)
        .bind(offer.buyer_id)
        .bind(offer.height)
        .bind(offer.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AliasOfferModel>> {
        let record = sqlx::query_as::<_, AliasOfferModel>(
            "SELECT * FROM alias_offer WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, offer: &AliasOfferModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE alias_offer SET price = ?, buyer_id = ?, height = ?, latest = ?
            WHERE db_id = ?
            "#,
        )
        .bind(offer.price)
        .bind(offer.buyer_id)
        .bind(offer.height)
        .bind(offer.latest)
        .bind(offer.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM alias_offer WHERE db_id = ?")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AliasOfferModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, AliasOfferModel>(
            "SELECT * FROM alias_offer WHERE latest = 1 ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alias_offer WHERE latest = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteAssetTransferRepository {
    pool: SqlitePool,
}

impl SqliteAssetTransferRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AssetTransferRepository for SqliteAssetTransferRepository {
    async fn find_by_asset(&self, asset_id: i64, limit: i64) -> RepositoryResult<Vec<AssetTransferModel>> {
        let records = sqlx::query_as::<_, AssetTransferModel>(
            "SELECT * FROM asset_transfer WHERE asset_id = ? ORDER BY height DESC LIMIT ?"
        )
        .bind(asset_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_sender(&self, sender_id: i64, limit: i64) -> RepositoryResult<Vec<AssetTransferModel>> {
        let records = sqlx::query_as::<_, AssetTransferModel>(
            "SELECT * FROM asset_transfer WHERE sender_id = ? ORDER BY height DESC LIMIT ?"
        )
        .bind(sender_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_recipient(&self, recipient_id: i64, limit: i64) -> RepositoryResult<Vec<AssetTransferModel>> {
        let records = sqlx::query_as::<_, AssetTransferModel>(
            "SELECT * FROM asset_transfer WHERE recipient_id = ? ORDER BY height DESC LIMIT ?"
        )
        .bind(recipient_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

#[async_trait]
impl Repository<AssetTransferModel> for SqliteAssetTransferRepository {
    async fn insert(&self, transfer: &AssetTransferModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO asset_transfer (id, asset_id, sender_id, recipient_id, quantity, timestamp, height)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(transfer.id)
        .bind(transfer.asset_id)
        .bind(transfer.sender_id)
        .bind(transfer.recipient_id)
        .bind(transfer.quantity)
        .bind(transfer.timestamp)
        .bind(transfer.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AssetTransferModel>> {
        let record = sqlx::query_as::<_, AssetTransferModel>(
            "SELECT * FROM asset_transfer WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &AssetTransferModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for asset_transfer".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for asset_transfer".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AssetTransferModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, AssetTransferModel>(
            "SELECT * FROM asset_transfer ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM asset_transfer")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteAskOrderRepository {
    pool: SqlitePool,
}

impl SqliteAskOrderRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AskOrderRepository for SqliteAskOrderRepository {
    async fn find_by_order_id(&self, id: i64) -> RepositoryResult<Option<AskOrderModel>> {
        let record = sqlx::query_as::<_, AskOrderModel>(
            "SELECT * FROM ask_order WHERE id = ? AND latest = 1 LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_asset(&self, asset_id: i64, limit: i64) -> RepositoryResult<Vec<AskOrderModel>> {
        let records = sqlx::query_as::<_, AskOrderModel>(
            "SELECT * FROM ask_order WHERE asset_id = ? AND latest = 1 ORDER BY price ASC, height ASC LIMIT ?"
        )
        .bind(asset_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<AskOrderModel>> {
        let records = sqlx::query_as::<_, AskOrderModel>(
            "SELECT * FROM ask_order WHERE account_id = ? AND latest = 1 ORDER BY height DESC"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_best_by_asset(&self, asset_id: i64) -> RepositoryResult<Option<AskOrderModel>> {
        let record = sqlx::query_as::<_, AskOrderModel>(
            "SELECT * FROM ask_order WHERE asset_id = ? AND latest = 1 ORDER BY price ASC, height ASC LIMIT 1"
        )
        .bind(asset_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update_quantity(&self, order_id: i64, quantity: i64) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE ask_order SET quantity = ? WHERE id = ? AND latest = 1"
        )
        .bind(quantity)
        .bind(order_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<AskOrderModel> for SqliteAskOrderRepository {
    async fn insert(&self, order: &AskOrderModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO ask_order (
                id, account_id, asset_id, price, transaction_index,
                transaction_height, quantity, creation_height, height, latest
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(order.id)
        .bind(order.account_id)
        .bind(order.asset_id)
        .bind(order.price)
        .bind(order.transaction_index)
        .bind(order.transaction_height)
        .bind(order.quantity)
        .bind(order.creation_height)
        .bind(order.height)
        .bind(order.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AskOrderModel>> {
        let record = sqlx::query_as::<_, AskOrderModel>(
            "SELECT * FROM ask_order WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &AskOrderModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("use update_quantity for ask_order".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for ask_order".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AskOrderModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, AskOrderModel>(
            "SELECT * FROM ask_order WHERE latest = 1 ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM ask_order WHERE latest = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteBidOrderRepository {
    pool: SqlitePool,
}

impl SqliteBidOrderRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BidOrderRepository for SqliteBidOrderRepository {
    async fn find_by_order_id(&self, id: i64) -> RepositoryResult<Option<BidOrderModel>> {
        let record = sqlx::query_as::<_, BidOrderModel>(
            "SELECT * FROM bid_order WHERE id = ? AND latest = 1 LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_asset(&self, asset_id: i64, limit: i64) -> RepositoryResult<Vec<BidOrderModel>> {
        let records = sqlx::query_as::<_, BidOrderModel>(
            "SELECT * FROM bid_order WHERE asset_id = ? AND latest = 1 ORDER BY price DESC, height ASC LIMIT ?"
        )
        .bind(asset_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<BidOrderModel>> {
        let records = sqlx::query_as::<_, BidOrderModel>(
            "SELECT * FROM bid_order WHERE account_id = ? AND latest = 1 ORDER BY height DESC"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_best_by_asset(&self, asset_id: i64) -> RepositoryResult<Option<BidOrderModel>> {
        let record = sqlx::query_as::<_, BidOrderModel>(
            "SELECT * FROM bid_order WHERE asset_id = ? AND latest = 1 ORDER BY price DESC, height ASC LIMIT 1"
        )
        .bind(asset_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update_quantity(&self, order_id: i64, quantity: i64) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE bid_order SET quantity = ? WHERE id = ? AND latest = 1"
        )
        .bind(quantity)
        .bind(order_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<BidOrderModel> for SqliteBidOrderRepository {
    async fn insert(&self, order: &BidOrderModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO bid_order (
                id, account_id, asset_id, price, transaction_index,
                transaction_height, quantity, creation_height, height, latest
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(order.id)
        .bind(order.account_id)
        .bind(order.asset_id)
        .bind(order.price)
        .bind(order.transaction_index)
        .bind(order.transaction_height)
        .bind(order.quantity)
        .bind(order.creation_height)
        .bind(order.height)
        .bind(order.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<BidOrderModel>> {
        let record = sqlx::query_as::<_, BidOrderModel>(
            "SELECT * FROM bid_order WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &BidOrderModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("use update_quantity for bid_order".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for bid_order".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<BidOrderModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, BidOrderModel>(
            "SELECT * FROM bid_order WHERE latest = 1 ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM bid_order WHERE latest = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteTradeRepository {
    pool: SqlitePool,
}

impl SqliteTradeRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TradeRepository for SqliteTradeRepository {
    async fn find_by_asset(&self, asset_id: i64, limit: i64) -> RepositoryResult<Vec<TradeModel>> {
        let records = sqlx::query_as::<_, TradeModel>(
            "SELECT * FROM trade WHERE asset_id = ? ORDER BY height DESC, timestamp DESC LIMIT ?"
        )
        .bind(asset_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_ask_order(&self, ask_order_id: i64) -> RepositoryResult<Vec<TradeModel>> {
        let records = sqlx::query_as::<_, TradeModel>(
            "SELECT * FROM trade WHERE ask_order_id = ? ORDER BY height DESC"
        )
        .bind(ask_order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_bid_order(&self, bid_order_id: i64) -> RepositoryResult<Vec<TradeModel>> {
        let records = sqlx::query_as::<_, TradeModel>(
            "SELECT * FROM trade WHERE bid_order_id = ? ORDER BY height DESC"
        )
        .bind(bid_order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_buyer(&self, buyer_id: i64, limit: i64) -> RepositoryResult<Vec<TradeModel>> {
        let records = sqlx::query_as::<_, TradeModel>(
            "SELECT * FROM trade WHERE buyer_id = ? ORDER BY height DESC LIMIT ?"
        )
        .bind(buyer_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_seller(&self, seller_id: i64, limit: i64) -> RepositoryResult<Vec<TradeModel>> {
        let records = sqlx::query_as::<_, TradeModel>(
            "SELECT * FROM trade WHERE seller_id = ? ORDER BY height DESC LIMIT ?"
        )
        .bind(seller_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

#[async_trait]
impl Repository<TradeModel> for SqliteTradeRepository {
    async fn insert(&self, trade: &TradeModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO trade (
                asset_id, block_id, ask_order_id, bid_order_id, ask_order_height,
                bid_order_height, seller_id, buyer_id, is_buy, quantity, price, timestamp, height
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(trade.asset_id)
        .bind(trade.block_id)
        .bind(trade.ask_order_id)
        .bind(trade.bid_order_id)
        .bind(trade.ask_order_height)
        .bind(trade.bid_order_height)
        .bind(trade.seller_id)
        .bind(trade.buyer_id)
        .bind(trade.is_buy)
        .bind(trade.quantity)
        .bind(trade.price)
        .bind(trade.timestamp)
        .bind(trade.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<TradeModel>> {
        let record = sqlx::query_as::<_, TradeModel>(
            "SELECT * FROM trade WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &TradeModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for trade".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for trade".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<TradeModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, TradeModel>(
            "SELECT * FROM trade ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM trade")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteGoodsRepository {
    pool: SqlitePool,
}

impl SqliteGoodsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GoodsRepository for SqliteGoodsRepository {
    async fn find_by_goods_id(&self, id: i64) -> RepositoryResult<Option<GoodsModel>> {
        let record = sqlx::query_as::<_, GoodsModel>(
            "SELECT * FROM goods WHERE id = ? AND latest = 1 LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_seller(&self, seller_id: i64, limit: i64) -> RepositoryResult<Vec<GoodsModel>> {
        let records = sqlx::query_as::<_, GoodsModel>(
            "SELECT * FROM goods WHERE seller_id = ? AND latest = 1 AND delisted = 0 ORDER BY height DESC LIMIT ?"
        )
        .bind(seller_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_in_stock(&self, limit: i64) -> RepositoryResult<Vec<GoodsModel>> {
        let records = sqlx::query_as::<_, GoodsModel>(
            "SELECT * FROM goods WHERE latest = 1 AND delisted = 0 AND quantity > 0 ORDER BY height DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn update_quantity(&self, goods_id: i64, quantity: i32) -> RepositoryResult<()> {
        sqlx::query("UPDATE goods SET quantity = ? WHERE id = ? AND latest = 1")
            .bind(quantity)
            .bind(goods_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn update_price(&self, goods_id: i64, price: i64) -> RepositoryResult<()> {
        sqlx::query("UPDATE goods SET price = ? WHERE id = ? AND latest = 1")
            .bind(price)
            .bind(goods_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn set_delisted(&self, goods_id: i64, delisted: bool) -> RepositoryResult<()> {
        sqlx::query("UPDATE goods SET delisted = ? WHERE id = ? AND latest = 1")
            .bind(delisted)
            .bind(goods_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<GoodsModel> for SqliteGoodsRepository {
    async fn insert(&self, goods: &GoodsModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO goods (
                id, seller_id, name, description, parsed_tags, tags,
                timestamp, quantity, price, delisted, height, latest, has_image
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(goods.id)
        .bind(goods.seller_id)
        .bind(&goods.name)
        .bind(&goods.description)
        .bind(&goods.parsed_tags)
        .bind(&goods.tags)
        .bind(goods.timestamp)
        .bind(goods.quantity)
        .bind(goods.price)
        .bind(goods.delisted)
        .bind(goods.height)
        .bind(goods.latest)
        .bind(goods.has_image)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<GoodsModel>> {
        let record = sqlx::query_as::<_, GoodsModel>("SELECT * FROM goods WHERE db_id = ?")
            .bind(db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &GoodsModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("use specific update methods for goods".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for goods".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<GoodsModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, GoodsModel>(
            "SELECT * FROM goods WHERE latest = 1 ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM goods WHERE latest = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteCurrencyRepository {
    pool: SqlitePool,
}

impl SqliteCurrencyRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CurrencyRepository for SqliteCurrencyRepository {
    async fn find_by_currency_id(&self, id: i64) -> RepositoryResult<Option<CurrencyModel>> {
        let record = sqlx::query_as::<_, CurrencyModel>(
            "SELECT * FROM currency WHERE id = ? AND latest = 1 LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_code(&self, code: &str) -> RepositoryResult<Option<CurrencyModel>> {
        let record = sqlx::query_as::<_, CurrencyModel>(
            "SELECT * FROM currency WHERE code = ? AND latest = 1 LIMIT 1"
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_owner(&self, account_id: i64) -> RepositoryResult<Vec<CurrencyModel>> {
        let records = sqlx::query_as::<_, CurrencyModel>(
            "SELECT * FROM currency WHERE account_id = ? AND latest = 1 ORDER BY height DESC"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<CurrencyModel>> {
        let records = sqlx::query_as::<_, CurrencyModel>(
            "SELECT * FROM currency WHERE height = ? AND latest = 1"
        )
        .bind(height)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    // P1新增方法实现

    async fn increase_supply(&self, currency_id: i64, delta: i64) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE currency SET initial_supply = initial_supply + ?, latest = 1
            WHERE id = ? AND latest = 1
            "#,
        )
        .bind(delta)
        .bind(currency_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn increase_reserve(&self, currency_id: i64, amount_per_unit: i64) -> RepositoryResult<()> {
        // 计算总reserve增加量：amount_per_unit * current supply
        let (current_supply,): (i64,) = sqlx::query_as(
            "SELECT COALESCE(initial_supply, 0) FROM currency WHERE id = ? AND latest = 1"
        )
        .bind(currency_id)
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        let total_increase = amount_per_unit * current_supply;

        sqlx::query(
            r#"
            UPDATE currency SET reserve_supply = reserve_supply + ?, latest = 1
            WHERE id = ? AND latest = 1
            "#,
        )
        .bind(total_increase)
        .bind(currency_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete_currency(&self, currency_id: i64) -> RepositoryResult<()> {
        // 标记为deleted（软删除）或硬删除
        // 这里使用硬删除，因为NRCS Java也是直接删除
        sqlx::query("DELETE FROM currency WHERE id = ?")
            .bind(currency_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<CurrencyModel> for SqliteCurrencyRepository {
    async fn insert(&self, currency: &CurrencyModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO currency (
                id, account_id, name, name_lower, code, description, type,
                initial_supply, reserve_supply, max_supply, creation_height,
                issuance_height, min_reserve_per_unit_nqt, min_difficulty,
                max_difficulty, ruleset, algorithm, decimals, height, latest
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(currency.id)
        .bind(currency.account_id)
        .bind(&currency.name)
        .bind(&currency.name_lower)
        .bind(&currency.code)
        .bind(&currency.description)
        .bind(currency.type_)
        .bind(currency.initial_supply)
        .bind(currency.reserve_supply)
        .bind(currency.max_supply)
        .bind(currency.creation_height)
        .bind(currency.issuance_height)
        .bind(currency.min_reserve_per_unit_nqt)
        .bind(currency.min_difficulty)
        .bind(currency.max_difficulty)
        .bind(currency.ruleset)
        .bind(currency.algorithm)
        .bind(currency.decimals)
        .bind(currency.height)
        .bind(currency.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<CurrencyModel>> {
        let record = sqlx::query_as::<_, CurrencyModel>("SELECT * FROM currency WHERE db_id = ?")
            .bind(db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &CurrencyModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for currency".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for currency".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<CurrencyModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, CurrencyModel>(
            "SELECT * FROM currency WHERE latest = 1 ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM currency WHERE latest = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteTaggedDataRepository {
    pool: SqlitePool,
}

impl SqliteTaggedDataRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TaggedDataRepository for SqliteTaggedDataRepository {
    async fn find_by_data_id(&self, id: i64) -> RepositoryResult<Option<TaggedDataModel>> {
        let record = sqlx::query_as::<_, TaggedDataModel>(
            "SELECT * FROM tagged_data WHERE id = ? AND latest = 1 LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_account(&self, account_id: i64, limit: i64) -> RepositoryResult<Vec<TaggedDataModel>> {
        let records = sqlx::query_as::<_, TaggedDataModel>(
            "SELECT * FROM tagged_data WHERE account_id = ? AND latest = 1 ORDER BY height DESC LIMIT ?"
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_type(&self, data_type: &str, limit: i64) -> RepositoryResult<Vec<TaggedDataModel>> {
        let records = sqlx::query_as::<_, TaggedDataModel>(
            "SELECT * FROM tagged_data WHERE type = ? AND latest = 1 ORDER BY height DESC LIMIT ?"
        )
        .bind(data_type)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn search_by_tag(&self, tag: &str, limit: i64) -> RepositoryResult<Vec<TaggedDataModel>> {
        let pattern = format!("%{}%", tag);
        let records = sqlx::query_as::<_, TaggedDataModel>(
            "SELECT * FROM tagged_data WHERE tags LIKE ? AND latest = 1 ORDER BY height DESC LIMIT ?"
        )
        .bind(&pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

#[async_trait]
impl Repository<TaggedDataModel> for SqliteTaggedDataRepository {
    async fn insert(&self, data: &TaggedDataModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO tagged_data (
                id, account_id, name, description, tags, parsed_tags, type,
                data, is_text, filename, channel, block_timestamp, transaction_timestamp, height, latest
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(data.id)
        .bind(data.account_id)
        .bind(&data.name)
        .bind(&data.description)
        .bind(&data.tags)
        .bind(&data.parsed_tags)
        .bind(&data.type_)
        .bind(&data.data)
        .bind(data.is_text)
        .bind(&data.filename)
        .bind(&data.channel)
        .bind(data.block_timestamp)
        .bind(data.transaction_timestamp)
        .bind(data.height)
        .bind(data.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<TaggedDataModel>> {
        let record = sqlx::query_as::<_, TaggedDataModel>("SELECT * FROM tagged_data WHERE db_id = ?")
            .bind(db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &TaggedDataModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for tagged_data".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for tagged_data".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<TaggedDataModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, TaggedDataModel>(
            "SELECT * FROM tagged_data WHERE latest = 1 ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tagged_data WHERE latest = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqlitePollRepository {
    pool: SqlitePool,
}

impl SqlitePollRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PollRepository for SqlitePollRepository {
    async fn find_by_poll_id(&self, id: i64) -> RepositoryResult<Option<PollModel>> {
        let record = sqlx::query_as::<_, PollModel>(
            "SELECT * FROM poll WHERE id = ? LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<PollModel>> {
        let records = sqlx::query_as::<_, PollModel>(
            "SELECT * FROM poll WHERE account_id = ? ORDER BY height DESC"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_active(&self, height: i32, limit: i64) -> RepositoryResult<Vec<PollModel>> {
        let records = sqlx::query_as::<_, PollModel>(
            "SELECT * FROM poll WHERE finish_height > ? ORDER BY finish_height ASC LIMIT ?"
        )
        .bind(height)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn update_voters_count(&self, _poll_id: i64, _count: i64) -> RepositoryResult<()> {
        Ok(())
    }
}

#[async_trait]
impl Repository<PollModel> for SqlitePollRepository {
    async fn insert(&self, poll: &PollModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO poll (
                id, account_id, name, description, options, min_num_options, max_num_options,
                min_range_value, max_range_value, timestamp, finish_height, voting_model,
                min_balance, min_balance_model, holding_id, height
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(poll.id)
        .bind(poll.account_id)
        .bind(&poll.name)
        .bind(&poll.description)
        .bind(&poll.options)
        .bind(poll.min_num_options)
        .bind(poll.max_num_options)
        .bind(poll.min_range_value)
        .bind(poll.max_range_value)
        .bind(poll.timestamp)
        .bind(poll.finish_height)
        .bind(poll.voting_model)
        .bind(poll.min_balance)
        .bind(poll.min_balance_model)
        .bind(poll.holding_id)
        .bind(poll.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<PollModel>> {
        let record = sqlx::query_as::<_, PollModel>("SELECT * FROM poll WHERE db_id = ?")
            .bind(db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &PollModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for poll".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for poll".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PollModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, PollModel>(
            "SELECT * FROM poll ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM poll")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteVoteRepository {
    pool: SqlitePool,
}

impl SqliteVoteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VoteRepository for SqliteVoteRepository {
    async fn find_by_poll(&self, poll_id: i64, limit: i64) -> RepositoryResult<Vec<VoteModel>> {
        let records = sqlx::query_as::<_, VoteModel>(
            "SELECT * FROM vote WHERE poll_id = ? ORDER BY height DESC LIMIT ?"
        )
        .bind(poll_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_voter(&self, voter_id: i64, limit: i64) -> RepositoryResult<Vec<VoteModel>> {
        let records = sqlx::query_as::<_, VoteModel>(
            "SELECT * FROM vote WHERE voter_id = ? ORDER BY height DESC LIMIT ?"
        )
        .bind(voter_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_poll_and_voter(&self, poll_id: i64, voter_id: i64) -> RepositoryResult<Option<VoteModel>> {
        let record = sqlx::query_as::<_, VoteModel>(
            "SELECT * FROM vote WHERE poll_id = ? AND voter_id = ? LIMIT 1"
        )
        .bind(poll_id)
        .bind(voter_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }
}

#[async_trait]
impl Repository<VoteModel> for SqliteVoteRepository {
    async fn insert(&self, vote: &VoteModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO vote (id, poll_id, voter_id, vote_bytes, height)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(vote.id)
        .bind(vote.poll_id)
        .bind(vote.voter_id)
        .bind(&vote.vote_bytes)
        .bind(vote.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<VoteModel>> {
        let record = sqlx::query_as::<_, VoteModel>("SELECT * FROM vote WHERE db_id = ?")
            .bind(db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &VoteModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for vote".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for vote".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<VoteModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, VoteModel>(
            "SELECT * FROM vote ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vote")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteShufflingRepository {
    pool: SqlitePool,
}

impl SqliteShufflingRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ShufflingRepository for SqliteShufflingRepository {
    async fn find_by_shuffling_id(&self, id: i64) -> RepositoryResult<Option<ShufflingModel>> {
        let record = sqlx::query_as::<_, ShufflingModel>(
            "SELECT * FROM shuffling WHERE id = ? AND latest = 1 LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_issuer(&self, issuer_id: i64) -> RepositoryResult<Vec<ShufflingModel>> {
        let records = sqlx::query_as::<_, ShufflingModel>(
            "SELECT * FROM shuffling WHERE issuer_id = ? AND latest = 1 ORDER BY height DESC"
        )
        .bind(issuer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_active(&self, limit: i64) -> RepositoryResult<Vec<ShufflingModel>> {
        let records = sqlx::query_as::<_, ShufflingModel>(
            "SELECT * FROM shuffling WHERE latest = 1 AND stage < 5 ORDER BY height DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn update_stage(&self, shuffling_id: i64, stage: i32) -> RepositoryResult<()> {
        sqlx::query("UPDATE shuffling SET stage = ? WHERE id = ? AND latest = 1")
            .bind(stage)
            .bind(shuffling_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<ShufflingModel> for SqliteShufflingRepository {
    async fn insert(&self, shuffling: &ShufflingModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO shuffling (
                id, holding_id, holding_type, issuer_id, amount, participant_count,
                blocks_remaining, stage, assignee_account_id, registrant_count,
                recipient_public_keys, height, latest
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(shuffling.id)
        .bind(shuffling.holding_id)
        .bind(shuffling.holding_type)
        .bind(shuffling.issuer_id)
        .bind(shuffling.amount)
        .bind(shuffling.participant_count)
        .bind(shuffling.blocks_remaining)
        .bind(shuffling.stage)
        .bind(shuffling.assignee_account_id)
        .bind(shuffling.registrant_count)
        .bind(&shuffling.recipient_public_keys)
        .bind(shuffling.height)
        .bind(shuffling.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<ShufflingModel>> {
        let record = sqlx::query_as::<_, ShufflingModel>("SELECT * FROM shuffling WHERE db_id = ?")
            .bind(db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &ShufflingModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("use update_stage for shuffling".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for shuffling".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<ShufflingModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, ShufflingModel>(
            "SELECT * FROM shuffling WHERE latest = 1 ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM shuffling WHERE latest = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqlitePurchaseRepository {
    pool: SqlitePool,
}

impl SqlitePurchaseRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PurchaseRepository for SqlitePurchaseRepository {
    async fn find_by_purchase_id(&self, id: i64) -> RepositoryResult<Option<PurchaseModel>> {
        let record = sqlx::query_as::<_, PurchaseModel>(
            "SELECT * FROM purchase WHERE id = ? AND latest = 1 LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_buyer(&self, buyer_id: i64, limit: i64) -> RepositoryResult<Vec<PurchaseModel>> {
        let records = sqlx::query_as::<_, PurchaseModel>(
            "SELECT * FROM purchase WHERE buyer_id = ? AND latest = 1 ORDER BY height DESC LIMIT ?"
        )
        .bind(buyer_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_seller(&self, seller_id: i64, limit: i64) -> RepositoryResult<Vec<PurchaseModel>> {
        let records = sqlx::query_as::<_, PurchaseModel>(
            "SELECT * FROM purchase WHERE seller_id = ? AND latest = 1 ORDER BY height DESC LIMIT ?"
        )
        .bind(seller_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_goods(&self, goods_id: i64, limit: i64) -> RepositoryResult<Vec<PurchaseModel>> {
        let records = sqlx::query_as::<_, PurchaseModel>(
            "SELECT * FROM purchase WHERE goods_id = ? AND latest = 1 ORDER BY height DESC LIMIT ?"
        )
        .bind(goods_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn update_pending(&self, purchase_id: i64, pending: bool) -> RepositoryResult<()> {
        sqlx::query("UPDATE purchase SET pending = ? WHERE id = ? AND latest = 1")
            .bind(pending)
            .bind(purchase_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn set_delivered(&self, purchase_id: i64, goods: &[u8], nonce: &[u8]) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE purchase SET goods = ?, goods_nonce = ?, pending = 0 WHERE id = ? AND latest = 1"
        )
        .bind(goods)
        .bind(nonce)
        .bind(purchase_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn set_refund(&self, purchase_id: i64, refund: i64, note: &[u8], nonce: &[u8]) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE purchase SET refund = ?, refund_note = ?, refund_nonce = ? WHERE id = ? AND latest = 1"
        )
        .bind(refund)
        .bind(note)
        .bind(nonce)
        .bind(purchase_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<PurchaseModel> for SqlitePurchaseRepository {
    async fn insert(&self, purchase: &PurchaseModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO purchase (
                id, buyer_id, goods_id, seller_id, quantity, price, deadline,
                note, nonce, timestamp, pending, goods, goods_nonce, goods_is_text,
                refund_note, refund_nonce, has_feedback_notes, has_public_feedbacks,
                discount, refund, height, latest
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(purchase.id)
        .bind(purchase.buyer_id)
        .bind(purchase.goods_id)
        .bind(purchase.seller_id)
        .bind(purchase.quantity)
        .bind(purchase.price)
        .bind(purchase.deadline)
        .bind(&purchase.note)
        .bind(&purchase.nonce)
        .bind(purchase.timestamp)
        .bind(purchase.pending)
        .bind(&purchase.goods)
        .bind(&purchase.goods_nonce)
        .bind(purchase.goods_is_text)
        .bind(&purchase.refund_note)
        .bind(&purchase.refund_nonce)
        .bind(purchase.has_feedback_notes)
        .bind(purchase.has_public_feedbacks)
        .bind(purchase.discount)
        .bind(purchase.refund)
        .bind(purchase.height)
        .bind(purchase.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<PurchaseModel>> {
        let record = sqlx::query_as::<_, PurchaseModel>("SELECT * FROM purchase WHERE db_id = ?")
            .bind(db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &PurchaseModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("use specific update methods for purchase".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for purchase".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PurchaseModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, PurchaseModel>(
            "SELECT * FROM purchase WHERE latest = 1 ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM purchase WHERE latest = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteAccountCurrencyRepository {
    pool: SqlitePool,
}

impl SqliteAccountCurrencyRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountCurrencyRepository for SqliteAccountCurrencyRepository {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<AccountCurrencyModel>> {
        let records = sqlx::query_as::<_, AccountCurrencyModel>(
            "SELECT * FROM account_currency WHERE account_id = ? AND latest = 1"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_currency(&self, currency_id: i64) -> RepositoryResult<Vec<AccountCurrencyModel>> {
        let records = sqlx::query_as::<_, AccountCurrencyModel>(
            "SELECT * FROM account_currency WHERE currency_id = ? AND latest = 1"
        )
        .bind(currency_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_account_and_currency(&self, account_id: i64, currency_id: i64) -> RepositoryResult<Option<AccountCurrencyModel>> {
        let record = sqlx::query_as::<_, AccountCurrencyModel>(
            "SELECT * FROM account_currency WHERE account_id = ? AND currency_id = ? AND latest = 1 LIMIT 1"
        )
        .bind(account_id)
        .bind(currency_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update_units(&self, account_id: i64, currency_id: i64, delta: i64) -> RepositoryResult<()> {
        let result = sqlx::query(
            "UPDATE account_currency SET units = units + ?, latest = 1 WHERE account_id = ? AND currency_id = ? AND latest = 1"
        )
        .bind(delta)
        .bind(account_id)
        .bind(currency_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        if result.rows_affected() == 0 && delta != 0 {
            let current_height: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(height), 0) FROM block")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
            sqlx::query(
                "INSERT INTO account_currency (account_id, currency_id, units, unconfirmed_units, height, latest) VALUES (?, ?, ?, 0, ?, 1)"
            )
            .bind(account_id)
            .bind(currency_id)
            .bind(delta)
            .bind(current_height)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        }

        Ok(())
    }

    async fn add_to_unconfirmed_units(&self, account_id: i64, currency_id: i64, delta: i64) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account_currency
            SET unconfirmed_units = unconfirmed_units + ?, latest = 1
            WHERE account_id = ? AND currency_id = ?
            "#,
        )
        .bind(delta)
        .bind(account_id)
        .bind(currency_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<AccountCurrencyModel> for SqliteAccountCurrencyRepository {
    async fn insert(&self, ac: &AccountCurrencyModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO account_currency (account_id, currency_id, units, unconfirmed_units, height, latest)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(ac.account_id)
        .bind(ac.currency_id)
        .bind(ac.units)
        .bind(ac.unconfirmed_units)
        .bind(ac.height)
        .bind(ac.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AccountCurrencyModel>> {
        let record = sqlx::query_as::<_, AccountCurrencyModel>("SELECT * FROM account_currency WHERE db_id = ?")
            .bind(db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &AccountCurrencyModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("use update_units for account_currency".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for account_currency".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AccountCurrencyModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, AccountCurrencyModel>(
            "SELECT * FROM account_currency WHERE latest = 1 LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM account_currency WHERE latest = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteCurrencyTransferRepository {
    pool: SqlitePool,
}

impl SqliteCurrencyTransferRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CurrencyTransferRepository for SqliteCurrencyTransferRepository {
    async fn find_by_currency(&self, currency_id: i64, limit: i64) -> RepositoryResult<Vec<CurrencyTransferModel>> {
        let records = sqlx::query_as::<_, CurrencyTransferModel>(
            "SELECT * FROM currency_transfer WHERE currency_id = ? ORDER BY height DESC LIMIT ?"
        )
        .bind(currency_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_sender(&self, sender_id: i64, limit: i64) -> RepositoryResult<Vec<CurrencyTransferModel>> {
        let records = sqlx::query_as::<_, CurrencyTransferModel>(
            "SELECT * FROM currency_transfer WHERE sender_id = ? ORDER BY height DESC LIMIT ?"
        )
        .bind(sender_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_recipient(&self, recipient_id: i64, limit: i64) -> RepositoryResult<Vec<CurrencyTransferModel>> {
        let records = sqlx::query_as::<_, CurrencyTransferModel>(
            "SELECT * FROM currency_transfer WHERE recipient_id = ? ORDER BY height DESC LIMIT ?"
        )
        .bind(recipient_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

#[async_trait]
impl Repository<CurrencyTransferModel> for SqliteCurrencyTransferRepository {
    async fn insert(&self, transfer: &CurrencyTransferModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO currency_transfer (id, currency_id, sender_id, recipient_id, units, timestamp, height)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(transfer.id)
        .bind(transfer.currency_id)
        .bind(transfer.sender_id)
        .bind(transfer.recipient_id)
        .bind(transfer.units)
        .bind(transfer.timestamp)
        .bind(transfer.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<CurrencyTransferModel>> {
        let record = sqlx::query_as::<_, CurrencyTransferModel>("SELECT * FROM currency_transfer WHERE db_id = ?")
            .bind(db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &CurrencyTransferModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for currency_transfer".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for currency_transfer".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<CurrencyTransferModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, CurrencyTransferModel>(
            "SELECT * FROM currency_transfer ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM currency_transfer")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteContractReferenceRepository {
    pool: SqlitePool,
}

impl SqliteContractReferenceRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ContractReferenceRepository for SqliteContractReferenceRepository {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<ContractReferenceModel>> {
        let records = sqlx::query_as::<_, ContractReferenceModel>(
            "SELECT * FROM contract_reference WHERE account_id = ? ORDER BY height DESC"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_contract_name(&self, name: &str) -> RepositoryResult<Option<ContractReferenceModel>> {
        let record = sqlx::query_as::<_, ContractReferenceModel>(
            "SELECT * FROM contract_reference WHERE contract_name = ? LIMIT 1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn delete_by_account_and_name(&self, account_id: i64, name: &str) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM contract_reference WHERE account_id = ? AND contract_name = ?")
            .bind(account_id)
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<ContractReferenceModel> for SqliteContractReferenceRepository {
    async fn insert(&self, cr: &ContractReferenceModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO contract_reference (id, account_id, contract_name, height)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(cr.id)
        .bind(cr.account_id)
        .bind(&cr.contract_name)
        .bind(cr.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<ContractReferenceModel>> {
        let record = sqlx::query_as::<_, ContractReferenceModel>("SELECT * FROM contract_reference WHERE db_id = ?")
            .bind(db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &ContractReferenceModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for contract_reference".to_string()))
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM contract_reference WHERE db_id = ?")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<ContractReferenceModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, ContractReferenceModel>(
            "SELECT * FROM contract_reference ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM contract_reference")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteAssetPropertyRepository {
    pool: SqlitePool,
}

impl SqliteAssetPropertyRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AssetPropertyRepository for SqliteAssetPropertyRepository {
    async fn find_by_asset(&self, asset_id: i64) -> RepositoryResult<Vec<AssetPropertyModel>> {
        let records = sqlx::query_as::<_, AssetPropertyModel>(
            "SELECT * FROM asset_property WHERE asset_id = ? AND latest = 1 ORDER BY height DESC"
        )
        .bind(asset_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_asset_and_account(&self, asset_id: i64, setter_id: i64) -> RepositoryResult<Vec<AssetPropertyModel>> {
        let records = sqlx::query_as::<_, AssetPropertyModel>(
            "SELECT * FROM asset_property WHERE asset_id = ? AND setter_id = ? AND latest = 1 ORDER BY height DESC"
        )
        .bind(asset_id)
        .bind(setter_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_asset_account_property(&self, asset_id: i64, setter_id: i64, property: &str) -> RepositoryResult<Option<AssetPropertyModel>> {
        let record = sqlx::query_as::<_, AssetPropertyModel>(
            "SELECT * FROM asset_property WHERE asset_id = ? AND setter_id = ? AND property = ? AND latest = 1 LIMIT 1"
        )
        .bind(asset_id)
        .bind(setter_id)
        .bind(property)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn delete_by_asset_account_property(&self, asset_id: i64, setter_id: i64, property: &str) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM asset_property WHERE asset_id = ? AND setter_id = ? AND property = ?")
            .bind(asset_id)
            .bind(setter_id)
            .bind(property)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<AssetPropertyModel> for SqliteAssetPropertyRepository {
    async fn insert(&self, prop: &AssetPropertyModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO asset_property (id, asset_id, setter_id, property, value, height, latest)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(prop.id)
        .bind(prop.asset_id)
        .bind(prop.setter_id)
        .bind(&prop.property)
        .bind(&prop.value)
        .bind(prop.height)
        .bind(prop.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AssetPropertyModel>> {
        let record = sqlx::query_as::<_, AssetPropertyModel>("SELECT * FROM asset_property WHERE db_id = ?")
            .bind(db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &AssetPropertyModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for asset_property".to_string()))
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM asset_property WHERE db_id = ?")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AssetPropertyModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, AssetPropertyModel>(
            "SELECT * FROM asset_property WHERE latest = 1 ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM asset_property WHERE latest = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteAssetHistoryRepository {
    pool: SqlitePool,
}

impl SqliteAssetHistoryRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AssetHistoryRepository for SqliteAssetHistoryRepository {
    async fn find_by_asset(&self, asset_id: i64, limit: i64) -> RepositoryResult<Vec<AssetHistoryModel>> {
        let records = sqlx::query_as::<_, AssetHistoryModel>(
            "SELECT * FROM asset_history WHERE asset_id = ? ORDER BY height DESC LIMIT ?"
        )
        .bind(asset_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_asset_and_account(&self, asset_id: i64, account_id: i64, limit: i64) -> RepositoryResult<Vec<AssetHistoryModel>> {
        let records = sqlx::query_as::<_, AssetHistoryModel>(
            "SELECT * FROM asset_history WHERE asset_id = ? AND account_id = ? ORDER BY height DESC LIMIT ?"
        )
        .bind(asset_id)
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

#[async_trait]
impl Repository<AssetHistoryModel> for SqliteAssetHistoryRepository {
    async fn insert(&self, history: &AssetHistoryModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO asset_history (id, full_hash, asset_id, account_id, quantity, timestamp, chain_id, height)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(history.id)
        .bind(&history.full_hash)
        .bind(history.asset_id)
        .bind(history.account_id)
        .bind(history.quantity)
        .bind(history.timestamp)
        .bind(history.chain_id)
        .bind(history.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AssetHistoryModel>> {
        let record = sqlx::query_as::<_, AssetHistoryModel>("SELECT * FROM asset_history WHERE db_id = ?")
            .bind(db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &AssetHistoryModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for asset_history".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for asset_history".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AssetHistoryModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, AssetHistoryModel>(
            "SELECT * FROM asset_history ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM asset_history")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteTaggedDataTagRepository {
    pool: SqlitePool,
}

impl SqliteTaggedDataTagRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TaggedDataTagRepository for SqliteTaggedDataTagRepository {
    async fn find_by_tag(&self, tag: &str, limit: i64) -> RepositoryResult<Vec<TaggedDataTagModel>> {
        let records = sqlx::query_as::<_, TaggedDataTagModel>(
            "SELECT * FROM tagged_data_tag WHERE tag = ? ORDER BY height DESC LIMIT ?"
        )
        .bind(tag)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_data_id(&self, id: i64) -> RepositoryResult<Vec<TaggedDataTagModel>> {
        let records = sqlx::query_as::<_, TaggedDataTagModel>(
            "SELECT * FROM tagged_data_tag WHERE id = ? ORDER BY height DESC"
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

#[async_trait]
impl Repository<TaggedDataTagModel> for SqliteTaggedDataTagRepository {
    async fn insert(&self, tag: &TaggedDataTagModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO tagged_data_tag (id, tag, height, latest)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(tag.id)
        .bind(&tag.tag)
        .bind(tag.height)
        .bind(tag.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<TaggedDataTagModel>> {
        let record = sqlx::query_as::<_, TaggedDataTagModel>("SELECT * FROM tagged_data_tag WHERE db_id = ?")
            .bind(db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &TaggedDataTagModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for tagged_data_tag".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for tagged_data_tag".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<TaggedDataTagModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, TaggedDataTagModel>(
            "SELECT * FROM tagged_data_tag WHERE latest = 1 ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tagged_data_tag WHERE latest = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct SqliteTaggedTimestampRepository {
    pool: SqlitePool,
}

impl SqliteTaggedTimestampRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TaggedTimestampRepository for SqliteTaggedTimestampRepository {
    async fn find_by_account(&self, account_id: i64, limit: i64) -> RepositoryResult<Vec<TaggedTimestampModel>> {
        let records = sqlx::query_as::<_, TaggedTimestampModel>(
            "SELECT * FROM tagged_timestamp WHERE account_id = ? ORDER BY height DESC LIMIT ?"
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_tag(&self, tag: &str, limit: i64) -> RepositoryResult<Vec<TaggedTimestampModel>> {
        let records = sqlx::query_as::<_, TaggedTimestampModel>(
            "SELECT * FROM tagged_timestamp WHERE tag = ? ORDER BY height DESC LIMIT ?"
        )
        .bind(tag)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_account_and_tag(&self, account_id: i64, tag: &str) -> RepositoryResult<Option<TaggedTimestampModel>> {
        let record = sqlx::query_as::<_, TaggedTimestampModel>(
            "SELECT * FROM tagged_timestamp WHERE account_id = ? AND tag = ? LIMIT 1"
        )
        .bind(account_id)
        .bind(tag)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }
}

#[async_trait]
impl Repository<TaggedTimestampModel> for SqliteTaggedTimestampRepository {
    async fn insert(&self, ts: &TaggedTimestampModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO tagged_timestamp (id, account_id, tag, timestamp, height, latest)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(ts.id)
        .bind(ts.account_id)
        .bind(&ts.tag)
        .bind(ts.timestamp)
        .bind(ts.height)
        .bind(ts.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<TaggedTimestampModel>> {
        let record = sqlx::query_as::<_, TaggedTimestampModel>("SELECT * FROM tagged_timestamp WHERE db_id = ?")
            .bind(db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &TaggedTimestampModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for tagged_timestamp".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for tagged_timestamp".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<TaggedTimestampModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, TaggedTimestampModel>(
            "SELECT * FROM tagged_timestamp WHERE latest = 1 ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tagged_timestamp WHERE latest = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

// ============================================================================
// TaggedDataExtendRepository
// ============================================================================

pub struct SqliteTaggedDataExtendRepository {
    pool: SqlitePool,
}

impl SqliteTaggedDataExtendRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<TaggedDataExtendModel> for SqliteTaggedDataExtendRepository {
    async fn insert(&self, model: &TaggedDataExtendModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO tagged_data_extend (id, extend_id, height, latest)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(model.id)
        .bind(model.extend_id)
        .bind(model.height)
        .bind(model.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<TaggedDataExtendModel>> {
        let record = sqlx::query_as::<_, TaggedDataExtendModel>("SELECT * FROM tagged_data_extend WHERE db_id = ?")
            .bind(db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &TaggedDataExtendModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for tagged_data_extend".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for tagged_data_extend".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<TaggedDataExtendModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, TaggedDataExtendModel>(
            "SELECT * FROM tagged_data_extend WHERE latest = 1 ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tagged_data_extend WHERE latest = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl TaggedDataExtendRepository for SqliteTaggedDataExtendRepository {
    async fn find_by_extend_id(&self, extend_id: i64) -> RepositoryResult<Vec<TaggedDataExtendModel>> {
        let records = sqlx::query_as::<_, TaggedDataExtendModel>(
            "SELECT * FROM tagged_data_extend WHERE extend_id = ? AND latest = 1 ORDER BY height DESC"
        )
        .bind(extend_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

pub struct SqliteAccountGuaranteedBalanceRepository {
    pool: SqlitePool,
}

impl SqliteAccountGuaranteedBalanceRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<AccountGuaranteedBalanceModel> for SqliteAccountGuaranteedBalanceRepository {
    async fn insert(&self, item: &AccountGuaranteedBalanceModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO account_guaranteed_balance (account_id, additions, height)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(item.account_id)
        .bind(item.additions)
        .bind(item.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AccountGuaranteedBalanceModel>> {
        let record = sqlx::query_as::<_, AccountGuaranteedBalanceModel>(
            "SELECT * FROM account_guaranteed_balance WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, item: &AccountGuaranteedBalanceModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account_guaranteed_balance
            SET account_id = ?, additions = ?, height = ?
            WHERE db_id = ?
            "#,
        )
        .bind(item.account_id)
        .bind(item.additions)
        .bind(item.height)
        .bind(item.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM account_guaranteed_balance WHERE db_id = ?")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AccountGuaranteedBalanceModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, AccountGuaranteedBalanceModel>(
            "SELECT * FROM account_guaranteed_balance ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM account_guaranteed_balance")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl AccountGuaranteedBalanceRepository for SqliteAccountGuaranteedBalanceRepository {
    async fn find_by_account_and_height(&self, account_id: i64, height: i32) -> RepositoryResult<Option<AccountGuaranteedBalanceModel>> {
        let record = sqlx::query_as::<_, AccountGuaranteedBalanceModel>(
            "SELECT * FROM account_guaranteed_balance WHERE account_id = ? AND height = ?"
        )
        .bind(account_id)
        .bind(height)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn upsert_additions(&self, account_id: i64, height: i32, additions: i64) -> RepositoryResult<()> {
        if additions <= 0 {
            return Ok(());
        }

        match self.find_by_account_and_height(account_id, height).await? {
            Some(mut existing) => {
                existing.additions += additions;
                self.update(&existing).await?;
            }
            None => {
                let new_record = AccountGuaranteedBalanceModel {
                    db_id: 0,
                    account_id,
                    additions,
                    height,
                };
                self.insert(&new_record).await?;
            }
        }
        Ok(())
    }

    async fn get_total_additions_since(&self, account_id: i64, since_height: i32, current_height: i32) -> RepositoryResult<i64> {
        let (total,): (i64,) = sqlx::query_as(
            "SELECT COALESCE(SUM(additions), 0) FROM account_guaranteed_balance \
             WHERE account_id = ? AND height >= ? AND height <= ?"
        )
        .bind(account_id)
        .bind(since_height)
        .bind(current_height)
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(total)
    }
}

// ==================== ExchangeRequest Repository ====================

pub struct SqliteExchangeRequestRepository {
    pool: SqlitePool,
}

impl SqliteExchangeRequestRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<ExchangeRequestModel> for SqliteExchangeRequestRepository {
    async fn insert(&self, request: &ExchangeRequestModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO exchange_request (id, account_id, currency_id, units, rate, is_buy, timestamp, height)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(request.id)
        .bind(request.account_id)
        .bind(request.currency_id)
        .bind(request.units)
        .bind(request.rate)
        .bind(request.is_buy)
        .bind(request.timestamp)
        .bind(request.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<ExchangeRequestModel>> {
        let record = sqlx::query_as::<_, ExchangeRequestModel>(
            "SELECT * FROM exchange_request WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, request: &ExchangeRequestModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE exchange_request SET
                account_id = ?, currency_id = ?, units = ?, rate = ?,
                is_buy = ?, timestamp = ?, height = ?
            WHERE db_id = ?
            "#,
        )
        .bind(request.account_id)
        .bind(request.currency_id)
        .bind(request.units)
        .bind(request.rate)
        .bind(request.is_buy)
        .bind(request.timestamp)
        .bind(request.height)
        .bind(request.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM exchange_request WHERE db_id = ?")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<ExchangeRequestModel>> {
        let records = sqlx::query_as::<_, ExchangeRequestModel>(
            "SELECT * FROM exchange_request ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit.unwrap_or(100))
        .bind(offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM exchange_request")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

// ==================== CurrencyMint Repository ====================

pub struct SqliteCurrencyMintRepository {
    pool: SqlitePool,
}

impl SqliteCurrencyMintRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<CurrencyMintModel> for SqliteCurrencyMintRepository {
    async fn insert(&self, mint: &CurrencyMintModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO currency_mint (currency_id, account_id, counter, height, latest)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(mint.currency_id)
        .bind(mint.account_id)
        .bind(mint.counter)
        .bind(mint.height)
        .bind(mint.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<CurrencyMintModel>> {
        let record = sqlx::query_as::<_, CurrencyMintModel>(
            "SELECT * FROM currency_mint WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, mint: &CurrencyMintModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE currency_mint SET
                currency_id = ?, account_id = ?, counter = ?,
                height = ?, latest = ?
            WHERE db_id = ?
            "#,
        )
        .bind(mint.currency_id)
        .bind(mint.account_id)
        .bind(mint.counter)
        .bind(mint.height)
        .bind(mint.latest)
        .bind(mint.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM currency_mint WHERE db_id = ?")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<CurrencyMintModel>> {
        let records = sqlx::query_as::<_, CurrencyMintModel>(
            "SELECT * FROM currency_mint ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit.unwrap_or(100))
        .bind(offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM currency_mint")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

// ============================================================================
// AccountInfoRepository
// ============================================================================

pub struct SqliteAccountInfoRepository {
    pool: SqlitePool,
}

impl SqliteAccountInfoRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<AccountInfoModel> for SqliteAccountInfoRepository {
    async fn insert(&self, model: &AccountInfoModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO account_info (account_id, name, description, height, latest)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(model.account_id)
        .bind(&model.name)
        .bind(&model.description)
        .bind(model.height)
        .bind(model.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AccountInfoModel>> {
        let record = sqlx::query_as::<_, AccountInfoModel>(
            "SELECT * FROM account_info WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, model: &AccountInfoModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account_info SET
                name = ?, description = ?, height = ?, latest = ?
            WHERE db_id = ?
            "#,
        )
        .bind(&model.name)
        .bind(&model.description)
        .bind(model.height)
        .bind(model.latest)
        .bind(model.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM account_info WHERE db_id = ?")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AccountInfoModel>> {
        let records = sqlx::query_as::<_, AccountInfoModel>(
            "SELECT * FROM account_info ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit.unwrap_or(100))
        .bind(offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM account_info")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl AccountInfoRepository for SqliteAccountInfoRepository {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Option<AccountInfoModel>> {
        let record = sqlx::query_as::<_, AccountInfoModel>(
            "SELECT * FROM account_info WHERE account_id = ? AND latest = 1"
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn upsert(&self, model: &AccountInfoModel) -> RepositoryResult<()> {
        // Mark old entries as not latest
        sqlx::query(
            "UPDATE account_info SET latest = 0 WHERE account_id = ? AND latest = 1"
        )
        .bind(model.account_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        // Insert new entry
        self.insert(model).await
    }
}

// ============================================================================
// AccountLeaseRepository
// ============================================================================

pub struct SqliteAccountLeaseRepository {
    pool: SqlitePool,
}

impl SqliteAccountLeaseRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<AccountLeaseModel> for SqliteAccountLeaseRepository {
    async fn insert(&self, model: &AccountLeaseModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO account_lease (
                lessor_id, current_leasing_height_from, current_leasing_height_to,
                current_lessee_id, next_leasing_height_from, next_leasing_height_to,
                next_lessee_id, height, latest
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(model.lessor_id)
        .bind(model.current_leasing_height_from)
        .bind(model.current_leasing_height_to)
        .bind(model.current_lessee_id)
        .bind(model.next_leasing_height_from)
        .bind(model.next_leasing_height_to)
        .bind(model.next_lessee_id)
        .bind(model.height)
        .bind(model.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AccountLeaseModel>> {
        let record = sqlx::query_as::<_, AccountLeaseModel>(
            "SELECT * FROM account_lease WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, model: &AccountLeaseModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account_lease SET
                current_leasing_height_from = ?, current_leasing_height_to = ?,
                current_lessee_id = ?, next_leasing_height_from = ?,
                next_leasing_height_to = ?, next_lessee_id = ?,
                height = ?, latest = ?
            WHERE db_id = ?
            "#,
        )
        .bind(model.current_leasing_height_from)
        .bind(model.current_leasing_height_to)
        .bind(model.current_lessee_id)
        .bind(model.next_leasing_height_from)
        .bind(model.next_leasing_height_to)
        .bind(model.next_lessee_id)
        .bind(model.height)
        .bind(model.latest)
        .bind(model.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM account_lease WHERE db_id = ?")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AccountLeaseModel>> {
        let records = sqlx::query_as::<_, AccountLeaseModel>(
            "SELECT * FROM account_lease ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit.unwrap_or(100))
        .bind(offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM account_lease")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl AccountLeaseRepository for SqliteAccountLeaseRepository {
    async fn find_by_account(&self, lessor_id: i64) -> RepositoryResult<Option<AccountLeaseModel>> {
        let record = sqlx::query_as::<_, AccountLeaseModel>(
            "SELECT * FROM account_lease WHERE lessor_id = ? AND latest = 1"
        )
        .bind(lessor_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn upsert(&self, model: &AccountLeaseModel) -> RepositoryResult<()> {
        // Mark old entries as not latest
        sqlx::query(
            "UPDATE account_lease SET latest = 0 WHERE lessor_id = ? AND latest = 1"
        )
        .bind(model.lessor_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        // Insert new entry
        self.insert(model).await
    }
}

// ============================================================================
// AccountPropertyRepository
// ============================================================================

pub struct SqliteAccountPropertyRepository {
    pool: SqlitePool,
}

impl SqliteAccountPropertyRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<AccountPropertyModel> for SqliteAccountPropertyRepository {
    async fn insert(&self, model: &AccountPropertyModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO account_property (id, recipient_id, setter_id, property, value, height, latest)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(model.id)
        .bind(model.recipient_id)
        .bind(model.setter_id)
        .bind(&model.property)
        .bind(&model.value)
        .bind(model.height)
        .bind(model.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AccountPropertyModel>> {
        let record = sqlx::query_as::<_, AccountPropertyModel>(
            "SELECT * FROM account_property WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, model: &AccountPropertyModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account_property SET
                recipient_id = ?, setter_id = ?, property = ?, value = ?,
                height = ?, latest = ?
            WHERE db_id = ?
            "#,
        )
        .bind(model.recipient_id)
        .bind(model.setter_id)
        .bind(&model.property)
        .bind(&model.value)
        .bind(model.height)
        .bind(model.latest)
        .bind(model.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM account_property WHERE db_id = ?")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AccountPropertyModel>> {
        let records = sqlx::query_as::<_, AccountPropertyModel>(
            "SELECT * FROM account_property ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit.unwrap_or(100))
        .bind(offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM account_property")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl AccountPropertyRepository for SqliteAccountPropertyRepository {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<AccountPropertyModel>> {
        let records = sqlx::query_as::<_, AccountPropertyModel>(
            "SELECT * FROM account_property WHERE recipient_id = ? AND latest = 1"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_property(&self, account_id: i64, property: &str) -> RepositoryResult<Option<AccountPropertyModel>> {
        let record = sqlx::query_as::<_, AccountPropertyModel>(
            "SELECT * FROM account_property WHERE recipient_id = ? AND property = ? AND latest = 1"
        )
        .bind(account_id)
        .bind(property)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn upsert(&self, model: &AccountPropertyModel) -> RepositoryResult<()> {
        // Mark old entries as not latest
        sqlx::query(
            "UPDATE account_property SET latest = 0 WHERE recipient_id = ? AND property = ? AND latest = 1"
        )
        .bind(model.recipient_id)
        .bind(&model.property)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        // Insert new entry
        self.insert(model).await
    }

    async fn delete_by_id(&self, id: i64) -> RepositoryResult<()> {
        sqlx::query("UPDATE account_property SET latest = 0 WHERE id = ? AND latest = 1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

// ============================================================================
// AccountControlPhasingRepository
// ============================================================================

pub struct SqliteAccountControlPhasingRepository {
    pool: SqlitePool,
}

impl SqliteAccountControlPhasingRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<AccountControlPhasingModel> for SqliteAccountControlPhasingRepository {
    async fn insert(&self, model: &AccountControlPhasingModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO account_control_phasing (
                account_id, whitelist, voting_model, quorum, min_balance,
                holding_id, min_balance_model, max_fees, min_duration,
                max_duration, height, latest
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(model.account_id)
        .bind(&model.whitelist)
        .bind(model.voting_model)
        .bind(model.quorum)
        .bind(model.min_balance)
        .bind(model.holding_id)
        .bind(model.min_balance_model)
        .bind(model.max_fees)
        .bind(model.min_duration)
        .bind(model.max_duration)
        .bind(model.height)
        .bind(model.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AccountControlPhasingModel>> {
        let record = sqlx::query_as::<_, AccountControlPhasingModel>(
            "SELECT * FROM account_control_phasing WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, model: &AccountControlPhasingModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account_control_phasing SET
                whitelist = ?, voting_model = ?, quorum = ?, min_balance = ?,
                holding_id = ?, min_balance_model = ?, max_fees = ?,
                min_duration = ?, max_duration = ?, height = ?, latest = ?
            WHERE db_id = ?
            "#,
        )
        .bind(&model.whitelist)
        .bind(model.voting_model)
        .bind(model.quorum)
        .bind(model.min_balance)
        .bind(model.holding_id)
        .bind(model.min_balance_model)
        .bind(model.max_fees)
        .bind(model.min_duration)
        .bind(model.max_duration)
        .bind(model.height)
        .bind(model.latest)
        .bind(model.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM account_control_phasing WHERE db_id = ?")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AccountControlPhasingModel>> {
        let records = sqlx::query_as::<_, AccountControlPhasingModel>(
            "SELECT * FROM account_control_phasing ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit.unwrap_or(100))
        .bind(offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM account_control_phasing")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl AccountControlPhasingRepository for SqliteAccountControlPhasingRepository {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Option<AccountControlPhasingModel>> {
        let record = sqlx::query_as::<_, AccountControlPhasingModel>(
            "SELECT * FROM account_control_phasing WHERE account_id = ? AND latest = 1"
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn upsert(&self, model: &AccountControlPhasingModel) -> RepositoryResult<()> {
        // Mark old entries as not latest
        sqlx::query(
            "UPDATE account_control_phasing SET latest = 0 WHERE account_id = ? AND latest = 1"
        )
        .bind(model.account_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        // Insert new entry
        self.insert(model).await
    }
}

// ============================================================================
// AssetDeleteRepository
// ============================================================================

pub struct SqliteAssetDeleteRepository {
    pool: SqlitePool,
}

impl SqliteAssetDeleteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<AssetDeleteModel> for SqliteAssetDeleteRepository {
    async fn insert(&self, model: &AssetDeleteModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO asset_delete (asset_id, account_id, quantity, height)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(model.asset_id)
        .bind(model.account_id)
        .bind(model.quantity)
        .bind(model.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AssetDeleteModel>> {
        let record = sqlx::query_as::<_, AssetDeleteModel>(
            "SELECT * FROM asset_delete WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _model: &AssetDeleteModel) -> RepositoryResult<()> {
        // Asset deletes are immutable
        Ok(())
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        // Asset deletes are immutable
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AssetDeleteModel>> {
        let records = sqlx::query_as::<_, AssetDeleteModel>(
            "SELECT * FROM asset_delete ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit.unwrap_or(100))
        .bind(offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM asset_delete")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl AssetDeleteRepository for SqliteAssetDeleteRepository {
    async fn find_by_asset(&self, asset_id: i64) -> RepositoryResult<Vec<AssetDeleteModel>> {
        let records = sqlx::query_as::<_, AssetDeleteModel>(
            "SELECT * FROM asset_delete WHERE asset_id = ? ORDER BY height DESC"
        )
        .bind(asset_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

// ============================================================================
// AssetDividendRepository
// ============================================================================

pub struct SqliteAssetDividendRepository {
    pool: SqlitePool,
}

impl SqliteAssetDividendRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<AssetDividendModel> for SqliteAssetDividendRepository {
    async fn insert(&self, model: &AssetDividendModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO asset_dividend (id, asset_id, amount, dividend_height, total_dividend, num_accounts, timestamp, height)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(model.id)
        .bind(model.asset_id)
        .bind(model.amount)
        .bind(model.dividend_height)
        .bind(model.total_dividend)
        .bind(model.num_accounts)
        .bind(model.timestamp)
        .bind(model.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AssetDividendModel>> {
        let record = sqlx::query_as::<_, AssetDividendModel>(
            "SELECT * FROM asset_dividend WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _model: &AssetDividendModel) -> RepositoryResult<()> {
        // Dividends are immutable
        Ok(())
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        // Dividends are immutable
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AssetDividendModel>> {
        let records = sqlx::query_as::<_, AssetDividendModel>(
            "SELECT * FROM asset_dividend ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit.unwrap_or(100))
        .bind(offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM asset_dividend")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl AssetDividendRepository for SqliteAssetDividendRepository {
    async fn find_by_asset(&self, asset_id: i64) -> RepositoryResult<Vec<AssetDividendModel>> {
        let records = sqlx::query_as::<_, AssetDividendModel>(
            "SELECT * FROM asset_dividend WHERE asset_id = ? ORDER BY height DESC"
        )
        .bind(asset_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

// ============================================================================
// PhasingPollRepository
// ============================================================================

pub struct SqlitePhasingPollRepository {
    pool: SqlitePool,
}

impl SqlitePhasingPollRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<PhasingPollModel> for SqlitePhasingPollRepository {
    async fn insert(&self, model: &PhasingPollModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO phasing_poll (id, account_id, whitelist_size, finish_height, voting_model, quorum,
                min_balance, holding_id, min_balance_model, hashed_secret, algorithm, height)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(model.id)
        .bind(model.account_id)
        .bind(model.whitelist_size)
        .bind(model.finish_height)
        .bind(model.voting_model)
        .bind(model.quorum)
        .bind(model.min_balance)
        .bind(model.holding_id)
        .bind(model.min_balance_model)
        .bind(&model.hashed_secret)
        .bind(model.algorithm)
        .bind(model.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<PhasingPollModel>> {
        let record = sqlx::query_as::<_, PhasingPollModel>(
            "SELECT * FROM phasing_poll WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, model: &PhasingPollModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE phasing_poll SET
                account_id = ?, whitelist_size = ?, finish_height = ?, voting_model = ?, quorum = ?,
                min_balance = ?, holding_id = ?, min_balance_model = ?,
                hashed_secret = ?, algorithm = ?, height = ?
            WHERE db_id = ?
            "#,
        )
        .bind(model.account_id)
        .bind(model.whitelist_size)
        .bind(model.finish_height)
        .bind(model.voting_model)
        .bind(model.quorum)
        .bind(model.min_balance)
        .bind(model.holding_id)
        .bind(model.min_balance_model)
        .bind(&model.hashed_secret)
        .bind(model.algorithm)
        .bind(model.height)
        .bind(model.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM phasing_poll WHERE db_id = ?")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PhasingPollModel>> {
        let records = sqlx::query_as::<_, PhasingPollModel>(
            "SELECT * FROM phasing_poll ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit.unwrap_or(100))
        .bind(offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM phasing_poll")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl PhasingPollRepository for SqlitePhasingPollRepository {
    async fn find_by_poll_id(&self, id: i64) -> RepositoryResult<Option<PhasingPollModel>> {
        let record = sqlx::query_as::<_, PhasingPollModel>(
            "SELECT * FROM phasing_poll WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn upsert(&self, model: &PhasingPollModel) -> RepositoryResult<()> {
        // Try to find existing record
        if let Some(existing) = self.find_by_poll_id(model.id).await? {
            // Update existing
            let mut updated = model.clone();
            updated.db_id = existing.db_id;
            self.update(&updated).await
        } else {
            // Insert new
            self.insert(model).await
        }
    }
}

// ============================================================================
// PhasingPollLinkedTransactionRepository
// ============================================================================

pub struct SqlitePhasingPollLinkedTransactionRepository {
    pool: SqlitePool,
}

impl SqlitePhasingPollLinkedTransactionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<PhasingPollLinkedTransactionModel> for SqlitePhasingPollLinkedTransactionRepository {
    async fn insert(&self, model: &PhasingPollLinkedTransactionModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO phasing_poll_linked_transaction (transaction_id, linked_full_hash, linked_transaction_id, height)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(model.transaction_id)
        .bind(&model.linked_full_hash)
        .bind(model.linked_transaction_id)
        .bind(model.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<PhasingPollLinkedTransactionModel>> {
        let record = sqlx::query_as::<_, PhasingPollLinkedTransactionModel>(
            "SELECT * FROM phasing_poll_linked_transaction WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _model: &PhasingPollLinkedTransactionModel) -> RepositoryResult<()> {
        // Linked transactions are immutable
        Ok(())
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        // Linked transactions are immutable
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PhasingPollLinkedTransactionModel>> {
        let records = sqlx::query_as::<_, PhasingPollLinkedTransactionModel>(
            "SELECT * FROM phasing_poll_linked_transaction ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit.unwrap_or(100))
        .bind(offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM phasing_poll_linked_transaction")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl PhasingPollLinkedTransactionRepository for SqlitePhasingPollLinkedTransactionRepository {
    async fn find_by_poll(&self, poll_id: i64) -> RepositoryResult<Vec<PhasingPollLinkedTransactionModel>> {
        let records = sqlx::query_as::<_, PhasingPollLinkedTransactionModel>(
            "SELECT * FROM phasing_poll_linked_transaction WHERE transaction_id = ?"
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

// ============================================================================
// PhasingPollResultRepository
// ============================================================================

pub struct SqlitePhasingPollResultRepository {
    pool: SqlitePool,
}

impl SqlitePhasingPollResultRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<PhasingPollResultModel> for SqlitePhasingPollResultRepository {
    async fn insert(&self, model: &PhasingPollResultModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO phasing_poll_result (id, result, approved, height)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(model.id)
        .bind(model.result)
        .bind(model.approved)
        .bind(model.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<PhasingPollResultModel>> {
        let record = sqlx::query_as::<_, PhasingPollResultModel>(
            "SELECT * FROM phasing_poll_result WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, model: &PhasingPollResultModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE phasing_poll_result SET
                id = ?, result = ?, approved = ?, height = ?
            WHERE db_id = ?
            "#,
        )
        .bind(model.id)
        .bind(model.result)
        .bind(model.approved)
        .bind(model.height)
        .bind(model.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM phasing_poll_result WHERE db_id = ?")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PhasingPollResultModel>> {
        let records = sqlx::query_as::<_, PhasingPollResultModel>(
            "SELECT * FROM phasing_poll_result ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit.unwrap_or(100))
        .bind(offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM phasing_poll_result")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl PhasingPollResultRepository for SqlitePhasingPollResultRepository {
    async fn find_by_poll(&self, poll_id: i64) -> RepositoryResult<Option<PhasingPollResultModel>> {
        let record = sqlx::query_as::<_, PhasingPollResultModel>(
            "SELECT * FROM phasing_poll_result WHERE id = ?"
        )
        .bind(poll_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn upsert(&self, model: &PhasingPollResultModel) -> RepositoryResult<()> {
        if let Some(existing) = self.find_by_poll(model.id).await? {
            let mut updated = model.clone();
            updated.db_id = existing.db_id;
            self.update(&updated).await
        } else {
            self.insert(model).await
        }
    }
}

// ============================================================================
// PhasingPollVoterRepository
// ============================================================================

pub struct SqlitePhasingPollVoterRepository {
    pool: SqlitePool,
}

impl SqlitePhasingPollVoterRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<PhasingPollVoterModel> for SqlitePhasingPollVoterRepository {
    async fn insert(&self, model: &PhasingPollVoterModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO phasing_poll_voter (transaction_id, voter_id, height)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(model.transaction_id)
        .bind(model.voter_id)
        .bind(model.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<PhasingPollVoterModel>> {
        let record = sqlx::query_as::<_, PhasingPollVoterModel>(
            "SELECT * FROM phasing_poll_voter WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _model: &PhasingPollVoterModel) -> RepositoryResult<()> {
        // Voters are immutable
        Ok(())
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        // Voters are immutable
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PhasingPollVoterModel>> {
        let records = sqlx::query_as::<_, PhasingPollVoterModel>(
            "SELECT * FROM phasing_poll_voter ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit.unwrap_or(100))
        .bind(offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM phasing_poll_voter")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl PhasingPollVoterRepository for SqlitePhasingPollVoterRepository {
    async fn find_by_poll(&self, poll_id: i64) -> RepositoryResult<Vec<PhasingPollVoterModel>> {
        let records = sqlx::query_as::<_, PhasingPollVoterModel>(
            "SELECT * FROM phasing_poll_voter WHERE transaction_id = ?"
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

// ============================================================================
// PhasingVoteRepository
// ============================================================================

pub struct SqlitePhasingVoteRepository {
    pool: SqlitePool,
}

impl SqlitePhasingVoteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<PhasingVoteModel> for SqlitePhasingVoteRepository {
    async fn insert(&self, model: &PhasingVoteModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO phasing_vote (vote_id, transaction_id, voter_id, height)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(model.vote_id)
        .bind(model.transaction_id)
        .bind(model.voter_id)
        .bind(model.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<PhasingVoteModel>> {
        let record = sqlx::query_as::<_, PhasingVoteModel>(
            "SELECT * FROM phasing_vote WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _model: &PhasingVoteModel) -> RepositoryResult<()> {
        // Votes are immutable
        Ok(())
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        // Votes are immutable
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PhasingVoteModel>> {
        let records = sqlx::query_as::<_, PhasingVoteModel>(
            "SELECT * FROM phasing_vote ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit.unwrap_or(100))
        .bind(offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM phasing_vote")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl PhasingVoteRepository for SqlitePhasingVoteRepository {
    async fn find_by_poll(&self, poll_id: i64) -> RepositoryResult<Vec<PhasingVoteModel>> {
        let records = sqlx::query_as::<_, PhasingVoteModel>(
            "SELECT * FROM phasing_vote WHERE transaction_id = ?"
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

// ============================================================================
// PollResultRepository
// ============================================================================

pub struct SqlitePollResultRepository {
    pool: SqlitePool,
}

impl SqlitePollResultRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<PollResultModel> for SqlitePollResultRepository {
    async fn insert(&self, model: &PollResultModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO poll_result (poll_id, result, weight, height)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(model.poll_id)
        .bind(model.result)
        .bind(model.weight)
        .bind(model.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<PollResultModel>> {
        let record = sqlx::query_as::<_, PollResultModel>(
            "SELECT * FROM poll_result WHERE db_id = ?"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, model: &PollResultModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE poll_result SET
                poll_id = ?, result = ?, height = ?
            WHERE db_id = ?
            "#,
        )
        .bind(model.poll_id)
        .bind(model.result)
        .bind(model.height)
        .bind(model.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM poll_result WHERE db_id = ?")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PollResultModel>> {
        let records = sqlx::query_as::<_, PollResultModel>(
            "SELECT * FROM poll_result ORDER BY height DESC LIMIT ? OFFSET ?"
        )
        .bind(limit.unwrap_or(100))
        .bind(offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM poll_result")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl PollResultRepository for SqlitePollResultRepository {
    async fn find_by_poll(&self, poll_id: i64) -> RepositoryResult<Vec<PollResultModel>> {
        let records = sqlx::query_as::<_, PollResultModel>(
            "SELECT * FROM poll_result WHERE poll_id = ?"
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn upsert(&self, model: &PollResultModel) -> RepositoryResult<()> {
        // Find existing by poll_id and result
        let existing = sqlx::query_as::<_, PollResultModel>(
            "SELECT * FROM poll_result WHERE poll_id = ? AND result = ?"
        )
        .bind(model.poll_id)
        .bind(model.result)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        if let Some(existing) = existing {
            let mut updated = model.clone();
            updated.db_id = existing.db_id;
            self.update(&updated).await
        } else {
            self.insert(model).await
        }
    }
}
