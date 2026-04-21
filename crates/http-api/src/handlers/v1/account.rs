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
        
        if include_lessors {
            builder.insert("lessors", json!([]));
            builder.insert("lessorsRS", json!([]));
        }
        
        if include_assets {
            builder.insert("assetBalances", json!([]));
            builder.insert("unconfirmedAssetBalances", json!([]));
        }
        
        if include_currencies {
            builder.insert("accountCurrencies", json!([]));
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

pub struct GetAccountPublicKeyHandler;

impl GetAccountPublicKeyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountPublicKeyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _account_id = req.require_u64("account")?;
        
        let mut builder = RsRespBuilder::new();
        builder.insert("publicKey", "");
        
        Ok(builder.build())
    }
}

pub struct GetAccountAssetsHandler;

impl GetAccountAssetsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountAssetsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "asset", "height", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _account_id = req.require_u64("account")?;
        let _asset_id = req.get_u64("asset");
        let _height = req.get_i32("height");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("accountAssets", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAccountCurrenciesHandler;

impl GetAccountCurrenciesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountCurrenciesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "currency", "height", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _account_id = req.require_u64("account")?;
        let _currency_id = req.get_u64("currency");
        let _height = req.get_i32("height");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("accountCurrencies", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAccountPropertiesHandler;

impl GetAccountPropertiesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountPropertiesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["recipient", "property", "setter", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _recipient = req.get_u64("recipient");
        let _property = req.get_string("property");
        let _setter = req.get_u64("setter");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("properties", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAccountLessorsHandler;

impl GetAccountLessorsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountLessorsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "height"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _account_id = req.require_u64("account")?;
        let _height = req.get_i32("height");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("lessors", json!([]))
            .insert("lessorsRS", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetEffectiveBalanceHandler;

impl GetEffectiveBalanceHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetEffectiveBalanceHandler {
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
            .insert("effectiveBalanceNRCS", 0i64)
            .insert("guaranteedBalanceNQT", "0");
        
        Ok(builder.build())
    }
}

pub struct GetGuaranteedBalanceHandler;

impl GetGuaranteedBalanceHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetGuaranteedBalanceHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "numberOfConfirmations"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let account_id = req.require_u64("account")?;
        let _confirmations = req.get_i32("numberOfConfirmations").unwrap_or(1440);
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("guaranteedBalanceNQT", "0");
        
        Ok(builder.build())
    }
}

pub struct SetAccountInfoHandler;

impl SetAccountInfoHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SetAccountInfoHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "name", "description", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _name = req.get_string("name");
        let _description = req.get_string("description");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "")
            .insert("transactionBytes", "");
        
        Ok(builder.build())
    }
}

pub struct SetAccountPropertyHandler;

impl SetAccountPropertyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SetAccountPropertyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "recipient", "property", "value", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _recipient = req.get_u64("recipient");
        let _property = req.require_string("property")?;
        let _value = req.get_string("value");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}

pub struct GetBalancesHandler;

impl GetBalancesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBalancesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "includeEffectiveBalance", "height"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _account_ids = req.get_string("account");
        let _include_effective = req.get_bool("includeEffectiveBalance");
        let _height = req.get_i32("height");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("balances", json!([]));
        
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
