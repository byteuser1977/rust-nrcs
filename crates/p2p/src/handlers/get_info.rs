use crate::{peer::Peers, protocol::PeerRequest};
use serde_json;
use std::sync::Arc;
use std::net::SocketAddr;
use tracing::{debug, warn};

/// GetInfo 处理器
///
/// 对应 Java: GetInfo.java
///
/// P2P 握手请求。远程 peer 发送自己的节点元数据，本处理器：
/// 1. 更新 peer 的属性 (services, version, application, platform, hallmark, announcedAddress, etc.)
/// 2. 返回本节点的 PeerInfo
pub struct GetInfoHandler {
    #[allow(dead_code)]
    peers: Arc<Peers>,
}

impl GetInfoHandler {
    pub fn new(peers: Arc<Peers>) -> Self {
        Self { peers }
    }

    pub async fn handle(&self, request: PeerRequest, peers: Arc<Peers>, peer_addr: SocketAddr) -> serde_json::Value {
        debug!("Handling GetInfo request from {}", peer_addr);

        // Extract hallmark string before locking peer (used later with separate lock)
        let hallmark_str = request.get::<String>("hallmark")
            .filter(|s| !s.is_empty());

        // 对应 Java: GetInfo.processRequest()
        // 从请求中读取并更新 peer 的元数据
        if let Some(peer_ref) = peers.get_peer(&peer_addr).await {
            let mut peer = peer_ref.lock().await;

            // services — 服务位掩码
            if let Some(services) = request.get::<serde_json::Value>("services") {
                let old_services = peer.services;
                peer.services = services.as_u64().unwrap_or(0);
                if peer.services != old_services {
                    debug!("Peer {} services changed: {} -> {}", peer_addr, old_services, peer.services);
                    peer.fire_event(crate::peer::PeerEvent::ChangedServices);
                }
            }

            // application
            if let Some(app) = request.get::<String>("application") {
                peer.application = Some(app);
            }

            // version
            if let Some(ver) = request.get::<String>("version") {
                peer.version = Some(ver);
            }

            // platform
            if let Some(plat) = request.get::<String>("platform") {
                peer.platform = Some(plat);
            }

            // announcedAddress
            if let Some(announced) = request.get::<String>("announcedAddress") {
                if !announced.is_empty() {
                    peer.set_announced_address(announced);
                }
            }

            // apiPort
            if let Some(port) = request.get::<serde_json::Value>("apiPort")
                .and_then(|v| v.as_u64())
            {
                peer.api_port = Some(port as u16);
            }

            // apiSSLPort
            if let Some(port) = request.get::<serde_json::Value>("apiSSLPort")
                .and_then(|v| v.as_u64())
            {
                peer.api_ssl_port = Some(port as u16);
            }

            // shareAddress
            if let Some(share) = request.get::<bool>("shareAddress") {
                peer.share_address = share;
            }

            // disabledAPIs — base64 encoded disabled API list
            if let Some(disabled) = request.get::<String>("disabledAPIs") {
                if let Ok(decoded) = base64::decode(&disabled) {
                    if let Ok(list) = String::from_utf8(decoded) {
                        peer.disabled_apis = Some(list.split(',').map(|s| s.trim().to_string()).collect());
                    }
                }
            }

            // apiServerIdleTimeout
            if let Some(_idle_timeout) = request.get::<serde_json::Value>("apiServerIdleTimeout") {
                // stored per-peer for proxy peer idle tracking
            }

            // blockchainState
            if let Some(state_val) = request.get::<serde_json::Value>("blockchainState") {
                // UP_TO_DATE=0, DOWNLOADING=1, FORK=2, LIGHT_CLIENT=3
                peer.blockchain_state = state_val.as_u64().map(|v| v as u8);
            }

            peer.last_updated = crate::peer::current_timestamp();
            debug!("Updated peer {} metadata: app={:?}, ver={:?}, svc={}",
                   peer_addr, peer.application, peer.version, peer.services);
        } else {
            warn!("GetInfo from unknown peer {}: not in registry", peer_addr);
        }

        // hallmark — 解析、验证并分组权重（需要单独加锁，因为 analyze_hallmark 会访问 Peers）
        if let Some(ref hs) = hallmark_str {
            if let Some(peer_ref) = peers.get_peer(&peer_addr).await {
                let mut p = peer_ref.lock().await;
                if p.analyze_hallmark(hs, &peers) {
                    debug!("Peer {} hallmark valid, weight={}", peer_addr,
                           p.hallmark.as_ref().map(|h| h.weight).unwrap_or(0));
                } else {
                    debug!("Peer {} hallmark invalid", peer_addr);
                }
            }
        }

        // 返回自己节点的信息
        // 对应 Java: return Peers.getMyPeerInfoResponse()
        let my_info = peers.get_my_peer_info().await;
        debug!("GetInfo response to {}: {:?}", peer_addr, my_info);
        my_info
    }
}
