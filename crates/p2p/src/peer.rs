use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, warn};

/// 节点状态
/// 
/// 对应 NRCS Java: PeerState.java
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerState {
    /// 未连接 (NON_CONNECTED)
    NonConnected,
    /// 已连接 (CONNECTED)
    Connected,
    /// 已断开 (DISCONNECTED)
    Disconnected,
}

/// 对等节点信息
/// 
/// 对应 NRCS Java: Peer.java
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    /// 节点地址
    pub address: SocketAddr,
    /// 公告地址
    #[serde(skip_serializing_if = "Option::is_none")]
    pub announced_address: Option<String>,
    /// 版本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// 应用名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application: Option<String>,
    /// 平台
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    /// 服务标志
    pub services: u64,
    /// API 端口
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_port: Option<u16>,
    /// API SSL 端口
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_ssl_port: Option<u16>,
    /// 节点状态
    pub state: PeerState,
    /// 是否为入站连接
    pub is_inbound: bool,
    /// 最后更新时间戳
    pub last_updated: i64,
    /// 最后连接尝试时间戳
    pub last_connect_attempt: i64,
    /// 最后入站请求时间戳
    pub last_inbound_request: i64,
    /// 黑名单时间戳
    pub blacklisting_time: i64,
    /// 黑名单原因
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blacklisting_cause: Option<String>,
    /// 是否为旧版本
    pub is_old_version: bool,
    /// 是否分享地址
    pub share_address: bool,
    /// 下载流量
    pub downloaded_volume: u64,
    /// 上传流量
    pub uploaded_volume: u64,
    /// 端口号
    pub port: u16,
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
            state: PeerState::NonConnected,
            is_inbound,
            last_updated: current_timestamp(),
            last_connect_attempt: 0,
            last_inbound_request: 0,
            blacklisting_time: 0,
            blacklisting_cause: None,
            is_old_version: false,
            share_address: true,
            downloaded_volume: 0,
            uploaded_volume: 0,
            port: address.port(),
        }
    }

    /// 黑名单节点
    /// 
    /// 对应 NRCS Java: Peer.blacklist(String cause)
    pub fn blacklist(&mut self, cause: String) {
        self.blacklisting_time = current_timestamp();
        self.blacklisting_cause = Some(cause);
        self.state = PeerState::NonConnected;
        self.last_inbound_request = 0;
        debug!("Peer {} blacklisted: {:?}", self.address, self.blacklisting_cause);
    }

    /// 解除黑名单
    /// 
    /// 对应 NRCS Java: Peer.unBlacklist()
    pub fn un_blacklist(&mut self) {
        if self.blacklisting_time == 0 {
            return;
        }
        self.state = PeerState::NonConnected;
        self.blacklisting_time = 0;
        self.blacklisting_cause = None;
        debug!("Peer {} unblacklisted", self.address);
    }

    /// 更新黑名单状态
    /// 
    /// 对应 NRCS Java: Peer.updateBlacklistedStatus(int curTime)
    pub fn update_blacklisted_status(&mut self, cur_time: i64, blacklisting_period: i64) {
        if self.blacklisting_time > 0
            && self.blacklisting_time + blacklisting_period <= cur_time {
            self.un_blacklist();
        }
        if self.is_old_version && self.last_updated < cur_time - 3600 {
            self.is_old_version = false;
        }
    }

    /// 设置版本号（含版本检查）
    ///
    /// 对应 NRCS Java: Peer.setVersion(String version)
    /// 当 application 与本节点相同且版本过旧时，自动标记为旧版本
    pub fn set_version(&mut self, version: Option<String>, application: Option<&str>, config: &crate::config::P2PConfig) {
        if let Some(ref v) = version {
            if v.len() > config.max_version_length {
                debug!("Invalid version length: {} for peer {}", v.len(), self.address);
                return;
            }
        }

        let version_changed = version.as_ref() != self.version.as_ref();
        self.version = version;
        self.is_old_version = false;

        if application.map(|a| a == config.application).unwrap_or(false) {
            if let Some(ref v) = self.version {
                if config.is_old_version(v) {
                    self.is_old_version = true;
                    if version_changed {
                        debug!("Blacklisting {} version {}", self.address, v);
                    }
                    self.blacklisting_cause = Some(format!("Old version: {}", v));
                    self.last_inbound_request = 0;
                    self.state = PeerState::NonConnected;
                }
            }
        }
    }

    /// 停用节点
    /// 
    /// 对应 NRCS Java: Peer.deactivate()
    pub fn deactivate(&mut self) {
        self.state = PeerState::Disconnected;
        self.last_updated = current_timestamp();
    }

    /// 检查是否提供服务
    /// 
    /// 对应 NRCS Java: Peer.providesService(PeerService)
    pub fn provides_service(&self, service_flag: u64) -> bool {
        self.services & service_flag != 0
    }

    /// 获取节点权重
    /// 
    /// 对应 NRCS Java: Peer.getWeight()
    /// 权重基于服务标志和下载/上传流量计算
    pub fn get_weight(&self) -> u64 {
        // 基础权重为 1
        let mut weight: u64 = 1;
        
        // 如果提供服务，增加权重
        // 服务标志: 1=API, 2=API_SSL, 4=CORS, 8=HALLMARK
        if self.services > 0 {
            weight += self.services.count_ones() as u64 * 10;
        }
        
        // 根据下载流量增加权重（每 1MB 增加 1 点权重）
        weight += self.downloaded_volume / (1024 * 1024);
        
        // 根据上传流量增加权重（每 1MB 增加 1 点权重）
        weight += self.uploaded_volume / (1024 * 1024);
        
        weight
    }

    /// 检查是否在黑名单中
    ///
    /// 对应 Java: Peer.isBlacklisted() - 检查 blacklistingTime 和 isOldVersion
    /// 注意: knownBlacklistedPeers 检查由 Peers.is_peer_blacklisted() 完成
    pub fn is_blacklisted(&self) -> bool {
        self.blacklisting_time > 0 || self.is_old_version
    }

    /// 发送请求到远程节点（WebSocket 优先 + HTTP 回退）
    ///
    /// 对应 Java: Peer.send(JSONObject request, int maxResponseSize)
    pub async fn send(
        &self,
        request: &crate::protocol::PeerRequest,
        config: &crate::config::P2PConfig,
    ) -> Result<serde_json::Value, crate::error::P2PError> {
        use crate::error::P2PError;

        // 1. 检查黑名单状态
        if self.is_blacklisted() {
            return Err(P2PError::blacklisted("Peer is blacklisted"));
        }

        // 2. 如果启用 WebSocket 且节点支持，优先使用 WebSocket
        if config.use_websockets {
            match Self::send_via_websocket(&self.address, request, config).await {
                Ok(response) => {
                    debug!("[Peer.send] WebSocket success to {}", self.address);
                    return Ok(response);
                }
                Err(e) => {
                    warn!("[Peer.send] WebSocket failed to {}: {}, falling back to HTTP", self.address, e);
                    // 继续尝试 HTTP
                }
            }
        }

        // 3. 回退到 HTTP POST
        Self::send_via_http(&self.address, request, config).await
    }

    /// 通过 WebSocket 发送请求（单次连接）
    async fn send_via_websocket(
        addr: &SocketAddr,
        request: &crate::protocol::PeerRequest,
        config: &crate::config::P2PConfig,
    ) -> Result<serde_json::Value, crate::error::P2PError> {
        use crate::error::{ErrorCode, P2PError};
        use crate::protocol::FrameCodec;
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::tungstenite::Message;
        use tokio::time::timeout;

        let url = format!("ws://{}/nrcs", addr);

        // 连接超时
        let (ws_stream, _) = timeout(
            std::time::Duration::from_millis(config.connect_timeout_ms),
            tokio_tungstenite::connect_async(&url),
        ).await
        .map_err(|_| P2PError::connection_timeout())?
        .map_err(|e| P2PError::from_str(ErrorCode::ConnectionFailed, e.to_string()))?;

        let (mut write, mut read) = ws_stream.split();
        let codec = FrameCodec;

        // 序列化并发送请求
        let payload = serde_json::to_vec(request)
            .map_err(|e| P2PError::serialization_error(format!("{}", e)))?;

        // 判断是否压缩
        let should_compress = config.gzip_enabled && payload.len() >= config.min_compress_size;
        let frame = codec.encode(&payload, should_compress);

        write.send(Message::Binary(frame)).await
            .map_err(|e| P2PError::from_str(ErrorCode::WriteFailed, format!("{}", e)))?;

        // 等待响应
        let response_result = timeout(
            std::time::Duration::from_millis(config.read_timeout_ms),
            read.next(),
        ).await;

        let response_body = match response_result {
            Ok(Some(Ok(msg))) => msg,
            Ok(Some(Err(e))) => return Err(P2PError::from_str(ErrorCode::ConnectionClosed, format!("{}", e))),
            Ok(None) => return Err(P2PError::connection_closed()),
            Err(_) => return Err(P2PError::read_timeout()),
        };

        match response_body {
            Message::Binary(data) => {
                match codec.decode(&data) {
                    Ok((_header, body)) => {
                        serde_json::from_slice::<serde_json::Value>(&body)
                            .map_err(|e| P2PError::deserialization_error(format!("{}", e)))
                    }
                    Err(e) => Err(P2PError::from_str(ErrorCode::ProtocolError, format!("{}", e)))
                }
            }
            Message::Text(text) => {
                let data = text.into_bytes();
                match codec.decode(&data) {
                    Ok((_header, body)) => {
                        serde_json::from_slice::<serde_json::Value>(&body)
                            .map_err(|e| P2PError::deserialization_error(format!("{}", e)))
                    }
                    Err(e) => Err(P2PError::from_str(ErrorCode::ProtocolError, format!("{}", e)))
                }
            }
            _ => Err(P2PError::unexpected_message_type())
        }
    }

    /// 通过 HTTP POST 发送请求
    async fn send_via_http(
        addr: &SocketAddr,
        request: &crate::protocol::PeerRequest,
        config: &crate::config::P2PConfig,
    ) -> Result<serde_json::Value, crate::error::P2PError> {
        use crate::error::P2PError;
        use reqwest::Client;
        use tokio::time::timeout;

        let client = Client::builder()
            .timeout(std::time::Duration::from_millis(config.read_timeout_ms))
            .build()
            .map_err(|e| P2PError::internal(format!("{}", e)))?;

        let url = format!("http://{}/nrcs", addr);

        let resp = timeout(
            std::time::Duration::from_millis(config.connect_timeout_ms + config.read_timeout_ms),
            client.post(&url).json(request).send(),
        ).await
        .map_err(|_| P2PError::request_timeout())?
        .map_err(|e| P2PError::http_error(format!("{}", e)))?;

        if !resp.status().is_success() {
            return Err(P2PError::http_status(resp.status().as_u16()));
        }

        let body = resp.bytes().await
            .map_err(|e| P2PError::read_failed(format!("{}", e)))?;

        serde_json::from_slice::<serde_json::Value>(&body)
            .map_err(|e| P2PError::deserialization_error(format!("{}", e)))
    }

    /// 连接到远程节点并交换信息（完整握手流程）
    ///
    /// 对应 Java: Peer.connect()
    pub async fn connect(
        &mut self,
        config: &crate::config::P2PConfig,
    ) -> Result<serde_json::Value, crate::error::P2PError> {
        use crate::protocol::{PeerRequest, RequestType};

        // 1. 更新最后连接尝试时间
        self.last_connect_attempt = current_timestamp();

        // 2. 构建 getInfo 请求
        let request = PeerRequest::new(RequestType::GetInfo, 1);

        // 3. 发送请求
        let response = self.send(&request, config).await?;

        // 4. 检查错误响应
        if response.get("error").is_some() {
            self.state = PeerState::NonConnected;
            return Err(crate::error::P2PError::peer_error(
                response.get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
            ));
        }

        // 5. 更新 Peer 属性
        self.update_from_getinfo_response(&response, config);

        // 6. 检查版本并设置连接状态
        // 对应 Java: Peer.connect() 中的版本检查逻辑
        if !self.is_old_version {
            self.state = PeerState::Connected;
            self.last_updated = current_timestamp();
        } else if !self.is_blacklisted() {
            self.blacklist(format!("Old version: {}", self.version.as_deref().unwrap_or("unknown")));
        }

        Ok(response)
    }

    /// 从 getInfo 响应中更新属性
    ///
    /// 对应 Java: Peer.connect() 中的属性更新逻辑
    fn update_from_getinfo_response(
        &mut self,
        response: &serde_json::Value,
        config: &crate::config::P2PConfig,
    ) {
        // services
        if let Some(services) = response.get("services").and_then(|v| v.as_i64()) {
            self.services = services as u64;
        }

        // application
        let app = response.get("application").and_then(|v| v.as_str()).map(|s| s.to_string());

        // version（使用 set_version 进行版本检查）
        let ver = response.get("version").and_then(|v| v.as_str()).map(|s| s.to_string());
        self.set_version(ver, app.as_deref(), config);
        if app.is_some() {
            self.application = app;
        }

        // platform
        if let Some(plat) = response.get("platform").and_then(|v| v.as_str()) {
            self.platform = Some(plat.to_string());
        }

        // hallmark（可选）
        if let Some(hallmark) = response.get("hallmark").and_then(|v| v.as_str()) {
            // TODO: 实现 analyze_hallmark() 解析验证
            debug!("Received hallmark from peer {}: {}", self.address, hallmark);
        }

        // announcedAddress（可能变更）
        if let Some(new_addr) = response.get("announcedAddress").or_else(|| response.get("announced_address"))
            .and_then(|v| v.as_str())
        {
            if self.announced_address.as_deref() != Some(new_addr) && !config.ignore_announced_address {
                self.set_announced_address(new_addr.to_string());
            }
        }

        // apiPort / apiSSLPort
        if let Some(port) = response.get("apiPort").or_else(|| response.get("api_port"))
            .and_then(|v| v.as_u64())
        {
            self.api_port = Some(port as u16);
        }
        if let Some(port) = response.get("apiSSLPort").or_else(|| response.get("api_ssl_port"))
            .and_then(|v| v.as_u64())
        {
            self.api_ssl_port = Some(port as u16);
        }

        // shareAddress
        if let Some(share) = response.get("shareAddress").or_else(|| response.get("share_address"))
            .and_then(|v| v.as_bool())
        {
            self.share_address = share;
        }

        debug!("Updated peer {} from getInfo: app={}, ver={}, svc={}",
               self.address,
               self.application.as_deref().unwrap_or("?"),
               self.version.as_deref().unwrap_or("?"),
               self.services);
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

    /// 转换为完整的 Java 兼容 PeerInfo 响应格式（包含所有字段）
    ///
    /// 对应 Java: Peers.getMyPeerInfoResponse()
    pub fn to_peer_info_full(&self, config: &crate::config::P2PConfig) -> serde_json::Value {
        use serde_json::{Map as JsonMap, Value};

        // 获取基础信息
        let base = self.to_peer_info();
        let mut map: JsonMap<String, Value> = match base {
            Value::Object(m) => m,
            _ => return base,
        };

        // 补充 myPeerInfo 特有字段
        map.insert("shareAddress".to_string(),
                  Value::Bool(config.share_my_address));

        // blockchainState（动态状态）
        if let Ok(state) = config.blockchain_state.try_read() {
            map.insert("blockchainState".to_string(),
                      Value::Number(serde_json::Number::from(*state)));
        }

        // apiServerIdleTimeout
        map.insert("apiServerIdleTimeout".to_string(),
                  Value::Number(serde_json::Number::from(config.api_idle_timeout_ms)));

        // hallmark（如果设置了）
        if let Some(ref hallmark) = config.my_hallmark {
            map.insert("hallmark".to_string(),
                      Value::String(hallmark.clone()));
        }

        // disabledAPIs（可选，暂返回空数组）
        map.insert("disabledAPIs".to_string(),
                  Value::Array(vec![]));

        Value::Object(map)
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
#[derive(Clone)]
pub struct Peers {
    /// 已知节点（包括已断开但可重连的）
    known_peers: Arc<RwLock<HashMap<SocketAddr, Arc<Mutex<Peer>>>>>,
    /// 活跃的 WebSocket/TCP 连接
    active_connections: Arc<Mutex<ActiveConnections>>,
    /// 已知黑名单节点地址（对应 Java: Peers.knownBlacklistedPeers）
    /// 使用 String 存储主机地址（host 或 announcedAddress），与 Java 一致
    known_blacklisted_peers: Arc<RwLock<HashSet<String>>>,
    /// 自己节点的信息
    my_peer_info: Arc<RwLock<Peer>>,
}

impl Peers {
    pub fn new(my_peer_info: Peer) -> Self {
        Self {
            known_peers: Arc::new(RwLock::new(HashMap::new())),
            active_connections: Arc::new(Mutex::new(ActiveConnections::new())),
            known_blacklisted_peers: Arc::new(RwLock::new(HashSet::new())),
            my_peer_info: Arc::new(RwLock::new(my_peer_info)),
        }
    }

    /// 获取自己节点的 PeerInfo（用于 GetInfo 响应）
    pub async fn get_my_peer_info(&self) -> serde_json::Value {
        let my_info = self.my_peer_info.read().await;
        my_info.to_peer_info()
    }

    /// 获取完整版的 PeerInfo（包含所有字段，对应 Java getMyPeerInfoResponse）
    pub async fn get_my_peer_info_full(&self, config: &crate::config::P2PConfig) -> serde_json::Value {
        let my_info = self.my_peer_info.read().await;
        my_info.to_peer_info_full(config)
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
        let peer_clone = peer.clone();
        let entry = known.entry(addr).or_insert_with(|| Arc::new(Mutex::new(peer)));
        // 更新元数据
        let mut peer_mutex = entry.lock().await;
        peer_mutex.update_metadata(
            peer_clone.version.clone(),
            peer_clone.application.clone(),
            peer_clone.platform.clone(),
            peer_clone.services,
            peer_clone.api_port,
            peer_clone.api_ssl_port,
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

    /// 获取当前出站连接数
    ///
    /// 对应 Java: Peers.getNumberOfOutboundConnections()
    pub async fn outbound_connection_count(&self) -> usize {
        let active = self.get_active_peers().await;
        active.iter().filter(|p| !p.is_inbound).count()
    }

    /// 获取当前入站连接数
    ///
    /// 对应 Java: Peers.getNumberOfInboundConnections()
    pub async fn inbound_connection_count(&self) -> usize {
        let active = self.get_active_peers().await;
        active.iter().filter(|p| p.is_inbound).count()
    }

    /// 检查是否可以建立新的出站连接
    ///
    /// 对应 Java: Peers.canAddOutboundConnection()
    pub async fn can_add_outbound(&self, config: &crate::config::P2PConfig) -> bool {
        self.outbound_connection_count().await < config.max_outbound_connections
    }

    /// 检查是否可以接受新的入站连接
    ///
    /// 对应 Java: Peers.canAcceptInboundConnection()
    pub async fn can_accept_inbound(&self, config: &crate::config::P2PConfig) -> bool {
        self.inbound_connection_count().await < config.max_inbound_connections
    }

    /// 获取所有已知节点列表
    pub async fn get_known_peers(&self) -> Vec<Peer> {
        let known = self.known_peers.read().await;
        let mut peers = Vec::new();
        for p in known.values() {
            let p = p.lock().await;
            peers.push(p.clone());
        }
        peers
    }

    /// 获取活跃节点列表
    pub async fn get_active_peers(&self) -> Vec<Peer> {
        let conns = self.active_connections.lock().await;
        let known = self.known_peers.read().await;

        let mut active = Vec::new();
        for addr in conns.connections.iter() {
            if let Some(p) = known.get(addr) {
                let p = p.lock().await;
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

    /// 检查节点是否在黑名单中（完整检查，对应 Java: Peer.isBlacklisted()）
    ///
    /// 检查条件：
    /// 1. Peer.blacklisting_time > 0（运行时动态黑名单）
    /// 2. Peer.is_old_version（版本过旧）
    /// 3. host 在 known_blacklisted_peers 中
    /// 4. announcedAddress 在 known_blacklisted_peers 中
    pub async fn is_peer_blacklisted(&self, peer: &Peer) -> bool {
        if peer.is_blacklisted() {
            return true;
        }

        let known_bl = self.known_blacklisted_peers.read().await;
        let host = peer.address.to_string();
        if known_bl.contains(&host) {
            return true;
        }
        if let Some(ref announced) = peer.announced_address {
            if known_bl.contains(announced) {
                return true;
            }
        }

        false
    }

    /// 检查地址是否在黑名单中（通过字符串地址）
    ///
    /// 对应 Java: Peer.isBlacklisted() 中对 host 的检查
    pub async fn is_blacklisted(&self, addr: &str) -> bool {
        let known_bl = self.known_blacklisted_peers.read().await;
        if known_bl.contains(addr) {
            return true;
        }
        drop(known_bl);

        if let Ok(socket_addr) = addr.parse::<SocketAddr>() {
            self.is_blacklisted_addr(&socket_addr).await
        } else {
            false
        }
    }

    /// 检查 SocketAddr 是否在黑名单中
    ///
    /// 检查 Peer 级别黑名单 + known_blacklisted_peers
    pub async fn is_blacklisted_addr(&self, addr: &SocketAddr) -> bool {
        let known_bl = self.known_blacklisted_peers.read().await;
        if known_bl.contains(&addr.to_string()) {
            return true;
        }
        drop(known_bl);

        let known = self.known_peers.read().await;
        if let Some(peer_ref) = known.get(addr) {
            let peer = peer_ref.lock().await;
            return peer.is_blacklisted();
        }

        false
    }

    /// 添加已知黑名单节点（对应 Java: Peers.knownBlacklistedPeers）
    ///
    /// 这些节点是永久黑名单，不会自动解除
    pub async fn add_known_blacklisted(&self, addr_str: String) {
        let mut known_bl = self.known_blacklisted_peers.write().await;
        known_bl.insert(addr_str);
    }

    /// 移除已知黑名单节点
    pub async fn remove_known_blacklisted(&self, addr_str: &str) {
        let mut known_bl = self.known_blacklisted_peers.write().await;
        known_bl.remove(addr_str);
    }

    /// 检查地址是否在已知黑名单中
    pub async fn is_known_blacklisted(&self, addr_str: &str) -> bool {
        let known_bl = self.known_blacklisted_peers.read().await;
        known_bl.contains(addr_str)
    }

    /// 获取已知黑名单节点数量
    pub async fn known_blacklisted_count(&self) -> usize {
        self.known_blacklisted_peers.read().await.len()
    }

    /// 获取已知黑名单节点列表
    pub async fn known_blacklisted_peers_list(&self) -> Vec<String> {
        self.known_blacklisted_peers.read().await.iter().cloned().collect()
    }

    /// Check if a peer address is already known
    pub async fn contains_peer(&self, addr: &SocketAddr) -> bool {
        let known = self.known_peers.read().await;
        known.contains_key(addr)
    }

    /// Get count of known peers
    pub async fn known_peers_count(&self) -> usize {
        self.known_peers.read().await.len()
    }

    /// Get any peer with specified state
    /// 
    /// 对应 NRCS Java: Peers.getAnyPeer(PeerState state, boolean applyHallmark)
    pub async fn get_any_peer(&self, state: PeerState, _prefer_hallmarked: bool) -> Option<Peer> {
        let known = self.known_peers.read().await;
        
        for p in known.values() {
            let peer = p.lock().await;
            if peer.state == state && self.is_peer_blacklisted(&peer).await {
                continue;
            }
            if peer.state == state {
                return Some(peer.clone());
            }
        }
        None
    }

    /// 获取公共节点列表（非黑名单、已连接、有公告地址）
    /// 
    /// 对应 NRCS Java: Peers.getPublicPeers(PeerState state, boolean applyPullThreshold)
    pub async fn get_public_peers(&self, state: PeerState) -> Vec<Peer> {
        let known = self.known_peers.read().await;
        
        let mut public_peers = Vec::new();
        for p in known.values() {
            let peer = p.lock().await;
            if !self.is_peer_blacklisted(&peer).await
                && peer.state == state 
                && peer.announced_address.is_some() {
                public_peers.push(peer.clone());
            }
        }
        public_peers
    }

    /// 使用加权随机选择获取节点
    /// 
    /// 对应 NRCS Java: Peers.getWeightedPeer(List<IPeer> selectedPeers)
    pub async fn get_weighted_peer(&self, state: PeerState) -> Option<Peer> {
        let selected_peers = self.get_public_peers(state).await;
        
        if selected_peers.is_empty() {
            return None;
        }

        // 计算总权重
        let total_weight: u64 = selected_peers.iter()
            .map(|p| {
                let w = p.get_weight();
                if w == 0 { 1 } else { w }
            })
            .sum();

        // 使用随机数选择节点
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut hit = rng.gen_range(0..total_weight);

        for peer in &selected_peers {
            let weight = peer.get_weight();
            let weight = if weight == 0 { 1 } else { weight };
            
            if hit < weight {
                return Some(peer.clone());
            }
            hit -= weight;
        }

        // 如果没有选中，返回第一个
        selected_peers.into_iter().next()
    }

    /// Remove a peer
    /// 
    /// 对应 NRCS Java: Peers.removePeer(Peer peer)
    pub async fn remove_peer(&self, addr: &SocketAddr) {
        let mut known = self.known_peers.write().await;
        known.remove(addr);
        debug!("Removed peer: {}", addr);
    }

    /// Get peer by address
    pub async fn get_peer(&self, addr: &SocketAddr) -> Option<Arc<Mutex<Peer>>> {
        let known = self.known_peers.read().await;
        known.get(addr).cloned()
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
            last_connect_attempt: 0,
            last_inbound_request: 0,
            blacklisting_time: 0,
            blacklisting_cause: None,
            is_old_version: false,
            share_address: true,
            downloaded_volume: 0,
            uploaded_volume: 0,
            port: 8080,
        };

        let json = serde_json::to_string(&peer).unwrap();
        let decoded: Peer = serde_json::from_str(&json).unwrap();

        assert_eq!(peer.address, decoded.address);
        assert_eq!(peer.announced_address, decoded.announced_address);
        assert_eq!(peer.version, decoded.version);
        assert_eq!(peer.services, decoded.services);
    }
}