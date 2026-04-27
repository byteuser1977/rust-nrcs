use crate::{peer::Peers, protocol::PeerRequest};
use serde_json;
use std::sync::Arc;
use tracing::debug;

/// GetPeers 处理器
/// 响应：返回 peers 列表
/// 格式：{"peers":[{peer info...}]}
pub struct GetPeersHandler {
    #[allow(dead_code)]
    peers: Arc<Peers>,
}

impl GetPeersHandler {
    pub fn new(peers: Arc<Peers>) -> Self {
        Self { peers }
    }

    pub async fn handle(&self, request: PeerRequest, peers: Arc<Peers>) -> serde_json::Value {
        debug!("Handling GetPeers request");

        // 获取所有已知节点
        let mut all_peers = peers.get_known_peers().await;

        // 按最后更新时间降序排序（最新的在前）
        all_peers.sort_by_key(|b| std::cmp::Reverse(b.last_updated));

        // 分页参数：limit (默认 1000，最大 1000), offset (默认 0)
        let limit = if let Some(v) = request.extra.get("limit") {
            v.as_i64().unwrap_or(1000).clamp(1, 1000) as usize
        } else {
            1000
        };
        let offset = if let Some(v) = request.extra.get("offset") {
            v.as_i64().unwrap_or(0).max(0) as usize
        } else {
            0
        };

        let peers_slice = all_peers.into_iter().skip(offset).take(limit);
        let peers_list = peers_slice.map(|p| p.to_peer_info()).collect::<Vec<_>>();

        let mut response = serde_json::Map::new();
        response.insert("peers".to_string(), serde_json::Value::Array(peers_list));
        serde_json::Value::Object(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer::Peer;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_get_peers_returns_limited_list() {
        // 创建 Peers 管理器
        let my_peer = Peer::new("127.0.0.1:8080".parse().unwrap(), false);
        let peers = Arc::new(crate::peer::Peers::new(my_peer));

        // 添加多个测试节点
        for i in 0..10 {
            let addr = format!("127.0.0.1:{}", 9000 + i).parse::<SocketAddr>().unwrap();
            let mut peer = Peer::new(addr, false);
            peer.last_updated = i as i64; // 不同时间戳
            peers.register_peer(peer).await;
        }

        let handler = GetPeersHandler::new(Arc::clone(&peers));

        // 无分页：默认 limit=1000，应返回全部 10 个
        let mut request = PeerRequest::new(crate::protocol::RequestType::GetPeers, 1);
        let response = handler.handle(request, Arc::clone(&peers)).await;
        let peers_arr = response.as_object().unwrap().get("peers").unwrap().as_array().unwrap();
        assert_eq!(peers_arr.len(), 10);

        // 测试 limit=5
        let mut request_limit = PeerRequest::new(crate::protocol::RequestType::GetPeers, 1);
        request_limit.set("limit", 5);
        let response = handler.handle(request_limit, Arc::clone(&peers)).await;
        let peers_arr = response.as_object().unwrap().get("peers").unwrap().as_array().unwrap();
        assert_eq!(peers_arr.len(), 5);

        // 测试 offset=5, limit=5 -> 应返回剩余 5 个
        let mut request_offset = PeerRequest::new(crate::protocol::RequestType::GetPeers, 1);
        request_offset.set("limit", 5);
        request_offset.set("offset", 5);
        let response = handler.handle(request_offset, Arc::clone(&peers)).await;
        let peers_arr = response.as_object().unwrap().get("peers").unwrap().as_array().unwrap();
        assert_eq!(peers_arr.len(), 5);
    }
}
