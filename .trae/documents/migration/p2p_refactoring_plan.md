# NRCS P2P 模块全面改造计划

## 📋 项目概述

**目标**: 对 rust-nrcs 的 P2P 模块进行全面改造和优化，确保与 Java NRCS 的功能完全对等。

**原则**:
- ✅ 保持现有区块同步功能正常工作
- ✅ 逐个函数/功能与 Java NRCS 对齐
- ✅ 遵循 Rust 最佳实践和项目开发规范
- ✅ 模块化设计，避免单文件过大

---

## 🔍 当前状态分析

### ✅ 已实现且功能完整

| 模块 | 文件 | 状态 | 说明 |
|------|------|------|------|
| 基础类型定义 | peer.rs | ✅ 完成 | Peer 结构体、PeerState 枚举、Peers 管理器 |
| 配置系统 | config.rs | ✅ 完成 | 完整的配置项（50+参数） |
| 错误处理 | error.rs | ✅ 完成 | 18种错误码 |
| 协议定义 | protocol.rs | ✅ 完成 | 帧编解码、请求/响应类型 |
| HTTP 服务器 | http.rs | ✅ 完成 | Axum 路由、请求处理 |
| WebSocket 服务端 | websocket.rs | ✅ 完成 | 二进制帧协议、连接管理 |
| Handler 基础框架 | handlers/mod.rs | ✅ 完成 | 12种请求类型的路由分发 |
| GetInfo 处理器 | get_info.rs | ✅ 完成 | 返回节点信息 |
| GetPeers 处理器 | get_peers.rs | ✅ 完成 | 支持分页 |
| AddPeers 处理器 | add_peers.rs | ✅ 完成 | 完整验证逻辑 |
| ProcessBlock 处理器 | process_block.rs | ✅ 完成 | 区块验证处理 |
| ProcessTransactions 处理器 | process_transactions.rs | ✅ 完成 | 交易解析验证 |
| 区块链同步 | blockchain_sync.rs | ✅ 完成 | 完整的同步逻辑 |

### ⚠️ 部分实现（需完善）

| 模块 | 文件 | 当前状态 | 缺失功能 |
|------|------|----------|----------|
| ConnectionDaemon | daemon/connection.rs | 框架存在 | connect_peer() 只有 TODO |
| DiscoveryDaemon | daemon/discovery.rs | 框架存在 | request_peers_from_peer() 和 share_my_peers() 只有 TODO |
| WebSocket 客户端 | websocket.rs (WebsocketClient) | 基础连接 | 无连接复用、无请求-响应匹配、无 GZIP |
| Peer 结构体 | peer.rs | 字段完整 | 缺少 hallmark 解析、流量更新方法 |

### ❌ 完全缺失（需新建）

| 功能 | Java 对应 | 优先级 | 复杂度 |
|------|-----------|--------|--------|
| **Peer.connect() 完整实现** | Peer.java:connect() | 🔴 高 | 高 |
| **Peer.send() 双协议发送** | Peer.java:send() | 🔴 高 | 高 |
| **Hallmark 解析与验证** | Peer.java:analyzeHallmark() | 🟡 中 | 中 |
| **Peers.sendToSomePeers() 广播** | Peers.java:sendToSomePeers() | 🔴 高 | 中 |
| **Peers.broadcastBlock() 区块广播** | Peers.java:broadcastBlock() | 🔴 高 | 中 |
| **Peers.broadcastTransaction() 交易广播** | Peers.java:broadcastTransaction() | 🔴 高 | 中 |
| **WebSocket 连接池管理** | PeerWebSocket.java | 🟡 中 | 高 |
| **请求-响应匹配机制** | PeerPostRequest.java | 🟡 中 | 中 |
| **GZIP 压缩/解压支持** | PeerWebSocket.java | 🟡 中 | 低 |
| **入站/出站连接数限制** | Peers.java | 🟡 中 | 中 |
| **节点持久化（DB）** | Peers.java:updateSavedPeers() | 🟢 低 | 中 |
| **知名节点连接** | ConnectionDaemon | 🟢 低 | 低 |
| **BundlerRate 广播** | Peers.java | 🟢 低 | 低 |
| **流量统计更新** | Peer.java | 🟢 低 | 低 |

---

## 🎯 改造计划（分阶段实施）

### 阶段一：核心通信层完善（优先级：🔴🔴🔴）

#### 任务 1.1：完善 Peer.send() 双协议通信方法
**文件**: `crates/p2p/src/peer.rs` 或新建 `crates/p2p/src/peer_client.rs`

**目标**: 实现 Java `Peer.send()` 的完整逻辑

**Java 参考代码位置**: [Peer.java#L500-L620](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/Peer.java#L500-L620)

**实现要点**:
```rust
impl Peer {
    /// 发送请求到远程节点（WebSocket 优先 + HTTP 回退）
    /// 
    /// 对应 Java: Peer.send(JSONObject request, int maxResponseSize)
    pub async fn send(
        &self,
        request: &PeerRequest,
        peers: &Arc<Peers>,
    ) -> P2PResult<serde_json::Value> {
        // 1. 检查黑名单状态
        if self.is_blacklisted() {
            return Err(P2PError::blacklisted("Peer is blacklisted"));
        }
        
        // 2. 优先使用 WebSocket（如果可用）
        if self.use_websocket {
            if let Some(response) = Self::send_via_websocket(request).await? {
                return Ok(response);
            }
        }
        
        // 3. 回退到 HTTP POST
        Self::send_via_http(&self.address, request).await
    }
}
```

**新增依赖**:
- 在 Peer 结构体中添加 `use_websocket: bool` 字段
- 添加 `inbound_websocket: Option<Arc<Mutex<WebSocketConnection>>>` 字段

**测试用例**:
- [ ] WebSocket 发送成功
- [ ] WebSocket 失败回退 HTTP
- [ ] HTTP 发送成功
- [ ] 黑名单节点拒绝发送
- [ ] 超时处理

---

#### 任务 1.2：实现 Peer.connect() 完整握手流程
**文件**: `crates/p2p/src/peer.rs`

**目标**: 实现 Java `Peer.connect()` 的完整逻辑

**Java 参考代码位置**: [Peer.java#L600-L700](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/Peer.java#L600-L700)

**实现要点**:
```rust
impl Peer {
    /// 连接到远程节点并交换信息
    /// 
    /// 对应 Java: Peer.connect()
    pub async fn connect(&mut self, peers: &Arc<Peers>) -> P2PResult<()> {
        // 1. 更新最后连接尝试时间
        self.last_connect_attempt = current_timestamp();
        
        // 2. 验证 announcedAddress 是否变更（如果已设置）
        if let Some(ref announced) = self.announced_address {
            // TODO: DNS 解析验证地址是否变更
        }
        
        // 3. 构建 getInfo 请求
        let my_info = peers.get_my_peer_info().await;
        let request = PeerRequest::new(RequestType::GetInfo, 1);
        
        // 4. 发送请求
        let response = self.send(&request, peers).await?;
        
        // 5. 检查错误响应
        if response.get("error").is_some() {
            self.state = PeerState::NonConnected;
            return Err(P2PError::from_response(&response));
        }
        
        // 6. 更新 Peer 属性
        self.update_from_getinfo_response(&response);
        
        // 7. 设置为已连接状态
        self.state = PeerState::Connected;
        self.last_updated = current_timestamp();
        
        Ok(())
    }
    
    /// 从 getInfo 响应中更新属性
    fn update_from_getinfo_response(&mut self, response: &serde_json::Value) {
        // services
        if let Some(services) = response.get("services").and_then(|v| v.as_str()) {
            if let Ok(s) = services.parse::<u64>() {
                self.services = s;
            }
        }
        
        // application
        if let Some(app) = response.get("application").and_then(|v| v.as_str()) {
            self.application = Some(app.to_string());
        }
        
        // version
        if let Some(ver) = response.get("version").and_then(|v| v.as_str()) {
            self.version = Some(ver.to_string());
        }
        
        // platform
        if let Some(plat) = response.get("platform").and_then(|v| v.as_str()) {
            self.platform = Some(plat.to_string());
        }
        
        // hallmark（可选）
        if let Some(hallmark) = response.get("hallmark").and_then(|v| v.as_str()) {
            self.analyze_hallmark(hallmark);
        }
        
        // announcedAddress（可能变更）
        if let Some(new_addr) = response.get("announcedAddress").and_then(|v| v.as_str()) {
            if self.announced_address.as_deref() != Some(new_addr) {
                self.set_announced_address(new_addr.to_string());
            }
        }
        
        // apiPort / apiSSLPort
        if let Some(port) = response.get("apiPort").and_then(|v| v.as_i64()) {
            self.api_port = Some(port as u16);
        }
        if let Some(port) = response.get("apiSSLPort").and_then(|v| v.as_i64()) {
            self.api_ssl_port = Some(port as u16);
        }
    }
}
```

**测试用例**:
- [ ] 成功连接并更新属性
- [ ] 连接失败处理
- [ ] 地址变更检测
- [ ] 错误响应处理

---

#### 任务 1.3：实现 WebSocket 连接池与请求-响应匹配
**文件**: 新建 `crates/p2p/src/connection_pool.rs`

**目标**: 实现 Java `PeerPostRequest` 的 CountDownLatch 机制

**Java 参考代码位置**: [PeerPostRequest.java](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/PeerPostRequest.java)

**实现要点**:
```rust
/// 等待中的请求（对应 Java PeerPostRequest）
pub struct PendingRequest {
    /// 完成信号
    complete: tokio::sync::oneshot::Sender<Result<String, String>>,
}

/// WebSocket 连接实例
pub struct WebSocketConnection {
    /// 远程地址
    addr: SocketAddr,
    /// WebSocket 写入端
    write: SplitSink<WebSocketStream<TcpStream>, Message>,
    /// 待响应请求映射（requestId -> PendingRequest）
    pending_requests: Arc<RwLock<HashMap<i64, PendingRequest>>>,
    /// 下一个 requestId
    next_request_id: AtomicI64,
    /// 是否已连接
    is_connected: AtomicBool,
}

impl WebSocketConnection {
    /// 发送请求并等待响应（同步阻塞风格）
    pub async fn send_and_wait(
        &self,
        request: &PeerRequest,
        timeout: Duration,
    ) -> P2PResult<serde_json::Value> {
        // 1. 生成唯一 requestId
        let request_id = self.next_request_id.fetch_add(1, Ordering::SeqCst);
        
        // 2. 创建等待通道
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        // 3. 注册待响应请求
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(request_id, PendingRequest { complete: tx });
        }
        
        // 4. 序列化并发送
        let payload = serde_json::to_vec(request)?;
        let frame = FrameCodec.encode(&payload, should_compress(&payload));
        self.write.send(Message::Binary(frame)).await?;
        
        // 5. 等待响应（带超时）
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(response_str)) => {
                let response: serde_json::Value = serde_json::from_str(&response_str)?;
                Ok(response)
            }
            Ok(Err(e)) => Err(P2PError::internal(e)),
            Err(_) => Err(P2PError::read_timeout()),
        }
    }
    
    /// 处理收到的响应（由读取线程调用）
    pub async fn handle_response(&self, request_id: i64, response: String) {
        let mut pending = self.pending_requests.write().await;
        if let Some(req) = pending.remove(&request_id) {
            let _ = req.complete.send(Ok(response));
        }
    }
    
    /// 连接关闭时清理所有待响应请求
    pub async fn cleanup_on_close(&self) {
        let mut pending = self.pending_requests.write().await;
        for (_, req) in pending.drain() {
            let _ = req.complete.send(Err("Connection closed".to_string()));
        }
    }
}
```

**测试用例**:
- [ ] 单次请求-响应匹配
- [ ] 并发多个请求
- [ ] 超时处理
- [ ] 连接关闭清理

---

### 阶段二：广播与同步机制（优先级：🔴🔴）

#### 任务 2.1：实现 Peers.sendToSomePeers() 广播机制
**文件**: `crates/p2p/src/manager.rs` 或 `crates/p2p/src/broadcast.rs`

**目标**: 实现 Java `Peers.sendToSomePeers()` 的完整逻辑

**Java 参考代码位置**: [Peers.java#L800-L900](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/Peers.java#L800-L900)

**实现要点**:
```rust
impl P2PManager {
    /// 向部分节点发送请求（加权随机选择）
    /// 
    /// 对应 Java: Peers.sendToSomePeers(JSONObject request)
    pub async fn send_to_some_peers(&self, request: &PeerRequest) -> P2PResult<Vec<(SocketAddr, P2PResult<serde_json::Value>)>> {
        let config = &self.config;
        let peers = &self.peers;
        
        // 1. 获取符合条件的公共节点列表
        let candidate_peers = peers.get_public_peers(PeerState::Connected).await;
        
        // 2. 应用 Hallmark 保护（如果启用）
        let filtered_peers = if config.enable_hallmark_protection {
            Self::apply_hallmark_filter(&candidate_peers, config.push_threshold)
        } else {
            candidate_peers
        };
        
        // 3. 限制发送数量
        let send_limit = config.send_to_peers_limit.min(filtered_peers.len());
        let selected_peers = Self::weighted_random_select(&filtered_peers, send_limit);
        
        // 4. 并发发送
        let mut results = Vec::new();
        let mut handles = Vec::new();
        
        for peer in selected_peers {
            let peers_clone = Arc::clone(peers);
            let request_clone = request.clone();
            let addr = peer.address;
            
            handles.push(tokio::spawn(async move {
                if let Some(peer_ref) = peers_clone.get_peer(&addr).await {
                    let peer = peer_ref.lock().await;
                    let result = peer.send(&request_clone, &peers_clone).await;
                    (addr, result)
                } else {
                    (addr, Err(P2PError::unknown_peer()))
                }
            }));
        }
        
        for handle in handles {
            results.push(handle.await.unwrap_or_else(|e| {
                ("0.0.0.0:0".parse().unwrap(), Err(P2PError::internal(e.to_string())))
            }));
        }
        
        Ok(results)
    }
    
    /// 应用 Hallmark 权重过滤
    fn apply_hallmark_filter(peers: &[Peer], threshold: i32) -> Vec<Peer> {
        peers.iter()
            .filter(|p| p.get_weight() >= threshold as u64)
            .cloned()
            .collect()
    }
    
    /// 加权随机选择节点
    fn weighted_random_select(peers: &[Peer], count: usize) -> Vec<Peer> {
        // 使用权重随机算法选择指定数量的节点
        // ... 实现细节
        peers.iter().take(count).cloned().collect()
    }
}
```

**测试用例**:
- [ ] 正常广播到 N 个节点
- [ ] Hallmark 过滤生效
- [ ] 发送数量限制
- [ ] 无可用节点时的处理

---

#### 任务 2.2：实现 Peers.broadcastBlock() 区块广播
**文件**: `crates/p2p/src/broadcast.rs`（新建）

**目标**: 实现 Java `Peers.broadcastBlock(IBlock)` 的完整逻辑

**Java 参考代码位置**: [Peers.java#L900-L950](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/Peers.java#L900-L950)

**实现要点**:
```rust
impl P2PManager {
    /// 广播新区块给其他节点
    /// 
    /// 对应 Java: Peers.broadcastBlock(IBlock block)
    pub async fn broadcast_block(&self, block: &Block) -> P2PResult<()> {
        // 1. 构建 processBlock 请求
        let mut request = PeerRequest::new(RequestType::ProcessBlock, 1);
        request.set("previousBlock", block.previous_block_id.unwrap_or(0).to_string());
        request.set("block", serde_json::to_value(block)?);
        request.set("timestamp", block.timestamp.to_string());
        
        // 2. 异步广播（不阻塞调用方）
        let manager = self.clone(); // 需要 Clone
        tokio::spawn(async move {
            match manager.send_to_some_peers(&request).await {
                Ok(results) => {
                    let success_count = results.iter()
                        .filter(|(_, r)| r.is_ok())
                        .count();
                    info!("Block broadcast completed: {}/{} successful", success_count, results.len());
                }
                Err(e) => {
                    warn!("Block broadcast failed: {}", e);
                }
            }
        });
        
        Ok(())
    }
}
```

**测试用例**:
- [ ] 区块广播成功
- [ ] 所有节点失败的处理
- [ ] 异步非阻塞验证

---

#### 任务 2.3：实现 Peers.broadcastTransaction() 交易广播
**文件**: `crates/p2p/src/broadcast.rs`

**目标**: 实现 Java `Peers.broadcastTransaction(ITransaction)` 的完整逻辑

**Java 参考代码位置**: [Peers.java#L950-L1000](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/Peers.java#L950-L1000)

**实现要点**:
```rust
impl P2PManager {
    /// 批量广播交易给其他节点
    /// 
    /// 对应 Java: Peers.broadcastTransaction(ITransaction transaction)
    pub async fn broadcast_transactions(&self, transactions: &[Transaction]) -> P2PResult<()> {
        if transactions.is_empty() {
            return Ok(());
        }
        
        // 1. 构建 processTransactions 请求
        let tx_jsons: Vec<serde_json::Value> = transactions.iter()
            .map(|tx| tx.to_peer_json())
            .collect();
        
        let mut request = PeerRequest::new(RequestType::ProcessTransactions, 1);
        request.set("transactions", tx_jsons);
        
        // 2. 异步广播
        let manager = self.clone();
        tokio::spawn(async move {
            match manager.send_to_some_peers(&request).await {
                Ok(results) => {
                    debug!("Transaction broadcast completed: {} peers", results.len());
                }
                Err(e) => {
                    debug!("Transaction broadcast error: {}", e);
                }
            }
        });
        
        Ok(())
    }
}
```

---

### 阶段三：后台守护进程完善（优先级：🔴🟡）

#### 任务 3.1：完善 ConnectionDaemon 核心逻辑
**文件**: `crates/p2p/src/daemon/connection.rs`

**当前问题**: `connect_peer()` 方法只有 TODO 占位符

**改造内容**:

```rust
impl ConnectionDaemon {
    /// 连接到单个节点（完整实现）
    async fn connect_peer(peers: &Arc<Peers>, peer: &Peer) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 1. 黑名单检查
        if peers.is_blacklisted_addr(&peer.address).await {
            return Err("Peer is blacklisted".into());
        }
        
        // 2. 获取可变引用
        if let Some(peer_ref) = peers.get_peer(&peer.address).await {
            let mut p = peer_ref.lock().await;
            
            // 3. 调用 Peer.connect() 方法
            match p.connect(peers).await {
                Ok(_) => {
                    info!("Successfully connected to peer: {}", peer.address);
                    
                    // 4. 注册活跃连接
                    peers.add_connection(peer.address).await;
                    Ok(())
                }
                Err(e) => {
                    warn!("Failed to connect to peer {}: {}", peer.address, e);
                    
                    // 5. 连接失败处理（根据错误类型决定是否加入黑名单）
                    if matches!(e.code(), ErrorCode::ConnectionTimeout | ErrorCode::ReadTimeout) {
                        p.blacklist(format!("Connection failed: {}", e));
                    } else {
                        p.deactivate();
                    }
                    Err(e.into())
                }
            }
        } else {
            Err("Peer not found".into())
        }
    }
    
    /// 连接知名节点（启动时或定期执行）
    async fn connect_well_known_peers(peers: &Arc<Peers>, config: &P2PConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 从配置或数据库加载知名节点列表
        let well_known_addrs = Self::get_well_known_peers(config);
        
        for addr in well_known_addrs {
            if !peers.contains_peer(&addr).await {
                let peer = Peer::new(addr, false); // outbound
                peers.register_peer(peer).await;
                
                // 尝试连接
                let all_peers = peers.get_known_peers().await;
                if let Some(p) = all_peers.iter().find(|p| p.address == addr) {
                    let _ = Self::connect_peer(peers, p).await;
                }
            }
        }
        
        Ok(())
    }
    
    /// 获取知名节点列表（从配置文件或硬编码）
    fn get_well_known_peers(_config: &P2PConfig) -> Vec<SocketAddr> {
        // TODO: 从配置文件读取或使用默认种子节点
        vec![
            "127.0.0.1:16974".parse().unwrap(), // 测试用
        ]
    }
    
    /// 清理过期的入站连接
    async fn cleanup_inbound_connections(peers: &Arc<Peers>, _config: &P2PConfig, now: i64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let active_peers = peers.get_active_peers().await;
        
        for peer in active_peers {
            // 如果是入站连接且超过 3600 秒没有活动，断开
            if peer.is_inbound && now - peer.last_inbound_request > 3600 {
                peers.remove_connection(&peer.address).await;
                
                // 更新 peer 状态
                if let Some(peer_ref) = peers.get_peer(&peer.address).await {
                    let mut p = peer_ref.lock().await;
                    p.deactivate();
                }
            }
        }
        
        Ok(())
    }
}
```

**测试用例**:
- [ ] 成功连接新节点
- [ ] 黑名单节点跳过
- [ ] 连接失败后的黑名单处理
- [ ] 入站连接过期清理

---

#### 任务 3.2：完善 DiscoveryDaemon 核心逻辑
**文件**: `crates/p2p/src/daemon/discovery.rs`

**当前问题**: `request_peers_from_peer()` 和 `share_my_peers()` 只有 TODO

**改造内容**:

```rust
impl DiscoveryDaemon {
    /// 从已连接节点获取更多节点信息
    async fn request_peers_from_peer(
        peers: &Arc<Peers>,
        peer: &Peer,
        _config: &P2PConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::websocket::WebsocketClient;
        use crate::protocol::{PeerRequest, RequestType};
        
        // 1. 构建 getPeers 请求
        let request = PeerRequest::new(RequestType::GetPeers, 1);
        
        // 2. 发送请求
        match WebsocketClient::send_request(peer.address, request).await {
            Ok(response) => {
                // 3. 解析返回的节点列表
                if let Some(peers_arr) = response.get("peers").and_then(|v| v.as_array()) {
                    let mut added = 0usize;
                    
                    for peer_info in peers_arr {
                        // 提取地址信息
                        if let Some(addr_str) = peer_info.get("announcedAddress")
                            .or_else(|| peer_info.get("address"))
                            .and_then(|v| v.as_str())
                        {
                            if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                                // 检查是否已知
                                if !peers.contains_peer(&addr).await {
                                    let new_peer = Peer::new(addr, false);
                                    peers.register_peer(new_peer).await;
                                    added += 1;
                                }
                            }
                        }
                    }
                    
                    if added > 0 {
                        info!("Discovered {} new peers from {}", added, peer.address);
                    }
                }
            }
            Err(e) => {
                debug!("Failed to get peers from {}: {}", peer.address, e);
            }
        }
        
        Ok(())
    }
    
    /// 向其他节点分享自己的已知节点
    async fn share_my_peers(
        peers: &Arc<Peers>,
        _config: &P2PConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::websocket::WebsocketClient;
        use crate::protocol::{PeerRequest, RequestType};
        
        // 1. 获取可分享的节点列表
        let all_peers = peers.get_known_peers().await;
        let mut shareable = Vec::new();
        
        for peer in &all_peers {
            // 条件：非黑名单、有公告地址、允许共享地址
            if !peers.is_blacklisted_addr(&peer.address).await
                && peer.announced_address.is_some()
                && peer.share_address
            {
                // 构建节点信息（只包含必要字段）
                let mut peer_info = serde_json::Map::new();
                if let Some(ref addr) = peer.announced_address {
                    // 解析 address:port 格式
                    if let Some((host, port)) = addr.rsplit_once(':') {
                        peer_info.insert("address".into(), host.into());
                        peer_info.insert("port".into(), port.parse::<i64>().unwrap_or(16974).into());
                    }
                }
                peer_info.insert("services".into(), serde_json::Value::Number(serde_json::Number::from(peer.services)));
                
                shareable.push(serde_json::Value::Object(peer_info));
            }
        }
        
        if shareable.is_empty() {
            return Ok(());
        }
        
        // 2. 选择几个已连接节点进行分享
        let connected = peers.get_public_peers(PeerState::Connected).await;
        let share_targets: Vec<&Peer> = connected.iter()
            .take(5) // 最多分享给 5 个节点
            .collect();
        
        // 3. 构建 addPeers 请求
        let mut request = PeerRequest::new(RequestType::AddPeers, 1);
        request.set("peers", shareable);
        
        // 4. 并发发送
        for target in share_targets {
            let target_addr = target.address;
            let req_clone = request.clone();
            let peers_clone = Arc::clone(peers);
            
            tokio::spawn(async move {
                match WebsocketClient::send_request(target_addr, req_clone).await {
                    Ok(resp) => {
                        if let Some(added) = resp.get("added").and_then(|v| v.as_i64()) {
                            debug!("Shared {} peers with {} (accepted {})", 
                                   shareable.len(), target_addr, added);
                        }
                    }
                    Err(e) => {
                        debug!("Failed to share peers with {}: {}", target_addr, e);
                    }
                }
            });
        }
        
        Ok(())
    }
}
```

**测试用例**:
- [ ] 成功从 peer 发现新节点
- [ ] 分享节点成功
- [ ] 无可分享节点的处理
- [ ] 网络错误的容错

---

### 阶段四：辅助功能增强（优先级：🟡）

#### 任务 4.1：实现 Hallmark 解析与验证
**文件**: `crates/p2p/src/hallmark.rs`（新建）

**目标**: 实现 Java `Peer.analyzeHallmark(String hallmark)`

**Java 参考代码位置**: [Peer.java#analyzeHallmark()](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/Peer.java)

**实现要点**:
```rust
/// Hallmark 信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HallmarkInfo {
    /// 公钥
    public_key: Option<String>,
    /// 权重
    weight: u32,
    /// 日期
    date: Option<u32>,
    /// 是否有效
    is_valid: bool,
}

impl Peer {
    /// 解析并验证 Hallmark
    /// 
    /// 对应 Java: Peer.analyzeHallmark(String hallmark)
    pub fn analyze_hallmark(&mut self, hallmark: &str) {
        // TODO: 实现 Base64 解码 + 公钥提取 + 签名验证
        // Hallmark 格式: base64_encoded_data
        // 包含: public_key + weight + date + signature
        
        let info = match Self::decode_hallmark(hallmark) {
            Ok(info) => info,
            Err(_) => {
                self.hallmark = None;
                self.hallmark_weight = 0;
                return;
            }
        };
        
        self.hallmark = Some(hallmark.to_string());
        self.hallmark_weight = info.weight;
    }
    
    fn decode_hallmark(_hallmark: &str) -> Result<HallmarkInfo, String> {
        // TODO: 实现解码逻辑
        Ok(HallmarkInfo {
            public_key: None,
            weight: 0,
            date: None,
            is_valid: false,
        })
    }
}
```

---

#### 任务 4.2：实现 GZIP 压缩/解压优化
**文件**: `crates/p2p/src/protocol.rs`（已有基础实现，需集成到 WebSocket 客户端）

**改造点**:
- 在 `FrameCodec.encode()` 中自动判断是否压缩（>=256 bytes）
- 在 `FrameCodec.decode()` 中自动解压（检查 flags bit0）
- 在 `WebsocketClient.send_request()` 中使用 FrameCodec 编码

---

#### 任务 4.3：实现连接数统计与限制
**文件**: `crates/p2p/src/peer.rs` (Peers struct)

**新增方法**:
```rust
impl Peers {
    /// 获取当前出站连接数
    pub async fn outbound_connection_count(&self) -> usize {
        let active = self.get_active_peers().await;
        active.iter().filter(|p| !p.is_inbound).count()
    }
    
    /// 获取当前入站连接数
    pub async fn inbound_connection_count(&self) -> usize {
        let active = self.get_active_peers().await;
        active.iter().filter(|p| p.is_inbound).count()
    }
    
    /// 检查是否可以建立新的出站连接
    pub async fn can_add_outbound(&self, config: &P2PConfig) -> bool {
        self.outbound_connection_count().await < config.max_outbound_connections
    }
    
    /// 检查是否接受新的入站连接
    pub async fn can_accept_inbound(&self, config: &P2PConfig) -> bool {
        self.inbound_connection_count().await < config.max_inbound_connections
    }
}
```

---

#### 任务 4.4：实现节点持久化（数据库存储）
**文件**: 新建 `crates/p2p/src/persistence.rs`

**目标**: 实现 Java `updateSavedPeers()` 和 `loadSavedPeers()`

**表结构参考** (NRCS schema):
```sql
CREATE TABLE IF NOT EXISTS PEER (
    ADDRESS VARCHAR(100) NOT NULL,
    ANNOUNCED_ADDRESS VARCHAR(100),
    SERVICES BIGINT DEFAULT 0,
    STATE TINYINT DEFAULT 0,
    LAST_UPDATED INT NOT NULL,
    LAST_CONNECT_ATTEMPT INT DEFAULT 0,
    BLACKLISTING_TIME INT DEFAULT 0,
    BLACKLISTING_CAUSE VARCHAR(200),
    IS_OLD_VERSION BOOLEAN DEFAULT FALSE,
    SHARE_ADDRESS BOOLEAN DEFAULT TRUE,
    DOWNLOADED_VOLUME BIGINT DEFAULT 0,
    UPLOADED_VOLUME BIGINT DEFAULT 0,
    APPLICATION VARCHAR(20),
    VERSION VARCHAR(10),
    PLATFORM VARCHAR(30),
    API_PORT INT DEFAULT 0,
    API_SSL_PORT INT DEFAULT 0,
    HALLMARK VARCHAR(500),
    HALLMARK_WEIGHT INT DEFAULT 0,
    PRIMARY KEY (ADDRESS)
);
```

**实现要点**:
```rust
pub struct PeerPersistence {
    pool: SqlitePool,
}

impl PeerPersistence {
    /// 保存节点到数据库
    pub async fn save_peer(&self, peer: &Peer) -> RepositoryResult<()> {
        sqlx::query_as::<_, (i64,)>(
            "INSERT OR REPLACE INTO PEER (...) VALUES (...)"
        )
        .bind(...)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    
    /// 从数据库加载所有节点
    pub async fn load_all_peers(&self) -> RepositoryResult<Vec<Peer>> {
        let rows = sqlx::query_as::<_, PeerModel>(
            "SELECT * FROM PEER"
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|m| m.to_domain()).collect())
    }
}
```

---

### 阶段五：BundlerRate 广播（优先级：🟢）

#### 任务 5.1：实现 BundlerRate 定时广播
**文件**: 完善 `crates/p2p/src/handlers/bundler_rate.rs` 和新建 `crates/p2p/src/daemon/bundler_rate_daemon.rs`

**目标**: 实现 Java BundlerRate 广播机制

**Java 参考代码位置**: [Peers.java#BUNDLER_RATE_BROADCAST_INTERVAL](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/Peers.java)

**实现要点**:
```rust
pub struct BundlerRateDaemon {
    peers: Arc<Peers>,
    config: P2PConfig,
    running: Arc<RwLock<bool>>,
    last_broadcast_time: Arc<RwLock<i64>>,
    last_rates_hash: Arc<RwLock<u64>>,
}

impl BundlerRateDaemon {
    /// 每 30 分钟或有变化时广播
    async fn broadcast_loop(&self) {
        loop {
            if !*self.running.read().await { break; }
            
            let now = current_timestamp();
            let last_time = *self.last_broadcast_time.read().await;
            
            // 检查是否到了广播时间
            if now - last_time >= self.config.bundler_rate_broadcast_interval_secs as i64 {
                // 获取当前的 bundler rates
                if let Some(rates) = self.get_current_rates().await {
                    let rates_hash = Self::calculate_hash(&rates);
                    
                    // 检查是否有变化
                    if rates_hash != *self.last_rates_hash.read().await {
                        self.do_broadcast(&rates).await;
                        *self.last_broadcast_time.write().await = now;
                        *self.last_rates_hash.write().await = rates_hash;
                    }
                }
            }
            
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
    
    async fn do_broadcast(&self, rates: &serde_json::Value) {
        let mut request = PeerRequest::new(RequestType::BundlerRate, 2); // protocol=2
        request.set("rates", rates);
        
        // 使用 sendToSomePeers 广播
        // ...
    }
}
```

---

## 📊 实施优先级总结

### 第一批（核心功能 - 必须完成）⭐⭐⭐

| 序号 | 任务 | 预估工作量 | 依赖关系 |
|------|------|------------|----------|
| 1.1 | Peer.send() 双协议通信 | 3-4 小时 | 无 |
| 1.2 | Peer.connect() 完整握手 | 2-3 小时 | 1.1 |
| 1.3 | WebSocket 连接池管理 | 4-5 小时 | 无 |
| 2.1 | sendToSomePeers() 广播 | 2-3 小时 | 1.1, 1.3 |
| 2.2 | broadcastBlock() 区块广播 | 1-2 小时 | 2.1 |
| 2.3 | broadcastTransaction() 交易广播 | 1-2 小时 | 2.1 |
| 3.1 | ConnectionDaemon 完善 | 3-4 小时 | 1.2 |
| 3.2 | DiscoveryDaemon 完善 | 2-3 小时 | 1.1 |

**小计**: 约 18-26 小时

### 第二批（增强功能 - 重要）⭐⭐

| 序号 | 任务 | 预估工作量 | 依赖关系 |
|------|------|------------|----------|
| 4.1 | Hallmark 解析验证 | 2-3 小时 | 无 |
| 4.2 | GZIP 压缩优化 | 1-2 小时 | 1.3 |
| 4.3 | 连接数统计限制 | 1-2 小时 | 无 |
| 4.4 | 节点持久化 DB | 3-4 小时 | 无 |

**小计**: 约 7-11 小时

### 第三批（锦上添花）⭐

| 序号 | 任务 | 预估工作量 | 依赖关系 |
|------|------|------------|----------|
| 5.1 | BundlerRate 广播 | 2-3 小时 | 2.1 |

**小计**: 约 2-3 小时

**总计预估**: 约 27-40 小时

---

## 🧪 测试策略

### 单元测试覆盖要求

每个任务完成后必须包含：

1. **正常路径测试**
   - 功能正确性验证
   
2. **边界条件测试**
   - 空输入、最大值、超时等
   
3. **错误处理测试**
   - 网络异常、格式错误、权限不足等
   
4. **并发安全测试**
   - 多线程同时访问、竞态条件

### 集成测试场景

1. **完整连接流程测试**
   ```
   启动 → 加载种子节点 → 连接 → getInfo → 状态更新 → 心跳保持
   ```

2. **区块同步流程测试**
   ```
   检测高度差异 → getCumulativeDifficulty → getMilestoneBlockIds → 
   getNextBlockIds → getNextBlocks → verify → 存储
   ```

3. **交易广播流程测试**
   ```
   创建交易 → 本地验证 → broadcastTransaction → 接收方验证 → 内存池
   ```

4. **节点发现流程测试**
   ```
   启动 → getMorePeers → addPeers → 新节点注册 → 连接新节点
   ```

---

## ⚠️ 注意事项

### 兼容性保证

1. **协议兼容**: 必须与 Java NRCS 的二进制帧格式完全一致
   - Version(4) + RequestID(8) + Flags(4) + Length(4) + Payload
   
2. **JSON 格式兼容**: 所有请求/响应字段名必须与 Java 一致
   - 使用 camelCase（如 `requestType`, `announcedAddress`）
   
3. **行为兼容**: 错误码、超时时间、重试策略必须一致

### 性能考虑

1. **异步优先**: 所有 I/O 操作必须使用 async/await
2. **并发控制**: 使用 Semaphore 限制并发连接数
3. **内存管理**: 及时清理断开连接的资源
4. **批量操作**: 节点发现、区块下载等使用批量请求

### 安全考虑

1. **黑名单机制**: 自动隔离异常节点
2. **连接数限制**: 防止 DDoS 攻击
3. **输入验证**: 所有外部数据必须验证
4. **Hallmark 验证**: 可选的身份认证

---

## 📁 文件变更清单

### 新增文件

```
crates/p2p/src/
├── connection_pool.rs      # WebSocket 连接池管理
├── broadcast.rs             # 广播机制（区块/交易）
├── hallmark.rs              # Hallmark 解析验证
├── persistence.rs           # 节点持久化（DB）
└── daemon/
    └── bundler_rate_daemon.rs  # BundlerRate 广播守护进程
```

### 修改文件

```
crates/p2p/src/
├── peer.rs                  # 增加 connect/send 方法
├── manager.rs               # 增加广播方法
├── websocket.rs             # 完善客户端连接复用
├── protocol.rs              # 集成 GZIP 压缩
├── daemon/connection.rs     # 实现核心连接逻辑
├── daemon/discovery.rs      # 实现节点发现逻辑
└── lib.rs                   # 导出新模块
```

---

## ✅ 验收标准

### 功能完整性

- [ ] 所有 Java NRCS 的 Peer API 都有对应的 Rust 实现
- [ ] 区块同步功能保持正常工作
- [ ] 可以成功与 Java NRCS 节点互联
- [ ] 节点发现、连接、维护自动化运行

### 代码质量

- [ ] `cargo clippy -- -D warnings` 无警告
- [ ] `cargo test --lib` 全部通过
- [ ] 核心模块单元测试覆盖率 ≥ 95%
- [ ] 所有公开 API 有文档注释

### 性能指标

- [ ] 单节点连接建立 < 2 秒
- [ ] 区块广播延迟 < 100ms
- [ ] 支持 50+ 并发连接
- [ ] 内存占用稳定（无明显泄漏）

---

**版本**: v1.1 (含配置管理对齐)  
**创建日期**: 2026-04-28  
**更新日期**: 2026-04-28  
**预计完成时间**: 2-3 周（按每天 2-3 小时计算）

---

## 🔧 附录 A：配置管理完整对齐（Java NRCS ↔ Rust）

### A.1 Java NRCS 完整配置项清单

基于 `nrcs-default.properties` 和 `nrcs.properties` 文件分析，以下是 Java NRCS 所有 Peer/P2P 相关的配置项：

#### A.1.1 基础网络配置

| 配置项 | Java 默认值 | 说明 | Rust 当前状态 |
|--------|------------|------|--------------|
| `nrcs.shareMyAddress` | `false` | 是否公告自己的地址给其他节点 | ✅ `share_my_address` |
| `nrcs.myAddress` | `""` | 外部可见的地址（IP:Port） | ⚠️ `my_address` (格式待确认) |
| `nrcs.myPlatform` | 自动获取 | 平台信息（OS + ARCH） | ✅ `my_platform` |
| `nrcs.myHallmark` | `""` | Hallmark 身份标识 | ✅ `my_hallmark` |

#### A.1.2 服务端口配置

| 配置项 | Java 默认值 | 说明 | Rust 当前状态 |
|--------|------------|------|--------------|
| `nrcs.peerServerPort` | `17974` (主网) / `16974` (测试网) | P2P 监听端口 | ✅ `listen_addr` (需拆分) |
| `nrcs.peerServerHost` | `0.0.0.0` | P2P 监听地址 | ✅ `listen_addr` (需拆分) |
| `API.openAPIPort` | `7876` (主网) / `6876` (测试网) | API 端口（用于 myPeerInfo） | ❌ **缺失** |
| `API.openAPISSLPort` | `7877` / `6877` | API SSL 端口 | ❌ **缺失** |
| `nrcs.apiServerIdleTimeout` | `30000` | API 服务器空闲超时(ms) | ❌ **缺失** |
| `nrcs.peerServerIdleTimeout` | `300000` | P2P 服务器空闲超时(ms) | ⚠️ 单位不一致 (秒 vs 毫秒) |

#### A.1.3 连接管理配置

| 配置项 | Java 默认值 | 说明 | Rust 当前状态 |
|--------|------------|------|--------------|
| `nrcs.maxNumberOfConnectedPublicPeers` | `20` | 最大公网连接数 | ✅ `max_connections` (但语义不同) |
| `nrcs.maxNumberOfInboundConnections` | `100` | 最大入站连接数 | ✅ `max_inbound_connections` |
| `nrcs.maxNumberOfOutboundConnections` | `20` | 最大出站连接数 | ✅ `max_outbound_connections` |
| `nrcs.connectTimeout` | `2000` | 连接超时(ms) | ⚠️ 值不同 (4000 vs 2000) |
| `nrcs.readTimeout` | `4000` | 读取超时(ms) | ✅ `read_timeout_ms` |
| `nrcs.useWebSockets` | `true` | 是否使用 WebSocket | ✅ `use_websockets` |
| `nrcs.isGzipEnabled` | `true` | 是否启用 GZIP 压缩 | ❌ **缺失** |
| `nrcs.minCompressSize` | `256` | 最小压缩大小(bytes) | ✅ `min_compress_size` |

#### A.1.4 黑名单与安全配置

| 配置项 | Java 默认值 | 说明 | Rust 当前状态 |
|--------|------------|------|--------------|
| `nrcs.blacklistingPeriod` | `600000` (10分钟) | 黑名单持续时间(ms) | ⚠️ **单位错误** (秒 vs 毫秒，值也不同) |
| `nrcs.enableHallmarkProtection` | `true` | 启用 Hallmark 保护 | ✅ `enable_hallmark_protection` |
| `nrcs.pushThreshold` | `0` | 发送数据时的 Hallmark 权重阈值 | ✅ `push_threshold` |
| `nrcs.pullThreshold` | `0` | 请求数据时的 Hallmark 权重阈值 | ✅ `pull_threshold` |
| `nrcs.sendToPeersLimit` | `10` | 广播时最大发送节点数 | ✅ `send_to_peers_limit` |
| `nrcs.ignorePeerAnnouncedAddress` | `false` | 忽略节点公告地址变更 | ❌ **缺失** |
| `nrcs.hideErrorDetails` | `false` | 隐藏错误详情 | ❌ **缺失** |
| `nrcs.knownBlacklistedPeers` | `""` | 已知黑名单节点列表 | ❌ **缺失** |

#### A.1.5 节点发现与持久化配置

| 配置项 | Java 默认值 | 说明 | Rust 当前状态 |
|--------|------------|------|--------------|
| `nrcs.maxNumberOfKnownPeers` | `2000` | 最大已知节点数 | ✅ `max_known_peers` |
| `nrcs.minNumberOfKnownPeers` | `1000` | 最小已知节点数 | ⚠️ 值不同 (100 vs 1000) |
| `nrcs.usePeersDb` | `true` | 使用数据库存储节点 | ✅ `use_peers_db` |
| `nrcs.savePeers` | `true` | 保存节点到数据库 | ✅ `save_peers` |
| `nrcs.getMorePeers` | `true` | 从已连接节点获取更多节点 | ✅ `get_more_peers` |
| `nrcs.defaultPeers` | 种子节点列表 | 默认初始节点 | ❌ **缺失** |
| `nrcs.wellKnownPeers` | `""` | 知名节点（保持连接） | ❌ **缺失** |
| `nrcs.testnetPeers` | `""` | 测试网知名节点 | ❌ **缺失** |
| `nrcs.defaultTestnetPeers` | `""` | 测试网默认初始节点 | ❌ **缺失** |

#### A.1.6 高级功能配置

| 配置项 | Java 默认值 | 说明 | Rust 当前状态 |
|--------|------------|------|--------------|
| `nrcs.enablePeerUPnP` | `false` | 启用 UPnP 端口映射 | ❌ **缺失** |
| `nrcs.cjdnsOnly` | `false` | 仅允许 cjdns 地址 | ❌ **缺失** |
| `nrcs.offline` | `false` | 离线模式（不连接任何节点） | ❌ **缺失** |
| `nrcs.remotePeerResourceBase` | `/nrcs` | 远程节点资源基础路径 | ❌ **缺失** |
| `nrcs.enablePeerServerDoSFilter` | `true` | 启用 DoS 防护过滤器 | ❌ **缺失** |
| `nrcs.enablePeerServerGZIPFilter` | `true` | 启用 HTTP GZIP 压缩 | ❌ **缺失** |
| `nrcs.dumpPeersVersion` | `false` | 导出节点版本信息 | ❌ **缺失** |
| `nrcs.rePeerStackWalk` | `false` | 调试用：重新执行 peer stack walk | ❌ **缺失** |
| `nrcs.verifyBatchOfBlocks` | `-1` | 区块批量验证 peers 数量 | ❌ **缺失** |

#### A.1.7 DoS 防护子配置

| 配置项 | Java 默认值 | 说明 | Rust 当前状态 |
|--------|------------|------|--------------|
| `nrcs.peerServerDoSFilter.maxRequestsPerSec` | `30` | 每秒最大请求数 | ❌ **缺失** |
| `nrcs.peerServerDoSFilter.delayMs` | `1000` | 超限延迟时间(ms) | ❌ **缺失** |
| `nrcs.peerServerDoSFilter.maxRequestMs` | `300000` | 单请求最大处理时间(ms) | ❌ **缺失** |
| `nrcs.peerServerDoSFilter.throttleMs` | `1000` | 节流时间(ms) | ❌ **缺失** |
| `nrcs.peerServerDoSFilter.maxThrottleMs` | `30000` | 最大节流时间(ms) | ❌ **缺失** |
| `nrcs.peerServerDoSFilter.maxWaitMs` | `50000` | 最大等待时间(ms) | ❌ **缺失** |
| `nrcs.peerServerDoSFilter.maxIdleTrackerMs` | `30000` | 空闲追踪器时间(ms) | ❌ **缺失** |
| `nrcs.peerServerDoSFilter.trackSessions` | `false` | 是否跟踪会话 | ❌ **缺失** |
| `nrcs.peerServerDoSFilter.remotePort` | `80` | 远程端口（用于白名单） | ❌ **缺失** |
| `nrcs.peerServerDoSFilter.insertPort` | `80` | 插入端口（用于白名单） | ❌ **缺失** |
| `nrcs.peerServerDoSFilter.whitelist` | `""` | IP 白名单 | ❌ **缺失** |
| `nrcs.peerServerDoSFilter.blacklist` | `""` | IP 黑名单 | ❌ **缺失** |

### A.2 myPeerInfo 字段对齐

Java NRCS 的 `getMyPeerInfoResponse()` 返回以下字段：

```java
// Peers.java - getMyPeerInfoResponse()
JSONObject response = new JSONObject();
response.put("application", Constant.APPLICATION);        // "NRCS"
response.put("version", Constant.VERSION);                  // "2.1.0"
response.put("platform", myPlatform);                       // "macos arm64"
response.put("services", getAllServices());                // 服务标志位
response.put("announcedAddress", announcedAddress);         // 公告地址
response.put("shareAddress", shareMyAddress);               // 是否共享
response.put("apiPort", API.openAPIPort);                   // API 端口
response.put("apiSSLPort", API.openAPISSLPort);             // API SSL 端口
response.put("blockchainState", blockchain.getState());     // 区块链状态
if (myHallmark != null) {
    response.put("hallmark", myHallmark);                 // Hallmark 标识
}
// 可选字段
response.put("disabledAPIs", API.getDisabledAPIS());       // 禁用的 API
response.put("apiServerIdleTimeout", apiServerIdleTimeout);// API 空闲超时
```

**Rust 当前实现对比** ([peer.rs#L221-L242](file:///Volumes/DATA/data/develop/git/rust-nrcs/crates/p2p/src/peer.rs#L221-L242)):

| 字段 | Java | Rust | 状态 |
|------|------|------|------|
| `application` | ✅ | ✅ | 一致 |
| `version` | ✅ | ✅ | 一致 |
| `platform` | ✅ | ✅ | 一致 |
| `services` | ✅ | ✅ | 一致 |
| `announcedAddress` | ✅ | ✅ | 一致 |
| `shareAddress` | ✅ | ✅ | 一致 |
| `apiPort` | ✅ | ✅ | 一致 |
| `apiSSLPort` | ✅ | ✅ | 一致 |
| `blockchainState` | ✅ | ❌ | **缺失** |
| `hallmark` | ✅ | ❌ | **缺失** |
| `disabledAPIs` | ✅ | ❌ | **缺失** |
| `apiServerIdleTimeout` | ✅ | ❌ | **缺失** |

### A.3 配置问题汇总与修复方案

#### 🔴 关键问题（必须修复）

| 问题ID | 问题描述 | 影响 | 修复方案 |
|--------|---------|------|----------|
| CFG-01 | **blacklistingPeriod 单位错误**: Java 用毫秒(600000=10min)，Rust 用秒(3600=1h) | 黑名单时间不一致，可能导致过早/过晚解除 | 统一为毫秒，默认值改为 600000 |
| CFG-02 | **minNumberOfKnownPeers 值错误**: Java=1000, Rust=100 | 可能导致过早清理已知节点 | 改为 1000 |
| CFG-03 | **connectTimeout 值不一致**: Java=2000ms, Rust=4000ms | 连接超时行为差异 | 改为 2000 或添加注释说明差异原因 |
| CFG-04 | **缺少 API 端口配置**: apiPort/apiSSLPort 未在 P2PConfig 中 | myPeerInfo 响应不完整 | 新增 `api_port` 和 `api_ssl_port` 字段 |
| CFG-05 | **myPeerInfo 缺少字段**: blockchainState/hallmark/disabledAPIs/apiServerIdleTimeout | getInfo 响应与 Java 不兼容 | 补充这些字段到 to_peer_info() 方法 |

#### 🟡 重要问题（建议修复）

| 问题ID | 问题描述 | 影响 | 修复方案 |
|--------|---------|------|----------|
| CFG-06 | **缺少种子节点配置**: defaultPeers/wellKnownPeers/testnetPeers | 无法自动发现节点 | 新增配置项和加载逻辑 |
| CFG-07 | **缺少离线模式**: offline 配置 | 无法完全断开网络 | 新增 offline 模式支持 |
| CFG-08 | **缺少黑名单预加载**: knownBlacklistedPeers | 启动时不隔离已知恶意节点 | 新增预加载逻辑 |
| CFG-09 | **缺少 ignorePeerAnnouncedAddress**: 忽略公告地址变更 | 安全性差异 | 新增配置项 |
| CFG-10 | **peerServerIdleTimeout 单位可能混淆**: Java=毫秒(300000), Rust=秒(300) | 实际超时时间差 1000 倍 | 明确单位或统一 |

#### 🟢 锦上添花（可选实现）

| 问题ID | 问题描述 | 影响 | 修复方案 |
|--------|---------|------|----------|
| CFG-11 | **缺少 UPnP 支持**: enablePeerUPnP | NAT 穿透不支持 | 可选实现（依赖外部库） |
| CFG-12 | **缺少 DoS 防护配置**: 12+ 个 DoSFilter 子配置 | 无速率限制保护 | 可选实现（Axum middleware） |
| CFG-13 | **缺少 cjdnsOnly 支持**: 仅允许 cjdns 地址 | 特殊网络环境不支持 | 可选实现 |
| CFG-14 | **缺少 GZIP 开关**: isGzipEnabled | 无法禁用压缩 | 新增配置项 |
| CFG-15 | **缺少 dumpPeersVersion**: 节点版本导出 | 调试功能缺失 | 可选实现 |

### A.4 配置改造任务清单

#### 任务 CFG-T1: 修复关键配置值不一致
**文件**: `crates/p2p/src/config.rs`, `crates/blockchain-types/src/constants.rs`

**改动内容**:
```rust
// constants.rs - 修正常量值
pub const BLACKLISTING_PERIOD_SECS: i64 = 600; // 改为 600 秒 = 10 分钟 (原 3600)
pub const MIN_KNOWN_PEERS: usize = 1000;      // 改为 1000 (原 100)
pub const CONNECT_TIMEOUT_MS: u64 = 2000;      // 改为 2000 (原 4000)

// config.rs - 新增缺失字段
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct P2PConfig {
    // ... 已有字段 ...
    
    // ===== 新增字段 =====
    /// API 端口（对应 Java: API.openAPIPort）
    #[serde(default = "default_api_port")]
    pub api_port: u16,
    
    /// API SSL 端口（对应 Java: API.openAPISSLPort）
    #[serde(default = "default_api_ssl_port")]
    pub api_ssl_port: u16,
    
    /// API 服务器空闲超时毫秒（对应 Java: nrcs.apiServerIdleTimeout）
    #[serde(default = "default_api_idle_timeout_ms")]
    pub api_idle_timeout_ms: u64,
    
    /// 是否启用 GZIP 压缩（对应 Java: nrcs.isGzipEnabled）
    #[serde(default = "default_gzip_enabled")]
    pub gzip_enabled: bool,
    
    /// 是否忽略节点公告地址变更（对应 Java: nrcs.ignorePeerAnnouncedAddress）
    #[serde(default)]
    pub ignore_announced_address: bool,
    
    /// 是否隐藏错误详情（对应 Java: nrcs.hideErrorDetails）
    #[serde(default)]
    pub hide_error_details: bool,
    
    /// 离线模式（对应 Java: nrcs.offline）
    #[serde(default)]
    pub offline_mode: bool,
    
    /// 种子节点列表（对应 Java: nrcs.defaultPeers）
    #[serde(default)]
    pub default_peers: Vec<String>,
    
    /// 知名节点列表（对应 Java: nrcs.wellKnownPeers）
    #[serde(default)]
    pub well_known_peers: Vec<String>,
    
    /// 已知黑名单节点（对应 Java: nrcs.knownBlacklistedPeers）
    #[serde(default)]
    pub known_blacklisted_peers: Vec<String>,
    
    /// 区块链状态（动态更新）
    #[serde(skip)]
    pub blockchain_state: Arc<RwLock<i32>>,
}

fn default_api_port() -> u16 { 7876 }
fn default_api_ssl_port() -> u16 { 7877 }
fn default_api_idle_timeout_ms() -> u64 { 30000 }
fn default_gzip_enabled() -> bool { true }
```

#### 任务 CFG-T2: 完善 myPeerInfo 响应字段
**文件**: `crates/p2p/src/peer.rs`

**改动内容**:
```rust
impl Peer {
    /// 转换为 Java 兼容的 PeerInfo 响应格式（完整版）
    pub fn to_peer_info_full(&self, config: &P2PConfig) -> serde_json::Value {
        let mut map = self.to_peer_info(); // 复用已有字段
        
        // 补充缺失字段
        if let Ok(state) = config.blockchain_state.read() {
            map.insert("blockchainState".to_string(), 
                      serde_json::Value::Number(serde_json::Number::from(*state)));
        }
        
        map.insert("apiServerIdleTimeout".to_string(), 
                  serde_json::Value::Number(serde_json::Number::from(config.api_idle_timeout_ms)));
        
        if let Some(ref hallmark) = config.my_hallmark {
            map.insert("hallmark".to_string(), 
                      serde_json::Value::String(hallmark.clone()));
        }
        
        // disabledAPIs（可选，暂返回空数组）
        map.insert("disabledAPIs".to_string(), 
                  serde_json::Value::Array(vec![]));
        
        serde_json::Value::Object(map)
    }
}
```

#### 任务 CFG-T3: 实现配置文件加载增强
**文件**: `crates/p2p/src/config.rs`

**新增方法**:
```rust
impl P2PConfig {
    /// 从多个来源加载配置（优先级从高到低）
    /// 1. 环境变量 (NRCS_*)
    /// 2. 配置文件 (nrcs.toml)
    /// 3. 默认值
    pub fn load_with_env_override(path: Option<&str>) -> Result<Self, String> {
        let mut config = match path {
            Some(p) => Self::load(p)?,
            None => Self::default(),
        };
        
        // 环境变量覆盖
        if let Ok(val) = std::env::var("NRCS_PEER_PORT") {
            if let Ok(port) = val.parse::<u16>() {
                // 更新 listen_addr 中的端口
            }
        }
        if let Ok(val) = std::env::var("NRCS_MY_ADDRESS") {
            config.my_address = Some(val);
        }
        if let Ok(val) = std::env::var("NRCS_SHARE_ADDRESS") {
            config.share_my_address = val.parse().unwrap_or(true);
        }
        // ... 更多环境变量
        
        Ok(config)
    }
    
    /// 加载种子节点并初始化到 Peers
    pub async fn init_bootstrap_peers(&self, peers: &Arc<Peers>) -> Result<(), String> {
        // 1. 从 defaultPeers 加载
        for addr_str in &self.default_peers {
            if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                let peer = Peer::new(addr, false);
                peers.register_peer(peer).await;
            }
        }
        
        // 2. 从 wellKnownPeers 加载（标记为知名节点）
        for addr_str in &self.well_known_peers {
            if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                let mut peer = Peer::new(addr, false);
                peer.services |= 0x01; // 标记为知名
                peers.register_peer(peer).await;
            }
        }
        
        // 3. 预加载黑名单
        for addr_str in &self.known_blacklisted_peers {
            if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                peers.blacklist(addr).await;
            }
        }
        
        info!("Loaded {} bootstrap peers, {} well-known peers, {} blacklisted",
              self.default_peers.len(), self.well_known_peers.len(), 
              self.known_blacklisted_peers.len());
        
        Ok(())
    }
}
```

#### 任务 CFG-T4: 创建标准配置文件模板
**文件**: `config/nrcs-peer.toml` (新建)

```toml
# NRCS P2P Configuration Template
# 对齐 Java NRCS nrcs-default.properties v2.1.0

[p2p]
# ===== 基础网络配置 =====
listen_addr = "0.0.0.0:17974"           # P2P 监听地址 (peerServerHost:peerServerPort)
my_address = ""                         # 外部公告地址 (myAddress)
share_my_address = true                 # 是否共享地址 (shareMyAddress)
my_platform = ""                        # 自动检测，通常不需要设置
my_hallmark = ""                        # Hallmark 标识 (myHallmark)

# ===== API 端口配置（用于 myPeerInfo 响应）=====
api_port = 7876                         # API 端口 (API.openAPIPort)
api_ssl_port = 7877                     # API SSL 端口 (API.openAPISSLPort)
api_idle_timeout_ms = 30000             # API 空闲超时 (apiServerIdleTimeout)

# ===== 连接管理配置 =====
max_connections = 20                    # 最大公网连接数 (maxNumberOfConnectedPublicPeers)
max_inbound_connections = 100           # 最大入站连接数 (maxNumberOfInboundConnections)
max_outbound_connections = 20           # 最大出站连接数 (maxNumberOfOutboundConnections)
connect_timeout_ms = 2000               # 连接超时 (connectTimeout)
read_timeout_ms = 4000                 # 读取超时 (readTimeout)
websocket_idle_timeout_secs = 300      # WebSocket 空闲超时秒 (peerServerIdleTimeout/1000)

# ===== 协议配置 =====
use_websockets = true                   # 使用 WebSocket (useWebSockets)
gzip_enabled = true                     # 启用 GZIP 压缩 (isGzipEnabled)
min_compress_size = 256                 # 最小压缩大小 (minCompressSize)

# ===== 黑名单与安全配置 =====
blacklisting_period_secs = 600          # 黑名单时长秒 (blacklistingPeriod/1000)
blacklisting_enabled = true             # 启用黑名单
blacklisting_threshold = 10             # 黑名单阈值
enable_hallmark_protection = true       # 启用 Hallmark 保护 (enableHallmarkProtection)
push_threshold = 0                      # 发送权重阈值 (pushThreshold)
pull_threshold = 0                      # 接收权重阈值 (pullThreshold)
send_to_peers_limit = 10                # 广播限制 (sendToPeersLimit)
ignore_announced_address = false        # 忽略公告地址变更 (ignorePeerAnnouncedAddress)
hide_error_details = false              # 隐藏错误详情 (hideErrorDetails)

# ===== 节点发现配置 =====
max_known_peers = 2000                  # 最大已知节点数 (maxNumberOfKnownPeers)
min_known_peers = 1000                  # 最小已知节点数 (minNumberOfKnownPeers)
use_peers_db = true                     # 使用数据库 (usePeersDb)
save_peers = true                       # 保存节点 (savePeers)
get_more_peers = true                   # 获取更多节点 (getMorePeers)

# ===== 种子节点配置 =====
default_peers = [                       # 默认初始节点 (defaultPeers)
    # "103.249.252.7:17974",
    # "103.235.221.97:17974",
]
well_known_peers = [                    # 知名节点 (wellKnownPeers)
    # 保持常连接的特殊节点
]
known_blacklisted_peers = [             # 预设黑名单 (knownBlacklistedPeers)
    # "malicious.node.example.com:17974",
]

# ===== 高级配置 =====
offline_mode = false                    # 离线模式 (offline)
# cjdns_only = false                    # 仅 cjdns 地址 (cjdnsOnly)
# enable_upnp = false                   # UPnP 支持 (enablePeerUPnP)

# ===== 守护进程间隔配置 =====
connection_daemon_interval_secs = 5      # 连接守护进程间隔
discovery_daemon_interval_secs = 30     # 发现守护进程间隔
unblacklist_daemon_interval_secs = 60   # 解除黑名单间隔
transaction_daemon_interval_secs = 30   # 交易广播间隔
send_transactions_batch_size = 10       # 交易批量大小时
bundler_rate_broadcast_interval_secs = 1800  # BundlerRate 广播间隔 (30分钟)
```

### A.5 配置验证检查清单

完成配置对齐后，需要逐项验证：

- [ ] **CFG-01**: blacklistingPeriod 单位统一为毫秒或秒，并在代码中明确注释
- [ ] **CFG-02**: minKnownPeers = 1000
- [ ] **CFG-03**: connectTimeout 与 Java 一致（或明确说明差异原因）
- [ ] **CFG-04**: P2PConfig 包含 api_port 和 api_ssl_port
- [ ] **CFG-05**: myPeerInfo 响应包含所有 Java 字段
- [ ] **CFG-06**: 支持从配置文件加载种子节点
- [ ] **CFG-07**: 实现 offline_mode 逻辑
- [ ] **CFG-08**: 启动时预加载已知黑名单
- [ ] **CFG-09**: 实现 ignoreAnnouncedAddress 逻辑
- [ ] **CFG-10**: 明确 idleTimeout 单位
- [ ] **CFG-11~15**: 可选功能按需实现

### A.6 配置兼容性测试场景

```rust
#[cfg(test)]
mod config_compatibility_tests {
    use super::*;
    
    #[test]
    fn test_config_matches_java_defaults() {
        let config = P2PConfig::default();
        
        // 验证关键配置值与 Java 一致
        assert_eq!(config.max_connections, 20, "maxNumberOfConnectedPublicPeers");
        assert_eq!(config.max_inbound_connections, 100, "maxNumberOfInboundConnections");
        assert_eq!(config.max_outbound_connections, 20, "maxNumberOfOutboundConnections");
        assert_eq!(config.max_known_peers, 2000, "maxNumberOfKnownPeers");
        assert_eq!(config.min_known_peers, 1000, "minNumberOfKnownPeers");
        assert_eq!(config.blacklisting_period_secs, 600, "blacklistingPeriod (10min)");
        assert_eq!(config.connect_timeout_ms, 2000, "connectTimeout");
        assert_eq!(config.read_timeout_ms, 4000, "readTimeout");
        assert_eq!(config.min_compress_size, 256, "minCompressSize");
        assert_eq!(config.send_to_peers_limit, 10, "sendToPeersLimit");
    }
    
    #[test]
    fn test_my_peer_info_fields_complete() {
        let config = P2PConfig::default();
        let my_peer = Peer::new(config.listen_socket_addr().unwrap(), false);
        let info = my_peer.to_peer_info_full(&config);
        
        // 验证所有必要字段存在
        assert!(info.get("application").is_some());
        assert!(info.get("version").is_some());
        assert!(info.get("platform").is_some());
        assert!(info.get("services").is_some());
        assert!(info.get("announcedAddress").is_some());
        assert!(info.get("shareAddress").is_some());
        assert!(info.get("apiPort").is_some());
        assert!(info.get("apiSSLPort").is_some());
        assert!(info.get("blockchainState").is_some());  // 新增
        assert!(info.get("apiServerIdleTimeout").is_some());  // 新增
    }
    
    #[test]
    fn test_bootstrap_peers_loading() {
        let config = P2PConfig::default();
        let peers = Arc::new(Peers::new(Peer::new("127.0.0.1:0".parse().unwrap(), false)));
        
        tokio_test::block_on(async {
            config.init_bootstrap_peers(&peers).await.unwrap();
            
            // 验证种子节点已加载
            assert!(peers.known_peers_count().await >= 0);
        });
    }
}
```

---

## 📝 附录 B：Java NRCS → Rust 映射速查表

### B.1 类/结构体映射

| Java 类/接口 | Rust 结构体/Trait | 文件位置 |
|-------------|------------------|---------|
| `Peer` (implements IPeer) | `Peer` | `crates/p2p/src/peer.rs` |
| `Peers` (static singleton) | `Peers` + `P2PManager` | `crates/p2p/src/peer.rs`, `manager.rs` |
| `PeerWebSocket` | `WebSocketConnection` | 待新建 `connection_pool.rs` |
| `PeerPostRequest` | `PendingRequest` (oneshot channel) | 待新建 `connection_pool.rs` |
| `PeerServlet` | `Handler` + HTTP/WebSocket server | `handlers/mod.rs`, `http.rs`, `websocket.rs` |
| `PeerRequestHandler` (abstract) | `Handler` trait | `handlers/mod.rs` |
| `GetInfo` | `GetInfoHandler` | `handlers/get_info.rs` |
| `GetPeers` | `GetPeersHandler` | `handlers/get_peers.rs` |
| `AddPeers` | `AddPeersHandler` | `handlers/add_peers.rs` |
| `ProcessBlock` | `ProcessBlockHandler` | `handlers/process_block.rs` |
| `ProcessTransactions` | `ProcessTransactionsHandler` | `handlers/process_transactions.rs` |
| `GetCumulativeDifficulty` | `GetCumulativeDifficultyHandler` | `handlers/get_cumulative_difficulty.rs` |
| `GetMilestoneBlockIds` | `GetMilestoneBlockIdsHandler` | `handlers/get_milestone_block_ids.rs` |
| `GetNextBlockIds` | `GetNextBlockIdsHandler` | `handlers/get_next_block_ids.rs` |
| `GetNextBlocks` | `GetNextBlocksHandler` | `handlers/get_next_blocks.rs` |
| `GetTransactions` | `GetTransactionsHandler` | `handlers/get_transactions.rs` |
| `BundlerRate` | `BundlerRateHandler` | `handlers/bundler_rate.rs` |

### B.2 方法映射

| Java 方法 | Rust 方法 | 状态 |
|----------|----------|------|
| `Peer.connect()` | `Peer::connect()` | ⚠️ 需完善 |
| `Peer.send(JSONObject)` | `Peer::send()` | ❌ 需新建 |
| `Peer.deactivate()` | `Peer::deactivate()` | ✅ 完成 |
| `Peer.blacklist(String)` | `Peer::blacklist()` | ✅ 完成 |
| `Peer.unBlacklist()` | `Peer::un_blacklist()` | ✅ 完成 |
| `Peer.analyzeHallmark(String)` | `Peer::analyze_hallmark()` | ❌ 需新建 |
| `Peer.providesService(int)` | `Peer::provides_service()` | ✅ 完成 |
| `Peer.getWeight()` | `Peer::get_weight()` | ✅ 完成 |
| `Peer.updateDownloadedVolume(long)` | ❌ 缺失 | 需新建 |
| `Peer.updateUploadedVolume(long)` | ❌ 缺失 | 需新建 |
| `Peers.addPeer(Peer)` | `P2PManager::add_peer()` | ✅ 完成 |
| `Peers.removePeer(Peer)` | `P2PManager::remove_peer()` | ✅ 完成 |
| `Peers.findOrCreatePeer(String)` | `Peers::find_or_create_peer()` | ✅ 完成 |
| `Peers.getAnyPeer(PeerState, boolean)` | `Peers::get_any_peer()` | ✅ 完成 |
| `Peers.sendToSomePeers(JSONObject)` | `P2PManager::send_to_some_peers()` | ⚠️ 需完善 |
| `Peers.broadcastBlock(IBlock)` | `P2PManager::broadcast_block()` | ❌ 需新建 |
| `Peers.broadcastTransaction(ITransaction)` | `P2PManager::broadcast_transactions()` | ❌ 需新建 |
| `Peers.getMyPeerInfoRequest()` | 内联在 connect() 中 | ⚠️ 需完善 |
| `Peers.getMyPeerInfoResponse()` | `Peer::to_peer_info_full()` | ⚠️ 需完善 |
| `Peers.updateSavedPeers()` | ❌ 缺失 | 需新建 |
| `Peers.loadSavedPeers()` | ❌ 缺失 | 需新建 |

### B.3 常量映射

| Java 常量 | Rust 常量 | 文件位置 |
|----------|----------|---------|
| `Constant.APPLICATION` | `APPLICATION` | `constants.rs` |
| `Constant.VERSION` | `VERSION` | `constants.rs` |
| `PeerState.NON_CONNECTED` | `PeerState::NonConnected` | `peer.rs` |
| `PeerState.CONNECTED` | `PeerState::Connected` | `peer.rs` |
| `PeerState.DISCONNECTED` | `PeerState::Disconnected` | `peer.rs` |
| `PeerService.API` | `0x01` | 需定义 |
| `PeerService.API_SSL` | `0x02` | 需定义 |
| `PeerService.CORS` | `0x04` | 需定义 |
| `PeerService.HALLMARK` | `0x08` | 需定义 |
| `FLAG_COMPRESSED` | `1` | `protocol.rs` |

---

**文档结束**
