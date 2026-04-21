//! Unblacklist Daemon (解除黑名单守护进程)
//!
//! 对应 NRCS Java: peerUnBlacklistingThread (60秒间隔)
//!
//! 职责:
//! - 检查黑名单是否过期
//! - 解除过期黑名单
//! - 重置旧版本标记

use crate::config::P2PConfig;
use crate::peer::{PeerState, Peers};
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

        info!("Unblacklist daemon started (interval: {}s)", self.config.unblacklist_daemon_interval_secs);
        
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
                    warn!("Unblacklist loop error: {}", e);
                }
            }
            info!("Unblacklist daemon stopped");
        });
    }

    /// Stop the unblacklist daemon
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Unblacklist daemon stopping...");
    }

    /// Main unblacklist loop
    /// 
    /// 对应 NRCS Java: peerUnBlacklistingThread.run()
    async fn unblacklist_loop(peers: &Arc<Peers>, config: &P2PConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = current_timestamp();

        // 遍历所有节点，检查黑名单状态
        let all_peers = peers.get_known_peers().await;
        
        for peer in all_peers {
            // 检查黑名单是否过期
            // 对应 NRCS Java: peer.updateBlacklistedStatus(curTime)
            if peer.blacklisting_time > 0
                && peer.blacklisting_time + config.blacklisting_period_secs <= now {
                debug!("Unblacklisting peer: {}", peer.address);
                // TODO: 实现解除黑名单
                // peer.un_blacklist();
            }

            // 检查是否是旧版本，如果是且最后更新超过1小时，则重置
            if peer.is_old_version && peer.last_updated < now - 3600 {
                debug!("Resetting old version flag for peer: {}", peer.address);
                // TODO: 重置旧版本标记
                // peer.is_old_version = false;
            }
        }

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

    #[test]
    fn test_unblacklist_daemon_creation() {
        // TODO: 添加测试
    }
}
