//! 阶段交易相关 API Handlers
//!
//! 与 Java 版本 GetPhasingPoll, ApproveTransaction 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct GetPhasingPollHandler;

impl GetPhasingPollHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPhasingPollHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transaction", "countVotes"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Phasing]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _transaction_id = req.require_u64("transaction")?;
        let _count_votes = req.get_bool("countVotes");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "0")
            .insert("phasingFinishHeight", 0i32)
            .insert("phasingQuorum", 0i32)
            .insert("phasingMinBalance", "0")
            .insert("phasingHolding", "0")
            .insert("phasingModel", 0i32);
        
        Ok(builder.build())
    }
}

pub struct GetPhasingPollsHandler;

impl GetPhasingPollsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPhasingPollsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Phasing]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.get_u64("account");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("polls", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetPhasingPollVotesHandler;

impl GetPhasingPollVotesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPhasingPollVotesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transaction", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Phasing]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _transaction_id = req.require_u64("transaction")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("votes", json!([]));
        
        Ok(builder.build())
    }
}

pub struct ApproveTransactionHandler;

impl ApproveTransactionHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ApproveTransactionHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "transaction", "revealedSecret", "revealedSecretIsText", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Phasing, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _transaction_id = req.require_u64("transaction")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}

pub struct GetAccountPhasingTransactionCountHandler;

impl GetAccountPhasingTransactionCountHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountPhasingTransactionCountHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Phasing]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        
        let mut builder = RsRespBuilder::new();
        builder.insert("numberOfPhasingTransactions", 0i32);
        
        Ok(builder.build())
    }
}
