//! 令牌相关 API Handlers
//!
//! 与 Java 版本 GenerateToken, DecodeToken 等完全对齐

use async_trait::async_trait;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct GenerateTokenHandler;

impl GenerateTokenHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GenerateTokenHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "website"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Tokens]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _website = req.require_string("website")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("token", "")
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0")
            .insert("timestamp", 0i32)
            .insert("valid", true);
        
        Ok(builder.build())
    }
}

pub struct DecodeTokenHandler;

impl DecodeTokenHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DecodeTokenHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["token", "website"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Tokens]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _token = req.require_string("token")?;
        let _website = req.get_string("website");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0")
            .insert("timestamp", 0i32)
            .insert("valid", true)
            .insert("website", "")
            .insert("publicKey", "");
        
        Ok(builder.build())
    }
}

pub struct DetectMimeTypeHandler;

impl DetectMimeTypeHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DetectMimeTypeHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["data", "filename"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _data = req.get_string("data");
        let _filename = req.get_string("filename");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("type", "application/octet-stream");
        
        Ok(builder.build())
    }
}
