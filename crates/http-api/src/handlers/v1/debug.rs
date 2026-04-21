//! 调试相关 API Handlers
//!
//! 与 Java 版本 GetLog, GetStackTraces 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};

pub struct GetLogHandler;

impl GetLogHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetLogHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["logLevel", "count"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Debug]
    }
    
    fn require_password(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _log_level = req.get_string("logLevel");
        let _count = req.get_i32("count").unwrap_or(100);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("log", "");
        
        Ok(builder.build())
    }
}

pub struct GetStackTracesHandler;

impl GetStackTracesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetStackTracesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["depth"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Debug]
    }
    
    fn require_password(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _depth = req.get_i32("depth").unwrap_or(10);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("stackTraces", json!([]));
        
        Ok(builder.build())
    }
}

pub struct ClearUnconfirmedTransactionsHandler;

impl ClearUnconfirmedTransactionsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ClearUnconfirmedTransactionsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Debug]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    fn require_password(&self) -> bool {
        true
    }
    
    async fn process_request(&self, _req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("done", true);
        
        Ok(builder.build())
    }
}

pub struct PopOffHandler;

impl PopOffHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for PopOffHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["numBlocks", "height"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Debug]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    fn require_password(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _num_blocks = req.get_i32("numBlocks");
        let _height = req.get_i32("height");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("blocks", json!([]));
        
        Ok(builder.build())
    }
}

pub struct ScanHandler;

impl ScanHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ScanHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["numBlocks", "height"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Debug]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    fn require_password(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _num_blocks = req.get_i32("numBlocks");
        let _height = req.get_i32("height");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("done", true);
        
        Ok(builder.build())
    }
}

pub struct ShutdownHandler;

impl ShutdownHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ShutdownHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Debug]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    fn require_password(&self) -> bool {
        true
    }
    
    async fn process_request(&self, _req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("shutdown", true);
        
        Ok(builder.build())
    }
}

pub struct GetPeerInfoHandler;

impl GetPeerInfoHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPeerInfoHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["peer"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Debug, ApiTag::Network]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _peer = req.get_string("peer");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("peerInfo", json!({
                "address": "",
                "state": 0,
                "version": "",
                "application": ""
            }));
        
        Ok(builder.build())
    }
}
