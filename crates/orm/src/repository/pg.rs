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
            r#"INSERT INTO shuffling_data ("SHUFFLING_ID", "ACCOUNT_ID", "DATA", "TRANSACTION_TIMESTAMP", "HEIGHT") VALUES ($1, $2, $3, $4, $5)"#
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
            r#"SELECT * FROM shuffling_data WHERE "DB_ID" = $1"#
        )
        .bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, model: &ShufflingDataModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"UPDATE shuffling_data SET "SHUFFLING_ID" = $1, "ACCOUNT_ID" = $2, "DATA" = $3, "TRANSACTION_TIMESTAMP" = $4, "HEIGHT" = $5 WHERE "DB_ID" = $6"#
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
        sqlx::query(r#"DELETE FROM shuffling_data WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM shuffling_data ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM shuffling_data WHERE "SHUFFLING_ID" = $1"#
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
                "SHUFFLING_ID", "ACCOUNT_ID", "NEXT_ACCOUNT_ID", "PARTICIPANT_INDEX",
                "STATE", "BLAME_DATA", "KEY_SEEDS", "DATA_TRANSACTION_FULL_HASH", "HEIGHT", "LATEST"
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
            r#"SELECT * FROM shuffling_participant WHERE "DB_ID" = $1"#
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
                "SHUFFLING_ID" = $1, "ACCOUNT_ID" = $2, "NEXT_ACCOUNT_ID" = $3,
                "PARTICIPANT_INDEX" = $4, "STATE" = $5, "BLAME_DATA" = $6,
                "KEY_SEEDS" = $7, "DATA_TRANSACTION_FULL_HASH" = $8, "HEIGHT" = $9, "LATEST" = $10
            WHERE "DB_ID" = $11"#
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
        sqlx::query(r#"DELETE FROM shuffling_participant WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM shuffling_participant WHERE "LATEST" = true ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
        )
        .bind(lim)
        .bind(off)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM shuffling_participant WHERE "LATEST" = true"#)
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
            r#"SELECT * FROM shuffling_participant WHERE "SHUFFLING_ID" = $1 AND "LATEST" = true"#
        )
        .bind(shuffling_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_account_and_shuffling(&self, account_id: i64, shuffling_id: i64) -> RepositoryResult<Option<ShufflingParticipantModel>> {
        let record = sqlx::query_as::<_, ShufflingParticipantModel>(
            r#"SELECT * FROM shuffling_participant WHERE "ACCOUNT_ID" = $1 AND "SHUFFLING_ID" = $2 AND "LATEST" = true LIMIT 1"#
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
        let record = sqlx::query_as::<_, BlockModel>(r#"SELECT * FROM block WHERE "HEIGHT" = $1"#).bind(height)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_id_column(&self, id: i64) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as::<_, BlockModel>(r#"SELECT * FROM block WHERE "ID" = $1"#).bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_hash(&self, hash: &[u8]) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as::<_, BlockModel>(r#"SELECT * FROM block WHERE "PAYLOAD_HASH" = $1 OR "GENERATION_SIGNATURE" = $1"#).bind(hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_latest(&self) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as::<_, BlockModel>(r#"SELECT * FROM block ORDER BY "HEIGHT" DESC LIMIT 1"#)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_range(&self, start_height: i32, end_height: i32) -> RepositoryResult<Vec<BlockModel>> {
        let records = sqlx::query_as::<_, BlockModel>(r#"SELECT * FROM block WHERE "HEIGHT" BETWEEN $1 AND $2 ORDER BY "HEIGHT" ASC"#).bind(start_height).bind(end_height)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_generator(&self, generator_id: i64) -> RepositoryResult<Vec<BlockModel>> {
        let records = sqlx::query_as::<_, BlockModel>(r#"SELECT * FROM block WHERE "GENERATOR_ID" = $1 ORDER BY "HEIGHT" DESC"#).bind(generator_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn get_height(&self) -> RepositoryResult<i32> {
        let record = sqlx::query_as::<_, BlockModel>(r#"SELECT * FROM block ORDER BY "HEIGHT" DESC LIMIT 1"#)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record.map(|b| b.height).unwrap_or(0))
    }

    async fn get_block_id_at_height(&self, height: i32) -> RepositoryResult<Option<i64>> {
        let record = sqlx::query_as::<_, BlockModel>(r#"SELECT * FROM block WHERE "HEIGHT" = $1"#).bind(height)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record.map(|b| b.id))
    }

    async fn has_block(&self, id: i64) -> RepositoryResult<bool> {
        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM block WHERE "ID" = $1"#
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
        
        let records = sqlx::query_as::<_, BlockModel>(r#"SELECT * FROM block WHERE "HEIGHT" > $1 ORDER BY "HEIGHT" ASC LIMIT $2"#).bind(height).bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        
        Ok(records.into_iter().map(|b| b.id).collect())
    }

    async fn update_next_block_id(&self, previous_block_id: i64, next_block_id: i64) -> RepositoryResult<()> {
        sqlx::query(r#"UPDATE block SET "NEXT_BLOCK_ID" = $1 WHERE "ID" = $2"#)
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
            sqlx::query(r#"DELETE FROM block WHERE "ID" = $1"#)
                .bind(block.id)
                .execute(&self.pool)
                .await
                .map_err(RepositoryError::DbError)?;
        }
        
        if let Some(last_block) = blocks.last() {
            sqlx::query(r#"UPDATE block SET "NEXT_BLOCK_ID" = NULL WHERE "ID" = $1"#)
                .bind(last_block.previous_block_id)
                .execute(&self.pool)
                .await
                .map_err(RepositoryError::DbError)?;
        }
        
        Ok(blocks)
    }

    async fn find_blocks_after_height(&self, height: i32) -> RepositoryResult<Vec<BlockModel>> {
        let records = sqlx::query_as::<_, BlockModel>(r#"SELECT * FROM block WHERE "HEIGHT" > $1 ORDER BY "HEIGHT" ASC"#).bind(height)
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
            sqlx::query(r#"DELETE FROM block WHERE "DB_ID" = $1"#)
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
                "ID", "VERSION", timestamp, "PREVIOUS_BLOCK_ID", "TOTAL_AMOUNT",
                "TOTAL_FEE", "PAYLOAD_LENGTH", "PREVIOUS_BLOCK_HASH", "CUMULATIVE_DIFFICULTY",
                "BASE_TARGET", "NEXT_BLOCK_ID", "HEIGHT", "GENERATION_SIGNATURE",
                "BLOCK_SIGNATURE", "PAYLOAD_HASH", "GENERATOR_ID"
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
        sqlx::query(r#"UPDATE block SET "NEXT_BLOCK_ID" = $1 WHERE "ID" = $2"#)
            .bind(next_block_id)
            .bind(previous_block_id)
            .execute(&mut **tx)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete_by_db_id_tx(&self, db_id: i64, tx: &mut DbTransaction<'_>) -> RepositoryResult<()> {
        sqlx::query(r#"DELETE FROM block WHERE "DB_ID" = $1"#)
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
                "ID", "VERSION", timestamp, "PREVIOUS_BLOCK_ID", "TOTAL_AMOUNT",
                "TOTAL_FEE", "PAYLOAD_LENGTH", "PREVIOUS_BLOCK_HASH", "CUMULATIVE_DIFFICULTY",
                "BASE_TARGET", "NEXT_BLOCK_ID", "HEIGHT", "GENERATION_SIGNATURE",
                "BLOCK_SIGNATURE", "PAYLOAD_HASH", "GENERATOR_ID"
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
        let record = sqlx::query_as::<_, BlockModel>(r#"SELECT * FROM block WHERE "DB_ID" = $1"#).bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, block: &BlockModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE block SET
                "VERSION" = $2, timestamp = $3, "PREVIOUS_BLOCK_ID" = $4,
                "TOTAL_AMOUNT" = $5, "TOTAL_FEE" = $6, "PAYLOAD_LENGTH" = $7,
                "PREVIOUS_BLOCK_HASH" = $8, "CUMULATIVE_DIFFICULTY" = $9,
                "BASE_TARGET" = $10, "NEXT_BLOCK_ID" = $11, "HEIGHT" = $12,
                "GENERATION_SIGNATURE" = $13, "BLOCK_SIGNATURE" = $14,
                "PAYLOAD_HASH" = $15, "GENERATOR_ID" = $16
            WHERE "DB_ID" = $1
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
        sqlx::query(r#"DELETE FROM block WHERE "DB_ID" = $1"#)
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<BlockModel>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let records = sqlx::query_as::<_, BlockModel>(r#"SELECT * FROM block ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#).bind(limit).bind(offset)
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
            r#"SELECT public_key, "HEIGHT" FROM public_key WHERE "ACCOUNT_ID" = $1 AND "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT 1"#
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
            INSERT INTO public_key ("ACCOUNT_ID", public_key, "HEIGHT")
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
            INSERT INTO account_property ("ID", "RECIPIENT_ID", "SETTER_ID", "PROPERTY", "VALUE", "HEIGHT", "LATEST")
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
            r#"SELECT * FROM account_property WHERE "DB_ID" = $1"#
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
                "RECIPIENT_ID" = $1, "SETTER_ID" = $2, "PROPERTY" = $3, "VALUE" = $4,
                "HEIGHT" = $5, "LATEST" = $6
            WHERE "DB_ID" = $7
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
        sqlx::query(r#"DELETE FROM account_property WHERE "DB_ID" = $1"#)
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AccountPropertyModel>> {
        let records = sqlx::query_as::<_, AccountPropertyModel>(
            r#"SELECT * FROM account_property ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM account_property WHERE "RECIPIENT_ID" = $1 AND "LATEST" = TRUE"#
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_property(&self, account_id: i64, property: &str) -> RepositoryResult<Option<AccountPropertyModel>> {
        let record = sqlx::query_as::<_, AccountPropertyModel>(
            r#"SELECT * FROM account_property WHERE "RECIPIENT_ID" = $1 AND "PROPERTY" = $2 AND "LATEST" = TRUE"#
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
            r#"UPDATE account_property SET "LATEST" = FALSE WHERE "RECIPIENT_ID" = $1 AND "PROPERTY" = $2 AND "LATEST" = TRUE"#
        )
        .bind(model.recipient_id)
        .bind(&model.property)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        self.insert(model).await
    }

    async fn delete_by_id(&self, id: i64) -> RepositoryResult<()> {
        sqlx::query(r#"UPDATE account_property SET "LATEST" = FALSE WHERE "ID" = $1 AND "LATEST" = TRUE"#)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn soft_delete_by_id(&self, db_id: i64) -> RepositoryResult<()> {
        sqlx::query(r#"UPDATE account_property SET "LATEST" = FALSE WHERE "DB_ID" = $1 AND "LATEST" = TRUE"#)
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
                "ACCOUNT_ID", "WHITELIST", "VOTING_MODEL", "QUORUM", "MIN_BALANCE",
                "HOLDING_ID", "MIN_BALANCE_MODEL", "MAX_FEES", "MIN_DURATION",
                "MAX_DURATION", "HEIGHT", "LATEST"
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
            r#"SELECT * FROM account_control_phasing WHERE "DB_ID" = $1"#
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
                "ACCOUNT_ID" = $1, "WHITELIST" = $2, "VOTING_MODEL" = $3, "QUORUM" = $4,
                "MIN_BALANCE" = $5, "HOLDING_ID" = $6, "MIN_BALANCE_MODEL" = $7,
                "MAX_FEES" = $8, "MIN_DURATION" = $9, "MAX_DURATION" = $10,
                "HEIGHT" = $11, "LATEST" = $12
            WHERE "DB_ID" = $13
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
        sqlx::query(r#"DELETE FROM account_control_phasing WHERE "DB_ID" = $1"#)
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AccountControlPhasingModel>> {
        let records = sqlx::query_as::<_, AccountControlPhasingModel>(
            r#"SELECT * FROM account_control_phasing ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM account_control_phasing WHERE "ACCOUNT_ID" = $1 AND "LATEST" = TRUE"#
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn upsert(&self, model: &AccountControlPhasingModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"UPDATE account_control_phasing SET "LATEST" = FALSE WHERE "ACCOUNT_ID" = $1 AND "LATEST" = TRUE"#
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
            INSERT INTO account_info ("ACCOUNT_ID", "NAME", "DESCRIPTION", "HEIGHT", "LATEST")
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
            r#"SELECT * FROM account_info WHERE "DB_ID" = $1"#
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
                "NAME" = $1, "DESCRIPTION" = $2, "HEIGHT" = $3, "LATEST" = $4
            WHERE "DB_ID" = $5
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
        sqlx::query(r#"DELETE FROM account_info WHERE "DB_ID" = $1"#)
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AccountInfoModel>> {
        let records = sqlx::query_as::<_, AccountInfoModel>(
            r#"SELECT * FROM account_info ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM account_info WHERE "ACCOUNT_ID" = $1 AND "LATEST" = TRUE"#
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn upsert(&self, model: &AccountInfoModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"UPDATE account_info SET "LATEST" = FALSE WHERE "ACCOUNT_ID" = $1 AND "LATEST" = TRUE"#
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
                "LESSOR_ID", "CURRENT_LEASING_HEIGHT_FROM", "CURRENT_LEASING_HEIGHT_TO",
                "CURRENT_LESSEE_ID", "NEXT_LEASING_HEIGHT_FROM", "NEXT_LEASING_HEIGHT_TO",
                "NEXT_LESSEE_ID", "HEIGHT", "LATEST"
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
            r#"SELECT * FROM account_lease WHERE "DB_ID" = $1"#
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
                "CURRENT_LEASING_HEIGHT_FROM" = $1, "CURRENT_LEASING_HEIGHT_TO" = $2,
                "CURRENT_LESSEE_ID" = $3, "NEXT_LEASING_HEIGHT_FROM" = $4,
                "NEXT_LEASING_HEIGHT_TO" = $5, "NEXT_LESSEE_ID" = $6,
                "HEIGHT" = $7, "LATEST" = $8
            WHERE "DB_ID" = $9
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
        sqlx::query(r#"DELETE FROM account_lease WHERE "DB_ID" = $1"#)
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<AccountLeaseModel>> {
        let records = sqlx::query_as::<_, AccountLeaseModel>(
            r#"SELECT * FROM account_lease ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM account_lease WHERE "LESSOR_ID" = $1 AND "LATEST" = TRUE"#
        )
        .bind(lessor_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn upsert(&self, model: &AccountLeaseModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"UPDATE account_lease SET "LATEST" = FALSE WHERE "LESSOR_ID" = $1 AND "LATEST" = TRUE"#
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
            r#"SELECT * FROM trade WHERE "ASSET_ID" = $1 ORDER BY "HEIGHT" DESC, timestamp DESC LIMIT $2"#
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
            r#"SELECT * FROM trade WHERE "ASK_ORDER_ID" = $1 ORDER BY "HEIGHT" DESC"#
        )
        .bind(ask_order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_bid_order(&self, bid_order_id: i64) -> RepositoryResult<Vec<TradeModel>> {
        let records = sqlx::query_as::<_, TradeModel>(
            r#"SELECT * FROM trade WHERE "BID_ORDER_ID" = $1 ORDER BY "HEIGHT" DESC"#
        )
        .bind(bid_order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_buyer(&self, buyer_id: i64, limit: i64) -> RepositoryResult<Vec<TradeModel>> {
        let records = sqlx::query_as::<_, TradeModel>(
            r#"SELECT * FROM trade WHERE "BUYER_ID" = $1 ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            r#"SELECT * FROM trade WHERE "SELLER_ID" = $1 ORDER BY "HEIGHT" DESC LIMIT $2"#
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
                "ASSET_ID", "BLOCK_ID", "ASK_ORDER_ID", "BID_ORDER_ID", "ASK_ORDER_HEIGHT",
                "BID_ORDER_HEIGHT", "SELLER_ID", "BUYER_ID", "IS_BUY", "QUANTITY", "PRICE", timestamp, "HEIGHT"
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
            r#"SELECT * FROM trade WHERE "DB_ID" = $1"#
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
            r#"SELECT * FROM trade ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM goods WHERE "ID" = $1 AND "LATEST" = TRUE LIMIT 1"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_seller(&self, seller_id: i64, limit: i64) -> RepositoryResult<Vec<GoodsModel>> {
        let records = sqlx::query_as::<_, GoodsModel>(
            r#"SELECT * FROM goods WHERE "SELLER_ID" = $1 AND "LATEST" = TRUE AND "DELISTED" = FALSE ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            r#"SELECT * FROM goods WHERE "LATEST" = TRUE AND "DELISTED" = FALSE AND "QUANTITY" > 0 ORDER BY "HEIGHT" DESC LIMIT $1"#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn update_quantity(&self, goods_id: i64, quantity: i32) -> RepositoryResult<()> {
        sqlx::query(r#"UPDATE goods SET "QUANTITY" = $1 WHERE "ID" = $2 AND "LATEST" = TRUE"#)
            .bind(quantity)
            .bind(goods_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn update_price(&self, goods_id: i64, price: i64) -> RepositoryResult<()> {
        sqlx::query(r#"UPDATE goods SET "PRICE" = $1 WHERE "ID" = $2 AND "LATEST" = TRUE"#)
            .bind(price)
            .bind(goods_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn set_delisted(&self, goods_id: i64, delisted: bool) -> RepositoryResult<()> {
        sqlx::query(r#"UPDATE goods SET "DELISTED" = $1 WHERE "ID" = $2 AND "LATEST" = TRUE"#)
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
                "ID", "SELLER_ID", "NAME", "DESCRIPTION", "PARSED_TAGS", "TAGS",
                timestamp, "QUANTITY", "PRICE", "DELISTED", "HEIGHT", "LATEST", "HAS_IMAGE"
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
        let record = sqlx::query_as::<_, GoodsModel>(r#"SELECT * FROM goods WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM goods WHERE "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM goods WHERE "LATEST" = TRUE"#)
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
            r#"SELECT * FROM currency WHERE "ID" = $1 AND "LATEST" = TRUE LIMIT 1"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_code(&self, code: &str) -> RepositoryResult<Option<CurrencyModel>> {
        let record = sqlx::query_as::<_, CurrencyModel>(
            r#"SELECT * FROM currency WHERE "CODE" = $1 AND "LATEST" = TRUE LIMIT 1"#
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_owner(&self, account_id: i64) -> RepositoryResult<Vec<CurrencyModel>> {
        let records = sqlx::query_as::<_, CurrencyModel>(
            r#"SELECT * FROM currency WHERE "ACCOUNT_ID" = $1 AND "LATEST" = TRUE ORDER BY "HEIGHT" DESC"#
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<CurrencyModel>> {
        let records = sqlx::query_as::<_, CurrencyModel>(
            r#"SELECT * FROM currency WHERE "HEIGHT" = $1 AND "LATEST" = TRUE"#
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
            UPDATE currency SET "INITIAL_SUPPLY" = "INITIAL_SUPPLY" + $2, "LATEST" = TRUE
            WHERE "ID" = $1 AND "LATEST" = TRUE
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
            r#"SELECT COALESCE("INITIAL_SUPPLY", 0) FROM currency WHERE "ID" = $1 AND "LATEST" = TRUE"#
        )
        .bind(currency_id)
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        let total_increase = amount_per_unit * current_supply;

        sqlx::query(
            r#"
            UPDATE currency SET "RESERVE_SUPPLY" = "RESERVE_SUPPLY" + $2, "LATEST" = TRUE
            WHERE "ID" = $1 AND "LATEST" = TRUE
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
        sqlx::query(r#"DELETE FROM currency WHERE "ID" = $1"#)
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
                "ID", "ACCOUNT_ID", "NAME", "NAME_LOWER", "CODE", "DESCRIPTION", type,
                "INITIAL_SUPPLY", "RESERVE_SUPPLY", "MAX_SUPPLY", "CREATION_HEIGHT",
                "ISSUANCE_HEIGHT", "MIN_RESERVE_PER_UNIT_NQT", "MIN_DIFFICULTY",
                "MAX_DIFFICULTY", "RULESET", "ALGORITHM", "DECIMALS", "HEIGHT", "LATEST"
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
        let record = sqlx::query_as::<_, CurrencyModel>(r#"SELECT * FROM currency WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM currency WHERE "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM currency WHERE "LATEST" = TRUE"#)
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
            sqlx::query(r#"DELETE FROM "transaction" WHERE "DB_ID" = $1"#)
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
                "ID", "DEADLINE", "RECIPIENT_ID", "AMOUNT", "FEE", "FULL_HASH",
                "HEIGHT", "BLOCK_ID", "SIGNATURE", timestamp, type, "SUBTYPE",
                "SENDER_ID", "BLOCK_TIMESTAMP", "REFERENCED_TRANSACTION_FULL_HASH",
                "TRANSACTION_INDEX", "PHASED", "ATTACHMENT_BYTES", "VERSION",
                "HAS_MESSAGE", "HAS_ENCRYPTED_MESSAGE", "HAS_PUBLIC_KEY_ANNOUNCEMENT",
                "HAS_PRUNABLE_MESSAGE", "HAS_PRUNABLE_ATTACHMENT", "EC_BLOCK_HEIGHT",
                "EC_BLOCK_ID", "HAS_ENCRYPTTOSELF_MESSAGE", "HAS_PRUNABLE_ENCRYPTED_MESSAGE"
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
        sqlx::query(r#"DELETE FROM "transaction" WHERE "DB_ID" = $1"#)
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
                "ID", "DEADLINE", "RECIPIENT_ID", "AMOUNT", "FEE", "FULL_HASH",
                "HEIGHT", "BLOCK_ID", "BLOCK_TIMESTAMP", "TRANSACTION_INDEX", "SIGNATURE", timestamp, type, "SUBTYPE",
                "SENDER_ID", "REFERENCED_TRANSACTION_FULL_HASH",
                "ATTACHMENT_BYTES", "VERSION", "PHASED",
                "HAS_MESSAGE", "HAS_ENCRYPTED_MESSAGE", "HAS_PUBLIC_KEY_ANNOUNCEMENT",
                "HAS_PRUNABLE_MESSAGE", "HAS_PRUNABLE_ATTACHMENT", "EC_BLOCK_HEIGHT",
                "EC_BLOCK_ID", "HAS_ENCRYPTTOSELF_MESSAGE", "HAS_PRUNABLE_ENCRYPTED_MESSAGE"
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
        let record = sqlx::query_as::<_, AccountModel>(r#"SELECT * FROM account WHERE "ID" = $1 AND "LATEST" = TRUE LIMIT 1"#).bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<AccountModel>> {
        let records = sqlx::query_as::<_, AccountModel>(r#"SELECT * FROM account WHERE "HEIGHT" = $1 AND "LATEST" = TRUE"#).bind(height)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_latest_by_id(&self, id: i64) -> RepositoryResult<Option<AccountModel>> {
        self.find_by_account_id(id).await
    }

    async fn find_by_address(&self, _address: &str) -> RepositoryResult<Option<AccountModel>> {
        let record = sqlx::query_as::<_, AccountModel>(r#"SELECT * FROM account WHERE "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT 1"#)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update_balance(&self, account_id: i64, balance: i64, unconfirmed_balance: i64, height: i32) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account
            SET "BALANCE" = $1, "UNCONFIRMED_BALANCE" = $2, "HEIGHT" = $3
            WHERE "ID" = $4 AND "LATEST" = TRUE
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
            SET "BALANCE" = "BALANCE" + $1, "HEIGHT" = $2
            WHERE "ID" = $3 AND "LATEST" = TRUE
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
                SET "BALANCE" = "BALANCE" + $1, "HEIGHT" = $2
                WHERE "ID" = $3 AND "LATEST" = TRUE
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
            SET "UNCONFIRMED_BALANCE" = "UNCONFIRMED_BALANCE" + $1, "HEIGHT" = $2
            WHERE "ID" = $3 AND "LATEST" = TRUE
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
                SET "UNCONFIRMED_BALANCE" = "UNCONFIRMED_BALANCE" + $1, "HEIGHT" = $2
                WHERE "ID" = $3 AND "LATEST" = TRUE
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
            SET "BALANCE" = "BALANCE" + $1, "UNCONFIRMED_BALANCE" = "UNCONFIRMED_BALANCE" + $1, "HEIGHT" = $2
            WHERE "ID" = $3 AND "LATEST" = TRUE
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
                SET "BALANCE" = "BALANCE" + $1, "UNCONFIRMED_BALANCE" = "UNCONFIRMED_BALANCE" + $1, "HEIGHT" = $2
                WHERE "ID" = $3 AND "LATEST" = TRUE
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
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM account WHERE "LATEST" = TRUE"#)
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(count)
    }

    async fn add_to_forged_balance(&self, account_id: i64, amount: i64, height: i32) -> RepositoryResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE account
            SET "FORGED_BALANCE" = "FORGED_BALANCE" + $1, "HEIGHT" = $2
            WHERE "ID" = $3 AND "LATEST" = TRUE
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
                SET "FORGED_BALANCE" = "FORGED_BALANCE" + $1, "HEIGHT" = $2
                WHERE "ID" = $3 AND "LATEST" = TRUE
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
                "ID", "BALANCE", "UNCONFIRMED_BALANCE", "FORGED_BALANCE",
                "ACTIVE_LESSEE_ID", "HAS_CONTROL_PHASING", "HEIGHT", "LATEST"
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
        let record = sqlx::query_as::<_, AccountModel>(r#"SELECT * FROM account WHERE "DB_ID" = $1"#).bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, account: &AccountModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account SET
                "BALANCE" = $2, "UNCONFIRMED_BALANCE" = $3, "FORGED_BALANCE" = $4,
                "ACTIVE_LESSEE_ID" = $5, "HAS_CONTROL_PHASING" = $6, "HEIGHT" = $7, "LATEST" = $8
            WHERE "DB_ID" = $1
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
        let records = sqlx::query_as::<_, AccountModel>(r#"SELECT * FROM account WHERE "LATEST" = TRUE ORDER BY "ID" LIMIT $1 OFFSET $2"#).bind(limit).bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM account WHERE "LATEST" = TRUE"#)
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
        let records = sqlx::query_as::<_, AccountAssetModel>(r#"SELECT * FROM account_asset WHERE "ACCOUNT_ID" = $1 AND "LATEST" = TRUE"#).bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_asset(&self, asset_id: i64) -> RepositoryResult<Vec<AccountAssetModel>> {
        let records = sqlx::query_as::<_, AccountAssetModel>(r#"SELECT * FROM account_asset WHERE "ASSET_ID" = $1 AND "LATEST" = TRUE"#).bind(asset_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_account_and_asset(&self, account_id: i64, asset_id: i64) -> RepositoryResult<Option<AccountAssetModel>> {
        let record = sqlx::query_as::<_, AccountAssetModel>(r#"SELECT * FROM account_asset WHERE "ACCOUNT_ID" = $1 AND "ASSET_ID" = $2 AND "LATEST" = TRUE LIMIT 1"#).bind(account_id).bind(asset_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update_quantity(&self, account_id: i64, asset_id: i64, quantity: i64, height: i32) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account_asset
            SET "QUANTITY" = $1, "HEIGHT" = $2, "LATEST" = TRUE
            WHERE "ACCOUNT_ID" = $3 AND "ASSET_ID" = $4
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
            SET "QUANTITY" = "QUANTITY" + $1, "LATEST" = TRUE
            WHERE "ACCOUNT_ID" = $2 AND "ASSET_ID" = $3
            "#,
        )
        .bind(delta)
        .bind(account_id)
        .bind(asset_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        if result.rows_affected() == 0 && delta != 0 {
            let current_height: i32 = sqlx::query_scalar(r#"SELECT COALESCE(MAX("HEIGHT"), 0) FROM block"#)
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
            sqlx::query(
                r#"INSERT INTO account_asset ("ACCOUNT_ID", "ASSET_ID", "QUANTITY", "UNCONFIRMED_QUANTITY", "HEIGHT", "LATEST") VALUES ($1, $2, $3, 0, $4, TRUE)"#
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
            SET "QUANTITY" = "QUANTITY" - $1, "LATEST" = TRUE
            WHERE "ACCOUNT_ID" = $2 AND "ASSET_ID" = $3 AND "QUANTITY" >= $1
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
            SET "UNCONFIRMED_QUANTITY" = "UNCONFIRMED_QUANTITY" + $1, "LATEST" = TRUE
            WHERE "ACCOUNT_ID" = $2 AND "ASSET_ID" = $3
            "#,
        )
        .bind(delta)
        .bind(account_id)
        .bind(asset_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        if result.rows_affected() == 0 && delta != 0 {
            let current_height: i32 = sqlx::query_scalar(r#"SELECT COALESCE(MAX("HEIGHT"), 0) FROM block"#)
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
            sqlx::query(
                r#"INSERT INTO account_asset ("ACCOUNT_ID", "ASSET_ID", "QUANTITY", "UNCONFIRMED_QUANTITY", "HEIGHT", "LATEST") VALUES ($1, $2, 0, $3, $4, TRUE)"#
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
                "ACCOUNT_ID", "ASSET_ID", "QUANTITY", "UNCONFIRMED_QUANTITY", "HEIGHT", "LATEST"
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
        let record = sqlx::query_as::<_, AccountAssetModel>(r#"SELECT * FROM account_asset WHERE "DB_ID" = $1"#).bind(db_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update(&self, aa: &AccountAssetModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE account_asset SET
                "QUANTITY" = $2, "UNCONFIRMED_QUANTITY" = $3, "HEIGHT" = $4, "LATEST" = $5
            WHERE "DB_ID" = $1
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
        let records = sqlx::query_as::<_, AccountAssetModel>(r#"SELECT * FROM account_asset WHERE "LATEST" = TRUE ORDER BY "DB_ID" LIMIT $1 OFFSET $2"#).bind(limit).bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM account_asset WHERE "LATEST" = TRUE"#)
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
        let record = sqlx::query_as::<_, AssetModel>(r#"SELECT * FROM asset WHERE "ID" = $1 AND "LATEST" = TRUE LIMIT 1"#).bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_owner(&self, owner_id: i64) -> RepositoryResult<Vec<AssetModel>> {
        let records = sqlx::query_as::<_, AssetModel>(r#"SELECT * FROM asset WHERE "ACCOUNT_ID" = $1 AND "LATEST" = TRUE ORDER BY "HEIGHT" DESC"#).bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<AssetModel>> {
        let records = sqlx::query_as::<_, AssetModel>(r#"SELECT * FROM asset WHERE "HEIGHT" = $1 AND "LATEST" = TRUE"#).bind(height)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_tradable(&self, limit: i64) -> RepositoryResult<Vec<AssetModel>> {
        let records = sqlx::query_as::<_, AssetModel>(r#"SELECT * FROM asset WHERE "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $1"#).bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn increase_quantity(&self, asset_id: i64, delta: i64) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            UPDATE asset
            SET "QUANTITY" = "QUANTITY" + $1, "LATEST" = TRUE
            WHERE "ID" = $2 AND "LATEST" = TRUE
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
            SET "QUANTITY" = "QUANTITY" - $1, "LATEST" = TRUE
            WHERE "ID" = $2 AND "LATEST" = TRUE AND "QUANTITY" >= $1
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
                "ID", "ACCOUNT_ID", "NAME", "DESCRIPTION", "QUANTITY", "DECIMALS",
                "HAS_CONTROL_PHASING", "INITIAL_QUANTITY", "HEIGHT", "LATEST"
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
        let record = sqlx::query_as::<_, AssetModel>(r#"SELECT * FROM asset WHERE "DB_ID" = $1"#).bind(db_id)
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
        let records = sqlx::query_as::<_, AssetModel>(r#"SELECT * FROM asset WHERE "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#).bind(limit).bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM asset WHERE "LATEST" = TRUE"#)
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
            r#"SELECT * FROM account_ledger WHERE "ACCOUNT_ID" = $1 ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            r#"SELECT * FROM account_ledger WHERE "BLOCK_ID" = $1 ORDER BY "HEIGHT" DESC"#
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
                "ACCOUNT_ID", "EVENT_TYPE", "EVENT_ID", "HOLDING_TYPE", "HOLDING_ID",
                "CHANGE", "BALANCE", "BLOCK_ID", "HEIGHT", timestamp
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
            r#"SELECT * FROM account_ledger WHERE "DB_ID" = $1"#
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
            r#"SELECT * FROM account_ledger ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM alias WHERE "ID" = $1 AND "LATEST" = TRUE LIMIT 1"#
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
            r#"SELECT * FROM alias WHERE "ALIAS_NAME_LOWER" = $1 AND "LATEST" = TRUE LIMIT 1"#
        )
        .bind(&name_lower)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_owner(&self, account_id: i64) -> RepositoryResult<Vec<AliasModel>> {
        let records = sqlx::query_as::<_, AliasModel>(
            r#"SELECT * FROM alias WHERE "ACCOUNT_ID" = $1 AND "LATEST" = TRUE ORDER BY "ALIAS_NAME_LOWER""#
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn update_owner(&self, alias_id: i64, new_owner_id: i64) -> RepositoryResult<()> {
        sqlx::query(
            r#"UPDATE alias SET "ACCOUNT_ID" = $1 WHERE "ID" = $2 AND "LATEST" = TRUE"#
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
            r#"UPDATE alias SET "ALIAS_URI" = $1 WHERE "ID" = $2 AND "LATEST" = TRUE"#
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
                "ID", "ACCOUNT_ID", "ALIAS_NAME", "ALIAS_NAME_LOWER", "ALIAS_URI",
                timestamp, "HEIGHT", "LATEST"
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
            r#"SELECT * FROM alias WHERE "DB_ID" = $1"#
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
                "ACCOUNT_ID" = $2, "ALIAS_NAME" = $3, "ALIAS_NAME_LOWER" = $4,
                "ALIAS_URI" = $5, timestamp = $6, "HEIGHT" = $7, "LATEST" = $8
            WHERE "DB_ID" = $1
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
            r#"SELECT * FROM alias WHERE "LATEST" = TRUE ORDER BY "ALIAS_NAME_LOWER" LIMIT $1 OFFSET $2"#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM alias WHERE "LATEST" = TRUE"#)
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
            r#"SELECT * FROM alias_offer WHERE "ID" = $1 AND "LATEST" = TRUE LIMIT 1"#
        )
        .bind(alias_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_buyer(&self, buyer_id: i64) -> RepositoryResult<Vec<AliasOfferModel>> {
        let records = sqlx::query_as::<_, AliasOfferModel>(
            r#"SELECT * FROM alias_offer WHERE "BUYER_ID" = $1 AND "LATEST" = TRUE"#
        )
        .bind(buyer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn update_price(&self, alias_id: i64, price: i64, buyer_id: Option<i64>) -> RepositoryResult<()> {
        sqlx::query(
            r#"UPDATE alias_offer SET "PRICE" = $1, "BUYER_ID" = $2 WHERE "ID" = $3 AND "LATEST" = TRUE"#
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
        sqlx::query(r#"DELETE FROM alias_offer WHERE "ID" = $1"#)
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
            INSERT INTO alias_offer ("ID", "PRICE", "BUYER_ID", "HEIGHT", "LATEST")
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
            r#"SELECT * FROM alias_offer WHERE "DB_ID" = $1"#
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
            UPDATE alias_offer SET "PRICE" = $2, "BUYER_ID" = $3, "HEIGHT" = $4, "LATEST" = $5
            WHERE "DB_ID" = $1
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
        sqlx::query(r#"DELETE FROM alias_offer WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM alias_offer WHERE "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM alias_offer WHERE "LATEST" = TRUE"#)
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
            r#"SELECT * FROM asset_transfer WHERE "ASSET_ID" = $1 ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            r#"SELECT * FROM asset_transfer WHERE "SENDER_ID" = $1 ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            r#"SELECT * FROM asset_transfer WHERE "RECIPIENT_ID" = $1 ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            INSERT INTO asset_transfer ("ASSET_ID", "SENDER_ID", "RECIPIENT_ID", "QUANTITY", timestamp, "HEIGHT")
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
            r#"SELECT * FROM asset_transfer WHERE "DB_ID" = $1"#
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
            r#"SELECT * FROM asset_transfer ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM ask_order WHERE "ID" = $1 AND "LATEST" = TRUE LIMIT 1"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_asset(&self, asset_id: i64, limit: i64) -> RepositoryResult<Vec<AskOrderModel>> {
        let records = sqlx::query_as::<_, AskOrderModel>(
            r#"SELECT * FROM ask_order WHERE "ASSET_ID" = $1 AND "LATEST" = TRUE ORDER BY "PRICE" ASC LIMIT $2"#
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
            r#"SELECT * FROM ask_order WHERE "ACCOUNT_ID" = $1 AND "LATEST" = TRUE ORDER BY "HEIGHT" DESC"#
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_best_by_asset(&self, asset_id: i64) -> RepositoryResult<Option<AskOrderModel>> {
        let record = sqlx::query_as::<_, AskOrderModel>(
            r#"SELECT * FROM ask_order WHERE "ASSET_ID" = $1 AND "LATEST" = TRUE ORDER BY "PRICE" ASC LIMIT 1"#
        )
        .bind(asset_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update_quantity(&self, order_id: i64, quantity: i64) -> RepositoryResult<()> {
        sqlx::query(
            r#"UPDATE ask_order SET "QUANTITY" = $1 WHERE "ID" = $2 AND "LATEST" = TRUE"#
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
            INSERT INTO ask_order ("ID", "ACCOUNT_ID", "ASSET_ID", "PRICE", "QUANTITY", "HEIGHT", "LATEST")
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
            r#"SELECT * FROM ask_order WHERE "DB_ID" = $1"#
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
            r#"SELECT * FROM ask_order WHERE "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM ask_order WHERE "LATEST" = TRUE"#)
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
            r#"SELECT * FROM bid_order WHERE "ID" = $1 AND "LATEST" = TRUE LIMIT 1"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_asset(&self, asset_id: i64, limit: i64) -> RepositoryResult<Vec<BidOrderModel>> {
        let records = sqlx::query_as::<_, BidOrderModel>(
            r#"SELECT * FROM bid_order WHERE "ASSET_ID" = $1 AND "LATEST" = TRUE ORDER BY "PRICE" DESC LIMIT $2"#
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
            r#"SELECT * FROM bid_order WHERE "ACCOUNT_ID" = $1 AND "LATEST" = TRUE ORDER BY "HEIGHT" DESC"#
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_best_by_asset(&self, asset_id: i64) -> RepositoryResult<Option<BidOrderModel>> {
        let record = sqlx::query_as::<_, BidOrderModel>(
            r#"SELECT * FROM bid_order WHERE "ASSET_ID" = $1 AND "LATEST" = TRUE ORDER BY "PRICE" DESC LIMIT 1"#
        )
        .bind(asset_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn update_quantity(&self, order_id: i64, quantity: i64) -> RepositoryResult<()> {
        sqlx::query(
            r#"UPDATE bid_order SET "QUANTITY" = $1 WHERE "ID" = $2 AND "LATEST" = TRUE"#
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
            INSERT INTO bid_order ("ID", "ACCOUNT_ID", "ASSET_ID", "PRICE", "QUANTITY", "HEIGHT", "LATEST")
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
            r#"SELECT * FROM bid_order WHERE "DB_ID" = $1"#
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
            r#"SELECT * FROM bid_order WHERE "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM bid_order WHERE "LATEST" = TRUE"#)
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
            r#"SELECT * FROM tagged_data WHERE "ID" = $1 AND "LATEST" = TRUE LIMIT 1"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_account(&self, account_id: i64, limit: i64) -> RepositoryResult<Vec<TaggedDataModel>> {
        let records = sqlx::query_as::<_, TaggedDataModel>(
            r#"SELECT * FROM tagged_data WHERE "ACCOUNT_ID" = $1 AND "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            r#"SELECT * FROM tagged_data WHERE type = $1 AND "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            r#"SELECT * FROM tagged_data WHERE ("TAGS" LIKE $1 OR "PARSED_TAGS" LIKE $1) AND "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $2"#
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
                "ID", "ACCOUNT_ID", "NAME", "DESCRIPTION", "TAGS", "PARSED_TAGS", type,
                "DATA", "IS_TEXT", "FILENAME", "CHANNEL", "BLOCK_TIMESTAMP",
                "TRANSACTION_TIMESTAMP", "HEIGHT", "LATEST"
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
        let record = sqlx::query_as::<_, TaggedDataModel>(r#"SELECT * FROM tagged_data WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM tagged_data WHERE "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM tagged_data WHERE "LATEST" = TRUE"#)
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
            r#"SELECT * FROM purchase WHERE "ID" = $1 AND "LATEST" = TRUE LIMIT 1"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_buyer(&self, buyer_id: i64, limit: i64) -> RepositoryResult<Vec<PurchaseModel>> {
        let records = sqlx::query_as::<_, PurchaseModel>(
            r#"SELECT * FROM purchase WHERE "BUYER_ID" = $1 AND "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            r#"SELECT * FROM purchase WHERE "SELLER_ID" = $1 AND "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            r#"SELECT * FROM purchase WHERE "GOODS_ID" = $1 AND "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $2"#
        )
        .bind(goods_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn update_pending(&self, purchase_id: i64, pending: bool) -> RepositoryResult<()> {
        sqlx::query(r#"UPDATE purchase SET "PENDING" = $1 WHERE "ID" = $2 AND "LATEST" = TRUE"#)
            .bind(pending)
            .bind(purchase_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn set_delivered(&self, purchase_id: i64, goods: &[u8], nonce: &[u8]) -> RepositoryResult<()> {
        sqlx::query(
            r#"UPDATE purchase SET goods = $1, "GOODS_NONCE" = $2, "PENDING" = FALSE WHERE "ID" = $3 AND "LATEST" = TRUE"#
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
            r#"UPDATE purchase SET "REFUND" = $1, "REFUND_NOTE" = $2, "REFUND_NONCE" = $3 WHERE "ID" = $4 AND "LATEST" = TRUE"#
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
                "ID", "BUYER_ID", "GOODS_ID", "SELLER_ID", "QUANTITY", "PRICE", "DEADLINE",
                "NOTE", "NONCE", timestamp, "PENDING", goods, "GOODS_NONCE", "GOODS_IS_TEXT",
                "REFUND_NOTE", "REFUND_NONCE", "HAS_FEEDBACK_NOTES", "HAS_PUBLIC_FEEDBACKS",
                "DISCOUNT", "REFUND", "HEIGHT", "LATEST"
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
        let record = sqlx::query_as::<_, PurchaseModel>(r#"SELECT * FROM purchase WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM purchase WHERE "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM purchase WHERE "LATEST" = TRUE"#)
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
            r#"SELECT * FROM poll WHERE "ID" = $1 LIMIT 1"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<PollModel>> {
        let records = sqlx::query_as::<_, PollModel>(
            r#"SELECT * FROM poll WHERE "ACCOUNT_ID" = $1 ORDER BY "HEIGHT" DESC"#
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_active(&self, height: i32, limit: i64) -> RepositoryResult<Vec<PollModel>> {
        let records = sqlx::query_as::<_, PollModel>(
            r#"SELECT * FROM poll WHERE "FINISH_HEIGHT" > $1 ORDER BY "FINISH_HEIGHT" ASC LIMIT $2"#
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
                "ID", "ACCOUNT_ID", "NAME", "DESCRIPTION", "OPTIONS", "MIN_NUM_OPTIONS", "MAX_NUM_OPTIONS",
                "MIN_RANGE_VALUE", "MAX_RANGE_VALUE", timestamp, "FINISH_HEIGHT", "VOTING_MODEL",
                "MIN_BALANCE", "MIN_BALANCE_MODEL", "HOLDING_ID", "HEIGHT"
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
        let record = sqlx::query_as::<_, PollModel>(r#"SELECT * FROM poll WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM poll ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM vote WHERE "POLL_ID" = $1 ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            r#"SELECT * FROM vote WHERE "VOTER_ID" = $1 ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            r#"SELECT * FROM vote WHERE "POLL_ID" = $1 AND "VOTER_ID" = $2 LIMIT 1"#
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
            INSERT INTO vote ("ID", "POLL_ID", "VOTER_ID", "VOTE_BYTES", "HEIGHT")
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
        let record = sqlx::query_as::<_, VoteModel>(r#"SELECT * FROM vote WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM vote ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM shuffling WHERE "ID" = $1 AND "LATEST" = TRUE LIMIT 1"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_by_issuer(&self, issuer_id: i64) -> RepositoryResult<Vec<ShufflingModel>> {
        let records = sqlx::query_as::<_, ShufflingModel>(
            r#"SELECT * FROM shuffling WHERE "ISSUER_ID" = $1 AND "LATEST" = TRUE ORDER BY "HEIGHT" DESC"#
        )
        .bind(issuer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_active(&self, limit: i64) -> RepositoryResult<Vec<ShufflingModel>> {
        let records = sqlx::query_as::<_, ShufflingModel>(
            r#"SELECT * FROM shuffling WHERE "LATEST" = TRUE AND "STAGE" < 5 ORDER BY "HEIGHT" DESC LIMIT $1"#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn update_stage(&self, shuffling_id: i64, stage: i32) -> RepositoryResult<()> {
        sqlx::query(r#"UPDATE shuffling SET "STAGE" = $1 WHERE "ID" = $2 AND "LATEST" = TRUE"#)
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
                "ID", "HOLDING_ID", "HOLDING_TYPE", "ISSUER_ID", "AMOUNT", "PARTICIPANT_COUNT",
                "BLOCKS_REMAINING", "STAGE", "ASSIGNEE_ACCOUNT_ID", "REGISTRANT_COUNT",
                "RECIPIENT_PUBLIC_KEYS", "HEIGHT", "LATEST"
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
        let record = sqlx::query_as::<_, ShufflingModel>(r#"SELECT * FROM shuffling WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM shuffling WHERE "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM shuffling WHERE "LATEST" = TRUE"#)
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
            r#"SELECT * FROM account_currency WHERE "ACCOUNT_ID" = $1 AND "LATEST" = TRUE"#
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_currency(&self, currency_id: i64) -> RepositoryResult<Vec<AccountCurrencyModel>> {
        let records = sqlx::query_as::<_, AccountCurrencyModel>(
            r#"SELECT * FROM account_currency WHERE "CURRENCY_ID" = $1 AND "LATEST" = TRUE"#
        )
        .bind(currency_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_account_and_currency(&self, account_id: i64, currency_id: i64) -> RepositoryResult<Option<AccountCurrencyModel>> {
        let record = sqlx::query_as::<_, AccountCurrencyModel>(
            r#"SELECT * FROM account_currency WHERE "ACCOUNT_ID" = $1 AND "CURRENCY_ID" = $2 AND "LATEST" = TRUE LIMIT 1"#
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
            r#"UPDATE account_currency SET "UNITS" = "UNITS" + $1, "LATEST" = TRUE WHERE "ACCOUNT_ID" = $2 AND "CURRENCY_ID" = $3 AND "LATEST" = TRUE"#
        )
        .bind(delta)
        .bind(account_id)
        .bind(currency_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        if result.rows_affected() == 0 && delta != 0 {
            let current_height: i32 = sqlx::query_scalar(r#"SELECT COALESCE(MAX("HEIGHT"), 0) FROM block"#)
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
            sqlx::query(
                r#"INSERT INTO account_currency ("ACCOUNT_ID", "CURRENCY_ID", "UNITS", "UNCONFIRMED_UNITS", "HEIGHT", "LATEST") VALUES ($1, $2, $3, 0, $4, TRUE)"#
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
            SET "UNCONFIRMED_UNITS" = "UNCONFIRMED_UNITS" + $1, "LATEST" = TRUE
            WHERE "ACCOUNT_ID" = $2 AND "CURRENCY_ID" = $3
            "#,
        )
        .bind(delta)
        .bind(account_id)
        .bind(currency_id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        if result.rows_affected() == 0 && delta != 0 {
            let current_height: i32 = sqlx::query_scalar(r#"SELECT COALESCE(MAX("HEIGHT"), 0) FROM block"#)
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
            sqlx::query(
                r#"INSERT INTO account_currency ("ACCOUNT_ID", "CURRENCY_ID", "UNITS", "UNCONFIRMED_UNITS", "HEIGHT", "LATEST") VALUES ($1, $2, 0, $3, $4, TRUE)"#
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
            INSERT INTO account_currency ("ACCOUNT_ID", "CURRENCY_ID", "UNITS", "UNCONFIRMED_UNITS", "HEIGHT", "LATEST")
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
        let record = sqlx::query_as::<_, AccountCurrencyModel>(r#"SELECT * FROM account_currency WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM account_currency WHERE "LATEST" = TRUE LIMIT $1 OFFSET $2"#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM account_currency WHERE "LATEST" = TRUE"#)
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
            r#"SELECT * FROM currency_transfer WHERE "CURRENCY_ID" = $1 ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            r#"SELECT * FROM currency_transfer WHERE "SENDER_ID" = $1 ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            r#"SELECT * FROM currency_transfer WHERE "RECIPIENT_ID" = $1 ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            INSERT INTO currency_transfer ("ID", "CURRENCY_ID", "SENDER_ID", "RECIPIENT_ID", "UNITS", timestamp, "HEIGHT")
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
        let record = sqlx::query_as::<_, CurrencyTransferModel>(r#"SELECT * FROM currency_transfer WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM currency_transfer ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM contract_reference WHERE "ACCOUNT_ID" = $1 ORDER BY "HEIGHT" DESC"#
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_contract_name(&self, name: &str) -> RepositoryResult<Option<ContractReferenceModel>> {
        let record = sqlx::query_as::<_, ContractReferenceModel>(
            r#"SELECT * FROM contract_reference WHERE "CONTRACT_NAME" = $1 LIMIT 1"#
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn delete_by_account_and_name(&self, account_id: i64, name: &str) -> RepositoryResult<()> {
        sqlx::query(r#"DELETE FROM contract_reference WHERE "ACCOUNT_ID" = $1 AND "CONTRACT_NAME" = $2"#)
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
            INSERT INTO contract_reference ("ID", "ACCOUNT_ID", "CONTRACT_NAME", "CONTRACT_PARAMS", "CONTRACT_TRANSACTION_CHAIN_ID", "CONTRACT_TRANSACTION_FULL_HASH", "HEIGHT", "LATEST")
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
        let record = sqlx::query_as::<_, ContractReferenceModel>(r#"SELECT * FROM contract_reference WHERE "DB_ID" = $1"#)
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
        sqlx::query(r#"DELETE FROM contract_reference WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM contract_reference ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM asset_property WHERE "ASSET_ID" = $1 AND "LATEST" = TRUE ORDER BY "HEIGHT" DESC"#
        )
        .bind(asset_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_asset_and_account(&self, asset_id: i64, setter_id: i64) -> RepositoryResult<Vec<AssetPropertyModel>> {
        let records = sqlx::query_as::<_, AssetPropertyModel>(
            r#"SELECT * FROM asset_property WHERE "ASSET_ID" = $1 AND "SETTER_ID" = $2 AND "LATEST" = TRUE ORDER BY "HEIGHT" DESC"#
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
            r#"SELECT * FROM asset_property WHERE "ASSET_ID" = $1 AND "SETTER_ID" = $2 AND "PROPERTY" = $3 AND "LATEST" = TRUE LIMIT 1"#
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
        sqlx::query(r#"DELETE FROM asset_property WHERE "ASSET_ID" = $1 AND "SETTER_ID" = $2 AND "PROPERTY" = $3"#)
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
            INSERT INTO asset_property ("ID", "ASSET_ID", "SETTER_ID", "PROPERTY", "VALUE", "HEIGHT", "LATEST")
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
        let record = sqlx::query_as::<_, AssetPropertyModel>(r#"SELECT * FROM asset_property WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM asset_property WHERE "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM asset_property WHERE "LATEST" = TRUE"#)
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
            r#"SELECT * FROM asset_history WHERE "ASSET_ID" = $1 ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            r#"SELECT * FROM asset_history WHERE "ASSET_ID" = $1 AND "ACCOUNT_ID" = $2 ORDER BY "HEIGHT" DESC LIMIT $3"#
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
            INSERT INTO asset_history ("ID", "FULL_HASH", "ASSET_ID", "ACCOUNT_ID", "QUANTITY", timestamp, "CHAIN_ID", "HEIGHT")
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
        let record = sqlx::query_as::<_, AssetHistoryModel>(r#"SELECT * FROM asset_history WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM asset_history ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM tagged_data_tag WHERE "TAG" = $1 ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            r#"SELECT * FROM tagged_data_tag WHERE "ID" = $1 ORDER BY "HEIGHT" DESC"#
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
            INSERT INTO tagged_data_tag ("ID", "TAG", "HEIGHT", "LATEST")
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
        let record = sqlx::query_as::<_, TaggedDataTagModel>(r#"SELECT * FROM tagged_data_tag WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM tagged_data_tag WHERE "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM tagged_data_tag WHERE "LATEST" = TRUE"#)
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
            r#"SELECT * FROM tagged_timestamp WHERE "ACCOUNT_ID" = $1 ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            r#"SELECT * FROM tagged_timestamp WHERE "TAG" = $1 ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            r#"SELECT * FROM tagged_timestamp WHERE "ACCOUNT_ID" = $1 AND "TAG" = $2 LIMIT 1"#
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
            INSERT INTO tagged_timestamp ("ID", "ACCOUNT_ID", "TAG", timestamp, "HEIGHT", "LATEST")
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
        let record = sqlx::query_as::<_, TaggedTimestampModel>(r#"SELECT * FROM tagged_timestamp WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM tagged_timestamp WHERE "LATEST" = TRUE ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM tagged_timestamp WHERE "LATEST" = TRUE"#)
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
            INSERT INTO account_guaranteed_balance ("ACCOUNT_ID", "ADDITIONS", "HEIGHT")
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
            r#"SELECT * FROM account_guaranteed_balance WHERE "DB_ID" = $1"#
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
            SET "ACCOUNT_ID" = $1, "ADDITIONS" = $2, "HEIGHT" = $3
            WHERE "DB_ID" = $4
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
        sqlx::query(r#"DELETE FROM account_guaranteed_balance WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM account_guaranteed_balance ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM account_guaranteed_balance WHERE "ACCOUNT_ID" = $1 AND "HEIGHT" = $2"#
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
            INSERT INTO exchange_request ("ID", "ACCOUNT_ID", "CURRENCY_ID", "UNITS", "RATE", "IS_BUY", timestamp, "HEIGHT")
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
            r#"SELECT * FROM exchange_request WHERE "DB_ID" = $1"#
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
                "ACCOUNT_ID" = $1, "CURRENCY_ID" = $2, "UNITS" = $3, "RATE" = $4,
                "IS_BUY" = $5, timestamp = $6, "HEIGHT" = $7
            WHERE "DB_ID" = $8
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
        sqlx::query(r#"DELETE FROM exchange_request WHERE "DB_ID" = $1"#)
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<ExchangeRequestModel>> {
        let records = sqlx::query_as::<_, ExchangeRequestModel>(
            r#"SELECT * FROM exchange_request ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            INSERT INTO currency_mint ("CURRENCY_ID", "ACCOUNT_ID", "COUNTER", "HEIGHT", "LATEST")
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
            r#"SELECT * FROM currency_mint WHERE "DB_ID" = $1"#
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
                "CURRENCY_ID" = $1, "ACCOUNT_ID" = $2, "COUNTER" = $3,
                "HEIGHT" = $4, "LATEST" = $5
            WHERE "DB_ID" = $6
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
        sqlx::query(r#"DELETE FROM currency_mint WHERE "DB_ID" = $1"#)
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<CurrencyMintModel>> {
        let records = sqlx::query_as::<_, CurrencyMintModel>(
            r#"SELECT * FROM currency_mint ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            INSERT INTO asset_delete ("ASSET_ID", "ACCOUNT_ID", "QUANTITY", "HEIGHT")
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
            r#"SELECT * FROM asset_delete WHERE "DB_ID" = $1"#
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
            r#"SELECT * FROM asset_delete ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM asset_delete WHERE "ASSET_ID" = $1 ORDER BY "HEIGHT" DESC"#
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
            INSERT INTO phasing_poll_result ("ID", "RESULT", "APPROVED", "HEIGHT")
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
            r#"SELECT * FROM phasing_poll_result WHERE "DB_ID" = $1"#
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
                "ID" = $1, "RESULT" = $2, "APPROVED" = $3, "HEIGHT" = $4
            WHERE "DB_ID" = $5
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
        sqlx::query(r#"DELETE FROM phasing_poll_result WHERE "DB_ID" = $1"#)
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PhasingPollResultModel>> {
        let records = sqlx::query_as::<_, PhasingPollResultModel>(
            r#"SELECT * FROM phasing_poll_result ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM phasing_poll_result WHERE "ID" = $1"#
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
            INSERT INTO coin_order_fxt ("ID", "ACCOUNT_ID", "CHAIN_ID", "EXCHANGE_ID", "FULL_HASH", "AMOUNT", "QUANTITY", "BID_PRICE", "ASK_PRICE", "CREATION_HEIGHT", "HEIGHT", "TRANSACTION_HEIGHT", "TRANSACTION_INDEX", "LATEST")
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
            r#"SELECT * FROM coin_order_fxt WHERE "DB_ID" = $1"#
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
                "ID" = $1, "ACCOUNT_ID" = $2, "CHAIN_ID" = $3, "EXCHANGE_ID" = $4, "FULL_HASH" = $5,
                "AMOUNT" = $6, "QUANTITY" = $7, "BID_PRICE" = $8, "ASK_PRICE" = $9,
                "CREATION_HEIGHT" = $10, "HEIGHT" = $11, "TRANSACTION_HEIGHT" = $12, "TRANSACTION_INDEX" = $13, "LATEST" = $14
            WHERE "DB_ID" = $15
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
        sqlx::query(r#"DELETE FROM coin_order_fxt WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM coin_order_fxt ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM coin_order_fxt WHERE "EXCHANGE_ID" = $1 AND "LATEST" = true"#
        )
        .bind(exchange_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<CoinOrderFxtModel>> {
        let records = sqlx::query_as::<_, CoinOrderFxtModel>(
            r#"SELECT * FROM coin_order_fxt WHERE "ACCOUNT_ID" = $1 AND "LATEST" = true"#
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
            INSERT INTO coin_trade_fxt ("CHAIN_ID", "EXCHANGE_ID", "ACCOUNT_ID", "BLOCK_ID", "HEIGHT", timestamp, "EXCHANGE_QUANTITY", "EXCHANGE_PRICE", "ORDER_ID")
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
            r#"SELECT * FROM coin_trade_fxt WHERE "DB_ID" = $1"#
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
                "CHAIN_ID" = $1, "EXCHANGE_ID" = $2, "ACCOUNT_ID" = $3, "BLOCK_ID" = $4,
                "HEIGHT" = $5, timestamp = $6, "EXCHANGE_QUANTITY" = $7, "EXCHANGE_PRICE" = $8, "ORDER_ID" = $9
            WHERE "DB_ID" = $10
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
        sqlx::query(r#"DELETE FROM coin_trade_fxt WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM coin_trade_fxt ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM coin_trade_fxt WHERE "EXCHANGE_ID" = $1 ORDER BY "HEIGHT" DESC LIMIT $2"#
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
            r#"SELECT * FROM coin_trade_fxt WHERE "ORDER_ID" = $1"#
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
            INSERT INTO phasing_poll_voter ("TRANSACTION_ID", "VOTER_ID", "HEIGHT")
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
            r#"SELECT * FROM phasing_poll_voter WHERE "DB_ID" = $1"#
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
            r#"SELECT * FROM phasing_poll_voter ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM phasing_poll_voter WHERE "TRANSACTION_ID" = $1"#
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
            INSERT INTO phasing_poll ("ID", "ACCOUNT_ID", "WHITELIST_SIZE", "FINISH_HEIGHT", "VOTING_MODEL", "QUORUM",
                "MIN_BALANCE", "HOLDING_ID", "MIN_BALANCE_MODEL", "HASHED_SECRET", "ALGORITHM", "HEIGHT")
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
            r#"SELECT * FROM phasing_poll WHERE "DB_ID" = $1"#
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
                "ACCOUNT_ID" = $1, "WHITELIST_SIZE" = $2, "FINISH_HEIGHT" = $3, "VOTING_MODEL" = $4, "QUORUM" = $5,
                "MIN_BALANCE" = $6, "HOLDING_ID" = $7, "MIN_BALANCE_MODEL" = $8,
                "HASHED_SECRET" = $9, "ALGORITHM" = $10, "HEIGHT" = $11
            WHERE "DB_ID" = $12
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
        sqlx::query(r#"DELETE FROM phasing_poll WHERE "DB_ID" = $1"#)
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PhasingPollModel>> {
        let records = sqlx::query_as::<_, PhasingPollModel>(
            r#"SELECT * FROM phasing_poll ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM phasing_poll WHERE "ID" = $1"#
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
            INSERT INTO phasing_poll_linked_transaction ("TRANSACTION_ID", "LINKED_FULL_HASH", "LINKED_TRANSACTION_ID", "HEIGHT")
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
            r#"SELECT * FROM phasing_poll_linked_transaction WHERE "DB_ID" = $1"#
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
            r#"SELECT * FROM phasing_poll_linked_transaction ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM phasing_poll_linked_transaction WHERE "TRANSACTION_ID" = $1"#
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
            INSERT INTO asset_dividend ("ID", "ASSET_ID", "AMOUNT", "DIVIDEND_HEIGHT", "TOTAL_DIVIDEND", "NUM_ACCOUNTS", timestamp, "HEIGHT")
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
            r#"SELECT * FROM asset_dividend WHERE "DB_ID" = $1"#
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
            r#"SELECT * FROM asset_dividend ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM asset_dividend WHERE "ASSET_ID" = $1 ORDER BY "HEIGHT" DESC"#
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
            INSERT INTO phasing_vote ("VOTE_ID", "TRANSACTION_ID", "VOTER_ID", "HEIGHT")
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
            r#"SELECT * FROM phasing_vote WHERE "DB_ID" = $1"#
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
            r#"SELECT * FROM phasing_vote ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM phasing_vote WHERE "TRANSACTION_ID" = $1"#
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
            INSERT INTO poll_result ("POLL_ID", "RESULT", "WEIGHT", "HEIGHT")
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
            r#"SELECT * FROM poll_result WHERE "DB_ID" = $1"#
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
                "POLL_ID" = $1, "RESULT" = $2, "HEIGHT" = $3
            WHERE "DB_ID" = $4
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
        sqlx::query(r#"DELETE FROM poll_result WHERE "DB_ID" = $1"#)
            .bind(db_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PollResultModel>> {
        let records = sqlx::query_as::<_, PollResultModel>(
            r#"SELECT * FROM poll_result ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM poll_result WHERE "POLL_ID" = $1"#
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn upsert(&self, model: &PollResultModel) -> RepositoryResult<()> {
        let existing = sqlx::query_as::<_, PollResultModel>(
            r#"SELECT * FROM poll_result WHERE "POLL_ID" = $1 AND "RESULT" = $2"#
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
            INSERT INTO phasing_poll_hashed_secret ("HASHED_SECRET", "HASHED_SECRET_ID", "ALGORITHM", "TRANSACTION_FULL_HASH", "TRANSACTION_ID", "CHAIN_ID", "FINISH_HEIGHT", "HEIGHT")
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
            r#"SELECT * FROM phasing_poll_hashed_secret WHERE "DB_ID" = $1"#
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
                "HASHED_SECRET" = $1, "HASHED_SECRET_ID" = $2, "ALGORITHM" = $3,
                "TRANSACTION_FULL_HASH" = $4, "TRANSACTION_ID" = $5,
                "CHAIN_ID" = $6, "FINISH_HEIGHT" = $7, "HEIGHT" = $8
            WHERE "DB_ID" = $9
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
        sqlx::query(r#"DELETE FROM phasing_poll_hashed_secret WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM phasing_poll_hashed_secret ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
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
            r#"SELECT * FROM phasing_poll_hashed_secret WHERE "HASHED_SECRET_ID" = $1"#
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
        sqlx::query(r#"INSERT INTO hub ("ACCOUNT_ID", "URIS", "MIN_FEE_PER_BYTE", "HEIGHT") VALUES ($1, $2, $3, $4)"#)
            .bind(m.account_id).bind(&m.uris).bind(m.min_fee_per_byte).bind(m.height)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn find_by_id(&self, id: i64) -> RepositoryResult<Option<HubModel>> {
        sqlx::query_as::<_, HubModel>(r#"SELECT * FROM hub WHERE "DB_ID" = $1"#).bind(id)
            .fetch_optional(&self.pool).await.map_err(RepositoryError::DbError)
    }
    async fn update(&self, m: &HubModel) -> RepositoryResult<()> {
        sqlx::query(r#"UPDATE hub SET "ACCOUNT_ID"=$1, "URIS"=$2, "MIN_FEE_PER_BYTE"=$3, "HEIGHT"=$4 WHERE "DB_ID"=$5"#)
            .bind(m.account_id).bind(&m.uris).bind(m.min_fee_per_byte).bind(m.height).bind(m.db_id)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        sqlx::query(r#"DELETE FROM hub WHERE "DB_ID"=$1"#).bind(id).execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<HubModel>> {
        let lim = limit.unwrap_or(100);
        let off = offset.unwrap_or(0);
        sqlx::query_as::<_, HubModel>(r#"SELECT * FROM hub ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#)
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
        sqlx::query_as::<_, HubModel>(r#"SELECT * FROM hub WHERE "ACCOUNT_ID" = $1 LIMIT 1"#).bind(account_id)
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
        sqlx::query(r#"INSERT INTO currency_founder ("CURRENCY_ID", "ACCOUNT_ID", "AMOUNT", "HEIGHT") VALUES ($1, $2, $3, $4)"#)
            .bind(m.currency_id).bind(m.account_id).bind(m.amount).bind(m.height)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn find_by_id(&self, id: i64) -> RepositoryResult<Option<CurrencyFounderModel>> {
        sqlx::query_as::<_, CurrencyFounderModel>(r#"SELECT * FROM currency_founder WHERE "DB_ID" = $1"#).bind(id)
            .fetch_optional(&self.pool).await.map_err(RepositoryError::DbError)
    }
    async fn update(&self, m: &CurrencyFounderModel) -> RepositoryResult<()> {
        sqlx::query(r#"UPDATE currency_founder SET "CURRENCY_ID"=$1, "ACCOUNT_ID"=$2, "AMOUNT"=$3, "HEIGHT"=$4 WHERE "DB_ID"=$5"#)
            .bind(m.currency_id).bind(m.account_id).bind(m.amount).bind(m.height).bind(m.db_id)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        sqlx::query(r#"DELETE FROM currency_founder WHERE "DB_ID"=$1"#).bind(id).execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<CurrencyFounderModel>> {
        let lim = limit.unwrap_or(100);
        let off = offset.unwrap_or(0);
        sqlx::query_as::<_, CurrencyFounderModel>(r#"SELECT * FROM currency_founder ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#)
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
        sqlx::query_as::<_, CurrencyFounderModel>(r#"SELECT * FROM currency_founder WHERE "CURRENCY_ID" = $1"#)
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
            r#"INSERT INTO prunable_message ("ID", "SENDER_ID", "RECIPIENT_ID", "MESSAGE", "MESSAGE_IS_TEXT", "IS_COMPRESSED", "ENCRYPTED_MESSAGE", "ENCRYPTED_IS_TEXT", "BLOCK_TIMESTAMP", "TRANSACTION_TIMESTAMP", "HEIGHT")
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"#)
            .bind(m.id).bind(m.sender_id).bind(m.recipient_id).bind(&m.message)
            .bind(m.message_is_text).bind(m.is_compressed).bind(&m.encrypted_message)
            .bind(m.encrypted_is_text).bind(m.block_timestamp).bind(m.transaction_timestamp).bind(m.height)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn find_by_id(&self, id: i64) -> RepositoryResult<Option<PrunableMessageModel>> {
        sqlx::query_as::<_, PrunableMessageModel>(r#"SELECT * FROM prunable_message WHERE "DB_ID" = $1"#).bind(id)
            .fetch_optional(&self.pool).await.map_err(RepositoryError::DbError)
    }
    async fn update(&self, m: &PrunableMessageModel) -> RepositoryResult<()> {
        sqlx::query(
            r#"UPDATE prunable_message SET "ID"=$1, "SENDER_ID"=$2, "RECIPIENT_ID"=$3, "MESSAGE"=$4, "MESSAGE_IS_TEXT"=$5,
               "IS_COMPRESSED"=$6, "ENCRYPTED_MESSAGE"=$7, "ENCRYPTED_IS_TEXT"=$8, "BLOCK_TIMESTAMP"=$9, "TRANSACTION_TIMESTAMP"=$10, "HEIGHT"=$11
            WHERE "DB_ID"=$12"#)
            .bind(m.id).bind(m.sender_id).bind(m.recipient_id).bind(&m.message)
            .bind(m.message_is_text).bind(m.is_compressed).bind(&m.encrypted_message)
            .bind(m.encrypted_is_text).bind(m.block_timestamp).bind(m.transaction_timestamp).bind(m.height).bind(m.db_id)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        sqlx::query(r#"DELETE FROM prunable_message WHERE "DB_ID"=$1"#).bind(id).execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PrunableMessageModel>> {
        let lim = limit.unwrap_or(100);
        let off = offset.unwrap_or(0);
        sqlx::query_as::<_, PrunableMessageModel>(r#"SELECT * FROM prunable_message ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#)
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
        sqlx::query(r#"INSERT INTO purchase_feedback (purchase_id, "FEEDBACK_DATA", "FEEDBACK_NONCE", "HEIGHT") VALUES ($1, $2, $3, $4)"#)
            .bind(m.id).bind(&m.feedback_data).bind(&m.feedback_nonce).bind(m.height)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn find_by_id(&self, id: i64) -> RepositoryResult<Option<PurchaseFeedbackModel>> {
        sqlx::query_as::<_, PurchaseFeedbackModel>(r#"SELECT * FROM purchase_feedback WHERE "DB_ID" = $1"#).bind(id)
            .fetch_optional(&self.pool).await.map_err(RepositoryError::DbError)
    }
    async fn update(&self, m: &PurchaseFeedbackModel) -> RepositoryResult<()> {
        sqlx::query(r#"UPDATE purchase_feedback SET "ID"=$1, "FEEDBACK_DATA"=$2, "FEEDBACK_NONCE"=$3, "HEIGHT"=$4 WHERE "DB_ID"=$5"#)
            .bind(m.id).bind(&m.feedback_data).bind(&m.feedback_nonce).bind(m.height).bind(m.db_id)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        sqlx::query(r#"DELETE FROM purchase_feedback WHERE "DB_ID"=$1"#).bind(id).execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }
    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<PurchaseFeedbackModel>> {
        let lim = limit.unwrap_or(100);
        let off = offset.unwrap_or(0);
        sqlx::query_as::<_, PurchaseFeedbackModel>(r#"SELECT * FROM purchase_feedback ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#)
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
        sqlx::query(r#"INSERT INTO referenced_transaction ("TRANSACTION_ID", "REFERENCED_TRANSACTION_ID") VALUES ($1, $2)"#)
            .bind(m.transaction_id).bind(m.referenced_transaction_id)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_by_id(&self, id: i64) -> RepositoryResult<Option<ReferencedTransactionModel>> {
        sqlx::query_as::<_, ReferencedTransactionModel>(r#"SELECT * FROM referenced_transaction WHERE "DB_ID" = $1"#).bind(id)
            .fetch_optional(&self.pool).await.map_err(RepositoryError::DbError)
    }

    async fn update(&self, m: &ReferencedTransactionModel) -> RepositoryResult<()> {
        sqlx::query(r#"UPDATE referenced_transaction SET "TRANSACTION_ID"=$1, "REFERENCED_TRANSACTION_ID"=$2 WHERE "DB_ID"=$3"#)
            .bind(m.transaction_id).bind(m.referenced_transaction_id).bind(m.db_id)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        sqlx::query(r#"DELETE FROM referenced_transaction WHERE "DB_ID"=$1"#).bind(id)
            .execute(&self.pool).await.map_err(RepositoryError::DbError)?;
        Ok(())
    }

    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<ReferencedTransactionModel>> {
        let lim = limit.unwrap_or(100);
        let off = offset.unwrap_or(0);
        sqlx::query_as::<_, ReferencedTransactionModel>(r#"SELECT * FROM referenced_transaction ORDER BY "DB_ID" DESC LIMIT $1 OFFSET $2"#)
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
        sqlx::query_as::<_, ReferencedTransactionModel>(r#"SELECT * FROM referenced_transaction WHERE "TRANSACTION_ID" = $1"#)
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
            INSERT INTO tagged_data_extend ("ID", "EXTEND_ID", "HEIGHT", "LATEST")
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
        let record = sqlx::query_as::<_, TaggedDataExtendModel>(r#"SELECT * FROM tagged_data_extend WHERE "DB_ID" = $1"#)
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
            r#"SELECT * FROM tagged_data_extend WHERE "LATEST" = true ORDER BY "HEIGHT" DESC LIMIT $1 OFFSET $2"#
        )
        .bind(lim)
        .bind(off)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM tagged_data_extend WHERE "LATEST" = true"#)
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
            r#"SELECT * FROM tagged_data_extend WHERE "EXTEND_ID" = $1 AND "LATEST" = true ORDER BY "HEIGHT" DESC"#
        )
        .bind(extend_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(records)
    }
}
