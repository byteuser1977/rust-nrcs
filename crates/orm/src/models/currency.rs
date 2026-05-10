//! Currency-related database models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use blockchain_types::{AccountId, Amount, CurrencyId, Height, Result, Timestamp, TransferId};
use blockchain_types::prelude::*;

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
pub struct CurrencyModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub name: String,
    pub name_lower: String,
    pub code: String,
    pub description: Option<String>,
    pub r#type: i32,
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
            currency_type: self.r#type as u8,
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
            r#type: currency.currency_type as i32,
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
#[sqlx(rename_all = "snake_case")]
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
#[sqlx(rename_all = "snake_case")]
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
#[sqlx(rename_all = "snake_case")]
pub struct CurrencyMintModel {
    pub db_id: i64,
    pub currency_id: i64,
    pub account_id: i64,
    pub counter: i64,
    pub height: i32,
    pub latest: bool,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
pub struct CurrencySupplyModel {
    pub db_id: i64,
    pub id: i64,
    pub current_supply: i64,
    pub current_reserve_per_unit_nqt: i64,
    pub height: i32,
    pub latest: bool,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
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
#[sqlx(rename_all = "snake_case")]
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

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
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
#[sqlx(rename_all = "snake_case")]
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

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
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
