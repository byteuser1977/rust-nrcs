//! Alias-related domain types

use serde::{Deserialize, Serialize};

use crate::{AccountId, Height, Timestamp};

pub type AliasId = u64;
pub type AliasOfferId = u64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alias {
    pub id: AliasId,
    pub owner_id: AccountId,
    pub name: String,
    pub uri: String,
    pub timestamp: Timestamp,
    pub height: Height,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AliasOffer {
    pub id: AliasOfferId,
    pub alias_id: AliasId,
    pub price: crate::Amount,
    pub buyer_id: Option<AccountId>,
    pub height: Height,
}
