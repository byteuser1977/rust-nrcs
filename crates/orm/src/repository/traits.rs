//! Repository pattern for database access (based on migrations schema)
//!
//! Provides traits and implementations for CRUD operations on blockchain entities.
//! Uses async/await with SQLx and connection pooling.

use async_trait::async_trait;
use thiserror::Error;

use crate::models::*;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("duplicate key: {0}")]
    DuplicateKey(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("blockchain error: {0}")]
    Blockchain(#[from] blockchain_types::BlockchainError),
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;

#[async_trait]
pub trait Repository<T>: Send + Sync {
    async fn insert(&self, item: &T) -> RepositoryResult<()>;
    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<T>>;
    async fn update(&self, item: &T) -> RepositoryResult<()>;
    async fn delete(&self, db_id: i64) -> RepositoryResult<()>;
    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<T>>;
    async fn count(&self) -> RepositoryResult<i64>;
}

#[async_trait]
pub trait BlockRepository: Repository<BlockModel> {
    async fn find_by_height(&self, height: i32) -> RepositoryResult<Option<BlockModel>>;
    async fn find_by_id_column(&self, id: i64) -> RepositoryResult<Option<BlockModel>>;
    async fn find_by_hash(&self, hash: &[u8]) -> RepositoryResult<Option<BlockModel>>;
    async fn find_latest(&self) -> RepositoryResult<Option<BlockModel>>;
    async fn find_range(&self, start_height: i32, end_height: i32) -> RepositoryResult<Vec<BlockModel>>;
    async fn find_by_generator(&self, generator_id: i64) -> RepositoryResult<Vec<BlockModel>>;
    async fn get_height(&self) -> RepositoryResult<i32>;
    async fn get_block_id_at_height(&self, height: i32) -> RepositoryResult<Option<i64>>;
    async fn has_block(&self, id: i64) -> RepositoryResult<bool>;
    async fn get_ids_after(&self, block_id: i64, limit: i32) -> RepositoryResult<Vec<i64>>;
    async fn update_next_block_id(&self, previous_block_id: i64, next_block_id: i64) -> RepositoryResult<()>;

    async fn delete_after_height(&self, height: i32) -> RepositoryResult<Vec<BlockModel>>;
    async fn find_blocks_after_height(&self, height: i32) -> RepositoryResult<Vec<BlockModel>>;

    /// Delete blocks by IDs (for rollback operations)
    async fn delete_blocks_by_ids(&self, db_ids: &[i64]) -> RepositoryResult<()>;
}

#[async_trait]
pub trait TransactionRepository: Repository<TransactionModel> {
    async fn find_by_txid(&self, id: i64) -> RepositoryResult<Option<TransactionModel>>;
    async fn find_by_full_hash(&self, full_hash: &[u8]) -> RepositoryResult<Option<TransactionModel>>;
    async fn find_by_sender(&self, sender_id: i64, limit: i64) -> RepositoryResult<Vec<TransactionModel>>;
    async fn find_by_recipient(&self, recipient_id: i64, limit: i64) -> RepositoryResult<Vec<TransactionModel>>;
    async fn find_by_block(&self, block_id: i64) -> RepositoryResult<Vec<TransactionModel>>;
    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<TransactionModel>>;
    async fn find_unconfirmed(&self, limit: i64) -> RepositoryResult<Vec<TransactionModel>>;

    /// Delete transactions by IDs (for rollback operations)
    async fn delete_transactions_by_ids(&self, db_ids: &[i64]) -> RepositoryResult<()>;
}

#[async_trait]
pub trait AccountRepository: Repository<AccountModel> {
    async fn find_by_account_id(&self, id: i64) -> RepositoryResult<Option<AccountModel>>;
    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<AccountModel>>;
    async fn find_latest_by_id(&self, id: i64) -> RepositoryResult<Option<AccountModel>>;
    async fn find_by_address(&self, address: &str) -> RepositoryResult<Option<AccountModel>>;
    async fn update_balance(&self, account_id: i64, balance: i64, unconfirmed_balance: i64, height: i32) -> RepositoryResult<()>;

    async fn get_or_create(&self, account_id: i64) -> RepositoryResult<AccountModel>;
    async fn add_to_balance(&self, account_id: i64, amount: i64, height: i32) -> RepositoryResult<()>;
    async fn add_to_unconfirmed_balance(&self, account_id: i64, amount: i64, height: i32) -> RepositoryResult<()>;
    async fn add_to_balance_and_unconfirmed(&self, account_id: i64, amount: i64, height: i32) -> RepositoryResult<()>;
    async fn add_to_forged_balance(&self, account_id: i64, amount: i64, height: i32) -> RepositoryResult<()>;
    async fn get_account_count(&self) -> RepositoryResult<i64>;
}

#[async_trait]
pub trait AccountAssetRepository: Repository<AccountAssetModel> {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<AccountAssetModel>>;
    async fn find_by_asset(&self, asset_id: i64) -> RepositoryResult<Vec<AccountAssetModel>>;
    async fn find_by_account_and_asset(&self, account_id: i64, asset_id: i64) -> RepositoryResult<Option<AccountAssetModel>>;
    async fn update_quantity(&self, account_id: i64, asset_id: i64, quantity: i64, height: i32) -> RepositoryResult<()>;
    async fn increase_quantity(&self, account_id: i64, asset_id: i64, delta: i64) -> RepositoryResult<()>;
    async fn decrease_quantity(&self, account_id: i64, asset_id: i64, delta: i64) -> RepositoryResult<()>;
    // Unconfirmed quantity operations (for mempool pre-deduction)
    async fn add_to_unconfirmed_quantity(&self, account_id: i64, asset_id: i64, delta: i64) -> RepositoryResult<()>;
}

#[async_trait]
pub trait AccountLedgerRepository: Repository<AccountLedgerModel> {
    async fn find_by_account(&self, account_id: i64, limit: i64) -> RepositoryResult<Vec<AccountLedgerModel>>;
    async fn find_by_block(&self, block_id: i64) -> RepositoryResult<Vec<AccountLedgerModel>>;
}

#[async_trait]
pub trait AssetRepository: Repository<AssetModel> {
    async fn find_by_asset_id(&self, id: i64) -> RepositoryResult<Option<AssetModel>>;
    async fn find_by_owner(&self, owner_id: i64) -> RepositoryResult<Vec<AssetModel>>;
    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<AssetModel>>;
    async fn find_tradable(&self, limit: i64) -> RepositoryResult<Vec<AssetModel>>;
    // Asset quantity operations
    async fn increase_quantity(&self, asset_id: i64, delta: i64) -> RepositoryResult<()>;
    async fn decrease_quantity(&self, asset_id: i64, delta: i64) -> RepositoryResult<()>;
}

#[async_trait]
pub trait AliasRepository: Repository<AliasModel> {
    async fn find_by_alias_id(&self, id: i64) -> RepositoryResult<Option<AliasModel>>;
    async fn find_by_name(&self, name: &str) -> RepositoryResult<Option<AliasModel>>;
    async fn find_by_owner(&self, account_id: i64) -> RepositoryResult<Vec<AliasModel>>;
    async fn update_owner(&self, alias_id: i64, new_owner_id: i64) -> RepositoryResult<()>;
    async fn update_uri(&self, alias_id: i64, uri: &str) -> RepositoryResult<()>;
}

#[async_trait]
pub trait AliasOfferRepository: Repository<AliasOfferModel> {
    async fn find_by_alias(&self, alias_id: i64) -> RepositoryResult<Option<AliasOfferModel>>;
    async fn find_by_buyer(&self, buyer_id: i64) -> RepositoryResult<Vec<AliasOfferModel>>;
    async fn update_price(&self, alias_id: i64, price: i64, buyer_id: Option<i64>) -> RepositoryResult<()>;
    async fn delete_by_alias(&self, alias_id: i64) -> RepositoryResult<()>;
}

#[async_trait]
pub trait AssetTransferRepository: Repository<AssetTransferModel> {
    async fn find_by_asset(&self, asset_id: i64, limit: i64) -> RepositoryResult<Vec<AssetTransferModel>>;
    async fn find_by_sender(&self, sender_id: i64, limit: i64) -> RepositoryResult<Vec<AssetTransferModel>>;
    async fn find_by_recipient(&self, recipient_id: i64, limit: i64) -> RepositoryResult<Vec<AssetTransferModel>>;
}

#[async_trait]
pub trait AskOrderRepository: Repository<AskOrderModel> {
    async fn find_by_order_id(&self, id: i64) -> RepositoryResult<Option<AskOrderModel>>;
    async fn find_by_asset(&self, asset_id: i64, limit: i64) -> RepositoryResult<Vec<AskOrderModel>>;
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<AskOrderModel>>;
    async fn find_best_by_asset(&self, asset_id: i64) -> RepositoryResult<Option<AskOrderModel>>;
    async fn update_quantity(&self, order_id: i64, quantity: i64) -> RepositoryResult<()>;
}

#[async_trait]
pub trait BidOrderRepository: Repository<BidOrderModel> {
    async fn find_by_order_id(&self, id: i64) -> RepositoryResult<Option<BidOrderModel>>;
    async fn find_by_asset(&self, asset_id: i64, limit: i64) -> RepositoryResult<Vec<BidOrderModel>>;
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<BidOrderModel>>;
    async fn find_best_by_asset(&self, asset_id: i64) -> RepositoryResult<Option<BidOrderModel>>;
    async fn update_quantity(&self, order_id: i64, quantity: i64) -> RepositoryResult<()>;
}

#[async_trait]
pub trait TradeRepository: Repository<TradeModel> {
    async fn find_by_asset(&self, asset_id: i64, limit: i64) -> RepositoryResult<Vec<TradeModel>>;
    async fn find_by_ask_order(&self, ask_order_id: i64) -> RepositoryResult<Vec<TradeModel>>;
    async fn find_by_bid_order(&self, bid_order_id: i64) -> RepositoryResult<Vec<TradeModel>>;
    async fn find_by_buyer(&self, buyer_id: i64, limit: i64) -> RepositoryResult<Vec<TradeModel>>;
    async fn find_by_seller(&self, seller_id: i64, limit: i64) -> RepositoryResult<Vec<TradeModel>>;
}

#[async_trait]
pub trait GoodsRepository: Repository<GoodsModel> {
    async fn find_by_goods_id(&self, id: i64) -> RepositoryResult<Option<GoodsModel>>;
    async fn find_by_seller(&self, seller_id: i64, limit: i64) -> RepositoryResult<Vec<GoodsModel>>;
    async fn find_in_stock(&self, limit: i64) -> RepositoryResult<Vec<GoodsModel>>;
    async fn update_quantity(&self, goods_id: i64, quantity: i32) -> RepositoryResult<()>;
    async fn update_price(&self, goods_id: i64, price: i64) -> RepositoryResult<()>;
    async fn set_delisted(&self, goods_id: i64, delisted: bool) -> RepositoryResult<()>;
}

#[async_trait]
pub trait PurchaseRepository: Repository<PurchaseModel> {
    async fn find_by_purchase_id(&self, id: i64) -> RepositoryResult<Option<PurchaseModel>>;
    async fn find_by_buyer(&self, buyer_id: i64, limit: i64) -> RepositoryResult<Vec<PurchaseModel>>;
    async fn find_by_seller(&self, seller_id: i64, limit: i64) -> RepositoryResult<Vec<PurchaseModel>>;
    async fn find_by_goods(&self, goods_id: i64, limit: i64) -> RepositoryResult<Vec<PurchaseModel>>;
    async fn update_pending(&self, purchase_id: i64, pending: bool) -> RepositoryResult<()>;
    async fn set_delivered(&self, purchase_id: i64, goods: &[u8], nonce: &[u8]) -> RepositoryResult<()>;
    async fn set_refund(&self, purchase_id: i64, refund: i64, note: &[u8], nonce: &[u8]) -> RepositoryResult<()>;
}

#[async_trait]
pub trait CurrencyRepository: Repository<CurrencyModel> {
    async fn find_by_currency_id(&self, id: i64) -> RepositoryResult<Option<CurrencyModel>>;
    async fn find_by_code(&self, code: &str) -> RepositoryResult<Option<CurrencyModel>>;
    async fn find_by_owner(&self, account_id: i64) -> RepositoryResult<Vec<CurrencyModel>>;
    async fn find_by_height(&self, height: i32) -> RepositoryResult<Vec<CurrencyModel>>;
    // P1新增方法
    async fn increase_supply(&self, currency_id: i64, delta: i64) -> RepositoryResult<()>;
    async fn increase_reserve(&self, currency_id: i64, amount_per_unit: i64) -> RepositoryResult<()>;
    async fn delete_currency(&self, currency_id: i64) -> RepositoryResult<()>;
}

#[async_trait]
pub trait AccountCurrencyRepository: Repository<AccountCurrencyModel> {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<AccountCurrencyModel>>;
    async fn find_by_currency(&self, currency_id: i64) -> RepositoryResult<Vec<AccountCurrencyModel>>;
    async fn find_by_account_and_currency(&self, account_id: i64, currency_id: i64) -> RepositoryResult<Option<AccountCurrencyModel>>;
    async fn update_units(&self, account_id: i64, currency_id: i64, delta: i64) -> RepositoryResult<()>;
    // Unconfirmed units operations (for mempool pre-deduction)
    async fn add_to_unconfirmed_units(&self, account_id: i64, currency_id: i64, delta: i64) -> RepositoryResult<()>;
}

#[async_trait]
pub trait CurrencyTransferRepository: Repository<CurrencyTransferModel> {
    async fn find_by_currency(&self, currency_id: i64, limit: i64) -> RepositoryResult<Vec<CurrencyTransferModel>>;
    async fn find_by_sender(&self, sender_id: i64, limit: i64) -> RepositoryResult<Vec<CurrencyTransferModel>>;
    async fn find_by_recipient(&self, recipient_id: i64, limit: i64) -> RepositoryResult<Vec<CurrencyTransferModel>>;
}

#[async_trait]
pub trait TaggedDataRepository: Repository<TaggedDataModel> {
    async fn find_by_data_id(&self, id: i64) -> RepositoryResult<Option<TaggedDataModel>>;
    async fn find_by_account(&self, account_id: i64, limit: i64) -> RepositoryResult<Vec<TaggedDataModel>>;
    async fn find_by_type(&self, data_type: &str, limit: i64) -> RepositoryResult<Vec<TaggedDataModel>>;
    async fn search_by_tag(&self, tag: &str, limit: i64) -> RepositoryResult<Vec<TaggedDataModel>>;
}

#[async_trait]
pub trait PollRepository: Repository<PollModel> {
    async fn find_by_poll_id(&self, id: i64) -> RepositoryResult<Option<PollModel>>;
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<PollModel>>;
    async fn find_active(&self, height: i32, limit: i64) -> RepositoryResult<Vec<PollModel>>;
    async fn update_voters_count(&self, poll_id: i64, count: i64) -> RepositoryResult<()>;
}

#[async_trait]
pub trait VoteRepository: Repository<VoteModel> {
    async fn find_by_poll(&self, poll_id: i64, limit: i64) -> RepositoryResult<Vec<VoteModel>>;
    async fn find_by_voter(&self, voter_id: i64, limit: i64) -> RepositoryResult<Vec<VoteModel>>;
    async fn find_by_poll_and_voter(&self, poll_id: i64, voter_id: i64) -> RepositoryResult<Option<VoteModel>>;
}

#[async_trait]
pub trait ShufflingRepository: Repository<ShufflingModel> {
    async fn find_by_shuffling_id(&self, id: i64) -> RepositoryResult<Option<ShufflingModel>>;
    async fn find_by_issuer(&self, issuer_id: i64) -> RepositoryResult<Vec<ShufflingModel>>;
    async fn find_active(&self, limit: i64) -> RepositoryResult<Vec<ShufflingModel>>;
    async fn update_stage(&self, shuffling_id: i64, stage: i32) -> RepositoryResult<()>;
}

#[async_trait]
pub trait ContractReferenceRepository: Repository<ContractReferenceModel> {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<ContractReferenceModel>>;
    async fn find_by_contract_name(&self, name: &str) -> RepositoryResult<Option<ContractReferenceModel>>;
    async fn delete_by_account_and_name(&self, account_id: i64, name: &str) -> RepositoryResult<()>;
}

#[async_trait]
pub trait AssetPropertyRepository: Repository<AssetPropertyModel> {
    async fn find_by_asset(&self, asset_id: i64) -> RepositoryResult<Vec<AssetPropertyModel>>;
    async fn find_by_asset_and_account(&self, asset_id: i64, account_id: i64) -> RepositoryResult<Vec<AssetPropertyModel>>;
    async fn find_by_asset_account_property(&self, asset_id: i64, account_id: i64, property: &str) -> RepositoryResult<Option<AssetPropertyModel>>;
    async fn delete_by_asset_account_property(&self, asset_id: i64, account_id: i64, property: &str) -> RepositoryResult<()>;
}

#[async_trait]
pub trait AssetHistoryRepository: Repository<AssetHistoryModel> {
    async fn find_by_asset(&self, asset_id: i64, limit: i64) -> RepositoryResult<Vec<AssetHistoryModel>>;
    async fn find_by_asset_and_account(&self, asset_id: i64, account_id: i64, limit: i64) -> RepositoryResult<Vec<AssetHistoryModel>>;
}

#[async_trait]
pub trait TaggedDataTagRepository: Repository<TaggedDataTagModel> {
    async fn find_by_tag(&self, tag: &str, limit: i64) -> RepositoryResult<Vec<TaggedDataTagModel>>;
    async fn find_by_data_id(&self, id: i64) -> RepositoryResult<Vec<TaggedDataTagModel>>;
}

#[async_trait]
pub trait TaggedDataExtendRepository: Repository<TaggedDataExtendModel> {
    async fn find_by_extend_id(&self, extend_id: i64) -> RepositoryResult<Vec<TaggedDataExtendModel>>;
}

#[async_trait]
pub trait TaggedTimestampRepository: Repository<TaggedTimestampModel> {
    async fn find_by_account(&self, account_id: i64, limit: i64) -> RepositoryResult<Vec<TaggedTimestampModel>>;
    async fn find_by_tag(&self, tag: &str, limit: i64) -> RepositoryResult<Vec<TaggedTimestampModel>>;
    async fn find_by_account_and_tag(&self, account_id: i64, tag: &str) -> RepositoryResult<Option<TaggedTimestampModel>>;
}

#[async_trait]
pub trait AccountGuaranteedBalanceRepository: Repository<AccountGuaranteedBalanceModel> {
    async fn find_by_account_and_height(&self, account_id: i64, height: i32) -> RepositoryResult<Option<AccountGuaranteedBalanceModel>>;
    async fn upsert_additions(&self, account_id: i64, height: i32, additions: i64) -> RepositoryResult<()>;
    async fn get_total_additions_since(&self, account_id: i64, since_height: i32, current_height: i32) -> RepositoryResult<i64>;
}

#[async_trait]
pub trait AccountInfoRepository: Repository<AccountInfoModel> {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Option<AccountInfoModel>>;
    async fn upsert(&self, model: &AccountInfoModel) -> RepositoryResult<()>;
}

#[async_trait]
pub trait AccountLeaseRepository: Repository<AccountLeaseModel> {
    async fn find_by_account(&self, lessor_id: i64) -> RepositoryResult<Option<AccountLeaseModel>>;
    async fn upsert(&self, model: &AccountLeaseModel) -> RepositoryResult<()>;
}

#[async_trait]
pub trait AccountPropertyRepository: Repository<AccountPropertyModel> {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Vec<AccountPropertyModel>>;
    async fn find_by_property(&self, account_id: i64, property: &str) -> RepositoryResult<Option<AccountPropertyModel>>;
    async fn upsert(&self, model: &AccountPropertyModel) -> RepositoryResult<()>;
    async fn delete_by_id(&self, id: i64) -> RepositoryResult<()>;
}

#[async_trait]
pub trait AccountControlPhasingRepository: Repository<AccountControlPhasingModel> {
    async fn find_by_account(&self, account_id: i64) -> RepositoryResult<Option<AccountControlPhasingModel>>;
    async fn upsert(&self, model: &AccountControlPhasingModel) -> RepositoryResult<()>;
}

#[async_trait]
pub trait AssetDeleteRepository: Repository<AssetDeleteModel> {
    async fn find_by_asset(&self, asset_id: i64) -> RepositoryResult<Vec<AssetDeleteModel>>;
}

#[async_trait]
pub trait AssetDividendRepository: Repository<AssetDividendModel> {
    async fn find_by_asset(&self, asset_id: i64) -> RepositoryResult<Vec<AssetDividendModel>>;
}

#[async_trait]
pub trait PhasingPollRepository: Repository<PhasingPollModel> {
    async fn find_by_poll_id(&self, id: i64) -> RepositoryResult<Option<PhasingPollModel>>;
    async fn upsert(&self, model: &PhasingPollModel) -> RepositoryResult<()>;
}

#[async_trait]
pub trait PhasingPollLinkedTransactionRepository: Repository<PhasingPollLinkedTransactionModel> {
    async fn find_by_poll(&self, poll_id: i64) -> RepositoryResult<Vec<PhasingPollLinkedTransactionModel>>;
}

#[async_trait]
pub trait PhasingPollResultRepository: Repository<PhasingPollResultModel> {
    async fn find_by_poll(&self, poll_id: i64) -> RepositoryResult<Option<PhasingPollResultModel>>;
    async fn upsert(&self, model: &PhasingPollResultModel) -> RepositoryResult<()>;
}

#[async_trait]
pub trait PhasingPollVoterRepository: Repository<PhasingPollVoterModel> {
    async fn find_by_poll(&self, poll_id: i64) -> RepositoryResult<Vec<PhasingPollVoterModel>>;
}

#[async_trait]
pub trait PhasingVoteRepository: Repository<PhasingVoteModel> {
    async fn find_by_poll(&self, poll_id: i64) -> RepositoryResult<Vec<PhasingVoteModel>>;
}

#[async_trait]
pub trait PollResultRepository: Repository<PollResultModel> {
    async fn find_by_poll(&self, poll_id: i64) -> RepositoryResult<Vec<PollResultModel>>;
    async fn upsert(&self, model: &PollResultModel) -> RepositoryResult<()>;
}

