use crate::{peer::Peers, protocol::PeerRequest};
use serde_json;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// ProcessBlock 处理器
/// 请求：完整的区块 JSON（同 Block.getJSONObject() 结构）
/// 处理：验证区块，然后异步提交到区块链处理器
/// 响应：空 JSON {}
pub struct ProcessBlockHandler {
    peers: Arc<Peers>,
}

impl ProcessBlockHandler {
    pub fn new(peers: Arc<Peers>) -> Self {
        Self { peers }
    }

    pub async fn handle(&self, request: PeerRequest, peers: Arc<Peers>) -> serde_json::Value {
        debug!("Handling ProcessBlock request");

        // 提取区块数据
        // Java 通常发送一个包含 "block" 字段的对象，或者在顶层
        let block_json = match request.get::<serde_json::Value>("block") {
            Some(block) => block,
            None => {
                // 尝试从请求的 extra 字段直接获取（未包装）
                warn!("ProcessBlock request missing 'block' field, checking top-level");
                // 这里需要更深入的 JSON 检查，暂时返回错误
                return serde_json::json!({ "error": "MISSING_BLOCK" });
            }
        };

        // TODO: 区块验证逻辑
        // 1. 验证签名
        // 2. 验证难度目标
        // 3. 验证时间戳
        // 4. 验证前一区块哈希链

        debug!("Received block: timestamp={:?}", block_json.get("timestamp"));

        // TODO: 异步提交到 blockchain_processor.process_peer_block()
        // 需要使用 tokio::spawn 避免阻塞 P2P 处理线程
        let block_clone = block_json.clone();
        let peers_clone = Arc::clone(&peers);
        tokio::spawn(async move {
            info!("Async processing block from peer");
            // TODO: 实际处理逻辑
            // blockchain_processor.process_peer_block(block_clone).await;
            warn!("Block processing not implemented");
        });

        // 立即返回成功（与 Java 行为一致）
        serde_json::json!({})
    }
}