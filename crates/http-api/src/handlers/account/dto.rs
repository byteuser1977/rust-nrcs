//! Account API DTOs
//!
//! 账户 API 的数据传输对象

use serde::{Deserialize, Serialize};

/// 获取账户请求
#[derive(Debug, Deserialize)]
pub struct GetAccountRequest {
    pub account: String,
}

/// 获取账户响应
#[derive(Debug, Serialize)]
pub struct GetAccountResponse {
    pub account: String,
    #[serde(rename = "accountRS")]
    pub account_rs: String,
    #[serde(rename = "balanceNQT")]
    pub balance_nqt: String,
    #[serde(rename = "unconfirmedBalanceNQT")]
    pub unconfirmed_balance_nqt: String,
    #[serde(rename = "forgedBalanceNQT")]
    pub forged_balance_nqt: String,
    #[serde(rename = "guaranteedBalanceNQT")]
    pub guaranteed_balance_nqt: String,
    #[serde(rename = "publicKey")]
    pub public_key: Option<String>,
    #[serde(rename = "requestProcessingTime")]
    pub request_processing_time: u32,
}

/// 获取余额请求
#[derive(Debug, Deserialize)]
pub struct GetBalanceRequest {
    pub account: String,
}

/// 获取余额响应
#[derive(Debug, Serialize)]
pub struct GetBalanceResponse {
    #[serde(rename = "balanceNQT")]
    pub balance_nqt: String,
    #[serde(rename = "unconfirmedBalanceNQT")]
    pub unconfirmed_balance_nqt: String,
    #[serde(rename = "forgedBalanceNQT")]
    pub forged_balance_nqt: String,
    #[serde(rename = "guaranteedBalanceNQT")]
    pub guaranteed_balance_nqt: String,
    #[serde(rename = "requestProcessingTime")]
    pub request_processing_time: u32,
}
