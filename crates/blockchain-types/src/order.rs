//! Order-related domain types

use serde::{Deserialize, Serialize};

use crate::{AccountId, Amount, AssetId, Height};

pub type OrderId = u64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AskOrder {
    pub id: OrderId,
    pub account_id: AccountId,
    pub asset_id: AssetId,
    pub price: Amount,
    pub quantity: Amount,
    pub creation_height: Height,
    pub transaction_height: Height,
    pub transaction_index: u16,
    pub height: Height,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BidOrder {
    pub id: OrderId,
    pub account_id: AccountId,
    pub asset_id: AssetId,
    pub price: Amount,
    pub quantity: Amount,
    pub creation_height: Height,
    pub transaction_height: Height,
    pub transaction_index: u16,
    pub height: Height,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SellOffer {
    pub id: OrderId,
    pub currency_id: u64,
    pub account_id: AccountId,
    pub rate: Amount,
    pub unit_limit: Amount,
    pub supply: Amount,
    pub expiration_height: Height,
    pub transaction_height: Height,
    pub creation_height: Height,
    pub transaction_index: u16,
    pub height: Height,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuyOffer {
    pub id: OrderId,
    pub currency_id: u64,
    pub account_id: AccountId,
    pub rate: Amount,
    pub unit_limit: Amount,
    pub supply: Amount,
    pub expiration_height: Height,
    pub transaction_height: Height,
    pub creation_height: Height,
    pub transaction_index: u16,
    pub height: Height,
}
