//! Blockchain state processor for blocks and transactions.

#[cfg(feature = "database")]
use super::*;
#[cfg(feature = "database")]
use sqlx::PgPool;
#[cfg(feature = "database")]
use tracing::{debug, error, info, warn};
#[cfg(feature = "database")]
use async_trait::async_trait;

#[cfg(feature = "database")]
pub struct BlockchainProcessor {
    pool: PgPool,
}

#[cfg(feature = "database")]
impl BlockchainProcessor {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn process_block(&self, block: &Block) -> Result<Height> {
        self.validate_basic(block)?;

        let prev_height = block.height.checked_sub(1).ok_or_else(||
            BlockchainError::InvalidTransaction("block height underflow".to_string())
        )?;

        if prev_height > 0 {
            let prev_exists = self.block_exists(prev_height).await?;
            if !prev_exists {
                return Err(BlockchainError::InvalidBlock(format!(
                    "previous block at height {} not found", prev_height
                )));
            }
        }

        let computed_payload_hash = self.compute_payload_hash(&block.transactions)?;
        if computed_payload_hash != block.payload_hash {
            return Err(BlockchainError::InvalidBlock(
                "payload hash mismatch".to_string()
            ));
        }

        self.insert_block(block).await?;

        info!(target: "blockchain", "Accepted block: height={}, id={}, generator={}, txs={}",
              block.height, block.compute_hash()?, block.generator_id, block.transactions.len());

        Ok(block.height)
    }

    fn validate_basic(&self, block: &Block) -> Result<()> {
        if block.version != BLOCK_VERSION {
            return Err(BlockchainError::InvalidBlock(
                format!("unsupported block version: {}", block.version)
            ));
        }
        if block.height == 0 {
            return Err(BlockchainError::InvalidBlock("height cannot be zero".to_string()));
        }
        Ok(())
    }

    async fn block_exists(&self, height: Height) -> Result<bool> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM block WHERE height = $1")
            .bind(height as i32)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| BlockchainError::Database(e.to_string()))?;
        Ok(count.0 > 0)
    }

    fn compute_payload_hash(&self, txs: &[Transaction]) -> Result<Hash256> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        for tx in txs {
            let bytes = serde_json::to_vec(tx)
                .map_err(|e| BlockchainError::Serialization(e.into()))?;
            hasher.update(bytes);
        }
        let hash = hasher.finalize();
        let arr: [u8; 32] = hash.try_into().unwrap();
        Ok(Hash256(arr))
    }

    async fn insert_block(&self, block: &Block) -> Result<()> {
        let model = BlockModel {
            db_id: None,
            id: block.height as i64,
            version: block.version as i32,
            timestamp: block.timestamp as i32,
            previous_block_id: None,
            total_amount: block.total_amount as i64,
            total_fee: block.total_fee as i64,
            payload_length: block.payload_length as i32,
            previous_block_hash: Some(block.previous_block_hash.0.to_vec()),
            cumulative_difficulty: block.cumulative_difficulty.clone(),
            base_target: block.base_target as i64,
            next_block_id: None,
            height: block.height as i32,
            generation_signature: block.generation_signature.0.to_vec(),
            block_signature: block.block_signature.0.to_vec(),
            payload_hash: block.payload_hash.0.to_vec(),
            generator_id: block.generator_id as i64,
        };

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
            model.id,
            model.version,
            model.timestamp,
            model.previous_block_id,
            model.total_amount,
            model.total_fee,
            model.payload_length,
            model.previous_block_hash.as_deref(),
            model.cumulative_difficulty.as_slice(),
            model.base_target,
            model.next_block_id,
            model.height,
            model.generation_signature.as_slice(),
            model.block_signature.as_slice(),
            model.payload_hash.as_slice(),
            model.generator_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| BlockchainError::Database(e.to_string()))?;

        Ok(())
    }
}

#[cfg(feature = "database")]
pub struct BlockchainVerifier {
    processor: BlockchainProcessor,
}

#[cfg(feature = "database")]
impl BlockchainVerifier {
    pub fn new(pool: PgPool) -> Self {
        Self {
            processor: BlockchainProcessor::new(pool),
        }
    }
}

#[cfg(all(feature = "database", feature = "p2p"))]
#[async_trait]
impl p2p::handlers::BlockVerifier for BlockchainVerifier {
    async fn verify_and_process(&self, block: Block) -> Result<()> {
        self.processor.process_block(&block).await?;
        Ok(())
    }
}
