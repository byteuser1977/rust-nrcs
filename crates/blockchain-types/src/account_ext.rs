//! Account extension domain types

use serde::{Deserialize, Serialize};

use crate::{AccountId, Height};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountInfo {
    pub account_id: AccountId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub height: Height,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountProperty {
    pub id: u64,
    pub recipient_id: AccountId,
    pub setter_id: AccountId,
    pub property: String,
    pub value: Option<String>,
    pub height: Height,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountPublicKey {
    pub account_id: AccountId,
    pub public_key: [u8; 32],
    pub height: Height,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountControlPhasing {
    pub account_id: AccountId,
    pub voting_model: u8,
    pub quorum: i64,
    pub min_balance: i64,
    pub min_balance_model: u8,
    pub holding_id: Option<u64>,
    pub whitelist: Vec<AccountId>,
    pub height: Height,
}
