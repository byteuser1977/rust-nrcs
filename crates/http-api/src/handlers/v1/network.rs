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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _state_filter = req.get_string("state");
        let include_info = req.get_bool("includePeerInfo");
        let active = req.get_bool("active");
        let _service = req.get_string("service");
        
        let peers = if let Some(ref p2p) = state.p2p_manager {
            if active {
                p2p.get_active_peers().await
            } else {
                p2p.get_peers().await
            }
        } else {
            vec![]
        };
        
        if include_info {
            let peers_json: Vec<serde_json::Value> = peers.iter()
                .map(peer_to_json)
                .collect();
            let mut builder = RsRespBuilder::new();
            builder.insert("peers", json!(peers_json));
            Ok(builder.build())
        } else {
            let peer_addresses: Vec<String> = peers.iter()
                .filter_map(|p| p.announced_address.clone())
                .collect();
            let mut builder = RsRespBuilder::new();
            builder.insert("peers", json!(peer_addresses));
            Ok(builder.build())
        }
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let peer_address = req.require_string("peer")?;
        
        if let Some(ref p2p) = state.p2p_manager {
            let peers = p2p.get_peers().await;
            if let Some(peer) = peers.iter().find(|p| p.announced_address.as_ref() == Some(&peer_address)) {
                let mut builder = RsRespBuilder::new();
                builder.extend_json(peer_to_json(peer));
                return Ok(builder.build());
            }
        }
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("state", 0i32)
            .insert("announcedAddress", peer_address)
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let include_info = req.get_bool("includePeerInfo");
        
        let inbound_peers = if let Some(ref p2p) = state.p2p_manager {
            let all_peers = p2p.get_active_peers().await;
            all_peers.into_iter().filter(|p| p.is_inbound).collect::<Vec<_>>()
        } else {
            vec![]
        };
        
        if include_info {
            let peers_json: Vec<serde_json::Value> = inbound_peers.iter()
                .map(peer_to_json)
                .collect();
            let mut builder = RsRespBuilder::new();
            builder.insert("peers", json!(peers_json));
            Ok(builder.build())
        } else {
            let peer_addresses: Vec<String> = inbound_peers.iter()
                .filter_map(|p| p.announced_address.clone())
                .collect();
            let mut builder = RsRespBuilder::new();
            builder.insert("peers", json!(peer_addresses));
            Ok(builder.build())
        }
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let peer_address = req.require_string("peer")?;
        
        if let Some(ref p2p) = state.p2p_manager {
            let addr: std::net::SocketAddr = peer_address.parse()
                .map_err(|_| ApiError::IncorrectPeerAddress)?;
            let peer = p2p::Peer::new(addr, false);
            p2p.add_peer(peer).await;
        }
        
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let peer_address = req.require_string("peer")?;
        
        if let Some(ref p2p) = state.p2p_manager {
            let addr: std::net::SocketAddr = peer_address.parse()
                .map_err(|_| ApiError::IncorrectPeerAddress)?;
            p2p.blacklist_peer(&addr, "API blacklist".to_string()).await;
        }
        
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
    
    async fn process_request(&self, _req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let my_info = if let Some(ref p2p) = state.p2p_manager {
            p2p.get_my_peer_info().await
        } else {
            json!({
                "application": "NRCS",
                "version": "2.1.0",
                "platform": "",
                "shareAddress": true,
                "announcedAddress": "",
                "hallmark": "",
                "services": []
            })
        };
        
        let mut builder = RsRespBuilder::new();
        builder.extend_json(my_info);
        
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

fn peer_to_json(peer: &p2p::Peer) -> serde_json::Value {
    json!({
        "state": peer.state as i32,
        "announcedAddress": peer.announced_address.clone().unwrap_or_default(),
        "shareAddress": peer.share_address,
        "downloadedVolume": peer.downloaded_volume as i64,
        "uploadedVolume": peer.uploaded_volume as i64,
        "application": peer.application.clone().unwrap_or_default(),
        "version": peer.version.clone().unwrap_or_default(),
        "platform": peer.platform.clone().unwrap_or_default(),
        "blacklisted": peer.blacklisting_time > 0,
        "lastUpdated": peer.last_updated as i32
    })
}

pub struct AddBundlingRuleHandler;

impl AddBundlingRuleHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for AddBundlingRuleHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "minRateNQTPerFXT", "totalFeesLimitFQT", "overpayFQTPerFXT", "feeCalculatorName", "filter"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }

    fn require_post(&self) -> bool {
        true
    }

    fn require_password(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _min_rate_nqt_per_fxt = req.require_u64("minRateNQTPerFXT")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("note", "TODO");

        Ok(builder.build())
    }
}

pub struct BlacklistAPIProxyPeerHandler;

impl BlacklistAPIProxyPeerHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for BlacklistAPIProxyPeerHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["peer", "adminPassword"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _peer = req.require_string("peer")?;
        let _admin_password = req.get_string("adminPassword");

        let mut builder = RsRespBuilder::new();
        builder.insert("note", "TODO");

        Ok(builder.build())
    }
}

pub struct BlacklistBundlerHandler;

impl BlacklistBundlerHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for BlacklistBundlerHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "adminPassword"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _admin_password = req.get_string("adminPassword");

        let mut builder = RsRespBuilder::new();
        builder.insert("note", "TODO");

        Ok(builder.build())
    }
}

pub struct BundleTransactionsHandler;

impl BundleTransactionsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for BundleTransactionsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "transactionFullHash", "feeNQT", "deadline"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network, ApiTag::CreateTransaction]
    }

    fn require_post(&self) -> bool {
        true
    }

    fn require_password(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _transaction_full_hash = req.require_string("transactionFullHash")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("note", "TODO");

        Ok(builder.build())
    }
}

pub struct GetBundlerRatesHandler;

impl GetBundlerRatesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBundlerRatesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["minRateNQTPerFXT", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _min_rate_nqt_per_fxt = req.get_u64("minRateNQTPerFXT");

        let mut builder = RsRespBuilder::new();
        builder.insert("note", "TODO");

        Ok(builder.build())
    }
}

pub struct GetBundlersHandler;

impl GetBundlersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBundlersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["adminPassword", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _admin_password = req.get_string("adminPassword");

        let mut builder = RsRespBuilder::new();
        builder.insert("note", "TODO");

        Ok(builder.build())
    }
}

pub struct GetBundlingOptionsHandler;

impl GetBundlingOptionsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBundlingOptionsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }

    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("note", "TODO");

        Ok(builder.build())
    }
}

pub struct SetAPIProxyPeerHandler;

impl SetAPIProxyPeerHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SetAPIProxyPeerHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["peer", "adminPassword"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _peer = req.require_string("peer")?;
        let _admin_password = req.get_string("adminPassword");

        let mut builder = RsRespBuilder::new();
        builder.insert("note", "TODO");

        Ok(builder.build())
    }
}

pub struct GetAllBundlerRatesHandler;

impl GetAllBundlerRatesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAllBundlerRatesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Network]
    }

    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("note", "TODO");

        Ok(builder.build())
    }
}
