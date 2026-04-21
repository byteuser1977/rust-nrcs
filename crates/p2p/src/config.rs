//! P2P Configuration (对应 NRCS Peers.java 配置常量)
//!
//! 所有配置项与 NRCS Java 版本一一对应
//! 默认值引用 blockchain-types 中的全局常量

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use blockchain_types::constants::*;

/// P2P Configuration
/// 
/// 对应 NRCS Java: Peers.java 中的配置常量
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct P2PConfig {
    // ============ 压缩配置 ============
    /// Minimum size for compression
    /// NRCS: MIN_COMPRESS_SIZE = 256
    #[serde(default = "default_min_compress_size")]
    pub min_compress_size: usize,

    // ============ 连接配置 ============
    /// Maximum number of connected public peers
    /// NRCS: maxNumberOfConnectedPublicPeers
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,

    /// Maximum number of inbound connections
    /// NRCS: maxNumberOfInboundConnections
    #[serde(default = "default_max_inbound_connections")]
    pub max_inbound_connections: usize,

    /// Maximum number of outbound connections
    /// NRCS: maxNumberOfOutboundConnections
    #[serde(default = "default_max_outbound_connections")]
    pub max_outbound_connections: usize,

    /// Connection timeout in milliseconds
    /// NRCS: connectTimeout
    #[serde(default = "default_connect_timeout_ms")]
    pub connect_timeout_ms: u64,

    /// Read timeout in milliseconds
    /// NRCS: readTimeout
    #[serde(default = "default_read_timeout_ms")]
    pub read_timeout_ms: u64,

    /// WebSocket idle timeout in seconds
    /// NRCS: webSocketIdleTimeout
    #[serde(default = "default_websocket_idle_timeout_secs")]
    pub websocket_idle_timeout_secs: u64,

    // ============ 消息大小限制 ============
    /// Maximum request size in bytes
    /// NRCS: MAX_REQUEST_SIZE = 64 * 1024 * 1024
    #[serde(default = "default_max_request_size")]
    pub max_request_size: usize,

    /// Maximum response size in bytes
    /// NRCS: MAX_RESPONSE_SIZE = 64 * 1024 * 1024
    #[serde(default = "default_max_response_size")]
    pub max_response_size: usize,

    /// Maximum message size in bytes
    /// NRCS: MAX_MESSAGE_SIZE = 40 * 1024 * 1024
    #[serde(default = "default_max_message_size")]
    pub max_message_size: usize,

    // ============ 黑名单配置 ============
    /// Blacklisting period in seconds
    /// NRCS: blacklistingPeriod
    #[serde(default = "default_blacklisting_period_secs")]
    pub blacklisting_period_secs: i64,

    /// Enable blacklisting
    /// NRCS: blacklistingEnabled
    #[serde(default = "default_blacklisting_enabled")]
    pub blacklisting_enabled: bool,

    /// Blacklisting threshold
    /// NRCS: blacklistingThreshold
    #[serde(default = "default_blacklisting_threshold")]
    pub blacklisting_threshold: i32,

    // ============ 节点发现配置 ============
    /// Get more peers from connected peers
    /// NRCS: getMorePeers
    #[serde(default = "default_get_more_peers")]
    pub get_more_peers: bool,

    /// Maximum number of known peers
    /// NRCS: maxNumberOfKnownPeers
    #[serde(default = "default_max_known_peers")]
    pub max_known_peers: usize,

    /// Minimum number of known peers
    /// NRCS: minNumberOfKnownPeers
    #[serde(default = "default_min_known_peers")]
    pub min_known_peers: usize,

    /// Use peer database
    /// NRCS: usePeersDb
    #[serde(default = "default_use_peers_db")]
    pub use_peers_db: bool,

    /// Save peers to database
    /// NRCS: savePeers
    #[serde(default = "default_save_peers")]
    pub save_peers: bool,

    // ============ WebSocket 配置 ============
    /// Use WebSocket for P2P communication
    /// NRCS: useWebSockets
    #[serde(default = "default_use_websockets")]
    pub use_websockets: bool,

    // ============ Hallmark 配置 ============
    /// Enable hallmark protection
    /// NRCS: enableHallmarkProtection
    #[serde(default = "default_enable_hallmark_protection")]
    pub enable_hallmark_protection: bool,

    /// Push threshold
    /// NRCS: pushThreshold
    #[serde(default = "default_push_threshold")]
    pub push_threshold: i32,

    /// Pull threshold
    /// NRCS: pullThreshold
    #[serde(default = "default_pull_threshold")]
    pub pull_threshold: i32,

    /// Send to peers limit
    /// NRCS: sendToPeersLimit
    #[serde(default = "default_send_to_peers_limit")]
    pub send_to_peers_limit: usize,

    // ============ 字段长度限制 ============
    /// Maximum version string length
    /// NRCS: MAX_VERSION_LENGTH = 10
    #[serde(default = "default_max_version_length")]
    pub max_version_length: usize,

    /// Maximum application string length
    /// NRCS: MAX_APPLICATION_LENGTH = 20
    #[serde(default = "default_max_application_length")]
    pub max_application_length: usize,

    /// Maximum platform string length
    /// NRCS: MAX_PLATFORM_LENGTH = 30
    #[serde(default = "default_max_platform_length")]
    pub max_platform_length: usize,

    /// Maximum announced address string length
    /// NRCS: MAX_ANNOUNCED_ADDRESS_LENGTH = 100
    #[serde(default = "default_max_announced_address_length")]
    pub max_announced_address_length: usize,

    // ============ 本节点配置 ============
    /// Listen address for P2P server
    #[serde(default = "default_listen_addr")]
    pub listen_addr: SocketAddr,

    /// My announced address
    /// NRCS: myAddress
    #[serde(default)]
    pub my_address: Option<String>,

    /// Share my address with other peers
    /// NRCS: shareMyAddress
    #[serde(default = "default_share_my_address")]
    pub share_my_address: bool,

    /// My platform
    /// NRCS: myPlatform
    #[serde(default = "default_my_platform")]
    pub my_platform: String,

    /// My hallmark
    /// NRCS: myHallmark
    #[serde(default)]
    pub my_hallmark: Option<String>,

    // ============ 守护进程间隔配置 ============
    /// Connection daemon interval in seconds
    /// NRCS: peerConnectingThread interval = 5
    #[serde(default = "default_connection_daemon_interval_secs")]
    pub connection_daemon_interval_secs: u64,

    /// Discovery daemon interval in seconds
    /// NRCS: getMorePeersThread interval = 30
    #[serde(default = "default_discovery_daemon_interval_secs")]
    pub discovery_daemon_interval_secs: u64,

    /// Unblacklist daemon interval in seconds
    /// NRCS: peerUnBlacklistingThread interval = 60
    #[serde(default = "default_unblacklist_daemon_interval_secs")]
    pub unblacklist_daemon_interval_secs: u64,

    /// Transaction broadcast interval in seconds
    /// NRCS: sendTransactionsThread interval = 30
    #[serde(default = "default_transaction_daemon_interval_secs")]
    pub transaction_daemon_interval_secs: u64,

    /// Send transactions batch size
    /// NRCS: sendTransactionsBatchSize = 10
    #[serde(default = "default_send_transactions_batch_size")]
    pub send_transactions_batch_size: usize,

    /// Bundler rate broadcast interval in seconds
    /// NRCS: BUNDLER_RATE_BROADCAST_INTERVAL = 30 * 60
    #[serde(default = "default_bundler_rate_broadcast_interval_secs")]
    pub bundler_rate_broadcast_interval_secs: u64,
}

// ============ 默认值函数（引用全局常量） ============

fn default_min_compress_size() -> usize { MIN_COMPRESS_SIZE }
fn default_max_connections() -> usize { MAX_CONNECTIONS }
fn default_max_inbound_connections() -> usize { MAX_INBOUND_CONNECTIONS }
fn default_max_outbound_connections() -> usize { MAX_OUTBOUND_CONNECTIONS }
fn default_connect_timeout_ms() -> u64 { CONNECT_TIMEOUT_MS }
fn default_read_timeout_ms() -> u64 { READ_TIMEOUT_MS }
fn default_websocket_idle_timeout_secs() -> u64 { WEBSOCKET_IDLE_TIMEOUT_SECS }
fn default_max_request_size() -> usize { MAX_REQUEST_SIZE }
fn default_max_response_size() -> usize { MAX_RESPONSE_SIZE }
fn default_max_message_size() -> usize { MAX_MESSAGE_SIZE }
fn default_blacklisting_period_secs() -> i64 { BLACKLISTING_PERIOD_SECS }
fn default_blacklisting_enabled() -> bool { true }
fn default_blacklisting_threshold() -> i32 { BLACKLISTING_THRESHOLD }
fn default_get_more_peers() -> bool { true }
fn default_max_known_peers() -> usize { MAX_KNOWN_PEERS }
fn default_min_known_peers() -> usize { MIN_KNOWN_PEERS }
fn default_use_peers_db() -> bool { true }
fn default_save_peers() -> bool { true }
fn default_use_websockets() -> bool { true }
fn default_enable_hallmark_protection() -> bool { true }
fn default_push_threshold() -> i32 { 0 }
fn default_pull_threshold() -> i32 { 0 }
fn default_send_to_peers_limit() -> usize { 10 }
fn default_max_version_length() -> usize { MAX_VERSION_LENGTH }
fn default_max_application_length() -> usize { MAX_APPLICATION_LENGTH }
fn default_max_platform_length() -> usize { MAX_PLATFORM_LENGTH }
fn default_max_announced_address_length() -> usize { MAX_ANNOUNCED_ADDRESS_LENGTH }
fn default_listen_addr() -> SocketAddr { "0.0.0.0:16974".parse().unwrap() }
fn default_share_my_address() -> bool { true }
fn default_my_platform() -> String { 
    format!("{} {}", 
        std::env::consts::OS, 
        std::env::consts::ARCH
    )
}
fn default_connection_daemon_interval_secs() -> u64 { CONNECTION_DAEMON_INTERVAL_SECS }
fn default_discovery_daemon_interval_secs() -> u64 { DISCOVERY_DAEMON_INTERVAL_SECS }
fn default_unblacklist_daemon_interval_secs() -> u64 { UNBLACKLIST_DAEMON_INTERVAL_SECS }
fn default_transaction_daemon_interval_secs() -> u64 { TRANSACTION_DAEMON_INTERVAL_SECS }
fn default_send_transactions_batch_size() -> usize { SEND_TRANSACTIONS_BATCH_SIZE }
fn default_bundler_rate_broadcast_interval_secs() -> u64 { BUNDLER_RATE_BROADCAST_INTERVAL_SECS }

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            min_compress_size: default_min_compress_size(),
            max_connections: default_max_connections(),
            max_inbound_connections: default_max_inbound_connections(),
            max_outbound_connections: default_max_outbound_connections(),
            connect_timeout_ms: default_connect_timeout_ms(),
            read_timeout_ms: default_read_timeout_ms(),
            websocket_idle_timeout_secs: default_websocket_idle_timeout_secs(),
            max_request_size: default_max_request_size(),
            max_response_size: default_max_response_size(),
            max_message_size: default_max_message_size(),
            blacklisting_period_secs: default_blacklisting_period_secs(),
            blacklisting_enabled: default_blacklisting_enabled(),
            blacklisting_threshold: default_blacklisting_threshold(),
            get_more_peers: default_get_more_peers(),
            max_known_peers: default_max_known_peers(),
            min_known_peers: default_min_known_peers(),
            use_peers_db: default_use_peers_db(),
            save_peers: default_save_peers(),
            use_websockets: default_use_websockets(),
            enable_hallmark_protection: default_enable_hallmark_protection(),
            push_threshold: default_push_threshold(),
            pull_threshold: default_pull_threshold(),
            send_to_peers_limit: default_send_to_peers_limit(),
            max_version_length: default_max_version_length(),
            max_application_length: default_max_application_length(),
            max_platform_length: default_max_platform_length(),
            max_announced_address_length: default_max_announced_address_length(),
            listen_addr: default_listen_addr(),
            my_address: None,
            share_my_address: default_share_my_address(),
            my_platform: default_my_platform(),
            my_hallmark: None,
            connection_daemon_interval_secs: default_connection_daemon_interval_secs(),
            discovery_daemon_interval_secs: default_discovery_daemon_interval_secs(),
            unblacklist_daemon_interval_secs: default_unblacklist_daemon_interval_secs(),
            transaction_daemon_interval_secs: default_transaction_daemon_interval_secs(),
            send_transactions_batch_size: default_send_transactions_batch_size(),
            bundler_rate_broadcast_interval_secs: default_bundler_rate_broadcast_interval_secs(),
        }
    }
}

impl P2PConfig {
    /// Load configuration from file
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: P2PConfig = if path.ends_with(".toml") {
            toml::from_str(&content)?
        } else if path.ends_with(".json") {
            serde_json::from_str(&content)?
        } else if path.ends_with(".yaml") || path.ends_with(".yml") {
            serde_yaml::from_str(&content)?
        } else {
            return Err("Unsupported config file format".into());
        };
        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_connections == 0 {
            return Err("max_connections must be greater than 0".to_string());
        }
        if self.max_inbound_connections == 0 {
            return Err("max_inbound_connections must be greater than 0".to_string());
        }
        if self.max_outbound_connections == 0 {
            return Err("max_outbound_connections must be greater than 0".to_string());
        }
        if self.connect_timeout_ms == 0 {
            return Err("connect_timeout_ms must be greater than 0".to_string());
        }
        if self.read_timeout_ms == 0 {
            return Err("read_timeout_ms must be greater than 0".to_string());
        }
        if self.blacklisting_period_secs < 0 {
            return Err("blacklisting_period_secs must be non-negative".to_string());
        }
        if self.max_known_peers < self.min_known_peers {
            return Err("max_known_peers must be >= min_known_peers".to_string());
        }
        Ok(())
    }

    /// Check if we have enough connected peers
    pub fn has_enough_connected_peers(&self, count: usize) -> bool {
        count >= self.max_connections
    }

    /// Check if we have too many known peers
    pub fn too_many_known_peers(&self, count: usize) -> bool {
        count >= self.max_known_peers
    }

    /// Check if we have too few known peers
    pub fn too_few_known_peers(&self, count: usize) -> bool {
        count < self.min_known_peers
    }

    /// Check if we have too many outbound connections
    pub fn too_many_outbound_connections(&self, count: usize) -> bool {
        count >= self.max_outbound_connections
    }

    /// Check if we have too many inbound connections
    pub fn too_many_inbound_connections(&self, count: usize) -> bool {
        count >= self.max_inbound_connections
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = P2PConfig::default();
        assert_eq!(config.min_compress_size, 256);
        assert_eq!(config.max_request_size, 64 * 1024 * 1024);
        assert_eq!(config.max_response_size, 64 * 1024 * 1024);
        assert_eq!(config.max_message_size, 40 * 1024 * 1024);
        assert_eq!(config.max_version_length, 10);
        assert_eq!(config.max_application_length, 20);
        assert_eq!(config.max_platform_length, 30);
        assert_eq!(config.max_announced_address_length, 100);
    }

    #[test]
    fn test_config_validation() {
        let config = P2PConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_peer_count_checks() {
        let config = P2PConfig::default();
        
        assert!(config.has_enough_connected_peers(20));
        assert!(!config.has_enough_connected_peers(10));
        
        assert!(config.too_many_known_peers(3000));
        assert!(!config.too_many_known_peers(100));
        
        assert!(config.too_few_known_peers(50));
        assert!(!config.too_few_known_peers(150));
    }
}
