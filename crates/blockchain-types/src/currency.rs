//! Currency-related domain types

use serde::{Deserialize, Serialize};

use crate::{AccountId, Amount, Height, Timestamp};

pub type CurrencyId = u64;
pub type TransferId = u64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Currency {
    pub id: CurrencyId,
    pub owner_id: AccountId,
    pub name: String,
    pub code: String,
    pub description: String,
    pub currency_type: u8,
    pub initial_supply: Amount,
    pub reserve_supply: Amount,
    pub max_supply: Amount,
    pub creation_height: Height,
    pub issuance_height: Height,
    pub min_reserve_per_unit_nqt: Amount,
    pub min_difficulty: u8,
    pub max_difficulty: u8,
    pub ruleset: u8,
    pub algorithm: u8,
    pub decimals: u8,
    pub created_at: Timestamp,
    pub last_updated: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrencyTransfer {
    pub id: TransferId,
    pub currency_id: CurrencyId,
    pub sender_id: AccountId,
    pub recipient_id: AccountId,
    pub units: Amount,
    pub timestamp: Timestamp,
    pub height: Height,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrencyFounder {
    pub currency_id: CurrencyId,
    pub account_id: AccountId,
    pub amount: Amount,
    pub height: Height,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountCurrency {
    pub account_id: AccountId,
    pub currency_id: CurrencyId,
    pub units: Amount,
    pub unconfirmed_units: Amount,
    pub height: Height,
}
