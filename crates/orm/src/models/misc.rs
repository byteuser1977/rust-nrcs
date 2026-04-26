//! Miscellaneous database models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct PeerModel {
    pub address: String,
    pub last_updated: Option<i32>,
    pub services: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct HubModel {
    pub db_id: i64,
    pub account_id: Option<i64>,
    pub min_fee_per_byte: Option<i64>,
    pub uris: Option<String>,
    pub height: Option<i32>,
    pub latest: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct ScanModel {
    pub rescan: bool,
    pub height: i32,
    pub validate: bool,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
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

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct ReferencedTransactionModel {
    pub db_id: i64,
    pub transaction_id: i64,
    pub referenced_transaction_id: i64,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
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

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct AccountFxtModel {
    pub id: i64,
    pub balance: Vec<u8>,
    pub height: i32,
}
