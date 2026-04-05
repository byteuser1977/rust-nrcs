use crate::{peer::Peers, protocol::PeerRequest};
use serde_json;
use std::sync::Arc;
use tracing::{debug, warn};

/// GetInfo 处理器
/// 响应包含本节点的 PeerInfo，字段需与 Java 的 Peers.getMyPeerInfoResponse() 完全匹配
pub struct GetInfoHandler {
    peers: Arc<Peers>,
}

impl GetInfoHandler {
    pub fn new(peers: Arc<Peers>) -> Self {
        Self { peers }
    }

    pub async fn handle(&self, _request: PeerRequest, peers: Arc<Peers>) -> serde_json::Value {
        debug!("Handling GetInfo request");

        // 从请求中读取可能更新元数据的字段（可选）
        // 如 services, hallmark, announcedAddress, application, version, platform, shareAddress, apiPort, apiSSLPort, disabledAPIs, apiServerIdleTimeout, blockchainState

        // 返回自己的 PeerInfo
        let my_info = peers.get_my_peer_info().await;

        debug!("GetInfo response: {:?}", my_info);
        my_info
    }
}