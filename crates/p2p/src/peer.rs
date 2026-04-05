use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};

/// 节点状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerState {
    Connected,
    Disconnected,
    Blacklisted,
    Banned,
}

/// 对等节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub address: SocketAddr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub announced_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    pub services: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_ssl_port: Option<u16>,
    pub state: PeerState,
    pub is_inbound: bool,
    pub last_updated: i64, // timestamp
}

impl Peer {
    pub fn new(address: SocketAddr, is_inbound: bool) -> Self {
        Self {
            address,
            announced_address: None,
            version: None,
            application: None,
            platform: None,
            services: 0,
            api_port: None,
            api_ssl_port: None,
            state: PeerState::Disconnected,
            is_inbound,
            last_updated: current_timestamp(),
        }
    }

    pub fn update_metadata(
        &mut self,
        version: Option<String>,
        application: Option<String>,
        platform: Option<String>,
        services: u64,
        api_port: Option<u16>,
        api_ssl_port: Option<u16>,
    ) {
        if version.is_some() {
            self.version = version;
        }
        if application.is_some() {
            self.application = application;
        }
        if platform.is_some() {
            self.platform = platform;
        }
        if services > 0 {
            self.services = services;
        }
        if api_port.is_some() {
            self.api_port = api_port;
        }
        if api_ssl_port.is_some() {
            self.api_ssl_port = api_ssl_port;
        }
        self.last_updated = current_timestamp();
    }

    pub fn set_announced_address(&mut self, addr: String) {
        self.announced_address = Some(addr);
    }

    pub fn set_state(&mut self, state: PeerState) {
        self.state = state;
        self.last_updated = current_timestamp();
    }

    /// 转换为 Java 兼容的 PeerInfo 响应格式
    pub fn to_peer_info(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert("version".to_string(), serde_json::Value::String(self.version.clone().unwrap_or_default()));
        map.insert("application".to_string(), serde_json::Value::String(self.application.clone().unwrap_or_default()));
        map.insert("platform".to_string(), serde_json::Value::String(self.platform.clone().unwrap_or_default()));
        map.insert("services".to_string(), serde_json::Value::Number(serde_json::Number::from(self.services)));
        map.insert("state".to_string(), serde_json::Value::String(format!("{:?}", self.state)));
        map.insert("isInbound".to_string(), serde_json::Value::Bool(self.is_inbound));
        map.insert("lastUpdated".to_string(), serde_json::Value::Number(serde_json::Number::from(self.last_updated)));

        if let Some(ref ann) = self.announced_address {
            map.insert("announcedAddress".to_string(), serde_json::Value::String(ann.clone()));
        }
        if let Some(ref port) = self.api_port {
            map.insert("apiPort".to_string(), serde_json::Value::Number(serde_json::Number::from(*port)));
        }
        if let Some(ref port) = self.api_ssl_port {
            map.insert("apiSSLPort".to_string(), serde_json::Value::Number(serde_json::Number::from(*port)));
        }

        serde_json::Value::Object(map)
    }
}

/// 活跃连接跟踪
#[derive(Debug, Default)]
pub struct ActiveConnections {
    connections: HashSet<SocketAddr>,
}

impl ActiveConnections {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, addr: SocketAddr) {
        self.connections.insert(addr);
    }

    pub fn remove(&mut self, addr: &SocketAddr) {
        self.connections.remove(addr);
    }

    pub fn count(&self) -> usize {
        self.connections.len()
    }

    pub fn contains(&self, addr: &SocketAddr) -> bool {
        self.connections.contains(addr)
    }
}

/// 节点管理器
pub struct Peers {
    /// 已知节点（包括已断开但可重连的）
    known_peers: RwLock<HashMap<SocketAddr, Arc<Mutex<Peer>>>>,
    /// 活跃的 WebSocket/TCP 连接
    active_connections: Mutex<ActiveConnections>,
    /// 黑名单
    blacklist: RwLock<HashSet<SocketAddr>>,
    /// 自己节点的信息
    my_peer_info: RwLock<Peer>,
}

impl Peers {
    pub fn new(my_peer_info: Peer) -> Self {
        Self {
            known_peers: RwLock::new(HashMap::new()),
            active_connections: Mutex::new(ActiveConnections::new()),
            blacklist: RwLock::new(HashSet::new()),
            my_peer_info: RwLock::new(my_peer_info),
        }
    }

    /// 获取自己节点的 PeerInfo（用于 GetInfo 响应）
    pub async fn get_my_peer_info(&self) -> serde_json::Value {
        let my_info = self.my_peer_info.read().await;
        my_info.to_peer_info()
    }

    /// 更新自己节点信息
    pub async fn update_my_peer_info(&self, peer: Peer) {
        let mut my_info = self.my_peer_info.write().await;
        *my_info = peer;
    }

    /// 注册为已知节点（即使未连接）
    pub async fn register_peer(&self, peer: Peer) {
        let mut known = self.known_peers.write().await;
        let addr = peer.address;
        let entry = known.entry(addr).or_insert_with(|| Arc::new(Mutex::new(peer)));
        // 更新元数据
        let mut peer_mutex = entry.lock().await;
        peer_mutex.update_metadata(
            peer.version.clone(),
            peer.application.clone(),
            peer.platform.clone(),
            peer.services,
            peer.api_port,
            peer.api_ssl_port,
        );
        debug!("Registered peer: {}", addr);
    }

    /// 添加活跃连接
    pub async fn add_connection(&self, addr: SocketAddr) {
        let mut conns = self.active_connections.lock().await;
        conns.add(addr);
        debug!("Added active connection: {}", addr);
    }

    /// 移除活跃连接
    pub async fn remove_connection(&self, addr: &SocketAddr) {
        let mut conns = self.active_connections.lock().await;
        conns.remove(addr);
        debug!("Removed active connection: {}", addr);
    }

    /// 获取活跃连接数
    pub async fn connection_count(&self) -> usize {
        let conns = self.active_connections.lock().await;
        conns.count()
    }

    /// 获取所有已知节点列表
    pub async fn get_known_peers(&self) -> Vec<Peer> {
        let known = self.known_peers.read().await;
        known.values()
            .map(|p| {
                let p = p.lock().unwrap_or_else(|e| e.into_inner());
                p.clone()
            })
            .collect()
    }

    /// 获取活跃节点列表
    pub async fn get_active_peers(&self) -> Vec<Peer> {
        let conns = self.active_connections.lock().await;
        let known = self.known_peers.read().await;

        let mut active = Vec::new();
        for addr in conns.connections.iter() {
            if let Some(p) = known.get(addr) {
                let p = p.lock().unwrap_or_else(|e| e.into_inner());
                active.push(p.clone());
            }
        }
        active
    }

    /// 查找或创建节点（用于 AddPeers 等场景）
    pub async fn find_or_create_peer(&self, addr: SocketAddr, is_inbound: bool) -> Arc<Mutex<Peer>> {
        let mut known = self.known_peers.write().await;
        known.entry(addr).or_insert_with(|| {
            Arc::new(Mutex::new(Peer::new(addr, is_inbound)))
        }).clone()
    }

    /// 检查是否在黑名单中
    pub async fn is_blacklisted(&self, addr: &SocketAddr) -> bool {
        let blacklist = self.blacklist.read().await;
        blacklist.contains(addr)
    }

    /// 添加到黑名单
    pub async fn blacklist(&self, addr: SocketAddr) {
        let mut blacklist = self.blacklist.write().await;
        blacklist.insert(addr);
        warn!("Peer blacklisted: {}", addr);
    }
}

fn current_timestamp() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_serialization() {
        let peer = Peer {
            address: "127.0.0.1:8080".parse().unwrap(),
            announced_address: Some("example.com:8080".to_string()),
            version: Some("1.0.0".to_string()),
            application: Some("NRCs".to_string()),
            platform: Some("Rust".to_string()),
            services: 0x1,
            api_port: Some(8081),
            api_ssl_port: Some(8082),
            state: PeerState::Connected,
            is_inbound: false,
            last_updated: 1234567890,
        };

        let json = serde_json::to_string(&peer).unwrap();
        let decoded: Peer = serde_json::from_str(&json).unwrap();

        assert_eq!(peer.address, decoded.address);
        assert_eq!(peer.announced_address, decoded.announced_address);
        assert_eq!(peer.version, decoded.version);
        assert_eq!(peer.services, decoded.services);
    }
}