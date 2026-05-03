//! Unblacklist Daemon (解除黑名单守护进程)
//!
//! 对应 NRCS Java: peerUnBlacklistingThread (60秒间隔)
//!
//! 职责:
//! - 检查黑名单是否过期
//! - 解除过期黑名单
//! - 重置旧版本标记

use crate::config::P2PConfig;
use crate::peer::Peers;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Unblacklist Daemon
///
/// 对应 NRCS Java: peerUnBlacklistingThread
pub struct UnblacklistDaemon {
    peers: Arc<Peers>,
    config: P2PConfig,
    running: Arc<RwLock<bool>>,
}

impl UnblacklistDaemon {
    pub fn new(peers: Arc<Peers>, config: P2PConfig) -> Self {
        Self {
            peers,
            config,
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) {
        let mut running = self.running.write().await;
        if *running {
            warn!("Unblacklist daemon is already running");
            return;
        }
        *running = true;
        drop(running);

        info!("[Unblacklist] Daemon started (interval: {}s)",
              self.config.unblacklist_daemon_interval_secs);

        let peers = Arc::clone(&self.peers);
        let config = self.config.clone();
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            loop {
                if !*running.read().await {
                    break;
                }

                tokio::time::sleep(Duration::from_secs(config.unblacklist_daemon_interval_secs)).await;

                if let Err(e) = Self::unblacklist_loop(&peers, &config).await {
                    warn!("[Unblacklist] Loop error: {}", e);
                }
            }
            info!("[Unblacklist] Daemon stopped");
        });
    }

    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("[Unblacklist] Daemon stopping...");
    }

    /// Main unblacklist loop
    ///
    /// 对应 NRCS Java: peerUnBlacklistingThread.run()
    /// 遍历所有节点，调用 peer.updateBlacklistedStatus(curTime)
    async fn unblacklist_loop(peers: &Arc<Peers>, config: &P2PConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = current_timestamp();
        let all_peers = peers.get_known_peers().await;
        let mut unblacklisted_count = 0usize;
        let mut reset_version_count = 0usize;

        for peer_snapshot in &all_peers {
            if let Some(peer_ref) = peers.get_peer(&peer_snapshot.address).await {
                let mut peer = peer_ref.lock().await;

                let was_blacklisted = peer.is_blacklisted();
                let had_old_version = peer.is_old_version;

                // 调用 Peer.update_blacklisted_status()（对应 Java 逻辑）
                peer.update_blacklisted_status(now, config.blacklisting_period_secs);

                // 统计变化
                if was_blacklisted && !peer.is_blacklisted() {
                    unblacklisted_count += 1;
                    debug!("[Unblacklist] Peer {} unblacklisted", peer.address);
                }
                if had_old_version && !peer.is_old_version {
                    reset_version_count += 1;
                    debug!("[Unblacklist] Reset old version flag for peer {}", peer.address);
                }
            }
        }

        if unblacklisted_count > 0 || reset_version_count > 0 {
            debug!("[Unblacklist] Processed {} peers: unblacklisted={}, reset_version={}",
                  all_peers.len(), unblacklisted_count, reset_version_count);
        }

        Ok(())
    }

    /// Manually unblacklist a peer (public API)
    ///
    /// 手动解除某个节点的黑名单状态
    /// 对应 Java: Peer.unBlacklist()
    pub async fn manual_unblacklist(
        &self,
        addr: &std::net::SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(peer_ref) = self.peers.get_peer(addr).await {
            let mut peer = peer_ref.lock().await;
            peer.un_blacklist();
            debug!("[Unblacklist] Manual unblacklist for {}", addr);
        }
        Ok(())
    }
}

fn current_timestamp() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer::Peer;

    #[tokio::test]
    async fn test_unblacklist_daemon_creation() {
        let addr: std::net::SocketAddr = "127.0.0.1:17974".parse().unwrap();
        let my_peer = Peer::new(addr, false);
        let peers = Arc::new(Peers::new(my_peer));
        let config = P2PConfig::default();

        let daemon = UnblacklistDaemon::new(Arc::clone(&peers), config);
        assert!(!*daemon.running.read().await);
    }

    #[tokio::test]
    async fn test_unblacklist_daemon_start_stop() {
        let addr: std::net::SocketAddr = "127.0.0.1:17974".parse().unwrap();
        let my_peer = Peer::new(addr, false);
        let peers = Arc::new(Peers::new(my_peer));
        let config = P2PConfig::default();

        let daemon = UnblacklistDaemon::new(Arc::clone(&peers), config);

        daemon.start().await;
        assert!(*daemon.running.read().await);

        daemon.stop().await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_unblacklist_loop_expires_blacklist() {
        let addr: std::net::SocketAddr = "127.0.0.1:17974".parse().unwrap();
        let my_peer = Peer::new(addr, false);
        let peers = Arc::new(Peers::new(my_peer));
        let config = P2PConfig::default();

        let test_addr: std::net::SocketAddr = "192.168.1.1:9000".parse().unwrap();
        let test_peer = Peer::new(test_addr, false);
        peers.register_peer(test_peer).await;

        // Set blacklisting_time to past (expired) after registration
        if let Some(peer_ref) = peers.get_peer(&test_addr).await {
            let mut p = peer_ref.lock().await;
            p.blacklisting_time = current_timestamp() - config.blacklisting_period_secs - 1;
            p.blacklisting_cause = Some("Test".to_string());
        }

        assert!(peers.is_blacklisted_addr(&test_addr).await);

        UnblacklistDaemon::unblacklist_loop(&peers, &config).await.unwrap();

        assert!(!peers.is_blacklisted_addr(&test_addr).await);
    }

    #[tokio::test]
    async fn test_unblacklist_loop_resets_old_version() {
        let addr: std::net::SocketAddr = "127.0.0.1:17974".parse().unwrap();
        let my_peer = Peer::new(addr, false);
        let peers = Arc::new(Peers::new(my_peer));
        let config = P2PConfig::default();

        let test_addr: std::net::SocketAddr = "192.168.1.2:9000".parse().unwrap();
        let test_peer = Peer::new(test_addr, false);
        peers.register_peer(test_peer).await;

        // Set is_old_version and old last_updated after registration
        // (register_peer resets last_updated via update_metadata)
        if let Some(peer_ref) = peers.get_peer(&test_addr).await {
            let mut p = peer_ref.lock().await;
            p.is_old_version = true;
            p.last_updated = current_timestamp() - 3601;
        }

        assert!(peers.is_blacklisted_addr(&test_addr).await);

        UnblacklistDaemon::unblacklist_loop(&peers, &config).await.unwrap();

        assert!(!peers.is_blacklisted_addr(&test_addr).await);
    }

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts > 1700000000);
    }
}
