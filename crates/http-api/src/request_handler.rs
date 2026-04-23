//! API 请求处理器定义
//!
//! 与 Java 版本 IRequestHandler 接口完全对齐

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::state::ApiState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;
use async_trait::async_trait;

pub struct ApiRequest {
    pub params: HashMap<String, String>,
}

impl ApiRequest {
    pub fn new(params: HashMap<String, String>) -> Self {
        Self { params }
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }
    
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.params.get(key).cloned()
    }
    
    pub fn get_bool(&self, key: &str) -> bool {
        self.params.get(key).map(|v| v.eq_ignore_ascii_case("true")).unwrap_or(false)
    }
    
    pub fn get_i32(&self, key: &str) -> Option<i32> {
        self.params.get(key).and_then(|v| v.parse().ok())
    }
    
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.params.get(key).and_then(|v| v.parse().ok())
    }
    
    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.params.get(key).and_then(|v| v.parse().ok())
    }
    
    pub fn require_string(&self, key: &str) -> Result<String, ApiError> {
        self.params.get(key).cloned()
            .ok_or_else(|| ApiError::MissingParameter(key.to_string()))
    }
    
    pub fn require_u64(&self, key: &str) -> Result<u64, ApiError> {
        self.params.get(key)
            .ok_or_else(|| ApiError::MissingParameter(key.to_string()))
            .and_then(|v| v.parse::<u64>()
                .map_err(|_| ApiError::IncorrectValue(key.to_string())))
    }
    
    pub fn require_i32(&self, key: &str) -> Result<i32, ApiError> {
        self.params.get(key)
            .ok_or_else(|| ApiError::MissingParameter(key.to_string()))
            .and_then(|v| v.parse::<i32>()
                .map_err(|_| ApiError::IncorrectValue(key.to_string())))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsResp {
    #[serde(rename = "errorCode", skip_serializing_if = "Option::is_none")]
    pub error_code: Option<i32>,
    
    #[serde(rename = "errorDescription", skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
    
    #[serde(rename = "requestProcessingTime")]
    pub request_processing_time: u64,
    
    #[serde(rename = "lastBlock", skip_serializing_if = "Option::is_none")]
    pub last_block: Option<String>,
}

impl RsResp {
    pub fn success() -> Self {
        Self {
            error_code: None,
            error_description: None,
            request_processing_time: 0,
            last_block: None,
        }
    }
    
    pub fn error(code: i32, description: impl Into<String>) -> Self {
        Self {
            error_code: Some(code),
            error_description: Some(description.into()),
            request_processing_time: 0,
            last_block: None,
        }
    }
    
    pub fn with_processing_time(mut self, time: u64) -> Self {
        self.request_processing_time = time;
        self
    }
    
    pub fn with_last_block(mut self, block: impl Into<String>) -> Self {
        self.last_block = Some(block.into());
        self
    }
}

pub struct RsRespBuilder {
    data: serde_json::Map<String, Value>,
}

impl RsRespBuilder {
    pub fn new() -> Self {
        Self {
            data: serde_json::Map::new(),
        }
    }
    
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<Value>) -> &mut Self {
        self.data.insert(key.into(), value.into());
        self
    }
    
    pub fn insert_opt(&mut self, key: impl Into<String>, value: Option<impl Into<Value>>) -> &mut Self {
        if let Some(v) = value {
            self.data.insert(key.into(), v.into());
        }
        self
    }
    
    pub fn extend_json(&mut self, json: Value) -> &mut Self {
        if let Value::Object(map) = json {
            for (k, v) in map {
                self.data.insert(k, v);
            }
        }
        self
    }
    
    pub fn build(self) -> RsRespWithData {
        RsRespWithData {
            base: RsResp::success(),
            data: self.data,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RsRespWithData {
    #[serde(flatten)]
    pub base: RsResp,
    
    #[serde(flatten)]
    pub data: serde_json::Map<String, Value>,
}

impl RsRespWithData {
    pub fn with_processing_time(mut self, time: u64) -> Self {
        self.base.request_processing_time = time;
        self
    }
    
    pub fn with_last_block(mut self, block: impl Into<String>) -> Self {
        self.base.last_block = Some(block.into());
        self
    }
    
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

#[async_trait]
pub trait RequestHandler: Send + Sync {
    fn parameters(&self) -> Vec<&'static str> {
        vec![]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![]
    }
    
    fn file_parameter(&self) -> Option<&'static str> {
        None
    }
    
    fn require_post(&self) -> bool {
        false
    }
    
    fn require_password(&self) -> bool {
        false
    }
    
    fn require_blockchain(&self) -> bool {
        true
    }
    
    fn allow_required_block_parameters(&self) -> bool {
        true
    }
    
    fn start_db_transactions(&self) -> bool {
        false
    }
    
    fn require_full_client(&self) -> bool {
        false
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError>;
}

pub type HandlerPtr = std::sync::Arc<dyn RequestHandler>;

pub struct HandlerContext {
    pub start_time: Instant,
    pub require_block: Option<u64>,
    pub require_last_block: Option<u64>,
}

impl HandlerContext {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            require_block: None,
            require_last_block: None,
        }
    }
    
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

pub mod responses {
    use super::*;
    
    pub fn incorrect_request() -> RsRespWithData {
        RsResp::error(1, "Incorrect request").into()
    }
    
    pub fn not_allowed() -> RsRespWithData {
        RsResp::error(2, "Not allowed").into()
    }
    
    pub fn post_required() -> RsRespWithData {
        RsResp::error(3, "POST required").into()
    }
    
    pub fn unknown_account() -> RsRespWithData {
        RsResp::error(5, "Unknown account").into()
    }
    
    pub fn unknown_block() -> RsRespWithData {
        RsResp::error(6, "Unknown block").into()
    }
    
    pub fn unknown_transaction() -> RsRespWithData {
        RsResp::error(7, "Unknown transaction").into()
    }
    
    pub fn incorrect_account() -> RsRespWithData {
        RsResp::error(8, "Incorrect account").into()
    }
    
    pub fn incorrect_block() -> RsRespWithData {
        RsResp::error(9, "Incorrect block").into()
    }
    
    pub fn incorrect_height() -> RsRespWithData {
        RsResp::error(10, "Incorrect height").into()
    }
    
    pub fn incorrect_timestamp() -> RsRespWithData {
        RsResp::error(11, "Incorrect timestamp").into()
    }
    
    pub fn missing_parameter(param: &str) -> RsRespWithData {
        RsResp::error(12, format!("Missing parameter: {}", param)).into()
    }
    
    pub fn incorrect_value(param: &str) -> RsRespWithData {
        RsResp::error(13, format!("Incorrect value: {}", param)).into()
    }
    
    pub fn disabled_api() -> RsRespWithData {
        RsResp::error(14, "Disabled API").into()
    }
    
    pub fn required_block_not_found() -> RsRespWithData {
        RsResp::error(15, "Required block not found").into()
    }
    
    pub fn required_last_block_not_found() -> RsRespWithData {
        RsResp::error(16, "Required last block not found").into()
    }
    
    pub fn light_client_disabled_api() -> RsRespWithData {
        RsResp::error(17, "Light client disabled API").into()
    }
}

impl From<RsResp> for RsRespWithData {
    fn from(resp: RsResp) -> Self {
        Self {
            base: resp,
            data: serde_json::Map::new(),
        }
    }
}
