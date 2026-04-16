//! Asset extension models (transfer, delete, dividend, etc.)

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
