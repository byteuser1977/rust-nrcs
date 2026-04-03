//! 节点相关处理器

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{error::ApiResult, response::NodeInfoResponse};

/// 获取节点信息
pub async fn get_node_info(
    State(state): State<ApiState>,
) -> ApiResult<Json<NodeInfoResponse>> {
    // TODO: 从节点状态获取真实信息
    let info = NodeInfoResponse {
        version: "0.1.0".to_string(),
        chain_id: "nrcs-mainnet-v1".to_string(),
        height: 0, // state.chain.head_height()
        syncing: false,
        peer_count: state.p2p_service.as_ref().map(|s| s.peer_count()).unwrap_or(0),
        cpu_usage: 0.0,
        memory_usage: 0,
        uptime_seconds: 0,
    };

    Ok(Json(info))
}

/// 获取已连接的Peer列表
#[derive(Debug, Serialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub address: String,
    pub height: u32,
    pub services: Vec<String>,
}

pub async fn list_peers(
    State(state): State<ApiState>,
) -> ApiResult<Json<Vec<PeerInfo>>> {
    // TODO: 从 P2P 服务获取 peer 列表
    let peers = vec![];

    Ok(Json(peers))
}