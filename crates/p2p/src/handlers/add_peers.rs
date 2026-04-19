use crate::{peer::Peers, protocol::PeerRequest};
use serde_json;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, info, warn};

const MAX_KNOWN_PEERS: usize = 2000;

/// AddPeers 处理器
/// 请求：peers 数组（每个 peer 包含 address, port, services 等）
/// 处理：验证并添加到已知 peers
/// 响应：统计添加的数量，重复数量，黑名单数量等
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
        let mut duplicate_count = 0;
        let mut blacklisted_count = 0;
        let mut invalid_count = 0;

        // 本地去重（同一请求内相同地址只处理一次）
        let mut seen_addrs = std::collections::HashSet::new();

        // 第一阶段：收集所有符合条件的新节点（尚未在已知列表中，且非黑名单，地址合法）
        let mut new_peers = Vec::new();

        for peer_data in peers_data.iter() {
            // 提取 address 和 port
            let addr_str = match peer_data.get("address").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => {
                    invalid_count += 1;
                    continue;
                }
            };
            let port = match peer_data.get("port").and_then(|v| v.as_i64()) {
                Some(p) => p,
                None => {
                    invalid_count += 1;
                    continue;
                }
            };
            let port_u16: u16 = match port.try_into() {
                Ok(p) => p,
                Err(_) => {
                    invalid_count += 1;
                    continue;
                }
            };
            let addr_str_full = format!("{}:{}", addr_str, port_u16);
            let addr: SocketAddr = match addr_str_full.parse() {
                Ok(a) => a,
                Err(_) => {
                    invalid_count += 1;
                    continue;
                }
            };

            // 检查是否为本机环回地址（localhost）
            if addr.ip().is_loopback() {
                warn!("Peer address {} is loopback, skipping", addr);
                invalid_count += 1;
                continue;
            }

            // 去重：同一请求内已出现
            if !seen_addrs.insert(addr) {
                duplicate_count += 1;
                continue;
            }

            // 检查黑名单
            if peers.is_blacklisted(&addr).await {
                warn!("Peer {} is blacklisted, skipping", addr);
                blacklisted_count += 1;
                continue;
            }

            // 检查是否已知
            if peers.contains_peer(&addr).await {
                duplicate_count += 1;
                continue;
            }

            // 构建 Peer 对象，提取其他字段
            let services = peer_data.get("services").and_then(|v| v.as_u64()).unwrap_or(0);

            let mut peer = crate::peer::Peer::new(addr, false); // outbound
            peer.services = services;

            // 提取可选字段
            if let Some(version) = peer_data.get("version").and_then(|v| v.as_str()) {
                peer.version = Some(version.to_string());
            }
            if let Some(platform) = peer_data.get("platform").and_then(|v| v.as_str()) {
                peer.platform = Some(platform.to_string());
            }
            if let Some(app) = peer_data.get("application").and_then(|v| v.as_str()) {
                peer.application = Some(app.to_string());
            }
            // 提取 apiPort 和 apiSSLPort
            if let Some(api_port) = peer_data.get("apiPort").and_then(|v| v.as_i64()) {
                if (1..=65535).contains(&api_port) {
                    peer.api_port = Some(api_port as u16);
                }
            }
            if let Some(api_ssl_port) = peer_data.get("apiSSLPort").and_then(|v| v.as_i64()) {
                if (1..=65535).contains(&api_ssl_port) {
                    peer.api_ssl_port = Some(api_ssl_port as u16);
                }
            }

            new_peers.push(peer);
        }

        // 容量检查：如果当前已知节点数已达上限，且有新节点需要添加，则拒绝
        let known_count = peers.known_peers_count().await;
        if known_count >= MAX_KNOWN_PEERS {
            if !new_peers.is_empty() {
                warn!("Max known peers ({}) reached, rejecting {} new peers", MAX_KNOWN_PEERS, new_peers.len());
                return serde_json::json!({
                    "error": "MAX_KNOWN_PEERS_EXCEEDED",
                    "max": MAX_KNOWN_PEERS,
                    "current": known_count
                });
            }
            // 如果没有新节点，直接返回统计信息（可能都是重复或黑名单）
        } else if known_count + new_peers.len() > MAX_KNOWN_PEERS {
            // 添加新节点后会超过上限，拒绝整个请求
            warn!("Adding {} new peers would exceed max capacity ({}). Rejecting.", new_peers.len(), MAX_KNOWN_PEERS);
            return serde_json::json!({
                "error": "MAX_KNOWN_PEERS_EXCEEDED",
                "max": MAX_KNOWN_PEERS,
                "current": known_count
            });
        }

        // 注册新节点
        for peer in new_peers {
            peers.register_peer(peer).await;
            added_count += 1;
        }

        info!("AddPeers: added {}, duplicate {}, blacklisted {}, invalid {}", added_count, duplicate_count, blacklisted_count, invalid_count);

        // 返回统计信息
        let mut response = serde_json::Map::new();
        response.insert("added".to_string(), serde_json::Value::Number(added_count.into()));
        response.insert("duplicate".to_string(), serde_json::Value::Number(duplicate_count.into()));
        response.insert("blacklisted".to_string(), serde_json::Value::Number(blacklisted_count.into()));
        response.insert("invalid".to_string(), serde_json::Value::Number(invalid_count.into()));

        serde_json::Value::Object(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer::Peer;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_add_peers_success() {
        let my_peer = Peer::new("127.0.0.1:8080".parse().unwrap(), false);
        let peers = Arc::new(crate::peer::Peers::new(my_peer));

        let handler = AddPeersHandler::new(Arc::clone(&peers));

        // 构造请求：添加一个合法节点
        let request_value = serde_json::json!({
            "requestType": "addPeers",
            "protocol": 1,
            "peers": [
                {
                    "address": "192.168.1.100",
                    "port": 9000,
                    "services": 1,
                    "version": "1.0.0",
                    "platform": "Rust",
                    "application": "NRCs",
                    "apiPort": 8081,
                    "apiSSLPort": 8082
                }
            ]
        });
        let request: PeerRequest = serde_json::from_value(request_value).unwrap();

        let response = handler.handle(request, Arc::clone(&peers)).await;
        let obj = response.as_object().unwrap();
        assert_eq!(obj.get("added").and_then(|v| v.as_i64()), Some(1));
        assert_eq!(obj.get("duplicate").and_then(|v| v.as_i64()), Some(0));
        assert_eq!(obj.get("blacklisted").and_then(|v| v.as_i64()), Some(0));
        // 验证已知节点数是否增加
        assert_eq!(peers.known_peers_count().await, 1);
    }

    #[tokio::test]
    async fn test_add_peers_duplicate() {
        let my_peer = Peer::new("127.0.0.1:8080".parse().unwrap(), false);
        let peers = Arc::new(crate::peer::Peers::new(my_peer));

        // 先添加一个节点
        let addr: SocketAddr = "192.168.1.100:9000".parse().unwrap();
        let mut peer = Peer::new(addr, false);
        peer.services = 1;
        peers.register_peer(peer).await;
        assert_eq!(peers.known_peers_count().await, 1);

        let handler = AddPeersHandler::new(Arc::clone(&peers));

        // 再次添加同一个节点
        let request_value = serde_json::json!({
            "requestType": "addPeers",
            "protocol": 1,
            "peers": [
                {
                    "address": "192.168.1.100",
                    "port": 9000,
                    "services": 1
                }
            ]
        });
        let request: PeerRequest = serde_json::from_value(request_value).unwrap();
        let response = handler.handle(request, Arc::clone(&peers)).await;
        let obj = response.as_object().unwrap();
        assert_eq!(obj.get("added").and_then(|v| v.as_i64()), Some(0));
        assert_eq!(obj.get("duplicate").and_then(|v| v.as_i64()), Some(1));
        assert_eq!(peers.known_peers_count().await, 1);
    }

    #[tokio::test]
    async fn test_add_peers_blacklist() {
        let my_peer = Peer::new("127.0.0.1:8080".parse().unwrap(), false);
        let peers = Arc::new(crate::peer::Peers::new(my_peer));
        let addr: SocketAddr = "192.168.1.100:9000".parse().unwrap();
        peers.blacklist(addr).await;

        let handler = AddPeersHandler::new(Arc::clone(&peers));

        let request_value = serde_json::json!({
            "requestType": "addPeers",
            "protocol": 1,
            "peers": [
                {
                    "address": "192.168.1.100",
                    "port": 9000,
                    "services": 1
                }
            ]
        });
        let request: PeerRequest = serde_json::from_value(request_value).unwrap();
        let response = handler.handle(request, Arc::clone(&peers)).await;
        let obj = response.as_object().unwrap();
        assert_eq!(obj.get("added").and_then(|v| v.as_i64()), Some(0));
        assert_eq!(obj.get("blacklisted").and_then(|v| v.as_i64()), Some(1));
    }

    #[tokio::test]
    async fn test_add_peers_localhost_rejected() {
        let my_peer = Peer::new("127.0.0.1:8080".parse().unwrap(), false);
        let peers = Arc::new(crate::peer::Peers::new(my_peer));

        let handler = AddPeersHandler::new(Arc::clone(&peers));

        let request_value = serde_json::json!({
            "requestType": "addPeers",
            "protocol": 1,
            "peers": [
                {
                    "address": "127.0.0.1",
                    "port": 9000,
                    "services": 1
                }
            ]
        });
        let request: PeerRequest = serde_json::from_value(request_value).unwrap();
        let response = handler.handle(request, Arc::clone(&peers)).await;
        let obj = response.as_object().unwrap();
        // should be counted as invalid, not added
        assert_eq!(obj.get("added").and_then(|v| v.as_i64()), Some(0));
        assert_eq!(obj.get("invalid").and_then(|v| v.as_i64()), Some(1));
    }

    #[tokio::test]
    async fn test_add_peers_capacity_enforced() {
        let my_peer = Peer::new("127.0.0.1:8080".parse().unwrap(), false);
        let peers = Arc::new(crate::peer::Peers::new(my_peer));

        // Pre-fill with 2000 peers
        for i in 0..2000 {
            let addr = format!("10.0.0.1:{}", 9000 + i).parse::<SocketAddr>().unwrap();
            let mut peer = Peer::new(addr, false);
            peer.services = 1;
            peers.register_peer(peer).await;
        }
        assert_eq!(peers.known_peers_count().await, 2000);

        let handler = AddPeersHandler::new(Arc::clone(&peers));

        // 尝试添加一个新 peer
        let request_value = serde_json::json!({
            "requestType": "addPeers",
            "protocol": 1,
            "peers": [
                {
                    "address": "10.0.0.2",
                    "port": 9000,
                    "services": 1
                }
            ]
        });
        let request: PeerRequest = serde_json::from_value(request_value).unwrap();
        let response = handler.handle(request, Arc::clone(&peers)).await;
        let obj = response.as_object().unwrap();
        assert!(obj.get("error").is_some());
        assert_eq!(obj.get("added").and_then(|v| v.as_i64()), None);
        // capacity unchanged
        assert_eq!(peers.known_peers_count().await, 2000);
    }
}
