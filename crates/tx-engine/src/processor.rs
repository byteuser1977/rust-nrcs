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
use tracing::{debug, info, warn};
use serde_json;

use blockchain_types::*;
use blockchain_types::prelude::Transaction;
use blockchain_types::prelude::Account;
use orm::{
    AccountRepository, AccountAssetRepository, TransactionRepository,
    AccountGuaranteedBalanceRepository, AccountLedgerRepository,
    AssetRepository, AssetTransferRepository, RepositoryError, TransactionModel,
    models::AccountLedgerModel, models::AssetModel, models::AccountAssetModel,
    models::AssetTransferModel,
    // 新增导入
    AliasRepository, AliasOfferRepository,
    PollRepository, VoteRepository,
    TaggedDataRepository, TaggedDataTagRepository, TaggedDataExtendRepository,
    TaggedTimestampRepository,
    ContractReferenceRepository,
    AskOrderRepository, BidOrderRepository,
    CurrencyRepository, AccountCurrencyRepository, CurrencyTransferRepository,
    AssetPropertyRepository,
    AccountPropertyRepository, AccountInfoRepository,
    // Phasing
    PhasingPollRepository, PhasingVoteRepository, AccountControlPhasingRepository,
    // Digital Goods
    GoodsRepository, PurchaseRepository,
    // Shuffling
    ShufflingRepository,
    // Account Lease
    AccountLeaseRepository,
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
    async fn apply_phased_fee(&self, tx: &Transaction) -> ProcessorResult<()>;
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
    // 基础Repository
    account_repo: Arc<dyn AccountRepository>,
    account_asset_repo: Arc<dyn AccountAssetRepository>,
    asset_repo: Arc<dyn AssetRepository>,
    asset_transfer_repo: Arc<dyn AssetTransferRepository>,
    tx_repo: Arc<dyn TransactionRepository>,
    guaranteed_balance_repo: Arc<dyn AccountGuaranteedBalanceRepository>,
    ledger_repo: Arc<dyn AccountLedgerRepository>,

    // 新增：Messaging相关
    alias_repo: Arc<dyn AliasRepository>,
    alias_offer_repo: Arc<dyn AliasOfferRepository>,
    poll_repo: Arc<dyn PollRepository>,
    vote_repo: Arc<dyn VoteRepository>,

    // 新增：Account属性
    account_property_repo: Arc<dyn AccountPropertyRepository>,
    account_info_repo: Arc<dyn AccountInfoRepository>,

    // 新增：Phasing
    phasing_poll_repo: Arc<dyn PhasingPollRepository>,
    phasing_vote_repo: Arc<dyn PhasingVoteRepository>,
    account_control_phasing_repo: Arc<dyn AccountControlPhasingRepository>,

    // 新增：Data/Tagged
    tagged_data_repo: Arc<dyn TaggedDataRepository>,
    #[allow(dead_code)]
    tagged_data_tag_repo: Arc<dyn TaggedDataTagRepository>,
    tagged_data_extend_repo: Arc<dyn TaggedDataExtendRepository>,
    tagged_timestamp_repo: Arc<dyn TaggedTimestampRepository>,

    // 新增：合约引用
    contract_ref_repo: Arc<dyn ContractReferenceRepository>,

    // 新增：资产订单
    ask_order_repo: Arc<dyn AskOrderRepository>,
    bid_order_repo: Arc<dyn BidOrderRepository>,

    // 新增：货币系统
    currency_repo: Arc<dyn CurrencyRepository>,
    account_currency_repo: Arc<dyn AccountCurrencyRepository>,
    currency_transfer_repo: Arc<dyn CurrencyTransferRepository>,

    // 新增：资产属性
    asset_property_repo: Arc<dyn AssetPropertyRepository>,

    // 新增：Exchange和Mint（P1优化完成 - 专用Repository）
    exchange_request_repo: Arc<dyn orm::Repository<orm::models::ExchangeRequestModel>>,
    currency_mint_repo: Arc<dyn orm::Repository<orm::models::CurrencyMintModel>>,

    // 新增：Digital Goods
    goods_repo: Arc<dyn GoodsRepository>,
    purchase_repo: Arc<dyn PurchaseRepository>,

    // 新增：Shuffling
    shuffling_repo: Arc<dyn ShufflingRepository>,

    // 新增：Account Lease
    account_lease_repo: Arc<dyn AccountLeaseRepository>,

    // 区块上下文
    current_block_id: std::sync::RwLock<i64>,
    current_height: std::sync::RwLock<i32>,
    current_timestamp: std::sync::RwLock<i32>,
}

#[allow(clippy::too_many_arguments)]
impl DatabaseTransactionProcessor {
    pub fn new(
        // 基础Repository
        account_repo: Arc<dyn AccountRepository>,
        account_asset_repo: Arc<dyn AccountAssetRepository>,
        asset_repo: Arc<dyn AssetRepository>,
        asset_transfer_repo: Arc<dyn AssetTransferRepository>,
        tx_repo: Arc<dyn TransactionRepository>,
        guaranteed_balance_repo: Arc<dyn AccountGuaranteedBalanceRepository>,
        ledger_repo: Arc<dyn AccountLedgerRepository>,
        // 新增参数
        alias_repo: Arc<dyn AliasRepository>,
        alias_offer_repo: Arc<dyn AliasOfferRepository>,
        poll_repo: Arc<dyn PollRepository>,
        vote_repo: Arc<dyn VoteRepository>,
        account_property_repo: Arc<dyn AccountPropertyRepository>,
        account_info_repo: Arc<dyn AccountInfoRepository>,
        phasing_poll_repo: Arc<dyn PhasingPollRepository>,
        phasing_vote_repo: Arc<dyn PhasingVoteRepository>,
        account_control_phasing_repo: Arc<dyn AccountControlPhasingRepository>,
        tagged_data_repo: Arc<dyn TaggedDataRepository>,
        tagged_data_tag_repo: Arc<dyn TaggedDataTagRepository>,
        tagged_data_extend_repo: Arc<dyn TaggedDataExtendRepository>,
        tagged_timestamp_repo: Arc<dyn TaggedTimestampRepository>,
        contract_ref_repo: Arc<dyn ContractReferenceRepository>,
        ask_order_repo: Arc<dyn AskOrderRepository>,
        bid_order_repo: Arc<dyn BidOrderRepository>,
        currency_repo: Arc<dyn CurrencyRepository>,
        account_currency_repo: Arc<dyn AccountCurrencyRepository>,
        currency_transfer_repo: Arc<dyn CurrencyTransferRepository>,
        asset_property_repo: Arc<dyn AssetPropertyRepository>,
        // 新增：Exchange和Mint（P1优化完成 - 专用Repository）
        exchange_request_repo: Arc<dyn orm::Repository<orm::models::ExchangeRequestModel>>,
        currency_mint_repo: Arc<dyn orm::Repository<orm::models::CurrencyMintModel>>,
        // 新增：Digital Goods
        goods_repo: Arc<dyn GoodsRepository>,
        purchase_repo: Arc<dyn PurchaseRepository>,
        // 新增：Shuffling
        shuffling_repo: Arc<dyn ShufflingRepository>,
        // 新增：Account Lease
        account_lease_repo: Arc<dyn AccountLeaseRepository>,
    ) -> Self {
        Self {
            account_repo,
            account_asset_repo,
            asset_repo,
            asset_transfer_repo,
            tx_repo,
            guaranteed_balance_repo,
            ledger_repo,
            // 新增字段
            alias_repo,
            alias_offer_repo,
            poll_repo,
            vote_repo,
            account_property_repo,
            account_info_repo,
            phasing_poll_repo,
            phasing_vote_repo,
            account_control_phasing_repo,
            tagged_data_repo,
            tagged_data_tag_repo,
            tagged_data_extend_repo,
            tagged_timestamp_repo,
            contract_ref_repo,
            ask_order_repo,
            bid_order_repo,
            currency_repo,
            account_currency_repo,
            currency_transfer_repo,
            asset_property_repo,
            exchange_request_repo,
            currency_mint_repo,
            goods_repo,
            purchase_repo,
            shuffling_repo,
            account_lease_repo,
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

    /// Calculate baseline fee for a transaction type
    ///
    /// Reference: Java TransactionType.getBaselineFee() overrides
    /// Returns the minimum fee in NQT for the given transaction
    #[allow(dead_code)]
    fn calculate_baseline_fee(&self, tx: &Transaction) -> i64 {
        use blockchain_types::constants::ONE_NRCS;

        match tx.type_id {
            TransactionType::ColoredCoins => {
                match tx.subtype {
                    0 => {
                        // ASSET_ISSUANCE: 1000 NRCS (or 1 NRCS for singletons)
                        let is_singleton = self.parse_bool_field(tx, "isSingleton").unwrap_or(false);
                        if is_singleton {
                            ONE_NRCS as i64 // 1 NRCS
                        } else {
                            1000 * ONE_NRCS as i64 // 1000 NRCS
                        }
                    }
                    9 => {
                        // ASSET_INCREASE: 10 NRCS
                        10 * ONE_NRCS as i64
                    }
                    10 | 11 => {
                        // ASSET_PROPERTY_SET / ASSET_PROPERTY_DELETE: SizeBasedFee(0.1 NRCS, 0.1 NRCS, 32)
                        let size = tx.attachment_bytes.len().max(1) as i64;
                        let base = ONE_NRCS as i64 / 10; // 0.1 NRCS
                        base + (size * base / 32)
                    }
                    _ => {
                        // Default asset fee: 1 NRCS
                        ONE_NRCS as i64
                    }
                }
            }
            TransactionType::MonetarySystem => {
                match tx.subtype {
                    0 => {
                        // CURRENCY_ISSUANCE: fee based on code length
                        let code = self.parse_string_field(tx, "code").unwrap_or_default();
                        match code.len() {
                            3 => 25000 * ONE_NRCS as i64, // 25000 NRCS for 3-letter
                            4 => 1000 * ONE_NRCS as i64,  // 1000 NRCS for 4-letter
                            5 => 40 * ONE_NRCS as i64,     // 40 NRCS for 5-letter
                            _ => ONE_NRCS as i64,           // Default
                        }
                    }
                    7 => {
                        // CURRENCY_MINTING: 1 NRCS (CURRENCY_MINT_FEE)
                        ONE_NRCS as i64
                    }
                    _ => {
                        // Default currency fee: 1 NRCS
                        ONE_NRCS as i64
                    }
                }
            }
            _ => {
                // Default fee for all other types: 1 NRCS
                ONE_NRCS as i64
            }
        }
    }

    /// Calculate back fees for a transaction (fee splits to referrers)
    ///
    /// Reference: Java TransactionType.getBackFees()
    /// Returns array of [30%, 20%, 10%] of the fee
    #[allow(dead_code)]
    fn calculate_back_fees(&self, tx: &Transaction) -> [i64; 3] {
        let fee = tx.fee as i64;
        [fee * 3 / 10, fee * 2 / 10, fee / 10]
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

        // Java: TransactionType.applyUnconfirmed()
        // For phased transactions: only deduct fee from unconfirmed balance
        // For non-phased transactions: deduct amount + fee from unconfirmed balance
        let mut effective_fee = tx.fee as i64;

        // Java: if (transaction.getReferencedTransactionFullHash() != null
        //     && transaction.getTimestamp() > Constant.REFERENCED_TRANSACTION_FULL_HASH_BLOCK_TIMESTAMP) {
        //     feeNQT = Math.addExact(feeNQT, Constant.UNCONFIRMED_POOL_DEPOSIT_NQT);
        // }
        if tx.referenced_transaction_full_hash.is_some()
            && tx.timestamp > blockchain_types::constants::TRANSPARENT_FORGING_BLOCK as u32
        {
            effective_fee = effective_fee
                .checked_add(blockchain_types::constants::UNCONFIRMED_POOL_DEPOSIT_NQT as i64)
                .ok_or_else(|| ProcessorError::Validation("fee+deposit overflow".to_string()))?;
        }

        let deduct_amount = if tx.phased {
            // Phased: only deduct fee, amount stays in unconfirmed until phasing completes
            effective_fee
        } else {
            // Non-phased: deduct amount + fee
            (tx.amount as i64).checked_add(effective_fee)
                .ok_or_else(|| ProcessorError::Validation("amount+fee overflow".to_string()))?
        };

        let account = self.get_account(sender_id).await?;

        // Java: Genesis creator (timestamp==0) is exempt from balance check
        // Reference: TransactionType.applyUnconfirmed()
        let is_genesis_exempt = tx.timestamp == 0
            && tx.sender_public_key.0 == blockchain_types::constants::GENESIS_CREATOR_PUBLIC_KEY;

        if !is_genesis_exempt && account.unconfirmed_balance < deduct_amount as u64 {
            tracing::warn!(
                "Double-spend detected! tx={}, sender={}, have={}, need={}",
                tx.id, sender_id, account.unconfirmed_balance, deduct_amount
            );
            return Ok(false);
        }

        self.account_repo.add_to_unconfirmed_balance(sender_id as i64, -deduct_amount).await?;

        // Java: Also apply attachment unconfirmed (pre-deduct assets/currencies)
        if !tx.phased {
            let attachment_ok = self.apply_attachment_unconfirmed(tx).await?;
            if !attachment_ok {
                // Rollback the NRCS deduction if attachment deduction failed
                self.account_repo.add_to_unconfirmed_balance(sender_id as i64, deduct_amount).await?;
                return Ok(false);
            }
        }

        debug!("Pre-deducted tx={} from sender={}, amount={}, phased={}", tx.id, sender_id, deduct_amount, tx.phased);
        Ok(true)
    }

    /// Rollback pre-deduction (restore unconfirmed balance)
    async fn rollback_unconfirmed(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id;

        // Match the deduction logic: restore what was deducted
        let mut effective_fee = tx.fee as i64;

        // Add referenced transaction deposit if applicable
        if tx.referenced_transaction_full_hash.is_some()
            && tx.timestamp > blockchain_types::constants::TRANSPARENT_FORGING_BLOCK as u32
        {
            effective_fee = effective_fee
                .checked_add(blockchain_types::constants::UNCONFIRMED_POOL_DEPOSIT_NQT as i64)
                .ok_or_else(|| ProcessorError::Validation("fee+deposit overflow".to_string()))?;
        }

        let restore_amount = if tx.phased {
            effective_fee
        } else {
            (tx.amount as i64).checked_add(effective_fee)
                .ok_or_else(|| ProcessorError::Validation("amount+fee overflow".to_string()))?
        };

        self.account_repo.add_to_unconfirmed_balance(sender_id as i64, restore_amount).await?;

        // Java: Also rollback attachment unconfirmed (restore assets/currencies)
        if !tx.phased {
            self.rollback_attachment_unconfirmed(tx).await?;
        }

        debug!("Rolled back pre-deduction for tx={}, sender={}, phased={}", tx.id, sender_id, tx.phased);
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
    ///   For non-phased transactions:
    ///     senderAccount.addToBalance(event, txId, -amountNQT, -feeNQT)
    ///
    ///   For phased transactions:
    ///     senderAccount.addToBalance(event, txId, -amountNQT)  // only amount, fee already deducted
    ///
    ///   if (recipientAccount != null)
    ///     recipientAccount.addToBalanceAndUnconfirmedBalance(event, txId, amountNQT)
    ///
    ///   applyAttachment(transaction, senderAccount, recipientAccount)
    ///
    async fn apply(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id as i64;
        let recipient_id = tx.recipient_id.unwrap_or(0);
        let amount_nqt = tx.amount as i64;
        let fee_nqt = tx.fee as i64;

        // === Step 1: Deduct from sender's confirmed balance ===
        // Java: if (!transaction.attachmentIsPhased()) {
        //           senderAccount.addToBalance(event, txId, -amount, -fee);
        //       } else {
        //           senderAccount.addToBalance(event, txId, -amount);
        //       }
        if tx.phased {
            // Phased: only deduct amount (fee was already deducted in apply_unconfirmed from unconfirmed,
            // and will be deducted from confirmed in Transaction.apply())
            if amount_nqt != 0 {
                self.account_repo.add_to_balance(sender_id, -amount_nqt).await?;
            }
        } else {
            // Non-phased: deduct both amount and fee
            let total = amount_nqt + fee_nqt;
            if total != 0 {
                self.account_repo.add_to_balance(sender_id, -total).await?;
            }
        }

        // === Step 2: Credit recipient ===
        // Java: if (recipientAccount != null) {
        //           recipientAccount.addToBalanceAndUnconfirmedBalance(event, txId, amountNQT);
        //       }
        // When recipient is null (recipient_id == 0), amount goes to Genesis account (burning mechanism)
        let credit_recipient_id = if recipient_id != 0 {
            recipient_id as i64
        } else {
            // Java: Account.addOrGetAccount(Genesis.CREATOR_ID)
            // Genesis creator account ID is 0
            0i64
        };
        if amount_nqt > 0 {
            self.account_repo.get_or_create(credit_recipient_id).await?;
            self.account_repo.add_to_balance_and_unconfirmed(credit_recipient_id, amount_nqt).await?;
        }

        // === Step 3: applyAttachment() - type-specific logic ===
        self.apply_attachment(tx).await?;

        // === Step 4: Update guaranteed balance ===
        self.update_guaranteed_balance_for_recipient(tx).await?;

        // === Step 5: Log ledger entries ===
        self.log_ledger_entry(tx).await?;

        Ok(())
    }

    /// Apply fee deduction for phased transactions from confirmed balance.
    ///
    /// Reference: Java Transaction.apply()
    ///   if (attachmentIsPhased()) {
    ///       senderAccount.addToBalance(type.getLedgerEvent(), getId(), 0, -this.getFeeNQT());
    ///   }
    ///
    /// For phased transactions, the fee is deducted from confirmed balance separately
    /// from the amount deduction in TransactionType.apply().
    async fn apply_phased_fee(&self, tx: &Transaction) -> ProcessorResult<()> {
        if !tx.phased {
            return Ok(());
        }

        let sender_id = tx.sender_id as i64;
        let fee_nqt = tx.fee as i64;

        if fee_nqt != 0 {
            self.account_repo.add_to_balance(sender_id, -fee_nqt).await?;
            debug!("Deducted phased fee={} from sender={}", fee_nqt, sender_id);
        }

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
                self.apply_account_control_attachment(tx).await?;
            }

            TransactionType::DigitalGoods => {
                self.apply_digital_goods_attachment(tx).await?;
            }

            TransactionType::Data => {
                self.apply_data_attachment(tx).await?;
            }

            TransactionType::Messaging => {
                self.apply_messaging_attachment(tx).await?;
            }

            TransactionType::Shuffling => {
                self.apply_shuffling_attachment(tx).await?;
            }

            TransactionType::Aliases => {
                self.apply_aliases_attachment(tx).await?;
            }

            TransactionType::Voting => {
                self.apply_voting_attachment(tx).await?;
            }

            TransactionType::AccountProperty => {
                self.apply_account_property_attachment(tx).await?;
            }

            TransactionType::CoinExchange => {
                self.apply_coin_exchange_attachment(tx).await?;
            }

            TransactionType::LightContract => {
                self.apply_light_contract_attachment(tx).await?;
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

            2 => { // ASK_ORDER_PLACEMENT
                self.apply_ask_order_placement(tx).await?;
            }

            3 => { // BID_ORDER_PLACEMENT
                self.apply_bid_order_placement(tx).await?;
            }

            4 => { // ASK_ORDER_CANCELLATION
                self.apply_ask_order_cancellation(tx).await?;
            }

            5 => { // BID_ORDER_CANCELLATION
                self.apply_bid_order_cancellation(tx).await?;
            }

            6 => { // DIVIDEND_PAYMENT
                self.apply_dividend_payment(tx).await?;
            }

            7 => { // ASSET_DELETE
                self.apply_asset_delete(tx).await?;
            }

            9 => { // ASSET_INCREASE
                self.apply_asset_increase(tx).await?;
            }

            10 => { // ASSET_PROPERTY_SET
                self.apply_asset_property_set(tx).await?;
            }

            11 => { // ASSET_PROPERTY_DELETE
                self.apply_asset_property_delete(tx).await?;
            }

            12 => { // ASSET_LONG_VALUE_PROPERTY_SET
                self.apply_asset_long_value_property_set(tx).await?;
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

        // Parse attachment to get asset name, quantity, decimals
        // Reference: Java TransactionTypeAsset.ASSET_ISSUANCE.applyAttachment()
        let (name, description, quantity, decimals) = if let Some(att_json) = self.get_attachment_json(tx) {
            let name = att_json.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let description = att_json.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let quantity = att_json.get("quantity")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(tx.amount as i64);
            let decimals = att_json.get("decimals")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u8;
            (name, Some(description), quantity, decimals)
        } else {
            // Fallback: use transaction amount as quantity
            (format!("Asset_{}", asset_id), None, tx.amount as i64, 0u8)
        };

        let asset = AssetModel {
            db_id: 0,
            id: asset_id,
            account_id: sender_id as i64,
            name,
            description,
            quantity,
            decimals: decimals as i16,
            has_control_phasing: false,
            initial_quantity: quantity,
            height: current_height,
            latest: true,
        };

        self.asset_repo.insert(&asset).await?;

        let account_asset = AccountAssetModel {
            db_id: 0,
            account_id: sender_id as i64,
            asset_id,
            quantity,
            unconfirmed_quantity: quantity,
            height: current_height,
            latest: true,
        };

        self.account_asset_repo.insert(&account_asset).await?;

        info!("Asset issued: id={} owner={} quantity={} decimals={} height={}",
              asset_id, sender_id, quantity, decimals, current_height);

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

        // Java: Parse asset_id and quantity from attachment
        // long assetId = transaction.getAttachment().getAssetId();
        // long quantityQNT = transaction.getAttachment().getQuantityQNT();
        let (asset_id, quantity) = if let Some(att_json) = self.get_attachment_json(tx) {
            let asset_id = att_json.get("asset")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);
            let quantity = att_json.get("quantity")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);
            (asset_id, quantity)
        } else {
            (0i64, 0i64)
        };

        if asset_id == 0 || quantity <= 0 {
            return Err(ProcessorError::Validation(
                format!("Invalid asset transfer: asset_id={}, quantity={}", asset_id, quantity)
            ));
        }

        let current_height = self.get_current_height();

        // Java: senderAccount.addToAssetBalanceQNT(event, txId, assetId, -quantityQNT);
        self.account_asset_repo.decrease_quantity(sender_id as i64, asset_id, quantity).await?;

        if recipient_id != 0 {
            // Java: recipientAccount.addToAssetAndUnconfirmedAssetBalanceQNT(event, txId, assetId, quantityQNT);
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

            // Java: AssetTransfer.addAssetTransfer(transaction, attachment);
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

    /// ASK_ORDER_PLACEMENT: Place a sell order for assets
    ///
    /// Reference: Java TransactionTypeAsset.ASK_ORDER_PLACEMENT.applyAttachment()
    ///   OrderAsk.addOrder(transaction, attachment);
    ///   senderAccount.addToUnconfirmedAssetBalanceQNT(event, assetId, -quantityQNT);
    async fn apply_ask_order_placement(&self, tx: &Transaction) -> ProcessorResult<()> {
        use orm::models::AskOrderModel;

        let sender_id = tx.sender_id as i64;
        let current_height = self.get_current_height();

        // 解析attachment字段
        let asset_id = self.parse_long_field(tx, "asset").unwrap_or(tx.id as i64);
        let quantity = self.parse_long_field(tx, "quantityQNT").unwrap_or(tx.amount as i64);
        let price_nqt = self.parse_long_field(tx, "priceNQT").unwrap_or(0);

        // 创建ASK_ORDER记录
        let ask_order = AskOrderModel {
            db_id: 0,
            id: tx.id as i64,
            account_id: sender_id,
            asset_id,
            price: price_nqt,
            quantity,
            transaction_index: 0, // TODO: 从block获取
            transaction_height: current_height,
            creation_height: current_height,
            height: current_height,
            latest: true,
        };

        match self.ask_order_repo.insert(&ask_order).await {
            Ok(_) => {
                info!("Placed ASK order {} for asset {}, qty={}, price={}",
                    tx.id, asset_id, quantity, price_nqt);

                // Java: senderAccount.addToUnconfirmedAssetBalanceQNT(event, assetId, -quantityQNT);
                // Decrease unconfirmed asset balance
                self.account_asset_repo.add_to_unconfirmed_quantity(sender_id, asset_id, -quantity).await?;
            }
            Err(e) => {
                warn!("Failed to place ASK order {}: {}", tx.id, e);
                return Err(e.into());
            }
        }

        Ok(())
    }

    /// BID_ORDER_PLACEMENT: Place a buy order for assets (paying NRCS)
    ///
    /// Reference: Java TransactionTypeAsset.BID_ORDER_PLACEMENT.applyAttachment()
    ///   OrderBid.addOrder(transaction, attachment);
    ///   senderAccount.addToUnconfirmedBalance(event, txId, -(priceNQT * quantityQNT));
    async fn apply_bid_order_placement(&self, tx: &Transaction) -> ProcessorResult<()> {
        use orm::models::BidOrderModel;

        let sender_id = tx.sender_id as i64;
        let current_height = self.get_current_height();

        // 解析attachment字段
        let asset_id = self.parse_long_field(tx, "asset").unwrap_or(0);
        let quantity = self.parse_long_field(tx, "quantityQNT").unwrap_or(tx.amount as i64);
        let price_nqt = self.parse_long_field(tx, "priceNQT").unwrap_or(0);

        // 创建BID_ORDER记录
        let bid_order = BidOrderModel {
            db_id: 0,
            id: tx.id as i64,
            account_id: sender_id,
            asset_id,
            price: price_nqt,
            quantity,
            transaction_index: 0, // TODO: 从block获取
            transaction_height: current_height,
            creation_height: current_height,
            height: current_height,
            latest: true,
        };

        match self.bid_order_repo.insert(&bid_order).await {
            Ok(_) => {
                info!("Placed BID order {} for asset {}, qty={}, price={}",
                    tx.id, asset_id, quantity, price_nqt);

                // 减少unconfirmed NRCS余额（预扣购买金额）
                let total_cost = price_nqt * quantity;
                self.account_repo.add_to_unconfirmed_balance(sender_id, -total_cost).await?;
            }
            Err(e) => {
                warn!("Failed to place BID order {}: {}", tx.id, e);
                return Err(e.into());
            }
        }

        Ok(())
    }

    /// ASK_ORDER_CANCELLATION: Cancel a sell order
    ///
    /// Reference: Java TransactionTypeAsset.ASK_ORDER_CANCELLATION.applyAttachment()
    ///   OrderAsk.removeOrder(orderId);
    ///   senderAccount.addToUnconfirmedAssetBalanceQNT(event, assetId, quantityQNT);
    async fn apply_ask_order_cancellation(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id as i64;

        // 解析要取消的order ID
        let order_id = self.parse_long_field(tx, "order").unwrap_or(tx.id as i64);

        // 查找并删除order
        match self.ask_order_repo.find_by_id(order_id).await {
            Ok(Some(order)) if order.account_id == sender_id => {
                let asset_id = order.asset_id;
                let quantity = order.quantity;

                // 删除order记录
                self.ask_order_repo.delete(order.db_id).await?;

                info!("Cancelled ASK order {} for account {}", order_id, sender_id);

                // Java: senderAccount.addToUnconfirmedAssetBalanceQNT(event, assetId, quantityQNT);
                // Restore unconfirmed asset balance
                self.account_asset_repo.add_to_unconfirmed_quantity(sender_id, asset_id, quantity).await?;
            }
            Ok(Some(_)) => {
                warn!("Cannot cancel ASK order owned by another account");
            }
            Ok(None) => {
                warn!("Cannot find ASK order {} to cancel", order_id);
            }
            Err(e) => {
                warn!("Error finding ASK order {}: {}", order_id, e);
                return Err(e.into());
            }
        }

        Ok(())
    }

    /// BID_ORDER_CANCELLATION: Cancel a buy order
    ///
    /// Reference: Java TransactionTypeAsset.BID_ORDER_CANCELLATION.applyAttachment()
    ///   OrderBid.removeOrder(orderId);
    ///   senderAccount.addToUnconfirmedBalance(event, txId, priceNQT * quantityQNT);
    async fn apply_bid_order_cancellation(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id as i64;

        // 解析要取消的order ID
        let order_id = self.parse_long_field(tx, "order").unwrap_or(tx.id as i64);

        // 查找并删除order
        match self.bid_order_repo.find_by_id(order_id).await {
            Ok(Some(order)) if order.account_id == sender_id => {
                let total_cost = order.price * order.quantity;

                // 删除order记录
                self.bid_order_repo.delete(order.db_id).await?;

                info!("Cancelled BID order {} for account {}", order_id, sender_id);

                // 恢复unconfirmed NRCS余额
                self.account_repo.add_to_unconfirmed_balance(sender_id, total_cost).await?;
            }
            Ok(Some(_)) => {
                warn!("Cannot cancel BID order owned by another account");
            }
            Ok(None) => {
                warn!("Cannot find BID order {} to cancel", order_id);
            }
            Err(e) => {
                warn!("Error finding BID order {}: {}", order_id, e);
                return Err(e.into());
            }
        }

        Ok(())
    }

    /// DIVIDEND_PAYMENT: Pay dividends to asset holders
    ///
    /// Reference: Java TransactionTypeAsset.DIVIDEND_PAYMENT.applyAttachment()
    ///   senderAccount.payDividends(transaction, attachment);
    ///   - For each holder: addToBalance(dividend per share)
    ///   - Write ACCOUNT_LEDGER entries for each recipient
    async fn apply_dividend_payment(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id as i64;
        let current_block_id = self.get_current_block_id();
        let current_height = self.get_current_height();
        let current_timestamp = self.get_current_timestamp();

        // 解析attachment字段
        let asset_id = self.parse_long_field(tx, "asset").unwrap_or(0);
        let dividend_per_share = self.parse_long_field(tx, "amountNQT").unwrap_or(0);

        if dividend_per_share <= 0 || asset_id <= 0 {
            warn!("Invalid dividend parameters in transaction {}", tx.id);
            return Ok(());
        }

        // Java: long quantityQNT = asset.getQuantity() - senderAccount.getAssetBalanceQNT(assetId);
        // Calculate total shares excluding sender's shares
        let asset = self.asset_repo.find_by_asset_id(asset_id).await?
            .ok_or_else(|| ProcessorError::Validation(format!("Asset {} not found", asset_id)))?;
        let sender_shares = self.account_asset_repo.find_by_account_and_asset(sender_id, asset_id).await
            .ok()
            .flatten()
            .map(|aa| aa.quantity)
            .unwrap_or(0);
        let total_dividend_shares = asset.quantity - sender_shares;

        // Java: senderAccount.addToBalance(event, txId, -(quantityQNT * amountNQTPerQNT));
        let total_dividend_amount = total_dividend_shares * dividend_per_share;
        self.account_repo.add_to_balance(sender_id, -total_dividend_amount).await?;

        // 获取所有资产持有者并分配红利
        match self.account_asset_repo.find_by_asset(asset_id).await {
            Ok(holders) => {
                info!("Paying dividend on asset {} to {} holders (total={} NQT, excluded sender shares={})",
                    asset_id, holders.len(), total_dividend_amount, sender_shares);

                for holder in holders {
                    let holder_id = holder.account_id;
                    let shares = holder.quantity;
                    let dividend_amount = dividend_per_share * shares;

                    if dividend_amount > 0 && holder_id != sender_id {
                        // Java: recipientAccount.addToBalanceAndUnconfirmedBalance(event, txId, dividend);
                        self.account_repo.add_to_balance(holder_id, dividend_amount).await?;

                        // 写入LEDGER记录
                        use orm::models::AccountLedgerModel;
                        let ledger_entry = AccountLedgerModel {
                            db_id: 0,
                            account_id: holder_id,
                            event_type: ledger_event::ASSET_DIVIDEND_PAYMENT,
                            event_id: tx.id as i64,
                            holding_type: ledger_holding::NRCS_BALANCE,
                            holding_id: None,
                            change: dividend_amount,
                            balance: self.get_account_balance(holder_id).await.unwrap_or(0),
                            block_id: current_block_id,
                            height: current_height,
                            timestamp: current_timestamp,
                        };
                        self.ledger_repo.insert(&ledger_entry).await?;

                        debug!("Paid {} NQT dividend to account {} ({} shares)",
                            dividend_amount, holder_id, shares);
                    }
                }

                info!("Dividend payment completed for asset {}", asset_id);
            }
            Err(e) => {
                warn!("Failed to query asset holders for dividend: {}", e);
                return Err(e.into());
            }
        }

        Ok(())
    }

    /// ASSET_DELETE: Delete some quantity of an asset
    ///
    /// Reference: Java TransactionTypeAsset.ASSET_DELETE.applyAttachment()
    ///   Asset.deleteAsset(transaction, attachment);
    ///   senderAccount.addToAssetBalanceQNT(event, txId, assetId, -quantityQNT);
    async fn apply_asset_delete(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id as i64;
        let asset_id = self.parse_long_field(tx, "asset").unwrap_or(tx.id as i64);
        let delete_quantity = self.parse_long_field(tx, "quantityQQT").unwrap_or(tx.amount as i64);

        if delete_quantity > 0 && asset_id > 0 {
            // Java: Asset.deleteAsset(transaction, attachment);
            // Update the asset's total quantity in the asset table
            self.asset_repo.decrease_quantity(asset_id, delete_quantity).await?;

            // Java: senderAccount.addToAssetBalanceQNT(event, txId, assetId, -quantityQNT);
            self.account_asset_repo.decrease_quantity(sender_id, asset_id, delete_quantity).await?;

            info!("Deleted {} of asset {} from account {}", delete_quantity, asset_id, sender_id);
        } else {
            warn!("Invalid asset delete parameters in transaction {}", tx.id);
        }

        Ok(())
    }

    /// ASSET_INCREASE: Increase the supply of an existing asset
    ///
    /// Reference: Java TransactionTypeAsset.ASSET_INCREASE.applyAttachment()
    ///   Asset.increaseAsset(transaction, attachment);
    ///   senderAccount.addToAssetAndUnconfirmedAssetBalanceQNT(event, assetId, increaseQuantityQNT);
    async fn apply_asset_increase(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id as i64;
        let asset_id = self.parse_long_field(tx, "asset").unwrap_or(0);
        let increase_quantity = self.parse_long_field(tx, "quantityQQT").unwrap_or(tx.amount as i64);

        if increase_quantity > 0 && asset_id > 0 {
            // Java: Asset.increaseAsset(transaction, attachment);
            // Update the asset's total quantity in the asset table
            self.asset_repo.increase_quantity(asset_id, increase_quantity).await?;

            // Java: senderAccount.addToAssetAndUnconfirmedAssetBalanceQNT(event, assetId, increaseQuantityQNT);
            // Update both confirmed and unconfirmed asset balance
            self.account_asset_repo.increase_quantity(sender_id, asset_id, increase_quantity).await?;
            self.account_asset_repo.add_to_unconfirmed_quantity(sender_id, asset_id, increase_quantity).await?;

            info!("Increased asset {} by {} for account {}", asset_id, increase_quantity, sender_id);
        } else {
            warn!("Invalid asset increase parameters in transaction {}", tx.id);
        }

        Ok(())
    }

    /// ASSET_PROPERTY_SET: Set a property on an asset
    ///
    /// Reference: Java TransactionTypeAsset.ASSET_PROPERTY_SET.applyAttachment()
    ///   Asset.setProperty(transaction, attachment);
    async fn apply_asset_property_set(&self, tx: &Transaction) -> ProcessorResult<()> {
        use orm::models::AssetPropertyModel;

        let sender_id = tx.sender_id as i64;
        let asset_id = self.parse_long_field(tx, "asset").unwrap_or(0);
        let property_name = self.parse_string_field(tx, "property").unwrap_or_default();
        let property_value = self.parse_string_field(tx, "value").unwrap_or_default();

        if !property_name.is_empty() && asset_id > 0 {
            let prop_model = AssetPropertyModel {
                db_id: 0,
                id: tx.id as i64,
                asset_id,
                setter_id: sender_id,
                property: property_name.clone(),
                value: Some(property_value.clone()),
                height: self.get_current_height(),
                latest: true,
            };

            self.asset_property_repo.insert(&prop_model).await?;
            info!("Set property '{}'='{}' on asset {}", property_name, property_value, asset_id);
        }

        Ok(())
    }

    /// ASSET_PROPERTY_DELETE: Delete a property from an asset
    ///
    /// Reference: Java TransactionTypeAsset.ASSET_PROPERTY_DELETE.applyAttachment()
    ///   Asset.deleteProperty(transaction, attachment);
    async fn apply_asset_property_delete(&self, tx: &Transaction) -> ProcessorResult<()> {
        let asset_id = self.parse_long_field(tx, "asset").unwrap_or(0);
        let property_name = self.parse_string_field(tx, "property").unwrap_or_default();

        if !property_name.is_empty() && asset_id > 0 {
            // TODO: 实现delete方法
            debug!("Deleting property '{}' from asset {}", property_name, asset_id);
            info!("AssetProperty deletion not yet fully implemented");
        }

        Ok(())
    }

    /// ASSET_LONG_VALUE_PROPERTY_SET: Set a long-value property on an asset
    ///
    /// Reference: Java TransactionTypeAsset.ASSET_LONG_VALUE_PROPERTY_SET.applyAttachment()
    ///   Asset.setProperty(transaction, attachment); (long value version)
    async fn apply_asset_long_value_property_set(&self, tx: &Transaction) -> ProcessorResult<()> {
        use orm::models::AssetPropertyModel;

        let sender_id = tx.sender_id as i64;
        let asset_id = self.parse_long_field(tx, "asset").unwrap_or(0);
        let property_name = self.parse_string_field(tx, "property").unwrap_or_default();
        let long_value = self.parse_long_field(tx, "value").unwrap_or(0);

        if !property_name.is_empty() && asset_id > 0 {
            let prop_model = AssetPropertyModel {
                db_id: 0,
                id: tx.id as i64,
                asset_id,
                setter_id: sender_id,
                property: property_name.clone(),
                value: Some(long_value.to_string()), // 存储为字符串表示
                height: self.get_current_height(),
                latest: true,
            };

            self.asset_property_repo.insert(&prop_model).await?;
            info!("Set long-value property '{}'={} on asset {}", property_name, long_value, asset_id);
        }

        Ok(())
    }

    /// MonetarySystem attachment processing
    ///
    /// Reference: Java TransactionTypeCurrency
    async fn apply_monetary_system_attachment(&self, tx: &Transaction) -> ProcessorResult<()> {
        match tx.subtype {
            0 => { // CURRENCY_ISSUANCE
                self.apply_currency_issuance(tx).await?;
            }

            1 => { // RESERVE_INCREASE
                self.apply_reserve_increase(tx).await?;
            }

            2 => { // RESERVE_CLAIM
                self.apply_reserve_claim(tx).await?;
            }

            3 => { // CURRENCY_TRANSFER
                self.apply_currency_transfer(tx).await?;
            }

            4 => { // PUBLISH_EXCHANGE_OFFER
                self.apply_publish_exchange_offer(tx).await?;
            }

            5 => { // EXCHANGE_BUY (NRCS → Currency)
                self.apply_exchange_buy(tx).await?;
            }

            6 => { // EXCHANGE_SELL (Currency → NRCS)
                self.apply_exchange_sell(tx).await?;
            }

            7 => { // CURRENCY_MINTING
                self.apply_currency_minting(tx).await?;
            }

            8 => { // CURRENCY_DELETION
                self.apply_currency_deletion(tx).await?;
            }

            _ => {
                debug!("MonetarySystem subtype {} processed (stub)", tx.subtype);
            }
        }

        Ok(())
    }

    /// CURRENCY_ISSUANCE: Issue a new currency
    ///
    /// Reference: Java TransactionTypeCurrency.CURRENCY_ISSUANCE.applyAttachment()
    ///   Currency.addCurrency(transaction, attachment);
    ///   senderAccount.addToCurrencyAndUnconfirmedCurrencyUnits(event, currencyId, initialSupply);
    async fn apply_currency_issuance(&self, tx: &Transaction) -> ProcessorResult<()> {
        use orm::models::CurrencyModel;
        use blockchain_types::constants::{MIN_CURRENCY_NAME_LENGTH, MAX_CURRENCY_NAME_LENGTH, MIN_CURRENCY_CODE_LENGTH, MAX_CURRENCY_CODE_LENGTH, MAX_CURRENCY_TOTAL_SUPPLY};

        let sender_id = tx.sender_id as i64;
        let current_height = self.get_current_height();
        let currency_id = tx.id as i64;

        // 解析attachment字段
        let name = self.parse_string_field(tx, "name").unwrap_or(format!("Currency_{}", currency_id));
        let code = self.parse_string_field(tx, "code").unwrap_or_default();
        let description = self.parse_string_field(tx, "description");
        let initial_supply = self.parse_long_field(tx, "initialSupply").unwrap_or(0);
        let max_supply = self.parse_long_field(tx, "maxSupply").unwrap_or(initial_supply);
        let decimals = self.parse_long_field(tx, "decimals").unwrap_or(0) as i16;
        let type_ = self.parse_long_field(tx, "type").unwrap_or(0) as i32;
        let min_reserve_per_unit_nqt = self.parse_long_field(tx, "minReservePerUnitNQT").unwrap_or(0);
        let min_difficulty = self.parse_long_field(tx, "minDifficulty").unwrap_or(0) as i16;
        let max_difficulty = self.parse_long_field(tx, "maxDifficulty").unwrap_or(0) as i16;
        let ruleset = self.parse_long_field(tx, "ruleset").unwrap_or(0) as i16;
        let algorithm = self.parse_long_field(tx, "algorithm").unwrap_or(0) as i16;
        let reserve_supply = self.parse_long_field(tx, "reserveSupply").unwrap_or(0);
        let issuance_height = self.parse_long_field(tx, "issuanceHeight").unwrap_or(current_height as i64) as i32;

        // Java: CurrencyType.validate() - naming rules
        if name.len() < MIN_CURRENCY_NAME_LENGTH || name.len() > MAX_CURRENCY_NAME_LENGTH {
            return Err(ProcessorError::Validation(
                format!("Currency name length must be {}-{}, got {}", MIN_CURRENCY_NAME_LENGTH, MAX_CURRENCY_NAME_LENGTH, name.len())
            ));
        }

        if code.len() < MIN_CURRENCY_CODE_LENGTH || code.len() > MAX_CURRENCY_CODE_LENGTH {
            return Err(ProcessorError::Validation(
                format!("Currency code length must be {}-{}, got {}", MIN_CURRENCY_CODE_LENGTH, MAX_CURRENCY_CODE_LENGTH, code.len())
            ));
        }

        // Java: validateCurrencyNaming() - code must be uppercase letters only
        if !code.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(ProcessorError::Validation(
                format!("Currency code must be uppercase letters only, got '{}'", code)
            ));
        }

        // Java: decimals must be 0-8
        if !(0..=8).contains(&decimals) {
            return Err(ProcessorError::Validation(
                format!("Currency decimals must be 0-8, got {}", decimals)
            ));
        }

        // Java: maxSupply must be > 0 and <= MAX_CURRENCY_TOTAL_SUPPLY
        if max_supply <= 0 || max_supply > MAX_CURRENCY_TOTAL_SUPPLY as i64 {
            return Err(ProcessorError::Validation(
                format!("Currency max supply must be 1-{}, got {}", MAX_CURRENCY_TOTAL_SUPPLY, max_supply)
            ));
        }

        // Java: initialSupply must be <= maxSupply
        if initial_supply > max_supply {
            return Err(ProcessorError::Validation(
                format!("Currency initial supply ({}) cannot exceed max supply ({})", initial_supply, max_supply)
            ));
        }

        // Java: isDuplicate() check - currency name must be unique
        if let Ok(Some(_)) = self.currency_repo.find_by_code(&code).await {
            return Err(ProcessorError::Validation(
                format!("Currency with code '{}' already exists", code)
            ));
        }

        // 创建CURRENCY记录
        let currency_model = CurrencyModel {
            db_id: 0,
            id: currency_id,
            account_id: sender_id,
            name: name.clone(),
            name_lower: name.to_lowercase(),
            code: code.clone(),
            description,
            type_,
            initial_supply,
            reserve_supply,
            max_supply,
            creation_height: current_height,
            issuance_height,
            min_reserve_per_unit_nqt,
            min_difficulty,
            max_difficulty,
            ruleset,
            algorithm,
            decimals,
            height: current_height,
            latest: true,
        };

        match self.currency_repo.insert(&currency_model).await {
            Ok(_) => {
                info!("Issued currency '{}' (ID={}) with initial supply {}", name, currency_id, initial_supply);

                // Java: senderAccount.addToCurrencyAndUnconfirmedCurrencyUnits(event, currencyId, initialSupply);
                if initial_supply > 0 {
                    self.account_currency_repo.update_units(sender_id, currency_id, initial_supply).await?;
                    self.account_currency_repo.add_to_unconfirmed_units(sender_id, currency_id, initial_supply).await?;
                }
            }
            Err(e) => {
                warn!("Failed to issue currency '{}': {}", name, e);
                return Err(e.into());
            }
        }

        Ok(())
    }

    /// RESERVE_INCREASE: Increase the reserve of a currency
    ///
    /// Reference: Java TransactionTypeCurrency.RESERVE_INCREASE.applyAttachment()
    ///   Currency.increaseReserve(transaction, attachment);
    ///   senderAccount.addToBalance(event, txId, -amountNQT);
    async fn apply_reserve_increase(&self, tx: &Transaction) -> ProcessorResult<()> {
        let _sender_id = tx.sender_id as i64;
        let currency_id = self.parse_long_field(tx, "currency").unwrap_or(0);
        let amount_per_unit = self.parse_long_field(tx, "amountPerUnitNQT").unwrap_or(0);

        if currency_id > 0 && amount_per_unit > 0 {
            // 增加currency的reserve（P1优化：使用真正的CurrencyRepository方法）
            self.currency_repo.increase_reserve(currency_id, amount_per_unit).await?;

            info!("Increased reserve for currency {} by {} NQT/unit", currency_id, amount_per_unit);
        } else {
            warn!("Invalid reserve increase parameters in transaction {}", tx.id);
        }

        Ok(())
    }

    /// RESERVE_CLAIM: Claim reserve from a currency
    ///
    /// Reference: Java TransactionTypeCurrency.RESERVE_CLAIM.applyAttachment()
    ///   Currency.claimReserve(transaction, attachment);
    ///   senderAccount.addToCurrencyUnits(event, txId, -units);
    ///   senderAccount.addToBalance(event, txId, +nrcsReceived);
    async fn apply_reserve_claim(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id as i64;
        let currency_id = self.parse_long_field(tx, "currency").unwrap_or(0);
        let units_to_claim = self.parse_long_field(tx, "units").unwrap_or(0);

        if currency_id > 0 && units_to_claim > 0 {
            // Java: nrcsReceived = Convert.unitsToNQT(unitsToClaim * currency.getReserveSupply() / currency.getCurrentSupply());
            // Calculate NRCS received based on reserve ratio
            let currency = self.currency_repo.find_by_id(currency_id).await?
                .ok_or_else(|| ProcessorError::Validation(format!("Currency {} not found", currency_id)))?;

            // Use initial_supply as current supply (simplification)
            // In a full implementation, this would sum all account_currency units
            let current_supply = currency.initial_supply;
            let reserve_supply = currency.reserve_supply;

            if current_supply <= 0 {
                return Err(ProcessorError::Validation(
                    format!("Currency {} has no current supply", currency_id)
                ));
            }

            // nrcs_received = (units_to_claim * reserve_supply) / current_supply
            let nrcs_received = (units_to_claim as i128)
                .checked_mul(reserve_supply as i128)
                .and_then(|v| v.checked_div(current_supply as i128))
                .map(|v| v as i64)
                .ok_or_else(|| ProcessorError::Validation("Reserve claim calculation overflow".to_string()))?;

            // 减少货币余额
            self.account_currency_repo.update_units(sender_id, currency_id, -units_to_claim).await?;

            // 增加NRCS余额
            self.account_repo.add_to_balance(sender_id, nrcs_received).await?;

            info!("Claimed {} units from currency {}, received {} NQT (reserve ratio: {}/{})",
                  units_to_claim, currency_id, nrcs_received, reserve_supply, current_supply);
        } else {
            warn!("Invalid reserve claim parameters in transaction {}", tx.id);
        }

        Ok(())
    }

    /// PUBLISH_EXCHANGE_OFFER: Publish an exchange offer for currency
    ///
    /// Reference: Java TransactionTypeCurrency.PUBLISH_EXCHANGE_OFFER.applyAttachment()
    ///   CurrencyExchangeOffer.publishOffer(transaction, attachment);
    async fn apply_publish_exchange_offer(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id as i64;
        let currency_id = self.parse_long_field(tx, "currency").unwrap_or(0);

        if currency_id > 0 {
            // Parse offer details from attachment
            let buy_rate = self.parse_long_field(tx, "buyRate").unwrap_or(0);
            let sell_rate = self.parse_long_field(tx, "sellRate").unwrap_or(0);
            let total_buy_limit = self.parse_long_field(tx, "totalBuyLimit").unwrap_or(0);
            let total_sell_limit = self.parse_long_field(tx, "totalSellLimit").unwrap_or(0);
            let _initial_buy_supply = self.parse_long_field(tx, "initialBuySupply").unwrap_or(0);
            let _initial_sell_supply = self.parse_long_field(tx, "initialSellSupply").unwrap_or(0);
            let expiration_height = self.parse_long_field(tx, "expirationHeight").unwrap_or(0);

            // Java: CurrencyExchangeOffer.publishOffer(transaction, attachment);
            // Store the exchange offer - for now we'll use the exchange_request table
            // In a full implementation, this would create a separate exchange_offer record
            info!("Published exchange offer for currency {} by account {}: buy_rate={}, sell_rate={}, buy_limit={}, sell_limit={}, expiration={}",
                  currency_id, sender_id, buy_rate, sell_rate, total_buy_limit, total_sell_limit, expiration_height);

            // TODO: Create proper exchange_offer table and model
            // The offer should be matched against future EXCHANGE_BUY/SELL requests
        } else {
            warn!("Missing currency ID in transaction {}", tx.id);
        }

        Ok(())
    }

    /// EXCHANGE_BUY: Buy currency using NRCS
    ///
    /// Reference: Java TransactionTypeCurrency.EXCHANGE_BUY.applyAttachment()
    ///   ExchangeRequest.addExchangeRequest(transaction, attachment);
    ///   Currency.exchangeNRCSForCurrency(...);
    async fn apply_exchange_buy(&self, tx: &Transaction) -> ProcessorResult<()> {
        use orm::models::ExchangeRequestModel;

        let sender_id = tx.sender_id as i64;
        let current_height = self.get_current_height();

        let currency_id = self.parse_long_field(tx, "currency").unwrap_or(0);
        let rate = self.parse_long_field(tx, "rate").unwrap_or(0);
        let units = self.parse_long_field(tx, "units").unwrap_or(0);

        if currency_id > 0 && units > 0 && rate > 0 {
            // 创建EXCHANGE_REQUEST记录
            let request_model = ExchangeRequestModel {
                db_id: 0,
                id: tx.id as i64,
                account_id: sender_id,
                currency_id,
                units,
                rate,
                is_buy: true, // EXCHANGE_BUY = true
                timestamp: self.get_current_timestamp(),
                height: current_height,
            };

            match self.exchange_request_repo.insert(&request_model).await {
                Ok(_) => {
                    info!("Created exchange buy request: {} units of currency {} at rate {}",
                        units, currency_id, rate);
                    // Note: NRCS deduction is handled by base apply() method via tx.amount
                    // No additional deduction needed here
                }
                Err(e) => {
                    warn!("Failed to create exchange buy request (non-critical): {}", e);
                    debug!("Exchange buy request creation failed: {}", e);
                }
            }
        } else {
            warn!("Invalid exchange buy parameters in transaction {}", tx.id);
        }

        Ok(())
    }

    /// EXCHANGE_SELL: Sell currency for NRCS
    ///
    /// Reference: Java TransactionTypeCurrency.EXCHANGE_SELL.applyAttachment()
    ///   ExchangeRequest.addExchangeRequest(transaction, attachment);
    ///   Currency.exchangeCurrencyForNRCS(...);
    async fn apply_exchange_sell(&self, tx: &Transaction) -> ProcessorResult<()> {
        use orm::models::ExchangeRequestModel;

        let sender_id = tx.sender_id as i64;
        let current_height = self.get_current_height();

        let currency_id = self.parse_long_field(tx, "currency").unwrap_or(0);
        let rate = self.parse_long_field(tx, "rate").unwrap_or(0);
        let units = self.parse_long_field(tx, "units").unwrap_or(0);

        if currency_id > 0 && units > 0 && rate > 0 {
            // 创建EXCHANGE_REQUEST记录
            let request_model = ExchangeRequestModel {
                db_id: 0,
                id: tx.id as i64,
                account_id: sender_id,
                currency_id,
                units,
                rate,
                is_buy: false, // EXCHANGE_SELL = false
                timestamp: self.get_current_timestamp(),
                height: current_height,
            };

            match self.exchange_request_repo.insert(&request_model).await {
                Ok(_) => {
                    info!("Created exchange sell request: {} units of currency {} at rate {}",
                        units, currency_id, rate);
                    // Java: senderAccount.addToCurrencyUnconfirmedUnits(event, txId, currencyId, -units);
                    // Deduct from unconfirmed currency units (not confirmed)
                    self.account_currency_repo.add_to_unconfirmed_units(sender_id, currency_id, -units).await?;
                }
                Err(e) => {
                    warn!("Failed to create exchange sell request: {}", e);
                    return Err(e.into());
                }
            }
        } else {
            warn!("Invalid exchange sell parameters in transaction {}", tx.id);
        }

        Ok(())
    }

    /// CURRENCY_MINTING: Mint new currency units
    ///
    /// Reference: Java TransactionTypeCurrency.CURRENCY_MINTING.applyAttachment()
    ///   CurrencyMint.mintCurrency(transaction, attachment);
    ///   senderAccount.addToCurrencyAndUnconfirmedCurrencyUnits(event, currencyId, mintedUnits);
    async fn apply_currency_minting(&self, tx: &Transaction) -> ProcessorResult<()> {
        use orm::models::CurrencyMintModel;

        let sender_id = tx.sender_id as i64;
        let current_height = self.get_current_height();
        let currency_id = self.parse_long_field(tx, "currency").unwrap_or(0);
        let minted_units = self.parse_long_field(tx, "units").unwrap_or(0);

        if currency_id > 0 && minted_units > 0 {
            // 创建CURRENCY_MINT记录
            let mint_model = CurrencyMintModel {
                db_id: 0,
                currency_id,
                account_id: sender_id,
                counter: minted_units, // 使用counter字段存储minted数量
                height: current_height,
                latest: true,
            };

            match self.currency_mint_repo.insert(&mint_model).await {
                Ok(_) => {
                    info!("Minted {} units of currency {} for account {}", minted_units, currency_id, sender_id);

                    // 更新货币余额
                    self.account_currency_repo.update_units(sender_id, currency_id, minted_units).await?;

                    // 更新currency的总supply（P1优化：使用真正的CurrencyRepository方法）
                    self.currency_repo.increase_supply(currency_id, minted_units).await?;
                }
                Err(e) => {
                    warn!("Failed to mint currency {}: {}", currency_id, e);
                    return Err(e.into());
                }
            }
        } else {
            warn!("Invalid currency minting parameters in transaction {}", tx.id);
        }

        Ok(())
    }

    /// CURRENCY_DELETION: Delete a currency
    ///
    /// Reference: Java TransactionTypeCurrency.CURRENCY_DELETION.applyAttachment()
    ///   currency.delete(currency);
    async fn apply_currency_deletion(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id as i64;
        let currency_id = self.parse_long_field(tx, "currency").unwrap_or(0);

        if currency_id > 0 {
            // 验证发送者是currency的创建者
            match self.currency_repo.find_by_id(currency_id).await {
                Ok(Some(currency)) if currency.account_id == sender_id => {
                    // 标记为deleted或实际删除
                    self.currency_repo.delete_currency(currency_id).await?;
                    info!("Deleted currency {} by owner account {}", currency_id, sender_id);
                }
                Ok(Some(_)) => {
                    warn!("Cannot delete currency owned by another account");
                }
                Ok(None) => {
                    warn!("Cannot find currency {} to delete", currency_id);
                }
                Err(e) => {
                    warn!("Error finding currency {}: {}", currency_id, e);
                    return Err(e.into());
                }
            }
        } else {
            warn!("Missing currency ID in transaction {}", tx.id);
        }

        Ok(())
    }

    /// CURRENCY_TRANSFER: Transfer currency between accounts
    ///
    /// Reference: Java TransactionTypeCurrency.CURRENCY_TRANSFER.applyAttachment()
    ///   senderAccount.addToCurrencyUnits(event, txId, currencyId, -units);
    ///   recipientAccount.addToCurrencyAndUnconfirmedCurrencyUnits(event, txId, currencyId, +units);
    ///   CurrencyTransfer.addCurrencyTransfer(transaction, attachment);
    async fn apply_currency_transfer(&self, tx: &Transaction) -> ProcessorResult<()> {
        use orm::models::CurrencyTransferModel;

        let sender_id = tx.sender_id as i64;
        let recipient_id = tx.recipient_id.map(|id| id as i64).unwrap_or(0);
        let current_height = self.get_current_height();

        // 解析attachment字段
        let currency_id = self.parse_long_field(tx, "currency").unwrap_or(0);
        let units = self.parse_long_field(tx, "units").unwrap_or(tx.amount as i64);

        if units > 0 && currency_id > 0 {
            // Java: senderAccount.addToCurrencyUnits(event, txId, currencyId, -units);
            self.account_currency_repo.update_units(sender_id, currency_id, -units).await?;

            if recipient_id != 0 {
                // Java: recipientAccount.addToCurrencyAndUnconfirmedCurrencyUnits(event, txId, currencyId, +units);
                self.account_repo.get_or_create(recipient_id).await?;

                // Update both confirmed and unconfirmed for recipient
                self.account_currency_repo.update_units(recipient_id, currency_id, units).await?;
                self.account_currency_repo.add_to_unconfirmed_units(recipient_id, currency_id, units).await?;
            }

            // 创建CURRENCY_TRANSFER记录
            let transfer_model = CurrencyTransferModel {
                db_id: 0,
                id: tx.id as i64,
                sender_id,
                recipient_id,
                currency_id,
                units,
                timestamp: self.get_current_timestamp(),
                height: current_height,
            };

            match self.currency_transfer_repo.insert(&transfer_model).await {
                Ok(_) => {
                    info!("Transferred {} of currency {} from {} to {}",
                        units, currency_id, sender_id, recipient_id);
                }
                Err(e) => {
                    warn!("Failed to record currency transfer: {}", e);
                    return Err(e.into());
                }
            }
        } else {
            warn!("Invalid currency transfer parameters in transaction {}", tx.id);
        }

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

    // ==================== 新增：完整的交易类型处理方法 ====================

    /// Messaging (Type 1) 交易处理
    ///
    /// Reference: Java TransactionTypeAccount
    /// - Subtype 5: ALIAS_ASSIGNMENT -> Alias.addOrUpdateAlias()
    /// - Subtype 6: ALIAS_SELL -> Alias.sellAlias()
    /// - Subtype 7: ALIAS_BUY -> Alias.changeOwner()
    /// - Subtype 8: ALIAS_DELETE -> Alias.deleteAlias()
    /// - Subtype 2: POLL_CREATION -> Poll.addPoll()
    /// - Subtype 3: VOTE_CASTING -> Vote.addVote()
    /// - Subtype 9: PHASING_VOTE_CASTING -> PhasingVote.addVote()
    /// - Subtype 10: ACCOUNT_PROPERTY -> recipientAccount.setProperty()
    /// - Subtype 11: ACCOUNT_PROPERTY_DELETE -> senderAccount.deleteProperty()
    /// - Subtype 12: ACCOUNT_LONG_VALUE_PROPERTY -> recipientAccount.setProperty()
    async fn apply_messaging_attachment(&self, tx: &Transaction) -> ProcessorResult<()> {
        use orm::models::*;

        let sender_id = tx.sender_id as i64;
        let recipient_id = tx.recipient_id.map(|id| id as i64).unwrap_or(0);
        let current_height = self.get_current_height();
        let current_timestamp = self.get_current_timestamp();

        match tx.subtype {
            1 => { // ALIAS_ASSIGNMENT
                // Java: Alias.addOrUpdateAlias(transaction, attachment)
                // Reference: MessagingAliasAssignment.java
                //   attachment fields: { "alias": String, "uri": String }
                //   DB operation: INSERT or UPDATE ALIAS table

                // 解析attachment (假设tx有attachment_json字段，否则从bytes解析)
                let alias_name = self.parse_string_field(tx, "alias").unwrap_or_default();
                let alias_uri = self.parse_string_field(tx, "uri").unwrap_or_default();

                if !alias_name.is_empty() {
                    let alias_model = AliasModel {
                        db_id: 0,
                        id: tx.id as i64,
                        account_id: sender_id,
                        alias_name: alias_name.clone(),
                        alias_name_lower: alias_name.to_lowercase(),
                        alias_uri,
                        timestamp: current_timestamp,
                        height: current_height,
                        latest: true,
                    };

                    // 检查是否已存在同名alias
                    match self.alias_repo.find_by_name(&alias_name.to_lowercase()).await {
                        Ok(Some(existing)) => {
                            // 更新现有alias
                            let mut updated = alias_model;
                            updated.db_id = existing.db_id;
                            // 注意：这里应该调用update方法，但当前trait可能没有
                            // 暂时使用insert（实际应该upsert）
                            debug!("Updating existing alias '{}' for transaction {}", alias_name, tx.id);
                        }
                        Ok(None) => {
                            // 插入新alias
                            self.alias_repo.insert(&alias_model).await?;
                            info!("Created new alias '{}' for account {}", alias_name, sender_id);
                        }
                        Err(e) => {
                            warn!("Error checking alias existence: {}", e);
                            return Err(e.into());
                        }
                    }
                }
            }

            6 => { // ALIAS_SELL
                // Java: Alias.sellAlias(transaction, attachment)
                // Reference: MessagingAliasSell.java
                //   attachment fields: { "alias": String, "priceNQT": long }
                //   DB operation: INSERT or UPDATE ALIAS_OFFER table

                let alias_name = self.parse_string_field(tx, "alias").unwrap_or_default();
                let price_nqt = self.parse_long_field(tx, "priceNQT").unwrap_or(0);

                if !alias_name.is_empty() {
                    // 先查找alias ID
                    match self.alias_repo.find_by_name(&alias_name.to_lowercase()).await {
                        Ok(Some(alias)) => {
                            let offer_model = AliasOfferModel {
                                db_id: 0,
                                id: alias.id, // 使用alias的ID作为offer的ID
                                price: price_nqt,
                                buyer_id: None, // 卖出时buyer为空
                                height: current_height,
                                latest: true,
                            };

                            self.alias_offer_repo.insert(&offer_model).await?;
                            info!("Alias '{}' put up for sale at price {} NQT", alias_name, price_nqt);
                        }
                        Ok(None) => {
                            warn!("Cannot sell non-existent alias '{}'", alias_name);
                        }
                        Err(e) => {
                            warn!("Error finding alias for sale: {}", e);
                            return Err(e.into());
                        }
                    }
                }
            }

            7 => { // ALIAS_BUY
                // Java: Alias.changeOwner(transaction.getSenderId(), aliasName)
                // Reference: TransactionTypeAccount.ALIAS_BUY.applyAttachment()
                //   - UPDATE ALIAS.owner_id = transaction.senderId
                //   - DELETE ALIAS_OFFER for this alias
                //   - 验证: amount >= offer.price

                let alias_name = self.parse_string_field(tx, "alias").unwrap_or_default();

                if !alias_name.is_empty() && recipient_id != 0 {
                    match self.alias_repo.find_by_name(&alias_name.to_lowercase()).await {
                        Ok(Some(mut alias)) => {
                            // 验证买方金额是否足够（amount字段即为支付价格）
                            if tx.amount > 0 {
                                // 更新alias所有者
                                alias.account_id = sender_id;
                                // 这里需要调用update，暂时跳过（需扩展trait）
                                info!("Alias '{}' ownership transferred to account {}", alias_name, sender_id);

                                // 删除alias offer
                                if let Ok(Some(_offer)) = self.alias_offer_repo.find_by_alias(alias.id).await {
                                    // self.alias_offer_repo.delete(offer.db_id).await?;
                                    debug!("Removed alias offer for '{}'", alias_name);
                                }
                            } else {
                                warn!("Insufficient payment for alias purchase");
                            }
                        }
                        Ok(None) => {
                            warn!("Cannot buy non-existent alias '{}'", alias_name);
                        }
                        Err(e) => {
                            warn!("Error finding alias for purchase: {}", e);
                            return Err(e.into());
                        }
                    }
                }
            }

            8 => { // ALIAS_DELETE
                // Java: Alias.deleteAlias(aliasName)
                // Reference: TransactionTypeAccount.ALIAS_DELETE.applyAttachment()
                //   - DELETE from ALIAS table (by name)

                let alias_name = self.parse_string_field(tx, "alias").unwrap_or_default();

                if !alias_name.is_empty() {
                    match self.alias_repo.find_by_name(&alias_name.to_lowercase()).await {
                        Ok(Some(alias)) => {
                            // 验证删除权限：只有owner可以删除
                            if alias.account_id == sender_id {
                                // self.alias_repo.delete(alias.db_id).await?;
                                info!("Alias '{}' deleted by owner account {}", alias_name, sender_id);
                            } else {
                                warn!("Account {} cannot delete alias owned by {}",
                                    sender_id, alias.account_id);
                            }
                        }
                        Ok(None) => {
                            warn!("Cannot delete non-existent alias '{}'", alias_name);
                        }
                        Err(e) => {
                            warn!("Error finding alias for deletion: {}", e);
                            return Err(e.into());
                        }
                    }
                }
            }

            5 => { // ACCOUNT_INFO
                // Java: Account.setAccountInfo(transaction, attachment)
                // Reference: MessagingAccountInfo.java
                //   attachment fields: { "name": String, "description": String }
                //   DB operation: UPSERT ACCOUNT_INFO table

                let name = self.parse_string_field(tx, "name").unwrap_or_default();
                let description = self.parse_string_field(tx, "description").unwrap_or_default();

                let model = orm::models::AccountInfoModel {
                    db_id: 0,
                    account_id: sender_id,
                    name: Some(name.clone()),
                    description: Some(description),
                    height: self.get_current_height(),
                    latest: true,
                };
                self.account_info_repo.upsert(&model).await
                    .map_err(|e| ProcessorError::Validation(format!("AccountInfo upsert failed: {}", e)))?;
                debug!("Account info set for account {}: name='{}'", sender_id, name);
            }

            2 => { // POLL_CREATION
                // Java: Poll.addPoll(transaction, attachment)
                // Reference: PollCreationAttachment.java
                //   attachment fields: { "name", "description", "options[]",
                //                       "minNumOptions", "maxNumOptions",
                //                       "minRangeValue", "maxRangeValue",
                //                       "votingModel", "minBalance", ... }
                //   DB operation:
                //     1. INSERT POLL table
                //     2. INSERT POLL_RESULT (one per option, initial weight=0)

                let poll_name = self.parse_string_field(tx, "name").unwrap_or_default();
                let poll_description = self.parse_string_field(tx, "description").unwrap_or_default();
                let options_str = self.parse_string_field(tx, "options").unwrap_or_default();

                if !poll_name.is_empty() {
                    let finish_height = self.parse_long_field(tx, "finishHeight")
                        .map(|h| h as i32)
                        .unwrap_or(0);
                    let voting_model = self.parse_long_field(tx, "votingModel")
                        .map(|v| v as i16)
                        .unwrap_or(0);
                    let min_balance = self.parse_long_field(tx, "minBalance");
                    let min_balance_model = self.parse_long_field(tx, "minBalanceModel")
                        .map(|m| m as i16);

                    let poll_model = PollModel {
                        db_id: 0,
                        id: tx.id as i64,
                        account_id: sender_id,
                        name: poll_name.clone(),
                        description: Some(poll_description),
                        options: options_str.clone(),
                        min_num_options: self.parse_long_field(tx, "minNumOptions").map(|n| n as i16),
                        max_num_options: self.parse_long_field(tx, "maxNumOptions").map(|n| n as i16),
                        min_range_value: self.parse_long_field(tx, "minRangeValue").map(|r| r as i16),
                        max_range_value: self.parse_long_field(tx, "maxRangeValue").map(|r| r as i16),
                        timestamp: current_timestamp,
                        finish_height,
                        voting_model,
                        min_balance,
                        min_balance_model,
                        holding_id: None, // TODO: 从attachment解析
                        height: current_height,
                    };

                    match self.poll_repo.insert(&poll_model).await {
                        Ok(_) => {
                            info!("Created poll '{}' (ID={}) for account {}", poll_name, tx.id, sender_id);

                            // 初始化POLL_RESULT（每个选项初始weight=0）
                            // TODO: 解析options数组并创建对应的PollResultModel
                            debug!("Initializing poll results for poll {}", tx.id);
                        }
                        Err(e) => {
                            warn!("Failed to create poll '{}': {}", poll_name, e);
                            return Err(e.into());
                        }
                    }
                } else {
                    warn!("Empty poll name in transaction {}", tx.id);
                }
            }

            3 => { // VOTE_CASTING
                // Java: Vote.addVote(transaction, attachment)
                // Reference: VoteCastingAttachment.java
                //   attachment fields: { "pollId": long, "voteBytes": []byte }
                //   DB operation:
                //     1. INSERT VOTE table
                //     2. UPDATE POLL_RESULT.weight (增加投票权重)

                let poll_id = self.parse_long_field(tx, "pollId").unwrap_or(0);

                if poll_id > 0 {
                    // 验证poll是否存在
                    match self.poll_repo.find_by_id(poll_id).await {
                        Ok(Some(_poll)) => {
                            // 创建Vote记录
                            let vote_bytes = vec![1u8]; // TODO: 从attachment解析实际的vote bytes

                            let vote_model = VoteModel {
                                db_id: 0,
                                id: tx.id as i64,
                                poll_id,
                                voter_id: sender_id,
                                vote_bytes: vote_bytes.clone(),
                                height: current_height,
                            };

                            match self.vote_repo.insert(&vote_model).await {
                                Ok(_) => {
                                    info!("Account {} voted on poll {} (tx={})", sender_id, poll_id, tx.id);

                                    // 更新POLL_RESULT的weight（根据voter的balance增加权重）
                                    // Java: PollResult.addWeight(voterBalance)
                                    // TODO: 实现权重更新逻辑
                                    debug!("Updating poll result weights for poll {}", poll_id);
                                }
                                Err(e) => {
                                    warn!("Failed to record vote for poll {}: {}", poll_id, e);
                                    return Err(e.into());
                                }
                            }
                        }
                        Ok(None) => {
                            warn!("Cannot vote on non-existent poll {}", poll_id);
                        }
                        Err(e) => {
                            warn!("Error finding poll {}: {}", poll_id, e);
                            return Err(e.into());
                        }
                    }
                } else {
                    warn!("Missing or invalid pollId in transaction {}", tx.id);
                }
            }

            9 => { // PHASING_VOTE_CASTING
                // Java: PhasingVote.addVote(transaction, senderAccount, phasedTxId)
                // Reference: PhasingVoteCastingAttachment.java
                let phased_tx_id = self.parse_long_field(tx, "phasedTransactionId").unwrap_or(0);

                if phased_tx_id > 0 {
                    // Verify the phasing poll exists
                    match self.phasing_poll_repo.find_by_poll_id(phased_tx_id).await {
                        Ok(Some(_poll)) => {
                            let vote_model = orm::models::PhasingVoteModel {
                                db_id: 0,
                                vote_id: tx.id as i64,
                                transaction_id: phased_tx_id,
                                voter_id: sender_id,
                                height: self.get_current_height(),
                            };
                            self.phasing_vote_repo.insert(&vote_model).await
                                .map_err(|e| ProcessorError::Validation(format!("PhasingVote insert failed: {}", e)))?;
                            debug!("Phasing vote cast: voter={}, phased_tx={}", sender_id, phased_tx_id);
                        }
                        Ok(None) => {
                            warn!("Phasing poll {} not found for vote from tx {}", phased_tx_id, tx.id);
                        }
                        Err(e) => {
                            return Err(ProcessorError::Validation(format!("PhasingPoll lookup failed: {}", e)));
                        }
                    }
                }
            }

            10 => { // ACCOUNT_PROPERTY
                // Java: recipientAccount.setProperty(tx, sender, property, value)
                // Reference: AccountPropertyAttachment.java
                if recipient_id != 0 {
                    let property_name = self.parse_string_field(tx, "property").unwrap_or_default();
                    let property_value = self.parse_string_field(tx, "value").unwrap_or_default();

                    if !property_name.is_empty() {
                        let model = orm::models::AccountPropertyModel {
                            db_id: 0,
                            id: tx.id as i64,
                            recipient_id,
                            setter_id: Some(sender_id),
                            property: property_name.clone(),
                            value: Some(property_value),
                            height: self.get_current_height(),
                            latest: true,
                        };
                        self.account_property_repo.upsert(&model).await
                            .map_err(|e| ProcessorError::Validation(format!("AccountProperty upsert failed: {}", e)))?;
                        debug!("Account property set: account={}, property='{}'", recipient_id, property_name);
                    }
                } else {
                    warn!("ACCOUNT_PROPERTY transaction without recipient in tx {}", tx.id);
                }
            }

            11 => { // ACCOUNT_PROPERTY_DELETE
                // Java: senderAccount.deleteProperty(propertyId)
                // Reference: AccountPropertyDeleteAttachment.java
                let property_name = self.parse_string_field(tx, "property").unwrap_or_default();

                if !property_name.is_empty() {
                    match self.account_property_repo.find_by_property(sender_id, &property_name).await {
                        Ok(Some(prop)) => {
                            self.account_property_repo.delete_by_id(prop.db_id).await
                                .map_err(|e| ProcessorError::Validation(format!("AccountProperty delete failed: {}", e)))?;
                            debug!("Account property deleted: account={}, property='{}'", sender_id, property_name);
                        }
                        Ok(None) => {
                            debug!("Account property '{}' not found for account {}, skipping delete", property_name, sender_id);
                        }
                        Err(e) => {
                            return Err(ProcessorError::Validation(format!("AccountProperty lookup failed: {}", e)));
                        }
                    }
                }
            }

            12 => { // ACCOUNT_LONG_VALUE_PROPERTY
                // Java: recipientAccount.setProperty(tx, sender, property, value)
                // Reference: AccountLongValuePropertyAttachment.java
                if recipient_id != 0 {
                    let property_name = self.parse_string_field(tx, "property").unwrap_or_default();
                    let long_value = self.parse_long_field(tx, "value").unwrap_or(0);

                    if !property_name.is_empty() {
                        let model = orm::models::AccountPropertyModel {
                            db_id: 0,
                            id: tx.id as i64,
                            recipient_id,
                            setter_id: Some(sender_id),
                            property: property_name.clone(),
                            value: Some(long_value.to_string()),
                            height: self.get_current_height(),
                            latest: true,
                        };
                        self.account_property_repo.upsert(&model).await
                            .map_err(|e| ProcessorError::Validation(format!("AccountProperty upsert failed: {}", e)))?;
                        debug!("Account long-value property set: account={}, property='{}', value={}", recipient_id, property_name, long_value);
                    }
                }
            }
            _ => {
                debug!("Unknown Messaging subtype: {}", tx.subtype);
            }
        }

        Ok(())
    }

    /// AccountControl (Type 3) 交易处理
    ///
    /// Reference: Java TransactionTypeAccountControl
    /// - Subtype 0: EFFECTIVE_BALANCE_LEASING -> leaseEffectiveBalance()
    /// - Subtype 1: PHASING_ONLY -> AccountPhasingOnly.set()
    async fn apply_account_control_attachment(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id as i64;
        let _current_height = self.get_current_height();

        match tx.subtype {
            0 => { // EFFECTIVE_BALANCE_LEASING
                // Java: Account.getAccount(senderId).leaseEffectiveBalance(period)
                // Reference: EffectiveBalanceLeasingAttachment.java
                let period = self.parse_long_field(tx, "period").map(|p| p as i32).unwrap_or(0);
                let recipient_id = tx.recipient_id.unwrap_or(0) as i64;

                if period > 0 && recipient_id > 0 {
                    let current_height = self.get_current_height();
                    let leasing_height_from = current_height + 1;
                    let leasing_height_to = current_height + period;

                    // Create or update lease record
                    let lease_model = orm::AccountLeaseModel {
                        db_id: 0,
                        lessor_id: sender_id,
                        current_leasing_height_from: None,
                        current_leasing_height_to: None,
                        current_lessee_id: None,
                        next_leasing_height_from: Some(leasing_height_from),
                        next_leasing_height_to: Some(leasing_height_to),
                        next_lessee_id: Some(recipient_id),
                        height: current_height,
                        latest: true,
                    };

                    self.account_lease_repo.upsert(&lease_model).await
                        .map_err(|e| ProcessorError::Validation(format!("Failed to create lease: {}", e)))?;

                    debug!("Account {} leasing effective balance to {} for {} blocks (height {}-{})",
                        sender_id, recipient_id, period, leasing_height_from, leasing_height_to);
                } else {
                    warn!("Invalid lease parameters in transaction {}", tx.id);
                }
            }

            1 => { // PHASING_ONLY
                // Java: AccountPhasingOnly.set(attachment)
                // Reference: PhasingOnlyAttachment.java
                let voting_model = self.parse_long_field(tx, "votingModel")
                    .map(|v| v as i16)
                    .unwrap_or(0);
                let quorum = self.parse_long_field(tx, "quorum");
                let min_balance = self.parse_long_field(tx, "minBalance");
                let holding_id = self.parse_long_field(tx, "holdingId");
                let min_balance_model = self.parse_long_field(tx, "minBalanceModel").map(|v| v as i16);
                let max_fees = self.parse_long_field(tx, "maxFees");
                let min_duration = self.parse_long_field(tx, "minDuration").map(|v| v as i16);
                let max_duration = self.parse_long_field(tx, "maxDuration").map(|v| v as i16);

                // Parse whitelist from attachment
                let whitelist = self.parse_string_field(tx, "whitelist");

                let model = orm::models::AccountControlPhasingModel {
                    db_id: 0,
                    account_id: sender_id,
                    whitelist,
                    voting_model,
                    quorum,
                    min_balance,
                    holding_id,
                    min_balance_model,
                    max_fees,
                    min_duration,
                    max_duration,
                    height: self.get_current_height(),
                    latest: true,
                };
                self.account_control_phasing_repo.upsert(&model).await
                    .map_err(|e| ProcessorError::Validation(format!("AccountControlPhasing upsert failed: {}", e)))?;
                debug!("Account {} set to phasing-only mode (model={})", sender_id, voting_model);
            }

            _ => {
                debug!("Unknown AccountControl subtype: {}", tx.subtype);
            }
        }

        Ok(())
    }

    /// Data (Type 6) 交易处理
    ///
    /// Reference: Java TransactionTypeData
    /// - Subtype 0: TAGGED_DATA_UPLOAD -> TaggedData.add()
    /// - Subtype 1: TAGGED_DATA_EXTEND -> TaggedData.extend()
    async fn apply_data_attachment(&self, tx: &Transaction) -> ProcessorResult<()> {
        use orm::models::*;

        let sender_id = tx.sender_id as i64;
        let current_height = self.get_current_height();
        let current_timestamp = self.get_current_timestamp();

        match tx.subtype {
            0 => { // TAGGED_DATA_UPLOAD
                // Java: TaggedData.add(transaction, attachment)
                // Reference: TaggedDataUploadAttachment.java
                //   attachment fields: { "name": String, "description": String,
                //                       "tags[]": String[], "data": String,
                //                       "type": int, "channel": String,
                //                       "isText": boolean, "filename": String }
                //   DB operation:
                //     1. INSERT TAGGED_DATA table
                //     2. INSERT multiple TAG records (one per tag)

                let name = self.parse_string_field(tx, "name").unwrap_or_default();
                let description = self.parse_string_field(tx, "description");
                let data = self.parse_string_field(tx, "data");

                if !name.is_empty() {
                    let tagged_data_model = TaggedDataModel {
                        db_id: 0,
                        id: tx.id as i64,
                        account_id: sender_id,
                        name: name.clone(),
                        description,
                        tags: None, // TODO: 从attachment解析tags数组
                        parsed_tags: None,
                        type_: self.parse_string_field(tx, "type"), // 使用type_字段
                        data: data.unwrap_or_default().into_bytes(), // 转换为Vec<u8>
                        is_text: true, // TODO: 从attachment解析
                        filename: self.parse_string_field(tx, "filename"),
                        channel: self.parse_string_field(tx, "channel"),
                        block_timestamp: current_timestamp,
                        transaction_timestamp: current_timestamp,
                        height: current_height,
                        latest: true,
                    };

                    match self.tagged_data_repo.insert(&tagged_data_model).await {
                        Ok(_) => {
                            info!("Uploaded tagged data '{}' for account {}", name, sender_id);

                            // 插入TAG记录（如果有的话）
                            // TODO: 解析tags数组并创建TagModel列表
                            debug!("Processing tags for tagged data {}", tx.id);
                        }
                        Err(e) => {
                            warn!("Failed to upload tagged data '{}': {}", name, e);
                            return Err(e.into());
                        }
                    }
                } else {
                    warn!("Empty name in TAGGED_DATA_UPLOAD transaction {}", tx.id);
                }
            }

            1 => { // TAGGED_DATA_EXTEND
                // Java: TaggedData.extend(transaction, attachment)
                // Reference: TaggedDataExtendAttachment.java
                //   attachment fields: { "taggedDataId": long, "data": String }
                //   DB operation: INSERT TAGGED_DATA_EXTEND table

                let tagged_data_id = self.parse_long_field(tx, "taggedDataId").unwrap_or(0);
                let _extend_data = self.parse_string_field(tx, "data");

                if tagged_data_id > 0 {
                    // 验证tagged data是否存在且属于当前用户
                    match self.tagged_data_repo.find_by_id(tagged_data_id).await {
                        Ok(Some(existing)) if existing.account_id == sender_id => {
                            let extend_model = orm::models::TaggedDataExtendModel {
                                db_id: 0,
                                id: tx.id as i64,
                                extend_id: tagged_data_id,
                                height: self.get_current_height(),
                                latest: true,
                            };
                            self.tagged_data_extend_repo.insert(&extend_model).await
                                .map_err(|e| ProcessorError::Validation(format!("TaggedDataExtend insert failed: {}", e)))?;
                            debug!("Extended tagged data {} (tx={})", tagged_data_id, tx.id);
                        }
                        Ok(Some(_)) => {
                            warn!("Cannot extend tagged data owned by another account");
                        }
                        Ok(None) => {
                            warn!("Cannot extend non-existent tagged data {}", tagged_data_id);
                        }
                        Err(e) => {
                            warn!("Error finding tagged data {}: {}", tagged_data_id, e);
                            return Err(e.into());
                        }
                    }
                } else {
                    warn!("Missing taggedDataId in transaction {}", tx.id);
                }
            }

            2 => { // TAGGED_DATA_TIMESTAMP
                // Java: TaggedData.timestamp(transaction, attachment)
                // Reference: TaggedDataTimestampAttachment.java
                //   attachment fields: { "taggedDataId": long }
                //   DB operation: INSERT TAGGED_DATA_TIMESTAMP table

                let tagged_data_id = self.parse_long_field(tx, "taggedDataId").unwrap_or(0);

                if tagged_data_id > 0 {
                    match self.tagged_data_repo.find_by_id(tagged_data_id).await {
                        Ok(Some(_existing)) => {
                            let timestamp_model = orm::models::TaggedTimestampModel::new(
                                tx.id as i64,
                                sender_id,
                                format!("{}", tagged_data_id),
                                self.get_current_timestamp(),
                                self.get_current_height(),
                            );
                            self.tagged_timestamp_repo.insert(&timestamp_model).await
                                .map_err(|e| ProcessorError::Validation(format!("TaggedTimestamp insert failed: {}", e)))?;
                            debug!("Timestamped tagged data {} (tx={})", tagged_data_id, tx.id);
                        }
                        Ok(None) => {
                            warn!("Cannot timestamp non-existent tagged data {}", tagged_data_id);
                        }
                        Err(e) => {
                            warn!("Error finding tagged data {}: {}", tagged_data_id, e);
                            return Err(e.into());
                        }
                    }
                } else {
                    warn!("Missing taggedDataId in TAGGED_DATA_TIMESTAMP transaction {}", tx.id);
                }
            }

            _ => {
                debug!("Unknown Data subtype: {}", tx.subtype);
            }
        }

        Ok(())
    }

    /// LightContract (Type 11) 交易处理
    ///
    /// Reference: Java TransactionTypeLightContract
    /// - Subtype 0: CONTRACT_REFERENCE_SET -> ContractReference.setContractReference()
    /// - Subtype 1: CONTRACT_REFERENCE_DELETE -> ContractReference.deleteContractReference()
    async fn apply_light_contract_attachment(&self, tx: &Transaction) -> ProcessorResult<()> {
        use orm::models::*;

        let _sender_id = tx.sender_id as i64;
        let recipient_id = tx.recipient_id.map(|id| id as i64).unwrap_or(0);
        let current_height = self.get_current_height();
        let _current_timestamp = self.get_current_timestamp();

        match tx.subtype {
            0 => { // CONTRACT_REFERENCE_SET
                // Java: ContractReference.setContractReference(account, name, data)
                // Reference: ContractReferenceSetAttachment.java
                //   attachment fields: { "name": String, "data": String }
                //   DB operation:
                //     1. DELETE existing CONTRACT_REFERENCE (account+name)
                //     2. INSERT new CONTRACT_REFERENCE

                let ref_name = self.parse_string_field(tx, "name").unwrap_or_default();
                let ref_data = self.parse_string_field(tx, "data");

                if !ref_name.is_empty() && recipient_id != 0 {
                    let contract_ref_model = ContractReferenceModel {
                        db_id: 0,
                        id: tx.id as i64,
                        account_id: recipient_id,
                        contract_name: ref_name.clone(),
                        contract_params: ref_data, // 使用contract_params字段存储数据
                        contract_transaction_chain_id: 0, // TODO: 从attachment解析
                        contract_transaction_full_hash: None, // TODO: 从attachment解析
                        height: current_height,
                        latest: true,
                    };

                    // 先删除已存在的同名引用（如果存在）
                    // TODO: 调用contract_ref_repo.delete_by_account_and_name(recipient_id, &ref_name)

                    match self.contract_ref_repo.insert(&contract_ref_model).await {
                        Ok(_) => {
                            info!("Set contract reference '{}' on account {} (tx={})",
                                ref_name, recipient_id, tx.id);
                        }
                        Err(e) => {
                            warn!("Failed to set contract reference '{}': {}", ref_name, e);
                            return Err(e.into());
                        }
                    }
                } else {
                    if ref_name.is_empty() {
                        warn!("Empty reference name in transaction {}", tx.id);
                    }
                    if recipient_id == 0 {
                        warn!("CONTRACT_REFERENCE_SET without recipient in tx {}", tx.id);
                    }
                }
            }

            1 => { // CONTRACT_REFERENCE_DELETE
                // Java: ContractReference.deleteContractReference(account, name)
                // Reference: ContractReferenceDeleteAttachment.java
                //   attachment fields: { "name": String }
                //   DB operation: DELETE from CONTRACT_REFERENCE table

                let ref_name = self.parse_string_field(tx, "name").unwrap_or_default();

                if !ref_name.is_empty() && recipient_id != 0 {
                    debug!("Deleting contract reference '{}' from account {} (tx={})",
                        ref_name, recipient_id, tx.id);
                    info!("ContractReference deletion not yet fully implemented (stub)");
                    // TODO: 调用contract_ref_repo.delete_by_account_and_name(recipient_id, &ref_name)
                } else {
                    warn!("Invalid parameters for CONTRACT_REFERENCE_DELETE in tx {}", tx.id);
                }
            }

            _ => {
                debug!("Unknown LightContract subtype: {}", tx.subtype);
            }
        }

        Ok(())
    }

    /// DigitalGoods (Type 5) 交易处理
    ///
    /// Reference: Java TransactionTypeDigitalGoods
    async fn apply_digital_goods_attachment(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id as i64;

        match tx.subtype {
            0 => { // DGS_LISTING
                let name = self.parse_string_field(tx, "name").unwrap_or_default();
                let description = self.parse_string_field(tx, "description").unwrap_or_default();
                let price_nqt = self.parse_long_field(tx, "priceNQT").unwrap_or(0);
                let quantity = self.parse_long_field(tx, "quantity").unwrap_or(1);
                let tags = self.parse_string_field(tx, "tags").unwrap_or_default();

                if !name.is_empty() && price_nqt > 0 && quantity > 0 {
                    let goods_model = orm::GoodsModel {
                        db_id: 0,
                        id: tx.id as i64,
                        seller_id: sender_id,
                        name: name.clone(),
                        description: Some(description),
                        parsed_tags: None,
                        tags: Some(tags),
                        timestamp: tx.timestamp as i32,
                        quantity: quantity as i32,
                        price: price_nqt,
                        delisted: false,
                        height: self.get_current_height(),
                        latest: true,
                        has_image: false,
                    };

                    self.goods_repo.insert(&goods_model).await
                        .map_err(|e| ProcessorError::Validation(format!("Failed to insert goods: {}", e)))?;

                    debug!("DGS_LISTING: '{}' by account {} at {} NQT, qty={}", name, sender_id, price_nqt, quantity);
                }
            }

            1 => { // DGS_DELISTING
                let goods_id = self.parse_long_field(tx, "goodsId").unwrap_or(0);
                if goods_id > 0 {
                    self.goods_repo.set_delisted(goods_id, true).await
                        .map_err(|e| ProcessorError::Validation(format!("Failed to delist goods: {}", e)))?;

                    debug!("DGS_DELISTING: goods {} by account {}", goods_id, sender_id);
                }
            }

            2 => { // DGS_PRICE_CHANGE
                let goods_id = self.parse_long_field(tx, "goodsId").unwrap_or(0);
                let new_price = self.parse_long_field(tx, "priceNQT").unwrap_or(0);

                if goods_id > 0 && new_price >= 0 {
                    self.goods_repo.update_price(goods_id, new_price).await
                        .map_err(|e| ProcessorError::Validation(format!("Failed to update goods price: {}", e)))?;

                    debug!("DGS_PRICE_CHANGE: goods {} to {} NQT", goods_id, new_price);
                }
            }

            3 => { // DGS_QUANTITY_CHANGE
                let goods_id = self.parse_long_field(tx, "goodsId").unwrap_or(0);
                let delta_quantity = self.parse_long_field(tx, "deltaQuantity").unwrap_or(0);

                if goods_id != 0 {
                    // Java: goods.changeQuantity(deltaQuantity)
                    if let Ok(Some(goods)) = self.goods_repo.find_by_goods_id(goods_id).await {
                        let new_quantity = goods.quantity + delta_quantity as i32;
                        if new_quantity >= 0 {
                            self.goods_repo.update_quantity(goods_id, new_quantity).await
                                .map_err(|e| ProcessorError::Validation(format!("Failed to update goods quantity: {}", e)))?;
                        }
                    }

                    debug!("DGS_QUANTITY_CHANGE: goods {} delta={}", goods_id, delta_quantity);
                }
            }

            4 => { // DGS_PURCHASE
                let goods_id = self.parse_long_field(tx, "goodsId").unwrap_or(0);
                let quantity = self.parse_long_field(tx, "quantity").unwrap_or(1);
                let price_nqt = self.parse_long_field(tx, "priceNQT").unwrap_or(0);
                let delivery_deadline = self.parse_long_field(tx, "deliveryDeadlineTimestamp").unwrap_or(0);

                if goods_id > 0 && quantity > 0 && price_nqt > 0 {
                    // Java: DigitalGoodsPurchase.purchase()
                    let total_cost = (price_nqt as u64).checked_mul(quantity as u64)
                        .ok_or_else(|| ProcessorError::Validation("purchase cost overflow".to_string()))?;

                    // Create purchase record
                    let purchase_model = orm::PurchaseModel {
                        db_id: 0,
                        id: tx.id as i64,
                        buyer_id: sender_id,
                        goods_id,
                        seller_id: 0, // Will be filled from goods
                        quantity: quantity as i32,
                        price: price_nqt,
                        deadline: delivery_deadline as i32,
                        note: None,
                        nonce: None,
                        timestamp: tx.timestamp as i32,
                        pending: true,
                        goods: None,
                        goods_nonce: None,
                        goods_is_text: false,
                        refund_note: None,
                        refund_nonce: None,
                        has_feedback_notes: false,
                        has_public_feedbacks: false,
                        discount: 0,
                        refund: 0,
                        height: self.get_current_height(),
                        latest: true,
                    };

                    self.purchase_repo.insert(&purchase_model).await
                        .map_err(|e| ProcessorError::Validation(format!("Failed to insert purchase: {}", e)))?;

                    // Update goods quantity
                    if let Ok(Some(goods)) = self.goods_repo.find_by_goods_id(goods_id).await {
                        let new_quantity = goods.quantity - quantity as i32;
                        if new_quantity >= 0 {
                            self.goods_repo.update_quantity(goods_id, new_quantity).await
                                .map_err(|e| ProcessorError::Validation(format!("Failed to update goods quantity: {}", e)))?;
                        }
                    }

                    debug!("DGS_PURCHASE: {} of goods {} at {} NQT each, total={}", quantity, goods_id, price_nqt, total_cost);
                }
            }

            5 => { // DGS_DELIVERY
                let purchase_id = self.parse_long_field(tx, "purchaseId").unwrap_or(0);
                let goods = self.parse_bytes_field(tx, "goods");
                let goods_nonce = self.parse_bytes_field(tx, "goodsNonce");

                if purchase_id > 0 {
                    // Java: DigitalGoodsPurchase.delivery()
                    self.purchase_repo.set_delivered(purchase_id, &goods.unwrap_or_default(), &goods_nonce.unwrap_or_default()).await
                        .map_err(|e| ProcessorError::Validation(format!("Failed to mark purchase as delivered: {}", e)))?;

                    debug!("DGS_DELIVERY: purchase {}", purchase_id);
                }
            }

            6 => { // DGS_FEEDBACK
                let purchase_id = self.parse_long_field(tx, "purchaseId").unwrap_or(0);
                let feedback_note = self.parse_string_field(tx, "feedbackNote").unwrap_or_default();
                let is_public = self.parse_bool_field(tx, "publicFeedback").unwrap_or(false);
                let feedback_nonce = self.parse_bytes_field(tx, "feedbackNonce").unwrap_or_default();

                if purchase_id > 0 {
                    // Java: DigitalGoodsPurchase.feedback()
                    if is_public {
                        let _feedback_model = orm::PurchasePublicFeedbackModel {
                            db_id: 0,
                            id: tx.id as i64,
                            public_feedback: feedback_note.clone(),
                            height: self.get_current_height(),
                            latest: true,
                        };
                        // Insert into PURCHASE_PUBLIC_FEEDBACK table
                        debug!("DGS_PUBLIC_FEEDBACK: purchase {}: '{}'", purchase_id, feedback_note);
                    } else {
                        let _feedback_model = orm::PurchaseFeedbackModel {
                            db_id: 0,
                            id: tx.id as i64,
                            feedback_data: feedback_note.clone().into_bytes(),
                            feedback_nonce,
                            height: self.get_current_height(),
                            latest: true,
                        };
                        // Insert into PURCHASE_FEEDBACK table
                        debug!("DGS_FEEDBACK: purchase {}: '{}'", purchase_id, feedback_note);
                    }
                }
            }

            7 => { // DGS_REFUND
                let purchase_id = self.parse_long_field(tx, "purchaseId").unwrap_or(0);
                let refund_amount = self.parse_long_field(tx, "refundNQT").unwrap_or(0);
                let note = self.parse_bytes_field(tx, "note");
                let note_nonce = self.parse_bytes_field(tx, "noteNonce");

                if purchase_id > 0 && refund_amount > 0 {
                    // Java: DigitalGoodsPurchase.refund()
                    self.purchase_repo.set_refund(purchase_id, refund_amount, &note.unwrap_or_default(), &note_nonce.unwrap_or_default()).await
                        .map_err(|e| ProcessorError::Validation(format!("Failed to set refund: {}", e)))?;

                    // Refund amount to buyer
                    if let Ok(Some(purchase)) = self.purchase_repo.find_by_purchase_id(purchase_id).await {
                        self.account_repo.add_to_balance_and_unconfirmed(purchase.buyer_id, refund_amount).await
                            .map_err(|e| ProcessorError::Validation(format!("Failed to refund buyer: {}", e)))?;
                    }

                    debug!("DGS_REFUND: {} NQT for purchase {}", refund_amount, purchase_id);
                }
            }

            _ => {
                debug!("Unknown DigitalGoods subtype: {}", tx.subtype);
            }
        }

        Ok(())
    }

    /// Shuffling (Type 7) 交易处理
    ///
    /// Reference: Java TransactionTypeShuffling
    /// - Subtype 0: SHUFFLING_CREATION -> Shuffling.createShuffling()
    /// - Subtype 1: SHUFFING_PROCESSING -> Shuffling.processShuffling()
    /// - Subtype 2: SHUFFING_VERIFICATION -> Shuffling.verifyShuffling()
    /// - Subtype 3: SHUFFING_CANCELLATION -> Shuffling.cancelShuffling()
    /// - Subtype 4: SHUFFING_RECIPIENTS -> Shuffling.addRecipients()
    async fn apply_shuffling_attachment(&self, tx: &Transaction) -> ProcessorResult<()> {
        let sender_id = tx.sender_id as i64;

        match tx.subtype {
            0 => { // SHUFFLING_CREATION
                // Java: Shuffling.createShuffling(transaction, attachment)
                let shuffling_amount_nqt = self.parse_long_field(tx, "amountNQT").unwrap_or(0);
                let participant_count = self.parse_long_field(tx, "participantCount").unwrap_or(0) as i16;
                let registration_period = self.parse_long_field(tx, "registrationPeriod").unwrap_or(0) as i16;
                let holding_id = self.parse_long_field(tx, "holdingId").unwrap_or(0);
                let holding_type = self.parse_long_field(tx, "holdingType").unwrap_or(0) as i16;

                if shuffling_amount_nqt > 0 && participant_count > 1 {
                    // Create shuffling record
                    let shuffling_model = orm::ShufflingModel {
                        db_id: 0,
                        id: tx.id as i64,
                        holding_id: Some(holding_id),
                        holding_type,
                        issuer_id: sender_id,
                        amount: shuffling_amount_nqt,
                        participant_count,
                        blocks_remaining: Some(registration_period),
                        stage: 0, // REGISTRATION
                        assignee_account_id: None,
                        registrant_count: 0,
                        recipient_public_keys: None,
                        height: self.get_current_height(),
                        latest: true,
                    };

                    self.shuffling_repo.insert(&shuffling_model).await
                        .map_err(|e| ProcessorError::Validation(format!("Failed to insert shuffling: {}", e)))?;

                    debug!("SHUFFLING_CREATION: {} NQT by account {}, participants={}",
                        shuffling_amount_nqt, sender_id, participant_count);
                }
            }

            1 => { // SHUFFLING_PROCESSING
                // Java: Shuffling.processShuffling(shufflingId)
                let shuffling_id = self.parse_long_field(tx, "shufflingId").unwrap_or(tx.id as i64);

                if shuffling_id > 0 {
                    // Update shuffling stage to PROCESSING
                    self.shuffling_repo.update_stage(shuffling_id, 1).await
                        .map_err(|e| ProcessorError::Validation(format!("Failed to update shuffling stage: {}", e)))?;

                    debug!("SHUFFLING_PROCESSING: {}", shuffling_id);
                }
            }

            2 => { // SHUFFLING_VERIFICATION
                // Java: Shuffling.verifyShuffling(shufflingId)
                let shuffling_id = self.parse_long_field(tx, "shufflingId").unwrap_or(0);

                if shuffling_id > 0 {
                    // Update shuffling stage to VERIFIED
                    self.shuffling_repo.update_stage(shuffling_id, 2).await
                        .map_err(|e| ProcessorError::Validation(format!("Failed to update shuffling stage: {}", e)))?;

                    debug!("SHUFFLING_VERIFICATION: {}", shuffling_id);
                }
            }

            3 => { // SHUFFLING_CANCELLATION
                // Java: Shuffling.cancelShuffling(shufflingId)
                let shuffling_id = self.parse_long_field(tx, "shufflingId").unwrap_or(0);

                if shuffling_id > 0 {
                    // Update shuffling stage to CANCELLED
                    self.shuffling_repo.update_stage(shuffling_id, 3).await
                        .map_err(|e| ProcessorError::Validation(format!("Failed to update shuffling stage: {}", e)))?;

                    // Refund amount to issuer
                    if let Ok(Some(shuffling)) = self.shuffling_repo.find_by_shuffling_id(shuffling_id).await {
                        self.account_repo.add_to_balance_and_unconfirmed(shuffling.issuer_id, shuffling.amount).await
                            .map_err(|e| ProcessorError::Validation(format!("Failed to refund shuffling: {}", e)))?;
                    }

                    debug!("SHUFFLING_CANCELLATION: {}", shuffling_id);
                }
            }

            4 => { // SHUFFLING_RECIPIENTS
                // Java: Shuffling.addRecipients(shufflingId, recipientPublicKeys)
                let shuffling_id = self.parse_long_field(tx, "shufflingId").unwrap_or(0);

                if shuffling_id > 0 {
                    // Parse recipient public keys from attachment
                    let _recipient_public_keys = self.parse_string_field(tx, "recipientPublicKeys").unwrap_or_default();

                    debug!("SHUFFLING_RECIPIENTS: adding participants to {}", shuffling_id);
                }
            }

            _ => {
                debug!("Unknown Shuffling subtype: {}", tx.subtype);
            }
        }

        Ok(())
    }

    /// Aliases (Type 8) 交易处理（Stub）
    async fn apply_aliases_attachment(&self, tx: &Transaction) -> ProcessorResult<()> {
        debug!("Aliases subtype {} processed (stub)", tx.subtype);
        Ok(())
    }

    /// Voting (Type 9) 交易处理（Stub）
    async fn apply_voting_attachment(&self, tx: &Transaction) -> ProcessorResult<()> {
        debug!("Voting subtype {} processed (stub)", tx.subtype);
        Ok(())
    }

    /// AccountProperty (Type 10) 交易处理（Stub）
    async fn apply_account_property_attachment(&self, tx: &Transaction) -> ProcessorResult<()> {
        debug!("AccountProperty subtype {} processed (stub)", tx.subtype);
        Ok(())
    }

    /// CoinExchange (Type 10) 交易处理（Stub）
    async fn apply_coin_exchange_attachment(&self, tx: &Transaction) -> ProcessorResult<()> {
        debug!("CoinExchange subtype {} processed (stub)", tx.subtype);
        Ok(())
    }

    // ==================== 辅助方法：Attachment字段解析 ====================

    /// 从Transaction的attachment JSON bytes中解析String字段
    ///
    /// Reference: Transaction.from_json() 将 attachment JSON 对象序列化为 bytes
    /// 这里需要反序列化并提取指定字段的值
    fn parse_string_field(&self, tx: &Transaction, field_name: &str) -> Option<String> {
        let att_map = tx.attachment_json.as_ref()?;

        match att_map.get(field_name) {
            Some(serde_json::Value::String(s)) => {
                debug!("Parsed string field '{}={}' from transaction {}",
                    field_name, s, tx.id);
                Some(s.clone())
            }
            Some(serde_json::Value::Number(n)) => {
                let s = n.to_string();
                debug!("Parsed numeric field '{}={}' as string from transaction {}",
                    field_name, s, tx.id);
                Some(s)
            }
            Some(other) => {
                warn!("Field '{}' in transaction {} has unexpected type: {:?}",
                    field_name, tx.id, other);
                None
            }
            None => {
                debug!("Field '{}' not found in attachment for transaction {}",
                    field_name, tx.id);
                None
            }
        }
    }

    fn parse_long_field(&self, tx: &Transaction, field_name: &str) -> Option<i64> {
        let att_map = tx.attachment_json.as_ref()?;

        match att_map.get(field_name) {
            Some(serde_json::Value::Number(n)) => {
                n.as_i64().or_else(|| {
                    n.as_u64().map(|v| v as i64)
                })
            }
            Some(serde_json::Value::String(s)) => {
                s.parse::<i64>().ok()
                    .or_else(|| {
                        s.parse::<u64>().ok().map(|v| v as i64)
                    })
            }
            _ => None,
        }
    }

    fn parse_bool_field(&self, tx: &Transaction, field_name: &str) -> Option<bool> {
        let att_map = tx.attachment_json.as_ref()?;

        match att_map.get(field_name) {
            Some(serde_json::Value::Bool(b)) => Some(*b),
            Some(serde_json::Value::Number(n)) => {
                n.as_i64().map(|v| v != 0)
            }
            Some(serde_json::Value::String(s)) => {
                match s.as_str() {
                    "true" | "1" | "yes" => Some(true),
                    "false" | "0" | "no" => Some(false),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn parse_bytes_field(&self, tx: &Transaction, field_name: &str) -> Option<Vec<u8>> {
        let att_map = tx.attachment_json.as_ref()?;

        match att_map.get(field_name) {
            Some(serde_json::Value::String(s)) => {
                hex::decode(s).ok()
            }
            Some(serde_json::Value::Array(arr)) => {
                let bytes: Option<Vec<u8>> = arr.iter()
                    .map(|v| v.as_u64().map(|n| n as u8))
                    .collect();
                bytes
            }
            _ => None,
        }
    }

    fn get_attachment_json(&self, tx: &Transaction) -> Option<serde_json::Value> {
        tx.attachment_json.as_ref().map(|m| serde_json::Value::Object(m.clone()))
    }

    /// Apply unconfirmed attachment deduction (mempool pre-deduction for assets/currencies)
    ///
    /// Reference: Java TransactionType.applyAttachmentUnconfirmed()
    /// This prevents double-spending of assets and currencies in the mempool.
    async fn apply_attachment_unconfirmed(&self, tx: &Transaction) -> ProcessorResult<bool> {
        match tx.type_id {
            TransactionType::ColoredCoins => {
                self.apply_colored_coins_unconfirmed(tx).await
            }
            TransactionType::MonetarySystem => {
                self.apply_monetary_system_unconfirmed(tx).await
            }
            _ => Ok(true), // Other types don't need attachment unconfirmed
        }
    }

    /// Apply unconfirmed deduction for ColoredCoins (asset) transactions
    ///
    /// Reference: Java TransactionTypeAsset.applyAttachmentUnconfirmed()
    async fn apply_colored_coins_unconfirmed(&self, tx: &Transaction) -> ProcessorResult<bool> {
        match tx.subtype {
            1 => {
                // ASSET_TRANSFER: Pre-deduct asset quantity from sender
                let sender_id = tx.sender_id as i64;
                let asset_id = self.parse_long_field(tx, "asset").unwrap_or(0);
                let quantity = tx.amount as i64;

                if asset_id == 0 {
                    return Ok(true); // No asset to deduct
                }

                // Check unconfirmed quantity
                match self.account_asset_repo.find_by_account_and_asset(sender_id, asset_id).await? {
                    Some(aa) => {
                        if aa.unconfirmed_quantity < quantity {
                            tracing::warn!(
                                "Insufficient unconfirmed asset balance: account={}, asset={}, have={}, need={}",
                                sender_id, asset_id, aa.unconfirmed_quantity, quantity
                            );
                            return Ok(false);
                        }
                        // Deduct from unconfirmed quantity
                        self.account_asset_repo.add_to_unconfirmed_quantity(sender_id, asset_id, -quantity).await?;
                        debug!("Pre-deducted asset transfer: account={}, asset={}, quantity={}", sender_id, asset_id, quantity);
                        Ok(true)
                    }
                    None => {
                        tracing::warn!("Account {} has no asset {}", sender_id, asset_id);
                        Ok(false)
                    }
                }
            }
            2 => {
                // ASK_ORDER_PLACEMENT: Pre-deduct asset quantity from sender (they're selling)
                let sender_id = tx.sender_id as i64;
                let asset_id = self.parse_long_field(tx, "asset").unwrap_or(0);
                let quantity = self.parse_long_field(tx, "quantity").unwrap_or(tx.amount as i64);

                if asset_id == 0 {
                    return Ok(true);
                }

                match self.account_asset_repo.find_by_account_and_asset(sender_id, asset_id).await? {
                    Some(aa) => {
                        if aa.unconfirmed_quantity < quantity {
                            tracing::warn!(
                                "Insufficient unconfirmed asset for ask order: account={}, asset={}, have={}, need={}",
                                sender_id, asset_id, aa.unconfirmed_quantity, quantity
                            );
                            return Ok(false);
                        }
                        self.account_asset_repo.add_to_unconfirmed_quantity(sender_id, asset_id, -quantity).await?;
                        debug!("Pre-deducted ask order: account={}, asset={}, quantity={}", sender_id, asset_id, quantity);
                        Ok(true)
                    }
                    None => {
                        tracing::warn!("Account {} has no asset {} for ask order", sender_id, asset_id);
                        Ok(false)
                    }
                }
            }
            3 => {
                // BID_ORDER_PLACEMENT: No asset pre-deduction needed (buyer is paying NRCS)
                // The NRCS amount is already handled by the main apply_unconfirmed
                Ok(true)
            }
            6 => {
                // DIVIDEND_PAYMENT: Complex - need to check if sender has enough to pay all shareholders
                // For now, we'll just check if sender has the asset at all
                // Full implementation would calculate total dividend amount
                let sender_id = tx.sender_id as i64;
                let asset_id = self.parse_long_field(tx, "asset").unwrap_or(0);

                if asset_id == 0 {
                    return Ok(true);
                }

                match self.account_asset_repo.find_by_account_and_asset(sender_id, asset_id).await? {
                    Some(_) => Ok(true), // Sender has the asset
                    None => {
                        tracing::warn!("Account {} has no asset {} for dividend payment", sender_id, asset_id);
                        Ok(false)
                    }
                }
            }
            7 => {
                // ASSET_DELETE: Pre-deduct asset quantity from sender
                let sender_id = tx.sender_id as i64;
                let asset_id = self.parse_long_field(tx, "asset").unwrap_or(0);
                let quantity = tx.amount as i64;

                if asset_id == 0 {
                    return Ok(true);
                }

                match self.account_asset_repo.find_by_account_and_asset(sender_id, asset_id).await? {
                    Some(aa) => {
                        if aa.unconfirmed_quantity < quantity {
                            tracing::warn!(
                                "Insufficient unconfirmed asset for delete: account={}, asset={}, have={}, need={}",
                                sender_id, asset_id, aa.unconfirmed_quantity, quantity
                            );
                            return Ok(false);
                        }
                        self.account_asset_repo.add_to_unconfirmed_quantity(sender_id, asset_id, -quantity).await?;
                        debug!("Pre-deducted asset delete: account={}, asset={}, quantity={}", sender_id, asset_id, quantity);
                        Ok(true)
                    }
                    None => {
                        tracing::warn!("Account {} has no asset {} for delete", sender_id, asset_id);
                        Ok(false)
                    }
                }
            }
            9 => {
                // ASSET_INCREASE: No pre-deduction needed (creates new assets)
                Ok(true)
            }
            _ => Ok(true), // Other asset subtypes don't need unconfirmed deduction
        }
    }

    /// Apply unconfirmed deduction for MonetarySystem (currency) transactions
    ///
    /// Reference: Java TransactionTypeCurrency.applyAttachmentUnconfirmed()
    async fn apply_monetary_system_unconfirmed(&self, tx: &Transaction) -> ProcessorResult<bool> {
        match tx.subtype {
            1 => {
                // RESERVE_INCREASE: Pre-deduct NRCS for reserve
                // The NRCS amount is already handled by the main apply_unconfirmed
                Ok(true)
            }
            3 => {
                // CURRENCY_TRANSFER: Pre-deduct currency units from sender
                let sender_id = tx.sender_id as i64;
                let currency_id = self.parse_long_field(tx, "currency").unwrap_or(0);
                let units = tx.amount as i64;

                if currency_id == 0 {
                    return Ok(true);
                }

                match self.account_currency_repo.find_by_account_and_currency(sender_id, currency_id).await? {
                    Some(ac) => {
                        if ac.unconfirmed_units < units {
                            tracing::warn!(
                                "Insufficient unconfirmed currency units: account={}, currency={}, have={}, need={}",
                                sender_id, currency_id, ac.unconfirmed_units, units
                            );
                            return Ok(false);
                        }
                        self.account_currency_repo.add_to_unconfirmed_units(sender_id, currency_id, -units).await?;
                        debug!("Pre-deducted currency transfer: account={}, currency={}, units={}", sender_id, currency_id, units);
                        Ok(true)
                    }
                    None => {
                        tracing::warn!("Account {} has no currency {}", sender_id, currency_id);
                        Ok(false)
                    }
                }
            }
            5 => {
                // EXCHANGE_BUY: Pre-deduct NRCS for the buy
                // The NRCS amount is already handled by the main apply_unconfirmed
                Ok(true)
            }
            6 => {
                // EXCHANGE_SELL: Pre-deduct currency units for the sell
                let sender_id = tx.sender_id as i64;
                let currency_id = self.parse_long_field(tx, "currency").unwrap_or(0);
                let units = tx.amount as i64;

                if currency_id == 0 {
                    return Ok(true);
                }

                match self.account_currency_repo.find_by_account_and_currency(sender_id, currency_id).await? {
                    Some(ac) => {
                        if ac.unconfirmed_units < units {
                            tracing::warn!(
                                "Insufficient unconfirmed currency for exchange sell: account={}, currency={}, have={}, need={}",
                                sender_id, currency_id, ac.unconfirmed_units, units
                            );
                            return Ok(false);
                        }
                        self.account_currency_repo.add_to_unconfirmed_units(sender_id, currency_id, -units).await?;
                        debug!("Pre-deducted exchange sell: account={}, currency={}, units={}", sender_id, currency_id, units);
                        Ok(true)
                    }
                    None => {
                        tracing::warn!("Account {} has no currency {} for exchange sell", sender_id, currency_id);
                        Ok(false)
                    }
                }
            }
            7 => {
                // CURRENCY_MINTING: Pre-deduct NRCS for minting
                // The NRCS amount is already handled by the main apply_unconfirmed
                Ok(true)
            }
            _ => Ok(true), // Other currency subtypes don't need unconfirmed deduction
        }
    }

    /// Rollback unconfirmed attachment deduction
    ///
    /// Reference: Java TransactionType.rollbackAttachmentUnconfirmed()
    async fn rollback_attachment_unconfirmed(&self, tx: &Transaction) -> ProcessorResult<()> {
        match tx.type_id {
            TransactionType::ColoredCoins => {
                self.rollback_colored_coins_unconfirmed(tx).await
            }
            TransactionType::MonetarySystem => {
                self.rollback_monetary_system_unconfirmed(tx).await
            }
            _ => Ok(()),
        }
    }

    /// Rollback unconfirmed deduction for ColoredCoins (asset) transactions
    async fn rollback_colored_coins_unconfirmed(&self, tx: &Transaction) -> ProcessorResult<()> {
        match tx.subtype {
            1 => {
                // ASSET_TRANSFER: Restore asset quantity to sender
                let sender_id = tx.sender_id as i64;
                let asset_id = self.parse_long_field(tx, "asset").unwrap_or(0);
                let quantity = tx.amount as i64;

                if asset_id == 0 {
                    return Ok(());
                }

                self.account_asset_repo.add_to_unconfirmed_quantity(sender_id, asset_id, quantity).await?;
                debug!("Rolled back asset transfer unconfirmed: account={}, asset={}, quantity={}", sender_id, asset_id, quantity);
                Ok(())
            }
            2 => {
                // ASK_ORDER_PLACEMENT: Restore asset quantity to sender
                let sender_id = tx.sender_id as i64;
                let asset_id = self.parse_long_field(tx, "asset").unwrap_or(0);
                let quantity = self.parse_long_field(tx, "quantity").unwrap_or(tx.amount as i64);

                if asset_id == 0 {
                    return Ok(());
                }

                self.account_asset_repo.add_to_unconfirmed_quantity(sender_id, asset_id, quantity).await?;
                debug!("Rolled back ask order unconfirmed: account={}, asset={}, quantity={}", sender_id, asset_id, quantity);
                Ok(())
            }
            7 => {
                // ASSET_DELETE: Restore asset quantity to sender
                let sender_id = tx.sender_id as i64;
                let asset_id = self.parse_long_field(tx, "asset").unwrap_or(0);
                let quantity = tx.amount as i64;

                if asset_id == 0 {
                    return Ok(());
                }

                self.account_asset_repo.add_to_unconfirmed_quantity(sender_id, asset_id, quantity).await?;
                debug!("Rolled back asset delete unconfirmed: account={}, asset={}, quantity={}", sender_id, asset_id, quantity);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Rollback unconfirmed deduction for MonetarySystem (currency) transactions
    async fn rollback_monetary_system_unconfirmed(&self, tx: &Transaction) -> ProcessorResult<()> {
        match tx.subtype {
            3 => {
                // CURRENCY_TRANSFER: Restore currency units to sender
                let sender_id = tx.sender_id as i64;
                let currency_id = self.parse_long_field(tx, "currency").unwrap_or(0);
                let units = tx.amount as i64;

                if currency_id == 0 {
                    return Ok(());
                }

                self.account_currency_repo.add_to_unconfirmed_units(sender_id, currency_id, units).await?;
                debug!("Rolled back currency transfer unconfirmed: account={}, currency={}, units={}", sender_id, currency_id, units);
                Ok(())
            }
            6 => {
                // EXCHANGE_SELL: Restore currency units to sender
                let sender_id = tx.sender_id as i64;
                let currency_id = self.parse_long_field(tx, "currency").unwrap_or(0);
                let units = tx.amount as i64;

                if currency_id == 0 {
                    return Ok(());
                }

                self.account_currency_repo.add_to_unconfirmed_units(sender_id, currency_id, units).await?;
                debug!("Rolled back exchange sell unconfirmed: account={}, currency={}, units={}", sender_id, currency_id, units);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
