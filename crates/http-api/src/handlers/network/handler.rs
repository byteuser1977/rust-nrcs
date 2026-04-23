//! Network API Handlers
//!
//! 网络相关的 API 处理器

use axum::{
    extract::State,
    Json,
};
use std::time::Instant;
use crate::handlers::network::{state::NetworkApiState, dto::*};
use crate::core::ApiError;

/// 获取节点列表
pub async fn get_peers(
    State(state): State<NetworkApiState>,
) -> Result<Json<GetPeersResponse>, ApiError> {
    let start_time = Instant::now();
    
    let peers = if let Some(_p2p_manager) = &state.p2p_manager {
        // TODO: 从 P2P 管理器获取节点列表
        vec![]
    } else {
        vec![]
    };
    
    let processing_time = start_time.elapsed().as_millis() as u32;
    
    Ok(Json(GetPeersResponse {
        peers,
        request_processing_time: processing_time,
    }))
}

/// 获取节点信息
pub async fn get_peer(
    State(state): State<NetworkApiState>,
    Json(req): Json<GetPeerRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let start_time = Instant::now();
    
    if let Some(_p2p_manager) = &state.p2p_manager {
        // TODO: 从 P2P 管理器获取节点信息
        let processing_time = start_time.elapsed().as_millis() as u32;
        
        Ok(Json(serde_json::json!({
            "address": req.peer,
            "port": 17974,
            "state": 1,
            "announcedAddress": req.peer,
            "shareAddress": true,
            "downloadedVolume": 0,
            "uploadedVolume": 0,
            "application": "NRCS",
            "version": env!("CARGO_PKG_VERSION"),
            "platform": "Rust",
            "blacklisted": false,
            "requestProcessingTime": processing_time
        })))
    } else {
        Err(ApiError::NotFound("P2P network not available".to_string()))
    }
}
