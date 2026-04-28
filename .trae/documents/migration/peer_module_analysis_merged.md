# NRCS Peer 模块技术方案（完整版）

## 1. 模块概述

Peer 模块是 NRCS 区块链网络通信的核心组件，负责节点之间的 P2P 通信、数据同步和状态管理。该模块实现了完整的客户端/服务端双角色功能，支持 HTTP 和 WebSocket 双协议通信。

### 1.1 核心架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        Peers (管理中心)                          │
│  ┌───────────────┐  ┌───────────────┐  ┌─────────────────────┐ │
│  │ Peer 实体管理   │  │ 连接池管理     │  │ 广播与同步服务       │ │
│  │ ConcurrentHashMap│ │ ThreadPool    │  │ ExecutorService     │ │
│  └───────────────┘  └───────────────┘  └─────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                        PeerServlet (服务端)                      │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ WebSocketServlet + PeerRequestHandler 路由分发            │   │
│  └──────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                     PeerWebSocket (双协议)                       │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Client: WebSocketClient (Jetty)                          │   │
│  │ Server: @WebSocket Session                               │   │
│  └──────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                         Peer (实体类)                           │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ IPeer 接口实现: 状态管理、连接、发送请求                    │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. 核心类说明

| 类名 | 文件路径 | 职责 |
|------|----------|------|
| **Peers** | `service/peer/Peers.java` | Peer 管理中心，静态单例，负责全局配置、Peer 池管理、连接调度 |
| **Peer** | `service/peer/Peer.java` | 单个 Peer 实体，封装连接状态、属性、发送逻辑 |
| **PeerServlet** | `service/peer/PeerServlet.java` | 服务端 Servlet，处理入站 HTTP/WebSocket 请求 |
| **PeerWebSocket** | `service/peer/PeerWebSocket.java` | WebSocket 通讯层，支持客户端和服务端双向模式 |
| **PeerRequestHandler** | `service/peer/PeerRequestHandler.java` | 抽象基类，定义 Peer API 请求处理器接口 |
| **PeerPostRequest** | `service/peer/PeerPostRequest.java` | WebSocket 请求-响应匹配器，基于 CountDownLatch |

---

## 3. Peer 作为客户端流程

### 3.1 初始化阶段

```java
// Peers.java - 静态初始化块
static {
    // 1. 读取配置参数
    myPlatform = System.getProperty("os.name") + " " + System.getProperty("os.arch");
    myAddress = Constant.getProperty("nrcs.myAddress", "");
    
    // 2. 构建 myPeerInfo（自身信息）
    myPeerInfo.put("announcedAddress", announcedAddress);
    myPeerInfo.put("application", Constant.APPLICATION);
    myPeerInfo.put("version", Constant.VERSION);
    myPeerInfo.put("platform", myPlatform);
    myPeerInfo.put("shareAddress", shareMyAddress);
    myPeerInfo.put("apiPort", API.openAPIPort);
    // ...
}
```

### 3.2 发起连接流程

```
┌──────────┐      ┌──────────┐      ┌──────────────────┐
│  Caller   │─────▶│ Peer.connect() │─────▶│ send(getMyPeerInfo) │
└──────────┘      └──────────┘      └──────────────────┘
                                              │
                        ┌─────────────────────┘
                        ▼
              ┌─────────────────────┐
              │ 1. 尝试 WebSocket    │
              │ 2. 回退到 HTTP POST  │
              └─────────────────────┘
                        │
                        ▼
              ┌─────────────────────┐
              │ 解析响应：           │
              │ - services          │
              │ - version           │
              │ - announcedAddress  │
              │ - hallmark          │
              │ - blockchainState   │
              └─────────────────────┘
```

**关键代码** ([Peer.java#L600-L700](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/Peer.java#L600-L700)):

```java
public void connect() {
    lastConnectAttempt = DateKit.getEpochTime();
    
    // 1. 验证 announcedAddress 是否变更
    if (!Peers.ignorePeerAnnouncedAddress && announcedAddress != null) {
        URI uri = new URI("http://" + announcedAddress);
        InetAddress inetAddress = InetAddress.getByName(uri.getHost());
        if (!inetAddress.equals(InetAddress.getByName(host))) {
            // 地址变更，替换 Peer
            Peers.removePeer(this);
            IPeer newPeer = Peers.findOrCreatePeer(inetAddress, announcedAddress, true);
            ((Peer) newPeer).connect();
            return;
        }
    }
    
    // 2. 发送 getInfo 请求获取对方信息
    JSONObject response = send(Peers.getMyPeerInfoRequest());
    if (response != null) {
        if (response.get("error") != null) {
            setState(PeerState.NON_CONNECTED);
            return;
        }
        
        // 3. 更新 Peer 属性
        services = Long.parseUnsignedLong((String) response.get("services"));
        setApplication((String) response.get("application"));
        setVersion((String) response.get("version"));
        setPlatform((String) response.get("platform"));
        analyzeHallmark((String) response.get("hallmark"));
        
        // 4. 处理 announcedAddress 变更
        String newAnnouncedAddress = (String) response.get("announcedAddress");
        if (newAnnouncedAddress != null && !newAnnouncedAddress.equals(announcedAddress)) {
            Peers.setAnnouncedAddress(this, newAnnouncedAddress);
        }
        
        // 5. 设置为已连接状态
        setState(PeerState.CONNECTED);
    }
}
```

### 3.3 发送请求流程

```
┌──────────┐     ┌──────────────────┐     ┌─────────────────────┐
│  Caller  │────▶│ Peer.send(request)│────▶│ useWebSocket?       │
└──────────┘     └──────────────────┘     └─────────────────────┘
                                                  │
                              ┌───────────────────┼───────────────────┐
                              ▼                   ▼                   │
                    ┌────────────────┐   ┌────────────────┐         │
                    │ WebSocket.doPost│   │ HTTP POST      │         │
                    │ (二进制帧)       │   │ (JSON文本)      │         │
                    └────────────────┘   └────────────────┘         │
                              │                   │                   │
                              └─────────┬─────────┘                   │
                                        ▼                             │
                              ┌─────────────────────┐                │
                              │ 返回 JSONObject     │◀───────────────┘
                              └─────────────────────┘
                                        │
                                        ▼
                              ┌─────────────────────┐
                              │ 错误处理 & 黑名单机制  │
                              └─────────────────────┘
```

**关键代码** ([Peer.java#L500-L620](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/Peer.java#L500-L620)):

```java
public JSONObject send(JSONObject request, int maxResponseSize) {
    HttpURLConnection connection = null;
    JSONObject response = null;
    
    try {
        // 1. 优先使用 WebSocket
        if (useWebSocket) {
            PeerWebSocket ws = getOrCreateWebSocket();
            if (ws != null && ws.isOpen()) {
                String wsResponse = ws.doPost(JSON.toJSONString(request));
                response = JSON.parseObject(wsResponse);
                updateDownloadedVolume(wsResponse.length());
            }
        } 
        
        // 2. 回退到 HTTP POST
        if (response == null) {
            URL url = new URL("http://" + host + ":" + getPort() + "/nrcs");
            connection = (HttpURLConnection) url.openConnection();
            connection.setRequestMethod("POST");
            connection.setRequestProperty("Content-Type", "text/plain; charset=UTF-8");
            
            // 发送请求体
            try (Writer writer = new OutputStreamWriter(connection.getOutputStream(), "UTF-8")) {
                request.writeJSONString(writer);
            }
            
            // 接收响应
            if (connection.getResponseCode() == 200) {
                InputStream is = connection.getInputStream();
                if ("gzip".equals(connection.getHeaderField("Content-Encoding"))) {
                    is = new GZIPInputStream(is);
                }
                response = JSON.parseObject(new String(readBytes(is), "UTF-8"));
            }
        }
        
        // 3. 错误处理
        if (response != null && response.get("error") != null) {
            deactivate();  // 断开连接
            if ("sequence error".equals(response.get("error"))) {
                connect();  // 重连
            }
        }
        
    } catch (IOException e) {
        blacklist(e);  // 加入黑名单
    } finally {
        if (connection != null) connection.disconnect();
    }
    
    return response;
}
```

---

## 4. Peer 作为服务端流程

### 4.1 服务端启动

**启动位置**: [Peers.java#L1500-L1580](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/Peers.java#L1500-L1580)

```java
private static class Init {
    private final static Server peerServer;

    static {
        if (Peers.shareMyAddress) {  // 配置允许共享地址才启动
            peerServer = new Server();
            ServerConnector connector = new ServerConnector(peerServer);
            connector.setPort(port);  // 默认端口 17974 (测试网 16974)
            connector.setIdleTimeout(idleTimeout);
            connector.setReuseAddress(true);
            peerServer.addConnector(connector);

            ServletContextHandler ctxHandler = new ServletContextHandler();
            ctxHandler.addServlet(new ServletHolder(new PeerServlet()), "/*");

            // 可选：DoS 防护过滤器
            // 可选：Gzip 压缩处理

            peerServer.setHandler(ctxHandler);
            peerServer.start();
        }
    }
}
```

### 4.2 请求处理流程

```
┌─────────────────────────────────────────────────────────────────┐
│                        入站请求                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌────────────────┐     ┌──────────────────────────────────┐   │
│   │ HTTP POST      │     │ WebSocket Binary Frame           │   │
│   │ (JSON Text)    │     │ (Binary Protocol)               │   │
│   └────────┬───────┘     └──────────────┬───────────────────┘   │
│            │                            │                       │
│            ▼                            ▼                       │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                  PeerServlet                             │   │
│   │                                                          │   │
│   │  doPost(HttpServletRequest, HttpServletResponse)         │   │
│   │  doPost(WebSocket, requestId, message)                   │   │
│   │                                                          │   │
│   │  1. 提取远程地址 → findOrCreatePeer()                    │   │
│   │  2. process(peer, reader)                                │   │
│   │  3. 返回 JSONStreamAware 响应                            │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                 │                               │
│                                 ▼                               │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │             process() 方法                               │   │
│   │                                                          │   │
│   │  1. 解析 requestType                                    │   │
│   │  2. 查找 PeerRequestHandler                             │   │
│   │  3. 检查 rejectWhileDownloading()                       │   │
│   │  4. handler.processRequest(request, peer)               │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**关键代码** ([PeerServlet.java#L100-L180](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/PeerServlet.java#L100-L180)):

```java
// HTTP POST 处理
@Override
protected void doPost(HttpServletRequest req, HttpServletResponse resp) 
    throws ServletException, IOException {
    
    // 1. 根据远程地址查找或创建 Peer
    IPeer peer = Peers.findOrCreatePeer(req.getRemoteAddr());
    if (peer == null) {
        jsonResponse = UNKNOWN_PEER;  // {"error": "Unknown peer"}
    } else {
        // 2. 处理请求
        jsonResponse = process(peer, req.getReader());
    }
    
    // 3. 写入响应
    resp.setContentType("text/plain; charset=UTF-8");
    JSON.writeJSONString(resp.getWriter(), jsonResponse);
}

// WebSocket POST 处理
void doPost(PeerWebSocket webSocket, long requestId, String request) {
    InetSocketAddress socketAddress = webSocket.getRemoteAddress();
    IPeer peer = Peers.findOrCreatePeer(socketAddress.getHostString());
    
    if (peer != null) {
        peer.setInboundWebSocket(webSocket);  // 记录入站连接
        jsonResponse = process(peer, new StringReader(request));
    }
    
    // 通过 WebSocket 返回响应
    webSocket.sendResponse(requestId, response);
}

// 核心处理方法
private JSONStreamAware process(IPeer peer, Reader reader) {
    JSONObject request = parseRequest(reader);
    
    // 校验 protocol 版本
    int protocol = getIntParam(request, "protocol", 1);
    if (protocol > 2) return UNSUPPORTED_PROTOCOL;
    
    // 路由到对应的 Handler
    String requestType = (String) request.get("requestType");
    PeerRequestHandler handler = peerRequestHandlers.get(requestType);
    
    if (handler == null) {
        return UNSUPPORTED_REQUEST_TYPE;  // {"error": "Unsupported request type"}
    }
    
    // 下载中检查（部分接口拒绝）
    if (handler.rejectWhileDownloading() && blockchainProcessor.isDownloading()) {
        return DOWNLOADING;  // {"error": "Downloading"}
    }
    
    // 执行处理
    return handler.processRequest(request, peer);
}
```

---

## 5. WebSocket 通讯机制详解

### 5.1 协议设计

#### 二进制消息格式

```
┌──────────┬──────────┬──────────┬──────────┬──────────────────┐
│ Version  │ RequestID│ Flags    │ Length   │ Payload          │
│ (4 bytes)│ (8 bytes)│ (4 bytes)│ (4 bytes)│ (Length bytes)   │
└──────────┴──────────┴──────────┴──────────┴──────────────────┘

字段说明:
- Version: 协议版本号 (当前=1)
- RequestId: 请求标识符 (用于请求-响应匹配)
- Flags: 标志位 (bit0=GZIP压缩)
- Length: Payload 字节长度
- Payload: UTF-8 编码的 JSON 字符串 (可能被 GZIP 压缩)
```

#### 关键代码 ([PeerWebSocket.java#L200-L280](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/PeerWebSocket.java#L200-L280)):

```java
// 发送请求 (Client)
public String doPost(String request) throws IOException {
    lock.lock();
    try {
        // 1. 构建二进制消息
        long requestId = nextRequestId++;
        byte[] requestBytes = request.getBytes("UTF-8");
        
        // 2. 压缩处理
        int flags = 0;
        if (Peers.isGzipEnabled && requestBytes.length >= Peers.MIN_COMPRESS_SIZE) {
            flags |= FLAG_COMPRESSED;
            requestBytes = gzipCompress(requestBytes);
        }
        
        // 3. 组装二进制帧
        ByteBuffer buf = ByteBuffer.allocate(requestBytes.length + 20);
        buf.putInt(VERSION)
           .putLong(requestId)
           .putInt(flags)
           .putInt(requestBytes.length)
           .put(requestBytes)
           .flip();
        
        // 4. 发送
        session.getRemote().sendBytes(buf);
        
        // 5. 等待响应 (同步阻塞)
        PeerPostRequest postRequest = new PeerPostRequest();
        requestMap.put(requestId, postRequest);
        return postRequest.get(Peers.readTimeout, TimeUnit.MILLISECONDS);
        
    } finally {
        lock.unlock();
    }
}

// 接收消息 (Server & Client)
@OnWebSocketMessage
public void onMessage(byte[] inbuf, int off, int len) {
    lock.lock();
    try {
        ByteBuffer buf = ByteBuffer.wrap(inbuf, off, len);
        
        // 1. 解析头部
        int version = Math.min(buf.getInt(), VERSION);
        long requestId = buf.getLong();
        int flags = buf.getInt();
        int length = buf.getInt();
        
        // 2. 提取 Payload
        byte[] msgBytes = new byte[buf.remaining()];
        buf.get(msgBytes);
        
        // 3. 解压
        if ((flags & FLAG_COMPRESSED) != 0) {
            msgBytes = gzipDecompress(msgBytes, length);
        }
        
        String message = new String(msgBytes, "UTF-8");
        
        // 4. 分发处理
        if (peerServlet != null) {
            // Server 模式: 提交线程池处理
            threadPool.execute(() -> peerServlet.doPost(this, requestId, message));
        } else {
            // Client 模式: 匹配等待的请求
            PeerPostRequest postRequest = requestMap.remove(requestId);
            if (postRequest != null) {
                postRequest.complete(message);
            }
        }
        
    } finally {
        lock.unlock();
    }
}
```

### 5.2 请求-响应匹配机制

```
客户端发送请求:
┌─────────────┐     ┌─────────────┐     ┌──────────────────────┐
│ 生成 requestId │────▶│ 放入 requestMap │────▶│ CountDownLatch.await() │
└─────────────┘     └─────────────┘     └──────────────────────┘
                                                  │
                                                  │ 阻塞等待
                                                  ▼
服务端处理完成:                           ┌──────────────────────┐
┌─────────────┐                           │ postRequest.complete()│
│ sendResponse │──────────────────────────▶│ latch.countDown()    │
│ (相同requestId)│                         └──────────────────────┘
└─────────────┘                                      │
                                                     ▼
                                            ┌──────────────────────┐
                                            │ 返回 response 字符串  │
                                            └──────────────────────┘
```

**关键类** ([PeerPostRequest.java](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/PeerPostRequest.java)):

```java
public class PeerPostRequest {
    private final CountDownLatch latch = new CountDownLatch(1);
    private volatile String response;
    private volatile IOException exception;
    
    public String get(long timeout, TimeUnit unit) throws InterruptedException, IOException {
        if (!latch.await(timeout, unit)) {
            throw new SocketTimeoutException("WebSocket read timeout exceeded");
        }
        if (exception != null) {
            throw exception;
        }
        return response;
    }
    
    public void complete(String response) {
        this.response = response;
        latch.countDown();
    }
    
    public void complete(IOException exception) {
        this.exception = exception;
        latch.countDown();
    }
}
```

### 5.3 连接生命周期

```
┌─────────────────────────────────────────────────────────────┐
│                    WebSocket 生命周期                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Client Side (Outbound):                                     │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 1. startClient(uri)                                   │   │
│  │    ├── 检查 session 是否已存在 (复用连接)               │   │
│  │    ├── 创建 ClientUpgradeRequest                      │   │
│  │    └── peerClient.connect(this, uri, req)             │   │
│  │                                                       │   │
│  │ 2. onConnect(session) ← 连接成功回调                   │   │
│  │                                                       │   │
│  │ 3. 多次 doPost() / sendResponse() 循环                 │   │
│  │                                                       │   │
│  │ 4. onClose(statusCode, reason) ← 连接关闭              │   │
│  │    ├── 清空 requestMap 中所有待响应的请求               │   │
│  │    └── 通知异常给所有等待线程                           │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  Server Side (Inbound):                                      │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 1. PeerSocketCreator.createWebSocket(req, resp)       │   │
│  │    └── new PeerServlet(PeerServlet.this)              │   │
│  │                                                       │   │
│  │ 2. onConnect(session) ← 客户端连接成功                 │   │
│  │                                                       │   │
│  │ 3. onMessage(buf) ← 收到请求                          │   │
│  │    └── threadPool.submit(() -> servlet.doPost(...))    │   │
│  │                                                       │   │
│  │ 4. sendResponse(requestId, response) ← 返回响应        │   │
│  │                                                       │   │
│  │ 5. onClose ← 客户端断开                               │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 5.4 连接复用与并发控制

```java
// PeerWebSocket.java - 客户端连接管理
public boolean startClient(URI uri) throws IOException {
    lock.lock();  // ReentrantLock 保证串行化
    try {
        // 1. 已有活跃连接直接复用
        if (session != null) {
            return true;  // useWebSocket = true
        }
        
        // 2. 限制重连频率 (10秒内不重复尝试)
        if (System.currentTimeMillis() <= connectTime + 10 * 1000) {
            return false;  // useWebSocket = false, 回退HTTP
        }
        
        connectTime = System.currentTimeMillis();
        ClientUpgradeRequest req = new ClientUpgradeRequest();
        Future<Session> conn = peerClient.connect(this, uri, req);
        conn.get(connectTimeout + 100, TimeUnit.MILLISECONDS);  // 阻塞等待
        
        return true;  // useWebSocket = true
        
    } catch (ExecutionException exc) {
        if (exc.getCause() instanceof UpgradeException) {
            // 对方不支持 WebSocket, 回退到 HTTP
            return false;
        }
        throw (IOException) exc.getCause();
    } finally {
        if (!useWebSocket) close();
        lock.unlock();
    }
}
```

---

## 6. Peer API 接口清单

### 6.1 请求路由表

| requestType | Handler 类 | 说明 | rejectWhileDownloading |
|-------------|------------|------|------------------------|
| **getInfo** | [GetInfo](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/GetInfo.java) | 获取节点信息 (版本、服务等) | false |
| **getPeers** | [GetPeers](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/GetPeers.java) | 获取已知 Peer 列表 | false |
| **addPeers** | [AddPeers](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/AddPeers.java) | 批量添加新 Peer | false |
| **getCumulativeDifficulty** | [GetCumulativeDifficulty](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/GetCumulativeDifficulty.java) | 获取累积难度 | true |
| **getMilestoneBlockIds** | [GetMilestoneBlockIds](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/GetMilestoneBlockIds.java) | 获取里程碑区块 ID | true |
| **getNextBlockIds** | [GetNextBlockIds](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/GetNextBlockIds.java) | 获取后续区块 ID 列表 | true |
| **getNextBlocks** | [GetNextBlocks](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/GetNextBlocks.java) | 获取后续区块数据 | true |
| **getTransactions** | [GetTransactions](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/GetTransactions.java) | 获取指定交易 | true |
| **getUnconfirmedTransactions** | [GetUnconfirmedTransactions](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/GetUnconfirmedTransactions.java) | 获取未确认交易 | true |
| **processBlock** | [ProcessBlock](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/ProcessBlock.java) | 处理新区块 | true |
| **processTransactions** | [ProcessTransactions](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/ProcessTransactions.java) | 处理新交易 | true |
| **bundlerRate** | [ProcessBundlerRate](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/ProcessBundlerRate.java) | 接收打包费率广播 | false |

### 6.2 核心接口详细说明

#### 6.2.1 getInfo - 节点信息交换

**用途**: 建立连接时首次调用，交换双方元数据

**请求格式**:
```json
{
    "requestType": "getInfo",
    "protocol": 1,
    "services": "12345",
    "hallmark": "base64_encoded_hallmark",
    "announcedAddress": "192.168.1.100:17974",
    "application": "NRCS",
    "version": "2.1.0",
    "platform": "Windows 10 amd64",
    "shareAddress": true,
    "apiPort": 7876,
    "apiSSLPort": 7877,
    "blockchainState": 0
}
```

**响应格式**:
```json
{
    "application": "NRCS",
    "version": "2.1.0",
    "platform": "Linux x86_64",
    "services": "12345",
    "announcedAddress": "10.0.0.1:17974",
    "shareAddress": true,
    "apiPort": 7876,
    "apiSSLPort": 7877,
    "blockchainState": 0,
    "hallmark": "...",
    "disabledAPIs": "...",
    "apiServerIdleTimeout": 30000
}
```

**处理逻辑** ([GetInfo.java](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/GetInfo.java)):
1. 更新对方 lastUpdated 时间戳
2. 解析并设置 services、hallmark
3. 验证 announcedAddress 合法性
4. 更新 application/version/platform
5. 返回本节点信息

#### 6.2.2 processBlock - 区块同步

**用途**: 向对端推送新区块

**请求格式**:
```json
{
    "requestType": "processBlock",
    "previousBlock": "previous_block_id",
    "block": "{...}",
    "timestamp": 1700000000
}
```

**响应格式**:
```json
{}
```

**处理逻辑** ([ProcessBlock.java](file:///d:/workspace/Git/nrcs/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/peer/ProcessBlock.java)):
1. 验证 previousBlock 是否匹配当前链顶
2. 异步提交到区块链处理器 (`peersService.submit`)
3. 若验证失败则黑名单对方

#### 6.2.3 addPeers / getPeers - Peer 发现

**addPeers 请求**:
```json
{
    "requestType": "addPeers",
    "peers": ["192.168.1.10:17974", "10.0.0.5:17974"],
    "services": ["12345", "12346"]
}
```

**getPeers 响应**:
```json
{
    "peers": ["192.168.1.20:17974", "10.0.0.15:17974"],
    "services": ["12347", "12348"]
}
```

### 6.3 广播机制

- **区块广播**：`Peers.sendToSomePeers(IBlock block)` → 组装 `processBlock` 请求
- **交易广播**：`Peers.sendToSomePeers(List<ITransaction>)` → 批量组装 `processTransactions`
- **Bundler 费率广播**：定时线程每 30 分钟或状态变更时广播

广播策略：
- 只发送给 `CONNECTED` 状态、非黑名单、有 announcedAddress 的节点
- 启用 hallmark 保护时，只推送给 weight >= pushThreshold 的节点
- 最多发送给 `sendToPeersLimit` 个节点（默认配置）
- 使用 `sendingService` 线程池异步发送

---

## 7. 数据返回方式

### 7.1 统一响应格式

所有 Peer-to-Peer API 使用 **JSON 文本** 格式:

**成功响应**:
```json
{
    // 具体业务字段...
    "peers": [...],
    "cumulativeDifficulty": "123456789"
}
```

**错误响应**:
```json
{
    "error": "错误描述字符串"
}
```

### 7.2 标准错误码

| 错误常量 | 值 | 场景 |
|---------|-----|------|
| UNSUPPORTED_REQUEST_TYPE | `"Unsupported request type"` | 未知的 requestType |
| UNSUPPORTED_PROTOCOL | `"Unsupported protocol"` | protocol 版本过高 |
| UNKNOWN_PEER | `"Unknown peer"` | 无法识别的远程地址 |
| SEQUENCE_ERROR | `"Sequence error"` | 区块高度不连续 |
| MAX_INBOUND_CONNECTIONS | `"Max inbound connections"` | 达到最大入站连接数 |
| DOWNLOADING | `"Downloading"` | 正在下载区块链 |
| LIGHT_CLIENT | `"Light client"` | 轻量客户端不支持 |
| INVALID_ANNOUNCED_ADDRESS | `"Invalid announced address"` | 公告地址无效 |

### 7.3 传输层差异

| 特性 | HTTP POST | WebSocket Binary |
|------|-----------|------------------|
| Content-Type | `text/plain; charset=UTF-8` | N/A (Binary) |
| 编码 | UTF-8 JSON 文本 | 二进制帧 (UTF-8 JSON payload) |
| 压缩 | HTTP Gzip Header | 自定义 Flags bit0 |
| 请求匹配 | N/A (无状态) | RequestId (8字节) |
| 连接复用 | 无 (每次新建 TCP) | 长连接复用 |
| 典型延迟 | 较高 (TCP握手+TLS) | 较低 (已有连接) |

---

## 8. Peer 状态机

```
                    ┌─────────────────┐
                    │  NON_CONNECTED  │ ◀── 初始状态
                    └────────┬────────┘
                             │ connect()
                             ▼
                    ┌─────────────────┐
              ┌────▶│    CONNECTED    │◀──┐
              │     └────────┬────────┘   │
              │              │            │
              │    disconnect/deactivate  │
              │              │            │
              │              ▼            │
              │     ┌─────────────────┐   │
              │     │   DISCONNECTED  │───┘
              │     └────────┬────────┘
              │              │
              │    blacklist()
              │              │
              │              ▼
              │     ┌─────────────────┐
              └────▶│   BLACKLISTED    │
                    │ (blacklistingTime│
                    │  > 0)            │
                    └────────┬────────┘
                             │ unBlacklist()
                             │ (超时自动解除)
                             ▼
                    ┌─────────────────┐
                    │  NON_CONNECTED  │
                    └─────────────────┘
```

**状态转换触发条件**:

| 当前状态 | 目标状态 | 触发条件 |
|---------|---------|----------|
| NON_CONNECTED | CONNECTED | `connect()` 成功且 getInfo 返回有效 |
| CONNECTED | DISCONNECTED | `deactivate()` 或 IO 异常 |
| CONNECTED/DISCONNECTED | BLACKLISTED | `blacklist()` 被调用 |
| BLACKLISTED | NON_CONNECTED | 超过 blacklistingPeriod 自动解除 |

---

## 9. 定时任务与服务发现

### 9.1 Peer 发现任务

```java
// Peers.java - getMorePeersThread
Runnable getMorePeersThread = () -> {
    // 1. 检查是否需要更多 Peer
    if (hasTooManyKnownPeers()) return;
    
    // 2. 随机选择一个已连接 Peer
    IPeer peer = getAnyPeer(PeerState.CONNECTED, true);
    
    // 3. 发送 getPeers 请求
    JSONObject response = peer.send(getPeersRequest);
    JSONArray peers = (JSONArray) response.get("peers");
    
    // 4. 批量添加新发现的 Peer
    for (int i = 0; i < peers.size(); i++) {
        String address = (String) peers.get(i);
        IPeer newPeer = findOrCreatePeer(address, true);
        if (newPeer != null) addPeer(newPeer);
    }
};
```

### 9.2 黑名单清理任务

```java
// 每 60 秒执行一次
Runnable peerUnBlacklistingThread = () -> {
    int curTime = DateKit.getEpochTime();
    for (IPeer peer : peers.values()) {
        peer.updateBlacklistedStatus(curTime);
        // 如果 blacklistingTime + blacklistingPeriod <= curTime
        // 则自动解除黑名单
    }
};
```

### 9.3 Bundler Rate 广播

```java
// 每 30 分钟或有变化时广播
if (now - ratesTime >= BUNDLER_RATE_BROADCAST_INTERVAL || bundlersChanged) {
    updateMyBundlerRates();
    JSONObject request = new JSONObject();
    request.put("requestType", "BundlerRate");
    request.put("rates", rates);
    request.put("protocol", 2);
    sendToSomePeers(request);  // 广播给部分 Peer
}
```

---

## 10. 关键线程与调度

| 线程名 | 周期 | 职责 |
|--------|------|------|
| `PeerConnecting` | 20s | 主动连接未连接节点、维护连接状态、广播 bundler rates |
| `PeerUnBlacklisting` | 60s | 定期解除超时黑名单节点 |
| `GetMorePeers` | 20s | 从已连接节点拉取更多节点地址 |
| `sendingService` | 线程池(10) | 异步广播消息 |
| `peersService` | 线程池(2-15) | 执行节点连接、请求发送等任务 |
| `threadPool` (WebSocket) | CPU核数~4倍 | 处理入站 WebSocket 消息 |

---

## 11. 配置参数汇总

| 参数名 | 默认值 | 说明 |
|--------|--------|------|
| nrcs.peerServerPort | 17974 | Peer 服务监听端口 |
| nrcs.peerTestServerPort | 16974 | 测试网端口 |
| nrcs.peerServerHost | 0.0.0.0 | 监听地址 |
| nrcs.peerServerIdleTimeout | 300000 | 连接空闲超时(ms) |
| nrcs.shareMyAddress | false | 是否共享自身地址 |
| nrcs.enablePeerUPnP | false | 启用 UPnP 端口映射 |
| nrcs.myAddress | "" | 公告的外部地址 |
| nrcs.myHallmark | "" | Hallmark 身份标识 |
| nrcs.hideErrorDetails | false | 隐藏错误详情 |
| nrcs.maxNumberOfConnectedPublicPeers | 50 | 最大公网连接数 |
| nrcs.maxNumberOfInboundConnections | 30 | 最大入站连接数 |
| nrcs.maxNumberOfOutboundConnections | 20 | 最大出站连接数 |

---

## 12. 总结与设计亮点

### 12.1 架构特点

1. **双协议支持**: WebSocket 优先 + HTTP 回退，兼顾性能与兼容性
2. **长连接复用**: WebSocket 支持多次请求复用同一 TCP 连接
3. **自适应压缩**: 大于 256 字节的载荷自动启用 GZIP 压缩
4. **异步处理**: 区块/交易提交使用独立线程池，避免阻塞 I/O 线程
5. **优雅降级**: WebSocket 连接失败时透明回退到 HTTP
6. **请求-响应匹配**: 通过 `requestId` + `ConcurrentHashMap` + `CountDownLatch` 实现异步转同步

### 12.2 安全机制

1. **Hallmark 验证**: 可选的身份认证机制
2. **黑名单系统**: 自动隔离异常节点
3. **DoS 过滤器**: 可选的 Jetty DoSFilter 集成
4. **连接数限制**: 分别控制入站/出站/公网连接上限
5. **地址验证**: 公告地址必须解析到实际 IP

### 12.3 性能优化

1. **并发安全**: ConcurrentHashMap 管理 Peer 池，ReentrantLock 保护 WebSocket 操作
2. **线程池分层**: peersService (2-15线程) 用于异步任务，threadPool 用于请求处理
3. **流量统计**: downloadedVolume/uploadedVolume 用于负载均衡决策
4. **权重计算**: 基于 Hallmark 和余额动态调整 Peer 优先级

---

**文档版本**: v1.1 (合并版)
**最后更新**: 2026-04-28
**适用代码版本**: NRCS 2.1.0-SNAPSHOT (JDK21)
