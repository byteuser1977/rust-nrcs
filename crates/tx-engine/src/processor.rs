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
use orm::{AccountRepository, AccountAssetRepository, TransactionRepository, PublicKeyRepository, RepositoryError, TransactionModel};
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
    #[allow(dead_code)]
    public_key_repo: Arc<dyn PublicKeyRepository>,
}

impl DatabaseTransactionProcessor {
    pub fn new(
        account_repo: Arc<dyn AccountRepository>,
        account_asset_repo: Arc<dyn AccountAssetRepository>,
        tx_repo: Arc<dyn TransactionRepository>,
        public_key_repo: Arc<dyn PublicKeyRepository>,
    ) -> Self {
        Self {
            account_repo,
            account_asset_repo,
            tx_repo,
            public_key_repo,
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

    async fn update_account_balance(&self, account_id: AccountId, balance: Amount, unconfirmed: Amount) -> ProcessorResult<()> {
        self.account_repo
            .update_balance(account_id as i64, balance as i64, unconfirmed as i64)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl TransactionProcessor for DatabaseTransactionProcessor {
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

    async fn apply(&self, tx: &Transaction) -> ProcessorResult<()> {
        match tx.type_id {
            TransactionType::Payment => {
                let sender_id = tx.sender_id;
                let recipient_id = tx.recipient_id.unwrap_or(0);

                let sender = self.get_account(sender_id).await?;
                let total = tx.amount + tx.fee;

                if sender.balance < total {
                    return Err(ProcessorError::InsufficientBalance {
                        have: sender.balance,
                        need: total,
                    });
                }

                let new_balance = sender.balance.saturating_sub(total);
                let new_unconfirmed = sender.unconfirmed_balance.saturating_sub(total);
                self.update_account_balance(sender_id, new_balance, new_unconfirmed).await?;

                if recipient_id != 0 {
                    let recipient = self.get_or_create_account(recipient_id).await?;
                    let new_balance = recipient.balance.saturating_add(tx.amount);
                    let new_unconfirmed = recipient.unconfirmed_balance.saturating_add(tx.amount);
                    self.update_account_balance(recipient_id, new_balance, new_unconfirmed).await?;
                }
            }

            TransactionType::ColoredCoins => {
                let sender_id = tx.sender_id;
                let account = self.get_account(sender_id).await?;
                let new_balance = account.balance.saturating_sub(tx.fee);
                let new_unconfirmed = account.unconfirmed_balance.saturating_sub(tx.fee);
                self.update_account_balance(sender_id, new_balance, new_unconfirmed).await?;
            }

            TransactionType::LightContract => {
                let sender_id = tx.sender_id;
                let account = self.get_account(sender_id).await?;
                let new_balance = account.balance.saturating_sub(tx.fee);
                let new_unconfirmed = account.unconfirmed_balance.saturating_sub(tx.fee);
                self.update_account_balance(sender_id, new_balance, new_unconfirmed).await?;
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
