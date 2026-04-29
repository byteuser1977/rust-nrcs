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
    /// Create a new unblacklist daemon
    pub fn new(peers: Arc<Peers>, config: P2PConfig) -> Self {
        Self {
            peers,
            config,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the unblacklist daemon
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

    /// Stop the unblacklist daemon
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("[Unblacklist] Daemon stopping...");
    }

    /// Main unblacklist loop（完整实现）
    ///
    /// 对应 NRCS Java: peerUnBlacklistingThread.run()
    async fn unblacklist_loop(peers: &Arc<Peers>, config: &P2PConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = current_timestamp();
        let _unblacklisted_count = 0usize;
        let _reset_version_count = 0usize;

        // 遍历所有节点，检查黑名单状态和版本标记
        let all_peers = peers.get_known_peers().await;
        let _total_count = all_peers.len();
        let mut unblacklisted_count = 0usize;
        let mut reset_version_count = 0usize;

        for peer in &all_peers {
            // 1. 检查黑名单是否过期
            // 对应 NRCS Java: peer.updateBlacklistedStatus(curTime)
            if peer.blacklisting_time > 0 {
                let blacklist_duration = now - peer.blacklisting_time;

                // 如果黑名单时间超过配置的周期，则解除黑名单
                if blacklist_duration >= config.blacklisting_period_secs {
                    debug!("[Unblacklist] Unblacklisting peer: {} (blacklisted for {}s)",
                          peer.address, blacklist_duration);

                    match Self::unblacklist_peer(peers, &peer.address).await {
                        Ok(()) => {
                            unblacklisted_count += 1;
                        }
                        Err(e) => {
                            warn!("[Unblacklist] Failed to unblacklist {}: {}", peer.address, e);
                        }
                    }
                }
            }

            // 2. 检查是否是旧版本，如果是且最后更新超过1小时，则重置
            // 对应 NRCS Java: 重置 isOldVersion 标记
            if peer.is_old_version {
                let time_since_update = now - peer.last_updated;

                // 如果超过 3600 秒（1小时）没有更新，重置旧版本标记
                if time_since_update >= 3600 {
                    debug!("[Unblacklist] Resetting old version flag for peer: {}",
                          peer.address);

                    match Self::reset_old_version_flag(peers, &peer.address).await {
                        Ok(()) => {
                            reset_version_count += 1;
                        }
                        Err(e) => {
                            warn!("[Unblacklist] Failed to reset version flag for {}: {}", peer.address, e);
                        }
                    }
                }
            }
        }

        if unblacklisted_count > 0 || reset_version_count > 0 {
            info!("[Unblacklist] Processed {} peers: unblacklisted={}, reset_version={}",
                  all_peers.len(), unblacklisted_count, reset_version_count);
        }

        Ok(())
    }

    /// Unblacklist a peer（完整实现）
    ///
    /// 对应 NRCS Java: peer.unBlacklist()
    async fn unblacklist_peer(
        peers: &Arc<Peers>,
        addr: &std::net::SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 如果节点存在，更新其状态
        if let Some(peer_ref) = peers.get_peer(addr).await {
            let mut peer = peer_ref.lock().await;

            // 重置黑名单相关字段
            peer.blacklisting_time = 0;
            peer.blacklisting_cause = None;
            peer.state = crate::peer::PeerState::NonConnected;

            debug!("[Unblacklist] Peer {} unblacklisted successfully", addr);
        }

        Ok(())
    }

    /// Reset old version flag for a peer（完整实现）
    ///
    /// 对应 NRCS Java: 重置 isOldVersion 标记
    async fn reset_old_version_flag(
        peers: &Arc<Peers>,
        addr: &std::net::SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(peer_ref) = peers.get_peer(addr).await {
            let mut peer = peer_ref.lock().await;

            // 重置旧版本标记
            peer.is_old_version = false;

            debug!("[Unblacklist] Reset old version flag for {}", addr);
        }

        Ok(())
    }

    /// Manually unblacklist a peer (public API)
    ///
    /// 手动解除某个节点的黑名单状态
    pub async fn manual_unblacklist(
        &self,
        addr: &std::net::SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Self::unblacklist_peer(&self.peers, addr).await?;
        info!("[Unblacklist] Manual unblacklist for {}", addr);
        Ok(())
    }
}

/// Get current timestamp in seconds
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

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts > 1700000000); // 2023 年以后的时间戳
    }
}
