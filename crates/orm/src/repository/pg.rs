use async_trait::async_trait;
use sqlx::PgPool;

use crate::models::*;
use crate::connection::DbTransaction;
use super::traits::*;
use super::public_key::PublicKeyRepository;
use blockchain_types::account_ext::AccountPublicKey;
use blockchain_types::{AccountId, Height};

pub struct PgBlockRepository {
    pool: PgPool,
}

impl PgBlockRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ==================== Shuffling Sub-table Repositories (PostgreSQL) ====================

pub struct PgShufflingDataRepository {
    pool: PgPool,
}

impl PgShufflingDataRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<ShufflingDataModel> for PgShufflingDataRepository {
    async fn insert(&self, model: &ShufflingDataModel) -> RepositoryResult<()> {
        sqlx::query(
            "INSERT INTO shuffling_data (shuffling_id, account_id, data, transaction_timestamp, height) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(model.shuffling_id)
        .bind(model.account_id)
        .bind(&model.data)
        .bind(model.transaction_timestamp)
        .bind(model.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<ShufflingDataModel>> {
        let record = sqlx::query_as::<_, ShufflingDataModel>(
            "SELECT * FROM shuffling_data WHERE db_id = $1"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, model: &ShufflingDataModel) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE shuffling_data SET shuffling_id = $1, account_id = $2, data = $3, transaction_timestamp = $4, height = $5 WHERE db_id = $6"
        )
        .bind(model.shuffling_id)
        .bind(model.account_id)
        .bind(&model.data)
        .bind(model.transaction_timestamp)
        .bind(model.height)
        .bind(model.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM shuffling_data WHERE db_id = $1")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<ShufflingDataModel>> {
        let lim = limit.unwrap_or(100);
        let off = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, ShufflingDataModel>(
            "SELECT * FROM shuffling_data ORDER BY height DESC LIMIT $1 OFFSET $2"
        )
        .bind(lim)
        .bind(off)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM shuffling_data")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl ShufflingDataRepository for PgShufflingDataRepository {
    async fn find_by_shuffling(&self, shuffling_id: i64) -> RepositoryResult<Vec<ShufflingDataModel>> {
        let records = sqlx::query_as::<_, ShufflingDataModel>(
            "SELECT * FROM shuffling_data WHERE shuffling_id = $1"
        )
        .bind(shuffling_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

pub struct PgShufflingParticipantRepository {
    pool: PgPool,
}

impl PgShufflingParticipantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<ShufflingParticipantModel> for PgShufflingParticipantRepository {
    async fn insert(&self, model: &ShufflingParticipantModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"INSERT INTO shuffling_participant (
                shuffling_id, account_id, next_account_id, participant_index,
                state, blame_data, key_seeds, data_transaction_full_hash, height, latest
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#
        )
        .bind(model.shuffling_id)
        .bind(model.account_id)
        .bind(model.next_account_id)
        .bind(model.participant_index)
        .bind(model.state)
        .bind(&model.blame_data)
        .bind(&model.key_seeds)
        .bind(&model.data_transaction_full_hash)
        .bind(model.height)
        .bind(model.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<ShufflingParticipantModel>> {
        let record = sqlx::query_as::<_, ShufflingParticipantModel>(
            "SELECT * FROM shuffling_participant WHERE db_id = $1"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, model: &ShufflingParticipantModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"UPDATE shuffling_participant SET
                shuffling_id = $1, account_id = $2, next_account_id = $3,
                participant_index = $4, state = $5, blame_data = $6,
                key_seeds = $7, data_transaction_full_hash = $8, height = $9, latest = $10
            WHERE db_id = $11"#
        )
        .bind(model.shuffling_id)
        .bind(model.account_id)
        .bind(model.next_account_id)
        .bind(model.participant_index)
        .bind(model.state)
        .bind(&model.blame_data)
        .bind(&model.key_seeds)
        .bind(&model.data_transaction_full_hash)
        .bind(model.height)
        .bind(model.latest)
        .bind(model.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM shuffling_participant WHERE db_id = $1")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<ShufflingParticipantModel>> {
        let lim = limit.unwrap_or(100);
        let off = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, ShufflingParticipantModel>(
            "SELECT * FROM shuffling_participant WHERE latest = true ORDER BY height DESC LIMIT $1 OFFSET $2"
        )
        .bind(lim)
        .bind(off)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM shuffling_participant WHERE latest = true")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl ShufflingParticipantRepository for PgShufflingParticipantRepository {
    async fn find_by_shuffling(&self, shuffling_id: i64) -> RepositoryResult<Vec<ShufflingParticipantModel>> {
        let records = sqlx::query_as::<_, ShufflingParticipantModel>(
            "SELECT * FROM shuffling_participant WHERE shuffling_id = $1 AND latest = true"
        )
        .bind(shuffling_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_account_and_shuffling(&self, account_id: i64, shuffling_id: i64) -> RepositoryResult<Option<ShufflingParticipantModel>> {
        let record = sqlx::query_as::<_, ShufflingParticipantModel>(
            "SELECT * FROM shuffling_participant WHERE account_id = $1 AND shuffling_id = $2 AND latest = true LIMIT 1"
        )
        .bind(account_id)
        .bind(shuffling_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }
}

#[async_trait]
impl BlockRepository for PgBlockRepository {
    async fn find_by_height(&self, height: i32) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as::<_, BlockModel>("SELECT * FROM block WHERE height = $1").bind(height)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_id_column(&self, id: i64) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as::<_, BlockModel>("SELECT * FROM block WHERE id = $1").bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_hash(&self, hash: &[u8]) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as::<_, BlockModel>("SELECT * FROM block WHERE payload_hash = $1 OR generation_signature = $1").bind(hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_latest(&self) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as::<_, BlockModel>("SELECT * FROM block ORDER BY height DESC LIMIT 1")
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_range(&self, start_height: i32, end_height: i32) -> RepositoryResult<Vec<BlockModel>> {
        let records = sqlx::query_as::<_, BlockModel>("SELECT * FROM block WHERE height BETWEEN $1 AND $2 ORDER BY height ASC").bind(start_height).bind(end_height)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_generator(&self, generator_id: i64) -> RepositoryResult<Vec<BlockModel>> {
        let records = sqlx::query_as::<_, BlockModel>("SELECT * FROM block WHERE generator_id = $1 ORDER BY height DESC").bind(generator_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn get_height(&self) -> RepositoryResult<i32> {
        let record = sqlx::query_as::<_, BlockModel>("SELECT * FROM block ORDER BY height DESC LIMIT 1")
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record.map(|b| b.height).unwrap_or(0))
    }

    async fn get_block_id_at_height(&self, height: i32) -> RepositoryResult<Option<i64>> {
        let record = sqlx::query_as::<_, BlockModel>("SELECT * FROM block WHERE height = $1").bind(height)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record.map(|b| b.id))
    }

    async fn has_block(&self, id: i64) -> RepositoryResult<bool> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM block WHERE id = $1"
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
        
        let records = sqlx::query_as::<_, BlockModel>("SELECT * FROM block WHERE height > $1 ORDER BY height ASC LIMIT $2").bind(height).bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        
        Ok(records.into_iter().map(|b| b.id).collect())
    }

    async fn update_next_block_id(&self, previous_block_id: i64, next_block_id: i64) -> RepositoryResult<()> {
        sqlx::query("UPDATE block SET next_block_id = $1 WHERE id = $2")
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
            sqlx::query("DELETE FROM block WHERE id = $1")
                .bind(block.id)
                .execute(&self.pool)
                .await
                .map_err(RepositoryError::DbError)?;
        }
        
        if let Some(last_block) = blocks.last() {
            sqlx::query("UPDATE block SET next_block_id = NULL WHERE id = $1")
                .bind(last_block.previous_block_id)
                .execute(&self.pool)
                .await
                .map_err(RepositoryError::DbError)?;
        }
        
        Ok(blocks)
    }

    async fn find_blocks_after_height(&self, height: i32) -> RepositoryResult<Vec<BlockModel>> {
        let records = sqlx::query_as::<_, BlockModel>("SELECT * FROM block WHERE height > $1 ORDER BY height ASC").bind(height)
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
            sqlx::query("DELETE FROM block WHERE db_id = $1")
                .bind(db_id)
                .execute(&self.pool)
                .await
                .map_err(RepositoryError::DbError)?;
        }
        Ok(())
    }

    async fn insert_tx(&self, block: &BlockModel, tx: &mut DbTransaction<'_>) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO block (
                id, version, timestamp, previous_block_id, total_amount,
                total_fee, payload_length, previous_block_hash, cumulative_difficulty,
                base_target, next_block_id, height, generation_signature,
                block_signature, payload_hash, generator_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
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
        .execute(&mut **tx)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn update_next_block_id_tx(&self, previous_block_id: i64, next_block_id: i64, tx: &mut DbTransaction<'_>) -> RepositoryResult<()> {
        sqlx::query("UPDATE block SET next_block_id = $1 WHERE id = $2")
            .bind(next_block_id)
            .bind(previous_block_id)
            .execute(&mut **tx)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete_by_db_id_tx(&self, db_id: i64, tx: &mut DbTransaction<'_>) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM block WHERE db_id = $1")
            .bind(db_id)
            .execute(&mut **tx)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<BlockModel> for PgBlockRepository {
    async fn insert(&self, block: &BlockModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO block (
                id, version, timestamp, previous_block_id, total_amount,
                total_fee, payload_length, previous_block_hash, cumulative_difficulty,
                base_target, next_block_id, height, generation_signature,
                block_signature, payload_hash, generator_id
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16
            )
            "#
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
        let record = sqlx::query_as::<_, BlockModel>("SELECT * FROM block WHERE db_id = $1").bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, block: &BlockModel) -> RepositoryResult<()> {
        sqlx::query(
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
        )
        .bind(block.db_id)
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

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM block WHERE db_id = $1")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<BlockModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, BlockModel>("SELECT * FROM block ORDER BY height DESC LIMIT $1 OFFSET $2").bind(limit).bind(offset)
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

pub struct PgPublicKeyRepository {
    pool: PgPool,
}

impl PgPublicKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PublicKeyRepository for PgPublicKeyRepository {
    async fn find_latest_by_account_id(&self, account_id: i64) -> RepositoryResult<Option<AccountPublicKey>> {
        let row: Option<(Vec<u8>, i32)> = sqlx::query_as(
            "SELECT public_key, height FROM public_key WHERE account_id = $1 AND latest = TRUE ORDER BY height DESC LIMIT 1"
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
            VALUES ($1, $2, $3)
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

pub struct PgAccountPropertyRepository {
    pool: PgPool,
}

impl PgAccountPropertyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<AccountPropertyModel> for PgAccountPropertyRepository {
    async fn insert(&self, model: &AccountPropertyModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO account_property (id, recipient_id, setter_id, property, value, height, latest)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
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
            "SELECT * FROM account_property WHERE db_id = $1"
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
                recipient_id = $1, setter_id = $2, property = $3, value = $4,
                height = $5, latest = $6
            WHERE db_id = $7
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
        sqlx::query("DELETE FROM account_property WHERE db_id = $1")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AccountPropertyModel>> {
        let records = sqlx::query_as::<_, AccountPropertyModel>(
            "SELECT * FROM account_property ORDER BY height DESC LIMIT $1 OFFSET $2"
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
impl AccountPropertyRepository for PgAccountPropertyRepository {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<AccountPropertyModel>> {
        let records = sqlx::query_as::<_, AccountPropertyModel>(
            "SELECT * FROM account_property WHERE recipient_id = $1 AND latest = TRUE"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_property(&self, account_id: i64, property: &str) -> RepositoryResult<Option<AccountPropertyModel>> {
        let record = sqlx::query_as::<_, AccountPropertyModel>(
            "SELECT * FROM account_property WHERE recipient_id = $1 AND property = $2 AND latest = TRUE"
        )
        .bind(account_id)
        .bind(property)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn upsert(&self, model: &AccountPropertyModel) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE account_property SET latest = FALSE WHERE recipient_id = $1 AND property = $2 AND latest = TRUE"
        )
        .bind(model.recipient_id)
        .bind(&model.property)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        self.insert(model).await
    }

    async fn delete_by_id(&self, id: i64) -> RepositoryResult<()> {
        sqlx::query("UPDATE account_property SET latest = FALSE WHERE id = $1 AND latest = TRUE")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn soft_delete_by_id(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("UPDATE account_property SET latest = FALSE WHERE db_id = $1 AND latest = TRUE")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

pub struct PgAccountControlPhasingRepository {
    pool: PgPool,
}

impl PgAccountControlPhasingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<AccountControlPhasingModel> for PgAccountControlPhasingRepository {
    async fn insert(&self, model: &AccountControlPhasingModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO account_control_phasing (
                account_id, whitelist, voting_model, quorum, min_balance,
                holding_id, min_balance_model, max_fees, min_duration,
                max_duration, height, latest
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
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
            "SELECT * FROM account_control_phasing WHERE db_id = $1"
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
                account_id = $1, whitelist = $2, voting_model = $3, quorum = $4,
                min_balance = $5, holding_id = $6, min_balance_model = $7,
                max_fees = $8, min_duration = $9, max_duration = $10,
                height = $11, latest = $12
            WHERE db_id = $13
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
        .bind(model.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM account_control_phasing WHERE db_id = $1")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AccountControlPhasingModel>> {
        let records = sqlx::query_as::<_, AccountControlPhasingModel>(
            "SELECT * FROM account_control_phasing ORDER BY height DESC LIMIT $1 OFFSET $2"
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
impl AccountControlPhasingRepository for PgAccountControlPhasingRepository {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Option<AccountControlPhasingModel>> {
        let record = sqlx::query_as::<_, AccountControlPhasingModel>(
            "SELECT * FROM account_control_phasing WHERE account_id = $1 AND latest = TRUE"
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn upsert(&self, model: &AccountControlPhasingModel) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE account_control_phasing SET latest = FALSE WHERE account_id = $1 AND latest = TRUE"
        )
        .bind(model.account_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        self.insert(model).await
    }
}

pub struct PgAccountInfoRepository {
    pool: PgPool,
}

impl PgAccountInfoRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<AccountInfoModel> for PgAccountInfoRepository {
    async fn insert(&self, model: &AccountInfoModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO account_info (account_id, name, description, height, latest)
            VALUES ($1, $2, $3, $4, $5)
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
            "SELECT * FROM account_info WHERE db_id = $1"
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
                name = $1, description = $2, height = $3, latest = $4
            WHERE db_id = $5
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
        sqlx::query("DELETE FROM account_info WHERE db_id = $1")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AccountInfoModel>> {
        let records = sqlx::query_as::<_, AccountInfoModel>(
            "SELECT * FROM account_info ORDER BY height DESC LIMIT $1 OFFSET $2"
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
impl AccountInfoRepository for PgAccountInfoRepository {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Option<AccountInfoModel>> {
        let record = sqlx::query_as::<_, AccountInfoModel>(
            "SELECT * FROM account_info WHERE account_id = $1 AND latest = TRUE"
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn upsert(&self, model: &AccountInfoModel) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE account_info SET latest = FALSE WHERE account_id = $1 AND latest = TRUE"
        )
        .bind(model.account_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        self.insert(model).await
    }
}

pub struct PgAccountLeaseRepository {
    pool: PgPool,
}

impl PgAccountLeaseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<AccountLeaseModel> for PgAccountLeaseRepository {
    async fn insert(&self, model: &AccountLeaseModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO account_lease (
                lessor_id, current_leasing_height_from, current_leasing_height_to,
                current_lessee_id, next_leasing_height_from, next_leasing_height_to,
                next_lessee_id, height, latest
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
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
            "SELECT * FROM account_lease WHERE db_id = $1"
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
                current_leasing_height_from = $1, current_leasing_height_to = $2,
                current_lessee_id = $3, next_leasing_height_from = $4,
                next_leasing_height_to = $5, next_lessee_id = $6,
                height = $7, latest = $8
            WHERE db_id = $9
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
        sqlx::query("DELETE FROM account_lease WHERE db_id = $1")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AccountLeaseModel>> {
        let records = sqlx::query_as::<_, AccountLeaseModel>(
            "SELECT * FROM account_lease ORDER BY height DESC LIMIT $1 OFFSET $2"
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
impl AccountLeaseRepository for PgAccountLeaseRepository {
    async fn find_by_account(&self, lessor_id: i64) -> RepositoryResult<Option<AccountLeaseModel>> {
        let record = sqlx::query_as::<_, AccountLeaseModel>(
            "SELECT * FROM account_lease WHERE lessor_id = $1 AND latest = TRUE"
        )
        .bind(lessor_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn upsert(&self, model: &AccountLeaseModel) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE account_lease SET latest = FALSE WHERE lessor_id = $1 AND latest = TRUE"
        )
        .bind(model.lessor_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        self.insert(model).await
    }
}

pub struct PgTradeRepository {
    pool: PgPool,
}

impl PgTradeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TradeRepository for PgTradeRepository {
    async fn find_by_asset(&self, asset_id: i64, limit: i64) -> RepositoryResult<Vec<TradeModel>> {
        let records = sqlx::query_as::<_, TradeModel>(
            "SELECT * FROM trade WHERE asset_id = $1 ORDER BY height DESC, timestamp DESC LIMIT $2"
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
            "SELECT * FROM trade WHERE ask_order_id = $1 ORDER BY height DESC"
        )
        .bind(ask_order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_bid_order(&self, bid_order_id: i64) -> RepositoryResult<Vec<TradeModel>> {
        let records = sqlx::query_as::<_, TradeModel>(
            "SELECT * FROM trade WHERE bid_order_id = $1 ORDER BY height DESC"
        )
        .bind(bid_order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_buyer(&self, buyer_id: i64, limit: i64) -> RepositoryResult<Vec<TradeModel>> {
        let records = sqlx::query_as::<_, TradeModel>(
            "SELECT * FROM trade WHERE buyer_id = $1 ORDER BY height DESC LIMIT $2"
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
            "SELECT * FROM trade WHERE seller_id = $1 ORDER BY height DESC LIMIT $2"
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
impl Repository<TradeModel> for PgTradeRepository {
    async fn insert(&self, trade: &TradeModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO trade (
                asset_id, block_id, ask_order_id, bid_order_id, ask_order_height,
                bid_order_height, seller_id, buyer_id, is_buy, quantity, price, timestamp, height
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
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
            "SELECT * FROM trade WHERE db_id = $1"
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
            "SELECT * FROM trade ORDER BY height DESC LIMIT $1 OFFSET $2"
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

pub struct PgGoodsRepository {
    pool: PgPool,
}

impl PgGoodsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GoodsRepository for PgGoodsRepository {
    async fn find_by_goods_id(&self, id: i64) -> RepositoryResult<Option<GoodsModel>> {
        let record = sqlx::query_as::<_, GoodsModel>(
            "SELECT * FROM goods WHERE id = $1 AND latest = TRUE LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_seller(&self, seller_id: i64, limit: i64) -> RepositoryResult<Vec<GoodsModel>> {
        let records = sqlx::query_as::<_, GoodsModel>(
            "SELECT * FROM goods WHERE seller_id = $1 AND latest = TRUE AND delisted = FALSE ORDER BY height DESC LIMIT $2"
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
            "SELECT * FROM goods WHERE latest = TRUE AND delisted = FALSE AND quantity > 0 ORDER BY height DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn update_quantity(&self, goods_id: i64, quantity: i32) -> RepositoryResult<()> {
        sqlx::query("UPDATE goods SET quantity = $1 WHERE id = $2 AND latest = TRUE")
            .bind(quantity)
            .bind(goods_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn update_price(&self, goods_id: i64, price: i64) -> RepositoryResult<()> {
        sqlx::query("UPDATE goods SET price = $1 WHERE id = $2 AND latest = TRUE")
            .bind(price)
            .bind(goods_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn set_delisted(&self, goods_id: i64, delisted: bool) -> RepositoryResult<()> {
        sqlx::query("UPDATE goods SET delisted = $1 WHERE id = $2 AND latest = TRUE")
            .bind(delisted)
            .bind(goods_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<GoodsModel> for PgGoodsRepository {
    async fn insert(&self, goods: &GoodsModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO goods (
                id, seller_id, name, description, parsed_tags, tags,
                timestamp, quantity, price, delisted, height, latest, has_image
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
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
        let record = sqlx::query_as::<_, GoodsModel>("SELECT * FROM goods WHERE db_id = $1")
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
            "SELECT * FROM goods WHERE latest = TRUE ORDER BY height DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM goods WHERE latest = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct PgCurrencyRepository {
    pool: PgPool,
}

impl PgCurrencyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CurrencyRepository for PgCurrencyRepository {
    async fn find_by_currency_id(&self, id: i64) -> RepositoryResult<Option<CurrencyModel>> {
        let record = sqlx::query_as::<_, CurrencyModel>(
            "SELECT * FROM currency WHERE id = $1 AND latest = TRUE LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_code(&self, code: &str) -> RepositoryResult<Option<CurrencyModel>> {
        let record = sqlx::query_as::<_, CurrencyModel>(
            "SELECT * FROM currency WHERE code = $1 AND latest = TRUE LIMIT 1"
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_owner(&self, account_id: i64) -> RepositoryResult<Vec<CurrencyModel>> {
        let records = sqlx::query_as::<_, CurrencyModel>(
            "SELECT * FROM currency WHERE account_id = $1 AND latest = TRUE ORDER BY height DESC"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<CurrencyModel>> {
        let records = sqlx::query_as::<_, CurrencyModel>(
            "SELECT * FROM currency WHERE height = $1 AND latest = TRUE"
        )
        .bind(height)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn increase_supply(&self, currency_id: i64, delta: i64) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE currency SET initial_supply = initial_supply + $2, latest = TRUE
            WHERE id = $1 AND latest = TRUE
            "#,
        )
        .bind(currency_id)
        .bind(delta)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn increase_reserve(&self, currency_id: i64, amount_per_unit: i64) -> RepositoryResult<()> {
        let (current_supply,): (i64,) = sqlx::query_as(
            "SELECT COALESCE(initial_supply, 0) FROM currency WHERE id = $1 AND latest = TRUE"
        )
        .bind(currency_id)
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        let total_increase = amount_per_unit * current_supply;

        sqlx::query(
            r#"
            UPDATE currency SET reserve_supply = reserve_supply + $2, latest = TRUE
            WHERE id = $1 AND latest = TRUE
            "#,
        )
        .bind(currency_id)
        .bind(total_increase)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete_currency(&self, currency_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM currency WHERE id = $1")
            .bind(currency_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<CurrencyModel> for PgCurrencyRepository {
    async fn insert(&self, currency: &CurrencyModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO currency (
                id, account_id, name, name_lower, code, description, type,
                initial_supply, reserve_supply, max_supply, creation_height,
                issuance_height, min_reserve_per_unit_nqt, min_difficulty,
                max_difficulty, ruleset, algorithm, decimals, height, latest
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
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
        let record = sqlx::query_as::<_, CurrencyModel>("SELECT * FROM currency WHERE db_id = $1")
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
            "SELECT * FROM currency WHERE latest = TRUE ORDER BY height DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM currency WHERE latest = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
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
        let record = sqlx::query_as::<_, TransactionModel>(
            "SELECT * FROM \"transaction\" WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_full_hash(&self, full_hash: &[u8]) -> RepositoryResult<Option<TransactionModel>> {
        let record = sqlx::query_as::<_, TransactionModel>(
            "SELECT * FROM \"transaction\" WHERE full_hash = $1"
        )
        .bind(full_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_sender(&self, sender_id: i64, limit: i64) -> RepositoryResult<Vec<TransactionModel>> {
        let records = sqlx::query_as::<_, TransactionModel>(
            "SELECT * FROM \"transaction\" WHERE sender_id = $1 ORDER BY timestamp DESC LIMIT $2"
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
            "SELECT * FROM \"transaction\" WHERE recipient_id = $1 ORDER BY timestamp DESC LIMIT $2"
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
            "SELECT * FROM \"transaction\" WHERE block_id = $1 ORDER BY transaction_index ASC"
        )
        .bind(block_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<TransactionModel>> {
        let records = sqlx::query_as::<_, TransactionModel>(
            "SELECT * FROM \"transaction\" WHERE height = $1 ORDER BY transaction_index ASC"
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
            sqlx::query(r#"DELETE FROM "transaction" WHERE db_id = $1"#)
                .bind(db_id)
                .execute(&self.pool)
                .await
                .map_err(RepositoryError::DbError)?;
        }
        Ok(())
    }

    async fn insert_tx(&self, tx_model: &TransactionModel, tx: &mut DbTransaction<'_>) -> RepositoryResult<()> {
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
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28)
            "#,
        )
        .bind(tx_model.id)
        .bind(tx_model.deadline)
        .bind(tx_model.recipient_id)
        .bind(tx_model.amount)
        .bind(tx_model.fee)
        .bind(tx_model.full_hash.as_slice())
        .bind(tx_model.height)
        .bind(tx_model.block_id)
        .bind(tx_model.signature.as_slice())
        .bind(tx_model.timestamp)
        .bind(tx_model.r#type)
        .bind(tx_model.subtype)
        .bind(tx_model.sender_id)
        .bind(tx_model.block_timestamp)
        .bind(tx_model.referenced_transaction_full_hash.as_deref())
        .bind(tx_model.transaction_index)
        .bind(tx_model.phased)
        .bind(tx_model.attachment_bytes.as_deref())
        .bind(tx_model.version)
        .bind(tx_model.has_message)
        .bind(tx_model.has_encrypted_message)
        .bind(tx_model.has_public_key_announcement)
        .bind(tx_model.has_prunable_message)
        .bind(tx_model.has_prunable_attachment)
        .bind(tx_model.ec_block_height)
        .bind(tx_model.ec_block_id)
        .bind(tx_model.has_encrypttoself_message)
        .bind(tx_model.has_prunable_encrypted_message)
        .execute(&mut **tx)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete_by_db_id_tx(&self, db_id: i64, tx: &mut DbTransaction<'_>) -> RepositoryResult<()> {
        sqlx::query(r#"DELETE FROM "transaction" WHERE db_id = $1"#)
            .bind(db_id)
            .execute(&mut **tx)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<TransactionModel> for PgTransactionRepository {
    async fn insert(&self, tx: &TransactionModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO "transaction" (
                id, deadline, recipient_id, amount, fee, full_hash,
                height, block_id, block_timestamp, transaction_index, signature, timestamp, type, subtype,
                sender_id, referenced_transaction_full_hash,
                attachment_bytes, version, phased,
                has_message, has_encrypted_message, has_public_key_announcement,
                has_prunable_message, has_prunable_attachment, ec_block_height,
                ec_block_id, has_encrypttoself_message, has_prunable_encrypted_message
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
                $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28
            )
            "#
        )
        .bind(tx.id)
        .bind(tx.deadline)
        .bind(tx.recipient_id)
        .bind(tx.amount)
        .bind(tx.fee)
        .bind(&tx.full_hash)
        .bind(tx.height)
        .bind(tx.block_id)
        .bind(tx.block_timestamp)
        .bind(tx.transaction_index)
        .bind(&tx.signature)
        .bind(tx.timestamp)
        .bind(tx.r#type)
        .bind(tx.subtype)
        .bind(tx.sender_id)
        .bind(&tx.referenced_transaction_full_hash)
        .bind(&tx.attachment_bytes)
        .bind(tx.version)
        .bind(tx.phased)
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
            "SELECT * FROM \"transaction\" WHERE db_id = $1"
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
        let record = sqlx::query_as::<_, AccountModel>("SELECT * FROM account WHERE id = $1 AND latest = TRUE LIMIT 1").bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<AccountModel>> {
        let records = sqlx::query_as::<_, AccountModel>("SELECT * FROM account WHERE height = $1 AND latest = TRUE").bind(height)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_latest_by_id(&self, id: i64) -> RepositoryResult<Option<AccountModel>> {
        self.find_by_account_id(id).await
    }

    async fn find_by_address(&self, _address: &str) -> RepositoryResult<Option<AccountModel>> {
        let record = sqlx::query_as::<_, AccountModel>("SELECT * FROM account WHERE latest = TRUE ORDER BY height DESC LIMIT 1")
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update_balance(&self, account_id: i64, balance: i64, unconfirmed_balance: i64, height: i32) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account
            SET balance = $1, unconfirmed_balance = $2, height = $3
            WHERE id = $4 AND latest = TRUE
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
            SET balance = balance + $1, height = $2
            WHERE id = $3 AND latest = TRUE
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
                SET balance = balance + $1, height = $2
                WHERE id = $3 AND latest = TRUE
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
            SET unconfirmed_balance = unconfirmed_balance + $1, height = $2
            WHERE id = $3 AND latest = TRUE
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
                SET unconfirmed_balance = unconfirmed_balance + $1, height = $2
                WHERE id = $3 AND latest = TRUE
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
            SET balance = balance + $1, unconfirmed_balance = unconfirmed_balance + $1, height = $2
            WHERE id = $3 AND latest = TRUE
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
                SET balance = balance + $1, unconfirmed_balance = unconfirmed_balance + $1, height = $2
                WHERE id = $3 AND latest = TRUE
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

    async fn get_account_count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM account WHERE latest = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }

    async fn add_to_forged_balance(&self, account_id: i64, amount: i64, height: i32) -> RepositoryResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE account
            SET forged_balance = forged_balance + $1, height = $2
            WHERE id = $3 AND latest = TRUE
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
                SET forged_balance = forged_balance + $1, height = $2
                WHERE id = $3 AND latest = TRUE
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
impl Repository<AccountModel> for PgAccountRepository {
    async fn insert(&self, account: &AccountModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO account (
                id, balance, unconfirmed_balance, forged_balance,
                active_lessee_id, has_control_phasing, height, latest
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
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
        let record = sqlx::query_as::<_, AccountModel>("SELECT * FROM account WHERE db_id = $1").bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, account: &AccountModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account SET
                balance = $2, unconfirmed_balance = $3, forged_balance = $4,
                active_lessee_id = $5, has_control_phasing = $6, height = $7, latest = $8
            WHERE db_id = $1
            "#,
        )
        .bind(account.db_id)
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

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for account (use logical delete)".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AccountModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, AccountModel>("SELECT * FROM account WHERE latest = TRUE ORDER BY id LIMIT $1 OFFSET $2").bind(limit).bind(offset)
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
        let records = sqlx::query_as::<_, AccountAssetModel>("SELECT * FROM account_asset WHERE account_id = $1 AND latest = TRUE").bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_asset(&self, asset_id: i64) -> RepositoryResult<Vec<AccountAssetModel>> {
        let records = sqlx::query_as::<_, AccountAssetModel>("SELECT * FROM account_asset WHERE asset_id = $1 AND latest = TRUE").bind(asset_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_account_and_asset(&self, account_id: i64, asset_id: i64) -> RepositoryResult<Option<AccountAssetModel>> {
        let record = sqlx::query_as::<_, AccountAssetModel>("SELECT * FROM account_asset WHERE account_id = $1 AND asset_id = $2 AND latest = TRUE LIMIT 1").bind(account_id).bind(asset_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update_quantity(&self, account_id: i64, asset_id: i64, quantity: i64, height: i32) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account_asset
            SET quantity = $1, height = $2, latest = TRUE
            WHERE account_id = $3 AND asset_id = $4
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
        let result = sqlx::query(
            r#"
            UPDATE account_asset
            SET quantity = quantity + $1, latest = TRUE
            WHERE account_id = $2 AND asset_id = $3
            "#,
        )
        .bind(delta)
        .bind(account_id)
        .bind(asset_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        if result.rows_affected() == 0 && delta != 0 {
            let current_height: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(height), 0) FROM block")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
            sqlx::query(
                "INSERT INTO account_asset (account_id, asset_id, quantity, unconfirmed_quantity, height, latest) VALUES ($1, $2, $3, 0, $4, TRUE)"
            )
            .bind(account_id)
            .bind(asset_id)
            .bind(delta)
            .bind(current_height)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        }
        Ok(())
    }

    async fn decrease_quantity(&self, account_id: i64, asset_id: i64, delta: i64) -> RepositoryResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE account_asset
            SET quantity = quantity - $1, latest = TRUE
            WHERE account_id = $2 AND asset_id = $3 AND quantity >= $1
            "#,
        )
        .bind(delta)
        .bind(account_id)
        .bind(asset_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::Validation("insufficient asset quantity".to_string()));
        }
        Ok(())
    }

    async fn add_to_unconfirmed_quantity(&self, account_id: i64, asset_id: i64, delta: i64) -> RepositoryResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE account_asset
            SET unconfirmed_quantity = unconfirmed_quantity + $1, latest = TRUE
            WHERE account_id = $2 AND asset_id = $3
            "#,
        )
        .bind(delta)
        .bind(account_id)
        .bind(asset_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        if result.rows_affected() == 0 && delta != 0 {
            let current_height: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(height), 0) FROM block")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
            sqlx::query(
                "INSERT INTO account_asset (account_id, asset_id, quantity, unconfirmed_quantity, height, latest) VALUES ($1, $2, 0, $3, $4, TRUE)"
            )
            .bind(account_id)
            .bind(asset_id)
            .bind(delta)
            .bind(current_height)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        }

        Ok(())
    }
}

#[async_trait]
impl Repository<AccountAssetModel> for PgAccountAssetRepository {
    async fn insert(&self, aa: &AccountAssetModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO account_asset (
                account_id, asset_id, quantity, unconfirmed_quantity, height, latest
            ) VALUES ($1, $2, $3, $4, $5, $6)
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
        let record = sqlx::query_as::<_, AccountAssetModel>("SELECT * FROM account_asset WHERE db_id = $1").bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, aa: &AccountAssetModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account_asset SET
                quantity = $2, unconfirmed_quantity = $3, height = $4, latest = $5
            WHERE db_id = $1
            "#,
        )
        .bind(aa.db_id)
        .bind(aa.quantity)
        .bind(aa.unconfirmed_quantity)
        .bind(aa.height)
        .bind(aa.latest)
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
        let records = sqlx::query_as::<_, AccountAssetModel>("SELECT * FROM account_asset WHERE latest = TRUE ORDER BY db_id LIMIT $1 OFFSET $2").bind(limit).bind(offset)
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
        let record = sqlx::query_as::<_, AssetModel>("SELECT * FROM asset WHERE id = $1 AND latest = TRUE LIMIT 1").bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_owner(&self, owner_id: i64) -> RepositoryResult<Vec<AssetModel>> {
        let records = sqlx::query_as::<_, AssetModel>("SELECT * FROM asset WHERE account_id = $1 AND latest = TRUE ORDER BY height DESC").bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<AssetModel>> {
        let records = sqlx::query_as::<_, AssetModel>("SELECT * FROM asset WHERE height = $1 AND latest = TRUE").bind(height)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_tradable(&self, limit: i64) -> RepositoryResult<Vec<AssetModel>> {
        let records = sqlx::query_as::<_, AssetModel>("SELECT * FROM asset WHERE latest = TRUE ORDER BY height DESC LIMIT $1").bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn increase_quantity(&self, asset_id: i64, delta: i64) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE asset
            SET quantity = quantity + $1, latest = TRUE
            WHERE id = $2 AND latest = TRUE
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
            SET quantity = quantity - $1, latest = TRUE
            WHERE id = $2 AND latest = TRUE AND quantity >= $1
            "#,
        )
        .bind(delta)
        .bind(asset_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<AssetModel> for PgAssetRepository {
    async fn insert(&self, asset: &AssetModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO asset (
                id, account_id, name, description, quantity, decimals,
                has_control_phasing, initial_quantity, height, latest
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
        let record = sqlx::query_as::<_, AssetModel>("SELECT * FROM asset WHERE db_id = $1").bind(db_id)
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
        let records = sqlx::query_as::<_, AssetModel>("SELECT * FROM asset WHERE latest = TRUE ORDER BY height DESC LIMIT $1 OFFSET $2").bind(limit).bind(offset)
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

pub struct PgAccountLedgerRepository {
    pool: PgPool,
}

impl PgAccountLedgerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountLedgerRepository for PgAccountLedgerRepository {
    async fn find_by_account(&self, account_id: i64, limit: i64) -> RepositoryResult<Vec<AccountLedgerModel>> {
        let records = sqlx::query_as::<_, AccountLedgerModel>(
            "SELECT * FROM account_ledger WHERE account_id = $1 ORDER BY height DESC LIMIT $2"
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
            "SELECT * FROM account_ledger WHERE block_id = $1 ORDER BY height DESC"
        )
        .bind(block_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

#[async_trait]
impl Repository<AccountLedgerModel> for PgAccountLedgerRepository {
    async fn insert(&self, ledger: &AccountLedgerModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO account_ledger (
                account_id, event_type, event_id, holding_type, holding_id,
                "CHANGE", balance, block_id, height, timestamp
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
            "SELECT * FROM account_ledger WHERE db_id = $1"
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
            "SELECT * FROM account_ledger ORDER BY height DESC LIMIT $1 OFFSET $2"
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

pub struct PgAliasRepository {
    pool: PgPool,
}

impl PgAliasRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AliasRepository for PgAliasRepository {
    async fn find_by_alias_id(&self, id: i64) -> RepositoryResult<Option<AliasModel>> {
        let record = sqlx::query_as::<_, AliasModel>(
            "SELECT * FROM alias WHERE id = $1 AND latest = TRUE LIMIT 1"
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
            "SELECT * FROM alias WHERE alias_name_lower = $1 AND latest = TRUE LIMIT 1"
        )
        .bind(&name_lower)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_owner(&self, account_id: i64) -> RepositoryResult<Vec<AliasModel>> {
        let records = sqlx::query_as::<_, AliasModel>(
            "SELECT * FROM alias WHERE account_id = $1 AND latest = TRUE ORDER BY alias_name_lower"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn update_owner(&self, alias_id: i64, new_owner_id: i64) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE alias SET account_id = $1 WHERE id = $2 AND latest = TRUE"
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
            "UPDATE alias SET alias_uri = $1 WHERE id = $2 AND latest = TRUE"
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
impl Repository<AliasModel> for PgAliasRepository {
    async fn insert(&self, alias: &AliasModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO alias (
                id, account_id, alias_name, alias_name_lower, alias_uri,
                timestamp, height, latest
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
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
            "SELECT * FROM alias WHERE db_id = $1"
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
                account_id = $2, alias_name = $3, alias_name_lower = $4,
                alias_uri = $5, timestamp = $6, height = $7, latest = $8
            WHERE db_id = $1
            "#,
        )
        .bind(alias.db_id)
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

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for alias (use logical delete)".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AliasModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, AliasModel>(
            "SELECT * FROM alias WHERE latest = TRUE ORDER BY alias_name_lower LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alias WHERE latest = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct PgAliasOfferRepository {
    pool: PgPool,
}

impl PgAliasOfferRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AliasOfferRepository for PgAliasOfferRepository {
    async fn find_by_alias(&self, alias_id: i64) -> RepositoryResult<Option<AliasOfferModel>> {
        let record = sqlx::query_as::<_, AliasOfferModel>(
            "SELECT * FROM alias_offer WHERE id = $1 AND latest = TRUE LIMIT 1"
        )
        .bind(alias_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_buyer(&self, buyer_id: i64) -> RepositoryResult<Vec<AliasOfferModel>> {
        let records = sqlx::query_as::<_, AliasOfferModel>(
            "SELECT * FROM alias_offer WHERE buyer_id = $1 AND latest = TRUE"
        )
        .bind(buyer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn update_price(&self, alias_id: i64, price: i64, buyer_id: Option<i64>) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE alias_offer SET price = $1, buyer_id = $2 WHERE id = $3 AND latest = TRUE"
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
        sqlx::query("DELETE FROM alias_offer WHERE id = $1")
            .bind(alias_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<AliasOfferModel> for PgAliasOfferRepository {
    async fn insert(&self, offer: &AliasOfferModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO alias_offer (id, price, buyer_id, height, latest)
            VALUES ($1, $2, $3, $4, $5)
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
            "SELECT * FROM alias_offer WHERE db_id = $1"
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
            UPDATE alias_offer SET price = $2, buyer_id = $3, height = $4, latest = $5
            WHERE db_id = $1
            "#,
        )
        .bind(offer.db_id)
        .bind(offer.price)
        .bind(offer.buyer_id)
        .bind(offer.height)
        .bind(offer.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM alias_offer WHERE db_id = $1")
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
            "SELECT * FROM alias_offer WHERE latest = TRUE ORDER BY height DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alias_offer WHERE latest = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct PgAssetTransferRepository {
    pool: PgPool,
}

impl PgAssetTransferRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AssetTransferRepository for PgAssetTransferRepository {
    async fn find_by_asset(&self, asset_id: i64, limit: i64) -> RepositoryResult<Vec<AssetTransferModel>> {
        let records = sqlx::query_as::<_, AssetTransferModel>(
            "SELECT * FROM asset_transfer WHERE asset_id = $1 ORDER BY height DESC LIMIT $2"
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
            "SELECT * FROM asset_transfer WHERE sender_id = $1 ORDER BY height DESC LIMIT $2"
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
            "SELECT * FROM asset_transfer WHERE recipient_id = $1 ORDER BY height DESC LIMIT $2"
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
impl Repository<AssetTransferModel> for PgAssetTransferRepository {
    async fn insert(&self, transfer: &AssetTransferModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO asset_transfer (asset_id, sender_id, recipient_id, quantity, timestamp, height)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
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
            "SELECT * FROM asset_transfer WHERE db_id = $1"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _transfer: &AssetTransferModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for asset_transfer".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for asset_transfer".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AssetTransferModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, AssetTransferModel>(
            "SELECT * FROM asset_transfer ORDER BY height DESC LIMIT $1 OFFSET $2"
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

pub struct PgAskOrderRepository {
    pool: PgPool,
}

impl PgAskOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AskOrderRepository for PgAskOrderRepository {
    async fn find_by_order_id(&self, id: i64) -> RepositoryResult<Option<AskOrderModel>> {
        let record = sqlx::query_as::<_, AskOrderModel>(
            "SELECT * FROM ask_order WHERE id = $1 AND latest = TRUE LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_asset(&self, asset_id: i64, limit: i64) -> RepositoryResult<Vec<AskOrderModel>> {
        let records = sqlx::query_as::<_, AskOrderModel>(
            "SELECT * FROM ask_order WHERE asset_id = $1 AND latest = TRUE ORDER BY price ASC LIMIT $2"
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
            "SELECT * FROM ask_order WHERE account_id = $1 AND latest = TRUE ORDER BY height DESC"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_best_by_asset(&self, asset_id: i64) -> RepositoryResult<Option<AskOrderModel>> {
        let record = sqlx::query_as::<_, AskOrderModel>(
            "SELECT * FROM ask_order WHERE asset_id = $1 AND latest = TRUE ORDER BY price ASC LIMIT 1"
        )
        .bind(asset_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update_quantity(&self, order_id: i64, quantity: i64) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE ask_order SET quantity = $1 WHERE id = $2 AND latest = TRUE"
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
impl Repository<AskOrderModel> for PgAskOrderRepository {
    async fn insert(&self, order: &AskOrderModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO ask_order (id, account_id, asset_id, price, quantity, height, latest)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(order.id)
        .bind(order.account_id)
        .bind(order.asset_id)
        .bind(order.price)
        .bind(order.quantity)
        .bind(order.height)
        .bind(order.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<AskOrderModel>> {
        let record = sqlx::query_as::<_, AskOrderModel>(
            "SELECT * FROM ask_order WHERE db_id = $1"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _order: &AskOrderModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for ask_order".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for ask_order".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AskOrderModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, AskOrderModel>(
            "SELECT * FROM ask_order WHERE latest = TRUE ORDER BY height DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM ask_order WHERE latest = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct PgBidOrderRepository {
    pool: PgPool,
}

impl PgBidOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BidOrderRepository for PgBidOrderRepository {
    async fn find_by_order_id(&self, id: i64) -> RepositoryResult<Option<BidOrderModel>> {
        let record = sqlx::query_as::<_, BidOrderModel>(
            "SELECT * FROM bid_order WHERE id = $1 AND latest = TRUE LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_asset(&self, asset_id: i64, limit: i64) -> RepositoryResult<Vec<BidOrderModel>> {
        let records = sqlx::query_as::<_, BidOrderModel>(
            "SELECT * FROM bid_order WHERE asset_id = $1 AND latest = TRUE ORDER BY price DESC LIMIT $2"
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
            "SELECT * FROM bid_order WHERE account_id = $1 AND latest = TRUE ORDER BY height DESC"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_best_by_asset(&self, asset_id: i64) -> RepositoryResult<Option<BidOrderModel>> {
        let record = sqlx::query_as::<_, BidOrderModel>(
            "SELECT * FROM bid_order WHERE asset_id = $1 AND latest = TRUE ORDER BY price DESC LIMIT 1"
        )
        .bind(asset_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update_quantity(&self, order_id: i64, quantity: i64) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE bid_order SET quantity = $1 WHERE id = $2 AND latest = TRUE"
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
impl Repository<BidOrderModel> for PgBidOrderRepository {
    async fn insert(&self, order: &BidOrderModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO bid_order (id, account_id, asset_id, price, quantity, height, latest)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(order.id)
        .bind(order.account_id)
        .bind(order.asset_id)
        .bind(order.price)
        .bind(order.quantity)
        .bind(order.height)
        .bind(order.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<BidOrderModel>> {
        let record = sqlx::query_as::<_, BidOrderModel>(
            "SELECT * FROM bid_order WHERE db_id = $1"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _order: &BidOrderModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for bid_order".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for bid_order".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<BidOrderModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, BidOrderModel>(
            "SELECT * FROM bid_order WHERE latest = TRUE ORDER BY height DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM bid_order WHERE latest = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct PgTaggedDataRepository {
    pool: PgPool,
}

impl PgTaggedDataRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TaggedDataRepository for PgTaggedDataRepository {
    async fn find_by_data_id(&self, id: i64) -> RepositoryResult<Option<TaggedDataModel>> {
        let record = sqlx::query_as::<_, TaggedDataModel>(
            "SELECT * FROM tagged_data WHERE id = $1 AND latest = TRUE LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_account(&self, account_id: i64, limit: i64) -> RepositoryResult<Vec<TaggedDataModel>> {
        let records = sqlx::query_as::<_, TaggedDataModel>(
            "SELECT * FROM tagged_data WHERE account_id = $1 AND latest = TRUE ORDER BY height DESC LIMIT $2"
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_type(&self, type_: &str, limit: i64) -> RepositoryResult<Vec<TaggedDataModel>> {
        let records = sqlx::query_as::<_, TaggedDataModel>(
            "SELECT * FROM tagged_data WHERE type = $1 AND latest = TRUE ORDER BY height DESC LIMIT $2"
        )
        .bind(type_)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn search_by_tag(&self, tag: &str, limit: i64) -> RepositoryResult<Vec<TaggedDataModel>> {
        let pattern = format!("%{}%", tag);
        let records = sqlx::query_as::<_, TaggedDataModel>(
            "SELECT * FROM tagged_data WHERE (tags LIKE $1 OR parsed_tags LIKE $1) AND latest = TRUE ORDER BY height DESC LIMIT $2"
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
impl Repository<TaggedDataModel> for PgTaggedDataRepository {
    async fn insert(&self, data: &TaggedDataModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO tagged_data (
                id, account_id, name, description, tags, parsed_tags, type,
                data, is_text, filename, channel, block_timestamp,
                transaction_timestamp, height, latest
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
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
        let record = sqlx::query_as::<_, TaggedDataModel>("SELECT * FROM tagged_data WHERE db_id = $1")
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
            "SELECT * FROM tagged_data WHERE latest = TRUE ORDER BY height DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tagged_data WHERE latest = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct PgPurchaseRepository {
    pool: PgPool,
}

impl PgPurchaseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PurchaseRepository for PgPurchaseRepository {
    async fn find_by_purchase_id(&self, id: i64) -> RepositoryResult<Option<PurchaseModel>> {
        let record = sqlx::query_as::<_, PurchaseModel>(
            "SELECT * FROM purchase WHERE id = $1 AND latest = TRUE LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_buyer(&self, buyer_id: i64, limit: i64) -> RepositoryResult<Vec<PurchaseModel>> {
        let records = sqlx::query_as::<_, PurchaseModel>(
            "SELECT * FROM purchase WHERE buyer_id = $1 AND latest = TRUE ORDER BY height DESC LIMIT $2"
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
            "SELECT * FROM purchase WHERE seller_id = $1 AND latest = TRUE ORDER BY height DESC LIMIT $2"
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
            "SELECT * FROM purchase WHERE goods_id = $1 AND latest = TRUE ORDER BY height DESC LIMIT $2"
        )
        .bind(goods_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn update_pending(&self, purchase_id: i64, pending: bool) -> RepositoryResult<()> {
        sqlx::query("UPDATE purchase SET pending = $1 WHERE id = $2 AND latest = TRUE")
            .bind(pending)
            .bind(purchase_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn set_delivered(&self, purchase_id: i64, goods: &[u8], nonce: &[u8]) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE purchase SET goods = $1, goods_nonce = $2, pending = FALSE WHERE id = $3 AND latest = TRUE"
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
            "UPDATE purchase SET refund = $1, refund_note = $2, refund_nonce = $3 WHERE id = $4 AND latest = TRUE"
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
impl Repository<PurchaseModel> for PgPurchaseRepository {
    async fn insert(&self, purchase: &PurchaseModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO purchase (
                id, buyer_id, goods_id, seller_id, quantity, price, deadline,
                note, nonce, timestamp, pending, goods, goods_nonce, goods_is_text,
                refund_note, refund_nonce, has_feedback_notes, has_public_feedbacks,
                discount, refund, height, latest
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
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
        let record = sqlx::query_as::<_, PurchaseModel>("SELECT * FROM purchase WHERE db_id = $1")
            .bind(db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &PurchaseModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for purchase".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for purchase".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PurchaseModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, PurchaseModel>(
            "SELECT * FROM purchase WHERE latest = TRUE ORDER BY height DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM purchase WHERE latest = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct PgPollRepository {
    pool: PgPool,
}

impl PgPollRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PollRepository for PgPollRepository {
    async fn find_by_poll_id(&self, id: i64) -> RepositoryResult<Option<PollModel>> {
        let record = sqlx::query_as::<_, PollModel>(
            "SELECT * FROM poll WHERE id = $1 LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<PollModel>> {
        let records = sqlx::query_as::<_, PollModel>(
            "SELECT * FROM poll WHERE account_id = $1 ORDER BY height DESC"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_active(&self, height: i32, limit: i64) -> RepositoryResult<Vec<PollModel>> {
        let records = sqlx::query_as::<_, PollModel>(
            "SELECT * FROM poll WHERE finish_height > $1 ORDER BY finish_height ASC LIMIT $2"
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
impl Repository<PollModel> for PgPollRepository {
    async fn insert(&self, poll: &PollModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO poll (
                id, account_id, name, description, options, min_num_options, max_num_options,
                min_range_value, max_range_value, timestamp, finish_height, voting_model,
                min_balance, min_balance_model, holding_id, height
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
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
        let record = sqlx::query_as::<_, PollModel>("SELECT * FROM poll WHERE db_id = $1")
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
            "SELECT * FROM poll ORDER BY height DESC LIMIT $1 OFFSET $2"
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

pub struct PgVoteRepository {
    pool: PgPool,
}

impl PgVoteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VoteRepository for PgVoteRepository {
    async fn find_by_poll(&self, poll_id: i64, limit: i64) -> RepositoryResult<Vec<VoteModel>> {
        let records = sqlx::query_as::<_, VoteModel>(
            "SELECT * FROM vote WHERE poll_id = $1 ORDER BY height DESC LIMIT $2"
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
            "SELECT * FROM vote WHERE voter_id = $1 ORDER BY height DESC LIMIT $2"
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
            "SELECT * FROM vote WHERE poll_id = $1 AND voter_id = $2 LIMIT 1"
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
impl Repository<VoteModel> for PgVoteRepository {
    async fn insert(&self, vote: &VoteModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO vote (id, poll_id, voter_id, vote_bytes, height)
            VALUES ($1, $2, $3, $4, $5)
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
        let record = sqlx::query_as::<_, VoteModel>("SELECT * FROM vote WHERE db_id = $1")
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
            "SELECT * FROM vote ORDER BY height DESC LIMIT $1 OFFSET $2"
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

pub struct PgShufflingRepository {
    pool: PgPool,
}

impl PgShufflingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ShufflingRepository for PgShufflingRepository {
    async fn find_by_shuffling_id(&self, id: i64) -> RepositoryResult<Option<ShufflingModel>> {
        let record = sqlx::query_as::<_, ShufflingModel>(
            "SELECT * FROM shuffling WHERE id = $1 AND latest = TRUE LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_issuer(&self, issuer_id: i64) -> RepositoryResult<Vec<ShufflingModel>> {
        let records = sqlx::query_as::<_, ShufflingModel>(
            "SELECT * FROM shuffling WHERE issuer_id = $1 AND latest = TRUE ORDER BY height DESC"
        )
        .bind(issuer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_active(&self, limit: i64) -> RepositoryResult<Vec<ShufflingModel>> {
        let records = sqlx::query_as::<_, ShufflingModel>(
            "SELECT * FROM shuffling WHERE latest = TRUE AND stage < 5 ORDER BY height DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn update_stage(&self, shuffling_id: i64, stage: i32) -> RepositoryResult<()> {
        sqlx::query("UPDATE shuffling SET stage = $1 WHERE id = $2 AND latest = TRUE")
            .bind(stage)
            .bind(shuffling_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<ShufflingModel> for PgShufflingRepository {
    async fn insert(&self, shuffling: &ShufflingModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO shuffling (
                id, holding_id, holding_type, issuer_id, amount, participant_count,
                blocks_remaining, stage, assignee_account_id, registrant_count,
                recipient_public_keys, height, latest
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
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
        let record = sqlx::query_as::<_, ShufflingModel>("SELECT * FROM shuffling WHERE db_id = $1")
            .bind(db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &ShufflingModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for shuffling".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for shuffling".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<ShufflingModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, ShufflingModel>(
            "SELECT * FROM shuffling WHERE latest = TRUE ORDER BY height DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM shuffling WHERE latest = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct PgAccountCurrencyRepository {
    pool: PgPool,
}

impl PgAccountCurrencyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountCurrencyRepository for PgAccountCurrencyRepository {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<AccountCurrencyModel>> {
        let records = sqlx::query_as::<_, AccountCurrencyModel>(
            "SELECT * FROM account_currency WHERE account_id = $1 AND latest = TRUE"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_currency(&self, currency_id: i64) -> RepositoryResult<Vec<AccountCurrencyModel>> {
        let records = sqlx::query_as::<_, AccountCurrencyModel>(
            "SELECT * FROM account_currency WHERE currency_id = $1 AND latest = TRUE"
        )
        .bind(currency_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_account_and_currency(&self, account_id: i64, currency_id: i64) -> RepositoryResult<Option<AccountCurrencyModel>> {
        let record = sqlx::query_as::<_, AccountCurrencyModel>(
            "SELECT * FROM account_currency WHERE account_id = $1 AND currency_id = $2 AND latest = TRUE LIMIT 1"
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
            "UPDATE account_currency SET units = units + $1, latest = TRUE WHERE account_id = $2 AND currency_id = $3 AND latest = TRUE"
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
                "INSERT INTO account_currency (account_id, currency_id, units, unconfirmed_units, height, latest) VALUES ($1, $2, $3, 0, $4, TRUE)"
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
        let result = sqlx::query(
            r#"
            UPDATE account_currency
            SET unconfirmed_units = unconfirmed_units + $1, latest = TRUE
            WHERE account_id = $2 AND currency_id = $3
            "#,
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
                "INSERT INTO account_currency (account_id, currency_id, units, unconfirmed_units, height, latest) VALUES ($1, $2, 0, $3, $4, TRUE)"
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
}

#[async_trait]
impl Repository<AccountCurrencyModel> for PgAccountCurrencyRepository {
    async fn insert(&self, ac: &AccountCurrencyModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO account_currency (account_id, currency_id, units, unconfirmed_units, height, latest)
            VALUES ($1, $2, $3, $4, $5, $6)
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
        let record = sqlx::query_as::<_, AccountCurrencyModel>("SELECT * FROM account_currency WHERE db_id = $1")
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
            "SELECT * FROM account_currency WHERE latest = TRUE LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM account_currency WHERE latest = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct PgCurrencyTransferRepository {
    pool: PgPool,
}

impl PgCurrencyTransferRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CurrencyTransferRepository for PgCurrencyTransferRepository {
    async fn find_by_currency(&self, currency_id: i64, limit: i64) -> RepositoryResult<Vec<CurrencyTransferModel>> {
        let records = sqlx::query_as::<_, CurrencyTransferModel>(
            "SELECT * FROM currency_transfer WHERE currency_id = $1 ORDER BY height DESC LIMIT $2"
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
            "SELECT * FROM currency_transfer WHERE sender_id = $1 ORDER BY height DESC LIMIT $2"
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
            "SELECT * FROM currency_transfer WHERE recipient_id = $1 ORDER BY height DESC LIMIT $2"
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
impl Repository<CurrencyTransferModel> for PgCurrencyTransferRepository {
    async fn insert(&self, transfer: &CurrencyTransferModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO currency_transfer (id, currency_id, sender_id, recipient_id, units, timestamp, height)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
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
        let record = sqlx::query_as::<_, CurrencyTransferModel>("SELECT * FROM currency_transfer WHERE db_id = $1")
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
            "SELECT * FROM currency_transfer ORDER BY height DESC LIMIT $1 OFFSET $2"
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

pub struct PgContractReferenceRepository {
    pool: PgPool,
}

impl PgContractReferenceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ContractReferenceRepository for PgContractReferenceRepository {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<ContractReferenceModel>> {
        let records = sqlx::query_as::<_, ContractReferenceModel>(
            "SELECT * FROM contract_reference WHERE account_id = $1 ORDER BY height DESC"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_contract_name(&self, name: &str) -> RepositoryResult<Option<ContractReferenceModel>> {
        let record = sqlx::query_as::<_, ContractReferenceModel>(
            "SELECT * FROM contract_reference WHERE contract_name = $1 LIMIT 1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn delete_by_account_and_name(&self, account_id: i64, name: &str) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM contract_reference WHERE account_id = $1 AND contract_name = $2")
            .bind(account_id)
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }
}

#[async_trait]
impl Repository<ContractReferenceModel> for PgContractReferenceRepository {
    async fn insert(&self, cr: &ContractReferenceModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO contract_reference (id, account_id, contract_name, contract_params, contract_transaction_chain_id, contract_transaction_full_hash, height, latest)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(cr.id)
        .bind(cr.account_id)
        .bind(&cr.contract_name)
        .bind(&cr.contract_params)
        .bind(cr.contract_transaction_chain_id)
        .bind(&cr.contract_transaction_full_hash)
        .bind(cr.height)
        .bind(cr.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<ContractReferenceModel>> {
        let record = sqlx::query_as::<_, ContractReferenceModel>("SELECT * FROM contract_reference WHERE db_id = $1")
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
        sqlx::query("DELETE FROM contract_reference WHERE db_id = $1")
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
            "SELECT * FROM contract_reference ORDER BY height DESC LIMIT $1 OFFSET $2"
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

pub struct PgAssetPropertyRepository {
    pool: PgPool,
}

impl PgAssetPropertyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AssetPropertyRepository for PgAssetPropertyRepository {
    async fn find_by_asset(&self, asset_id: i64) -> RepositoryResult<Vec<AssetPropertyModel>> {
        let records = sqlx::query_as::<_, AssetPropertyModel>(
            "SELECT * FROM asset_property WHERE asset_id = $1 AND latest = TRUE ORDER BY height DESC"
        )
        .bind(asset_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_asset_and_account(&self, asset_id: i64, setter_id: i64) -> RepositoryResult<Vec<AssetPropertyModel>> {
        let records = sqlx::query_as::<_, AssetPropertyModel>(
            "SELECT * FROM asset_property WHERE asset_id = $1 AND setter_id = $2 AND latest = TRUE ORDER BY height DESC"
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
            "SELECT * FROM asset_property WHERE asset_id = $1 AND setter_id = $2 AND property = $3 AND latest = TRUE LIMIT 1"
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
        sqlx::query("DELETE FROM asset_property WHERE asset_id = $1 AND setter_id = $2 AND property = $3")
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
impl Repository<AssetPropertyModel> for PgAssetPropertyRepository {
    async fn insert(&self, prop: &AssetPropertyModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO asset_property (id, asset_id, setter_id, property, value, height, latest)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
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
        let record = sqlx::query_as::<_, AssetPropertyModel>("SELECT * FROM asset_property WHERE db_id = $1")
            .bind(db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _item: &AssetPropertyModel) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("update not implemented for asset_property".to_string()))
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Err(RepositoryError::Validation("delete not implemented for asset_property".to_string()))
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AssetPropertyModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, AssetPropertyModel>(
            "SELECT * FROM asset_property WHERE latest = TRUE ORDER BY height DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM asset_property WHERE latest = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct PgAssetHistoryRepository {
    pool: PgPool,
}

impl PgAssetHistoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AssetHistoryRepository for PgAssetHistoryRepository {
    async fn find_by_asset(&self, asset_id: i64, limit: i64) -> RepositoryResult<Vec<AssetHistoryModel>> {
        let records = sqlx::query_as::<_, AssetHistoryModel>(
            "SELECT * FROM asset_history WHERE asset_id = $1 ORDER BY height DESC LIMIT $2"
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
            "SELECT * FROM asset_history WHERE asset_id = $1 AND account_id = $2 ORDER BY height DESC LIMIT $3"
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
impl Repository<AssetHistoryModel> for PgAssetHistoryRepository {
    async fn insert(&self, history: &AssetHistoryModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO asset_history (id, full_hash, asset_id, account_id, quantity, timestamp, chain_id, height)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
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
        let record = sqlx::query_as::<_, AssetHistoryModel>("SELECT * FROM asset_history WHERE db_id = $1")
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
            "SELECT * FROM asset_history ORDER BY height DESC LIMIT $1 OFFSET $2"
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

pub struct PgTaggedDataTagRepository {
    pool: PgPool,
}

impl PgTaggedDataTagRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TaggedDataTagRepository for PgTaggedDataTagRepository {
    async fn find_by_tag(&self, tag: &str, limit: i64) -> RepositoryResult<Vec<TaggedDataTagModel>> {
        let records = sqlx::query_as::<_, TaggedDataTagModel>(
            "SELECT * FROM tagged_data_tag WHERE tag = $1 ORDER BY height DESC LIMIT $2"
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
            "SELECT * FROM tagged_data_tag WHERE id = $1 ORDER BY height DESC"
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

#[async_trait]
impl Repository<TaggedDataTagModel> for PgTaggedDataTagRepository {
    async fn insert(&self, tag: &TaggedDataTagModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO tagged_data_tag (id, tag, height, latest)
            VALUES ($1, $2, $3, $4)
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
        let record = sqlx::query_as::<_, TaggedDataTagModel>("SELECT * FROM tagged_data_tag WHERE db_id = $1")
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
            "SELECT * FROM tagged_data_tag WHERE latest = TRUE ORDER BY height DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tagged_data_tag WHERE latest = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct PgTaggedTimestampRepository {
    pool: PgPool,
}

impl PgTaggedTimestampRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TaggedTimestampRepository for PgTaggedTimestampRepository {
    async fn find_by_account(&self, account_id: i64, limit: i64) -> RepositoryResult<Vec<TaggedTimestampModel>> {
        let records = sqlx::query_as::<_, TaggedTimestampModel>(
            "SELECT * FROM tagged_timestamp WHERE account_id = $1 ORDER BY height DESC LIMIT $2"
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
            "SELECT * FROM tagged_timestamp WHERE tag = $1 ORDER BY height DESC LIMIT $2"
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
            "SELECT * FROM tagged_timestamp WHERE account_id = $1 AND tag = $2 LIMIT 1"
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
impl Repository<TaggedTimestampModel> for PgTaggedTimestampRepository {
    async fn insert(&self, ts: &TaggedTimestampModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO tagged_timestamp (id, account_id, tag, timestamp, height, latest)
            VALUES ($1, $2, $3, $4, $5, $6)
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
        let record = sqlx::query_as::<_, TaggedTimestampModel>("SELECT * FROM tagged_timestamp WHERE db_id = $1")
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
            "SELECT * FROM tagged_timestamp WHERE latest = TRUE ORDER BY height DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tagged_timestamp WHERE latest = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

pub struct PgAccountGuaranteedBalanceRepository {
    pool: PgPool,
}

impl PgAccountGuaranteedBalanceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<AccountGuaranteedBalanceModel> for PgAccountGuaranteedBalanceRepository {
    async fn insert(&self, item: &AccountGuaranteedBalanceModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO account_guaranteed_balance (account_id, additions, height)
            VALUES ($1, $2, $3)
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
            "SELECT * FROM account_guaranteed_balance WHERE db_id = $1"
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
            SET account_id = $1, additions = $2, height = $3
            WHERE db_id = $4
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
        sqlx::query("DELETE FROM account_guaranteed_balance WHERE db_id = $1")
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
            "SELECT * FROM account_guaranteed_balance ORDER BY height DESC LIMIT $1 OFFSET $2"
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
impl AccountGuaranteedBalanceRepository for PgAccountGuaranteedBalanceRepository {
    async fn find_by_account_and_height(&self, account_id: i64, height: i32) -> RepositoryResult<Option<AccountGuaranteedBalanceModel>> {
        let record = sqlx::query_as::<_, AccountGuaranteedBalanceModel>(
            "SELECT * FROM account_guaranteed_balance WHERE account_id = $1 AND height = $2"
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
             WHERE account_id = $1 AND height >= $2 AND height <= $3"
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

pub struct PgExchangeRequestRepository {
    pool: PgPool,
}

impl PgExchangeRequestRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<ExchangeRequestModel> for PgExchangeRequestRepository {
    async fn insert(&self, request: &ExchangeRequestModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO exchange_request (id, account_id, currency_id, units, rate, is_buy, timestamp, height)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
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
            "SELECT * FROM exchange_request WHERE db_id = $1"
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
                account_id = $1, currency_id = $2, units = $3, rate = $4,
                is_buy = $5, timestamp = $6, height = $7
            WHERE db_id = $8
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
        sqlx::query("DELETE FROM exchange_request WHERE db_id = $1")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<ExchangeRequestModel>> {
        let records = sqlx::query_as::<_, ExchangeRequestModel>(
            "SELECT * FROM exchange_request ORDER BY height DESC LIMIT $1 OFFSET $2"
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

pub struct PgCurrencyMintRepository {
    pool: PgPool,
}

impl PgCurrencyMintRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<CurrencyMintModel> for PgCurrencyMintRepository {
    async fn insert(&self, mint: &CurrencyMintModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO currency_mint (currency_id, account_id, counter, height, latest)
            VALUES ($1, $2, $3, $4, $5)
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
            "SELECT * FROM currency_mint WHERE db_id = $1"
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
                currency_id = $1, account_id = $2, counter = $3,
                height = $4, latest = $5
            WHERE db_id = $6
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
        sqlx::query("DELETE FROM currency_mint WHERE db_id = $1")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<CurrencyMintModel>> {
        let records = sqlx::query_as::<_, CurrencyMintModel>(
            "SELECT * FROM currency_mint ORDER BY height DESC LIMIT $1 OFFSET $2"
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

pub struct PgAssetDeleteRepository {
    pool: PgPool,
}

impl PgAssetDeleteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<AssetDeleteModel> for PgAssetDeleteRepository {
    async fn insert(&self, model: &AssetDeleteModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO asset_delete (asset_id, account_id, quantity, height)
            VALUES ($1, $2, $3, $4)
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
            "SELECT * FROM asset_delete WHERE db_id = $1"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _model: &AssetDeleteModel) -> RepositoryResult<()> {
        Ok(())
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AssetDeleteModel>> {
        let records = sqlx::query_as::<_, AssetDeleteModel>(
            "SELECT * FROM asset_delete ORDER BY height DESC LIMIT $1 OFFSET $2"
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
impl AssetDeleteRepository for PgAssetDeleteRepository {
    async fn find_by_asset(&self, asset_id: i64) -> RepositoryResult<Vec<AssetDeleteModel>> {
        let records = sqlx::query_as::<_, AssetDeleteModel>(
            "SELECT * FROM asset_delete WHERE asset_id = $1 ORDER BY height DESC"
        )
        .bind(asset_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

pub struct PgPhasingPollResultRepository {
    pool: PgPool,
}

impl PgPhasingPollResultRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<PhasingPollResultModel> for PgPhasingPollResultRepository {
    async fn insert(&self, model: &PhasingPollResultModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO phasing_poll_result (id, result, approved, height)
            VALUES ($1, $2, $3, $4)
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
            "SELECT * FROM phasing_poll_result WHERE db_id = $1"
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
                id = $1, result = $2, approved = $3, height = $4
            WHERE db_id = $5
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
        sqlx::query("DELETE FROM phasing_poll_result WHERE db_id = $1")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PhasingPollResultModel>> {
        let records = sqlx::query_as::<_, PhasingPollResultModel>(
            "SELECT * FROM phasing_poll_result ORDER BY height DESC LIMIT $1 OFFSET $2"
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
impl PhasingPollResultRepository for PgPhasingPollResultRepository {
    async fn find_by_poll(&self, poll_id: i64) -> RepositoryResult<Option<PhasingPollResultModel>> {
        let record = sqlx::query_as::<_, PhasingPollResultModel>(
            "SELECT * FROM phasing_poll_result WHERE id = $1"
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

// ==================== CoinExchange Repositories (PostgreSQL) ====================

pub struct PgCoinOrderFxtRepository {
    pool: PgPool,
}

impl PgCoinOrderFxtRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<CoinOrderFxtModel> for PgCoinOrderFxtRepository {
    async fn insert(&self, model: &CoinOrderFxtModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO coin_order_fxt (id, account_id, chain_id, exchange_id, full_hash, amount, quantity, bid_price, ask_price, creation_height, height, transaction_height, transaction_index, latest)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(model.id)
        .bind(model.account_id)
        .bind(model.chain_id)
        .bind(model.exchange_id)
        .bind(&model.full_hash)
        .bind(model.amount)
        .bind(model.quantity)
        .bind(model.bid_price)
        .bind(model.ask_price)
        .bind(model.creation_height)
        .bind(model.height)
        .bind(model.transaction_height)
        .bind(model.transaction_index)
        .bind(model.latest)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<CoinOrderFxtModel>> {
        let record = sqlx::query_as::<_, CoinOrderFxtModel>(
            "SELECT * FROM coin_order_fxt WHERE db_id = $1"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, model: &CoinOrderFxtModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE coin_order_fxt SET
                id = $1, account_id = $2, chain_id = $3, exchange_id = $4, full_hash = $5,
                amount = $6, quantity = $7, bid_price = $8, ask_price = $9,
                creation_height = $10, height = $11, transaction_height = $12, transaction_index = $13, latest = $14
            WHERE db_id = $15
            "#,
        )
        .bind(model.id)
        .bind(model.account_id)
        .bind(model.chain_id)
        .bind(model.exchange_id)
        .bind(&model.full_hash)
        .bind(model.amount)
        .bind(model.quantity)
        .bind(model.bid_price)
        .bind(model.ask_price)
        .bind(model.creation_height)
        .bind(model.height)
        .bind(model.transaction_height)
        .bind(model.transaction_index)
        .bind(model.latest)
        .bind(model.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM coin_order_fxt WHERE db_id = $1")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<CoinOrderFxtModel>> {
        let lim = limit.unwrap_or(100);
        let off = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, CoinOrderFxtModel>(
            "SELECT * FROM coin_order_fxt ORDER BY height DESC LIMIT $1 OFFSET $2"
        )
        .bind(lim)
        .bind(off)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM coin_order_fxt")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl CoinOrderFxtRepository for PgCoinOrderFxtRepository {
    async fn find_by_exchange(&self, exchange_id: i32) -> RepositoryResult<Vec<CoinOrderFxtModel>> {
        let records = sqlx::query_as::<_, CoinOrderFxtModel>(
            "SELECT * FROM coin_order_fxt WHERE exchange_id = $1 AND latest = true"
        )
        .bind(exchange_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<CoinOrderFxtModel>> {
        let records = sqlx::query_as::<_, CoinOrderFxtModel>(
            "SELECT * FROM coin_order_fxt WHERE account_id = $1 AND latest = true"
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

pub struct PgCoinTradeFxtRepository {
    pool: PgPool,
}

impl PgCoinTradeFxtRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<CoinTradeFxtModel> for PgCoinTradeFxtRepository {
    async fn insert(&self, model: &CoinTradeFxtModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO coin_trade_fxt (chain_id, exchange_id, account_id, block_id, height, timestamp, exchange_quantity, exchange_price, order_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(model.chain_id)
        .bind(model.exchange_id)
        .bind(model.account_id)
        .bind(model.block_id)
        .bind(model.height)
        .bind(model.timestamp)
        .bind(model.exchange_quantity)
        .bind(model.exchange_price)
        .bind(model.order_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<CoinTradeFxtModel>> {
        let record = sqlx::query_as::<_, CoinTradeFxtModel>(
            "SELECT * FROM coin_trade_fxt WHERE db_id = $1"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, model: &CoinTradeFxtModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE coin_trade_fxt SET
                chain_id = $1, exchange_id = $2, account_id = $3, block_id = $4,
                height = $5, timestamp = $6, exchange_quantity = $7, exchange_price = $8, order_id = $9
            WHERE db_id = $10
            "#,
        )
        .bind(model.chain_id)
        .bind(model.exchange_id)
        .bind(model.account_id)
        .bind(model.block_id)
        .bind(model.height)
        .bind(model.timestamp)
        .bind(model.exchange_quantity)
        .bind(model.exchange_price)
        .bind(model.order_id)
        .bind(model.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM coin_trade_fxt WHERE db_id = $1")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<CoinTradeFxtModel>> {
        let lim = limit.unwrap_or(100);
        let off = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, CoinTradeFxtModel>(
            "SELECT * FROM coin_trade_fxt ORDER BY height DESC LIMIT $1 OFFSET $2"
        )
        .bind(lim)
        .bind(off)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM coin_trade_fxt")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl CoinTradeFxtRepository for PgCoinTradeFxtRepository {
    async fn find_by_exchange(&self, exchange_id: i32, limit: i64) -> RepositoryResult<Vec<CoinTradeFxtModel>> {
        let records = sqlx::query_as::<_, CoinTradeFxtModel>(
            "SELECT * FROM coin_trade_fxt WHERE exchange_id = $1 ORDER BY height DESC LIMIT $2"
        )
        .bind(exchange_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_order(&self, order_id: i64) -> RepositoryResult<Vec<CoinTradeFxtModel>> {
        let records = sqlx::query_as::<_, CoinTradeFxtModel>(
            "SELECT * FROM coin_trade_fxt WHERE order_id = $1"
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

pub struct PgPhasingPollVoterRepository {
    pool: PgPool,
}

impl PgPhasingPollVoterRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<PhasingPollVoterModel> for PgPhasingPollVoterRepository {
    async fn insert(&self, model: &PhasingPollVoterModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO phasing_poll_voter (transaction_id, voter_id, height)
            VALUES ($1, $2, $3)
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
            "SELECT * FROM phasing_poll_voter WHERE db_id = $1"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _model: &PhasingPollVoterModel) -> RepositoryResult<()> {
        Ok(())
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PhasingPollVoterModel>> {
        let records = sqlx::query_as::<_, PhasingPollVoterModel>(
            "SELECT * FROM phasing_poll_voter ORDER BY height DESC LIMIT $1 OFFSET $2"
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
impl PhasingPollVoterRepository for PgPhasingPollVoterRepository {
    async fn find_by_poll(&self, poll_id: i64) -> RepositoryResult<Vec<PhasingPollVoterModel>> {
        let records = sqlx::query_as::<_, PhasingPollVoterModel>(
            "SELECT * FROM phasing_poll_voter WHERE transaction_id = $1"
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

pub struct PgPhasingPollRepository {
    pool: PgPool,
}

impl PgPhasingPollRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<PhasingPollModel> for PgPhasingPollRepository {
    async fn insert(&self, model: &PhasingPollModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO phasing_poll (id, account_id, whitelist_size, finish_height, voting_model, quorum,
                min_balance, holding_id, min_balance_model, hashed_secret, algorithm, height)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
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
            "SELECT * FROM phasing_poll WHERE db_id = $1"
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
                account_id = $1, whitelist_size = $2, finish_height = $3, voting_model = $4, quorum = $5,
                min_balance = $6, holding_id = $7, min_balance_model = $8,
                hashed_secret = $9, algorithm = $10, height = $11
            WHERE db_id = $12
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
        sqlx::query("DELETE FROM phasing_poll WHERE db_id = $1")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PhasingPollModel>> {
        let records = sqlx::query_as::<_, PhasingPollModel>(
            "SELECT * FROM phasing_poll ORDER BY height DESC LIMIT $1 OFFSET $2"
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
impl PhasingPollRepository for PgPhasingPollRepository {
    async fn find_by_poll_id(&self, id: i64) -> RepositoryResult<Option<PhasingPollModel>> {
        let record = sqlx::query_as::<_, PhasingPollModel>(
            "SELECT * FROM phasing_poll WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }
}

pub struct PgPhasingPollLinkedTransactionRepository {
    pool: PgPool,
}

impl PgPhasingPollLinkedTransactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<PhasingPollLinkedTransactionModel> for PgPhasingPollLinkedTransactionRepository {
    async fn insert(&self, model: &PhasingPollLinkedTransactionModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO phasing_poll_linked_transaction (transaction_id, linked_full_hash, linked_transaction_id, height)
            VALUES ($1, $2, $3, $4)
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
            "SELECT * FROM phasing_poll_linked_transaction WHERE db_id = $1"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _model: &PhasingPollLinkedTransactionModel) -> RepositoryResult<()> {
        Ok(())
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PhasingPollLinkedTransactionModel>> {
        let records = sqlx::query_as::<_, PhasingPollLinkedTransactionModel>(
            "SELECT * FROM phasing_poll_linked_transaction ORDER BY height DESC LIMIT $1 OFFSET $2"
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
impl PhasingPollLinkedTransactionRepository for PgPhasingPollLinkedTransactionRepository {
    async fn find_by_poll(&self, poll_id: i64) -> RepositoryResult<Vec<PhasingPollLinkedTransactionModel>> {
        let records = sqlx::query_as::<_, PhasingPollLinkedTransactionModel>(
            "SELECT * FROM phasing_poll_linked_transaction WHERE transaction_id = $1"
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

pub struct PgAssetDividendRepository {
    pool: PgPool,
}

impl PgAssetDividendRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<AssetDividendModel> for PgAssetDividendRepository {
    async fn insert(&self, model: &AssetDividendModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO asset_dividend (id, asset_id, amount, dividend_height, total_dividend, num_accounts, timestamp, height)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
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
            "SELECT * FROM asset_dividend WHERE db_id = $1"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _model: &AssetDividendModel) -> RepositoryResult<()> {
        Ok(())
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AssetDividendModel>> {
        let records = sqlx::query_as::<_, AssetDividendModel>(
            "SELECT * FROM asset_dividend ORDER BY height DESC LIMIT $1 OFFSET $2"
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
impl AssetDividendRepository for PgAssetDividendRepository {
    async fn find_by_asset(&self, asset_id: i64) -> RepositoryResult<Vec<AssetDividendModel>> {
        let records = sqlx::query_as::<_, AssetDividendModel>(
            "SELECT * FROM asset_dividend WHERE asset_id = $1 ORDER BY height DESC"
        )
        .bind(asset_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

pub struct PgPhasingVoteRepository {
    pool: PgPool,
}

impl PgPhasingVoteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<PhasingVoteModel> for PgPhasingVoteRepository {
    async fn insert(&self, model: &PhasingVoteModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO phasing_vote (vote_id, transaction_id, voter_id, height)
            VALUES ($1, $2, $3, $4)
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
            "SELECT * FROM phasing_vote WHERE db_id = $1"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, _model: &PhasingVoteModel) -> RepositoryResult<()> {
        Ok(())
    }

    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PhasingVoteModel>> {
        let records = sqlx::query_as::<_, PhasingVoteModel>(
            "SELECT * FROM phasing_vote ORDER BY height DESC LIMIT $1 OFFSET $2"
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
impl PhasingVoteRepository for PgPhasingVoteRepository {
    async fn find_by_poll(&self, poll_id: i64) -> RepositoryResult<Vec<PhasingVoteModel>> {
        let records = sqlx::query_as::<_, PhasingVoteModel>(
            "SELECT * FROM phasing_vote WHERE transaction_id = $1"
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

pub struct PgPollResultRepository {
    pool: PgPool,
}

impl PgPollResultRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<PollResultModel> for PgPollResultRepository {
    async fn insert(&self, model: &PollResultModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO poll_result (poll_id, result, weight, height)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(model.poll_id)
        .bind(&model.result)
        .bind(model.weight)
        .bind(model.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<PollResultModel>> {
        let record = sqlx::query_as::<_, PollResultModel>(
            "SELECT * FROM poll_result WHERE db_id = $1"
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
                poll_id = $1, result = $2, height = $3
            WHERE db_id = $4
            "#,
        )
        .bind(model.poll_id)
        .bind(&model.result)
        .bind(model.height)
        .bind(model.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM poll_result WHERE db_id = $1")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PollResultModel>> {
        let records = sqlx::query_as::<_, PollResultModel>(
            "SELECT * FROM poll_result ORDER BY height DESC LIMIT $1 OFFSET $2"
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
impl PollResultRepository for PgPollResultRepository {
    async fn find_by_poll(&self, poll_id: i64) -> RepositoryResult<Vec<PollResultModel>> {
        let records = sqlx::query_as::<_, PollResultModel>(
            "SELECT * FROM poll_result WHERE poll_id = $1"
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn upsert(&self, model: &PollResultModel) -> RepositoryResult<()> {
        let existing = sqlx::query_as::<_, PollResultModel>(
            "SELECT * FROM poll_result WHERE poll_id = $1 AND result = $2"
        )
        .bind(model.poll_id)
        .bind(&model.result)
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

// ============================================================================
// PhasingPollHashedSecretRepository (PostgreSQL)
// ============================================================================

pub struct PgPhasingPollHashedSecretRepository {
    pool: PgPool,
}

impl PgPhasingPollHashedSecretRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<PhasingPollHashedSecretModel> for PgPhasingPollHashedSecretRepository {
    async fn insert(&self, model: &PhasingPollHashedSecretModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO phasing_poll_hashed_secret (hashed_secret, hashed_secret_id, algorithm, transaction_full_hash, transaction_id, chain_id, finish_height, height)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&model.hashed_secret)
        .bind(model.hashed_secret_id)
        .bind(model.algorithm)
        .bind(&model.transaction_full_hash)
        .bind(model.transaction_id)
        .bind(model.chain_id)
        .bind(model.finish_height)
        .bind(model.height)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<PhasingPollHashedSecretModel>> {
        let record = sqlx::query_as::<_, PhasingPollHashedSecretModel>(
            "SELECT * FROM phasing_poll_hashed_secret WHERE db_id = $1"
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, model: &PhasingPollHashedSecretModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE phasing_poll_hashed_secret SET
                hashed_secret = $1, hashed_secret_id = $2, algorithm = $3,
                transaction_full_hash = $4, transaction_id = $5,
                chain_id = $6, finish_height = $7, height = $8
            WHERE db_id = $9
            "#,
        )
        .bind(&model.hashed_secret)
        .bind(model.hashed_secret_id)
        .bind(model.algorithm)
        .bind(&model.transaction_full_hash)
        .bind(model.transaction_id)
        .bind(model.chain_id)
        .bind(model.finish_height)
        .bind(model.height)
        .bind(model.db_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM phasing_poll_hashed_secret WHERE db_id = $1")
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PhasingPollHashedSecretModel>> {
        let lim = limit.unwrap_or(100);
        let off = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, PhasingPollHashedSecretModel>(
            "SELECT * FROM phasing_poll_hashed_secret ORDER BY height DESC LIMIT $1 OFFSET $2"
        )
        .bind(lim)
        .bind(off)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM phasing_poll_hashed_secret")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl PhasingPollHashedSecretRepository for PgPhasingPollHashedSecretRepository {
    async fn find_by_poll(&self, poll_id: i64) -> RepositoryResult<Vec<PhasingPollHashedSecretModel>> {
        let records = sqlx::query_as::<_, PhasingPollHashedSecretModel>(
            "SELECT * FROM phasing_poll_hashed_secret WHERE hashed_secret_id = $1"
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}

// ============================================================================
// HubRepository (PostgreSQL)
// ============================================================================

pub struct PgHubRepository {
    pool: PgPool,
}

impl PgHubRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl Repository<HubModel> for PgHubRepository {
    async fn insert(&self, m: &HubModel) -> RepositoryResult<()> {
        sqlx::query("INSERT INTO hub (account_id, uris, min_fee_per_byte, height) VALUES ($1, $2, $3, $4)")
            .bind(m.account_id).bind(&m.uris).bind(m.min_fee_per_byte).bind(m.height)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn find_by_id(&self, id: i64) -> RepositoryResult<Option<HubModel>> {
        sqlx::query_as::<_, HubModel>("SELECT * FROM hub WHERE db_id = $1").bind(id)
            .fetch_optional(&self.pool).await.map_err(RepositoryError::DbError)
    }
    async fn update(&self, m: &HubModel) -> RepositoryResult<()> {
        sqlx::query("UPDATE hub SET account_id=$1, uris=$2, min_fee_per_byte=$3, height=$4 WHERE db_id=$5")
            .bind(m.account_id).bind(&m.uris).bind(m.min_fee_per_byte).bind(m.height).bind(m.db_id)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM hub WHERE db_id=$1").bind(id).execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<HubModel>> {
        let lim = limit.unwrap_or(100);
        let off = offset.unwrap_or(0);
        sqlx::query_as::<_, HubModel>("SELECT * FROM hub ORDER BY height DESC LIMIT $1 OFFSET $2")
            .bind(lim).bind(off).fetch_all(&self.pool).await.map_err(RepositoryError::DbError)
    }
    async fn count(&self) -> RepositoryResult<i64> {
        let (c,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM hub").fetch_one(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(c)
    }
}

#[async_trait]
impl HubRepository for PgHubRepository {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Option<HubModel>> {
        sqlx::query_as::<_, HubModel>("SELECT * FROM hub WHERE account_id = $1 LIMIT 1").bind(account_id)
            .fetch_optional(&self.pool).await.map_err(RepositoryError::DbError)
    }
}

// ============================================================================
// CurrencyFounderRepository (PostgreSQL)
// ============================================================================

pub struct PgCurrencyFounderRepository {
    pool: PgPool,
}

impl PgCurrencyFounderRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl Repository<CurrencyFounderModel> for PgCurrencyFounderRepository {
    async fn insert(&self, m: &CurrencyFounderModel) -> RepositoryResult<()> {
        sqlx::query("INSERT INTO currency_founder (currency_id, account_id, amount, height) VALUES ($1, $2, $3, $4)")
            .bind(m.currency_id).bind(m.account_id).bind(m.amount).bind(m.height)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn find_by_id(&self, id: i64) -> RepositoryResult<Option<CurrencyFounderModel>> {
        sqlx::query_as::<_, CurrencyFounderModel>("SELECT * FROM currency_founder WHERE db_id = $1").bind(id)
            .fetch_optional(&self.pool).await.map_err(RepositoryError::DbError)
    }
    async fn update(&self, m: &CurrencyFounderModel) -> RepositoryResult<()> {
        sqlx::query("UPDATE currency_founder SET currency_id=$1, account_id=$2, amount=$3, height=$4 WHERE db_id=$5")
            .bind(m.currency_id).bind(m.account_id).bind(m.amount).bind(m.height).bind(m.db_id)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM currency_founder WHERE db_id=$1").bind(id).execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<CurrencyFounderModel>> {
        let lim = limit.unwrap_or(100);
        let off = offset.unwrap_or(0);
        sqlx::query_as::<_, CurrencyFounderModel>("SELECT * FROM currency_founder ORDER BY height DESC LIMIT $1 OFFSET $2")
            .bind(lim).bind(off).fetch_all(&self.pool).await.map_err(RepositoryError::DbError)
    }
    async fn count(&self) -> RepositoryResult<i64> {
        let (c,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM currency_founder").fetch_one(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(c)
    }
}

#[async_trait]
impl CurrencyFounderRepository for PgCurrencyFounderRepository {
    async fn find_by_currency(&self, currency_id: i64) -> RepositoryResult<Vec<CurrencyFounderModel>> {
        sqlx::query_as::<_, CurrencyFounderModel>("SELECT * FROM currency_founder WHERE currency_id = $1")
            .bind(currency_id).fetch_all(&self.pool).await.map_err(RepositoryError::DbError)
    }
}

// ============================================================================
// PrunableMessageRepository (PostgreSQL)
// ============================================================================

pub struct PgPrunableMessageRepository {
    pool: PgPool,
}

impl PgPrunableMessageRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl Repository<PrunableMessageModel> for PgPrunableMessageRepository {
    async fn insert(&self, m: &PrunableMessageModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"INSERT INTO prunable_message (id, sender_id, recipient_id, message, message_is_text, is_compressed, encrypted_message, encrypted_is_text, block_timestamp, transaction_timestamp, height)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"#)
            .bind(m.id).bind(m.sender_id).bind(m.recipient_id).bind(&m.message)
            .bind(m.message_is_text).bind(m.is_compressed).bind(&m.encrypted_message)
            .bind(m.encrypted_is_text).bind(m.block_timestamp).bind(m.transaction_timestamp).bind(m.height)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn find_by_id(&self, id: i64) -> RepositoryResult<Option<PrunableMessageModel>> {
        sqlx::query_as::<_, PrunableMessageModel>("SELECT * FROM prunable_message WHERE db_id = $1").bind(id)
            .fetch_optional(&self.pool).await.map_err(RepositoryError::DbError)
    }
    async fn update(&self, m: &PrunableMessageModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"UPDATE prunable_message SET id=$1, sender_id=$2, recipient_id=$3, message=$4, message_is_text=$5,
               is_compressed=$6, encrypted_message=$7, encrypted_is_text=$8, block_timestamp=$9, transaction_timestamp=$10, height=$11
            WHERE db_id=$12"#)
            .bind(m.id).bind(m.sender_id).bind(m.recipient_id).bind(&m.message)
            .bind(m.message_is_text).bind(m.is_compressed).bind(&m.encrypted_message)
            .bind(m.encrypted_is_text).bind(m.block_timestamp).bind(m.transaction_timestamp).bind(m.height).bind(m.db_id)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM prunable_message WHERE db_id=$1").bind(id).execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PrunableMessageModel>> {
        let lim = limit.unwrap_or(100);
        let off = offset.unwrap_or(0);
        sqlx::query_as::<_, PrunableMessageModel>("SELECT * FROM prunable_message ORDER BY height DESC LIMIT $1 OFFSET $2")
            .bind(lim).bind(off).fetch_all(&self.pool).await.map_err(RepositoryError::DbError)
    }
    async fn count(&self) -> RepositoryResult<i64> {
        let (c,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM prunable_message").fetch_one(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(c)
    }
}

// ============================================================================
// PurchaseFeedbackRepository (PostgreSQL)
// ============================================================================

pub struct PgPurchaseFeedbackRepository {
    pool: PgPool,
}

impl PgPurchaseFeedbackRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl Repository<PurchaseFeedbackModel> for PgPurchaseFeedbackRepository {
    async fn insert(&self, m: &PurchaseFeedbackModel) -> RepositoryResult<()> {
        sqlx::query("INSERT INTO purchase_feedback (purchase_id, feedback_data, feedback_nonce, height) VALUES ($1, $2, $3, $4)")
            .bind(m.id).bind(&m.feedback_data).bind(&m.feedback_nonce).bind(m.height)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn find_by_id(&self, id: i64) -> RepositoryResult<Option<PurchaseFeedbackModel>> {
        sqlx::query_as::<_, PurchaseFeedbackModel>("SELECT * FROM purchase_feedback WHERE db_id = $1").bind(id)
            .fetch_optional(&self.pool).await.map_err(RepositoryError::DbError)
    }
    async fn update(&self, m: &PurchaseFeedbackModel) -> RepositoryResult<()> {
        sqlx::query("UPDATE purchase_feedback SET id=$1, feedback_data=$2, feedback_nonce=$3, height=$4 WHERE db_id=$5")
            .bind(m.id).bind(&m.feedback_data).bind(&m.feedback_nonce).bind(m.height).bind(m.db_id)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM purchase_feedback WHERE db_id=$1").bind(id).execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PurchaseFeedbackModel>> {
        let lim = limit.unwrap_or(100);
        let off = offset.unwrap_or(0);
        sqlx::query_as::<_, PurchaseFeedbackModel>("SELECT * FROM purchase_feedback ORDER BY height DESC LIMIT $1 OFFSET $2")
            .bind(lim).bind(off).fetch_all(&self.pool).await.map_err(RepositoryError::DbError)
    }
    async fn count(&self) -> RepositoryResult<i64> {
        let (c,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM purchase_feedback").fetch_one(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(c)
    }
}

#[async_trait]
impl PurchaseFeedbackRepository for PgPurchaseFeedbackRepository {
    async fn find_by_purchase(&self, purchase_id: i64) -> RepositoryResult<Vec<PurchaseFeedbackModel>> {
        sqlx::query_as::<_, PurchaseFeedbackModel>("SELECT * FROM purchase_feedback WHERE purchase_id = $1")
            .bind(purchase_id).fetch_all(&self.pool).await.map_err(RepositoryError::DbError)
    }
}

// ============================================================================
// ReferencedTransactionRepository (PostgreSQL)
// ============================================================================

pub struct PgReferencedTransactionRepository {
    pool: PgPool,
}

impl PgReferencedTransactionRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl Repository<ReferencedTransactionModel> for PgReferencedTransactionRepository {
    async fn insert(&self, m: &ReferencedTransactionModel) -> RepositoryResult<()> {
        sqlx::query("INSERT INTO referenced_transaction (transaction_id, referenced_transaction_id) VALUES ($1, $2)")
            .bind(m.transaction_id).bind(m.referenced_transaction_id)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, id: i64) -> RepositoryResult<Option<ReferencedTransactionModel>> {
        sqlx::query_as::<_, ReferencedTransactionModel>("SELECT * FROM referenced_transaction WHERE db_id = $1").bind(id)
            .fetch_optional(&self.pool).await.map_err(RepositoryError::DbError)
    }

    async fn update(&self, m: &ReferencedTransactionModel) -> RepositoryResult<()> {
        sqlx::query("UPDATE referenced_transaction SET transaction_id=$1, referenced_transaction_id=$2 WHERE db_id=$3")
            .bind(m.transaction_id).bind(m.referenced_transaction_id).bind(m.db_id)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM referenced_transaction WHERE db_id=$1").bind(id)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<ReferencedTransactionModel>> {
        let lim = limit.unwrap_or(100);
        let off = offset.unwrap_or(0);
        sqlx::query_as::<_, ReferencedTransactionModel>("SELECT * FROM referenced_transaction ORDER BY db_id DESC LIMIT $1 OFFSET $2")
            .bind(lim).bind(off).fetch_all(&self.pool).await.map_err(RepositoryError::DbError)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (c,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM referenced_transaction")
            .fetch_one(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(c)
    }
}

#[async_trait]
impl ReferencedTransactionRepository for PgReferencedTransactionRepository {
    async fn find_by_transaction(&self, transaction_id: i64) -> RepositoryResult<Vec<ReferencedTransactionModel>> {
        sqlx::query_as::<_, ReferencedTransactionModel>("SELECT * FROM referenced_transaction WHERE transaction_id = $1")
            .bind(transaction_id).fetch_all(&self.pool).await.map_err(RepositoryError::DbError)
    }
}

// ============================================================================
// TaggedDataExtendRepository (PostgreSQL)
// ============================================================================

pub struct PgTaggedDataExtendRepository {
    pool: PgPool,
}

impl PgTaggedDataExtendRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<TaggedDataExtendModel> for PgTaggedDataExtendRepository {
    async fn insert(&self, model: &TaggedDataExtendModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO tagged_data_extend (id, extend_id, height, latest)
            VALUES ($1, $2, $3, $4)
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
        let record = sqlx::query_as::<_, TaggedDataExtendModel>("SELECT * FROM tagged_data_extend WHERE db_id = $1")
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
        let lim = limit.unwrap_or(100);
        let off = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, TaggedDataExtendModel>(
            "SELECT * FROM tagged_data_extend WHERE latest = true ORDER BY height DESC LIMIT $1 OFFSET $2"
        )
        .bind(lim)
        .bind(off)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tagged_data_extend WHERE latest = true")
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }
}

#[async_trait]
impl TaggedDataExtendRepository for PgTaggedDataExtendRepository {
    async fn find_by_extend_id(&self, extend_id: i64) -> RepositoryResult<Vec<TaggedDataExtendModel>> {
        let records = sqlx::query_as::<_, TaggedDataExtendModel>(
            "SELECT * FROM tagged_data_extend WHERE extend_id = $1 AND latest = true ORDER BY height DESC"
        )
        .bind(extend_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}
