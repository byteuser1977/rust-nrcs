//! Blockchain verifier implementation
//!
//! Implements BlockVerifier trait using repositories

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, debug, warn};

use blockchain_types::prelude::*;
use blockchain_types::block::{Block, PreviousBlockData, calculate_base_target_and_cumulative_difficulty, INITIAL_BASE_TARGET, biguint_to_signed_bytes_be, signed_bytes_be_to_biguint};
use blockchain_types::transaction::Transaction;

use crate::handlers::BlockVerifier;
use orm::{BlockRepository, TransactionRepository, BlockModel, TransactionModel};

pub struct BlockchainVerifier {
    block_repo: Arc<dyn BlockRepository>,
    tx_repo: Arc<dyn TransactionRepository>,
}

impl BlockchainVerifier {
    pub fn new(
        block_repo: Arc<dyn BlockRepository>,
        tx_repo: Arc<dyn TransactionRepository>,
    ) -> Self {
        Self { block_repo, tx_repo }
    }

    fn validate_basic(&self, block: &Block) -> Result<()> {
        // 允许版本 3 的区块（正常区块）和版本 -1 的区块（创世区块）
        if block.version != BLOCK_VERSION && block.version != -1 {
            return Err(BlockchainError::InvalidBlock(
                format!("unsupported block version: {}", block.version)
            ));
        }
        Ok(())
    }

    fn compute_payload_hash(&self, txs: &[Transaction]) -> Result<Hash256> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        for tx in txs {
            hasher.update(tx.full_hash.0);
        }
        let hash = hasher.finalize();
        let arr: [u8; 32] = hash.into();
        Ok(Hash256(arr))
    }

    #[allow(dead_code)]
    async fn block_exists(&self, height: u32) -> Result<bool> {
        match self.block_repo.find_by_height(height as i32).await {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(BlockchainError::Database(e.to_string())),
        }
    }

    async fn insert_block(&self, block: &Block) -> Result<()> {
        if let Ok(Some(_)) = self.block_repo.find_by_height(block.height as i32).await {
            debug!("Block at height {} already exists, skipping", block.height);
            return Ok(());
        }

        let block_model = BlockModel::from_domain(block)?;
        
        self.block_repo.insert(&block_model).await
            .map_err(|e| BlockchainError::Database(e.to_string()))?;

        let block_id = block.get_id() as i64;
        
        if let Some(prev_id) = block.previous_block_id.filter(|&id| id != 0) {
            if let Err(e) = self.block_repo.update_next_block_id(prev_id as i64, block_id).await {
                warn!("Failed to update next_block_id for block {}: {}", prev_id, e);
            }
        }
        
        for (idx, tx) in block.transactions.iter().enumerate() {
            let mut tx_model = TransactionModel::from_domain(tx)?;
            tx_model.height = block.height as i32;
            tx_model.block_id = block_id;
            tx_model.transaction_index = idx as i16;
            
            if let Err(e) = self.tx_repo.insert(&tx_model).await {
                warn!("Failed to insert transaction {}: {}", tx.id, e);
            }
        }

        Ok(())
    }
    
    fn model_to_previous_block_data(model: &BlockModel) -> PreviousBlockData {
        PreviousBlockData {
            base_target: model.base_target as u64,
            cumulative_difficulty: model.cumulative_difficulty.clone(),
            timestamp: model.timestamp as u32,
            height: model.height,
            id: model.id as u64,
        }
    }
}

#[async_trait]
impl BlockVerifier for BlockchainVerifier {
    async fn verify_and_process(&self, mut block: Block) -> anyhow::Result<()> {
        let block_height = block.height;
        let block_id = block.get_id();
        
        if block_height <= 2 {
            let serialized = block.serialize_for_id();
            info!("Block {} serialized bytes (first 40): {:02x?}", block_height, &serialized[..40.min(serialized.len())]);
            info!("Block {} gen_pub_key: {:?}", block_height, block.generator_public_key.as_ref().map(hex::encode));
            info!("Block {} prev_block_hash: {:02x?}", block_height, block.previous_block_hash.0.iter().take(8).collect::<Vec<_>>());
            info!("Block {} generation_signature: {:02x?}", block_height, block.generation_signature.iter().take(8).collect::<Vec<_>>());
            info!("Block {} block_signature: {:02x?}", block_height, block.block_signature.0.iter().take(8).collect::<Vec<_>>());
        }

        self.validate_basic(&block)?;

        if let Ok(Some(_)) = self.block_repo.find_by_height(block_height as i32).await {
            debug!("Block at height {} already exists, skipping", block_height);
            return Ok(());
        }

        let computed_payload_hash = self.compute_payload_hash(&block.transactions)?;
        if computed_payload_hash != block.payload_hash {
            debug!("Payload hash mismatch: computed {:?} != block {:?}", 
                   computed_payload_hash, block.payload_hash);
        }

        if block_height > 0 {
            let prev_height = (block_height - 1) as i32;
            match self.block_repo.find_by_height(prev_height).await {
                Ok(Some(prev_model)) => {
                    let prev_data = Self::model_to_previous_block_data(&prev_model);
                    
                    let block_hm2 = if prev_data.height >= 2 && prev_data.height % 2 == 0 {
                        match self.block_repo.find_by_height(prev_data.height - 2).await {
                            Ok(Some(m)) => Some(Self::model_to_previous_block_data(&m)),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    
                    let (base_target, cumulative_difficulty) = calculate_base_target_and_cumulative_difficulty(
                        block.timestamp,
                        block_height as i32,
                        &prev_data,
                        block_hm2.as_ref(),
                    );
                    
                    info!("Calculated base_target={} cumulative_difficulty={:?} for block height={}", 
                          base_target, cumulative_difficulty, block_height);
                    
                    block.base_target = base_target;
                    block.cumulative_difficulty = cumulative_difficulty;
                }
                Ok(None) => {
                    warn!("Previous block at height {} not found, using default base_target", prev_height);
                    block.base_target = INITIAL_BASE_TARGET;
                    let two64 = num_bigint::BigUint::from(18446744073709551616u128);
                    let diff_add = two64 / num_bigint::BigUint::from(INITIAL_BASE_TARGET);
                    block.cumulative_difficulty = biguint_to_signed_bytes_be(diff_add);
                }
                Err(e) => {
                    warn!("Error finding previous block at height {}: {}", prev_height, e);
                    block.base_target = INITIAL_BASE_TARGET;
                    let two64 = num_bigint::BigUint::from(18446744073709551616u128);
                    let diff_add = two64 / num_bigint::BigUint::from(INITIAL_BASE_TARGET);
                    block.cumulative_difficulty = biguint_to_signed_bytes_be(diff_add);
                }
            }
        } else {
            block.base_target = INITIAL_BASE_TARGET;
            let two64 = num_bigint::BigUint::from(18446744073709551616u128);
            let diff_add = two64 / num_bigint::BigUint::from(INITIAL_BASE_TARGET);
            block.cumulative_difficulty = biguint_to_signed_bytes_be(diff_add);
        }

        self.insert_block(&block).await?;

        info!("Accepted block: height={}, id={}, generator={}, base_target={}, txs={}",
              block_height, block_id, block.get_generator_id(), block.base_target, block.transactions.len());

        Ok(())
    }

    async fn has_block(&self, block_id: u64) -> anyhow::Result<bool> {
        match self.block_repo.has_block(block_id as i64).await {
            Ok(exists) => Ok(exists),
            Err(e) => Err(anyhow::anyhow!("Database error: {}", e)),
        }
    }

    async fn get_last_block_id(&self) -> anyhow::Result<Option<u64>> {
        match self.block_repo.find_latest().await {
            Ok(Some(block)) => Ok(Some(block.id as u64)),
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Database error: {}", e)),
        }
    }

    async fn get_last_block_cumulative_difficulty(&self) -> anyhow::Result<Vec<u8>> {
        match self.block_repo.find_latest().await {
            Ok(Some(block)) => Ok(block.cumulative_difficulty),
            Ok(None) => Ok(vec![]),
            Err(e) => Err(anyhow::anyhow!("Database error: {}", e)),
        }
    }

    async fn can_connect_block(&self, previous_block_id: u64) -> anyhow::Result<bool> {
        match self.get_last_block_id().await? {
            Some(last_id) => Ok(last_id == previous_block_id),
            None => Ok(false),
        }
    }

    async fn process_fork_block(&self, block: Block) -> anyhow::Result<()> {
        warn!("Fork block detected at height {}, storing for later processing", block.height);
        self.verify_and_process(block).await
    }

    async fn get_block_height(&self, block_id: u64) -> anyhow::Result<Option<u32>> {
        match self.block_repo.find_by_id_column(block_id as i64).await {
            Ok(Some(block)) => Ok(Some(block.height as u32)),
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Database error: {}", e)),
        }
    }

    async fn get_height(&self) -> anyhow::Result<u32> {
        match self.block_repo.find_latest().await {
            Ok(Some(block)) => Ok(block.height as u32),
            Ok(None) => Ok(0),
            Err(e) => Err(anyhow::anyhow!("Database error: {}", e)),
        }
    }

    async fn get_cumulative_difficulty(&self) -> anyhow::Result<String> {
        match self.block_repo.find_latest().await {
            Ok(Some(block)) => {
                let bytes = &block.cumulative_difficulty;
                let biguint = signed_bytes_be_to_biguint(bytes);
                Ok(biguint.to_string())
            }
            Ok(None) => Ok("0".to_string()),
            Err(e) => Err(anyhow::anyhow!("Database error: {}", e)),
        }
    }
}

pub struct NoOpBlockVerifier;

impl NoOpBlockVerifier {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoOpBlockVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BlockVerifier for NoOpBlockVerifier {
    async fn verify_and_process(&self, _block: Block) -> anyhow::Result<()> {
        Ok(())
    }

    async fn has_block(&self, _block_id: u64) -> anyhow::Result<bool> {
        Ok(false)
    }

    async fn get_last_block_id(&self) -> anyhow::Result<Option<u64>> {
        Ok(None)
    }

    async fn get_last_block_cumulative_difficulty(&self) -> anyhow::Result<Vec<u8>> {
        Ok(vec![])
    }

    async fn can_connect_block(&self, _previous_block_id: u64) -> anyhow::Result<bool> {
        Ok(true)
    }

    async fn process_fork_block(&self, _block: Block) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get_block_height(&self, _block_id: u64) -> anyhow::Result<Option<u32>> {
        Ok(None)
    }

    async fn get_height(&self) -> anyhow::Result<u32> {
        Ok(0)
    }

    async fn get_cumulative_difficulty(&self) -> anyhow::Result<String> {
        Ok("0".to_string())
    }
}
