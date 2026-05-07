//! 洗牌相关 API Handlers
//!
//! 与 Java 版本 GetShuffling, ShufflingCreate 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct GetShufflingHandler;

impl GetShufflingHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetShufflingHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["shuffling"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Shuffling]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _shuffling_id = req.require_u64("shuffling")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("shuffling", "0")
            .insert("shufflingFullHash", "")
            .insert("issuer", "0")
            .insert("issuerRS", "NRCS-0-0-0")
            .insert("amount", "0")
            .insert("participantCount", 0i32)
            .insert("registrationPeriod", 0i32)
            .insert("blocksRemaining", 0i32)
            .insert("isFinished", false);
        
        Ok(builder.build())
    }
}

pub struct GetAllShufflingsHandler;

impl GetAllShufflingsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAllShufflingsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex", "includeFinished"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Shuffling]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        let _include_finished = req.get_bool("includeFinished");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("shufflings", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAccountShufflingsHandler;

impl GetAccountShufflingsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountShufflingsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "firstIndex", "lastIndex", "includeFinished"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Shuffling]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("shufflings", json!([]));
        
        Ok(builder.build())
    }
}

pub struct ShufflingCreateHandler;

impl ShufflingCreateHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ShufflingCreateHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "holding", "holdingType", "amount", "participantCount", 
             "registrationPeriod", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Shuffling, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _holding = req.get_u64("holding");
        let _amount = req.require_string("amount")?;
        let _participant_count = req.require_i32("participantCount")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "")
            .insert("transactionBytes", "");
        
        Ok(builder.build())
    }
}

pub struct ShufflingRegisterHandler;

impl ShufflingRegisterHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ShufflingRegisterHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "shufflingFullHash", "shuffling", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Shuffling, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _shuffling_id = req.get_u64("shuffling");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}

pub struct ShufflingProcessHandler;

impl ShufflingProcessHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ShufflingProcessHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "shuffling", "recipientSecretPhrase", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Shuffling, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _shuffling_id = req.require_u64("shuffling")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}

pub struct ShufflingVerifyHandler;

impl ShufflingVerifyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ShufflingVerifyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "shuffling", "shufflingStateHash", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Shuffling, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _shuffling_id = req.require_u64("shuffling")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}

pub struct ShufflingCancelHandler;

impl ShufflingCancelHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ShufflingCancelHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "shuffling", "cancellingAccount", "feeNQT", "deadline"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Shuffling, ApiTag::CreateTransaction]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _shuffling_id = req.require_u64("shuffling")?;

        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");

        Ok(builder.build())
    }
}

pub struct GetAssignedShufflingsHandler;

impl GetAssignedShufflingsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAssignedShufflingsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "firstIndex", "lastIndex", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Shuffling]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);

        let mut builder = RsRespBuilder::new();
        builder.insert("note", "TODO");

        Ok(builder.build())
    }
}

pub struct GetHoldingShufflingsHandler;

impl GetHoldingShufflingsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetHoldingShufflingsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["holding", "holdingType", "firstIndex", "lastIndex", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Shuffling]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _holding = req.get_u64("holding");
        let _holding_type = req.get_i32("holdingType");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);

        let mut builder = RsRespBuilder::new();
        builder.insert("note", "TODO");

        Ok(builder.build())
    }
}

pub struct GetShufflersHandler;

impl GetShufflersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetShufflersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["adminPassword", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Shuffling]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _admin_password = req.get_string("adminPassword");

        let mut builder = RsRespBuilder::new();
        builder.insert("note", "TODO");

        Ok(builder.build())
    }
}

pub struct GetShufflingParticipantsHandler;

impl GetShufflingParticipantsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetShufflingParticipantsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["shuffling", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Shuffling]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _shuffling_id = req.require_u64("shuffling")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("note", "TODO");

        Ok(builder.build())
    }
}

pub struct StartShufflerHandler;

impl StartShufflerHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for StartShufflerHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "shufflingFullHash", "recipientSecretPhrase", "recipientPublicKey"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Shuffling]
    }

    fn require_post(&self) -> bool {
        true
    }

    fn require_password(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _shuffling_full_hash = req.require_string("shufflingFullHash")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("note", "TODO");

        Ok(builder.build())
    }
}

pub struct StopShufflerHandler;

impl StopShufflerHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for StopShufflerHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "shufflingFullHash", "adminPassword"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Shuffling]
    }

    fn require_post(&self) -> bool {
        true
    }

    fn require_password(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _shuffling_full_hash = req.require_string("shufflingFullHash")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("note", "TODO");

        Ok(builder.build())
    }
}
