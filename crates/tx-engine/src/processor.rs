//! Transaction Processor
//!
//! 核心交易处理逻辑，完全对齐 Java NRCS TransactionType.apply() 框架：
//!
//! Java 基类 apply() 统一逻辑：
//!   1. senderAccount.addToBalance(event, txId, -amountNQT, -feeNQT)
//!   2. if (recipientAccount != null) recipientAccount.addToBalanceAndUnconfirmedBalance(event, txId, amountNQT)
//!   3. applyAttachment(transaction, senderAccount, recipientAccount)
//!
//! 各交易类型只需实现 applyAttachment() 的特定逻辑

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info};

use blockchain_types::*;
use blockchain_types::prelude::Transaction;
use blockchain_types::prelude::Account;
use orm::{
    AccountRepository, AccountAssetRepository, TransactionRepository,
    AccountGuaranteedBalanceRepository, AccountLedgerRepository,
    AssetRepository, AssetTransferRepository, RepositoryError, TransactionModel,
    models::AccountLedgerModel, models::AssetModel, models::AccountAssetModel,
    models::AssetTransferModel
};
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

#[allow(dead_code)]
mod ledger_event {
    pub const BLOCK_GENERATED: i16 = 1;
    pub const REJECT_PHASED_TRANSACTION: i16 = 2;
    pub const ORDINARY_PAYMENT: i16 = 3;
    pub const ACCOUNT_INFO: i16 = 4;
    pub const ALIAS_ASSIGNMENT: i16 = 5;
    pub const ALIAS_BUY: i16 = 6;
    pub const ALIAS_DELETE: i16 = 7;
    pub const ALIAS_SELL: i16 = 8;
    pub const ARBITRARY_MESSAGE: i16 = 9;
    pub const HUB_ANNOUNCEMENT: i16 = 10;
    pub const PHASING_VOTE_CASTING: i16 = 11;
    pub const POLL_CREATION: i16 = 12;
    pub const VOTE_CASTING: i16 = 13;
    pub const ASSET_ASK_ORDER_CANCELLATION: i16 = 14;
    pub const ASSET_ASK_ORDER_PLACEMENT: i16 = 15;
    pub const ASSET_BID_ORDER_CANCELLATION: i16 = 16;
    pub const ASSET_BID_ORDER_PLACEMENT: i16 = 17;
    pub const ASSET_DIVIDEND_PAYMENT: i16 = 18;
    pub const ASSET_ISSUANCE: i16 = 19;
    pub const ASSET_TRADE: i16 = 20;
    pub const ASSET_TRANSFER: i16 = 21;
    pub const DIGITAL_GOODS_DELISTED: i16 = 22;
    pub const DIGITAL_GOODS_DELISTING: i16 = 23;
    pub const DIGITAL_GOODS_DELIVERY: i16 = 24;
    pub const DIGITAL_GOODS_FEEDBACK: i16 = 25;
    pub const DIGITAL_GOODS_LISTING: i16 = 26;
    pub const DIGITAL_GOODS_PRICE_CHANGE: i16 = 27;
    pub const DIGITAL_GOODS_PURCHASE: i16 = 28;
    pub const DIGITAL_GOODS_PURCHASE_EXPIRED: i16 = 29;
    pub const DIGITAL_GOODS_QUANTITY_CHANGE: i16 = 30;
    pub const DIGITAL_GOODS_REFUND: i16 = 31;
    pub const ACCOUNT_CONTROL_EFFECTIVE_BALANCE_LEASING: i16 = 32;
    pub const CURRENCY_DELETION: i16 = 33;
    pub const CURRENCY_DISTRIBUTION: i16 = 34;
    pub const CURRENCY_EXCHANGE: i16 = 35;
    pub const CURRENCY_EXCHANGE_BUY: i16 = 36;
    pub const CURRENCY_EXCHANGE_SELL: i16 = 37;
    pub const CURRENCY_ISSUANCE: i16 = 38;
    pub const CURRENCY_MINTING: i16 = 39;
    pub const CURRENCY_OFFER_EXPIRED: i16 = 40;
    pub const CURRENCY_OFFER_REPLACED: i16 = 41;
    pub const CURRENCY_PUBLISH_EXCHANGE_OFFER: i16 = 42;
    pub const CURRENCY_RESERVE_CLAIM: i16 = 43;
    pub const CURRENCY_RESERVE_INCREASE: i16 = 44;
    pub const CURRENCY_TRANSFER: i16 = 45;
    pub const CURRENCY_UNDO_CROWDFUNDING: i16 = 46;
    pub const TAGGED_DATA_UPLOAD: i16 = 47;
    pub const TAGGED_DATA_EXTEND: i16 = 48;
    pub const ASSET_DELETE: i16 = 49;
    pub const TRANSACTION_FEE: i16 = 50;
    pub const SHUFFLING_REGISTRATION: i16 = 51;
    pub const SHUFFLING_PROCESSING: i16 = 52;
    pub const SHUFFLING_CANCELLATION: i16 = 53;
    pub const SHUFFLING_DISTRIBUTION: i16 = 54;
    pub const ACCOUNT_CONTROL_PHASING_ONLY: i16 = 55;
    pub const ACCOUNT_PROPERTY: i16 = 56;
    pub const ACCOUNT_PROPERTY_DELETE: i16 = 57;
    pub const COIN_EXCHANGE_ORDER_ISSUE: i16 = 58;
    pub const COIN_EXCHANGE_ORDER_CANCEL: i16 = 59;
    pub const COIN_EXCHANGE_TRADE: i16 = 60;
    pub const ASSET_INCREASE: i16 = 61;
    pub const ASSET_SET_PHASING_CONTROL: i16 = 62;
    pub const CONTRACT_REFERENCE_SET: i16 = 63;
    pub const CONTRACT_REFERENCE_DELETE: i16 = 64;
    pub const ASSET_PROPERTY_SET: i16 = 65;
    pub const ASSET_PROPERTY_DELETE: i16 = 66;
    pub const ASSET_LONG_VALUE_PROPERTY_SET: i16 = 67;
    pub const ACCOUNT_LONG_VALUE_PROPERTY_SET: i16 = 68;
}

#[allow(dead_code)]
mod ledger_holding {
    pub const UNCONFIRMED_NRCS_BALANCE: i16 = 1;
    pub const NRCS_BALANCE: i16 = 2;
    pub const UNCONFIRMED_ASSET_BALANCE: i16 = 3;
    pub const ASSET_BALANCE: i16 = 4;
    pub const UNCONFIRMED_CURRENCY_BALANCE: i16 = 5;
    pub const CURRENCY_BALANCE: i16 = 6;
}

fn get_ledger_event(tx: &Transaction) -> i16 {
    match tx.type_id {
        TransactionType::Payment => ledger_event::ORDINARY_PAYMENT,
        TransactionType::Messaging => match tx.subtype {
            0 => ledger_event::ARBITRARY_MESSAGE,
            1 => ledger_event::ALIAS_ASSIGNMENT,
            2 => ledger_event::POLL_CREATION,
            3 => ledger_event::VOTE_CASTING,
            4 => ledger_event::HUB_ANNOUNCEMENT,
            5 => ledger_event::ACCOUNT_INFO,
            6 => ledger_event::ALIAS_SELL,
            7 => ledger_event::ALIAS_BUY,
            8 => ledger_event::ALIAS_DELETE,
            9 => ledger_event::PHASING_VOTE_CASTING,
            10 => ledger_event::ACCOUNT_PROPERTY,
            11 => ledger_event::ACCOUNT_PROPERTY_DELETE,
            12 => ledger_event::ACCOUNT_LONG_VALUE_PROPERTY_SET,
            _ => ledger_event::ARBITRARY_MESSAGE,
        },
        TransactionType::ColoredCoins => match tx.subtype {
            0 => ledger_event::ASSET_ISSUANCE,
            1 => ledger_event::ASSET_TRANSFER,
            2 => ledger_event::ASSET_ASK_ORDER_PLACEMENT,
            3 => ledger_event::ASSET_BID_ORDER_PLACEMENT,
            4 => ledger_event::ASSET_ASK_ORDER_CANCELLATION,
            5 => ledger_event::ASSET_BID_ORDER_CANCELLATION,
            6 => ledger_event::ASSET_DIVIDEND_PAYMENT,
            7 => ledger_event::ASSET_DELETE,
            8 => ledger_event::ASSET_INCREASE,
            9 => ledger_event::ASSET_SET_PHASING_CONTROL,
            10 => ledger_event::ASSET_PROPERTY_SET,
            11 => ledger_event::ASSET_PROPERTY_DELETE,
            12 => ledger_event::ASSET_LONG_VALUE_PROPERTY_SET,
            _ => ledger_event::ASSET_ISSUANCE,
        },
        TransactionType::DigitalGoods => match tx.subtype {
            0 => ledger_event::DIGITAL_GOODS_LISTING,
            1 => ledger_event::DIGITAL_GOODS_DELISTING,
            2 => ledger_event::DIGITAL_GOODS_PRICE_CHANGE,
            3 => ledger_event::DIGITAL_GOODS_QUANTITY_CHANGE,
            4 => ledger_event::DIGITAL_GOODS_PURCHASE,
            5 => ledger_event::DIGITAL_GOODS_DELIVERY,
            6 => ledger_event::DIGITAL_GOODS_FEEDBACK,
            7 => ledger_event::DIGITAL_GOODS_REFUND,
            _ => ledger_event::DIGITAL_GOODS_LISTING,
        },
        TransactionType::AccountControl => match tx.subtype {
            0 => ledger_event::ACCOUNT_CONTROL_EFFECTIVE_BALANCE_LEASING,
            1 => ledger_event::ACCOUNT_CONTROL_PHASING_ONLY,
            _ => ledger_event::ACCOUNT_CONTROL_EFFECTIVE_BALANCE_LEASING,
        },
        TransactionType::MonetarySystem => match tx.subtype {
            0 => ledger_event::CURRENCY_ISSUANCE,
            1 => ledger_event::CURRENCY_RESERVE_INCREASE,
            2 => ledger_event::CURRENCY_RESERVE_CLAIM,
            3 => ledger_event::CURRENCY_TRANSFER,
            4 => ledger_event::CURRENCY_PUBLISH_EXCHANGE_OFFER,
            5 => ledger_event::CURRENCY_EXCHANGE_BUY,
            6 => ledger_event::CURRENCY_EXCHANGE_SELL,
            7 => ledger_event::CURRENCY_MINTING,
            8 => ledger_event::CURRENCY_DELETION,
            _ => ledger_event::CURRENCY_ISSUANCE,
        },
        TransactionType::Data => match tx.subtype {
            0 => ledger_event::TAGGED_DATA_UPLOAD,
            1 => ledger_event::TAGGED_DATA_EXTEND,
            _ => ledger_event::TAGGED_DATA_UPLOAD,
        },
        TransactionType::Shuffling => match tx.subtype {
            0 | 1 => ledger_event::SHUFFLING_REGISTRATION,
            _ => ledger_event::SHUFFLING_PROCESSING,
        },
        TransactionType::Aliases => match tx.subtype {
            0 => ledger_event::ALIAS_ASSIGNMENT,
            1 => ledger_event::ALIAS_SELL,
            2 => ledger_event::ALIAS_BUY,
            3 => ledger_event::ALIAS_DELETE,
            _ => ledger_event::ALIAS_ASSIGNMENT,
        },
        TransactionType::Voting => match tx.subtype {
            0 => ledger_event::POLL_CREATION,
            1 => ledger_event::VOTE_CASTING,
            2 => ledger_event::PHASING_VOTE_CASTING,
            _ => ledger_event::POLL_CREATION,
        },
        TransactionType::AccountProperty => match tx.subtype {
            10 => ledger_event::ACCOUNT_PROPERTY,
            11 => ledger_event::ACCOUNT_PROPERTY_DELETE,
            _ => ledger_event::ACCOUNT_PROPERTY,
        },
        TransactionType::CoinExchange => match tx.subtype {
            0 => ledger_event::COIN_EXCHANGE_ORDER_ISSUE,
            1 => ledger_event::COIN_EXCHANGE_ORDER_CANCEL,
            _ => ledger_event::COIN_EXCHANGE_ORDER_ISSUE,
        },
        TransactionType::LightContract => match tx.subtype {
            0 => ledger_event::CONTRACT_REFERENCE_SET,
            1 => ledger_event::CONTRACT_REFERENCE_DELETE,
            _ => ledger_event::CONTRACT_REFERENCE_SET,
        },
        _ => ledger_event::ORDINARY_PAYMENT,
    }
}

#[async_trait]
pub trait TransactionProcessor: Send + Sync {
    async fn validate(&self, tx: &Transaction) -> ProcessorResult<()>;
    async fn apply_unconfirmed(&self, tx: &Transaction) -> ProcessorResult<bool>;
    async fn rollback_unconfirmed(&self, tx: &Transaction) -> ProcessorResult<()>;
    async fn apply(&self, tx: &Transaction) -> ProcessorResult<()>;
    async fn execute(&self, tx: &Transaction) -> ProcessorResult<TxReceiptInfo>;
    fn set_current_block(&self, block_id: i64, height: i32, timestamp: i32);

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
    account_asset_repo: Arc<dyn AccountAssetRepository>,
    asset_repo: Arc<dyn AssetRepository>,
    asset_transfer_repo: Arc<dyn AssetTransferRepository>,
    tx_repo: Arc<dyn TransactionRepository>,
    guaranteed_balance_repo: Arc<dyn AccountGuaranteedBalanceRepository>,
    ledger_repo: Arc<dyn AccountLedgerRepository>,
    current_block_id: std::sync::RwLock<i64>,
    current_height: std::sync::RwLock<i32>,
    current_timestamp: std::sync::RwLock<i32>,
}

impl DatabaseTransactionProcessor {
    pub fn new(
        account_repo: Arc<dyn AccountRepository>,
        account_asset_repo: Arc<dyn AccountAssetRepository>,
        asset_repo: Arc<dyn AssetRepository>,
        asset_transfer_repo: Arc<dyn AssetTransferRepository>,
        tx_repo: Arc<dyn TransactionRepository>,
        guaranteed_balance_repo: Arc<dyn AccountGuaranteedBalanceRepository>,
        ledger_repo: Arc<dyn AccountLedgerRepository>,
    ) -> Self {
        Self {
            account_repo,
            account_asset_repo,
            asset_repo,
            asset_transfer_repo,
            tx_repo,
            guaranteed_balance_repo,
            ledger_repo,
            current_block_id: std::sync::RwLock::new(0),
            current_height: std::sync::RwLock::new(0),
            current_timestamp: std::sync::RwLock::new(0),
        }
    }

    pub fn set_current_height(&self, height: i32) {
        *self.current_height.write().unwrap() = height;
    }

    fn get_current_block_id(&self) -> i64 {
        *self.current_block_id.read().unwrap()
    }

    fn get_current_height(&self) -> i32 {
        *self.current_height.read().unwrap()
    }

    fn get_current_timestamp(&self) -> i32 {
        *self.current_timestamp.read().unwrap()
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

    async fn get_account_balance(&self, account_id: i64) -> Option<i64> {
        match self.account_repo.find_by_account_id(account_id).await {
            Ok(Some(model)) => Some(model.balance),
            _ => None,
        }
    }

    async fn get_asset_balance(&self, account_id: i64, asset_id: i64) -> Option<i64> {
        match self.account_asset_repo.find_by_account_and_asset(account_id, asset_id).await {
            Ok(Some(model)) => Some(model.quantity),
            _ => None,
        }
    }
}

#[async_trait]
impl TransactionProcessor for DatabaseTransactionProcessor {
    /// Phase 1: Pre-deduct unconfirmed balance (double-spend detection)
    ///
    /// Reference: Java TransactionType.applyUnconfirmed()
    /// - totalAmountNQT = amountNQT + feeNQT
    /// - Check sender unconfirmed balance >= totalAmountNQT
    /// - Deduct totalAmountNQT from unconfirmed balance
    /// - Return false if insufficient (double-spend!)
    async fn apply_unconfirmed(&self, tx: &Transaction) -> ProcessorResult<bool> {
        let sender_id = tx.sender_id;

        let total = tx.amount.checked_add(tx.fee)
            .ok_or_else(|| ProcessorError::Validation("amount+fee overflow".to_string()))?;

        let account = self.get_account(sender_id).await?;

        if account.unconfirmed_balance < total {
            tracing::warn!(
                "Double-spend detected! tx={}, sender={}, have={}, need={}",
                tx.id, sender_id, account.unconfirmed_balance, total
            );
            return Ok(false);
        }

        self.account_repo.add_to_unconfirmed_balance(sender_id as i64, -(total as i64)).await?;

        debug!("Pre-deducted tx={} from sender={}, amount={}", tx.id, sender_id, total);
        Ok(true)
    }

    /// Rollback pre-deduction (restore unconfirmed balance)
    async fn rollback_unconfirmed(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id;
        let total = tx.amount.checked_add(tx.fee)
            .ok_or_else(|| ProcessorError::Validation("amount+fee overflow".to_string()))?;

        self.account_repo.add_to_unconfirmed_balance(sender_id as i64, total as i64).await?;
        debug!("Rolled back pre-deduction for tx={}, sender={}", tx.id, sender_id);
        Ok(())
    }

    async fn validate(&self, tx: &Transaction) -> ProcessorResult<()> {
        tx.validate_basic()?;

        if !tx.verify_signature() {
            return Err(ProcessorError::Validation("signature verification failed".to_string()));
        }

        let account = self.get_account(tx.sender_id).await?;
        let required = tx.amount + tx.fee;
        if account.balance < required {
            return Err(ProcessorError::InsufficientBalance {
                have: account.balance,
                need: required,
            });
        }

        Ok(())
    }

    /// Phase 2: Execute transaction officially
    ///
    /// Reference: Java TransactionType.apply() - UNIFIED framework for ALL transaction types:
    ///
    ///   1. senderAccount.addToBalance(event, txId, -amountNQT, -feeNQT)
    ///      -> Deducts amount+fee from confirmed balance
    ///      -> Updates guaranteed balance
    ///      -> Logs ledger entries (TRANSACTION_FEE for fee, event for amount)
    ///
    ///   2. if (recipientAccount != null)
    ///      recipientAccount.addToBalanceAndUnconfirmedBalance(event, txId, amountNQT)
    ///      -> Adds amount to both confirmed and unconfirmed balance
    ///      -> Logs ledger entry (event for amount credit)
    ///
    ///   3. applyAttachment(transaction, senderAccount, recipientAccount)
    ///      -> Type-specific logic (create asset, transfer asset, etc.)
    ///
    async fn apply(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id as i64;
        let recipient_id = tx.recipient_id.unwrap_or(0);
        let amount_nqt = tx.amount as i64;
        let fee_nqt = tx.fee as i64;

        // === Step 1: senderAccount.addToBalance(event, txId, -amountNQT, -feeNQT) ===
        // This deducts -(amountNQT + feeNQT) from confirmed balance
        let total = amount_nqt + fee_nqt;
        if total != 0 {
            self.account_repo.add_to_balance(sender_id, -total).await?;
        }

        // === Step 2: if (recipientAccount != null) recipientAccount.addToBalanceAndUnconfirmedBalance(event, txId, amountNQT) ===
        if recipient_id != 0 && amount_nqt > 0 {
            self.account_repo.get_or_create(recipient_id as i64).await?;
            self.account_repo.add_to_balance_and_unconfirmed(recipient_id as i64, amount_nqt).await?;
        }

        // === Step 3: applyAttachment() - type-specific logic ===
        self.apply_attachment(tx).await?;

        // === Step 4: Update guaranteed balance ===
        // Reference: Java Account.addToBalance() calls addToGuaranteedBalance(totalAmountNQT)
        //   where totalAmountNQT = amountNQT + feeNQT
        //   addToGuaranteedBalance only records when totalAmountNQT > 0 (balance increases)
        //
        // For sender: totalAmountNQT = -(amount + fee) → negative → NOT recorded
        // For recipient: totalAmountNQT = +amount → positive → RECORDED
        self.update_guaranteed_balance_for_recipient(tx).await?;

        // === Step 5: Log ledger entries ===
        self.log_ledger_entry(tx).await?;

        Ok(())
    }

    async fn execute(&self, tx: &Transaction) -> ProcessorResult<TxReceiptInfo> {
        use chrono::Utc;

        self.apply(tx).await?;

        let tx_model = TransactionModel::from_domain(tx)?;
        self.tx_repo.insert(&tx_model).await?;

        let receipt_info = TxReceiptInfo {
            transaction_id: tx.id,
            status: TxStatus::Success,
            block_height: Some(tx.height),
            gas_used: 0,
            logs: vec![],
            contract_address: None,
            executed_at: Utc::now().timestamp() as Timestamp,
        };

        Ok(receipt_info)
    }

    fn set_current_block(&self, block_id: i64, height: i32, timestamp: i32) {
        *self.current_block_id.write().unwrap() = block_id;
        *self.current_height.write().unwrap() = height;
        *self.current_timestamp.write().unwrap() = timestamp;
    }
}

impl DatabaseTransactionProcessor {
    /// applyAttachment() - type-specific logic
    ///
    /// Reference: Java TransactionType.applyAttachment()
    /// Each transaction type has its own attachment processing logic.
    async fn apply_attachment(&self, tx: &Transaction) -> ProcessorResult<()> {
        match tx.type_id {
            TransactionType::Payment => {
                // Java: if recipient is null, transfer amount to Genesis account
                // (handled by base class apply, nothing extra in applyAttachment)
            }

            TransactionType::ColoredCoins => {
                self.apply_colored_coins_attachment(tx).await?;
            }

            TransactionType::MonetarySystem => {
                self.apply_monetary_system_attachment(tx).await?;
            }

            TransactionType::AccountControl => {
                // Java: EFFECTIVE_BALANCE_LEASING -> leaseEffectiveBalance()
                //        SET_PHASING_ONLY -> AccountPhasingOnly.set()
                // Both are no-ops for now (future implementation)
            }

            TransactionType::DigitalGoods => {
                // Java: Various DGS operations
                // All are no-ops for now (future implementation)
            }

            TransactionType::Data => {
                // Java: TAGGED_DATA_UPLOAD -> TaggedData.add()
                //        TAGGED_DATA_EXTEND -> TaggedData.extend()
                // No-ops for now (future implementation)
            }

            TransactionType::Messaging => {
                // Java: Various messaging operations (alias, poll, vote, etc.)
                // No-ops for now (future implementation)
            }

            TransactionType::Shuffling => {
                // Java: Various shuffling operations
                // No-ops for now (future implementation)
            }

            TransactionType::Aliases => {
                // Java: Alias operations
                // No-ops for now (future implementation)
            }

            TransactionType::Voting => {
                // Java: Voting operations
                // No-ops for now (future implementation)
            }

            TransactionType::AccountProperty => {
                // Java: Account property operations
                // No-ops for now (future implementation)
            }

            TransactionType::CoinExchange => {
                // Java: Coin exchange operations
                // No-ops for now (future implementation)
            }

            TransactionType::LightContract => {
                // Java: CONTRACT_REFERENCE_SET -> ContractReference.setContractReference()
                //        CONTRACT_REFERENCE_DELETE -> ContractReference.deleteContractReference()
                // No-ops for now (future implementation)
            }

            _ => {}
        }

        Ok(())
    }

    /// ColoredCoins attachment processing
    ///
    /// Reference: Java TransactionTypeAsset
    async fn apply_colored_coins_attachment(&self, tx: &Transaction) -> ProcessorResult<()> {
        match tx.subtype {
            0 => {
                self.apply_asset_issuance(tx).await?;
            }

            1 => {
                self.apply_asset_transfer(tx).await?;
            }

            2..=12 => {
                debug!("ColoredCoins subtype {} processed (stub)", tx.subtype);
            }

            _ => {}
        }

        Ok(())
    }

    /// ASSET_ISSUANCE: Create asset and add to issuer's account
    ///
    /// Reference: Java TransactionTypeAsset.ASSET_ISSUANCE.applyAttachment()
    ///   Asset.addAsset(transaction, attachment);
    ///   senderAccount.addToAssetAndUnconfirmedAssetBalanceQNT(event, assetId, assetId, quantityQNT);
    async fn apply_asset_issuance(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id;
        let asset_id = tx.id as i64;
        let current_height = self.get_current_height();

        let asset = AssetModel {
            db_id: 0,
            id: asset_id,
            account_id: sender_id as i64,
            name: format!("Asset_{}", asset_id),
            description: None,
            quantity: tx.amount as i64,
            decimals: 0,
            has_control_phasing: false,
            initial_quantity: tx.amount as i64,
            height: current_height,
            latest: true,
        };

        self.asset_repo.insert(&asset).await?;

        let account_asset = AccountAssetModel {
            db_id: 0,
            account_id: sender_id as i64,
            asset_id,
            quantity: tx.amount as i64,
            unconfirmed_quantity: tx.amount as i64,
            height: current_height,
            latest: true,
        };

        self.account_asset_repo.insert(&account_asset).await?;

        info!("Asset issued: id={} owner={} quantity={} height={}",
              asset_id, sender_id, tx.amount, current_height);

        Ok(())
    }

    /// ASSET_TRANSFER: Transfer asset between accounts
    ///
    /// Reference: Java TransactionTypeAsset.ASSET_TRANSFER.applyAttachment()
    ///   senderAccount.addToAssetBalanceQNT(event, txId, assetId, -quantityQNT);
    ///   recipientAccount.addToAssetAndUnconfirmedAssetBalanceQNT(event, txId, assetId, quantityQNT);
    ///   AssetTransfer.addAssetTransfer(transaction, attachment);
    async fn apply_asset_transfer(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id;
        let recipient_id = tx.recipient_id.unwrap_or(0);
        let asset_id = tx.id as i64;
        let quantity = tx.amount as i64;
        let current_height = self.get_current_height();

        self.account_asset_repo.decrease_quantity(sender_id as i64, asset_id, quantity).await?;

        if recipient_id != 0 {
            self.account_repo.get_or_create(recipient_id as i64).await?;

            match self.account_asset_repo.find_by_account_and_asset(recipient_id as i64, asset_id).await? {
                Some(_) => {
                    self.account_asset_repo.increase_quantity(recipient_id as i64, asset_id, quantity).await?;
                }
                None => {
                    let new_account_asset = AccountAssetModel {
                        db_id: 0,
                        account_id: recipient_id as i64,
                        asset_id,
                        quantity,
                        unconfirmed_quantity: quantity,
                        height: current_height,
                        latest: true,
                    };
                    self.account_asset_repo.insert(&new_account_asset).await?;
                }
            }

            let transfer = AssetTransferModel::new(
                tx.id as i64,
                asset_id,
                sender_id as i64,
                recipient_id as i64,
                quantity,
                self.get_current_timestamp(),
                current_height,
            );
            self.asset_transfer_repo.insert(&transfer).await?;
        }

        info!("Asset transferred: asset_id={} from={} to={} quantity={}",
              asset_id, sender_id, recipient_id, quantity);

        Ok(())
    }

    /// MonetarySystem attachment processing
    ///
    /// Reference: Java TransactionTypeCurrency
    async fn apply_monetary_system_attachment(&self, tx: &Transaction) -> ProcessorResult<()> {
        debug!("MonetarySystem subtype {} processed (stub)", tx.subtype);
        Ok(())
    }

    /// Update guaranteed balance for recipient (balance increase)
    ///
    /// Reference: Java Account.addToGuaranteedBalance(long amountNQT):
    ///   if (amountNQT <= 0) return;
    ///   AccountGuaranteedBalanceRepository.save(accountId, additions, height);
    ///
    /// Called by addToBalance() and addToBalanceAndUnconfirmedBalance()
    /// with totalAmountNQT = amountNQT + feeNQT.
    /// Only records when totalAmountNQT > 0 (i.e., balance increases).
    ///
    /// In practice:
    /// - Sender: totalAmountNQT = -(amount + fee) → negative → skip
    /// - Recipient: totalAmountNQT = +amount → positive → record
    async fn update_guaranteed_balance_for_recipient(&self, tx: &Transaction) -> ProcessorResult<()> {
        let recipient_id = tx.recipient_id.unwrap_or(0);
        let amount_nqt = tx.amount as i64;

        if recipient_id != 0 && amount_nqt > 0 {
            let current_height = self.get_current_height();
            self.guaranteed_balance_repo.upsert_additions(
                recipient_id as i64,
                current_height,
                amount_nqt
            ).await?;
        }

        Ok(())
    }

    /// Log ledger entries - COMPLETE implementation aligned with Java NRCS
    ///
    /// Reference: Java Account.addToBalance() + addToBalanceAndUnconfirmedBalance()
    ///   + Account.addToAssetBalanceQNT() (for asset transactions)
    ///   + Account.addToCurrencyUnits() (for currency transactions)
    ///
    /// Java LedgerEntry constructor:
    ///   new LedgerEntry(event, eventId, accountId, holding, holdingId, change, balance)
    ///   Block block = Blockchain.getInstance().getLastBlock();
    ///   this.setBlockId(block.getId());
    ///   this.setHeight(block.getHeight());
    ///   this.setTimestamp(block.getTimestamp());
    ///
    /// Java addToBalance() writes 2 entries for sender:
    ///   1. Fee: event=TRANSACTION_FEE(50), holding=NRCS_BALANCE(2), change=-feeNQT
    ///      balance = this.getBalance() - amountNQT (before amount deducted)
    ///   2. Amount: event=event_type, holding=NRCS_BALANCE(2), change=-amountNQT
    ///      balance = this.getBalance() (after all deductions)
    ///
    /// Java addToBalanceAndUnconfirmedBalance() writes for recipient:
    ///   1. Amount: event=event_type, holding=NRCS_BALANCE(2), change=+amountNQT
    ///      balance = this.getBalance()
    ///
    /// Java addToAssetBalanceQNT() writes:
    ///   1. Asset: event=event_type, holding=ASSET_BALANCE(4), holdingId=assetId, change=quantityQNT
    ///      balance = assetBalance
    ///
    /// Java addToCurrencyUnits() writes:
    ///   1. Currency: event=event_type, holding=CURRENCY_BALANCE(6), holdingId=currencyId, change=units
    ///      balance = currencyUnits
    async fn log_ledger_entry(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id as i64;
        let recipient_id = tx.recipient_id.unwrap_or(0);
        let amount_nqt = tx.amount as i64;
        let fee_nqt = tx.fee as i64;
        let event = get_ledger_event(tx);
        let current_block_id = self.get_current_block_id();
        let current_height = self.get_current_height();
        let current_timestamp = self.get_current_timestamp();

        // Sender balance AFTER deduction (balance already includes -(amountNQT + feeNQT))
        let sender_balance_after = self.get_account_balance(sender_id).await.unwrap_or(0);

        // === SENDER entries (from addToBalance) ===
        // Reference: Java line 1069-1074:
        //   if (feeNQT != 0) logEntry(TRANSACTION_FEE, ..., feeNQT, balance - amountNQT)
        //   if (amountNQT != 0) logEntry(event, ..., amountNQT, balance)

        if fee_nqt != 0 {
            // Fee entry: change=-feeNQT, balance=senderBalance-amountNQT
            let entry = AccountLedgerModel {
                db_id: 0,
                account_id: sender_id,
                event_type: ledger_event::TRANSACTION_FEE,
                event_id: tx.id as i64,
                holding_type: ledger_holding::NRCS_BALANCE,
                holding_id: None,
                change: -fee_nqt,
                balance: sender_balance_after + amount_nqt,
                block_id: current_block_id,
                height: current_height,
                timestamp: current_timestamp,
            };
            self.ledger_repo.insert(&entry).await?;
        }

        if amount_nqt != 0 {
            // Amount entry: change=-amountNQT, balance=senderBalance
            let entry = AccountLedgerModel {
                db_id: 0,
                account_id: sender_id,
                event_type: event,
                event_id: tx.id as i64,
                holding_type: ledger_holding::NRCS_BALANCE,
                holding_id: None,
                change: -amount_nqt,
                balance: sender_balance_after,
                block_id: current_block_id,
                height: current_height,
                timestamp: current_timestamp,
            };
            self.ledger_repo.insert(&entry).await?;
        }

        // === RECIPIENT entries (from addToBalanceAndUnconfirmedBalance) ===
        // Reference: Java line 1118-1123:
        //   if (amountNQT != 0) logEntry(event, ..., amountNQT, balance)

        if recipient_id != 0 && amount_nqt > 0 {
            let recipient_balance_after = self.get_account_balance(recipient_id as i64).await.unwrap_or(0);

            let entry = AccountLedgerModel {
                db_id: 0,
                account_id: recipient_id as i64,
                event_type: event,
                event_id: tx.id as i64,
                holding_type: ledger_holding::NRCS_BALANCE,
                holding_id: None,
                change: amount_nqt,
                balance: recipient_balance_after,
                block_id: current_block_id,
                height: current_height,
                timestamp: current_timestamp,
            };
            self.ledger_repo.insert(&entry).await?;
        }

        // === ASSET/CURRENCY entries (from applyAttachment) ===
        // For ASSET_TRANSFER: write asset balance changes with ASSET_BALANCE(4) and assetId
        // For CURRENCY_TRANSFER: write currency balance changes with CURRENCY_BALANCE(6) and currencyId

        match tx.type_id {
            TransactionType::ColoredCoins => {
                let asset_id = tx.id as i64;
                let quantity = tx.amount as i64;

                match tx.subtype {
                    0 => { // ASSET_ISSUANCE: issuer gets assets
                        let asset_balance_after = self.get_asset_balance(sender_id, asset_id).await.unwrap_or(0);
                        let entry = AccountLedgerModel {
                            db_id: 0,
                            account_id: sender_id,
                            event_type: ledger_event::ASSET_ISSUANCE,
                            event_id: tx.id as i64,
                            holding_type: ledger_holding::ASSET_BALANCE,
                            holding_id: Some(asset_id),
                            change: quantity,
                            balance: asset_balance_after,
                            block_id: current_block_id,
                            height: current_height,
                            timestamp: current_timestamp,
                        };
                        self.ledger_repo.insert(&entry).await?;
                    }
                    1 => { // ASSET_TRANSFER: sender loses, receiver gains
                        // Sender: decrease asset balance
                        let sender_asset_balance = self.get_asset_balance(sender_id, asset_id).await.unwrap_or(0);
                        let entry = AccountLedgerModel {
                            db_id: 0,
                            account_id: sender_id,
                            event_type: ledger_event::ASSET_TRANSFER,
                            event_id: tx.id as i64,
                            holding_type: ledger_holding::ASSET_BALANCE,
                            holding_id: Some(asset_id),
                            change: -quantity,
                            balance: sender_asset_balance,
                            block_id: current_block_id,
                            height: current_height,
                            timestamp: current_timestamp,
                        };
                        self.ledger_repo.insert(&entry).await?;

                        // Receiver: increase asset balance
                        if recipient_id != 0 {
                                let recv_asset_balance = self.get_asset_balance(recipient_id as i64, asset_id).await.unwrap_or(0);
                                let entry = AccountLedgerModel {
                                    db_id: 0,
                                    account_id: recipient_id as i64,
                                    event_type: ledger_event::ASSET_TRANSFER,
                                    event_id: tx.id as i64,
                                    holding_type: ledger_holding::ASSET_BALANCE,
                                    holding_id: Some(asset_id),
                                    change: quantity,
                                    balance: recv_asset_balance,
                                    block_id: current_block_id,
                                    height: current_height,
                                    timestamp: current_timestamp,
                                };
                                self.ledger_repo.insert(&entry).await?;
                            }
                        }
                    }
                    _ => {}
                }
            }
            TransactionType::MonetarySystem => {
                // Currency transfer with CURRENCY_BALANCE(6)
                let currency_id = tx.id as i64;
                let units = tx.amount as i64;

                if units != 0 && tx.subtype == 3 { // CURRENCY_TRANSFER
                    // Sender: decrease currency units
                    let entry = AccountLedgerModel {
                        db_id: 0,
                        account_id: sender_id,
                        event_type: ledger_event::CURRENCY_TRANSFER,
                        event_id: tx.id as i64,
                        holding_type: ledger_holding::CURRENCY_BALANCE,
                        holding_id: Some(currency_id),
                        change: -units,
                        balance: 0, // Would need to query actual currency balance
                        block_id: current_block_id,
                        height: current_height,
                        timestamp: current_timestamp,
                    };
                    self.ledger_repo.insert(&entry).await?;

                    // Receiver: increase currency units
                    if recipient_id != 0 {
                        let entry = AccountLedgerModel {
                            db_id: 0,
                            account_id: recipient_id as i64,
                            event_type: ledger_event::CURRENCY_TRANSFER,
                            event_id: tx.id as i64,
                            holding_type: ledger_holding::CURRENCY_BALANCE,
                            holding_id: Some(currency_id),
                            change: units,
                            balance: 0,
                            block_id: current_block_id,
                            height: current_height,
                            timestamp: current_timestamp,
                        };
                        self.ledger_repo.insert(&entry).await?;
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }
}
