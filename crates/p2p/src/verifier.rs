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
    pool: sqlx::SqlitePool,
    state: Arc<Mutex<()>>,
}

impl BlockchainVerifier {
    pub fn new(
        block_repo: Arc<dyn BlockRepository>,
        tx_repo: Arc<dyn TransactionRepository>,
        tx_processor: Arc<dyn TransactionProcessor>,
        block_reward_applicator: Arc<BlockRewardApplicator>,
        pool: sqlx::SqlitePool,
    ) -> Self {
        Self {
            block_repo,
            tx_repo,
            tx_processor,
            block_reward_applicator,
            pool,
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

    /// 验证区块内所有交易
    ///
    /// 对应 Java: BlockchainProcessor.validateTransactions()
    fn validate_transactions(&self, block: &Block) -> Result<()> {
        let mut total_amount: u64 = 0;
        let mut total_fee: u64 = 0;
        let mut total_payload_length: u32 = 0;

        // 对应 Java BlockchainProcessor.validateTransactions() 中的 hasPrunedTransactions 检测
        // 当 P2P 同步时，prunable attachment 数据可能被裁剪以节省带宽
        let mut has_pruned_transactions = false;

        for (idx, tx) in block.transactions.iter().enumerate() {
            // 1. 验证交易基本字段
            tx.validate_basic()
                .map_err(|e| BlockchainError::InvalidTransaction(
                    format!("transaction {} basic validation failed: {}", idx, e)
                ))?;

            // 2. 验证交易签名（Curve25519 EC-KCDSA）
            if !tx.verify_signature() {
                return Err(BlockchainError::InvalidTransaction(
                    format!("transaction {} signature verification failed", tx.id)
                ));
            }

            // 3. 累加金额和费用
            total_amount = total_amount.checked_add(tx.amount)
                .ok_or_else(|| BlockchainError::InvalidTransaction(
                    format!("transaction {} amount overflow", tx.id)
                ))?;
            total_fee = total_fee.checked_add(tx.fee)
                .ok_or_else(|| BlockchainError::InvalidTransaction(
                    format!("transaction {} fee overflow", tx.id)
                ))?;

            // 4. 累加 payload 长度（对应 Java: payloadLength += transaction.getFullSize()）
            // 基础大小 176 = signatureOffset(96) + signature(64) + version扩展(16)
            let tx_payload_len = 176 + tx.attachment_bytes.len() as u32;
            total_payload_length = total_payload_length.checked_add(tx_payload_len)
                .ok_or_else(|| BlockchainError::InvalidTransaction(
                    format!("transaction {} payload length overflow", tx.id)
                ))?;

            // 5. 检测 pruned transaction（对应 Java: IPrunable && !hasPrunableData()）
            if !has_pruned_transactions && Self::is_transaction_pruned(tx) {
                has_pruned_transactions = true;
            }
        }

        // 6. 验证区块头中的总金额和总费用
        if total_amount != block.total_amount {
            return Err(BlockchainError::InvalidTransaction(
                format!("total amount mismatch: header={}, computed={}", block.total_amount, total_amount)
            ));
        }
        if total_fee != block.total_fee {
            return Err(BlockchainError::InvalidTransaction(
                format!("total fee mismatch: header={}, computed={}", block.total_fee, total_fee)
            ));
        }

        // 7. 验证 payload length（对应 Java BlockchainProcessor.java:1157）
        // Java 逻辑:
        //   无 pruned transactions → 必须精确匹配 (payloadLength == header)
        //   有 pruned transactions → 允许 <= (payloadLength <= header，因为数据被裁剪)
        let payload_match = if has_pruned_transactions {
            total_payload_length <= block.payload_length
        } else {
            total_payload_length == block.payload_length
        };

        if !payload_match {
            return Err(BlockchainError::InvalidTransaction(
                format!(
                    "payload length mismatch: header={}, computed={}, has_pruned={}",
                    block.payload_length, total_payload_length, has_pruned_transactions
                )
            ));
        } else if has_pruned_transactions && total_payload_length < block.payload_length {
            debug!(
                "Payload length with pruned data: computed={}, block_header={}",
                total_payload_length, block.payload_length
            );
        }

        Ok(())
    }

    /// 检测交易是否包含被裁剪的 prunable attachment 数据
    ///
    /// 对应 Java: appendage instanceof IPrunable && !appendage.hasPrunableData()
    ///
    /// P2P GetNextBlocks 返回的 JSON 中，prunable attachment 的实际数据可能被裁剪，
    /// 只保留 hash 用于验证。这导致从 JSON 重建的 attachment_bytes 比原始数据短。
    fn is_transaction_pruned(tx: &Transaction) -> bool {
        tx.has_prunable_message || tx.has_prunable_encrypted_message || tx.has_prunable_attachment
    }

    /// 计算区块的 Payload Hash
    ///
    /// 对应 Java: BlockchainProcessor.validateTransactions() 中的 digest.update(transaction.getBytes())
    /// 使用 SHA256( tx1.getBytes() + tx2.getBytes() + ... + txN.getBytes() )
    fn compute_payload_hash(&self, txs: &[Transaction]) -> Result<Hash256> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        for tx in txs {
            hasher.update(tx.get_bytes());
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

    #[allow(dead_code)]
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

    /// Transaction-aware version of insert_block
    async fn insert_block_tx<'a>(
        &self,
        block: &Block,
        tx: &mut sqlx::Transaction<'a, sqlx::Sqlite>,
    ) -> anyhow::Result<()> {
        let block_model = BlockModel::from_domain(block)?;

        // Insert block using raw SQL with transaction
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
        .bind(block_model.id)
        .bind(block_model.version)
        .bind(block_model.timestamp)
        .bind(block_model.previous_block_id)
        .bind(block_model.total_amount)
        .bind(block_model.total_fee)
        .bind(block_model.payload_length)
        .bind(&block_model.previous_block_hash)
        .bind(&block_model.cumulative_difficulty)
        .bind(block_model.base_target)
        .bind(block_model.next_block_id)
        .bind(block_model.height)
        .bind(&block_model.generation_signature)
        .bind(&block_model.block_signature)
        .bind(&block_model.payload_hash)
        .bind(block_model.generator_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to insert block: {}", e))?;

        let block_id = block.get_id() as i64;

        // Update previous block's next_block_id
        if let Some(prev_id) = block.previous_block_id.filter(|&id| id != 0) {
            sqlx::query("UPDATE block SET next_block_id = ? WHERE id = ?")
                .bind(block_id)
                .bind(prev_id as i64)
                .execute(&mut **tx)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to update next_block_id: {}", e))?;
        }

        // Insert transactions
        for (idx, tx_item) in block.transactions.iter().enumerate() {
            let mut tx_model = TransactionModel::from_domain(tx_item)?;
            tx_model.height = block.height as i32;
            tx_model.block_id = block_id;
            tx_model.transaction_index = idx as i16;

            sqlx::query(
                r#"
                INSERT INTO "transaction" (
                    id, deadline, sender_id, recipient_id, amount,
                    fee, height, block_id, transaction_index, timestamp,
                    type, subtype, block_timestamp, full_hash, signature,
                    referenced_transaction_full_hash, attachment_bytes,
                    version, phased, has_message, has_encrypted_message,
                    has_public_key_announcement, has_prunable_message,
                    has_prunable_attachment, ec_block_height, ec_block_id,
                    has_encrypttoself_message, has_prunable_encrypted_message
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                )
                "#,
            )
            .bind(tx_model.id)
            .bind(tx_model.deadline)
            .bind(tx_model.sender_id)
            .bind(tx_model.recipient_id)
            .bind(tx_model.amount)
            .bind(tx_model.fee)
            .bind(tx_model.height)
            .bind(tx_model.block_id)
            .bind(tx_model.transaction_index)
            .bind(tx_model.timestamp)
            .bind(tx_model.r#type)
            .bind(tx_model.subtype)
            .bind(tx_model.block_timestamp)
            .bind(&tx_model.full_hash)
            .bind(&tx_model.signature)
            .bind(&tx_model.referenced_transaction_full_hash)
            .bind(&tx_model.attachment_bytes)
            .bind(tx_model.version)
            .bind(tx_model.phased)
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
            .map_err(|e| anyhow::anyhow!("Failed to insert transaction {}: {}", tx_item.id, e))?;
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

    /// Transaction-aware version of accept_block
    /// Note: The tx_processor and block_reward_applicator use their own connections,
    /// so this method delegates to the original accept_block.
    /// The database transaction protects the block/transaction inserts.
    #[allow(dead_code)]
    async fn accept_block_tx<'a>(
        &self,
        block: &Block,
        _tx: &mut sqlx::Transaction<'a, sqlx::Sqlite>,
    ) -> anyhow::Result<()> {
        self.accept_block(block).await
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

        // Set current block context for ledger entries
        // Reference: Java LedgerEntry constructor gets block_id/height/timestamp from Blockchain.getLastBlock()
        self.tx_processor.set_current_block(block.id.unwrap_or(0) as i64, block.height as i32, block.timestamp as i32);

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

            // Java Transaction.apply(): if (attachmentIsPhased()) {
            //     senderAccount.addToBalance(type.getLedgerEvent(), getId(), 0, -this.getFeeNQT());
            // }
            // For phased transactions, deduct fee from confirmed balance separately
            if transaction.phased {
                if let Err(e) = self.tx_processor.apply_phased_fee(transaction).await {
                    error!("CRITICAL: Failed to apply phased fee for tx {}: {}", transaction.id, e);
                    return Err(anyhow::anyhow!(
                        "phased fee application failed (tx={}): {}",
                        transaction.id, e
                    ));
                }
            }
        }
        debug!("✅ Phase 3 complete: {} transactions executed", block.transactions.len());

        debug!("✅ Accept complete: height={}, txs={}", block.height, block.transactions.len());

        Ok(())
    }

    /// 清理已插入但 accept 失败的区块数据
    ///
    /// 当 insert 成功但 accept（apply_unconfirmed/apply）失败时调用，
    /// 删除指定高度的区块及其所有交易，保持数据库一致性
    async fn cleanup_inserted_block(&self, height: i32) -> anyhow::Result<()> {
        let mut db_tx = self.pool.begin().await
            .map_err(|e| anyhow::anyhow!("Failed to begin cleanup transaction: {}", e))?;

        if let Some(block_model) = self.block_repo.find_by_height(height).await
            .map_err(|e| anyhow::anyhow!("Failed to find block at height {}: {}", height, e))?
        {
            let block_id = block_model.id;

            let txs = self.tx_repo.find_by_block(block_id).await
                .map_err(|e| anyhow::anyhow!("Failed to find transactions for block {}: {}", block_id, e))?;

            for tx in &txs {
                sqlx::query("DELETE FROM \"transaction\" WHERE db_id = ?")
                    .bind(tx.db_id)
                    .execute(&mut *db_tx)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to delete transaction {}: {}", tx.db_id, e))?;
            }

            debug!("Cleaned up {} transactions from block {}", txs.len(), block_id);

            sqlx::query("DELETE FROM block WHERE db_id = ?")
                .bind(block_model.db_id)
                .execute(&mut *db_tx)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to delete block {}: {}", block_model.db_id, e))?;

            debug!("Cleaned up block at height={}, id={}", height, block_id);
        }

        db_tx.commit().await
            .map_err(|e| anyhow::anyhow!("Failed to commit cleanup transaction: {}", e))
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

        // Step 2.5: Verify block signature (Curve25519 EC-KCDSA)
        // 对应 Java: block.verifyBlockSignature()
        if block_height > 0 {
            match block.verify_block_signature() {
                Ok(true) => {
                    debug!("Block signature verified: height={}", block_height);
                }
                Ok(false) => {
                    return Err(anyhow::anyhow!(
                        "Block signature verification failed at height {}", block_height
                    ));
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Block signature verification error at height {}: {}", block_height, e
                    ));
                }
            }
        }

        // Step 2.6: Validate all transactions in the block
        // 对应 Java: BlockchainProcessor.validateTransactions()
        self.validate_transactions(&block)?;

        // Step 3: Check if block already exists
        if let Ok(Some(_)) = self.block_repo.find_by_height(block_height as i32).await {
            debug!("Block at height {} already exists, skipping", block_height);
            return Ok(());
        }

        // Step 4: Validate payload hash
        // Reference: Java BlockchainProcessor - validates payload hash
        let computed_payload_hash = self.compute_payload_hash(&block.transactions)?;
        if computed_payload_hash != block.payload_hash {
            return Err(anyhow::anyhow!(
                "Payload hash mismatch at height {}: computed {:?} != block {:?}",
                block_height, computed_payload_hash, block.payload_hash
            ));
        }

        if block_height > 0 {
            let prev_height = (block_height - 1) as i32;
            match self.block_repo.find_by_height(prev_height).await {
                Ok(Some(prev_model)) => {
                    // Verify generation signature
                    // 对应 Java: block.verifyGenerationSignature()
                    if block.version >= 2 {
                        match block.verify_generation_signature(&prev_model.generation_signature) {
                            Ok(true) => {
                                debug!("Generation signature verified: height={}", block_height);
                            }
                            Ok(false) => {
                                return Err(anyhow::anyhow!(
                                    "Generation signature verification failed at height {}", block_height
                                ));
                            }
                            Err(e) => {
                                return Err(anyhow::anyhow!(
                                    "Generation signature verification error at height {}: {}", block_height, e
                                ));
                            }
                        }
                    }

                    // Verify timestamp ordering: current block timestamp must be > previous block timestamp
                    // Reference: Java BlockchainProcessor
                    if block.timestamp <= prev_model.timestamp as u32 {
                        return Err(anyhow::anyhow!(
                            "Block timestamp {} at height {} is not greater than previous block timestamp {}",
                            block.timestamp, block_height, prev_model.timestamp
                        ));
                    }

                    // Verify previous block hash matches
                    // Reference: Java BlockchainProcessor - validates previousBlockId and previousBlockHash
                    if let Some(prev_block_id) = block.previous_block_id {
                        if prev_block_id != 0 && prev_block_id as i64 != prev_model.id {
                            return Err(anyhow::anyhow!(
                                "Previous block ID mismatch at height {}: block has {}, expected {}",
                                block_height, prev_block_id, prev_model.id
                            ));
                        }
                    }

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

        // Step 5: Insert block to database（独立事务）
        // 对应 Java: Db.beginTransaction() / Db.commitTransaction()
        // 注意：insert 和 accept 必须拆分为两个独立事务，因为：
        //   - insert_block_tx 使用 db_tx 连接写入
        //   - accept_block 内部调用 tx_processor.apply_unconfirmed() 使用 tx_processor 自有的 pool 连接
        //   - SQLite 不允许两个不同连接同时持有写锁（SQLITE_BUSY, error code 5）
        {
            let mut insert_tx = self.pool.begin().await
                .map_err(|e| anyhow::anyhow!("Failed to begin insert transaction: {}", e))?;

            match self.insert_block_tx(&block, &mut insert_tx).await {
                Ok(()) => {
                    insert_tx.commit().await
                        .map_err(|e| anyhow::anyhow!("Failed to commit insert transaction: {}", e))?;
                }
                Err(e) => {
                    insert_tx.rollback().await.ok();
                    return Err(e);
                }
            }
        }

        // Step 6: Execute accept flow（tx_processor 使用自有连接，无锁冲突）
        // 对应 Java: BlockProcessor.accept()
        // 如果 accept 失败，需要清理已插入的区块数据
        match self.accept_block(&block).await {
            Ok(()) => {
                info!("Block accepted: height={}, id={}, txs={}", block_height, block_id, block.transactions.len());
                Ok(())
            }
            Err(e) => {
                warn!(
                    "❌ Failed to accept block at height={}: {}, cleaning up inserted block",
                    block_height, e
                );
                // accept 失败时回滚：删除已插入的区块和交易数据
                if let Err(cleanup_err) = self.cleanup_inserted_block(block.height as i32).await {
                    error!("Failed to cleanup block at height {}: {}", block_height, cleanup_err);
                }
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

    /// 回滚区块链到指定高度
    ///
    /// 对应 Java: BlockchainProcessor.popOffTo(int height)
    ///
    /// 1. 获取所有高于目标高度的区块
    /// 2. 删除这些区块中的所有交易
    /// 3. 删除这些区块
    /// 4. 返回被删除的区块列表
    async fn pop_off_to(&self, height: u32) -> anyhow::Result<Vec<Block>> {
        let _guard = self.state.lock().await;

        info!("Popping off blocks after height {}", height);

        // Step 1: Find blocks to be removed
        let blocks_to_remove = self.block_repo.find_blocks_after_height(height as i32).await
            .map_err(|e| anyhow::anyhow!("Failed to find blocks after height {}: {}", height, e))?;

        if blocks_to_remove.is_empty() {
            info!("No blocks to pop off after height {}", height);
            return Ok(vec![]);
        }

        info!("Found {} blocks to pop off (heights {} to {})",
            blocks_to_remove.len(),
            blocks_to_remove.first().map(|b| b.height).unwrap_or(0),
            blocks_to_remove.last().map(|b| b.height).unwrap_or(0)
        );

        // Step 2: Wrap in database transaction for atomicity
        let db_tx = self.pool.begin().await
            .map_err(|e| anyhow::anyhow!("Failed to begin database transaction: {}", e))?;

        // Step 3: Collect all transaction IDs to delete
        let mut all_tx_ids: Vec<i64> = Vec::new();
        for block_model in &blocks_to_remove {
            let block_id = block_model.id;
            let txs = self.tx_repo.find_by_block(block_id).await
                .map_err(|e| anyhow::anyhow!("Failed to find transactions for block {}: {}", block_id, e))?;

            for tx in &txs {
                all_tx_ids.push(tx.db_id);
            }

            debug!("Found {} transactions in block {} to delete", txs.len(), block_id);
        }

        // Step 4: Delete all transactions using repository method
        if !all_tx_ids.is_empty() {
            self.tx_repo.delete_transactions_by_ids(&all_tx_ids).await
                .map_err(|e| anyhow::anyhow!("Failed to delete transactions: {}", e))?;
            debug!("Deleted {} transactions total", all_tx_ids.len());
        }

        // Step 5: Collect all block IDs to delete
        let block_db_ids: Vec<i64> = blocks_to_remove.iter().map(|b| b.db_id).collect();

        // Step 6: Delete blocks using repository method
        if !block_db_ids.is_empty() {
            self.block_repo.delete_blocks_by_ids(&block_db_ids).await
                .map_err(|e| anyhow::anyhow!("Failed to delete blocks: {}", e))?;
        }

        // Step 7: Commit
        db_tx.commit().await
            .map_err(|e| anyhow::anyhow!("Failed to commit pop_off_to transaction: {}", e))?;

        info!("Popped off {} blocks successfully", blocks_to_remove.len());

        // Convert BlockModel to Block for return
        let blocks: Vec<Block> = blocks_to_remove.iter()
            .filter_map(|m| m.to_domain().ok())
            .collect();

        Ok(blocks)
    }
}
