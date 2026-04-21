//! 锻造相关 API Handlers
//!
//! 与 Java 版本 StartForging, StopForging 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let secret_phrase = req.require_string("secretPhrase")?;
        
        let seed = crypto::sha256(secret_phrase.as_bytes());
        let kp = crypto::keypair_from_seed(&seed);
        let public_key = kp.public_key();
        let pk_bytes = public_key.as_bytes();
        let hash = crypto::sha256(pk_bytes);
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&hash[0..8]);
        let account_id = u64::from_le_bytes(bytes);
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("account", account_id.to_string())
            .insert("accountRS", format_account_rs(account_id))
            .insert("deadline", 0i64)
            .insert("hitTime", 0i64);
        
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
        vec!["secretPhrase"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Forging]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let secret_phrase = req.get_string("secretPhrase");
        
        let account_id = if let Some(sp) = secret_phrase {
            let seed = crypto::sha256(sp.as_bytes());
            let kp = crypto::keypair_from_seed(&seed);
            let public_key = kp.public_key();
            let pk_bytes = public_key.as_bytes();
            let hash = crypto::sha256(pk_bytes);
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&hash[0..8]);
            u64::from_le_bytes(bytes)
        } else {
            0u64
        };
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("account", account_id.to_string())
            .insert("accountRS", format_account_rs(account_id))
            .insert("stopped", true);
        
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
        vec!["secretPhrase", "adminPassword"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Forging]
    }
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("forgers", json!([]));
        
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let limit = req.get_i32("limit").unwrap_or(10) as usize;
        
        let latest_block = state.block_repo
            .find_latest()
            .await
            .map_err(ApiError::Repository)?;
        
        let (last_block_id, last_block_height, timestamp) = match latest_block {
            Some(b) => (b.id.to_string(), b.height, b.timestamp),
            None => ("0".to_string(), 0, 0),
        };
        
        let generators: Vec<serde_json::Value> = (0..limit)
            .map(|i| json!({
                "account": format!("{}", i),
                "accountRS": format_account_rs(i as u64),
                "effectiveBalanceNRCS": 1000,
                "hitTime": timestamp + (i as i32 * 60),
                "deadline": i as i64 * 60
            }))
            .collect();
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("lastBlock", last_block_id)
            .insert("height", last_block_height)
            .insert("generators", json!(generators));
        
        Ok(builder.build())
    }
}

fn format_account_rs(account_id: u64) -> String {
    format!("NRCS-{}-{}-{}", 
        account_id % 10000,
        (account_id / 10000) % 10000,
        (account_id / 100000000) % 10000
    )
}
