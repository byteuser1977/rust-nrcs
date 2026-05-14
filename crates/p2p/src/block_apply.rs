//! Block reward application logic
//!
//! Implements Block.apply() from Java NRCS:
//! - Distribute block rewards to generator
//! - Handle back fees distribution
//! - Update forged_balance
//! - Record BLOCK_GENERATED entries in account_ledger

use std::sync::Arc;

use blockchain_types::prelude::{Block, AccountId, Transaction, TransactionType};
use orm::{AccountRepository, BlockRepository, PublicKeyRepository, AccountLedgerRepository, AccountGuaranteedBalanceRepository};
use orm::models::AccountLedgerModel;
use tracing::debug;

/// Block reward applicator (corresponds to Java's Block.apply())
pub struct BlockRewardApplicator {
    account_repo: Arc<dyn AccountRepository>,
    block_repo: Arc<dyn BlockRepository>,
    public_key_repo: Arc<dyn PublicKeyRepository>,
    ledger_repo: Option<Arc<dyn AccountLedgerRepository>>,
    guaranteed_balance_repo: Option<Arc<dyn AccountGuaranteedBalanceRepository>>,
}

impl BlockRewardApplicator {
    pub fn new(
        account_repo: Arc<dyn AccountRepository>,
        block_repo: Arc<dyn BlockRepository>,
        public_key_repo: Arc<dyn PublicKeyRepository>,
    ) -> Self {
        Self { account_repo, block_repo, public_key_repo, ledger_repo: None, guaranteed_balance_repo: None }
    }

    pub fn with_ledger(
        account_repo: Arc<dyn AccountRepository>,
        block_repo: Arc<dyn BlockRepository>,
        public_key_repo: Arc<dyn PublicKeyRepository>,
        ledger_repo: Arc<dyn AccountLedgerRepository>,
    ) -> Self {
        Self { account_repo, block_repo, public_key_repo, ledger_repo: Some(ledger_repo), guaranteed_balance_repo: None }
    }

    pub fn with_guaranteed_balance(
        account_repo: Arc<dyn AccountRepository>,
        block_repo: Arc<dyn BlockRepository>,
        public_key_repo: Arc<dyn PublicKeyRepository>,
        ledger_repo: Option<Arc<dyn AccountLedgerRepository>>,
        guaranteed_balance_repo: Arc<dyn AccountGuaranteedBalanceRepository>,
    ) -> Self {
        Self { account_repo, block_repo, public_key_repo, ledger_repo, guaranteed_balance_repo: Some(guaranteed_balance_repo) }
    }

    /// Apply block rewards (Java: Block.apply())
    ///
    /// Core logic:
    /// 1. Get or create generator account
    /// 2. Bind public key if first appearance
    /// 3. Calculate and distribute Back Fees
    /// 4. Distribute net fees to current generator
    /// 5. Update forged_balance
    pub async fn apply_block_rewards(&self, block: &Block) -> anyhow::Result<()> {
        let generator_id = block.get_generator_id();

        debug!("Applying block rewards for height={}, generator={}", block.height, generator_id);

        // Step 1: Ensure generator account exists
        self.ensure_generator_account(generator_id).await?;

        // Step 2: Bind public key (first-time accounts)
        if let Some(ref pk) = block.generator_public_key {
            self.bind_public_key(generator_id, pk, block.height as i32).await?;
        }

        // Step 3: Calculate and distribute Back Fees
        let total_back_fees = self.calculate_and_distribute_back_fees(block).await?;

        // Step 4: Distribute net fees to current generator
        let total_fee = block.total_fee as i64;
        let net_fee = total_fee.checked_sub(total_back_fees)
            .ok_or_else(|| anyhow::anyhow!("fee underflow: {} - {}", total_fee, total_back_fees))?;

        self.account_repo.add_to_balance_and_unconfirmed(
            generator_id as i64,
            net_fee,
            block.height as i32
        ).await.map_err(|e| anyhow::anyhow!("{}", e))?;

        if let Some(ref gb_repo) = self.guaranteed_balance_repo {
            if net_fee > 0 {
                gb_repo.upsert_additions(generator_id as i64, block.height as i32, net_fee).await
                    .map_err(|e| anyhow::anyhow!("failed to update guaranteed balance: {}", e))?;
            }
        }

        // Step 5: Update forged_balance
        self.account_repo.add_to_forged_balance(
            generator_id as i64,
            net_fee,
            block.height as i32
        ).await.map_err(|e| anyhow::anyhow!("{}", e))?;

        // Step 6: Record BLOCK_GENERATED ledger entry (if ledger_repo available)
        // ✅ 修复：添加出块奖励的账本记录
        // NRCS Java: Block.apply() 会调用 Account.addToBalance()，后者会记录 EVENT_TYPE=1
        if let Some(ref ledger) = self.ledger_repo {
            if net_fee > 0 {
                let balance_after = self.account_repo.find_by_account_id(generator_id as i64).await
                    .map_err(|e| anyhow::anyhow!("failed to get account: {}", e))?
                    .map(|acc| acc.balance)
                    .unwrap_or(0);

                let entry = AccountLedgerModel {
                    db_id: 0,
                    account_id: generator_id as i64,
                    event_type: 1, // BLOCK_GENERATED
                    event_id: block.id.unwrap_or(0) as i64,
                    holding_type: 1, // UNCONFIRMED_NRCS_BALANCE (区块奖励先进入未确认余额)
                    holding_id: Some(0), // ✅ 修复：对齐 NRCS，holding_id=0（不是 NULL）
                    change: net_fee,
                    balance: balance_after,
                    block_id: block.id.unwrap_or(0) as i64,
                    height: block.height as i32,
                    timestamp: block.timestamp as i32,
                };

                ledger.insert(&entry).await
                    .map_err(|e| anyhow::anyhow!("failed to insert BLOCK_GENERATED ledger: {}", e))?;
            }
        }

        debug!("Block reward: height={}, generator={}, net_fee={}", block.height, generator_id, net_fee);

        Ok(())
    }

    /// Ensure generator account exists (create if not)
    async fn ensure_generator_account(&self, account_id: AccountId) -> anyhow::Result<()> {
        match self.account_repo.find_by_account_id(account_id as i64).await {
            Ok(Some(_)) => Ok(()),
            Ok(None) => {
                debug!("Creating new generator account: {}", account_id);
                let _ = self.account_repo.get_or_create(account_id as i64).await
                    .map_err(|e| anyhow::anyhow!("failed to create account: {}", e))?;
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("database error: {}", e))
        }
    }

    /// Bind public key to account (for first-time appearing accounts)
    async fn bind_public_key(&self, account_id: AccountId, public_key: &[u8; 32], height: i32) -> anyhow::Result<()> {
        use blockchain_types::account_ext::AccountPublicKey;

        // Check if public key already exists
        match self.public_key_repo.find_latest_by_account_id(account_id as i64).await {
            Ok(Some(existing_pk)) => {
                if existing_pk.public_key.is_empty() || existing_pk.public_key == *public_key {
                    return Ok(());
                }
                debug!("Public key already bound for account {}", account_id);
                Ok(())
            }
            Ok(None) => {
                debug!("Binding public key to account {}", account_id);
                let pk_model = AccountPublicKey {
                    account_id: account_id as AccountId,
                    public_key: *public_key,
                    height: height as blockchain_types::Height,
                };
                self.public_key_repo.insert(&pk_model).await
                    .map_err(|e| anyhow::anyhow!("failed to bind public key: {}", e))
            }
            Err(e) => Err(anyhow::anyhow!("database error: {}", e))
        }
    }

    /// Calculate and distribute Back Fees to previous 3 blocks' generators
    ///
    /// Reference: Java Block.apply() and TransactionType.getBackFees()
    ///
    /// Back fees apply to:
    /// - Asset issuance (type=2, subtype=0) - except singleton issuance
    /// - Currency issuance (type=5, subtype=0)
    ///
    /// Formula: [fee * 3/10, fee * 2/10, fee * 1/10] distributed to previous 3 blocks' generators
    ///
    /// Only active after SHUFFLING_BLOCK (300000)
    async fn calculate_and_distribute_back_fees(&self, block: &Block) -> anyhow::Result<i64> {
        const SHUFFLING_BLOCK: u32 = 300_000;  // Reference: Java Constant.SHUFFLING_BLOCK

        if block.height <= SHUFFLING_BLOCK {
            return Ok(0);
        }

        // Calculate back fees from all transactions in the block
        // Reference: Java Block.apply() - iterates transactions and sums getBackFees()
        let mut back_fees = [0i64; 3];

        for tx in &block.transactions {
            let tx_back_fees = self.get_back_fees(tx);
            for i in 0..3 {
                back_fees[i] += tx_back_fees[i];
            }
        }

        let mut total_back_fees = 0i64;

        for (i, &fee) in back_fees.iter().enumerate() {
            if fee == 0 { break; }

            total_back_fees += fee;

            let target_height = (block.height as i32) - (i as i32) - 1;

            if let Some(prev_block) = self.block_repo
                .find_by_height(target_height).await
                .map_err(|e| anyhow::anyhow!("db error: {}", e))?
            {
                let prev_generator_id = prev_block.generator_id as u64;

                debug!(
                    "Back fees {} to generator at height {}: {}",
                    fee, target_height, prev_generator_id
                );

                self.account_repo.add_to_balance_and_unconfirmed(
                    prev_generator_id as i64,
                    fee,
                    target_height
                ).await.map_err(|e| anyhow::anyhow!("{}", e))?;

                if let Some(ref gb_repo) = self.guaranteed_balance_repo {
                    gb_repo.upsert_additions(prev_generator_id as i64, target_height, fee).await
                        .map_err(|e| anyhow::anyhow!("failed to update guaranteed balance for back fee: {}", e))?;
                }

                self.account_repo.add_to_forged_balance(
                    prev_generator_id as i64,
                    fee,
                    target_height
                ).await.map_err(|e| anyhow::anyhow!("{}", e))?;
            } else {
                debug!("Previous block not found at height {}", target_height);
            }
        }

        Ok(total_back_fees)
    }

    /// Calculate back fees for a single transaction
    ///
    /// Reference: Java TransactionType.getBackFees()
    /// - Asset issuance: [fee * 3/10, fee * 2/10, fee * 1/10]
    /// - Currency issuance: [fee * 3/10, fee * 2/10, fee * 1/10]
    /// - Other types: empty (no back fees)
    fn get_back_fees(&self, tx: &Transaction) -> [i64; 3] {
        // Asset issuance (ColoredCoins, subtype=0) or Currency issuance (MonetarySystem, subtype=0)
        let is_asset_issuance = tx.type_id == TransactionType::ColoredCoins && tx.subtype == 0;
        let is_currency_issuance = tx.type_id == TransactionType::MonetarySystem && tx.subtype == 0;

        if is_asset_issuance || is_currency_issuance {
            let fee = tx.fee as i64;
            [
                fee * 3 / 10,  // 30% to previous block generator
                fee * 2 / 10,  // 20% to 2 blocks ago generator
                fee / 10,      // 10% to 3 blocks ago generator
            ]
        } else {
            [0; 3]
        }
    }
}
