//! Database models based on migrations schema (001_initial.sql)
//!
//! All models are derived from `sqlx::FromRow` and include
//! `to_domain` / `from_domain` methods for domain object conversion.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use blockchain_types::{
    Account, AccountAsset, AccountId, Amount, Asset, AssetId, Block, BlockId, BlockchainError,
    Hash256, Hash512, Height, Result, Signature, Timestamp, Transaction, TransactionId,
    TransactionType,
};

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct BlockModel {
    pub db_id: i64,
    pub id: i64,
    pub version: i32,
    pub timestamp: i32,
    pub previous_block_id: Option<i64>,
    pub total_amount: i64,
    pub total_fee: i64,
    pub payload_length: i32,
    pub previous_block_hash: Option<Vec<u8>>,
    pub cumulative_difficulty: Vec<u8>,
    pub base_target: i64,
    pub next_block_id: Option<i64>,
    pub height: i32,
    pub generation_signature: Vec<u8>,
    pub block_signature: Vec<u8>,
    pub payload_hash: Vec<u8>,
    pub generator_id: i64,
}

impl BlockModel {
    pub fn to_domain(&self) -> Result<Block> {
        let mut block = Block {
            version: self.version as u32,
            timestamp: self.timestamp as Timestamp,
            height: self.height as Height,
            previous_block_hash: self
                .previous_block_hash
                .as_ref()
                .map(|h| h.as_slice().try_into().ok())
                .flatten()
                .unwrap_or([0u8; 32]),
            payload_hash: self.payload_hash.as_slice().try_into().map_err(|_| {
                BlockchainError::InvalidHash("payload_hash length mismatch".to_string())
            })?,
            generator_id: self.generator_id as AccountId,
            nonce: 0,
            base_target: self.base_target as u64,
            cumulative_difficulty: self.cumulative_difficulty.clone(),
            total_amount: self.total_amount as Amount,
            total_fee: self.total_fee as Amount,
            payload_length: self.payload_length as u32,
            generation_signature: self
                .generation_signature
                .as_slice()
                .try_into()
                .unwrap_or([0u8; 64]),
            block_signature: self
                .block_signature
                .as_slice()
                .try_into()
                .unwrap_or([0u8; 64]),
            transactions: vec![],
        };
        Ok(block)
    }

    pub fn from_domain(block: &Block) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: block.height as i64,
            version: block.version as i32,
            timestamp: block.timestamp as i32,
            previous_block_id: None,
            total_amount: block.total_amount as i64,
            total_fee: block.total_fee as i64,
            payload_length: block.payload_length as i32,
            previous_block_hash: Some(block.previous_block_hash.to_vec()),
            cumulative_difficulty: block.cumulative_difficulty.clone(),
            base_target: block.base_target as i64,
            next_block_id: None,
            height: block.height as i32,
            generation_signature: block.generation_signature.to_vec(),
            block_signature: block.block_signature.to_vec(),
            payload_hash: block.payload_hash.to_vec(),
            generator_id: block.generator_id as i64,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct TransactionModel {
    pub db_id: i64,
    pub id: i64,
    pub deadline: i16,
    pub recipient_id: Option<i64>,
    pub amount: i64,
    pub fee: i64,
    pub full_hash: Vec<u8>,
    pub height: i32,
    pub block_id: i64,
    pub signature: Vec<u8>,
    pub timestamp: i32,
    #[sqlx(rename = "type")]
    pub type_: i16,
    pub subtype: i16,
    pub sender_id: i64,
    pub block_timestamp: i32,
    pub referenced_transaction_full_hash: Option<Vec<u8>>,
    pub transaction_index: i16,
    pub phased: bool,
    pub attachment_bytes: Option<Vec<u8>>,
    pub version: i16,
    pub has_message: bool,
    pub has_encrypted_message: bool,
    pub has_public_key_announcement: bool,
    pub has_prunable_message: bool,
    pub has_prunable_attachment: bool,
    pub ec_block_height: Option<i32>,
    pub ec_block_id: Option<i64>,
    pub has_encrypttoself_message: bool,
    pub has_prunable_encrypted_message: bool,
}

impl TransactionModel {
    pub fn to_domain(&self) -> Result<Transaction> {
        let sender_id = self.sender_id as AccountId;
        let tx = Transaction {
            version: self.version as u8,
            type_id: TransactionType::from(self.type_ as u8),
            subtype: self.subtype as u8,
            timestamp: self.timestamp as Timestamp,
            deadline: self.deadline as u16,
            sender_id,
            recipient_id: self.recipient_id.map(|id| id as AccountId),
            amount: self.amount as Amount,
            fee: self.fee as Amount,
            height: self.height as Height,
            block_id: self.block_id as BlockId,
            signature: self.signature.as_slice().try_into().unwrap_or([0u8; 64]),
            full_hash: self.full_hash.as_slice().try_into().map_err(|_| {
                BlockchainError::InvalidHash("full_hash length mismatch".to_string())
            })?,
            attachment_bytes: self.attachment_bytes.clone().unwrap_or_default(),
            phased: self.phased,
            has_message: self.has_message,
            has_encrypted_message: self.has_encrypted_message,
            has_public_key_announcement: self.has_public_key_announcement,
            has_prunable_attachment: self.has_prunable_attachment,
            ec_block_height: self.ec_block_height.map(|h| h as u32),
            ec_block_id: self.ec_block_id.map(|id| id as u64),
            has_encrypttoself_message: self.has_encrypttoself_message,
            has_prunable_encrypted_message: self.has_prunable_encrypted_message,
        };
        Ok(tx)
    }

    pub fn from_domain(tx: &Transaction) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: 0,
            deadline: tx.deadline as i16,
            recipient_id: tx.recipient_id.map(|id| id as i64),
            amount: tx.amount as i64,
            fee: tx.fee as i64,
            full_hash: tx.full_hash.to_vec(),
            height: tx.height as i32,
            block_id: tx.block_id as i64,
            signature: tx.signature.to_vec(),
            timestamp: tx.timestamp as i32,
            type_: u8::from(tx.type_id) as i16,
            subtype: tx.subtype as i16,
            sender_id: tx.sender_id as i64,
            block_timestamp: 0,
            referenced_transaction_full_hash: None,
            transaction_index: 0,
            phased: tx.phased,
            attachment_bytes: Some(tx.attachment_bytes.clone()),
            version: tx.version as i16,
            has_message: tx.has_message,
            has_encrypted_message: tx.has_encrypted_message,
            has_public_key_announcement: tx.has_public_key_announcement,
            has_prunable_message: tx.has_prunable_attachment,
            has_prunable_attachment: tx.has_prunable_attachment,
            ec_block_height: tx.ec_block_height.map(|h| h as i32),
            ec_block_id: tx.ec_block_id.map(|id| id as i64),
            has_encrypttoself_message: tx.has_encrypttoself_message,
            has_prunable_encrypted_message: tx.has_prunable_encrypted_message,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AccountModel {
    pub db_id: i64,
    pub id: i64,
    pub balance: i64,
    pub unconfirmed_balance: i64,
    pub forged_balance: i64,
    pub active_lessee_id: Option<i64>,
    pub has_control_phasing: bool,
    pub height: i32,
    pub latest: bool,
}

impl AccountModel {
    pub fn to_domain(&self) -> Result<Account> {
        Ok(Account {
            id: self.id as AccountId,
            address: None,
            balance: self.balance as Amount,
            unconfirmed_balance: self.unconfirmed_balance as Amount,
            reserved_balance: 0,
            guaranteed_balance: self.forged_balance as Amount,
            assets: std::collections::HashMap::new(),
            properties: std::collections::HashMap::new(),
            lease: None,
            created_at: 0,
            last_updated: 0,
            current_height: self.height as Height,
        })
    }

    pub fn from_domain(account: &Account) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: account.id as i64,
            balance: account.balance as i64,
            unconfirmed_balance: account.unconfirmed_balance as i64,
            forged_balance: account.guaranteed_balance as i64,
            active_lessee_id: None,
            has_control_phasing: false,
            height: account.current_height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AccountAssetModel {
    pub db_id: i64,
    pub account_id: i64,
    pub asset_id: i64,
    pub quantity: i64,
    pub unconfirmed_quantity: i64,
    pub height: i32,
    pub latest: bool,
}

impl AccountAssetModel {
    pub fn to_domain(&self) -> Result<AccountAsset> {
        Ok(AccountAsset {
            account_id: self.account_id as AccountId,
            asset_id: self.asset_id as AssetId,
            quantity: self.quantity as Amount,
            last_updated: self.height as Timestamp,
        })
    }

    pub fn from_domain(aa: &AccountAsset) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            account_id: aa.account_id as i64,
            asset_id: aa.asset_id as i64,
            quantity: aa.quantity as i64,
            unconfirmed_quantity: aa.quantity as i64,
            height: aa.last_updated as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AssetModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub quantity: i64,
    pub decimals: i16,
    pub has_control_phasing: bool,
    pub initial_quantity: i64,
    pub height: i32,
    pub latest: bool,
}
impl AssetModel {
    pub fn to_domain(&self) -> Result<Asset> {
        Ok(Asset {
            id: self.id as AssetId,
            owner_id: self.account_id as AccountId,
            name: self.name.clone(),
            description: self.description.clone().unwrap_or_default(),
            quantity: self.quantity as Amount,
            decimals: self.decimals as u8,
            mintable: false,
            transferable: true,
            data: vec![],
            created_at: self.height as Timestamp,
            last_updated: self.height as Timestamp,
            deleted: false,
        })
    }

    pub fn from_domain(asset: &Asset) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: asset.id as i64,
            account_id: asset.owner_id as i64,
            name: asset.name.clone(),
            description: Some(asset.description.clone()),
            quantity: asset.quantity as i64,
            decimals: asset.decimals as i16,
            has_control_phasing: false,
            initial_quantity: asset.quantity as i64,
            height: asset.created_at as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct CurrencyModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub name: String,
    pub name_lower: String,
    pub code: String,
    pub description: Option<String>,
    pub type_: i32,
    pub initial_supply: i64,
    pub reserve_supply: i64,
    pub max_supply: i64,
    pub creation_height: i32,
    pub issuance_height: i32,
    pub min_reserve_per_unit_nqt: i64,
    pub min_difficulty: i16,
    pub max_difficulty: i16,
    pub ruleset: i16,
    pub algorithm: i16,
    pub decimals: i16,
    pub height: i32,
    pub latest: bool,
}
impl CurrencyModel {
    pub fn to_domain(&self) -> Result<Currency> {
        Ok(Currency {
            id: self.id as CurrencyId,
            owner_id: self.account_id as AccountId,
            name: self.name.clone(),
            code: self.code.clone(),
            description: self.description.clone().unwrap_or_default(),
            currency_type: self.type_ as u8,
            initial_supply: self.initial_supply as Amount,
            reserve_supply: self.reserve_supply as Amount,
            max_supply: self.max_supply as Amount,
            creation_height: self.creation_height as Height,
            issuance_height: self.issuance_height as Height,
            min_reserve_per_unit_nqt: self.min_reserve_per_unit_nqt as Amount,
            min_difficulty: self.min_difficulty as u8,
            max_difficulty: self.max_difficulty as u8,
            ruleset: self.ruleset as u8,
            algorithm: self.algorithm as u8,
            decimals: self.decimals as u8,
            created_at: self.height as Timestamp,
            last_updated: self.height as Timestamp,
        })
    }

    pub fn from_domain(currency: &Currency) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: currency.id as i64,
            account_id: currency.owner_id as i64,
            name: currency.name.clone(),
            name_lower: currency.name.to_lowercase(),
            code: currency.code.clone(),
            description: Some(currency.description.clone()),
            type_: currency.currency_type as i32,
            initial_supply: currency.initial_supply as i64,
            reserve_supply: currency.reserve_supply as i64,
            max_supply: currency.max_supply as i64,
            creation_height: currency.creation_height as i32,
            issuance_height: currency.issuance_height as i32,
            min_reserve_per_unit_nqt: currency.min_reserve_per_unit_nqt as i64,
            min_difficulty: currency.min_difficulty as i16,
            max_difficulty: currency.max_difficulty as i16,
            ruleset: currency.ruleset as i16,
            algorithm: currency.algorithm as i16,
            decimals: currency.decimals as i16,
            height: currency.created_at as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AliasModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub alias_name: String,
    pub alias_name_lower: String,
    pub alias_uri: String,
    pub timestamp: i32,
    pub height: i32,
    pub latest: bool,
}

impl AliasModel {
    pub fn to_domain(&self) -> Result<Alias> {
        Ok(Alias {
            id: self.id as AliasId,
            owner_id: self.account_id as AccountId,
            name: self.alias_name.clone(),
            uri: self.alias_uri.clone(),
            timestamp: self.timestamp as Timestamp,
            height: self.height as Height,
        })
    }

    pub fn from_domain(alias: &Alias) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: alias.id as i64,
            account_id: alias.owner_id as i64,
            alias_name: alias.name.clone(),
            alias_name_lower: alias.name.to_lowercase(),
            alias_uri: alias.uri.clone(),
            timestamp: alias.timestamp as i32,
            height: alias.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AliasOfferModel {
    pub db_id: i64,
    pub id: i64,
    pub price: i64,
    pub buyer_id: Option<i64>,
    pub height: i32,
    pub latest: bool,
}

impl AliasOfferModel {
    pub fn to_domain(&self) -> Result<AliasOffer> {
        Ok(AliasOffer {
            id: self.id as AliasOfferId,
            alias_id: 0,
            price: self.price as Amount,
            buyer_id: self.buyer_id.map(|id| id as AccountId),
            height: self.height as Height,
        })
    }

    pub fn from_domain(offer: &AliasOffer) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: offer.id as i64,
            price: offer.price as i64,
            buyer_id: offer.buyer_id.map(|id| id as i64),
            height: offer.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AccountInfoModel {
    pub db_id: i64,
    pub account_id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub height: i32,
    pub latest: bool,
}
impl AccountInfoModel {
    pub fn to_domain(&self) -> Result<AccountInfo> {
        Ok(AccountInfo {
            account_id: self.account_id as AccountId,
            name: self.name.clone(),
            description: self.description.clone(),
            height: self.height as Height,
        })
    }

    pub fn from_domain(info: &AccountInfo) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            account_id: info.account_id as i64,
            name: Some(info.name.clone()),
            description: Some(info.description.clone()),
            height: info.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AccountLeaseModel {
    pub db_id: i64,
    pub lessor_id: i64,
    pub current_leasing_height_from: Option<i32>,
    pub current_leasing_height_to: Option<i32>,
    pub current_lessee_id: Option<i64>,
    pub next_leasing_height_from: Option<i32>,
    pub next_leasing_height_to: Option<i32>,
    pub next_lessee_id: Option<i64>,
    pub height: i32,
    pub latest: bool,
}

impl AccountLeaseModel {
    pub fn to_domain(&self) -> Result<AccountLease> {
        Ok(AccountLease {
            lessee_id: self.current_lessee_id.unwrap_or(0) as AccountId,
            amount: 0,
            start_height: self.current_leasing_height_from.unwrap_or(0) as Height,
            end_height: self.current_leasing_height_to.unwrap_or(0) as Height,
        })
    }

    pub fn from_domain(lease: &AccountLease) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            lessor_id: 0,
            current_leasing_height_from: Some(lease.start_height as i32),
            current_leasing_height_to: Some(lease.end_height as i32),
            current_lessee_id: Some(lease.lessee_id as i64),
            next_leasing_height_from: None,
            next_leasing_height_to: None,
            next_lessee_id: None,
            height: 0,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PublicKeyModel {
    pub db_id: i64,
    pub account_id: i64,
    pub public_key: Option<Vec<u8>>,
    pub height: i32,
    pub latest: bool,
}

impl PublicKeyModel {
    pub fn to_domain(&self) -> Result<PublicKey> {
        Ok(PublicKey {
            account_id: self.account_id as AccountId,
            public_key: self
                .public_key
                .clone()
                .map(|pk| pk.as_slice().try_into().ok())
                .flatten()
                .unwrap_or([0u8; 32]),
            height: self.height as Height,
        })
    }

    pub fn from_domain(pk: &PublicKey) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            account_id: pk.account_id as i64,
            public_key: Some(pk.public_key.to_vec()),
            height: pk.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AccountCurrencyModel {
    pub db_id: i64,
    pub account_id: i64,
    pub currency_id: i64,
    pub units: i64,
    pub unconfirmed_units: i64,
    pub height: i32,
    pub latest: bool,
}

impl AccountCurrencyModel {
    pub fn to_domain(&self) -> Result<AccountCurrency> {
        Ok(AccountCurrency {
            account_id: self.account_id as AccountId,
            currency_id: self.currency_id as CurrencyId,
            units: self.units as Amount,
            unconfirmed_units: self.unconfirmed_units as Amount,
            height: self.height as Height,
        })
    }

    pub fn from_domain(ac: &AccountCurrency) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            account_id: ac.account_id as i64,
            currency_id: ac.currency_id as i64,
            units: ac.units as i64,
            unconfirmed_units: ac.unconfirmed_units as i64,
            height: ac.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AccountControlPhasingModel {
    pub db_id: i64,
    pub account_id: i64,
    pub whitelist: Option<String>,
    pub voting_model: i16,
    pub quorum: Option<i64>,
    pub min_balance: Option<i64>,
    pub holding_id: Option<i64>,
    pub min_balance_model: Option<i16>,
    pub max_fees: Option<i64>,
    pub min_duration: Option<i16>,
    pub max_duration: Option<i16>,
    pub height: i32,
    pub latest: bool,
}
impl AccountControlPhasingModel {
    pub fn to_domain(&self) -> Result<PhasingControl> {
        Ok(PhasingControl {
            account_id: self.account_id as AccountId,
            voting_model: self.voting_model as u8,
            quorum: self.quorum.map(|q| q as u64),
            min_balance: self.min_balance.map(|b| b as Amount),
            holding_id: self.holding_id.map(|h| h as u64),
            min_balance_model: self.min_balance_model.map(|m| m as u8),
            whitelist: self
                .whitelist
                .as_ref()
                .and_then(|w| serde_json::from_str(w).ok())
                .unwrap_or_default(),
            height: self.height as Height,
        })
    }

    pub fn from_domain(pc: &PhasingControl) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            account_id: pc.account_id as i64,
            whitelist: Some(serde_json::to_string(&pc.whitelist).unwrap_or_default()),
            voting_model: pc.voting_model as i16,
            quorum: pc.quorum.map(|q| q as i64),
            min_balance: pc.min_balance.map(|b| b as i64),
            holding_id: pc.holding_id.map(|h| h as i64),
            min_balance_model: pc.min_balance_model.map(|m| m as i16),
            max_fees: None,
            min_duration: None,
            max_duration: None,
            height: pc.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AccountGuaranteedBalanceModel {
    pub db_id: i64,
    pub account_id: i64,
    pub additions: i64,
    pub height: i32,
}
impl AccountGuaranteedBalanceModel {
    pub fn to_domain(&self) -> Result<Amount> {
        Ok(self.additions as Amount)
    }

    pub fn from_domain(balance: &Amount, account_id: AccountId) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            account_id: account_id as i64,
            additions: *balance as i64,
            height: 0,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AccountLedgerModel {
    pub db_id: i64,
    pub account_id: i64,
    pub event_type: i16,
    pub event_id: i64,
    pub holding_type: i16,
    pub holding_id: Option<i64>,
    pub change: i64,
    pub balance: i64,
    pub block_id: i64,
    pub height: i32,
    pub timestamp: i32,
}

impl AccountLedgerModel {
    pub fn to_domain(&self) -> Result<AccountLedgerEntry> {
        Ok(AccountLedgerEntry {
            account_id: self.account_id as AccountId,
            event_type: self.event_type as u8,
            event_id: self.event_id as u64,
            holding_type: self.holding_type as u8,
            holding_id: self.holding_id.map(|h| h as u64),
            change: self.change as Amount,
            balance: self.balance as Amount,
            block_id: self.block_id as BlockId,
            height: self.height as Height,
            timestamp: self.timestamp as Timestamp,
        })
    }

    pub fn from_domain(entry: &AccountLedgerEntry) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            account_id: entry.account_id as i64,
            event_type: entry.event_type as i16,
            event_id: entry.event_id as i64,
            holding_type: entry.holding_type as i16,
            holding_id: entry.holding_id.map(|h| h as i64),
            change: entry.change as i64,
            balance: entry.balance as i64,
            block_id: entry.block_id as i64,
            height: entry.height as i32,
            timestamp: entry.timestamp as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AccountPropertyModel {
    pub db_id: i64,
    pub id: i64,
    pub recipient_id: i64,
    pub setter_id: Option<i64>,
    pub property: String,
    pub value: Option<String>,
    pub height: i32,
    pub latest: bool,
}

impl AccountPropertyModel {
    pub fn to_domain(&self) -> Result<AccountProperty> {
        Ok(AccountProperty {
            id: self.id as PropertyId,
            recipient_id: self.recipient_id as AccountId,
            setter_id: self.setter_id.map(|s| s as AccountId),
            property: self.property.clone(),
            value: self.value.clone(),
            height: self.height as Height,
        })
    }

    pub fn from_domain(prop: &AccountProperty) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: prop.id as i64,
            recipient_id: prop.recipient_id as i64,
            setter_id: prop.setter_id.map(|s| s as i64),
            property: prop.property.clone(),
            value: Some(prop.value.clone()),
            height: prop.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AssetTransferModel {
    pub db_id: i64,
    pub id: i64,
    pub asset_id: i64,
    pub sender_id: i64,
    pub recipient_id: i64,
    pub quantity: i64,
    pub timestamp: i32,
    pub height: i32,
}

impl AssetTransferModel {
    pub fn to_domain(&self) -> Result<AssetTransfer> {
        Ok(AssetTransfer {
            id: self.id as TransferId,
            asset_id: self.asset_id as AssetId,
            sender_id: self.sender_id as AccountId,
            recipient_id: self.recipient_id as AccountId,
            quantity: self.quantity as Amount,
            timestamp: self.timestamp as Timestamp,
            height: self.height as Height,
        })
    }

    pub fn from_domain(transfer: &AssetTransfer) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: transfer.id as i64,
            asset_id: transfer.asset_id as i64,
            sender_id: transfer.sender_id as i64,
            recipient_id: transfer.recipient_id as i64,
            quantity: transfer.quantity as i64,
            timestamp: transfer.timestamp as i32,
            height: transfer.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AssetDeleteModel {
    pub db_id: i64,
    pub id: i64,
    pub asset_id: i64,
    pub account_id: i64,
    pub quantity: i64,
    pub timestamp: i32,
    pub height: i32,
}

impl AssetDeleteModel {
    pub fn to_domain(&self) -> Result<AssetDelete> {
        Ok(AssetDelete {
            id: self.id as AssetDeleteId,
            asset_id: self.asset_id as AssetId,
            account_id: self.account_id as AccountId,
            quantity: self.quantity as Amount,
            timestamp: self.timestamp as Timestamp,
            height: self.height as Height,
        })
    }

    pub fn from_domain(delete: &AssetDelete) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: delete.id as i64,
            asset_id: delete.asset_id as i64,
            account_id: delete.account_id as i64,
            quantity: delete.quantity as i64,
            timestamp: delete.timestamp as i32,
            height: delete.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AssetDividendModel {
    pub db_id: i64,
    pub id: i64,
    pub asset_id: i64,
    pub amount: i64,
    pub dividend_height: i32,
    pub total_dividend: i64,
    pub num_accounts: i64,
    pub timestamp: i32,
    pub height: i32,
}
impl AssetDividendModel {
    pub fn to_domain(&self) -> Result<AssetDividend> {
        Ok(AssetDividend {
            id: self.id as DividendId,
            asset_id: self.asset_id as AssetId,
            amount: self.amount as Amount,
            dividend_height: self.dividend_height as Height,
            total_dividend: self.total_dividend as Amount,
            num_accounts: self.num_accounts as u64,
            timestamp: self.timestamp as Timestamp,
            height: self.height as Height,
        })
    }

    pub fn from_domain(div: &AssetDividend) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: div.id as i64,
            asset_id: div.asset_id as i64,
            amount: div.amount as i64,
            dividend_height: div.dividend_height as i32,
            total_dividend: div.total_dividend as i64,
            num_accounts: div.num_accounts as i64,
            timestamp: div.timestamp as i32,
            height: div.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AssetHistoryModel {
    pub db_id: i64,
    pub id: i64,
    pub full_hash: Vec<u8>,
    pub asset_id: i64,
    pub account_id: i64,
    pub quantity: i64,
    pub timestamp: i32,
    pub chain_id: i32,
    pub height: i32,
}
impl AssetHistoryModel {
    pub fn to_domain(&self) -> Result<AssetHistory> {
        Ok(AssetHistory {
            id: self.id as AssetHistoryId,
            full_hash: self.full_hash.as_slice().try_into().unwrap_or([0u8; 32]),
            asset_id: self.asset_id as AssetId,
            account_id: self.account_id as AccountId,
            quantity: self.quantity as Amount,
            timestamp: self.timestamp as Timestamp,
            chain_id: self.chain_id as u32,
            height: self.height as Height,
        })
    }

    pub fn from_domain(history: &AssetHistory) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: history.id as i64,
            full_hash: history.full_hash.to_vec(),
            asset_id: history.asset_id as i64,
            account_id: history.account_id as i64,
            quantity: history.quantity as i64,
            timestamp: history.timestamp as i32,
            chain_id: history.chain_id as i32,
            height: history.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AssetControlPhasingModel {
    pub db_id: i64,
    pub asset_id: i64,
    pub voting_model: i16,
    pub quorum: Option<i64>,
    pub min_balance: Option<i64>,
    pub holding_id: Option<i64>,
    pub min_balance_model: Option<i16>,
    pub whitelist: Option<String>,
    pub expression: Option<String>,
    pub sender_property_setter_id: Option<i64>,
    pub sender_property_name: Option<String>,
    pub sender_property_value: Option<String>,
    pub recipient_property_setter_id: Option<i64>,
    pub recipient_property_name: Option<String>,
    pub recipient_property_value: Option<String>,
    pub height: i32,
    pub latest: bool,
}

impl AssetControlPhasingModel {
    pub fn to_domain(&self) -> Result<AssetControlPhasing> {
        Ok(AssetControlPhasing {
            asset_id: self.asset_id as AssetId,
            voting_model: self.voting_model as u8,
            quorum: self.quorum.map(|q| q as u64),
            min_balance: self.min_balance.map(|b| b as Amount),
            holding_id: self.holding_id.map(|h| h as u64),
            min_balance_model: self.min_balance_model.map(|m| m as u8),
            whitelist: self
                .whitelist
                .as_ref()
                .and_then(|w| serde_json::from_str(w).ok())
                .unwrap_or_default(),
            height: self.height as Height,
        })
    }

    pub fn from_domain(acp: &AssetControlPhasing) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            asset_id: acp.asset_id as i64,
            voting_model: acp.voting_model as i16,
            quorum: acp.quorum.map(|q| q as i64),
            min_balance: acp.min_balance.map(|b| b as i64),
            holding_id: acp.holding_id.map(|h| h as i64),
            min_balance_model: acp.min_balance_model.map(|m| m as i16),
            whitelist: Some(serde_json::to_string(&acp.whitelist).unwrap_or_default()),
            expression: None,
            sender_property_setter_id: None,
            sender_property_name: None,
            sender_property_value: None,
            recipient_property_setter_id: None,
            recipient_property_name: None,
            recipient_property_value: None,
            height: acp.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct CurrencyMintModel {
    pub db_id: i64,
    pub currency_id: i64,
    pub account_id: i64,
    pub counter: i64,
    pub height: i32,
    pub latest: bool,
}
impl CurrencyMintModel {
    pub fn to_domain(&self) -> Result<CurrencyMint> {
        Ok(CurrencyMint {
            currency_id: self.currency_id as CurrencyId,
            account_id: self.account_id as AccountId,
            counter: self.counter as u64,
            height: self.height as Height,
        })
    }

    pub fn from_domain(mint: &CurrencyMint) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            currency_id: mint.currency_id as i64,
            account_id: mint.account_id as i64,
            counter: mint.counter as i64,
            height: mint.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct CurrencySupplyModel {
    pub db_id: i64,
    pub id: i64,
    pub current_supply: i64,
    pub current_reserve_per_unit_nqt: i64,
    pub height: i32,
    pub latest: bool,
}
impl CurrencySupplyModel {
    pub fn to_domain(&self) -> Result<CurrencySupply> {
        Ok(CurrencySupply {
            id: self.id as CurrencyId,
            current_supply: self.current_supply as Amount,
            current_reserve_per_unit_nqt: self.current_reserve_per_unit_nqt as Amount,
            height: self.height as Height,
        })
    }

    pub fn from_domain(supply: &CurrencySupply) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: supply.id as i64,
            current_supply: supply.current_supply as i64,
            current_reserve_per_unit_nqt: supply.current_reserve_per_unit_nqt as i64,
            height: supply.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct CurrencyTransferModel {
    pub db_id: i64,
    pub id: i64,
    pub currency_id: i64,
    pub sender_id: i64,
    pub recipient_id: i64,
    pub units: i64,
    pub timestamp: i32,
    pub height: i32,
}
impl CurrencyTransferModel {
    pub fn to_domain(&self) -> Result<CurrencyTransfer> {
        Ok(CurrencyTransfer {
            id: self.id as TransferId,
            currency_id: self.currency_id as CurrencyId,
            sender_id: self.sender_id as AccountId,
            recipient_id: self.recipient_id as AccountId,
            units: self.units as Amount,
            timestamp: self.timestamp as Timestamp,
            height: self.height as Height,
        })
    }

    pub fn from_domain(transfer: &CurrencyTransfer) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: transfer.id as i64,
            currency_id: transfer.currency_id as i64,
            sender_id: transfer.sender_id as i64,
            recipient_id: transfer.recipient_id as i64,
            units: transfer.units as i64,
            timestamp: transfer.timestamp as i32,
            height: transfer.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct CurrencyFounderModel {
    pub db_id: i64,
    pub currency_id: i64,
    pub account_id: i64,
    pub amount: i64,
    pub height: i32,
    pub latest: bool,
}

impl CurrencyFounderModel {
    pub fn to_domain(&self) -> Result<CurrencyFounder> {
        Ok(CurrencyFounder {
            currency_id: self.currency_id as CurrencyId,
            account_id: self.account_id as AccountId,
            amount: self.amount as Amount,
            height: self.height as Height,
        })
    }

    pub fn from_domain(founder: &CurrencyFounder) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            currency_id: founder.currency_id as i64,
            account_id: founder.account_id as i64,
            amount: founder.amount as i64,
            height: founder.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AskOrderModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub asset_id: i64,
    pub price: i64,
    pub transaction_index: i16,
    pub transaction_height: i32,
    pub quantity: i64,
    pub creation_height: i32,
    pub height: i32,
    pub latest: bool,
}
impl AskOrderModel {
    pub fn to_domain(&self) -> Result<AskOrder> {
        Ok(AskOrder {
            id: self.id as OrderId,
            account_id: self.account_id as AccountId,
            asset_id: self.asset_id as AssetId,
            price: self.price as Amount,
            quantity: self.quantity as Amount,
            creation_height: self.creation_height as Height,
            transaction_height: self.transaction_height as Height,
            transaction_index: self.transaction_index as u16,
            height: self.height as Height,
        })
    }

    pub fn from_domain(order: &AskOrder) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: order.id as i64,
            account_id: order.account_id as i64,
            asset_id: order.asset_id as i64,
            price: order.price as i64,
            transaction_index: order.transaction_index as i16,
            transaction_height: order.transaction_height as i32,
            quantity: order.quantity as i64,
            creation_height: order.creation_height as i32,
            height: order.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct BidOrderModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub asset_id: i64,
    pub price: i64,
    pub transaction_index: i16,
    pub transaction_height: i32,
    pub quantity: i64,
    pub creation_height: i32,
    pub height: i32,
    pub latest: bool,
}

impl BidOrderModel {
    pub fn to_domain(&self) -> Result<BidOrder> {
        Ok(BidOrder {
            id: self.id as OrderId,
            account_id: self.account_id as AccountId,
            asset_id: self.asset_id as AssetId,
            price: self.price as Amount,
            quantity: self.quantity as Amount,
            creation_height: self.creation_height as Height,
            transaction_height: self.transaction_height as Height,
            transaction_index: self.transaction_index as u16,
            height: self.height as Height,
        })
    }

    pub fn from_domain(order: &BidOrder) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: order.id as i64,
            account_id: order.account_id as i64,
            asset_id: order.asset_id as i64,
            price: order.price as i64,
            transaction_index: order.transaction_index as i16,
            transaction_height: order.transaction_height as i32,
            quantity: order.quantity as i64,
            creation_height: order.creation_height as i32,
            height: order.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct SellOfferModel {
    pub db_id: i64,
    pub id: i64,
    pub currency_id: i64,
    pub account_id: i64,
    pub rate: i64,
    pub unit_limit: i64,
    pub supply: i64,
    pub expiration_height: i32,
    pub transaction_height: i32,
    pub creation_height: i32,
    pub transaction_index: i16,
    pub height: i32,
    pub latest: bool,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct BuyOfferModel {
    pub db_id: i64,
    pub id: i64,
    pub currency_id: i64,
    pub account_id: i64,
    pub rate: i64,
    pub unit_limit: i64,
    pub supply: i64,
    pub expiration_height: i32,
    pub transaction_height: i32,
    pub creation_height: i32,
    pub transaction_index: i16,
    pub height: i32,
    pub latest: bool,
}
impl BuyOfferModel {
    pub fn to_domain(&self) -> Result<BuyOffer> {
        Ok(BuyOffer {
            id: self.id as OrderId,
            currency_id: self.currency_id as CurrencyId,
            account_id: self.account_id as AccountId,
            rate: self.rate as Amount,
            unit_limit: self.unit_limit as Amount,
            supply: self.supply as Amount,
            expiration_height: self.expiration_height as Height,
            transaction_height: self.transaction_height as Height,
            creation_height: self.creation_height as Height,
            transaction_index: self.transaction_index as u16,
            height: self.height as Height,
        })
    }

    pub fn from_domain(offer: &BuyOffer) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: offer.id as i64,
            currency_id: offer.currency_id as i64,
            account_id: offer.account_id as i64,
            rate: offer.rate as i64,
            unit_limit: offer.unit_limit as i64,
            supply: offer.supply as i64,
            expiration_height: offer.expiration_height as i32,
            transaction_height: offer.transaction_height as i32,
            creation_height: offer.creation_height as i32,
            transaction_index: offer.transaction_index as i16,
            height: offer.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct TradeModel {
    pub db_id: i64,
    pub asset_id: i64,
    pub block_id: i64,
    pub ask_order_id: i64,
    pub bid_order_id: i64,
    pub ask_order_height: i32,
    pub bid_order_height: i32,
    pub seller_id: i64,
    pub buyer_id: i64,
    pub is_buy: bool,
    pub quantity: i64,
    pub price: i64,
    pub timestamp: i32,
    pub height: i32,
}

impl TradeModel {
    pub fn to_domain(&self) -> Result<Trade> {
        Ok(Trade {
            id: self.db_id as TradeId,
            asset_id: self.asset_id as AssetId,
            block_id: self.block_id as BlockId,
            ask_order_id: self.ask_order_id as OrderId,
            bid_order_id: self.bid_order_id as OrderId,
            seller_id: self.seller_id as AccountId,
            buyer_id: self.buyer_id as AccountId,
            is_buy: self.is_buy,
            quantity: self.quantity as Amount,
            price: self.price as Amount,
            timestamp: self.timestamp as Timestamp,
            height: self.height as Height,
        })
    }

    pub fn from_domain(trade: &Trade) -> Result<Self> {
        Ok(Self {
            db_id: trade.id as i64,
            asset_id: trade.asset_id as i64,
            block_id: trade.block_id as i64,
            ask_order_id: trade.ask_order_id as i64,
            bid_order_id: trade.bid_order_id as i64,
            ask_order_height: 0,
            bid_order_height: 0,
            seller_id: trade.seller_id as i64,
            buyer_id: trade.buyer_id as i64,
            is_buy: trade.is_buy,
            quantity: trade.quantity as i64,
            price: trade.price as i64,
            timestamp: trade.timestamp as i32,
            height: trade.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct ExchangeModel {
    pub db_id: i64,
    pub transaction_id: i64,
    pub currency_id: i64,
    pub block_id: i64,
    pub offer_id: i64,
    pub seller_id: i64,
    pub buyer_id: i64,
    pub units: i64,
    pub rate: i64,
    pub timestamp: i32,
    pub height: i32,
}
impl ExchangeModel {
    pub fn to_domain(&self) -> Result<Exchange> {
        Ok(Exchange {
            id: self.db_id as ExchangeId,
            transaction_id: self.transaction_id as TransactionId,
            currency_id: self.currency_id as CurrencyId,
            block_id: self.block_id as BlockId,
            offer_id: self.offer_id as OrderId,
            seller_id: self.seller_id as AccountId,
            buyer_id: self.buyer_id as AccountId,
            units: self.units as Amount,
            rate: self.rate as Amount,
            timestamp: self.timestamp as Timestamp,
            height: self.height as Height,
        })
    }

    pub fn from_domain(exchange: &Exchange) -> Result<Self> {
        Ok(Self {
            db_id: exchange.id as i64,
            transaction_id: exchange.transaction_id as i64,
            currency_id: exchange.currency_id as i64,
            block_id: exchange.block_id as i64,
            offer_id: exchange.offer_id as i64,
            seller_id: exchange.seller_id as i64,
            buyer_id: exchange.buyer_id as i64,
            units: exchange.units as i64,
            rate: exchange.rate as i64,
            timestamp: exchange.timestamp as i32,
            height: exchange.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct ExchangeRequestModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub currency_id: i64,
    pub units: i64,
    pub rate: i64,
    pub is_buy: bool,
    pub timestamp: i32,
    pub height: i32,
}

impl ExchangeRequestModel {
    pub fn to_domain(&self) -> Result<ExchangeRequest> {
        Ok(ExchangeRequest {
            id: self.id as ExchangeRequestId,
            account_id: self.account_id as AccountId,
            currency_id: self.currency_id as CurrencyId,
            units: self.units as Amount,
            rate: self.rate as Amount,
            is_buy: self.is_buy,
            timestamp: self.timestamp as Timestamp,
            height: self.height as Height,
        })
    }

    pub fn from_domain(req: &ExchangeRequest) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: req.id as i64,
            account_id: req.account_id as i64,
            currency_id: req.currency_id as i64,
            units: req.units as i64,
            rate: req.rate as i64,
            is_buy: req.is_buy,
            timestamp: req.timestamp as i32,
            height: req.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PollModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub options: String,
    pub min_num_options: Option<i16>,
    pub max_num_options: Option<i16>,
    pub min_range_value: Option<i16>,
    pub max_range_value: Option<i16>,
    pub timestamp: i32,
    pub finish_height: i32,
    pub voting_model: i16,
    pub min_balance: Option<i64>,
    pub min_balance_model: Option<i16>,
    pub holding_id: Option<i64>,
    pub height: i32,
}

impl PollModel {
    pub fn to_domain(&self) -> Result<Poll> {
        let options: Vec<String> = serde_json::from_str(&self.options).unwrap_or_default();
        Ok(Poll {
            id: self.id as PollId,
            account_id: self.account_id as AccountId,
            name: self.name.clone(),
            description: self.description.clone().unwrap_or_default(),
            options,
            min_num_options: self.min_num_options.map(|m| m as u8),
            max_num_options: self.max_num_options.map(|m| m as u8),
            min_range_value: self.min_range_value.map(|m| m as i32),
            max_range_value: self.max_range_value.map(|m| m as i32),
            timestamp: self.timestamp as Timestamp,
            finish_height: self.finish_height as Height,
            voting_model: self.voting_model as u8,
            min_balance: self.min_balance.map(|b| b as Amount),
            min_balance_model: self.min_balance_model.map(|m| m as u8),
            holding_id: self.holding_id.map(|h| h as u64),
            height: self.height as Height,
        })
    }

    pub fn from_domain(poll: &Poll) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: poll.id as i64,
            account_id: poll.account_id as i64,
            name: poll.name.clone(),
            description: Some(poll.description.clone()),
            options: serde_json::to_string(&poll.options).unwrap_or_default(),
            min_num_options: poll.min_num_options.map(|m| m as i16),
            max_num_options: poll.max_num_options.map(|m| m as i16),
            min_range_value: poll.min_range_value.map(|m| m as i16),
            max_range_value: poll.max_range_value.map(|m| m as i16),
            timestamp: poll.timestamp as i32,
            finish_height: poll.finish_height as i32,
            voting_model: poll.voting_model as i16,
            min_balance: poll.min_balance.map(|b| b as i64),
            min_balance_model: poll.min_balance_model.map(|m| m as i16),
            holding_id: poll.holding_id.map(|h| h as i64),
            height: poll.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct VoteModel {
    pub db_id: i64,
    pub id: i64,
    pub poll_id: i64,
    pub voter_id: i64,
    pub vote_bytes: Vec<u8>,
    pub height: i32,
}

impl VoteModel {
    pub fn to_domain(&self) -> Result<Vote> {
        Ok(Vote {
            id: self.id as VoteId,
            poll_id: self.poll_id as PollId,
            voter_id: self.voter_id as AccountId,
            vote_bytes: self.vote_bytes.clone(),
            height: self.height as Height,
        })
    }

    pub fn from_domain(vote: &Vote) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: vote.id as i64,
            poll_id: vote.poll_id as i64,
            voter_id: vote.voter_id as i64,
            vote_bytes: vote.vote_bytes.clone(),
            height: vote.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PollResultModel {
    pub db_id: i64,
    pub poll_id: i64,
    pub result: Option<i64>,
    pub weight: i64,
    pub height: i32,
}

impl PollResultModel {
    pub fn to_domain(&self) -> Result<PollResult> {
        Ok(PollResult {
            poll_id: self.poll_id as PollId,
            result: self.result.map(|r| r as u64),
            weight: self.weight as u64,
            height: self.height as Height,
        })
    }

    pub fn from_domain(result: &PollResult) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            poll_id: result.poll_id as i64,
            result: result.result.map(|r| r as i64),
            weight: result.weight as i64,
            height: result.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PhasingPollModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub whitelist_size: i16,
    pub finish_height: i32,
    pub voting_model: i16,
    pub quorum: Option<i64>,
    pub min_balance: Option<i64>,
    pub holding_id: Option<i64>,
    pub min_balance_model: Option<i16>,
    pub hashed_secret: Option<Vec<u8>>,
    pub algorithm: Option<i16>,
    pub height: i32,
}

impl PhasingPollModel {
    pub fn to_domain(&self) -> Result<PhasingPoll> {
        Ok(PhasingPoll {
            id: self.id as PhasingPollId,
            account_id: self.account_id as AccountId,
            whitelist_size: self.whitelist_size as u16,
            finish_height: self.finish_height as Height,
            voting_model: self.voting_model as u8,
            quorum: self.quorum.map(|q| q as u64),
            min_balance: self.min_balance.map(|b| b as Amount),
            holding_id: self.holding_id.map(|h| h as u64),
            min_balance_model: self.min_balance_model.map(|m| m as u8),
            hashed_secret: self.hashed_secret.clone(),
            algorithm: self.algorithm.map(|a| a as u8),
            height: self.height as Height,
        })
    }

    pub fn from_domain(poll: &PhasingPoll) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: poll.id as i64,
            account_id: poll.account_id as i64,
            whitelist_size: poll.whitelist_size as i16,
            finish_height: poll.finish_height as i32,
            voting_model: poll.voting_model as i16,
            quorum: poll.quorum.map(|q| q as i64),
            min_balance: poll.min_balance.map(|b| b as i64),
            holding_id: poll.holding_id.map(|h| h as i64),
            min_balance_model: poll.min_balance_model.map(|m| m as i16),
            hashed_secret: poll.hashed_secret.clone(),
            algorithm: poll.algorithm.map(|a| a as i16),
            height: poll.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PhasingPollResultModel {
    pub db_id: i64,
    pub id: i64,
    pub result: i64,
    pub approved: bool,
    pub height: i32,
}

impl PhasingPollResultModel {
    pub fn to_domain(&self) -> Result<PhasingPollResult> {
        Ok(PhasingPollResult {
            id: self.id as PhasingPollResultId,
            result: self.result as u64,
            approved: self.approved,
            height: self.height as Height,
        })
    }

    pub fn from_domain(result: &PhasingPollResult) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: result.id as i64,
            result: result.result as i64,
            approved: result.approved,
            height: result.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PhasingPollLinkedTransactionModel {
    pub db_id: i64,
    pub transaction_id: i64,
    pub linked_full_hash: Vec<u8>,
    pub linked_transaction_id: i64,
    pub height: i32,
}
impl PhasingPollLinkedTransactionModel {
    pub fn to_domain(&self) -> Result<PhasingPollLinkedTransaction> {
        Ok(PhasingPollLinkedTransaction {
            transaction_id: self.transaction_id as TransactionId,
            linked_transaction_id: self.linked_transaction_id as TransactionId,
            linked_full_hash: self
                .linked_full_hash
                .as_slice()
                .try_into()
                .unwrap_or([0u8; 32]),
            height: self.height as Height,
        })
    }

    pub fn from_domain(linked: &PhasingPollLinkedTransaction) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            transaction_id: linked.transaction_id as i64,
            referenced_transaction_full_hash: Some(linked.linked_full_hash.to_vec()),
            linked_transaction_id: linked.linked_transaction_id as i64,
            height: linked.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PhasingPollHashedSecretModel {
    pub db_id: i64,
    pub hashed_secret: Vec<u8>,
    pub hashed_secret_id: i64,
    pub algorithm: i16,
    pub transaction_full_hash: Option<Vec<u8>>,
    pub transaction_id: i64,
    pub chain_id: i32,
    pub finish_height: i32,
    pub height: i32,
}
impl PhasingPollHashedSecretModel {
    pub fn to_domain(&self) -> Result<PhasingPollHashedSecret> {
        Ok(PhasingPollHashedSecret {
            hashed_secret: self.hashed_secret.clone(),
            hashed_secret_id: self.hashed_secret_id as Hash256,
            algorithm: self.algorithm as u8,
            transaction_id: self.transaction_id as TransactionId,
            transaction_full_hash: self
                .transaction_full_hash
                .as_ref()
                .and_then(|h| h.as_slice().try_into().ok()),
            chain_id: self.chain_id as u32,
            finish_height: self.finish_height as Height,
            height: self.height as Height,
        })
    }

    pub fn from_domain(secret: &PhasingPollHashedSecret) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            hashed_secret: secret.hashed_secret.clone(),
            hashed_secret_id: secret.hashed_secret_id as i64,
            algorithm: secret.algorithm as i16,
            transaction_full_hash: secret.transaction_full_hash.map(|h| h.to_vec()),
            transaction_id: secret.transaction_id as i64,
            chain_id: secret.chain_id as i32,
            finish_height: secret.finish_height as i32,
            height: secret.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PhasingVoteModel {
    pub db_id: i64,
    pub vote_id: i64,
    pub transaction_id: i64,
    pub voter_id: i64,
    pub height: i32,
}

impl PhasingVoteModel {
    pub fn to_domain(&self) -> Result<PhasingVote> {
        Ok(PhasingVote {
            vote_id: self.vote_id as VoteId,
            transaction_id: self.transaction_id as TransactionId,
            voter_id: self.voter_id as AccountId,
            height: self.height as Height,
        })
    }

    pub fn from_domain(vote: &PhasingVote) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            vote_id: vote.vote_id as i64,
            transaction_id: vote.transaction_id as i64,
            voter_id: vote.voter_id as i64,
            height: vote.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PhasingPollVoterModel {
    pub db_id: i64,
    pub transaction_id: i64,
    pub voter_id: i64,
    pub height: i32,
}
impl PhasingPollVoterModel {
    pub fn to_domain(&self) -> Result<PhasingPollVoter> {
        Ok(PhasingPollVoter {
            transaction_id: self.transaction_id as TransactionId,
            voter_id: self.voter_id as AccountId,
            height: self.height as Height,
        })
    }

    pub fn from_domain(voter: &PhasingPollVoter) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            transaction_id: voter.transaction_id as i64,
            voter_id: voter.voter_id as i64,
            height: voter.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct ShufflingModel {
    pub db_id: i64,
    pub id: i64,
    pub holding_id: Option<i64>,
    pub holding_type: i16,
    pub issuer_id: i64,
    pub amount: i64,
    pub participant_count: i16,
    pub blocks_remaining: Option<i16>,
    pub stage: i16,
    pub assignee_account_id: Option<i64>,
    pub registrant_count: i16,
    pub recipient_public_keys: Option<String>,
    pub height: i32,
    pub latest: bool,
}

impl ShufflingModel {
    pub fn to_domain(&self) -> Result<Shuffling> {
        Ok(Shuffling {
            id: self.id as ShufflingId,
            holding_id: self.holding_id.map(|h| h as u64),
            holding_type: self.holding_type as u8,
            issuer_id: self.issuer_id as AccountId,
            amount: self.amount as Amount,
            participant_count: self.participant_count as u8,
            blocks_remaining: self.blocks_remaining.map(|b| b as u16),
            stage: self.stage as u8,
            assignee_account_id: self.assignee_account_id.map(|a| a as AccountId),
            registrant_count: self.registrant_count as u8,
            recipient_public_keys: self
                .recipient_public_keys
                .as_ref()
                .and_then(|k| serde_json::from_str(k).ok())
                .unwrap_or_default(),
            height: self.height as Height,
        })
    }

    pub fn from_domain(shuffle: &Shuffling) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: shuffle.id as i64,
            holding_id: shuffle.holding_id.map(|h| h as i64),
            holding_type: shuffle.holding_type as i16,
            issuer_id: shuffle.issuer_id as i64,
            amount: shuffle.amount as i64,
            participant_count: shuffle.participant_count as i16,
            blocks_remaining: shuffle.blocks_remaining.map(|b| b as i16),
            stage: shuffle.stage as i16,
            assignee_account_id: shuffle.assignee_account_id.map(|a| a as i64),
            registrant_count: shuffle.registrant_count as i16,
            recipient_public_keys: Some(
                serde_json::to_string(&shuffle.recipient_public_keys).unwrap_or_default(),
            ),
            height: shuffle.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct ShufflingDataModel {
    pub db_id: i64,
    pub shuffling_id: i64,
    pub account_id: i64,
    pub data: Option<String>,
    pub transaction_timestamp: i32,
    pub height: i32,
}

impl ShufflingDataModel {
    pub fn to_domain(&self) -> Result<ShufflingData> {
        Ok(ShufflingData {
            shuffling_id: self.shuffling_id as ShufflingId,
            account_id: self.account_id as AccountId,
            data: self.data.clone(),
            transaction_timestamp: self.transaction_timestamp as Timestamp,
            height: self.height as Height,
        })
    }

    pub fn from_domain(data: &ShufflingData) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            shuffling_id: data.shuffling_id as i64,
            account_id: data.account_id as i64,
            data: Some(data.data.clone()),
            transaction_timestamp: data.transaction_timestamp as i32,
            height: data.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct ShufflingParticipantModel {
    pub db_id: i64,
    pub shuffling_id: i64,
    pub account_id: i64,
    pub next_account_id: Option<i64>,
    pub participant_index: i16,
    pub state: i16,
    pub blame_data: Option<String>,
    pub key_seeds: Option<String>,
    pub data_transaction_full_hash: Option<Vec<u8>>,
    pub height: i32,
    pub latest: bool,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct TaggedDataModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub parsed_tags: Option<String>,
    pub type_: Option<String>,
    pub data: Vec<u8>,
    pub is_text: bool,
    pub filename: Option<String>,
    pub channel: Option<String>,
    pub block_timestamp: i32,
    pub transaction_timestamp: i32,
    pub height: i32,
    pub latest: bool,
}

impl TaggedDataModel {
    pub fn to_domain(&self) -> Result<TaggedData> {
        Ok(TaggedData {
            id: self.id as TaggedDataId,
            account_id: self.account_id as AccountId,
            name: self.name.clone(),
            description: self.description.clone().unwrap_or_default(),
            tags: self.tags.clone(),
            parsed_tags: self
                .parsed_tags
                .as_ref()
                .and_then(|t| serde_json::from_str(t).ok())
                .unwrap_or_default(),
            type_: self.type_.clone(),
            data: self.data.clone(),
            is_text: self.is_text,
            filename: self.filename.clone(),
            channel: self.channel.clone(),
            block_timestamp: self.block_timestamp as Timestamp,
            transaction_timestamp: self.transaction_timestamp as Timestamp,
            height: self.height as Height,
        })
    }

    pub fn from_domain(data: &TaggedData) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: data.id as i64,
            account_id: data.account_id as i64,
            name: data.name.clone(),
            description: Some(data.description.clone()),
            tags: Some(data.tags.clone()),
            parsed_tags: Some(serde_json::to_string(&data.parsed_tags).unwrap_or_default()),
            type_: Some(data.type_.clone()),
            data: data.data.clone(),
            is_text: data.is_text,
            filename: Some(data.filename.clone()),
            channel: Some(data.channel.clone()),
            block_timestamp: data.block_timestamp as i32,
            transaction_timestamp: data.transaction_timestamp as i32,
            height: data.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct TaggedDataExtendModel {
    pub db_id: i64,
    pub id: i64,
    pub extend_id: i64,
    pub height: i32,
    pub latest: bool,
}

impl TaggedDataExtendModel {
    pub fn to_domain(&self) -> Result<TaggedDataExtend> {
        Ok(TaggedDataExtend {
            id: self.id as TaggedDataExtendId,
            extend_id: self.extend_id as TaggedDataId,
            height: self.height as Height,
        })
    }

    pub fn from_domain(extend: &TaggedDataExtend) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: extend.id as i64,
            extend_id: extend.extend_id as i64,
            height: extend.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct TaggedDataTimestampModel {
    pub db_id: i64,
    pub id: i64,
    pub timestamp: i32,
    pub height: i32,
    pub latest: bool,
}

impl TaggedDataTimestampModel {
    pub fn to_domain(&self) -> Result<TaggedDataTimestamp> {
        Ok(TaggedDataTimestamp {
            id: self.id as TaggedDataTimestampId,
            timestamp: self.timestamp as Timestamp,
            height: self.height as Height,
        })
    }

    pub fn from_domain(ts: &TaggedDataTimestamp) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: ts.id as i64,
            timestamp: ts.timestamp as i32,
            height: ts.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PrunableMessageModel {
    pub db_id: i64,
    pub id: i64,
    pub sender_id: i64,
    pub recipient_id: Option<i64>,
    pub message: Option<Vec<u8>>,
    pub message_is_text: bool,
    pub is_compressed: bool,
    pub encrypted_message: Option<Vec<u8>>,
    pub encrypted_is_text: Option<bool>,
    pub block_timestamp: i32,
    pub transaction_timestamp: i32,
    pub height: i32,
}

impl PrunableMessageModel {
    pub fn to_domain(&self) -> Result<PrunableMessage> {
        Ok(PrunableMessage {
            id: self.id as MessageId,
            sender_id: self.sender_id as AccountId,
            recipient_id: self.recipient_id.map(|r| r as AccountId),
            message: self.message.clone(),
            message_is_text: self.message_is_text,
            is_compressed: self.is_compressed,
            encrypted_message: self.encrypted_message.clone(),
            encrypted_is_text: self.encrypted_is_text,
            block_timestamp: self.block_timestamp as Timestamp,
            transaction_timestamp: self.transaction_timestamp as Timestamp,
            height: self.height as Height,
        })
    }

    pub fn from_domain(msg: &PrunableMessage) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: msg.id as i64,
            sender_id: msg.sender_id as i64,
            recipient_id: msg.recipient_id.map(|r| r as i64),
            message: Some(msg.message.clone()),
            message_is_text: msg.message_is_text,
            is_compressed: msg.is_compressed,
            encrypted_message: msg.encrypted_message.clone(),
            encrypted_is_text: Some(msg.encrypted_is_text),
            block_timestamp: msg.block_timestamp as i32,
            transaction_timestamp: msg.transaction_timestamp as i32,
            height: msg.height as i32,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct GoodsModel {
    pub db_id: i64,
    pub id: i64,
    pub seller_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub parsed_tags: Option<String>,
    pub tags: Option<String>,
    pub timestamp: i32,
    pub quantity: i32,
    pub price: i64,
    pub delisted: bool,
    pub height: i32,
    pub latest: bool,
    pub has_image: bool,
}
impl GoodsModel {
    pub fn to_domain(&self) -> Result<Goods> {
        Ok(Goods {
            id: self.id as GoodsId,
            seller_id: self.seller_id as AccountId,
            name: self.name.clone(),
            description: self.description.clone().unwrap_or_default(),
            tags: self.tags.clone(),
            parsed_tags: self
                .parsed_tags
                .as_ref()
                .and_then(|t| serde_json::from_str(t).ok())
                .unwrap_or_default(),
            timestamp: self.timestamp as Timestamp,
            quantity: self.quantity as u32,
            price: self.price as Amount,
            delisted: self.delisted,
            height: self.height as Height,
            has_image: self.has_image,
        })
    }

    pub fn from_domain(goods: &Goods) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: goods.id as i64,
            seller_id: goods.seller_id as i64,
            name: goods.name.clone(),
            description: Some(goods.description.clone()),
            parsed_tags: Some(serde_json::to_string(&goods.parsed_tags).unwrap_or_default()),
            tags: Some(goods.tags.clone()),
            timestamp: goods.timestamp as i32,
            quantity: goods.quantity as i32,
            price: goods.price as i64,
            delisted: goods.delisted,
            height: goods.height as i32,
            latest: true,
            has_image: goods.has_image,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PurchaseModel {
    pub db_id: i64,
    pub id: i64,
    pub buyer_id: i64,
    pub goods_id: i64,
    pub seller_id: i64,
    pub quantity: i32,
    pub price: i64,
    pub deadline: i32,
    pub note: Option<Vec<u8>>,
    pub nonce: Option<Vec<u8>>,
    pub timestamp: i32,
    pub pending: bool,
    pub goods: Option<Vec<u8>>,
    pub goods_nonce: Option<Vec<u8>>,
    pub goods_is_text: bool,
    pub refund_note: Option<Vec<u8>>,
    pub refund_nonce: Option<Vec<u8>>,
    pub has_feedback_notes: bool,
    pub has_public_feedbacks: bool,
    pub discount: i64,
    pub refund: i64,
    pub height: i32,
    pub latest: bool,
}

impl PurchaseModel {
    pub fn to_domain(&self) -> Result<Purchase> {
        Ok(Purchase {
            id: self.id as PurchaseId,
            buyer_id: self.buyer_id as AccountId,
            goods_id: self.goods_id as GoodsId,
            seller_id: self.seller_id as AccountId,
            quantity: self.quantity as u32,
            price: self.price as Amount,
            deadline: self.deadline as u32,
            note: self.note.clone(),
            nonce: self.nonce.clone(),
            timestamp: self.timestamp as Timestamp,
            pending: self.pending,
            goods: self.goods.clone(),
            goods_nonce: self.goods_nonce.clone(),
            goods_is_text: self.goods_is_text,
            refund_note: self.refund_note.clone(),
            refund_nonce: self.refund_nonce.clone(),
            has_feedback_notes: self.has_feedback_notes,
            has_public_feedbacks: self.has_public_feedbacks,
            discount: self.discount as Amount,
            refund: self.refund as Amount,
            height: self.height as Height,
        })
    }

    pub fn from_domain(purchase: &Purchase) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: purchase.id as i64,
            buyer_id: purchase.buyer_id as i64,
            goods_id: purchase.goods_id as i64,
            seller_id: purchase.seller_id as i64,
            quantity: purchase.quantity as i32,
            price: purchase.price as i64,
            deadline: purchase.deadline as i32,
            note: purchase.note.clone(),
            nonce: purchase.nonce.clone(),
            timestamp: purchase.timestamp as i32,
            pending: purchase.pending,
            goods: purchase.goods.clone(),
            goods_nonce: purchase.goods_nonce.clone(),
            goods_is_text: purchase.goods_is_text,
            refund_note: purchase.refund_note.clone(),
            refund_nonce: purchase.refund_nonce.clone(),
            has_feedback_notes: purchase.has_feedback_notes,
            has_public_feedbacks: purchase.has_public_feedbacks,
            discount: purchase.discount as i64,
            refund: purchase.refund as i64,
            height: purchase.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PurchaseFeedbackModel {
    pub db_id: i64,
    pub id: i64,
    pub feedback_data: Vec<u8>,
    pub feedback_nonce: Vec<u8>,
    pub height: i32,
    pub latest: bool,
}

impl PurchaseFeedbackModel {
    pub fn to_domain(&self) -> Result<PurchaseFeedback> {
        Ok(PurchaseFeedback {
            id: self.id as FeedbackId,
            feedback_data: self.feedback_data.clone(),
            feedback_nonce: self.feedback_nonce.clone(),
            height: self.height as Height,
        })
    }

    pub fn from_domain(feedback: &PurchaseFeedback) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: feedback.id as i64,
            feedback_data: feedback.feedback_data.clone(),
            feedback_nonce: feedback.feedback_nonce.clone(),
            height: feedback.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PurchasePublicFeedbackModel {
    pub db_id: i64,
    pub id: i64,
    pub public_feedback: String,
    pub height: i32,
    pub latest: bool,
}
impl PurchasePublicFeedbackModel {
    pub fn to_domain(&self) -> Result<PurchasePublicFeedback> {
        Ok(PurchasePublicFeedback {
            id: self.id as PublicFeedbackId,
            public_feedback: self.public_feedback.clone(),
            height: self.height as Height,
        })
    }

    pub fn from_domain(feedback: &PurchasePublicFeedback) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: feedback.id as i64,
            public_feedback: feedback.public_feedback.clone(),
            height: feedback.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PeerModel {
    pub address: String,
    pub last_updated: Option<i32>,
    pub services: Option<i64>,
}

impl PeerModel {
    pub fn to_domain(&self) -> Result<Peer> {
        Ok(Peer {
            address: self.address.clone(),
            last_updated: self.last_updated.map(|t| t as Timestamp),
            services: self.services.map(|s| s as u64),
        })
    }

    pub fn from_domain(peer: &Peer) -> Result<Self> {
        Ok(Self {
            address: peer.address.clone(),
            last_updated: peer.last_updated.map(|t| t as i32),
            services: peer.services.map(|s| s as i64),
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct HubModel {
    pub db_id: i64,
    pub account_id: Option<i64>,
    pub min_fee_per_byte: Option<i64>,
    pub uris: Option<String>,
    pub height: Option<i32>,
    pub latest: Option<bool>,
}
impl HubModel {
    pub fn to_domain(&self) -> Result<HubInfo> {
        Ok(HubInfo {
            account_id: self.account_id.map(|a| a as AccountId),
            min_fee_per_byte: self.min_fee_per_byte.map(|f| f as Amount),
            uris: self.uris.clone(),
            height: self.height.map(|h| h as Height),
        })
    }

    pub fn from_domain(hub: &HubInfo) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            account_id: hub.account_id.map(|a| a as i64),
            min_fee_per_byte: hub.min_fee_per_byte.map(|f| f as i64),
            uris: hub.uris.clone(),
            height: hub.height.map(|h| h as i32),
            latest: Some(true),
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct CoinOrderFxtModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub chain_id: i32,
    pub exchange_id: i32,
    pub full_hash: Vec<u8>,
    pub amount: i64,
    pub quantity: i64,
    pub bid_price: i64,
    pub ask_price: i64,
    pub creation_height: i32,
    pub height: i32,
    pub transaction_height: i32,
    pub transaction_index: i16,
    pub latest: bool,
}
impl CoinOrderFxtModel {
    pub fn to_domain(&self) -> Result<CoinOrderFxt> {
        Ok(CoinOrderFxt {
            id: self.id as OrderId,
            account_id: self.account_id as AccountId,
            chain_id: self.chain_id as u32,
            exchange_id: self.exchange_id as u32,
            full_hash: self.full_hash.as_slice().try_into().unwrap_or([0u8; 32]),
            amount: self.amount as Amount,
            quantity: self.quantity as Amount,
            bid_price: self.bid_price as Amount,
            ask_price: self.ask_price as Amount,
            creation_height: self.creation_height as Height,
            height: self.height as Height,
            transaction_height: self.transaction_height as Height,
            transaction_index: self.transaction_index as u16,
        })
    }

    pub fn from_domain(order: &CoinOrderFxt) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: order.id as i64,
            account_id: order.account_id as i64,
            chain_id: order.chain_id as i32,
            exchange_id: order.exchange_id as i32,
            full_hash: order.full_hash.to_vec(),
            amount: order.amount as i64,
            quantity: order.quantity as i64,
            bid_price: order.bid_price as i64,
            ask_price: order.ask_price as i64,
            creation_height: order.creation_height as i32,
            height: order.height as i32,
            transaction_height: order.transaction_height as i32,
            transaction_index: order.transaction_index as i16,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct CoinTradeFxtModel {
    pub db_id: i64,
    pub chain_id: i32,
    pub exchange_id: i32,
    pub account_id: i64,
    pub block_id: i64,
    pub height: i32,
    pub timestamp: i32,
    pub exchange_quantity: i64,
    pub exchange_price: i64,
    pub order_id: i64,
    pub order_full_hash: Vec<u8>,
    pub match_id: i64,
    pub match_full_hash: Vec<u8>,
}
impl CoinTradeFxtModel {
    pub fn to_domain(&self) -> Result<CoinTradeFxt> {
        Ok(CoinTradeFxt {
            chain_id: self.chain_id as u32,
            exchange_id: self.exchange_id as u32,
            account_id: self.account_id as AccountId,
            block_id: self.block_id as BlockId,
            height: self.height as Height,
            timestamp: self.timestamp as Timestamp,
            exchange_quantity: self.exchange_quantity as Amount,
            exchange_price: self.exchange_price as Amount,
            order_id: self.order_id as OrderId,
            order_full_hash: self
                .order_full_hash
                .as_slice()
                .try_into()
                .unwrap_or([0u8; 32]),
            match_id: self.match_id as MatchId,
            match_full_hash: self
                .match_full_hash
                .as_slice()
                .try_into()
                .unwrap_or([0u8; 32]),
        })
    }

    pub fn from_domain(trade: &CoinTradeFxt) -> Result<Self> {
        Ok(Self {
            chain_id: trade.chain_id as i32,
            exchange_id: trade.exchange_id as i32,
            account_id: trade.account_id as i64,
            block_id: trade.block_id as i64,
            height: trade.height as i32,
            timestamp: trade.timestamp as i32,
            exchange_quantity: trade.exchange_quantity as i64,
            exchange_price: trade.exchange_price as i64,
            order_id: trade.order_id as i64,
            order_full_hash: trade.order_full_hash.to_vec(),
            match_id: trade.match_id as i64,
            match_full_hash: trade.match_full_hash.to_vec(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct BalanceModel {
    pub db_id: i64,
    pub account_id: i64,
    pub balance: i64,
    pub unconfirmed_balance: i64,
    pub height: i32,
    pub latest: bool,
}

impl BalanceModel {
    pub fn to_domain(&self) -> Result<Balance> {
        Ok(Balance {
            account_id: self.account_id as AccountId,
            balance: self.balance as Amount,
            unconfirmed_balance: self.unconfirmed_balance as Amount,
            height: self.height as Height,
        })
    }

    pub fn from_domain(balance: &Balance) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            account_id: balance.account_id as i64,
            balance: balance.balance as i64,
            unconfirmed_balance: balance.unconfirmed_balance as i64,
            height: balance.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AssetPropertyModel {
    pub db_id: i64,
    pub id: i64,
    pub asset_id: i64,
    pub setter_id: i64,
    pub property: String,
    pub value: Option<String>,
    pub height: i32,
    pub latest: bool,
}

impl AssetPropertyModel {
    pub fn to_domain(&self) -> Result<AssetProperty> {
        Ok(AssetProperty {
            id: self.id as PropertyId,
            asset_id: self.asset_id as AssetId,
            setter_id: self.setter_id as AccountId,
            property: self.property.clone(),
            value: self.value.clone(),
            height: self.height as Height,
        })
    }

    pub fn from_domain(prop: &AssetProperty) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: prop.id as i64,
            asset_id: prop.asset_id as i64,
            setter_id: prop.setter_id as i64,
            property: prop.property.clone(),
            value: Some(prop.value.clone()),
            height: prop.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AssetControlPhasingSubPollModel {
    pub db_id: i64,
    pub asset_id: i64,
    pub name: Option<String>,
    pub voting_model: i16,
    pub quorum: Option<i64>,
    pub min_balance: Option<i64>,
    pub holding_id: Option<i64>,
    pub min_balance_model: Option<i16>,
    pub whitelist: Option<String>,
    pub sender_property_setter_id: Option<i64>,
    pub sender_property_name: Option<String>,
    pub sender_property_value: Option<String>,
    pub recipient_property_setter_id: Option<i64>,
    pub recipient_property_name: Option<String>,
    pub recipient_property_value: Option<String>,
    pub height: i32,
    pub latest: bool,
}

impl AssetControlPhasingSubPollModel {
    pub fn to_domain(&self) -> Result<AssetControlPhasingSubPoll> {
        Ok(AssetControlPhasingSubPoll {
            asset_id: self.asset_id as AssetId,
            name: self.name.clone(),
            voting_model: self.voting_model as u8,
            quorum: self.quorum.map(|q| q as u64),
            min_balance: self.min_balance.map(|b| b as Amount),
            holding_id: self.holding_id.map(|h| h as u64),
            min_balance_model: self.min_balance_model.map(|m| m as u8),
            whitelist: self
                .whitelist
                .as_ref()
                .and_then(|w| serde_json::from_str(w).ok())
                .unwrap_or_default(),
            sender_property_setter_id: self.sender_property_setter_id.map(|s| s as AccountId),
            sender_property_name: self.sender_property_name.clone(),
            sender_property_value: self.sender_property_value.clone(),
            recipient_property_setter_id: self.recipient_property_setter_id.map(|s| s as AccountId),
            recipient_property_name: self.recipient_property_name.clone(),
            recipient_property_value: self.recipient_property_value.clone(),
            height: self.height as Height,
        })
    }

    pub fn from_domain(subpoll: &AssetControlPhasingSubPoll) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            asset_id: subpoll.asset_id as i64,
            name: Some(subpoll.name.clone()),
            voting_model: subpoll.voting_model as i16,
            quorum: subpoll.quorum.map(|q| q as i64),
            min_balance: subpoll.min_balance.map(|b| b as i64),
            holding_id: subpoll.holding_id.map(|h| h as i64),
            min_balance_model: subpoll.min_balance_model.map(|m| m as i16),
            whitelist: Some(serde_json::to_string(&subpoll.whitelist).unwrap_or_default()),
            sender_property_setter_id: subpoll.sender_property_setter_id.map(|s| s as i64),
            sender_property_name: Some(subpoll.sender_property_name.clone()),
            sender_property_value: Some(subpoll.sender_property_value.clone()),
            recipient_property_setter_id: subpoll.recipient_property_setter_id.map(|s| s as i64),
            recipient_property_name: Some(subpoll.recipient_property_name.clone()),
            recipient_property_value: Some(subpoll.recipient_property_value.clone()),
            height: subpoll.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct ScanModel {
    pub rescan: bool,
    pub height: i32,
    pub validate: bool,
}

impl ScanModel {
    pub fn to_domain(&self) -> Result<Scan> {
        Ok(Scan {
            rescan: self.rescan,
            height: self.height as Height,
            validate: self.validate,
        })
    }

    pub fn from_domain(scan: &Scan) -> Result<Self> {
        Ok(Self {
            rescan: scan.rescan,
            height: scan.height as i32,
            validate: scan.validate,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct TagModel {
    pub db_id: i64,
    pub tag: String,
    pub in_stock_count: i32,
    pub total_count: i32,
    pub height: i32,
    pub latest: bool,
}
impl TagModel {
    pub fn to_domain(&self) -> Result<Tag> {
        Ok(Tag {
            tag: self.tag.clone(),
            in_stock_count: self.in_stock_count as u32,
            total_count: self.total_count as u32,
            height: self.height as Height,
        })
    }

    pub fn from_domain(tag: &Tag) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            tag: tag.tag.clone(),
            in_stock_count: tag.in_stock_count as i32,
            total_count: tag.total_count as i32,
            height: tag.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct DataTagModel {
    pub db_id: i64,
    pub tag: String,
    pub tag_count: i32,
    pub height: i32,
    pub latest: bool,
}
impl DataTagModel {
    pub fn to_domain(&self) -> Result<DataTag> {
        Ok(DataTag {
            tag: self.tag.clone(),
            tag_count: self.tag_count as u32,
            height: self.height as Height,
        })
    }

    pub fn from_domain(data_tag: &DataTag) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            tag: data_tag.tag.clone(),
            tag_count: data_tag.tag_count as i32,
            height: data_tag.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct UnconfirmedTransactionModel {
    pub db_id: i64,
    pub id: i64,
    pub expiration: i32,
    pub transaction_height: i32,
    pub fee_per_byte: i64,
    pub arrival_timestamp: i64,
    pub transaction_bytes: Vec<u8>,
    pub height: i32,
    pub prunable_json: Option<String>,
}

impl UnconfirmedTransactionModel {
    pub fn to_domain(&self) -> Result<UnconfirmedTransaction> {
        Ok(UnconfirmedTransaction {
            id: self.id as TransactionId,
            expiration: self.expiration as Timestamp,
            transaction_height: self.transaction_height as Height,
            fee_per_byte: self.fee_per_byte as Amount,
            arrival_timestamp: self.arrival_timestamp as Timestamp,
            transaction_bytes: self.transaction_bytes.clone(),
            height: self.height as Height,
            prunable_json: self.prunable_json.clone(),
        })
    }

    pub fn from_domain(utx: &UnconfirmedTransaction) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: utx.id as i64,
            expiration: utx.expiration as i32,
            transaction_height: utx.transaction_height as i32,
            fee_per_byte: utx.fee_per_byte as i64,
            arrival_timestamp: utx.arrival_timestamp as i64,
            transaction_bytes: utx.transaction_bytes.clone(),
            height: utx.height as i32,
            prunable_json: Some(utx.prunable_json.clone()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct ReferencedTransactionModel {
    pub db_id: i64,
    pub transaction_id: i64,
    pub referenced_transaction_id: i64,
}

impl ReferencedTransactionModel {
    pub fn to_domain(&self) -> Result<ReferencedTransaction> {
        Ok(ReferencedTransaction {
            transaction_id: self.transaction_id as TransactionId,
            referenced_transaction_id: self.referenced_transaction_id as TransactionId,
        })
    }

    pub fn from_domain(rt: &ReferencedTransaction) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            transaction_id: rt.transaction_id as i64,
            referenced_transaction_id: rt.referenced_transaction_id as i64,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct ContractReferenceModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub contract_name: String,
    pub contract_params: Option<String>,
    pub contract_transaction_chain_id: i32,
    pub contract_transaction_full_hash: Option<Vec<u8>>,
    pub height: i32,
    pub latest: bool,
}

impl ContractReferenceModel {
    pub fn to_domain(&self) -> Result<ContractReference> {
        Ok(ContractReference {
            id: self.id as ContractReferenceId,
            account_id: self.account_id as AccountId,
            contract_name: self.contract_name.clone(),
            contract_params: self.contract_params.clone(),
            contract_transaction_chain_id: self.contract_transaction_chain_id as u32,
            contract_transaction_full_hash: self
                .contract_transaction_full_hash
                .as_ref()
                .and_then(|h| h.as_slice().try_into().ok()),
            height: self.height as Height,
        })
    }

    pub fn from_domain(contract: &ContractReference) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: contract.id as i64,
            account_id: contract.account_id as i64,
            contract_name: contract.contract_name.clone(),
            contract_params: Some(contract.contract_params.clone()),
            contract_transaction_chain_id: contract.contract_transaction_chain_id as i32,
            contract_transaction_full_hash: contract
                .contract_transaction_full_hash
                .map(|h| h.to_vec()),
            height: contract.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AccountFxtModel {
    pub id: i64,
    pub balance: Vec<u8>,
    pub height: i32,
}
impl AccountFxtModel {
    pub fn to_domain(&self) -> Result<AccountFxt> {
        Ok(AccountFxt {
            account_id: self.id as AccountId,
            balance: self.balance.clone(),
            height: self.height as Height,
        })
    }

    pub fn from_domain(fxt: &AccountFxt) -> Result<Self> {
        Ok(Self {
            id: fxt.account_id as i64,
            balance: fxt.balance.clone(),
            height: fxt.height as i32,
        })
    }
}
