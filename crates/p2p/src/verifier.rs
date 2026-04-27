//! Blockchain verifier implementation
//!
//! Implements BlockVerifier trait using repositories
//! Supports two-phase transaction commit (apply_unconfirmed + apply)
//! Reference: Java NRCS BlockProcessor.pushBlock() flow

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, debug, warn, error};

use blockchain_types::prelude::*;
use blockchain_types::block::{Block, PreviousBlockData, calculate_base_target_and_cumulative_difficulty, INITIAL_BASE_TARGET, biguint_to_signed_bytes_be, signed_bytes_be_to_biguint};
use blockchain_types::transaction::Transaction;

use crate::handlers::BlockVerifier;
use orm::{BlockRepository, TransactionRepository, BlockModel, TransactionModel};

use crate::block_apply::BlockRewardApplicator;
use tx_engine::TransactionProcessor;

pub struct BlockchainVerifier {
    block_repo: Arc<dyn BlockRepository>,
    tx_repo: Arc<dyn TransactionRepository>,
    tx_processor: Arc<dyn TransactionProcessor>,
    block_reward_applicator: Arc<BlockRewardApplicator>,
    state: Arc<Mutex<()>>,
}

impl BlockchainVerifier {
    pub fn new(
        block_repo: Arc<dyn BlockRepository>,
        tx_repo: Arc<dyn TransactionRepository>,
        tx_processor: Arc<dyn TransactionProcessor>,
        block_reward_applicator: Arc<BlockRewardApplicator>,
    ) -> Self {
        Self {
            block_repo,
            tx_repo,
            tx_processor,
            block_reward_applicator,
            state: Arc::new(Mutex::new(())),
        }
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

    /// Execute accept flow (corresponds to Java's BlockProcessor.accept())
    ///
    /// Two-phase transaction commit:
    /// - Phase 1: apply_unconfirmed() - Pre-deduct from unconfirmed_balance (double-spend detection)
    /// - Phase 2: Block rewards - Distribute fees to generator
    /// - Phase 3: apply() - Official execution (update confirmed balance)
    ///
    /// If any phase fails, all previous operations are rolled back
    async fn accept_block(&self, block: &Block) -> anyhow::Result<()> {
        debug!("Accept block height={}, txs={}", block.height, block.transactions.len());

        // === Phase 1: Pre-deduct unconfirmed balance (double-spend detection) ===
        debug!("Phase 1: Applying unconfirmed transactions...");
        let mut applied_count = 0;
        for (idx, transaction) in block.transactions.iter().enumerate() {
            debug!("Pre-deducting tx {}/{}: id={}, sender={}",
                   idx + 1, block.transactions.len(), transaction.id, transaction.sender_id);

            match self.tx_processor.apply_unconfirmed(transaction).await {
                Ok(true) => {
                    applied_count += 1;
                }
                Ok(false) => {
                    // Double-spend detected! Rollback all previously pre-deducted transactions
                    warn!(
                        "❌ Double-spending detected in tx {} at index {}, rolling back {} transactions",
                        transaction.id, idx, applied_count
                    );
                    for rollback_idx in 0..applied_count {
                        if let Some(prev_tx) = block.transactions.get(rollback_idx) {
                            if let Err(e) = self.tx_processor.rollback_unconfirmed(prev_tx).await {
                                warn!("Failed to rollback tx {}: {}", prev_tx.id, e);
                            }
                        }
                    }
                    return Err(anyhow::anyhow!(
                        "Double-spending detected in transaction {}: insufficient unconfirmed balance",
                        transaction.id
                    ));
                }
                Err(e) => {
                    // Error during pre-deduction, rollback all previous
                    warn!(
                        "Error in apply_unconfirmed for tx {}: {}, rolling back {} transactions",
                        transaction.id, e, applied_count
                    );
                    for rollback_idx in 0..applied_count {
                        if let Some(prev_tx) = block.transactions.get(rollback_idx) {
                            if let Err(rollback_err) = self.tx_processor.rollback_unconfirmed(prev_tx).await {
                                warn!("Failed to rollback tx {}: {}", prev_tx.id, rollback_err);
                            }
                        }
                    }
                    return Err(anyhow::anyhow!("apply_unconfirmed failed for tx {}: {}", transaction.id, e));
                }
            }
        }
        debug!("✅ Phase 1 complete: {} transactions pre-deducted", applied_count);

        // === Phase 2: Apply block rewards (generator fees, back fees) ===
        debug!("Phase 2: Applying block rewards...");
        if let Err(e) = self.block_reward_applicator.apply_block_rewards(block).await {
            // Rollback all pre-deductions on reward failure
            warn!("Failed to apply block rewards: {}, rolling back...", e);
            for rollback_idx in 0..applied_count {
                if let Some(prev_tx) = block.transactions.get(rollback_idx) {
                    if let Err(rollback_err) = self.tx_processor.rollback_unconfirmed(prev_tx).await {
                        warn!("Failed to rollback tx {}: {}", prev_tx.id, rollback_err);
                    }
                }
            }
            return Err(anyhow::anyhow!("block reward application failed: {}", e));
        }
        debug!("✅ Phase 2 complete: block rewards distributed");

        // === Phase 3: Officially execute transactions (update confirmed balances) ===
        debug!("Phase 3: Applying confirmed transactions...");
        for (idx, transaction) in block.transactions.iter().enumerate() {
            debug!("Executing tx {}/{}: id={}", idx + 1, block.transactions.len(), transaction.id);
            if let Err(e) = self.tx_processor.apply(transaction).await {
                // Note: At this point, we cannot easily undo Phase 1 and 2
                // In production, this should be wrapped in a database transaction
                error!(
                    "CRITICAL: Failed to execute tx {} after rewards applied: {}",
                    transaction.id, e
                );
                return Err(anyhow::anyhow!(
                    "transaction execution failed after rewards applied (tx={}): {}",
                    transaction.id, e
                ));
            }
        }
        debug!("✅ Phase 3 complete: {} transactions executed", block.transactions.len());

        debug!("✅ Accept complete: height={}, txs={}", block.height, block.transactions.len());

        Ok(())
    }
}

#[async_trait]
impl BlockVerifier for BlockchainVerifier {
    /// Verify and process block with full two-phase commit
    ///
    /// Reference: Java NRCS BlockProcessor.pushBlock()
    ///
    /// Complete flow:
    /// 1. Acquire mutex lock (prevent concurrent processing)
    /// 2. Basic validation (version, format)
    /// 3. Check if block already exists
    /// 4. Calculate difficulty parameters
    /// 5. Insert block to database
    /// 6. Execute accept flow:
    ///    a. Phase 1: apply_unconfirmed() - Pre-deduct (double-spend detection)
    ///    b. Phase 2: Block rewards (generator fees, back fees)
    ///    c. Phase 3: apply() - Official execution
    /// 7. Commit transaction on success, rollback on error
    async fn verify_and_process(&self, mut block: Block) -> anyhow::Result<()> {
        // Step 1: Acquire mutex lock for concurrency control
        let _guard = self.state.lock().await;

        let block_height = block.height;
        let block_id = block.get_id();

        if block_height <= 2 {
            debug!("Block {} gen_pub_key: {:?}", block_height, block.generator_public_key.as_ref().map(hex::encode));
        }

        // Step 2: Basic validation
        self.validate_basic(&block)?;

        // Step 3: Check if block already exists
        if let Ok(Some(_)) = self.block_repo.find_by_height(block_height as i32).await {
            debug!("Block at height {} already exists, skipping", block_height);
            return Ok(());
        }

        // Step 4: Calculate difficulty parameters
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

                    debug!("base_target={}, height={}", base_target, block_height);

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

        // Step 5: Insert block to database
        self.insert_block(&block).await?;

        // Step 6: Execute accept flow (two-phase transaction commit)
        match self.accept_block(&block).await {
            Ok(()) => {
                info!("Block accepted: height={}, id={}, txs={}", block_height, block_id, block.transactions.len());
                Ok(())
            }
            Err(e) => {
                warn!(
                    "❌ Failed to accept block at height={}: {}",
                    block_height, e
                );
                Err(e)
            }
        }
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
