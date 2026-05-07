//! 资金监控相关 API Handlers
//!
//! 与 Java 版本 GetFundingMonitor, StartFundingMonitor 等完全对齐

use async_trait::async_trait;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct GetFundingMonitorHandler;

impl GetFundingMonitorHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetFundingMonitorHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["holdingType", "holding", "property", "secretPhrase", "includeMonitoredAccounts", "account", "adminPassword", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Addons]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _holding_type = req.get_i32("holdingType");
        let _holding = req.get_u64("holding");
        let _property = req.get_string("property");

        let mut builder = RsRespBuilder::new();
        builder.insert("note", "TODO");

        Ok(builder.build())
    }
}

pub struct StartFundingMonitorHandler;

impl StartFundingMonitorHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for StartFundingMonitorHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["holdingType", "holding", "property", "amount", "threshold", "interval", "secretPhrase"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Addons]
    }

    fn require_post(&self) -> bool {
        true
    }

    fn require_password(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _holding_type = req.require_i32("holdingType")?;
        let _amount = req.require_string("amount")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("note", "TODO");

        Ok(builder.build())
    }
}

pub struct StopFundingMonitorHandler;

impl StopFundingMonitorHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for StopFundingMonitorHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["holdingType", "holding", "property", "secretPhrase", "adminPassword"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Addons]
    }

    fn require_post(&self) -> bool {
        true
    }

    fn require_password(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _holding_type = req.get_i32("holdingType");
        let _holding = req.get_u64("holding");

        let mut builder = RsRespBuilder::new();
        builder.insert("note", "TODO");

        Ok(builder.build())
    }
}
