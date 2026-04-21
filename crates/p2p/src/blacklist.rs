//! Blacklist Manager (黑名单管理器)
//!
//! 对应 NRCS Java: Peers.java 中的黑名单相关方法
//!
//! 职责:
//! - 管理黑名单
//! - 管理白名单
//! - 持久化支持

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Blacklist entry
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BlacklistEntry {
    /// Node address
    pub address: SocketAddr,
    /// Blacklist reason
    pub reason: String,
    /// Blacklist timestamp
    pub timestamp: i64,
    /// Expiration time (None means permanent)
    pub expires_at: Option<i64>,
    /// Is manual blacklist
    pub is_manual: bool,
}

/// Whitelist entry
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WhitelistEntry {
    /// Account ID
    pub account_id: u64,
    /// Added timestamp
    pub added_at: i64,
    /// Is manual
    pub is_manual: bool,
}

/// Blacklist Manager
pub struct BlacklistManager {
    /// Blacklist entries
    blacklist: Arc<RwLock<HashMap<SocketAddr, BlacklistEntry>>>,
    /// Whitelist entries (account ID based)
    whitelist: Arc<RwLock<HashMap<u64, WhitelistEntry>>>,
    /// Known blacklisted peers (from config)
    known_blacklisted: Arc<RwLock<HashSet<SocketAddr>>>,
}

impl BlacklistManager {
    /// Create a new blacklist manager
    pub fn new() -> Self {
        Self {
            blacklist: Arc::new(RwLock::new(HashMap::new())),
            whitelist: Arc::new(RwLock::new(HashMap::new())),
            known_blacklisted: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Add to blacklist
    /// 
    /// 对应 NRCS Java: Peers.blacklist(Peer peer, String cause)
    pub async fn add_to_blacklist(
        &self,
        address: SocketAddr,
        reason: String,
        duration_secs: Option<i64>,
        is_manual: bool,
    ) {
        let now = current_timestamp();
        let expires_at = duration_secs.map(|d| now + d);

        let entry = BlacklistEntry {
            address,
            reason: reason.clone(),
            timestamp: now,
            expires_at,
            is_manual,
        };

        let mut blacklist = self.blacklist.write().await;
        blacklist.insert(address, entry);

        info!("Peer {} blacklisted: {} (manual: {})", address, reason, is_manual);
    }

    /// Remove from blacklist
    /// 
    /// 对应 NRCS Java: Peer.unBlacklist()
    pub async fn remove_from_blacklist(&self, address: &SocketAddr) {
        let mut blacklist = self.blacklist.write().await;
        if blacklist.remove(address).is_some() {
            debug!("Peer {} removed from blacklist", address);
        }
    }

    /// Check if address is blacklisted
    pub async fn is_blacklisted(&self, address: &SocketAddr) -> bool {
        let blacklist = self.blacklist.read().await;
        if let Some(entry) = blacklist.get(address) {
            // Check if expired
            if let Some(expires_at) = entry.expires_at {
                let now = current_timestamp();
                if now >= expires_at {
                    return false;
                }
            }
            return true;
        }
        false
    }

    /// Get blacklist entry
    pub async fn get_blacklist_entry(&self, address: &SocketAddr) -> Option<BlacklistEntry> {
        let blacklist = self.blacklist.read().await;
        blacklist.get(address).cloned()
    }

    /// Get all blacklisted addresses
    pub async fn get_all_blacklisted(&self) -> Vec<SocketAddr> {
        let blacklist = self.blacklist.read().await;
        blacklist.keys().cloned().collect()
    }

    /// Clean expired entries
    /// 
    /// 对应 NRCS Java: peerUnBlacklistingThread
    pub async fn clean_expired(&self) {
        let now = current_timestamp();
        let mut blacklist = self.blacklist.write().await;
        
        let expired: Vec<_> = blacklist
            .iter()
            .filter_map(|(addr, entry)| {
                if let Some(expires_at) = entry.expires_at {
                    if now >= expires_at {
                        return Some(*addr);
                    }
                }
                None
            })
            .collect();

        for addr in expired {
            blacklist.remove(&addr);
            debug!("Peer {} blacklist expired", addr);
        }
    }

    /// Add to whitelist
    pub async fn add_to_whitelist(&self, account_id: u64, is_manual: bool) {
        let entry = WhitelistEntry {
            account_id,
            added_at: current_timestamp(),
            is_manual,
        };

        let mut whitelist = self.whitelist.write().await;
        whitelist.insert(account_id, entry);

        info!("Account {} added to whitelist (manual: {})", account_id, is_manual);
    }

    /// Remove from whitelist
    pub async fn remove_from_whitelist(&self, account_id: u64) {
        let mut whitelist = self.whitelist.write().await;
        if whitelist.remove(&account_id).is_some() {
            debug!("Account {} removed from whitelist", account_id);
        }
    }

    /// Check if account is whitelisted
    pub async fn is_whitelisted(&self, account_id: u64) -> bool {
        let whitelist = self.whitelist.read().await;
        whitelist.contains_key(&account_id)
    }

    /// Add known blacklisted peer
    /// 
    /// 对应 NRCS Java: Peers.knownBlacklistedPeers
    pub async fn add_known_blacklisted(&self, address: SocketAddr) {
        let mut known = self.known_blacklisted.write().await;
        known.insert(address);
    }

    /// Check if address is known blacklisted
    pub async fn is_known_blacklisted(&self, address: &SocketAddr) -> bool {
        let known = self.known_blacklisted.read().await;
        known.contains(address)
    }

    /// Get blacklist count
    pub async fn blacklist_count(&self) -> usize {
        self.blacklist.read().await.len()
    }

    /// Get whitelist count
    pub async fn whitelist_count(&self) -> usize {
        self.whitelist.read().await.len()
    }

    /// Persist blacklist to file
    pub async fn persist(&self, path: &str) -> std::io::Result<()> {
        let blacklist = self.blacklist.read().await;
        let entries: Vec<_> = blacklist.values().collect();
        
        let json = serde_json::to_string_pretty(&entries)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        std::fs::write(path, json)?;
        debug!("Blacklist persisted to {}", path);
        Ok(())
    }

    /// Load blacklist from file
    pub async fn load(&self, path: &str) -> std::io::Result<()> {
        let content = std::fs::read_to_string(path)?;
        
        let entries: Vec<BlacklistEntry> = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        let mut blacklist = self.blacklist.write().await;
        for entry in entries {
            blacklist.insert(entry.address, entry);
        }
        
        info!("Loaded {} blacklist entries from {}", blacklist.len(), path);
        Ok(())
    }
}

impl Default for BlacklistManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current timestamp in seconds
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blacklist_add_remove() {
        let manager = BlacklistManager::new();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        manager.add_to_blacklist(addr, "Test".to_string(), None, false).await;
        assert!(manager.is_blacklisted(&addr).await);

        manager.remove_from_blacklist(&addr).await;
        assert!(!manager.is_blacklisted(&addr).await);
    }

    #[tokio::test]
    async fn test_blacklist_expiration() {
        let manager = BlacklistManager::new();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        // Add with 1 second expiration
        manager.add_to_blacklist(addr, "Test".to_string(), Some(1), false).await;
        assert!(manager.is_blacklisted(&addr).await);

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Clean expired
        manager.clean_expired().await;
        assert!(!manager.is_blacklisted(&addr).await);
    }

    #[tokio::test]
    async fn test_whitelist() {
        let manager = BlacklistManager::new();
        let account_id = 12345u64;

        manager.add_to_whitelist(account_id, true).await;
        assert!(manager.is_whitelisted(account_id).await);

        manager.remove_from_whitelist(account_id).await;
        assert!(!manager.is_whitelisted(account_id).await);
    }
}
