use crate::{peer::Peers, protocol::PeerRequest};
use serde_json;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// AddPeers 处理器
/// 请求：peers 数组（每个 peer 包含 address, port, services 等）
/// 处理：验证并添加到已知 peers，尝试连接
/// 响应：通常空对象或统计信息
pub struct AddPeersHandler {
    peers: Arc<Peers>,
}

impl AddPeersHandler {
    pub fn new(peers: Arc<Peers>) -> Self {
        Self { peers }
    }

    pub async fn handle(&self, request: PeerRequest, peers: Arc<Peers>) -> serde_json::Value {
        debug!("Handling AddPeers request");

        // 解析 peers 数组
        let peers_data: Vec<serde_json::Value> = match request.get("peers") {
            Some(p) => p,
            None => {
                warn!("AddPeers missing 'peers' field");
                return serde_json::json!({ "error": "MISSING_PEERS" });
            }
        };

        let mut added_count = 0;
        let mut blacklisted_count = 0;

        for peer_data in peers_data {
            if let Ok(addr_str) = peer_data.get("address").and_then(|v| v.as_str()) {
                let port = peer_data.get("port").and_then(|v| v.as_i64());
                let services = peer_data.get("services").and_then(|v| v.as_u64()).unwrap_or(0);

                // 构建 SocketAddr (简化版，实际需要更复杂的地址解析)
                if let (Some(port_i64), Ok(port_u16)) = (port, port_i64.try_into().ok()) {
                    let addr_str_full = format!("{}:{}", addr_str, port_u16);
                    if let Ok(addr) = addr_str_full.parse::<SocketAddr>() {
                        // 检查是否在黑名单
                        if peers.is_blacklisted(&addr).await {
                            warn!("Peer {} is blacklisted, skipping", addr);
                            blacklisted_count += 1;
                            continue;
                        }

                        // 创建并注册 peer
                        let mut peer = crate::peer::Peer::new(addr, false); // outbound
                        peer.services = services;
                        // 从 peer_data 提取其他字段...
                        if let Some(version) = peer_data.get("version").and_then(|v| v.as_str()) {
                            peer.version = Some(version.to_string());
                        }
                        if let Some(platform) = peer_data.get("platform").and_then(|v| v.as_str()) {
                            peer.platform = Some(platform.to_string());
                        }
                        if let Some(app) = peer_data.get("application").and_then(|v| v.as_str()) {
                            peer.application = Some(app.to_string());
                        }

                        peers.register_peer(peer).await;
                        added_count += 1;
                        debug!("Added peer: {} (services: {})", addr, services);
                    } else {
                        warn!("Invalid peer address: {}", addr_str_full);
                    }
                }
            }
        }

        info!("AddPeers: added {}, blacklisted {}", added_count, blacklisted_count);

        // 返回统计信息
        let mut response = serde_json::Map::new();
        response.insert("added".to_string(), serde_json::Value::Number(added_count.into()));
        response.insert("blacklisted".to_string(), serde_json::Value::Number(blacklisted_count.into()));

        serde_json::Value::Object(response)
    }
}