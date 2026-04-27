//! Transaction Processor
//!
//! 核心交易处理逻辑：
//! - 验证交易签名、余额、nonce、逻辑约束
//! - 应用交易到状态（修改账户余额、资产、合约存储）
//! - 返回交易收据

use async_trait::async_trait;
use std::sync::Arc;

use blockchain_types::*;
use blockchain_types::prelude::Transaction;
use blockchain_types::prelude::Account;
use orm::{AccountRepository, AccountAssetRepository, TransactionRepository, RepositoryError, TransactionModel};
use thiserror::Error;

use crate::types::{TxReceiptInfo, TxStatus};

#[derive(Debug, Error)]
pub enum ProcessorError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: Amount, need: Amount },

    #[error("account not found: {0}")]
    AccountNotFound(AccountId),

    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("blockchain error: {0}")]
    Blockchain(#[from] BlockchainError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type ProcessorResult<T> = std::result::Result<T, ProcessorError>;

#[async_trait]
pub trait TransactionProcessor: Send + Sync {
    async fn validate(&self, tx: &Transaction) -> ProcessorResult<()>;

    /// Phase 1: Pre-deduct (double-spend detection)
    /// Returns false if double-spending detected (insufficient unconfirmed balance)
    async fn apply_unconfirmed(&self, tx: &Transaction) -> ProcessorResult<bool>;

    /// Rollback pre-deduction
    async fn rollback_unconfirmed(&self, tx: &Transaction) -> ProcessorResult<()>;

    /// Phase 2: Execute transaction officially
    async fn apply(&self, tx: &Transaction) -> ProcessorResult<()>;
    async fn execute(&self, tx: &Transaction) -> ProcessorResult<TxReceiptInfo>;

    async fn validate_batch(&self, txs: &[Transaction]) -> ProcessorResult<()> {
        for tx in txs {
            self.validate(tx).await?;
        }
        Ok(())
    }

    async fn execute_batch(&self, txs: &[Transaction]) -> ProcessorResult<Vec<TxReceiptInfo>> {
        let mut receipts = Vec::new();
        for tx in txs {
            let receipt = self.execute(tx).await?;
            receipts.push(receipt);
        }
        Ok(receipts)
    }
}

pub struct DatabaseTransactionProcessor {
    account_repo: Arc<dyn AccountRepository>,
    #[allow(dead_code)]
    account_asset_repo: Arc<dyn AccountAssetRepository>,
    tx_repo: Arc<dyn TransactionRepository>,
}

impl DatabaseTransactionProcessor {
    pub fn new(
        account_repo: Arc<dyn AccountRepository>,
        account_asset_repo: Arc<dyn AccountAssetRepository>,
        tx_repo: Arc<dyn TransactionRepository>,
    ) -> Self {
        Self {
            account_repo,
            account_asset_repo,
            tx_repo,
        }
    }

    async fn get_account(&self, account_id: AccountId) -> ProcessorResult<Account> {
        let model = self.account_repo
            .find_by_account_id(account_id as i64)
            .await?
            .ok_or_else(|| ProcessorError::AccountNotFound(account_id))?;
        Ok(Account {
            id: model.id as AccountId,
            address: None,
            balance: model.balance as Amount,
            unconfirmed_balance: model.unconfirmed_balance as Amount,
            reserved_balance: 0,
            guaranteed_balance: 0,
            assets: Default::default(),
            properties: Default::default(),
            lease: None,
            created_at: 0,
            last_updated: 0,
            current_height: 0,
        })
    }

    #[allow(dead_code)]
    async fn get_or_create_account(&self, account_id: AccountId) -> ProcessorResult<Account> {
        match self.get_account(account_id).await {
            Ok(account) => Ok(account),
            Err(ProcessorError::AccountNotFound(_)) => {
                Ok(Account {
                    id: account_id,
                    address: None,
                    balance: 0,
                    unconfirmed_balance: 0,
                    reserved_balance: 0,
                    guaranteed_balance: 0,
                    assets: Default::default(),
                    properties: Default::default(),
                    lease: None,
                    created_at: 0,
                    last_updated: 0,
                    current_height: 0,
                })
            }
            Err(e) => Err(e),
        }
    }

    #[allow(dead_code)]
    async fn update_account_balance(&self, account_id: AccountId, balance: Amount, unconfirmed: Amount) -> ProcessorResult<()> {
        self.account_repo
            .update_balance(account_id as i64, balance as i64, unconfirmed as i64)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl TransactionProcessor for DatabaseTransactionProcessor {
    /// Phase 1: Pre-deduct unconfirmed balance (double-spend detection)
    ///
    /// Reference: Java TransactionType.applyUnconfirmed()
    /// - Checks UNCONFIRMED balance (not confirmed balance)
    /// - Deducts amount + fee from unconfirmed_balance
    /// - Returns false if insufficient funds (double-spend detected!)
    async fn apply_unconfirmed(&self, tx: &Transaction) -> ProcessorResult<bool> {
        let sender_id = tx.sender_id;

        let total = tx.amount.checked_add(tx.fee)
            .ok_or_else(|| ProcessorError::Validation("amount+fee overflow".to_string()))?;

        let account = self.get_account(sender_id).await?;

        // ✅ Core double-spend check: verify UNCONFIRMED balance
        // Java uses: senderAccount.getUnconfirmedBalance() < totalAmountNQT
        if account.unconfirmed_balance < total {
            tracing::warn!(
                "Double-spend detected! tx={}, sender={}, have={}, need={}",
                tx.id, sender_id, account.unconfirmed_balance, total
            );
            return Ok(false);  // Double-spend!
        }

        // Deduct from unconfirmed_balance only
        let delta = -(total as i64);
        self.account_repo.add_to_unconfirmed_balance(sender_id as i64, delta).await?;

        tracing::debug!(
            "Pre-deducted tx={} from sender={}, amount={}",
            tx.id, sender_id, total
        );

        Ok(true)
    }

    /// Rollback pre-deduction (restore unconfirmed balance)
    async fn rollback_unconfirmed(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id;

        let total = tx.amount.checked_add(tx.fee)
            .ok_or_else(|| ProcessorError::Validation("amount+fee overflow".to_string()))?;

        // Restore unconfirmed balance
        self.account_repo.add_to_unconfirmed_balance(
            sender_id as i64,
            total as i64  // Positive value to restore
        ).await?;

        tracing::debug!(
            "Rolled back pre-deduction for tx={}, sender={}",
            tx.id, sender_id
        );

        Ok(())
    }

    async fn validate(&self, tx: &Transaction) -> ProcessorResult<()> {
        tx.validate_basic()?;

        if !tx.verify_signature() {
            return Err(ProcessorError::Validation("signature verification failed".to_string()));
        }

        match tx.type_id {
            TransactionType::Payment => {
                let account = self.get_account(tx.sender_id).await?;
                let required = tx.amount + tx.fee;
                if account.balance < required {
                    return Err(ProcessorError::InsufficientBalance {
                        have: account.balance,
                        need: required,
                    });
                }
            }
            TransactionType::ColoredCoins => {
                let account = self.get_account(tx.sender_id).await?;
                if account.balance < tx.fee {
                    return Err(ProcessorError::InsufficientBalance {
                        have: account.balance,
                        need: tx.fee,
                    });
                }
            }
            TransactionType::LightContract => {
                let account = self.get_account(tx.sender_id).await?;
                if account.balance < tx.fee {
                    return Err(ProcessorError::InsufficientBalance {
                        have: account.balance,
                        need: tx.fee,
                    });
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Phase 2: Execute transaction officially (after pre-deduction succeeded)
    ///
    /// Reference: Java TransactionType.apply()
    /// - Deducts from CONFIRMED balance (balance field)
    /// - unconfirmed_balance already deducted in apply_unconfirmed phase
    /// - Uses INCREMENTAL updates (not direct value setting)
    /// - Uses CHECKED arithmetic to prevent silent overflow/underflow
    async fn apply(&self, tx: &Transaction) -> ProcessorResult<()> {
        match tx.type_id {
            TransactionType::Payment => {
                let sender_id = tx.sender_id;
                let recipient_id = tx.recipient_id.unwrap_or(0);

                // Calculate total deduction (amount + fee)
                let total = tx.amount.checked_add(tx.fee)
                    .ok_or_else(|| ProcessorError::Validation("amount+fee overflow".to_string()))?;

                // Sender: deduct from BALANCE only
                // Note: unconfirmed_balance already deducted in apply_unconfirmed()
                self.account_repo.add_to_balance(
                    sender_id as i64,
                    -(total as i64)
                ).await?;

                if recipient_id != 0 {
                    // Ensure recipient account exists
                    self.account_repo.get_or_create(recipient_id as i64).await?;

                    // Recipient: add to both balance and unconfirmed_balance
                    self.account_repo.add_to_balance_and_unconfirmed(
                        recipient_id as i64,
                        tx.amount as i64
                    ).await?;
                }

                tracing::debug!(
                    "Applied payment tx={}, sender={}, recipient={}, amount={}",
                    tx.id, sender_id, recipient_id, tx.amount
                );
            }

            TransactionType::ColoredCoins => {
                let sender_id = tx.sender_id;

                // Only deduct fee for colored coins
                self.account_repo.add_to_balance(sender_id as i64, -(tx.fee as i64)).await?;
                self.account_repo.add_to_unconfirmed_balance(sender_id as i64, -(tx.fee as i64)).await?;
            }

            TransactionType::LightContract => {
                let sender_id = tx.sender_id;

                // Only deduct fee for smart contract
                self.account_repo.add_to_balance(sender_id as i64, -(tx.fee as i64)).await?;
                self.account_repo.add_to_unconfirmed_balance(sender_id as i64, -(tx.fee as i64)).await?;
            }

            _ => {}
        }

        Ok(())
    }

    async fn execute(&self, tx: &Transaction) -> ProcessorResult<TxReceiptInfo> {
        use chrono::Utc;

        self.apply(tx).await?;

        let gas_used = match tx.type_id {
            TransactionType::Payment => 100_000,
            TransactionType::ColoredCoins => 200_000,
            TransactionType::LightContract => 500_000,
            _ => 100_000,
        };

        let tx_model = TransactionModel::from_domain(tx)?;
        self.tx_repo.insert(&tx_model).await?;

        let receipt_info = TxReceiptInfo {
            transaction_id: tx.id,
            status: TxStatus::Success,
            block_height: Some(tx.height),
            gas_used,
            logs: vec![],
            contract_address: None,
            executed_at: Utc::now().timestamp() as Timestamp,
        };

        Ok(receipt_info)
    }
}
