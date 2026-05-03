//! Blacklist Manager (黑名单管理器)
//!
//! 对应 NRCS Java: Peers.java 中的黑名单相关方法
//!
//! 职责:
//! - 黑名单持久化（加载/保存）
//! - 黑名单事件监听
//!
//! 注意: 运行时黑名单管理已集成到 Peers 结构体中
//! - Peer.blacklisting_time / blacklisting_cause: 运行时动态黑名单
//! - Peer.is_old_version: 版本过旧黑名单
//! - Peers.known_blacklisted_peers: 配置文件永久黑名单

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// 黑名单事件类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlacklistEvent {
    Blacklist(SocketAddr, String),
    Unblacklist(SocketAddr),
}

/// 黑名单事件监听器
pub type BlacklistListener = Box<dyn Fn(BlacklistEvent) + Send + Sync>;

/// Blacklist Manager
///
/// 负责黑名单的持久化和事件通知
/// 运行时黑名单管理由 Peers 和 Peer 结构体负责
pub struct BlacklistManager {
    /// 事件监听器
    listeners: Arc<RwLock<Vec<BlacklistListener>>>,
}

impl BlacklistManager {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 添加事件监听器
    pub async fn add_listener(&self, listener: BlacklistListener) {
        let mut listeners = self.listeners.write().await;
        listeners.push(listener);
    }

    /// 通知黑名单事件
    pub async fn notify(&self, event: BlacklistEvent) {
        let listeners = self.listeners.read().await;
        for listener in listeners.iter() {
            listener(event.clone());
        }
    }

    /// 持久化黑名单到文件
    ///
    /// 保存当前所有被黑名单的节点地址到文件
    pub async fn persist_blacklist(
        peers: &crate::peer::Peers,
        path: &str,
    ) -> std::io::Result<()> {
        let all_peers = peers.get_known_peers().await;
        let mut blacklisted_addrs = Vec::new();

        for peer in &all_peers {
            if peer.is_blacklisted() {
                blacklisted_addrs.push(peer.address.to_string());
            }
        }

        let _known_bl_count = peers.known_blacklisted_count().await;
        let known_bl = peers.known_blacklisted_peers_list().await;
        for addr in known_bl {
            if !blacklisted_addrs.contains(&addr) {
                blacklisted_addrs.push(addr);
            }
        }

        let json = serde_json::to_string_pretty(&blacklisted_addrs)
            .map_err(std::io::Error::other)?;

        std::fs::write(path, json)?;
        debug!("Blacklist persisted to {} ({} entries)", path, blacklisted_addrs.len());
        Ok(())
    }

    /// 从文件加载黑名单
    pub async fn load_blacklist(
        peers: &crate::peer::Peers,
        path: &str,
    ) -> std::io::Result<()> {
        let content = std::fs::read_to_string(path)?;

        let entries: Vec<String> = serde_json::from_str(&content)
            .map_err(std::io::Error::other)?;

        for addr_str in &entries {
            peers.add_known_blacklisted(addr_str.clone()).await;
        }

        info!("Loaded {} blacklist entries from {}", entries.len(), path);
        Ok(())
    }
}

impl Default for BlacklistManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer::{Peer, Peers};

    #[tokio::test]
    async fn test_blacklist_persist_and_load() {
        let my_peer = Peer::new("127.0.0.1:8080".parse().unwrap(), false);
        let peers = Peers::new(my_peer);

        peers.add_known_blacklisted("192.168.1.1:9000".to_string()).await;
        peers.add_known_blacklisted("192.168.1.2:9000".to_string()).await;

        let tmp_dir = std::env::temp_dir();
        let path = tmp_dir.join("nrcs_test_blacklist.json");
        let path_str = path.to_str().unwrap();

        BlacklistManager::persist_blacklist(&peers, path_str).await.unwrap();

        let my_peer2 = Peer::new("127.0.0.1:8081".parse().unwrap(), false);
        let peers2 = Peers::new(my_peer2);
        BlacklistManager::load_blacklist(&peers2, path_str).await.unwrap();

        assert!(peers2.is_known_blacklisted("192.168.1.1:9000").await);
        assert!(peers2.is_known_blacklisted("192.168.1.2:9000").await);

        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn test_blacklist_event_notification() {
        let manager = BlacklistManager::new();
        let received = Arc::new(std::sync::Mutex::new(Vec::new()));
        let received_clone = received.clone();

        manager.add_listener(Box::new(move |event| {
            let mut r = received_clone.lock().unwrap();
            r.push(format!("{:?}", event));
        })).await;

        manager.notify(BlacklistEvent::Blacklist(
            "127.0.0.1:8080".parse().unwrap(),
            "Test".to_string(),
        )).await;

        let events = received.lock().unwrap();
        assert_eq!(events.len(), 1);
    }
}
