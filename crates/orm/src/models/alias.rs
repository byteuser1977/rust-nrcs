//! Alias-related database models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use blockchain_types::{AccountId, AliasId, AliasOfferId, Amount, Height, Result, Timestamp};
use blockchain_types::prelude::*;

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
pub struct AliasModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub alias_name: String,
    pub alias_name_lower: String,
    pub alias_uri: String,
    pub timestamp: i32,
    pub height: i32,
    pub latest: bool,
}

impl AliasModel {
    pub fn to_domain(&self) -> Result<Alias> {
        Ok(Alias {
            id: self.id as AliasId,
            owner_id: self.account_id as AccountId,
            name: self.alias_name.clone(),
            uri: self.alias_uri.clone(),
            timestamp: self.timestamp as Timestamp,
            height: self.height as Height,
        })
    }

    pub fn from_domain(alias: &Alias) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: alias.id as i64,
            account_id: alias.owner_id as i64,
            alias_name: alias.name.clone(),
            alias_name_lower: alias.name.to_lowercase(),
            alias_uri: alias.uri.clone(),
            timestamp: alias.timestamp as i32,
            height: alias.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "snake_case")]
pub struct AliasOfferModel {
    pub db_id: i64,
    pub id: i64,
    pub price: i64,
    pub buyer_id: Option<i64>,
    pub height: i32,
    pub latest: bool,
}

impl AliasOfferModel {
    pub fn to_domain(&self) -> Result<AliasOffer> {
        Ok(AliasOffer {
            id: self.id as AliasOfferId,
            alias_id: 0,
            price: self.price as Amount,
            buyer_id: self.buyer_id.map(|id| id as AccountId),
            height: self.height as Height,
        })
    }

    pub fn from_domain(offer: &AliasOffer) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: offer.id as i64,
            price: offer.price as i64,
            buyer_id: offer.buyer_id.map(|id| id as i64),
            height: offer.height as i32,
            latest: true,
        })
    }
}
