//! Phasing poll and vote-related database models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PhasingPollLinkedTransactionModel {
    pub db_id: i64,
    pub transaction_id: i64,
    pub linked_full_hash: Vec<u8>,
    pub linked_transaction_id: i64,
    pub height: i32,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PhasingPollResultModel {
    pub db_id: i64,
    pub id: i64,
    pub result: i64,
    pub approved: bool,
    pub height: i32,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PhasingPollVoterModel {
    pub db_id: i64,
    pub transaction_id: i64,
    pub voter_id: i64,
    pub height: i32,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct PhasingVoteModel {
    pub db_id: i64,
    pub vote_id: i64,
    pub transaction_id: i64,
    pub voter_id: i64,
    pub height: i32,
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
