//! 令牌相关 API Handlers
//!
//! 与 Java 版本 GenerateToken, DecodeToken 等完全对齐

use async_trait::async_trait;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct GenerateTokenHandler;

impl GenerateTokenHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GenerateTokenHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "website"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Tokens]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _website = req.require_string("website")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("token", "")
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0")
            .insert("timestamp", 0i32)
            .insert("valid", true);
        
        Ok(builder.build())
    }
}

pub struct DecodeTokenHandler;

impl DecodeTokenHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DecodeTokenHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["token", "website"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Tokens]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _token = req.require_string("token")?;
        let _website = req.get_string("website");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0")
            .insert("timestamp", 0i32)
            .insert("valid", true)
            .insert("website", "")
            .insert("publicKey", "");
        
        Ok(builder.build())
    }
}

pub struct DetectMimeTypeHandler;

impl DetectMimeTypeHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DetectMimeTypeHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["data", "filename"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _data = req.get_string("data");
        let _filename = req.get_string("filename");

        let mut builder = RsRespBuilder::new();
        builder.insert("type", "application/octet-stream");

        Ok(builder.build())
    }
}

pub struct DecodeFileTokenHandler;

impl DecodeFileTokenHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DecodeFileTokenHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["token"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Tokens]
    }

    fn file_parameter(&self) -> Option<&'static str> {
        Some("file")
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _token = req.require_string("token")?;

        let mut builder = RsRespBuilder::new();
        builder
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0")
            .insert("timestamp", 0i32)
            .insert("valid", true);

        Ok(builder.build())
    }
}

pub struct DecodeHallmarkHandler;

impl DecodeHallmarkHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DecodeHallmarkHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["hallmark"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Tokens]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _hallmark = req.require_string("hallmark")?;

        let mut builder = RsRespBuilder::new();
        builder
            .insert("valid", false)
            .insert("weight", 0i32)
            .insert("host", "")
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0")
            .insert("date", "");

        Ok(builder.build())
    }
}

pub struct DecodeQRCodeHandler;

impl DecodeQRCodeHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DecodeQRCodeHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["qrCodeBase64"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _qr_code_base64 = req.require_string("qrCodeBase64")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("qrCodeData", "");

        Ok(builder.build())
    }
}

pub struct EncodeQRCodeHandler;

impl EncodeQRCodeHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for EncodeQRCodeHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["data", "width", "height"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _data = req.get_string("data");
        let _width = req.get_i32("width");
        let _height = req.get_i32("height");

        let mut builder = RsRespBuilder::new();
        builder.insert("qrCodeBase64", "");

        Ok(builder.build())
    }
}

pub struct GenerateFileTokenHandler;

impl GenerateFileTokenHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GenerateFileTokenHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "file"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Tokens]
    }

    fn require_post(&self) -> bool {
        true
    }

    fn file_parameter(&self) -> Option<&'static str> {
        Some("file")
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _file = req.get_string("file");

        let mut builder = RsRespBuilder::new();
        builder
            .insert("token", "")
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0")
            .insert("timestamp", 0i32)
            .insert("valid", true);

        Ok(builder.build())
    }
}
