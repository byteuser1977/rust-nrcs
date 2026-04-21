//! 账户相关 DTO
//!
//! 与 Java APIGetAccount 等完全对齐

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiGetAccount {
    #[serde(rename = "balanceNQT", skip_serializing_if = "Option::is_none")]
    pub balance_nqt: Option<String>,
    
    #[serde(rename = "unconfirmedBalanceNQT", skip_serializing_if = "Option::is_none")]
    pub unconfirmed_balance_nqt: Option<String>,
    
    #[serde(rename = "forgedBalanceNQT", skip_serializing_if = "Option::is_none")]
    pub forged_balance_nqt: Option<String>,
    
    #[serde(rename = "effectiveBalanceNRCS", skip_serializing_if = "Option::is_none")]
    pub effective_balance_nrcs: Option<i64>,
    
    #[serde(rename = "guaranteedBalanceNQT", skip_serializing_if = "Option::is_none")]
    pub guaranteed_balance_nqt: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    
    #[serde(rename = "accountRS", skip_serializing_if = "Option::is_none")]
    pub account_rs: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publicKey: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currentLessee: Option<String>,
    
    #[serde(rename = "currentLesseeRS", skip_serializing_if = "Option::is_none")]
    pub current_lessee_rs: Option<String>,
    
    #[serde(rename = "currentLeasingHeightFrom", skip_serializing_if = "Option::is_none")]
    pub current_leasing_height_from: Option<i32>,
    
    #[serde(rename = "currentLeasingHeightTo", skip_serializing_if = "Option::is_none")]
    pub current_leasing_height_to: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nextLessee: Option<String>,
    
    #[serde(rename = "nextLesseeRS", skip_serializing_if = "Option::is_none")]
    pub next_lessee_rs: Option<String>,
    
    #[serde(rename = "nextLeasingHeightFrom", skip_serializing_if = "Option::is_none")]
    pub next_leasing_height_from: Option<i32>,
    
    #[serde(rename = "nextLeasingHeightTo", skip_serializing_if = "Option::is_none")]
    pub next_leasing_height_to: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accountControls: Option<Vec<String>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lessors: Option<Vec<String>>,
    
    #[serde(rename = "lessorsRS", skip_serializing_if = "Option::is_none")]
    pub lessors_rs: Option<Vec<String>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lessorsInfo: Option<Vec<ApiAccountLease>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assetBalances: Option<Vec<ApiAssetBalance>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unconfirmedAssetBalances: Option<Vec<ApiAssetBalance>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accountCurrencies: Option<Vec<ApiAccountCurrency>>,
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
    
    #[serde(rename = "unconfirmedBalanceQNT", skip_serializing_if = "Option::is_none")]
    pub unconfirmed_balance_qnt: Option<String>,
    
    #[serde(rename = "balanceQNT", skip_serializing_if = "Option::is_none")]
    pub balance_qnt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiGetBalance {
    #[serde(rename = "balanceNQT", skip_serializing_if = "Option::is_none")]
    pub balance_nqt: Option<String>,
    
    #[serde(rename = "unconfirmedBalanceNQT", skip_serializing_if = "Option::is_none")]
    pub unconfirmed_balance_nqt: Option<String>,
    
    #[serde(rename = "effectiveBalanceNRCS", skip_serializing_if = "Option::is_none")]
    pub effective_balance_nrcs: Option<i64>,
    
    #[serde(rename = "guaranteedBalanceNQT", skip_serializing_if = "Option::is_none")]
    pub guaranteed_balance_nqt: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    
    #[serde(rename = "accountRS", skip_serializing_if = "Option::is_none")]
    pub account_rs: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiGetAccountId {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    
    #[serde(rename = "accountRS", skip_serializing_if = "Option::is_none")]
    pub account_rs: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publicKey: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiGetAccountPublicKey {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publicKey: Option<String>,
}
