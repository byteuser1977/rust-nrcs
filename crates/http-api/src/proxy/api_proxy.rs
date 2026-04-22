//! API 代理模块
//!
//! 对应 Java: APIProxy
//!
//! 管理代理节点选择、黑名单、请求转发

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use rand::seq::SliceRandom;
use thiserror::Error;
use tracing::{debug, info, warn};

use blockchain_types::constants::*;

#[derive(Debug, Error)]
pub enum ApiProxyError {
    #[error("No available API peers")]
    NoAvailablePeers,
    
    #[error("Peer {0} is blacklisted")]
    PeerBlacklisted(String),
    
    #[error("Request type {0} cannot be forwarded")]
    NotForwardable(String),
    
    #[error("Proxy request failed: {0}")]
    ProxyFailed(String),
}

pub type ApiProxyResult<T> = std::result::Result<T, ApiProxyError>;

#[derive(Debug, Clone)]
pub struct ProxyPeer {
    pub host: String,
    pub announced_address: String,
    pub api_port: u16,
    pub disabled_apis: HashSet<String>,
    pub is_connectable: bool,
}

impl ProxyPeer {
    pub fn new(host: String, announced_address: String, api_port: u16) -> Self {
        Self {
            host,
            announced_address,
            api_port,
            disabled_apis: HashSet::new(),
            is_connectable: true,
        }
    }
    
    pub fn api_url(&self) -> String {
        format!("http://{}:{}/nrcs", self.host, self.api_port)
    }
    
    pub fn is_api_enabled(&self, request_type: &str) -> bool {
        !self.disabled_apis.contains(request_type)
    }
}

#[derive(Debug, Clone)]
pub struct BlacklistEntry {
    pub host: String,
    pub blacklisted_at: Instant,
    pub expires_at: Instant,
}

impl BlacklistEntry {
    pub fn new(host: String, period_secs: u64) -> Self {
        let now = Instant::now();
        Self {
            host,
            blacklisted_at: now,
            expires_at: now + Duration::from_secs(period_secs),
        }
    }
    
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

pub struct ApiProxyConfig {
    pub enabled: bool,
    pub forced_server_url: Option<String>,
    pub blacklisting_period_secs: u64,
    pub max_blacklisted_peers: usize,
    pub is_light_client: bool,
}

impl Default for ApiProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            forced_server_url: None,
            blacklisting_period_secs: BLACKLISTING_PERIOD_SECS as u64,
            max_blacklisted_peers: 1000,
            is_light_client: false,
        }
    }
}

pub struct ApiProxy {
    config: ApiProxyConfig,
    blacklisted_peers: Arc<RwLock<HashMap<String, BlacklistEntry>>>,
    peers: Arc<RwLock<Vec<ProxyPeer>>>,
    forced_peer: Arc<RwLock<Option<String>>>,
    main_peer_address: Arc<RwLock<Option<String>>>,
    not_forwarded_requests: HashSet<String>,
}

impl ApiProxy {
    pub fn new(config: ApiProxyConfig) -> Self {
        let not_forwarded_requests = Self::build_not_forwarded_set();
        
        Self {
            config,
            blacklisted_peers: Arc::new(RwLock::new(HashMap::new())),
            peers: Arc::new(RwLock::new(Vec::new())),
            forced_peer: Arc::new(RwLock::new(None)),
            main_peer_address: Arc::new(RwLock::new(None)),
            not_forwarded_requests,
        }
    }
    
    fn build_not_forwarded_set() -> HashSet<String> {
        let mut set = HashSet::new();
        set.insert("getBlockchainStatus".to_string());
        set.insert("getState".to_string());
        set.insert("getLog".to_string());
        set.insert("getStackTraces".to_string());
        set.insert("clearUnconfirmedTransactions".to_string());
        set.insert("popOff".to_string());
        set.insert("scan".to_string());
        set.insert("shutdown".to_string());
        set.insert("getPeerInfo".to_string());
        set.insert("dumpPeers".to_string());
        set.insert("addPeer".to_string());
        set.insert("blacklistPeer".to_string());
        set
    }
    
    pub fn is_activated(&self, is_downloading: bool) -> bool {
        self.config.is_light_client || (self.config.enabled && is_downloading)
    }
    
    pub fn is_forwardable(&self, request_type: &str, require_blockchain: bool, require_full_client: bool) -> bool {
        if !require_blockchain {
            return false;
        }
        
        if require_full_client {
            return false;
        }
        
        if self.not_forwarded_requests.contains(request_type) {
            return false;
        }
        
        true
    }
    
    pub async fn get_serving_peer(&self, request_type: &str) -> ApiProxyResult<ProxyPeer> {
        if let Some(ref url) = self.config.forced_server_url {
            return Ok(ProxyPeer::new(
                url.clone(),
                url.clone(),
                80,
            ));
        }
        
        let forced_peer = self.forced_peer.read().await.clone();
        if let Some(host) = forced_peer {
            let peers = self.peers.read().await;
            if let Some(peer) = peers.iter().find(|p| p.host == host) {
                return Ok(peer.clone());
            }
        }
        
        self.cleanup_expired_blacklist().await;
        
        let peers = self.peers.read().await;
        let blacklisted = self.blacklisted_peers.read().await;
        
        let connectable_peers: Vec<&ProxyPeer> = peers
            .iter()
            .filter(|p| {
                p.is_connectable 
                    && !blacklisted.contains_key(&p.host)
                    && p.is_api_enabled(request_type)
            })
            .collect();
        
        if connectable_peers.is_empty() {
            return Err(ApiProxyError::NoAvailablePeers);
        }
        
        let peer = connectable_peers
            .choose(&mut rand::thread_rng())
            .expect("at least one peer");
        
        let mut main_address = self.main_peer_address.write().await;
        *main_address = Some(peer.announced_address.clone());
        
        Ok((*peer).clone())
    }
    
    pub async fn blacklist_host(&self, host: &str) -> bool {
        let mut blacklisted = self.blacklisted_peers.write().await;
        
        if blacklisted.len() >= self.config.max_blacklisted_peers {
            warn!("Too many blacklisted peers");
            return false;
        }
        
        let entry = BlacklistEntry::new(host.to_string(), self.config.blacklisting_period_secs);
        blacklisted.insert(host.to_string(), entry);
        
        info!("Blacklisted API peer: {}", host);
        true
    }
    
    async fn cleanup_expired_blacklist(&self) {
        let mut blacklisted = self.blacklisted_peers.write().await;
        let before_count = blacklisted.len();
        
        blacklisted.retain(|_, entry| {
            if entry.is_expired() {
                debug!("Unblacklisting API peer: {}", entry.host);
                false
            } else {
                true
            }
        });
        
        if blacklisted.len() != before_count {
            debug!("Cleaned up {} expired blacklist entries", before_count - blacklisted.len());
        }
    }
    
    pub async fn add_peer(&self, peer: ProxyPeer) {
        let mut peers = self.peers.write().await;
        
        if let Some(existing) = peers.iter_mut().find(|p| p.host == peer.host) {
            *existing = peer;
        } else {
            peers.push(peer);
        }
    }
    
    pub async fn set_forced_peer(&self, host: Option<String>) {
        let mut forced = self.forced_peer.write().await;
        *forced = host;
    }
    
    pub async fn get_main_peer_address(&self) -> Option<String> {
        let address = self.main_peer_address.read().await.clone();
        if address.is_none() {
            if let Ok(peer) = self.get_serving_peer("").await {
                let mut addr = self.main_peer_address.write().await;
                *addr = Some(peer.announced_address.clone());
                return Some(peer.announced_address);
            }
        }
        address
    }
    
    pub async fn get_blacklisted_count(&self) -> usize {
        self.blacklisted_peers.read().await.len()
    }
    
    pub async fn get_peer_count(&self) -> usize {
        self.peers.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_is_forwardable() {
        let proxy = ApiProxy::new(ApiProxyConfig::default());
        
        assert!(proxy.is_forwardable("getAccount", true, false));
        assert!(proxy.is_forwardable("getTransaction", true, false));
        
        assert!(!proxy.is_forwardable("getBlockchainStatus", true, false));
        assert!(!proxy.is_forwardable("getState", true, false));
        
        assert!(!proxy.is_forwardable("getTime", false, false));
        
        assert!(!proxy.is_forwardable("shutdown", true, false));
    }

    #[tokio::test]
    async fn test_blacklist_host() {
        let proxy = ApiProxy::new(ApiProxyConfig::default());
        
        assert!(proxy.blacklist_host("192.168.1.1").await);
        assert_eq!(proxy.get_blacklisted_count().await, 1);
    }

    #[tokio::test]
    async fn test_add_peer() {
        let proxy = ApiProxy::new(ApiProxyConfig::default());
        
        let peer = ProxyPeer::new("192.168.1.1".to_string(), "peer1.example.com".to_string(), 8080);
        proxy.add_peer(peer).await;
        
        assert_eq!(proxy.get_peer_count().await, 1);
    }

    #[tokio::test]
    async fn test_get_serving_peer_no_peers() {
        let proxy = ApiProxy::new(ApiProxyConfig::default());
        
        let result = proxy.get_serving_peer("getAccount").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_forced_server_url() {
        let config = ApiProxyConfig {
            forced_server_url: Some("http://forced.example.com".to_string()),
            ..Default::default()
        };
        let proxy = ApiProxy::new(config);
        
        let result = proxy.get_serving_peer("getAccount").await;
        assert!(result.is_ok());
        let peer = result.unwrap();
        assert_eq!(peer.host, "http://forced.example.com");
    }
}
