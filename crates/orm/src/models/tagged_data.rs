//! Tagged data-related database models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
pub struct TaggedDataModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub parsed_tags: Option<String>,
    #[sqlx(rename = "type_")]
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
#[sqlx(rename_all = "snake_case")]
pub struct TaggedDataExtendModel {
    pub db_id: i64,
    pub id: i64,
    pub extend_id: i64,
    pub height: i32,
    pub latest: bool,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
pub struct TaggedDataTimestampModel {
    pub db_id: i64,
    pub id: i64,
    pub timestamp: i32,
    pub height: i32,
    pub latest: bool,
}

/// Tagged Data Tag Model
///
/// 对应 Java: TaggedDataTag.java
#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
pub struct TaggedDataTagModel {
    pub db_id: i64,
    pub id: i64,
    pub tag: String,
    pub height: i32,
    pub latest: bool,
}

impl TaggedDataTagModel {
    pub fn new(id: i64, tag: String, height: i32) -> Self {
        Self {
            db_id: 0,
            id,
            tag,
            height,
            latest: true,
        }
    }
}

/// Tagged Timestamp Model
///
/// 对应 Java: TaggedTimestamp.java
#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
pub struct TaggedTimestampModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub tag: String,
    pub timestamp: i32,
    pub height: i32,
    pub latest: bool,
}

impl TaggedTimestampModel {
    pub fn new(id: i64, account_id: i64, tag: String, timestamp: i32, height: i32) -> Self {
        Self {
            db_id: 0,
            id,
            account_id,
            tag,
            timestamp,
            height,
            latest: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tagged_data_tag_model_new() {
        let tag = TaggedDataTagModel::new(1, "important".to_string(), 1000);
        assert_eq!(tag.id, 1);
        assert_eq!(tag.tag, "important");
        assert_eq!(tag.height, 1000);
        assert!(tag.latest);
    }

    #[test]
    fn test_tagged_timestamp_model_new() {
        let ts = TaggedTimestampModel::new(1, 100, "milestone".to_string(), 12345, 1000);
        assert_eq!(ts.id, 1);
        assert_eq!(ts.account_id, 100);
        assert_eq!(ts.tag, "milestone");
        assert_eq!(ts.timestamp, 12345);
        assert_eq!(ts.height, 1000);
    }
}
