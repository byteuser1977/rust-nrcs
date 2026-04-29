//! Network API Handlers
//!
//! 网络相关的 API 处理器

use axum::{
    extract::State,
    Json,
};
use std::time::Instant;
use p2p::PeerState;
use crate::handlers::network::{state::NetworkApiState, dto::*};
use crate::core::ApiError;

/// 获取节点列表（完整版）
///
/// 对应 NRCS Java: PeerServlet.getPeers()
pub async fn get_peers(
    State(state): State<NetworkApiState>,
) -> Result<Json<GetPeersResponse>, ApiError> {
    let start_time = Instant::now();

    let peers = if let Some(p2p_manager) = &state.p2p_manager {
        // 从 P2P 管理器获取已连接的公共节点列表
        let active_peers = p2p_manager.get_active_peers().await;
        let public_peers: Vec<PeerInfo> = active_peers.iter()
            .filter(|p| !p.is_blacklisted() && p.announced_address.is_some())
            .map(|p| {
                PeerInfo {
                    address: p.address.to_string(),
                    port: p.port,
                    state: match p.state {
                        PeerState::NonConnected => 0,
                        PeerState::Connected => 1,
                        PeerState::Disconnected => 2,
                    },
                    announced_address: p.announced_address.clone()
                        .unwrap_or_else(|| p.address.to_string()),
                    share_address: p.share_address,
                    downloaded_volume: p.downloaded_volume,
                    uploaded_volume: p.uploaded_volume,
                    application: p.application.clone()
                        .unwrap_or_else(|| "Unknown".to_string()),
                    version: p.version.clone()
                        .unwrap_or_else(|| "Unknown".to_string()),
                    platform: p.platform.clone()
                        .unwrap_or_else(|| "Unknown".to_string()),
                    blacklisted: p.is_blacklisted(),
                }
            })
            .collect();
        public_peers
    } else {
        vec![]
    };

    let processing_time = start_time.elapsed().as_millis() as u32;

    Ok(Json(GetPeersResponse {
        peers,
        request_processing_time: processing_time,
    }))
}

/// 获取单个节点信息
pub async fn get_peer(
    State(state): State<NetworkApiState>,
    Json(req): Json<GetPeerRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let start_time = Instant::now();

    if let Some(p2p_manager) = &state.p2p_manager {
        // 尝试从已知节点中查找
        let all_peers = p2p_manager.get_peers().await;
        
        if let Some(peer) = all_peers.iter().find(|p| {
            p.address.to_string() == req.peer ||
            p.announced_address.as_deref() == Some(&req.peer)
        }) {
            let processing_time = start_time.elapsed().as_millis() as u32;
            
            Ok(Json(serde_json::json!({
                "address": peer.address.to_string(),
                "port": peer.port,
                "state": match peer.state {
                    PeerState::NonConnected => 0,
                    PeerState::Connected => 1,
                    PeerState::Disconnected => 2,
                },
                "announcedAddress": peer.announced_address.as_deref()
                    .unwrap_or(&peer.address.to_string()),
                "shareAddress": peer.share_address,
                "downloadedVolume": peer.downloaded_volume,
                "uploadedVolume": peer.uploaded_volume,
                "application": peer.application.as_deref().unwrap_or("NRCS"),
                "version": peer.version.as_deref().unwrap_or("Unknown"),
                "platform": peer.platform.as_deref().unwrap_or("Rust"),
                "blacklisted": peer.is_blacklisted(),
                "services": peer.services,
                "requestProcessingTime": processing_time
            })))
        } else {
            Err(ApiError::NotFound(format!("Peer {} not found", req.peer)))
        }
    } else {
        Err(ApiError::NotFound("P2P network not available".to_string()))
    }
}

/// 获取本节点信息（完整版，对应 Java getMyPeerInfoResponse）
///
/// 对应 NRCS Java: Peers.getMyPeerInfoResponse()
pub async fn get_my_peer_info(
    State(state): State<NetworkApiState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if let Some(p2p_manager) = &state.p2p_manager {
        // 使用完整的 myPeerInfo 响应（包含所有字段）
        let info = p2p_manager.get_my_peer_info_full().await;
        Ok(Json(info))
    } else {
        Err(ApiError::NotFound("P2P network not available".to_string()))
    }
}

/// 获取网络状态摘要
pub async fn get_network_status(
    State(state): State<NetworkApiState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if let Some(p2p_manager) = &state.p2p_manager {
        let connected_count = p2p_manager.connected_peers_count().await;
        let known_count = p2p_manager.known_peers_count().await;
        
        Ok(Json(serde_json::json!({
            "numberOfActivePeers": connected_count,
            "numberOfKnownPeers": known_count,
            "numberOfUnconfirmedPeers": 0,  // TODO: 实现
            "isScanning": false,
            "application": blockchain_types::constants::APPLICATION,
            "version": blockchain_types::constants::VERSION,
            "platform": std::env::consts::OS.to_string(),
            "blacklistingPeriod": p2p_manager.config().blacklisting_period_secs,
            "maxNumberOfConnectedPublicPeers": p2p_manager.config().max_connections,
            "maxNumberOfInboundConnections": p2p_manager.config().max_inbound_connections,
            "maxNumberOfOutboundConnections": p2p_manager.config().max_outbound_connections,
        })))
    } else {
        Err(ApiError::NotFound("P2P network not available".to_string()))
    }
}
