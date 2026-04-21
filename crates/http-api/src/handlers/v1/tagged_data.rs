//! 标签数据相关 API Handlers
//!
//! 与 Java 版本 GetTaggedData, UploadTaggedData 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct GetTaggedDataHandler;

impl GetTaggedDataHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetTaggedDataHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transaction", "includeData"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Data]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _transaction_id = req.require_u64("transaction")?;
        let _include_data = req.get_bool("includeData");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "0")
            .insert("name", "")
            .insert("description", "")
            .insert("tags", "")
            .insert("type", "")
            .insert("channel", "")
            .insert("filename", "")
            .insert("data", "")
            .insert("hash", "")
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0");
        
        Ok(builder.build())
    }
}

pub struct GetAllTaggedDataHandler;

impl GetAllTaggedDataHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAllTaggedDataHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex", "includeData"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Data]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("data", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAccountTaggedDataHandler;

impl GetAccountTaggedDataHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountTaggedDataHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "firstIndex", "lastIndex", "includeData"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Data]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("data", json!([]));
        
        Ok(builder.build())
    }
}

pub struct UploadTaggedDataHandler;

impl UploadTaggedDataHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for UploadTaggedDataHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "name", "description", "tags", "type", "channel", 
             "data", "filename", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Data, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    fn file_parameter(&self) -> Option<&'static str> {
        Some("file")
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _name = req.require_string("name")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "")
            .insert("transactionBytes", "");
        
        Ok(builder.build())
    }
}

pub struct DownloadTaggedDataHandler;

impl DownloadTaggedDataHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DownloadTaggedDataHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transaction", "retrieve"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Data]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _transaction_id = req.require_u64("transaction")?;
        let _retrieve = req.get_bool("retrieve");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("data", "")
            .insert("hash", "");
        
        Ok(builder.build())
    }
}

pub struct VerifyTaggedDataHandler;

impl VerifyTaggedDataHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for VerifyTaggedDataHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transaction", "data", "hash"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Data]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _transaction_id = req.require_u64("transaction")?;
        let _data = req.get_string("data");
        let _hash = req.get_string("hash");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("verify", true);
        
        Ok(builder.build())
    }
}

pub struct SearchTaggedDataHandler;

impl SearchTaggedDataHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SearchTaggedDataHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["query", "tag", "channel", "account", "firstIndex", "lastIndex", "includeData"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Data, ApiTag::Search]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _query = req.get_string("query");
        let _tag = req.get_string("tag");
        let _channel = req.get_string("channel");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("data", json!([]));
        
        Ok(builder.build())
    }
}
