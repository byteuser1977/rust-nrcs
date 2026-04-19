//! Chain Service (simplified placeholder implementation)

use std::sync::Arc;

use blockchain_types::prelude::*;
use blockchain_types::consensus::{BlockchainState, AccountSnapshot};
use orm::{BlockRepository, BlockModel};
use tx_engine::TransactionProcessor;
use account::AccountManager;
use chrono::Utc;
use tracing::info;
use sqlx::PgPool;
use num_bigint::BigUint;
use num_traits::{Zero, ToPrimitive};

pub struct ChainService {
    block_repo: Arc<dyn BlockRepository>,
    tx_repo: Arc<dyn orm::TransactionRepository>,
    tx_processor: Arc<dyn TransactionProcessor>,
    account_manager: Arc<dyn AccountManager>,
    db_pool: PgPool,
    current_height: Arc<tokio::sync::Mutex<Height>>,
    // Consensus engine for difficulty and block reward
    consensus: Arc<dyn consensus::ConsensusEngine + Send + Sync>,
    block_reward: Amount,
}

impl ChainService {
    pub fn new(
        block_repo: Arc<dyn BlockRepository>,
        tx_repo: Arc<dyn orm::TransactionRepository>,
        tx_processor: Arc<dyn TransactionProcessor>,
        account_manager: Arc<dyn AccountManager>,
        db_pool: PgPool,
        consensus: Arc<dyn consensus::ConsensusEngine + Send + Sync>,
        block_reward: Amount,
    ) -> Self {
        Self {
            block_repo,
            tx_repo,
            tx_processor,
            account_manager,
            db_pool,
            current_height: Arc::new(tokio::sync::Mutex::new(0)),
            consensus,
            block_reward,
        }
    }

    pub async fn current_height(&self) -> Height {
        *self.current_height.lock().await
    }

    pub async fn get_block(&self, height: Height) -> anyhow::Result<Option<Block>> {
        info!("Getting block at height {}", height);
        Ok(None)
    }

    pub async fn get_latest_block(&self) -> anyhow::Result<Option<Block>> {
        let model = self.block_repo.find_latest().await?;
        if let Some(m) = model {
            let block = m.to_domain().map_err(|e| anyhow::anyhow!("{:?}", e))?;
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    pub async fn get_public_key(&self, account_id: AccountId) -> anyhow::Result<Option<PublicKey>> {
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            "SELECT public_key FROM public_key WHERE account_id = $1 AND latest = TRUE ORDER BY height DESC LIMIT 1"
        )
        .bind(account_id as i64)
        .fetch_optional(&self.db_pool)
        .await?;
        if let Some((pk_vec,)) = row {
            if let Ok(bytes) = pk_vec.try_into() {
                return Ok(Some(PublicKey::Ed25519(bytes)));
            }
        }
        Ok(None)
    }

    pub async fn get_current_state(&self) -> anyhow::Result<BlockchainState> {
        // Get latest block
        let latest_block_opt = self.block_repo.find_latest().await?;
        let (height, last_block_hash, last_base_target, cumulative_difficulty, last_generation_signature, last_timestamp) = 
            if let Some(block_model) = latest_block_opt {
                let block = block_model.to_domain()?;
                let block_hash = block.compute_hash()?;
                (
                    block.height,
                    block_hash,
                    block.base_target,
                    block.cumulative_difficulty,
                    block.generation_signature,
                    block.timestamp,
                )
            } else {
                // Genesis state
                return Ok(BlockchainState::new(
                    0,
                    [0; 32],
                    1_000_000,
                    vec![],
                    [0; 64],
                    0,
                    vec![],
                ));
            };

        // Get accounts with positive balance
        let account_models = self.account_repo.find_all(Some(10000), None).await?;
        let mut snapshots = Vec::new();
        for am in account_models {
            let account = am.to_domain()?;
            if account.balance > 0 {
                let has_pubkey = self.get_public_key(account.id).await?.is_some();
                snapshots.push(AccountSnapshot {
                    id: account.id,
                    balance: account.balance,
                    lease: None,
                    has_public_key: has_pubkey,
                });
            }
        }

        Ok(BlockchainState::new(
            height,
            last_block_hash,
            last_base_target,
            cumulative_difficulty,
            last_generation_signature,
            last_timestamp,
            snapshots,
        ))
    }

    pub async fn process_block(&self, block: Block) -> anyhow::Result<()> {
        // Verify block signature
        let pubkey_opt = self.account_manager.get_public_key(block.generator_id).await?;
        let pubkey = pubkey_opt.ok_or_else(|| anyhow::anyhow!("missing public key for generator"))?;
        block.verify_signature(&pubkey)?;

        // Verify payload hash
        let computed_payload = Block::compute_merkle_root(&block.transactions)?;
        if computed_payload != block.payload_hash {
            return Err(anyhow::anyhow!("payload hash mismatch"));
        }

        // Verify height
        let current = self.current_height().await;
        if block.height != current + 1 {
            return Err(anyhow::anyhow!("height mismatch: expected {}, got {}", current + 1, block.height));
        }

        // Verify previous block hash
        let latest = self.get_latest_block().await?;
        if let Some(latest_block) = latest {
            let latest_hash = latest_block.compute_hash()?;
            if block.previous_block_hash != latest_hash {
                return Err(anyhow::anyhow!("previous block hash mismatch"));
            }
        } else {
            if block.height != 1 {
                return Err(anyhow::anyhow!("first block must be height 1"));
            }
        }

        // Verify difficulty using consensus
        self.consensus.verify_difficulty(&block)?;

        // Insert block
        let mut block_model = BlockModel::from_domain(&block)?;
        block_model.id = block.height as i64;
        block_model.height = block.height as i32;
        self.block_repo.insert(&block_model).await?;

        // Update current height
        *self.current_height.lock().await = block.height;

        // Apply transactions
        for tx in &block.transactions {
            self.tx_processor.validate(tx).await?;
            self.tx_processor.apply(tx).await?;
        }

        // Credit block reward
        self.account_manager.credit(block.generator_id, self.block_reward).await?;

        Ok(())
    }

    pub async fn create_block(
        &self,
        generator_id: AccountId,
        transactions: Vec<Transaction>,
    ) -> anyhow::Result<Block> {
        let current_height = self.current_height().await;
        let latest_block_opt = self.get_latest_block().await?;
        let (prev_hash, prev_base_target, prev_cumulative, _prev_gen_sig) = match latest_block_opt {
            Some(latest) => {
                let prev_hash = latest.compute_hash()?;
                (prev_hash, latest.base_target, latest.cumulative_difficulty, latest.generation_signature)
            }
            None => {
                ([0; 32], 1_000_000, vec![], [0; 64])
            }
        };

        // Compute payload hash
        let payload_hash = Block::compute_merkle_root(&transactions)?;

        // Calculate next difficulty
        let recent_blocks: Vec<Block> = self.block_repo
            .find_range(
                std::cmp::max(0, current_height as i32 - 10) as i32,
                current_height as i32,
            )
            .await?
            .into_iter()
            .filter_map(|m| m.to_domain().ok())
            .collect();
        let next_base_target = self.consensus.calculate_next_difficulty(&recent_blocks);

        // Compute cumulative difficulty
        let mut cum_big = BigUint::from_bytes_be(&prev_cumulative);
        cum_big += BigUint::from(prev_base_target);
        let new_cumulative = cum_big.to_bytes_be();

        // Timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as Timestamp;

        // Build block
        let mut block = Block::new(current_height + 1, prev_hash, generator_id);
        block.timestamp = now;
        block.payload_hash = payload_hash;
        block.base_target = next_base_target;
        block.cumulative_difficulty = new_cumulative;
        block.generation_signature = [0; 64]; // TODO: generate via consensus
        block.total_amount = transactions.iter().map(|tx| tx.amount).sum();
        block.total_fee = transactions.iter().map(|tx| tx.fee).sum();
        block.payload_length = transactions.iter().map(|tx| tx.size()).sum::<usize>() as u32;
        block.nonce = 0;
        block.transactions = transactions;

        Ok(block)
    }

    pub async fn start_sync(&self) -> anyhow::Result<()> {
        info!("Chain sync started");
        // TODO: implement sync
        Ok(())
    }


    pub async fn create_block(
        &self,
        _generator_id: AccountId,
        _transactions: Vec<Transaction>,
    ) -> anyhow::Result<Block> {
        let prev_block = Block::new(
            self.current_height().await + 1,
            [0u8; 32],
            _generator_id,
        );
        Ok(prev_block)
    }

    pub async fn start_sync(&self) -> anyhow::Result<()> {
        info!("Chain sync started");
        Ok(())
    }
}
