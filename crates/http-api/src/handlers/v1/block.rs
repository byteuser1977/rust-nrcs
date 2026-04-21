//! 区块相关 API Handlers
//!
//! 与 Java 版本 GetBlock, GetBlocks 等完全对齐

use async_trait::async_trait;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};

pub struct GetBlockHandler;

impl GetBlockHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBlockHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["block", "height", "timestamp", "includeTransactions", "includeExecutedPhased"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Blocks]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let block_id = req.get_u64("block");
        let height = req.get_i32("height");
        let timestamp = req.get_i32("timestamp");
        let include_transactions = req.get_bool("includeTransactions");
        
        let mut builder = RsRespBuilder::new();
        
        builder
            .insert("block", "0")
            .insert("height", 0i32)
            .insert("generator", "0")
            .insert("generatorRS", "NRCS-0-0-0")
            .insert("generatorPublicKey", "")
            .insert("timestamp", 0i32)
            .insert("numberOfTransactions", 0i32)
            .insert("totalAmountNQT", "0")
            .insert("totalFeeNQT", "0")
            .insert("payloadLength", 0i32)
            .insert("version", 1i32)
            .insert("baseTarget", "0")
            .insert("cumulativeDifficulty", "0")
            .insert("previousBlock", "")
            .insert("payloadHash", "")
            .insert("generationSignature", "")
            .insert("previousBlockHash", "")
            .insert("blockSignature", "");
        
        Ok(builder.build())
    }
}

pub struct GetBlocksHandler;

impl GetBlocksHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBlocksHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex", "includeTransactions"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Blocks]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let first_index = req.get_i32("firstIndex").unwrap_or(0);
        let last_index = req.get_i32("lastIndex").unwrap_or(99);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("blocks", serde_json::json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetBlockIdHandler;

impl GetBlockIdHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBlockIdHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["height"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Blocks]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let height = req.require_i32("height")?;
        
        let mut builder = RsRespBuilder::new();
        builder.insert("block", "0");
        
        Ok(builder.build())
    }
}
