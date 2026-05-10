//! Asset extension models (transfer, delete, dividend, etc.)

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
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

impl AssetTransferModel {
    pub fn new(id: i64, asset_id: i64, sender_id: i64, recipient_id: i64, quantity: i64, timestamp: i32, height: i32) -> Self {
        Self {
            db_id: 0,
            id,
            asset_id,
            sender_id,
            recipient_id,
            quantity,
            timestamp,
            height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
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
#[sqlx(rename_all = "snake_case")]
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

/// Asset History Model
///
/// 对应 Java: AssetHistory.java, BaseAssetHistory.java
/// 记录资产转移历史
#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
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

impl AssetHistoryModel {
    pub fn new(id: i64, asset_id: i64, account_id: i64, quantity: i64, timestamp: i32, height: i32) -> Self {
        Self {
            db_id: 0,
            id,
            full_hash: vec![],
            asset_id,
            account_id,
            quantity,
            timestamp,
            chain_id: 0,
            height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
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

/// Asset Property Model
///
/// 对应 Java: AssetProperty.java, BaseAssetProperty.java
/// 存储资产的自定义属性
#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
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

impl AssetPropertyModel {
    pub fn new(id: i64, asset_id: i64, setter_id: i64, property: String, value: Option<String>, height: i32) -> Self {
        Self {
            db_id: 0,
            id,
            asset_id,
            setter_id,
            property,
            value,
            height,
            latest: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_transfer_model_new() {
        let transfer = AssetTransferModel::new(1, 100, 200, 300, 1000, 12345, 1000);
        assert_eq!(transfer.id, 1);
        assert_eq!(transfer.asset_id, 100);
        assert_eq!(transfer.sender_id, 200);
        assert_eq!(transfer.recipient_id, 300);
    }

    #[test]
    fn test_asset_history_model_new() {
        let history = AssetHistoryModel::new(1, 100, 200, 1000, 12345, 1000);
        assert_eq!(history.id, 1);
        assert_eq!(history.asset_id, 100);
        assert_eq!(history.account_id, 200);
    }

    #[test]
    fn test_asset_property_model_new() {
        let prop = AssetPropertyModel::new(1, 100, 200, "color".to_string(), Some("red".to_string()), 1000);
        assert_eq!(prop.id, 1);
        assert_eq!(prop.asset_id, 100);
        assert_eq!(prop.setter_id, 200);
        assert_eq!(prop.property, "color");
        assert_eq!(prop.value, Some("red".to_string()));
    }
}
