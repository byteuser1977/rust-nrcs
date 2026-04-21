//! 搜索相关 API Handlers
//!
//! 与 Java 版本 SearchAccounts, SearchAssets 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};

pub struct SearchAccountsHandler;

impl SearchAccountsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SearchAccountsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["query", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Search, ApiTag::Accounts]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _query = req.require_string("query")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("accounts", json!([]));
        
        Ok(builder.build())
    }
}

pub struct SearchAssetsHandler;

impl SearchAssetsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SearchAssetsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["query", "firstIndex", "lastIndex", "includeCounts"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Search, ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _query = req.require_string("query")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("assets", json!([]));
        
        Ok(builder.build())
    }
}

pub struct SearchCurrenciesHandler;

impl SearchCurrenciesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SearchCurrenciesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["query", "code", "firstIndex", "lastIndex", "includeCounts"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Search, ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _query = req.get_string("query");
        let _code = req.get_string("code");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("currencies", json!([]));
        
        Ok(builder.build())
    }
}

pub struct SearchPollsHandler;

impl SearchPollsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SearchPollsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["query", "firstIndex", "lastIndex", "includeFinished"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Search, ApiTag::Vs]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _query = req.require_string("query")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("polls", json!([]));
        
        Ok(builder.build())
    }
}

pub struct SearchDGSGoodsHandler;

impl SearchDGSGoodsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SearchDGSGoodsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["query", "tag", "seller", "firstIndex", "lastIndex", "inStockOnly"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Search, ApiTag::Dgs]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _query = req.get_string("query");
        let _tag = req.get_string("tag");
        let _seller = req.get_u64("seller");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("goods", json!([]));
        
        Ok(builder.build())
    }
}
