//! P2P 广播机制
//!
//! 对应 Java NRCS: Peers.java 中的广播方法
//!
//! 功能:
//! - sendToSomePeers: 加权随机选择节点发送
//! - broadcastBlock: 异步广播区块
//! - broadcastTransaction: 批量广播交易
//! - Hallmark 权重过滤

use crate::config::P2PConfig;
use crate::peer::{Peer, PeerState};
use crate::protocol::PeerRequest;
use serde_json::{self, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// 广播结果
#[derive(Debug)]
pub struct BroadcastResult {
    /// 目标地址
    pub addr: SocketAddr,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果失败）
    pub error: Option<String>,
}

/// P2P 广播管理器
///
/// 对应 Java NRCS: Peers.java (sendToSomePeers, broadcastBlock, broadcastTransaction)
pub struct BroadcastManager {
    /// 配置引用
    config: Arc<P2PConfig>,
}

impl BroadcastManager {
    /// 创建新的广播管理器
    pub fn new(config: Arc<P2PConfig>) -> Self {
        Self { config }
    }

    /// 向部分节点发送请求（加权随机选择）
    ///
    /// 对应 Java: Peers.sendToSomePeers(JSONObject request)
    ///
    /// 算法:
    /// 1. 获取所有已连接的公共节点
    /// 2. 应用 Hallmark 保护过滤（如果启用）
    /// 3. 根据权重随机选择指定数量的节点
    /// 4. 并发发送请求
    pub async fn send_to_some_peers(
        &self,
        request: &PeerRequest,
        peers: &Arc<crate::peer::Peers>,
    ) -> Vec<BroadcastResult> {
        // 1. 获取符合条件的公共节点列表
        let candidate_peers = peers.get_public_peers(PeerState::Connected).await;

        // 2. 应用 Hallmark 保护（如果启用）
        let filtered_peers = if self.config.enable_hallmark_protection {
            Self::apply_hallmark_filter(&candidate_peers, self.config.push_threshold as u64)
        } else {
            candidate_peers
        };

        // 3. 限制发送数量
        let send_limit = self.config.send_to_peers_limit.min(filtered_peers.len());
        let selected_peers = Self::weighted_random_select(&filtered_peers, send_limit);

        debug!("[Broadcast] Sending to {} of {} available peers",
               selected_peers.len(), filtered_peers.len());

        // 4. 并发发送
        let mut results = Vec::with_capacity(selected_peers.len());
        let mut handles = Vec::new();

        for peer in selected_peers {
            let peers_clone = Arc::clone(peers);
            let request_clone = request.clone();
            let config_clone = Arc::clone(&self.config);
            let addr = peer.address;

            handles.push(tokio::spawn(async move {
                if let Some(peer_ref) = peers_clone.get_peer(&addr).await {
                    let p = peer_ref.lock().await;
                    match p.send(&request_clone, &config_clone).await {
                        Ok(_response) => {
                            debug!("[Broadcast] Success to {}", addr);
                            BroadcastResult { addr, success: true, error: None }
                        }
                        Err(e) => {
                            warn!("[Broadcast] Failed to {}: {}", addr, e);
                            BroadcastResult { addr, success: false, error: Some(e.to_string()) }
                        }
                    }
                } else {
                    BroadcastResult { addr, success: false, error: Some("Peer not found".to_string()) }
                }
            }));
        }

        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("[Broadcast] Task failed: {}", e);
                }
            }
        }

        let success_count = results.iter().filter(|r| r.success).count();
        info!("[Broadcast] Completed: {}/{} successful", success_count, results.len());

        results
    }

    /// 广播新区块给其他节点
    ///
    /// 对应 Java: Peers.broadcastBlock(IBlock block)
    ///
    /// 特点:
    /// - 异步非阻塞，不等待所有响应完成
    /// - 自动构建 processBlock 请求格式
    pub async fn broadcast_block(
        &self,
        block_json: &Value,
        previous_block_id: u64,
        timestamp: i64,
        peers: &Arc<crate::peer::Peers>,
    ) {
        info!("[Broadcast] Broadcasting block (prev={}, ts={})", previous_block_id, timestamp);

        // 构建 processBlock 请求
        let mut request = PeerRequest::new(crate::protocol::RequestType::ProcessBlock, 1);
        request.set("previousBlock", previous_block_id.to_string());
        request.set("block", block_json.clone());
        request.set("timestamp", timestamp.to_string());

        // 异步广播（不阻塞调用方）
        let manager = Self::new(Arc::clone(&self.config));
        let peers_clone = Arc::clone(peers);

        tokio::spawn(async move {
            let results = manager.send_to_some_peers(&request, &peers_clone).await;
            let success_count = results.iter().filter(|r| r.success).count();
            if success_count > 0 {
                info!("[Broadcast] Block broadcast completed: {}/{} successful",
                      success_count, results.len());
            } else {
                warn!("[Broadcast] Block broadcast: all {} requests failed", results.len());
            }

            // 统计失败的节点，可能需要黑名单处理
            for result in &results {
                if !result.success {
                    debug!("[Broadcast] Block broadcast failed to {}: {:?}",
                           result.addr, result.error);
                }
            }
        });
    }

    /// 批量广播交易给其他节点
    ///
    /// 对应 Java: Peers.broadcastTransaction(ITransaction transaction)
    ///
    /// 特点:
    /// - 支持批量发送多个交易
    /// - 自动构建 processTransactions 请求格式
    /// - 异步非阻塞
    pub async fn broadcast_transactions(
        &self,
        transactions_json: &[Value],
        peers: &Arc<crate::peer::Peers>,
    ) {
        if transactions_json.is_empty() {
            return;
        }

        info!("[Broadcast] Broadcasting {} transactions", transactions_json.len());

        // 构建 processTransactions 请求
        let mut request = PeerRequest::new(crate::protocol::RequestType::ProcessTransactions, 1);
        request.set("transactions", transactions_json.to_vec());

        // 异步广播
        let manager = Self::new(Arc::clone(&self.config));
        let peers_clone = Arc::clone(peers);

        tokio::spawn(async move {
            let results = manager.send_to_some_peers(&request, &peers_clone).await;
            let success_count = results.iter().filter(|r| r.success).count();
            debug!("[Broadcast] Transaction broadcast completed: {}/{} successful",
                   success_count, results.len());
        });
    }

    /// 应用 Hallmark 权重过滤
    ///
    /// 对应 Java: Peers.applyHallmarkProtection()
    fn apply_hallmark_filter(peers: &[Peer], threshold: u64) -> Vec<Peer> {
        if threshold == 0 {
            return peers.to_vec(); // 阈值为 0 表示不过滤
        }

        peers.iter()
            .filter(|p| p.get_weight() >= threshold)
            .cloned()
            .collect()
    }

    /// 加权随机选择节点
    ///
    /// 基于节点的服务标志和流量统计计算权重，
    /// 使用轮盘赌算法进行随机选择
    fn weighted_random_select(peers: &[Peer], count: usize) -> Vec<Peer> {
        if peers.is_empty() || count == 0 {
            return vec![];
        }

        if peers.len() <= count {
            return peers.to_vec();
        }

        // 计算总权重
        let total_weight: u64 = peers.iter()
            .map(|p| p.get_weight())
            .sum();

        if total_weight == 0 {
            // 所有权重为 0 时，均匀随机选择
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            let mut shuffled = peers.to_vec();
            shuffled.shuffle(&mut rng);
            return shuffled.into_iter().take(count).collect();
        }

        // 轮盘赌算法选择
        let mut selected = Vec::with_capacity(count);
        let mut remaining = peers.to_vec();

        while selected.len() < count && !remaining.is_empty() {
            let random_value = rand::random::<u64>() % total_weight;
            let mut cumulative_weight = 0u64;
            let mut found_index = None;

            for (i, peer) in remaining.iter().enumerate() {
                cumulative_weight += peer.get_weight();
                if cumulative_weight > random_value {
                    found_index = Some(i);
                    break;
                }
            }

            if let Some(index) = found_index {
                selected.push(remaining.remove(index));
            } else {
                // 兜底：取最后一个
                if let Some(last) = remaining.pop() {
                    selected.push(last);
                }
            }
        }

        selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weighted_random_select() {
        let peers = vec![
            create_test_peer("127.0.0.1:8081".parse().unwrap(), 100),
            create_test_peer("127.0.0.1:8082".parse().unwrap(), 200),
            create_test_peer("127.0.0.1:8083".parse().unwrap(), 300),
            create_test_peer("127.0.0.1:8084".parse().unwrap(), 400),
            create_test_peer("127.0.0.1:8085".parse().unwrap(), 500),
        ];

        let selected = BroadcastManager::weighted_random_select(&peers, 3);
        assert_eq!(selected.len(), 3);
    }

    #[test]
    fn test_apply_hallmark_filter() {
        let peers = vec![
            create_test_peer("127.0.0.1:8081".parse().unwrap(), 10),
            create_test_peer("127.0.0.1:8082".parse().unwrap(), 50),
            create_test_peer("127.0.0.1:8083".parse().unwrap(), 100),
        ];

        // 阈值 30 应该过滤掉第一个
        let filtered = BroadcastManager::apply_hallmark_filter(&peers, 30);
        assert_eq!(filtered.len(), 2);

        // 阈值 0 不应过滤任何
        let all = BroadcastManager::apply_hallmark_filter(&peers, 0);
        assert_eq!(all.len(), 3);
    }

    fn create_test_peer(addr: SocketAddr, weight: u64) -> Peer {
        let mut peer = Peer::new(addr, false);
        peer.downloaded_volume = weight * 1024 * 1024; // 模拟下载流量影响权重
        peer
    }
}
