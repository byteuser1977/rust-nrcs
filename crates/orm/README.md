# ORM Layer

Object-Relational Mapping layer for NRCS blockchain using SQLx.

## Implemented Models

All 66 tables from the H2 schema are covered.

| Table Name | Rust Model |
|------------|------------|
| ACCOUNT | AccountModel |
| ACCOUNT_ASSET | AccountAssetModel |
| ACCOUNT_CONTROL_PHASING | AccountControlPhasingModel |
| ACCOUNT_CURRENCY | AccountCurrencyModel |
| ACCOUNT_FXT | AccountFxtModel |
| ACCOUNT_GUARANTEED_BALANCE | AccountGuaranteedBalanceModel |
| ACCOUNT_INFO | AccountInfoModel |
| ACCOUNT_LEASE | AccountLeaseModel |
| ACCOUNT_LEDGER | AccountLedgerModel |
| ACCOUNT_PROPERTY | AccountPropertyModel |
| ALIAS | AliasModel |
| ALIAS_OFFER | AliasOfferModel |
| ASK_ORDER | AskOrderModel |
| ASSET | AssetModel |
| ASSET_CONTROL_PHASING | AssetControlPhasingModel |
| ASSET_CONTROL_PHASING_SUB_POLL | AssetControlPhasingSubPollModel |
| ASSET_DELETE | AssetDeleteModel |
| ASSET_DIVIDEND | AssetDividendModel |
| ASSET_HISTORY | AssetHistoryModel |
| ASSET_PROPERTY | AssetPropertyModel |
| ASSET_TRANSFER | AssetTransferModel |
| BALANCE | BalanceModel |
| BID_ORDER | BidOrderModel |
| BLOCK | BlockModel |
| BUY_OFFER | BuyOfferModel |
| COIN_ORDER_FXT | CoinOrderFxtModel |
| COIN_TRADE_FXT | CoinTradeFxtModel |
| CONTRACT_REFERENCE | ContractReferenceModel |
| CONTRACT | ContractModel |
| CURRENCY | CurrencyModel |
| CURRENCY_FOUNDER | CurrencyFounderModel |
| CURRENCY_MINT | CurrencyMintModel |
| CURRENCY_SUPPLY | CurrencySupplyModel |
| CURRENCY_TRANSFER | CurrencyTransferModel |
| DATA_TAG | DataTagModel |
| EXCHANGE | ExchangeModel |
| EXCHANGE_REQUEST | ExchangeRequestModel |
| GOODS | GoodsModel |
| HUB | HubModel |
| PEER | PeerModel |
| PHASING_POLL | PhasingPollModel |
| PHASING_POLL_HASHED_SECRET | PhasingPollHashedSecretModel |
| PHASING_POLL_LINKED_TRANSACTION | PhasingPollLinkedTransactionModel |
| PHASING_POLL_RESULT | PhasingPollResultModel |
| PHASING_POLL_VOTER | PhasingPollVoterModel |
| PHASING_VOTE | PhasingVoteModel |
| POLL | PollModel |
| POLL_RESULT | PollResultModel |
| PRUNABLE_MESSAGE | PrunableMessageModel |
| PUBLIC_KEY | PublicKeyModel |
| PURCHASE | PurchaseModel |
| PURCHASE_FEEDBACK | PurchaseFeedbackModel |
| PURCHASE_PUBLIC_FEEDBACK | PurchasePublicFeedbackModel |
| REFERENCED_TRANSACTION | ReferencedTransactionModel |
| SCAN | ScanModel |
| SELL_OFFER | SellOfferModel |
| SHUFFLING | ShufflingModel |
| SHUFFLING_DATA | ShufflingDataModel |
| SHUFFLING_PARTICIPANT | ShufflingParticipantModel |
| TAG | TagModel |
| TAGGED_DATA | TaggedDataModel |
| TAGGED_DATA_EXTEND | TaggedDataExtendModel |
| TAGGED_DATA_TIMESTAMP | TaggedDataTimestampModel |
| TRADE | TradeModel |
| TRANSACTION | TransactionModel |
| UNCONFIRMED_TRANSACTION | UnconfirmedTransactionModel |
| VOTE | VoteModel |

## Repository Traits

Each model has an associated repository trait defined in [`generated_models.rs`](src/generated_models.rs) providing basic CRUD operations via the common `Repository<T>` trait.

## Usage

```rust
use orm::{models::*, Repository};

// Example: Find a block by ID
// let block = BlockRepository::find_by_id(conn, block_id).await?;
```

## Module Structure

- `models.rs`: Hand-written models with domain conversion (Block, Transaction, Account, etc.)
- `generated_models.rs`: Auto-generated models for the remaining tables (DB-02)
- `repository.rs`: Repository traits and implementations
- `lib.rs`: Crate root

## Generation

Models are generated from the H2 schema using `tools/gen_models.ps1`. Regenerate after schema changes:

```powershell
powershell -ExecutionPolicy Bypass -File tools/gen_models.ps1
```

This will produce `src/generated_models.rs` and update the module layout automatically.
