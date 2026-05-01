//! Account-related database models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use blockchain_types::{AccountId, Amount, Height, Result, Timestamp};
use blockchain_types::prelude::*;

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct AccountModel {
    pub db_id: i64,
    pub id: i64,
    pub balance: i64,
    pub unconfirmed_balance: i64,
    pub forged_balance: i64,
    pub active_lessee_id: Option<i64>,
    pub has_control_phasing: bool,
    pub height: i32,
    pub latest: bool,
}

impl AccountModel {
    pub fn to_domain(&self) -> Result<Account> {
        Ok(Account {
            id: self.id as AccountId,
            address: None,
            balance: self.balance as Amount,
            unconfirmed_balance: self.unconfirmed_balance as Amount,
            forged_balance: self.forged_balance as Amount,
            reserved_balance: 0,
            guaranteed_balance: 0,
            assets: std::collections::HashMap::new(),
            properties: std::collections::HashMap::new(),
            lease: self.active_lessee_id.map(|id| AccountLease {
                lessee_id: id as AccountId,
                amount: 0,
                start_height: 0,
                end_height: 0,
            }),
            has_control_phasing: self.has_control_phasing,
            created_at: self.height as Timestamp,
            last_updated: self.height as Timestamp,
            current_height: self.height as Height,
        })
    }

    pub fn from_domain(account: &Account) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: account.id as i64,
            balance: account.balance as i64,
            unconfirmed_balance: account.unconfirmed_balance as i64,
            forged_balance: account.forged_balance as i64,
            active_lessee_id: account.lease.as_ref().map(|l| l.lessee_id as i64),
            has_control_phasing: account.has_control_phasing,
            height: account.current_height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct AccountAssetModel {
    pub db_id: i64,
    pub account_id: i64,
    pub asset_id: i64,
    pub quantity: i64,
    pub unconfirmed_quantity: i64,
    pub height: i32,
    pub latest: bool,
}

impl AccountAssetModel {
    pub fn to_domain(&self) -> Result<AccountAsset> {
        Ok(AccountAsset {
            account_id: self.account_id as AccountId,
            asset_id: self.asset_id as AssetId,
            quantity: self.quantity as Amount,
            last_updated: self.height as Timestamp,
        })
    }

    pub fn from_domain(aa: &AccountAsset) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            account_id: aa.account_id as i64,
            asset_id: aa.asset_id as i64,
            quantity: aa.quantity as i64,
            unconfirmed_quantity: aa.quantity as i64,
            height: aa.last_updated as i32,
            latest: true,
        })
    }
}
