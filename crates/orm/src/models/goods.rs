//! Goods and marketplace-related database models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct GoodsModel {
    pub db_id: i64,
    pub id: i64,
    pub seller_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub parsed_tags: Option<String>,
    pub tags: Option<String>,
    pub timestamp: i32,
    pub quantity: i32,
    pub price: i64,
    pub delisted: bool,
    pub height: i32,
    pub latest: bool,
    pub has_image: bool,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct PurchaseModel {
    pub db_id: i64,
    pub id: i64,
    pub buyer_id: i64,
    pub goods_id: i64,
    pub seller_id: i64,
    pub quantity: i32,
    pub price: i64,
    pub deadline: i32,
    pub note: Option<Vec<u8>>,
    pub nonce: Option<Vec<u8>>,
    pub timestamp: i32,
    pub pending: bool,
    pub goods: Option<Vec<u8>>,
    pub goods_nonce: Option<Vec<u8>>,
    pub goods_is_text: bool,
    pub refund_note: Option<Vec<u8>>,
    pub refund_nonce: Option<Vec<u8>>,
    pub has_feedback_notes: bool,
    pub has_public_feedbacks: bool,
    pub discount: i64,
    pub refund: i64,
    pub height: i32,
    pub latest: bool,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct PurchaseFeedbackModel {
    pub db_id: i64,
    pub id: i64,
    pub feedback_data: Vec<u8>,
    pub feedback_nonce: Vec<u8>,
    pub height: i32,
    pub latest: bool,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct PurchasePublicFeedbackModel {
    pub db_id: i64,
    pub id: i64,
    pub public_feedback: String,
    pub height: i32,
    pub latest: bool,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct TagModel {
    pub db_id: i64,
    pub tag: String,
    pub in_stock_count: i32,
    pub total_count: i32,
    pub height: i32,
    pub latest: bool,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct DataTagModel {
    pub db_id: i64,
    pub tag: String,
    pub tag_count: i32,
    pub height: i32,
    pub latest: bool,
}
