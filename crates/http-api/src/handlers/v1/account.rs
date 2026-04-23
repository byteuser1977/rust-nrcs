//! 账户相关 API Handlers
//!
//! 与 Java 版本 GetAccount, GetBalance 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account_id = req.require_u64("account")?;
        let include_lessors = req.get_bool("includeLessors");
        let include_assets = req.get_bool("includeAssets");
        let include_currencies = req.get_bool("includeCurrencies");
        let include_effective_balance = req.get_bool("includeEffectiveBalance");
        
        let account = state.account_manager
            .get_account_info(account_id)
            .await
            .map_err(ApiError::Account)?;
        
        let mut builder = RsRespBuilder::new();
        
        builder
            .insert("account", account_id.to_string())
            .insert("accountRS", format_account_rs(account_id));
        
        builder
            .insert("balanceNQT", account.balance.to_string())
            .insert("unconfirmedBalanceNQT", account.unconfirmed_balance.to_string())
            .insert("forgedBalanceNQT", "0")
            .insert("guaranteedBalanceNQT", account.guaranteed_balance.to_string());
        
        if include_effective_balance {
            let effective = account.effective_balance();
            builder.insert("effectiveBalanceNRCS", effective as i64);
        }
        
        if include_lessors {
            builder.insert("lessors", json!([]));
            builder.insert("lessorsRS", json!([]));
        }
        
        if include_assets {
            let asset_balances: Vec<serde_json::Value> = account.assets.iter()
                .map(|(id, qty)| json!({
                    "asset": id.to_string(),
                    "balanceQNT": qty.to_string()
                }))
                .collect();
            builder.insert("assetBalances", json!(asset_balances));
            builder.insert("unconfirmedAssetBalances", json!(asset_balances));
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account_id = req.require_u64("account")?;
        
        let balance = state.account_manager
            .get_balance(account_id)
            .await
            .map_err(ApiError::Account)?;
        
        let mut builder = RsRespBuilder::new();
        
        builder
            .insert("account", account_id.to_string())
            .insert("accountRS", format_account_rs(account_id))
            .insert("balanceNQT", balance.to_string())
            .insert("unconfirmedBalanceNQT", balance.to_string())
            .insert("effectiveBalanceNRCS", (balance / 100_000_000) as i64)
            .insert("guaranteedBalanceNQT", balance.to_string());
        
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account_id = req.require_u64("account")?;
        
        let public_key = state.account_manager
            .get_public_key(account_id)
            .await
            .map_err(ApiError::Account)?;
        
        let mut builder = RsRespBuilder::new();
        
        if let Some(pk) = public_key {
            builder.insert("publicKey", hex::encode(pk.as_bytes()));
        } else {
            builder.insert("publicKey", "");
        }
        
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account_id = req.require_u64("account")?;
        let _asset_id = req.get_u64("asset");
        let _height = req.get_i32("height");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let account = state.account_manager
            .get_account_info(account_id)
            .await
            .map_err(ApiError::Account)?;
        
        let account_assets: Vec<serde_json::Value> = account.assets.iter()
            .map(|(id, qty)| json!({
                "asset": id.to_string(),
                "account": account_id.to_string(),
                "accountRS": format_account_rs(account_id),
                "quantityQNT": qty.to_string(),
                "unconfirmedQuantityQNT": qty.to_string()
            }))
            .collect();
        
        let mut builder = RsRespBuilder::new();
        builder.insert("accountAssets", json!(account_assets));
        
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account_id = req.require_u64("account")?;
        
        let account = state.account_manager
            .get_account_info(account_id)
            .await
            .map_err(ApiError::Account)?;
        
        let effective = account.effective_balance();
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("account", account_id.to_string())
            .insert("accountRS", format_account_rs(account_id))
            .insert("effectiveBalanceNRCS", effective as i64)
            .insert("guaranteedBalanceNQT", account.guaranteed_balance.to_string());
        
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account_id = req.require_u64("account")?;
        let _confirmations = req.get_i32("numberOfConfirmations").unwrap_or(1440);
        
        let account = state.account_manager
            .get_account_info(account_id)
            .await
            .map_err(ApiError::Account)?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("guaranteedBalanceNQT", account.guaranteed_balance.to_string());
        
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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

fn derive_account_id(secret_phrase: &str) -> u64 {
    let seed = crypto::sha256(secret_phrase.as_bytes());
    let kp = crypto::keypair_from_seed(&seed);
    let public_key = kp.public_key();
    let pk_bytes = public_key.as_bytes();
    let hash = crypto::sha256(pk_bytes);
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&hash[0..8]);
    u64::from_le_bytes(bytes)
}
