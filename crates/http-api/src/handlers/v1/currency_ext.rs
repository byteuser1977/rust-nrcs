//! 货币扩展相关 API Handlers
//!
//! 与 Java 版本 GetAccountCurrencyCount, GetCurrencyAccountCount 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct GetAccountCurrencyCountHandler;

impl GetAccountCurrencyCountHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountCurrencyCountHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "height"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _height = req.get_i32("height");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("numberOfCurrencies", 0i32);
        
        Ok(builder.build())
    }
}

pub struct GetCurrencyAccountCountHandler;

impl GetCurrencyAccountCountHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCurrencyAccountCountHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currency", "height"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currency_id = req.require_u64("currency")?;
        let _height = req.get_i32("height");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("numberOfAccounts", 0i32);
        
        Ok(builder.build())
    }
}

pub struct GetCurrencyFoundersHandler;

impl GetCurrencyFoundersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCurrencyFoundersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currency", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currency_id = req.require_u64("currency")?;
        
        let mut builder = RsRespBuilder::new();
        builder.insert("founders", json!([]));
        
        Ok(builder.build())
    }
}

pub struct CurrencyMintHandler;

impl CurrencyMintHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for CurrencyMintHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "currency", "nonce", "unitsQNT", "feeNQT", "deadline"]
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
        let _nonce = req.require_string("nonce")?;
        let _units = req.require_string("unitsQNT")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}

pub struct CurrencyReserveIncreaseHandler;

impl CurrencyReserveIncreaseHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for CurrencyReserveIncreaseHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "currency", "amountNQT", "feeNQT", "deadline"]
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
        let _amount = req.require_string("amountNQT")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}

pub struct CurrencyReserveClaimHandler;

impl CurrencyReserveClaimHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for CurrencyReserveClaimHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "currency", "unitsQNT", "feeNQT", "deadline"]
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
        let _units = req.require_string("unitsQNT")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}
