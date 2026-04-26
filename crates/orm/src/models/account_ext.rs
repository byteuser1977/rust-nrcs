//! Account extension models (info, property, control phasing, etc.)

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use blockchain_types::{AccountId, Height, Result};
use blockchain_types::prelude::*;

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct AccountInfoModel {
    pub db_id: i64,
    pub account_id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub height: i32,
    pub latest: bool,
}

impl AccountInfoModel {
    pub fn to_domain(&self) -> Result<AccountInfo> {
        Ok(AccountInfo {
            account_id: self.account_id as AccountId,
            name: self.name.clone(),
            description: self.description.clone(),
            height: self.height as Height,
        })
    }

    pub fn from_domain(info: &AccountInfo) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            account_id: info.account_id as i64,
            name: info.name.clone(),
            description: info.description.clone(),
            height: info.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct AccountLeaseModel {
    pub db_id: i64,
    pub lessor_id: i64,
    pub current_leasing_height_from: Option<i32>,
    pub current_leasing_height_to: Option<i32>,
    pub current_lessee_id: Option<i64>,
    pub next_leasing_height_from: Option<i32>,
    pub next_leasing_height_to: Option<i32>,
    pub next_lessee_id: Option<i64>,
    pub height: i32,
    pub latest: bool,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct PublicKeyModel {
    pub db_id: i64,
    pub account_id: i64,
    pub public_key: Option<Vec<u8>>,
    pub height: i32,
    pub latest: bool,
}

impl PublicKeyModel {
    pub fn to_domain(&self) -> Result<AccountPublicKey> {
        Ok(AccountPublicKey {
            account_id: self.account_id as AccountId,
            public_key: self
                .public_key
                .clone()
                .and_then(|pk| pk.as_slice().try_into().ok())
                .unwrap_or([0u8; 32]),
            height: self.height as Height,
        })
    }

    pub fn from_domain(pk: &AccountPublicKey) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            account_id: pk.account_id as i64,
            public_key: Some(pk.public_key.to_vec()),
            height: pk.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct AccountControlPhasingModel {
    pub db_id: i64,
    pub account_id: i64,
    pub whitelist: Option<String>,
    pub voting_model: i16,
    pub quorum: Option<i64>,
    pub min_balance: Option<i64>,
    pub holding_id: Option<i64>,
    pub min_balance_model: Option<i16>,
    pub max_fees: Option<i64>,
    pub min_duration: Option<i16>,
    pub max_duration: Option<i16>,
    pub height: i32,
    pub latest: bool,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct AccountGuaranteedBalanceModel {
    pub db_id: i64,
    pub account_id: i64,
    pub additions: i64,
    pub height: i32,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct AccountLedgerModel {
    pub db_id: i64,
    pub account_id: i64,
    pub event_type: i16,
    pub event_id: i64,
    pub holding_type: i16,
    pub holding_id: Option<i64>,
    pub change: i64,
    pub balance: i64,
    pub block_id: i64,
    pub height: i32,
    pub timestamp: i32,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct AccountPropertyModel {
    pub db_id: i64,
    pub id: i64,
    pub recipient_id: i64,
    pub setter_id: Option<i64>,
    pub property: String,
    pub value: Option<String>,
    pub height: i32,
    pub latest: bool,
}

impl AccountPropertyModel {
    pub fn to_domain(&self) -> Result<AccountProperty> {
        Ok(AccountProperty {
            id: self.id as u64,
            recipient_id: self.recipient_id as AccountId,
            setter_id: self.setter_id.unwrap_or(0) as AccountId,
            property: self.property.clone(),
            value: self.value.clone(),
            height: self.height as Height,
        })
    }

    pub fn from_domain(prop: &AccountProperty) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: prop.id as i64,
            recipient_id: prop.recipient_id as i64,
            setter_id: Some(prop.setter_id as i64),
            property: prop.property.clone(),
            value: prop.value.clone(),
            height: prop.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct BalanceModel {
    pub db_id: i64,
    pub account_id: i64,
    pub balance: i64,
    pub unconfirmed_balance: i64,
    pub height: i32,
    pub latest: bool,
}
