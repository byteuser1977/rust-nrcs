//! Tagged data-related database models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct TaggedDataExtendModel {
    pub db_id: i64,
    pub id: i64,
    pub extend_id: i64,
    pub height: i32,
    pub latest: bool,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct TaggedDataTimestampModel {
    pub db_id: i64,
    pub id: i64,
    pub timestamp: i32,
    pub height: i32,
    pub latest: bool,
}
