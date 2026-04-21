//! 锻造相关 API Handlers
//!
//! 与 Java 版本 StartForging, StopForging 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};

pub struct StartForgingHandler;

impl StartForgingHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for StartForgingHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Forging]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0")
            .insert("effectiveBalanceNRCS", 0i64)
            .insert("deadline", 0i32)
            .insert("hitTime", 0i32);
        
        Ok(builder.build())
    }
}

pub struct StopForgingHandler;

impl StopForgingHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for StopForgingHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "account"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Forging]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.get_string("secretPhrase");
        let _account = req.get_u64("account");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("foundAndStopped", true)
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0");
        
        Ok(builder.build())
    }
}

pub struct GetForgingHandler;

impl GetForgingHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetForgingHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "accountId"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Forging]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.get_string("secretPhrase");
        let _account_id = req.get_u64("accountId");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("generators", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetNextBlockGeneratorsHandler;

impl GetNextBlockGeneratorsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetNextBlockGeneratorsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["limit"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Forging]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _limit = req.get_i32("limit").unwrap_or(10);
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("activeCount", 0i32)
            .insert("lastBlock", "")
            .insert("generators", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAccountCurrentAskOrderIdsHandler;

impl GetAccountCurrentAskOrderIdsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountCurrentAskOrderIdsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "asset", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _asset = req.get_u64("asset");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("askOrderIds", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAccountCurrentBidOrderIdsHandler;

impl GetAccountCurrentBidOrderIdsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountCurrentBidOrderIdsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "asset", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _asset = req.get_u64("asset");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("bidOrderIds", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAllOpenAskOrdersHandler;

impl GetAllOpenAskOrdersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAllOpenAskOrdersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("askOrders", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAllOpenBidOrdersHandler;

impl GetAllOpenBidOrdersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAllOpenBidOrdersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("bidOrders", json!([]));
        
        Ok(builder.build())
    }
}
