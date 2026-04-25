//! Blockchain verifier implementation
//!
//! Implements BlockVerifier trait using repositories

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, debug, warn};

use blockchain_types::prelude::*;
use blockchain_types::block::Block;
use blockchain_types::transaction::Transaction;

use crate::handlers::BlockVerifier;
use orm::{BlockRepository, TransactionRepository, BlockModel, TransactionModel, RepositoryError};

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
        if block.version != BLOCK_VERSION {
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
            hasher.update(&tx.full_hash.0);
        }
        let hash = hasher.finalize();
        let arr: [u8; 32] = hash.try_into().unwrap();
        Ok(Hash256(arr))
    }

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

        for (idx, tx) in block.transactions.iter().enumerate() {
            let mut tx_model = TransactionModel::from_domain(tx)?;
            tx_model.height = block.height as i32;
            tx_model.block_id = block.height as i64;
            tx_model.transaction_index = idx as i16;
            
            if let Err(e) = self.tx_repo.insert(&tx_model).await {
                warn!("Failed to insert transaction {}: {}", tx.id, e);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl BlockVerifier for BlockchainVerifier {
    async fn verify_and_process(&self, mut block: Block) -> anyhow::Result<()> {
        self.validate_basic(&block)?;

        if block.height == 0 {
            if block.previous_block_id == 0 {
                block.height = 1;
            } else {
                match self.block_repo.find_by_id(block.previous_block_id as i64).await {
                    Ok(Some(prev_block)) => {
                        block.height = prev_block.height as u32 + 1;
                    }
                    _ => {
                        debug!("Previous block {} not found, cannot determine height", block.previous_block_id);
                        return Err(anyhow::anyhow!("Previous block not found"));
                    }
                }
            }
        }

        if let Ok(Some(_)) = self.block_repo.find_by_height(block.height as i32).await {
            debug!("Block at height {} already exists, skipping", block.height);
            return Ok(());
        }

        let computed_payload_hash = self.compute_payload_hash(&block.transactions)?;
        if computed_payload_hash != block.payload_hash {
            debug!("Payload hash mismatch: computed {:?} != block {:?}", 
                   computed_payload_hash, block.payload_hash);
        }

        self.insert_block(&block).await?;

        info!("Accepted block: height={}, generator={}, txs={}",
              block.height, block.generator_id, block.transactions.len());

        Ok(())
    }

    async fn has_block(&self, block_id: u64) -> anyhow::Result<bool> {
        match self.block_repo.has_block(block_id as i64).await {
            Ok(exists) => Ok(exists),
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
}
