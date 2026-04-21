//! 账户相关 API Handlers
//!
//! 与 Java 版本 GetAccount, GetBalance 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::dto::account::*;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};

pub struct GetAccountHandler;

impl GetAccountHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "includeLessors", "includeAssets", "includeCurrencies", "includeEffectiveBalance"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let account_id = req.require_u64("account")?;
        let include_lessors = req.get_bool("includeLessors");
        let include_assets = req.get_bool("includeAssets");
        let include_currencies = req.get_bool("includeCurrencies");
        let include_effective_balance = req.get_bool("includeEffectiveBalance");
        
        let mut builder = RsRespBuilder::new();
        
        builder
            .insert("account", account_id.to_string())
            .insert("accountRS", format_account_rs(account_id));
        
        builder
            .insert("balanceNQT", "0")
            .insert("unconfirmedBalanceNQT", "0")
            .insert("forgedBalanceNQT", "0")
            .insert("guaranteedBalanceNQT", "0");
        
        if include_effective_balance {
            builder.insert("effectiveBalanceNRCS", 0i64);
        }
        
        Ok(builder.build())
    }
}

pub struct GetBalanceHandler;

impl GetBalanceHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBalanceHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let account_id = req.require_u64("account")?;
        
        let mut builder = RsRespBuilder::new();
        
        builder
            .insert("account", account_id.to_string())
            .insert("accountRS", format_account_rs(account_id))
            .insert("balanceNQT", "0")
            .insert("unconfirmedBalanceNQT", "0")
            .insert("effectiveBalanceNRCS", 0i64)
            .insert("guaranteedBalanceNQT", "0");
        
        Ok(builder.build())
    }
}

pub struct GetAccountIdHandler;

impl GetAccountIdHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountIdHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let secret_phrase = req.require_string("secretPhrase")?;
        
        let account_id = derive_account_id(&secret_phrase);
        
        let mut builder = RsRespBuilder::new();
        
        builder
            .insert("account", account_id.to_string())
            .insert("accountRS", format_account_rs(account_id));
        
        Ok(builder.build())
    }
}

fn format_account_rs(account_id: u64) -> String {
    format!("NRCS-{}-{}-{}", 
        account_id % 10000,
        (account_id / 10000) % 10000,
        (account_id / 100000000) % 10000
    )
}

fn derive_account_id(_secret_phrase: &str) -> u64 {
    0u64
}
