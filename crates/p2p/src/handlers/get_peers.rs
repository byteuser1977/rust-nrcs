use crate::{peer::Peers, protocol::PeerRequest};
use serde_json;
use std::sync::Arc;
use tracing::debug;

/// GetPeers 处理器
/// 响应：返回 peers 列表
/// 格式：{"peers":[{peer info...}]} 或 {"knownPeers":[...]}
pub struct GetPeersHandler {
    peers: Arc<Peers>,
}

impl GetPeersHandler {
    pub fn new(peers: Arc<Peers>) -> Self {
        Self { peers }
    }

    pub async fn handle(&self, _request: PeerRequest, peers: Arc<Peers>) -> serde_json::Value {
        debug!("Handling GetPeers request");

        // TODO: Java 可能有两种格式，需测试确认
        // 暂时返回活跃连接的 peers
        let active_peers = peers.get_active_peers().await;
        let mut peers_list = Vec::new();

        for peer in active_peers {
            peers_list.push(peer.to_peer_info());
        }

        // 与 Java 对齐：Java 可能返回 {"peers": [...]} 或 {"knownPeers": [...]}
        // 参考 Java 代码：return new Peer[]{ ... }
        // 暂时使用 "peers" 字段
        let mut response = serde_json::Map::new();
        response.insert("peers".to_string(), serde_json::Value::Array(peers_list));

        serde_json::Value::Object(response)
    }
}