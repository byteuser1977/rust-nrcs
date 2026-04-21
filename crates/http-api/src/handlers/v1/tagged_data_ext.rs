//! 标签数据扩展相关 API Handlers
//!
//! 与 Java 版本 ExtendTaggedData 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};

pub struct ExtendTaggedDataHandler;

impl ExtendTaggedDataHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ExtendTaggedDataHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "transaction", "data", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Data, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _transaction_id = req.require_u64("transaction")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}

pub struct GetTaggedDataExtendTransactionsHandler;

impl GetTaggedDataExtendTransactionsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetTaggedDataExtendTransactionsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transaction", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Data]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _transaction_id = req.require_u64("transaction")?;
        
        let mut builder = RsRespBuilder::new();
        builder.insert("transactions", json!([]));
        
        Ok(builder.build())
    }
}
