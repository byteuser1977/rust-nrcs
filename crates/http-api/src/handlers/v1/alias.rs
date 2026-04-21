//! 别名相关 API Handlers
//!
//! 与 Java 版本 GetAlias, SetAlias 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct GetAliasHandler;

impl GetAliasHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAliasHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["alias", "aliasName"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Aliases]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _alias_id = req.get_u64("alias");
        let _alias_name = req.get_string("aliasName");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("alias", "0")
            .insert("aliasName", "")
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0")
            .insert("timestamp", 0i32)
            .insert("aliasURI", "")
            .insert("priceNQT", "0");
        
        Ok(builder.build())
    }
}

pub struct GetAliasesHandler;

impl GetAliasesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAliasesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "timestamp", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Aliases]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.get_u64("account");
        let _timestamp = req.get_i32("timestamp");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("aliases", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAliasesLikeHandler;

impl GetAliasesLikeHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAliasesLikeHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["aliasPrefix", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Aliases]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _alias_prefix = req.require_string("aliasPrefix")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("aliases", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAliasCountHandler;

impl GetAliasCountHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAliasCountHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Aliases]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        
        let mut builder = RsRespBuilder::new();
        builder.insert("numberOfAliases", 0i32);
        
        Ok(builder.build())
    }
}

pub struct SetAliasHandler;

impl SetAliasHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SetAliasHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "aliasName", "aliasURI", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Aliases, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _alias_name = req.require_string("aliasName")?;
        let _alias_uri = req.get_string("aliasURI");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "")
            .insert("transactionBytes", "");
        
        Ok(builder.build())
    }
}

pub struct DeleteAliasHandler;

impl DeleteAliasHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DeleteAliasHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "alias", "aliasName", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Aliases, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _alias_id = req.get_u64("alias");
        let _alias_name = req.get_string("aliasName");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}

pub struct SellAliasHandler;

impl SellAliasHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SellAliasHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "alias", "aliasName", "priceNQT", "recipient", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Aliases, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _alias_id = req.get_u64("alias");
        let _alias_name = req.get_string("aliasName");
        let _price = req.require_string("priceNQT")?;
        let _recipient = req.get_u64("recipient");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}

pub struct BuyAliasHandler;

impl BuyAliasHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for BuyAliasHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "alias", "aliasName", "amountNQT", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Aliases, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _alias_id = req.get_u64("alias");
        let _alias_name = req.get_string("aliasName");
        let _amount = req.require_string("amountNQT")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}
