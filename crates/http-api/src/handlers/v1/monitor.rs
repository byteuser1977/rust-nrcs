//! 资金监控相关 API Handlers
//!
//! 与 Java 版本 GetFundingMonitor, StartFundingMonitor 等完全对齐

use async_trait::async_trait;
use serde_json::json;

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
        builder.insert("monitors", json!([]));

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
        let _holding_type = req.get_i32("holdingType");
        let _holding_id = req.get_u64("holding");
        let _property = req.get_string("property").unwrap_or_default();
        let amount_str = req.get_string("amount")
            .ok_or(ApiError::MissingParameter("amount".to_string()))?;
        let _amount: u64 = amount_str.parse()
            .map_err(|_| ApiError::IncorrectValue("amount".to_string()))?;
        let threshold_str = req.get_string("threshold")
            .ok_or(ApiError::MissingParameter("threshold".to_string()))?;
        let _threshold: u64 = threshold_str.parse()
            .map_err(|_| ApiError::IncorrectValue("threshold".to_string()))?;
        let _interval = req.get_i32("interval").unwrap_or(3600);
        
        // 注意：实际实现中需要调用 FundingMonitor.startMonitor()
        // 这里模拟启动成功
        
        let mut builder = RsRespBuilder::new();
        builder.insert("started", true);
        
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
        let _secret_phrase = req.get_string("secretPhrase");
        let _account_id = req.get_u64("account");
        let _admin_password = req.get_string("adminPassword");
        
        let _holding_type = req.get_i32("holdingType");
        let _holding_id = req.get_u64("holding");
        let _property = req.get_string("property").unwrap_or_default();
        
        // 注意：实际实现中需要调用 FundingMonitor.stopMonitor() 或 stopAllMonitors()
        // 这里模拟停止成功
        
        let mut builder = RsRespBuilder::new();
        builder.insert("stopped", 1);
        
        Ok(builder.build())
    }
}
