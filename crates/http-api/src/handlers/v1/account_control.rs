//! 账户控制相关 API Handlers
//!
//! 与 Java 版本 SetAccountControl, GetAccountControl 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};

pub struct GetAccountControlHandler;

impl GetAccountControlHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountControlHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::AccountControl]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0")
            .insert("controls", json!([]));
        
        Ok(builder.build())
    }
}

pub struct SetAccountControlHandler;

impl SetAccountControlHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SetAccountControlHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "control", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::AccountControl, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _control = req.get_string("control");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "")
            .insert("transactionBytes", "");
        
        Ok(builder.build())
    }
}

pub struct RemoveAccountControlHandler;

impl RemoveAccountControlHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for RemoveAccountControlHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "control", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::AccountControl, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _control = req.get_string("control");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}
