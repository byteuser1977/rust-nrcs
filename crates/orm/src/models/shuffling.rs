//! Shuffling-related database models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
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

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
pub struct ShufflingDataModel {
    pub db_id: i64,
    pub shuffling_id: i64,
    pub account_id: i64,
    pub data: Option<String>,
    pub transaction_timestamp: i32,
    pub height: i32,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
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
