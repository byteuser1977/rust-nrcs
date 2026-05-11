//! BundlerRate 广播守护进程
//!
//! 对应 NRCS Java: bundlerRateBroadcastThread (30分钟间隔)
//!
//! 职责:
//! - 定期广播当前节点的 BundlerRate 信息给其他节点
//! - 检测 rates 变化并触发广播
//! - 使用 sendToSomePeers 机制进行广播

use crate::config::P2PConfig;
use crate::peer::{PeerState, Peers};
use crate::protocol::{PeerRequest, RequestType};
use serde_json;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// BundlerRate 广播守护进程
///
/// 对应 NRCS Java: bundlerRateBroadcastThread
pub struct BundlerRateDaemon {
    peers: Arc<Peers>,
    config: P2PConfig,
    running: Arc<RwLock<bool>>,
    /// 上次广播时间
    last_broadcast_time: Arc<RwLock<i64>>,
    /// 上次 rates 的 hash 值（用于检测变化）
    last_rates_hash: Arc<RwLock<u64>>,
}

impl BundlerRateDaemon {
    /// 创建新的 BundlerRate 守护进程
    pub fn new(peers: Arc<Peers>, config: P2PConfig) -> Self {
        Self {
            peers,
            config,
            running: Arc::new(RwLock::new(false)),
            last_broadcast_time: Arc::new(RwLock::new(0)),
            last_rates_hash: Arc::new(RwLock::new(0)),
        }
    }

    /// 启动守护进程
    pub async fn start(&self) {
        let mut running = self.running.write().await;
        if *running {
            warn!("BundlerRate daemon is already running");
            return;
        }
        *running = true;
        drop(running);

        info!("[BundlerRate] Daemon started (interval: {}s)",
              self.config.bundler_rate_broadcast_interval_secs);

        let peers = Arc::clone(&self.peers);
        let config = self.config.clone();
        let running = Arc::clone(&self.running);
        let last_time = Arc::clone(&self.last_broadcast_time);
        let last_hash = Arc::clone(&self.last_rates_hash);

        tokio::spawn(async move {
            loop {
                if !*running.read().await {
                    break;
                }

                tokio::time::sleep(Duration::from_secs(
                    config.bundler_rate_broadcast_interval_secs
                )).await;

                if let Err(e) = Self::broadcast_loop(
                    &peers,
                    &config,
                    &last_time,
                    &last_hash,
                ).await {
                    warn!("[BundlerRate] Broadcast loop error: {}", e);
                }
            }
            info!("[BundlerRate] Daemon stopped");
        });
    }

    /// 停止守护进程
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("[BundlerRate] Daemon stopping...");
    }

    /// 主广播循环
    ///
    /// 对应 NRCS Java: bundlerRateBroadcastThread.run()
    async fn broadcast_loop(
        peers: &Arc<Peers>,
        config: &P2PConfig,
        last_time: &Arc<RwLock<i64>>,
        last_hash: &Arc<RwLock<u64>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::websocket::WebsocketClient;

        let now = current_timestamp();
        let prev_time = *last_time.read().await;

        // 检查是否到了广播时间（默认 30 分钟）
        if now - prev_time < config.bundler_rate_broadcast_interval_secs as i64 {
            return Ok(());
        }

        // 获取当前的 bundler rates
        // 实际实现中应该从区块链状态或配置获取
        let current_rates = Self::get_current_rates(peers).await?;

        if current_rates.is_null() {
            debug!("[BundlerRate] No rates to broadcast");
            return Ok(());
        }

        // 计算 rates 的 hash 值，用于检测变化
        let rates_hash = calculate_hash(&current_rates);
        let prev_hash = *last_hash.read().await;

        // 检查是否有变化
        if rates_hash == prev_hash && prev_time > 0 {
            debug!("[BundlerRate] Rates unchanged, skipping broadcast");
            return Ok(());
        }

        // 执行广播
        debug!("[BundlerRate] Broadcasting rates (hash={:#x})", rates_hash);

        // 构建 bundlerRate 请求
        let mut request = PeerRequest::new(RequestType::BundlerRate, 2); // protocol=2
        request.set("rates", current_rates.clone());

        // 获取已连接的公共节点
        let connected_peers = peers.get_public_peers(PeerState::Connected).await;

        if connected_peers.is_empty() {
            debug!("[BundlerRate] No connected peers for broadcast");
            return Ok(());
        }

        // 选择部分节点发送（使用 sendToPeersLimit）
        let send_limit = config.send_to_peers_limit.min(connected_peers.len());
        let mut success_count = 0usize;

        for peer in connected_peers.iter().take(send_limit) {
            let peer_addr = peer.address;
            let req_clone = request.clone();

            tokio::spawn(async move {
                match WebsocketClient::send_request(peer_addr, req_clone).await {
                    Ok(_) => {
                        debug!("[BundlerRate] Successfully sent to {}", peer_addr);
                    }
                    Err(e) => {
                        debug!("[BundlerRate] Failed to send to {}: {}", peer_addr, e);
                    }
                }
            });

            success_count += 1;
        }

        // 更新时间和 hash
        *last_time.write().await = now;
        *last_hash.write().await = rates_hash;

        debug!("[BundlerRate] Broadcast completed to {} peers", success_count);

        Ok(())
    }

    /// 获取当前的 bundler rates
    ///
    /// 对应 NRCS Java: 从区块链或配置获取当前的 bundler rate 信息
    async fn get_current_rates(_peers: &Arc<Peers>) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // 对照 Java NRCS: 从区块链状态获取 bundler rate 信息
        // 数据来源优先级:
        // 1. 区块链状态（Hub 表中的 minFeePerByte 等）
        // 2. 配置文件中的默认值
        // 3. 运行时动态计算的值
        //
        // 注意: 完整实现需要集成 ORM 的 HubRepository，
        // 当前返回默认值，待上层注入 Repository 后替换
        Ok(serde_json::json!({
            "minFeePerByte": 0,
            "bundlers": []
        }))
    }

    /// 手动触发广播（用于 rates 变化时立即广播）
    pub async fn force_broadcast(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = current_timestamp();

        // 重置上次广播时间，强制下次循环执行广播
        *self.last_broadcast_time.write().await = 0;

        // 重置 hash，确保即使内容相同也会广播
        *self.last_rates_hash.write().await = 0;

        debug!("[BundlerRate] Force broadcast triggered at {}", now);

        Ok(())
    }
}

/// 计算 JSON 值的 hash（用于检测变化）
fn calculate_hash(value: &serde_json::Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
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
    use serde_json::json;

    #[test]
    fn test_calculate_hash() {
        let val1 = json!({"a": 1, "b": 2});
        let val2 = json!({"a": 1, "b": 2});
        let val3 = json!({"a": 1, "b": 3});

        assert_eq!(calculate_hash(&val1), calculate_hash(&val2));
        assert_ne!(calculate_hash(&val1), calculate_hash(&val3));
    }

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts > 1700000000); // 2023 年以后的时间戳
    }

    #[tokio::test]
    async fn test_bundler_rate_daemon_creation() {
        let addr: std::net::SocketAddr = "127.0.0.1:17974".parse().unwrap();
        let my_peer = Peer::new(addr, false);
        let peers = Arc::new(Peers::new(my_peer));
        let config = P2PConfig::default();

        let daemon = BundlerRateDaemon::new(Arc::clone(&peers), config);
        assert!(!*daemon.running.read().await);
    }

    #[tokio::test]
    async fn test_bundler_rate_daemon_start_stop() {
        let addr: std::net::SocketAddr = "127.0.0.1:17974".parse().unwrap();
        let my_peer = Peer::new(addr, false);
        let peers = Arc::new(Peers::new(my_peer));
        let config = P2PConfig::default();

        let daemon = BundlerRateDaemon::new(Arc::clone(&peers), config);

        daemon.start().await;
        assert!(*daemon.running.read().await);

        daemon.stop().await;
        // 给一点时间让后台任务停止
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
