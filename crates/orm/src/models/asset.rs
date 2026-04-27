//! Asset-related database models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use blockchain_types::{AccountId, Amount, AssetId, Result, Timestamp};
use blockchain_types::prelude::*;

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct AssetModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub quantity: i64,
    pub decimals: i16,
    pub has_control_phasing: bool,
    pub initial_quantity: i64,
    pub height: i32,
    pub latest: bool,
}
impl AssetModel {
    pub fn to_domain(&self) -> Result<Asset> {
        Ok(Asset {
            id: self.id as AssetId,
            owner_id: self.account_id as AccountId,
            name: self.name.clone(),
            description: self.description.clone().unwrap_or_default(),
            quantity: self.quantity as Amount,
            decimals: self.decimals as u8,
            mintable: false,
            transferable: true,
            data: vec![],
            created_at: self.height as Timestamp,
            last_updated: self.height as Timestamp,
            deleted: false,
        })
    }

    pub fn from_domain(asset: &Asset) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: asset.id as i64,
            account_id: asset.owner_id as i64,
            name: asset.name.clone(),
            description: Some(asset.description.clone()),
            quantity: asset.quantity as i64,
            decimals: asset.decimals as i16,
            has_control_phasing: false,
            initial_quantity: asset.quantity as i64,
            height: asset.created_at as i32,
            latest: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_model() {
        let model = AssetModel {
            db_id: 0,
            id: 1,
            account_id: 100,
            name: "Test Asset".to_string(),
            description: Some("Test Description".to_string()),
            quantity: 1000000,
            decimals: 4,
            has_control_phasing: false,
            initial_quantity: 1000000,
            height: 1000,
            latest: true,
        };
        
        assert_eq!(model.id, 1);
        assert_eq!(model.name, "Test Asset");
    }
}
