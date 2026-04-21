//! P2P网络配置 - 引用全局常量
//!
//! 对应 Java: Peers.java 和 nrcs-default.properties 中的P2P配置

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use crate::constants::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    pub min_compress_size: usize,
    pub max_connections: usize,
    pub max_inbound_connections: usize,
    pub max_outbound_connections: usize,
    pub connect_timeout_ms: u64,
    pub read_timeout_ms: u64,
    pub websocket_idle_timeout_secs: u64,
    pub max_request_size: usize,
    pub max_response_size: usize,
    pub max_message_size: usize,
    pub blacklisting_period_secs: i64,
    pub blacklisting_enabled: bool,
    pub blacklisting_threshold: i32,
    pub get_more_peers: bool,
    pub max_known_peers: usize,
    pub min_known_peers: usize,
    pub use_peers_db: bool,
    pub save_peers: bool,
    pub use_websockets: bool,
    pub enable_hallmark_protection: bool,
    pub push_threshold: i32,
    pub pull_threshold: i32,
    pub send_to_peers_limit: usize,
    pub max_version_length: usize,
    pub max_application_length: usize,
    pub max_platform_length: usize,
    pub max_announced_address_length: usize,
    pub listen_addr: String,
    pub my_address: Option<String>,
    pub share_my_address: bool,
    pub my_platform: String,
    pub my_hallmark: Option<String>,
    pub connection_daemon_interval_secs: u64,
    pub discovery_daemon_interval_secs: u64,
    pub unblacklist_daemon_interval_secs: u64,
    pub transaction_daemon_interval_secs: u64,
    pub send_transactions_batch_size: usize,
    pub bundler_rate_broadcast_interval_secs: u64,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            min_compress_size: MIN_COMPRESS_SIZE,
            max_connections: MAX_CONNECTIONS,
            max_inbound_connections: MAX_INBOUND_CONNECTIONS,
            max_outbound_connections: MAX_OUTBOUND_CONNECTIONS,
            connect_timeout_ms: CONNECT_TIMEOUT_MS,
            read_timeout_ms: READ_TIMEOUT_MS,
            websocket_idle_timeout_secs: WEBSOCKET_IDLE_TIMEOUT_SECS,
            max_request_size: MAX_REQUEST_SIZE,
            max_response_size: MAX_RESPONSE_SIZE,
            max_message_size: MAX_MESSAGE_SIZE,
            blacklisting_period_secs: BLACKLISTING_PERIOD_SECS,
            blacklisting_enabled: true,
            blacklisting_threshold: BLACKLISTING_THRESHOLD,
            get_more_peers: true,
            max_known_peers: MAX_KNOWN_PEERS,
            min_known_peers: MIN_KNOWN_PEERS,
            use_peers_db: true,
            save_peers: true,
            use_websockets: true,
            enable_hallmark_protection: true,
            push_threshold: 0,
            pull_threshold: 0,
            send_to_peers_limit: 10,
            max_version_length: MAX_VERSION_LENGTH,
            max_application_length: MAX_APPLICATION_LENGTH,
            max_platform_length: MAX_PLATFORM_LENGTH,
            max_announced_address_length: MAX_ANNOUNCED_ADDRESS_LENGTH,
            listen_addr: "0.0.0.0:16974".to_string(),
            my_address: None,
            share_my_address: true,
            my_platform: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
            my_hallmark: None,
            connection_daemon_interval_secs: CONNECTION_DAEMON_INTERVAL_SECS,
            discovery_daemon_interval_secs: DISCOVERY_DAEMON_INTERVAL_SECS,
            unblacklist_daemon_interval_secs: UNBLACKLIST_DAEMON_INTERVAL_SECS,
            transaction_daemon_interval_secs: TRANSACTION_DAEMON_INTERVAL_SECS,
            send_transactions_batch_size: SEND_TRANSACTIONS_BATCH_SIZE,
            bundler_rate_broadcast_interval_secs: BUNDLER_RATE_BROADCAST_INTERVAL_SECS,
        }
    }
}

impl P2PConfig {
    pub fn from_toml(value: &toml::Value) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut config = Self::default();
        
        if let Some(listen_addr) = value.get("listen_addr").and_then(|v| v.as_str()) {
            config.listen_addr = listen_addr.to_string();
        }
        if let Some(max_connections) = value.get("max_connections").and_then(|v| v.as_integer()) {
            config.max_connections = max_connections as usize;
        }
        if let Some(max_inbound_connections) = value.get("max_inbound_connections").and_then(|v| v.as_integer()) {
            config.max_inbound_connections = max_inbound_connections as usize;
        }
        if let Some(max_outbound_connections) = value.get("max_outbound_connections").and_then(|v| v.as_integer()) {
            config.max_outbound_connections = max_outbound_connections as usize;
        }
        if let Some(my_address) = value.get("my_address").and_then(|v| v.as_str()) {
            config.my_address = Some(my_address.to_string());
        }
        if let Some(share_my_address) = value.get("share_my_address").and_then(|v| v.as_bool()) {
            config.share_my_address = share_my_address;
        }
        if let Some(use_websockets) = value.get("use_websockets").and_then(|v| v.as_bool()) {
            config.use_websockets = use_websockets;
        }
        
        Ok(config)
    }
    
    pub fn listen_socket_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        self.listen_addr.parse()
    }
    
    pub fn has_enough_connected_peers(&self, count: usize) -> bool {
        count >= self.max_connections
    }
    
    pub fn too_many_known_peers(&self, count: usize) -> bool {
        count >= self.max_known_peers
    }
    
    pub fn too_few_known_peers(&self, count: usize) -> bool {
        count < self.min_known_peers
    }
    
    pub fn too_many_outbound_connections(&self, count: usize) -> bool {
        count >= self.max_outbound_connections
    }
    
    pub fn too_many_inbound_connections(&self, count: usize) -> bool {
        count >= self.max_inbound_connections
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_p2p_config() {
        let config = P2PConfig::default();
        assert_eq!(config.min_compress_size, MIN_COMPRESS_SIZE);
        assert_eq!(config.max_connections, MAX_CONNECTIONS);
        assert!(config.max_known_peers > config.min_known_peers);
    }

    #[test]
    fn test_peer_count_checks() {
        let config = P2PConfig::default();
        
        assert!(config.has_enough_connected_peers(MAX_CONNECTIONS));
        assert!(!config.has_enough_connected_peers(10));
        
        assert!(config.too_many_known_peers(3000));
        assert!(!config.too_many_known_peers(100));
        
        assert!(config.too_few_known_peers(50));
        assert!(!config.too_few_known_peers(150));
    }
}
