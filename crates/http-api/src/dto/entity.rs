//! 实体相关 DTO
//!
//! 与 Java 版本 entity 类完全对齐

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAssetBalance {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    
    #[serde(rename = "balanceQNT", skip_serializing_if = "Option::is_none")]
    pub balance_qnt: Option<String>,
    
    #[serde(rename = "unconfirmedBalanceQNT", skip_serializing_if = "Option::is_none")]
    pub unconfirmed_balance_qnt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAccountCurrency {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    
    #[serde(rename = "balanceQNT", skip_serializing_if = "Option::is_none")]
    pub balance_qnt: Option<String>,
    
    #[serde(rename = "unconfirmedBalanceQNT", skip_serializing_if = "Option::is_none")]
    pub unconfirmed_balance_qnt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAccountLease {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lessor: Option<String>,
    
    #[serde(rename = "lessorRS", skip_serializing_if = "Option::is_none")]
    pub lessor_rs: Option<String>,
    
    #[serde(rename = "effectiveBalanceNQT", skip_serializing_if = "Option::is_none")]
    pub effective_balance_nqt: Option<String>,
    
    #[serde(rename = "guaranteedBalanceNQT", skip_serializing_if = "Option::is_none")]
    pub guaranteed_balance_nqt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAlias {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliasName: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    
    #[serde(rename = "accountRS", skip_serializing_if = "Option::is_none")]
    pub account_rs: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliasURI: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priceNQT: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buyer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAsset {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    
    #[serde(rename = "accountRS", skip_serializing_if = "Option::is_none")]
    pub account_rs: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantityQNT: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decimals: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numberOfTrades: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numberOfTransfers: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numberOfAccounts: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCurrency {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    
    #[serde(rename = "accountRS", skip_serializing_if = "Option::is_none")]
    pub account_rs: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decimals: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialSupplyQNT: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currentSupplyQNT: Option<String>,
    
    #[serde(rename = "type")]
    pub currency_type: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiPeer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub announcedAddress: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shareAddress: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloadedVolume: Option<i64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uploadedVolume: Option<i64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blacklisted: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastUpdated: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiPoll {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    
    #[serde(rename = "accountRS", skip_serializing_if = "Option::is_none")]
    pub account_rs: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finishHeight: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTrade {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trade: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    
    #[serde(rename = "accountRS", skip_serializing_if = "Option::is_none")]
    pub account_rs: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantityQNT: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priceNQT: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
}
