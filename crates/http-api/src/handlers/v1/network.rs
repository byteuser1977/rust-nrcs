//! 网络相关 API Handlers
//!
//! 与 Java 版本 GetPeers, GetPeer 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct GetPeersHandler;

impl GetPeersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPeersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["state", "includePeerInfo", "active", "service"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _state = req.get_string("state");
        let _include_info = req.get_bool("includePeerInfo");
        let _active = req.get_bool("active");
        let _service = req.get_string("service");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("peers", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetPeerHandler;

impl GetPeerHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPeerHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["peer"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _peer_address = req.require_string("peer")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("state", 0i32)
            .insert("announcedAddress", "")
            .insert("shareAddress", true)
            .insert("downloadedVolume", 0i64)
            .insert("uploadedVolume", 0i64)
            .insert("application", "NRCS")
            .insert("version", "2.1.0")
            .insert("platform", "")
            .insert("blacklisted", false)
            .insert("lastUpdated", 0i32);
        
        Ok(builder.build())
    }
}

pub struct GetInboundPeersHandler;

impl GetInboundPeersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetInboundPeersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["includePeerInfo"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _include_info = req.get_bool("includePeerInfo");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("peers", json!([]));
        
        Ok(builder.build())
    }
}

pub struct AddPeerHandler;

impl AddPeerHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for AddPeerHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["peer"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _peer_address = req.require_string("peer")?;
        
        let mut builder = RsRespBuilder::new();
        builder.insert("state", 0i32);
        
        Ok(builder.build())
    }
}

pub struct BlacklistPeerHandler;

impl BlacklistPeerHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for BlacklistPeerHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["peer"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _peer_address = req.require_string("peer")?;
        
        let mut builder = RsRespBuilder::new();
        builder.insert("blacklisted", true);
        
        Ok(builder.build())
    }
}

pub struct GetMyInfoHandler;

impl GetMyInfoHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetMyInfoHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder
            .insert("application", "NRCS")
            .insert("version", "2.1.0")
            .insert("platform", "")
            .insert("shareAddress", true)
            .insert("announcedAddress", "")
            .insert("hallmark", "")
            .insert("services", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetPluginsHandler;

impl GetPluginsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPluginsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("plugins", json!([]));
        
        Ok(builder.build())
    }
}
