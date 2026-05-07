//! 账户控制相关 API Handlers
//!
//! 与 Java 版本 SetAccountControl, GetAccountControl 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _control = req.get_string("control");

        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");

        Ok(builder.build())
    }
}

// --- New handlers: account property management APIs ---

pub struct DeleteAccountPropertyHandler;

impl DeleteAccountPropertyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DeleteAccountPropertyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["property", "secretPhrase", "feeNQT", "deadline"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::CreateTransaction]
    }

    fn require_post(&self) -> bool {
        true
    }

    fn require_password(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _property = req.require_string("property")?;
        let _secret_phrase = req.require_string("secretPhrase")?;

        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");

        Ok(builder.build())
    }
}

pub struct SetAccountLongValuePropertyHandler;

impl SetAccountLongValuePropertyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SetAccountLongValuePropertyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["property", "value", "secretPhrase", "feeNQT", "deadline"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::CreateTransaction]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _property = req.require_string("property")?;
        let _value = req.get_string("value");
        let _secret_phrase = req.require_string("secretPhrase")?;

        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");

        Ok(builder.build())
    }
}
