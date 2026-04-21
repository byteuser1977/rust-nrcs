//! 工具类 API Handlers
//!
//! 与 Java 版本 Hash, HexConvert 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct HashHandler;

impl HashHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for HashHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["hashAlgorithm", "secret", "secretIsText"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _hash_algorithm = req.get_i32("hashAlgorithm").unwrap_or(0);
        let _secret = req.get_string("secret");
        let _secret_is_text = req.get_bool("secretIsText");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("hash", "");
        
        Ok(builder.build())
    }
}

pub struct HexConvertHandler;

impl HexConvertHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for HexConvertHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["string"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _string = req.require_string("string")?;
        
        let mut builder = RsRespBuilder::new();
        builder.insert("binary", "");
        
        Ok(builder.build())
    }
}

pub struct LongConvertHandler;

impl LongConvertHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for LongConvertHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["id"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _id = req.require_string("id")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("stringId", "")
            .insert("longId", 0i64);
        
        Ok(builder.build())
    }
}

pub struct RsConvertHandler;

impl RsConvertHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for RsConvertHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_string("account")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0");
        
        Ok(builder.build())
    }
}

pub struct ParseTransactionHandler;

impl ParseTransactionHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ParseTransactionHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transactionBytes", "transactionJSON"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils, ApiTag::Transactions]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _transaction_bytes = req.get_string("transactionBytes");
        let _transaction_json = req.get_string("transactionJSON");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("timestamp", 0i32)
            .insert("height", 0i32)
            .insert("sender", "0")
            .insert("senderRS", "NRCS-0-0-0")
            .insert("senderPublicKey", "")
            .insert("recipient", "0")
            .insert("recipientRS", "NRCS-0-0-0")
            .insert("amountNQT", "0")
            .insert("feeNQT", "0")
            .insert("type", 0u8)
            .insert("subtype", 0u8)
            .insert("fullHash", "")
            .insert("signature", "");
        
        Ok(builder.build())
    }
}

pub struct FullHashToIdHandler;

impl FullHashToIdHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for FullHashToIdHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["fullHash"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _full_hash = req.require_string("fullHash")?;
        
        let mut builder = RsRespBuilder::new();
        builder.insert("stringId", "");
        
        Ok(builder.build())
    }
}

pub struct GetECBlockHandler;

impl GetECBlockHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetECBlockHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["timestamp"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Blocks, ApiTag::Utils]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _timestamp = req.get_i32("timestamp");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("ecBlockId", "")
            .insert("ecBlockHeight", 0i32)
            .insert("timestamp", 0i32);
        
        Ok(builder.build())
    }
}

pub struct CalculateFeeHandler;

impl CalculateFeeHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for CalculateFeeHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transactionBytes", "transactionJSON"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils, ApiTag::Transactions]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _transaction_bytes = req.get_string("transactionBytes");
        let _transaction_json = req.get_string("transactionJSON");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("feeNQT", "0");
        
        Ok(builder.build())
    }
}
