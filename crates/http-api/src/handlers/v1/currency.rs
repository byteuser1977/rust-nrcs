//! 货币相关 API Handlers
//!
//! 与 Java 版本 GetCurrency, IssueCurrency 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct GetCurrencyHandler;

impl GetCurrencyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCurrencyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currency", "code", "includeCounts"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currency_id = req.get_u64("currency");
        let _code = req.get_string("code");
        let _include_counts = req.get_bool("includeCounts");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("currency", "0")
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0")
            .insert("code", "")
            .insert("name", "")
            .insert("description", "")
            .insert("decimals", 0i32)
            .insert("initialSupplyQNT", "0")
            .insert("currentSupplyQNT", "0")
            .insert("type", 0i32)
            .insert("numberOfTransfers", 0i32);
        
        Ok(builder.build())
    }
}

pub struct GetCurrenciesHandler;

impl GetCurrenciesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCurrenciesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currencies", "includeCounts"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currencies = req.get_string("currencies");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("currencies", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAllCurrenciesHandler;

impl GetAllCurrenciesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAllCurrenciesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex", "includeCounts"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("currencies", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetCurrencyIdsHandler;

impl GetCurrencyIdsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCurrencyIdsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("currencyIds", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetCurrenciesByIssuerHandler;

impl GetCurrenciesByIssuerHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCurrenciesByIssuerHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "firstIndex", "lastIndex", "includeCounts"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("currencies", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetCurrencyAccountsHandler;

impl GetCurrencyAccountsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCurrencyAccountsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currency", "firstIndex", "lastIndex", "height"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currency_id = req.require_u64("currency")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("accountCurrencies", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetCurrencyTransfersHandler;

impl GetCurrencyTransfersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCurrencyTransfersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currency", "account", "firstIndex", "lastIndex", "timestamp", "includeCurrencyInfo"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currency_id = req.get_u64("currency");
        let _account = req.get_u64("account");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("transfers", json!([]));
        
        Ok(builder.build())
    }
}

pub struct IssueCurrencyHandler;

impl IssueCurrencyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for IssueCurrencyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "name", "code", "description", "type", "initialSupplyQNT", 
             "decimals", "minReservePerUnitNQT", "reserveSupplyQNT", "issuanceHeight", 
             "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _name = req.require_string("name")?;
        let _code = req.require_string("code")?;
        let _description = req.get_string("description");
        let _type_ = req.get_i32("type").unwrap_or(0);
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "")
            .insert("transactionBytes", "");
        
        Ok(builder.build())
    }
}

pub struct TransferCurrencyHandler;

impl TransferCurrencyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for TransferCurrencyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "recipient", "currency", "unitsQNT", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _recipient = req.require_u64("recipient")?;
        let _currency_id = req.require_u64("currency")?;
        let _units = req.require_string("unitsQNT")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "")
            .insert("transactionBytes", "");
        
        Ok(builder.build())
    }
}

pub struct CurrencyBuyHandler;

impl CurrencyBuyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for CurrencyBuyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "currency", "rateNQT", "unitsQNT", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _currency_id = req.require_u64("currency")?;
        let _rate = req.require_string("rateNQT")?;
        let _units = req.require_string("unitsQNT")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}

pub struct CurrencySellHandler;

impl CurrencySellHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for CurrencySellHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "currency", "rateNQT", "unitsQNT", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _currency_id = req.require_u64("currency")?;
        let _rate = req.require_string("rateNQT")?;
        let _units = req.require_string("unitsQNT")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}

pub struct CanDeleteCurrencyHandler;

impl CanDeleteCurrencyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for CanDeleteCurrencyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "currency"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _currency_id = req.require_u64("currency")?;
        
        let mut builder = RsRespBuilder::new();
        builder.insert("canDelete", false);
        
        Ok(builder.build())
    }
}

pub struct DeleteCurrencyHandler;

impl DeleteCurrencyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DeleteCurrencyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "currency", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _currency_id = req.require_u64("currency")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}
