//! Poll and vote-related database models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PollResultModel {
    pub db_id: i64,
    pub poll_id: i64,
    pub result: Option<i64>,
    pub weight: i64,
    pub height: i32,
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
