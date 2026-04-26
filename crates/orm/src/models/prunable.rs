//! Prunable message-related database models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct PrunableMessageModel {
    pub db_id: i64,
    pub id: i64,
    pub sender_id: i64,
    pub recipient_id: Option<i64>,
    pub message: Option<Vec<u8>>,
    pub message_is_text: bool,
    pub is_compressed: bool,
    pub encrypted_message: Option<Vec<u8>>,
    pub encrypted_is_text: bool,
    pub block_timestamp: i32,
    pub transaction_timestamp: i32,
    pub height: i32,
}
