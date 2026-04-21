# NRCS P2P Module

完整的 P2P 网络通信模块，与 NRCS Java 版本完全兼容。

## 模块结构

```
crates/p2p/
├── src/
│   ├── lib.rs              # 模块入口
│   ├── config.rs           # P2P 配置（对应 Peers.java 配置常量）
│   ├── error.rs            # 错误码定义（对应 Errors.java）
│   ├── peer.rs             # Peer 结构体（对应 Peer.java）
│   ├── manager.rs          # P2P 管理器（对应 Peers.java 单例）
│   ├── blacklist.rs        # 黑名单管理器
│   ├── protocol/           # 协议定义
│   ├── websocket.rs        # WebSocket 实现（对应 PeerWebSocket.java）
│   ├── http.rs             # HTTP Servlet 实现（对应 PeerServlet.java）
│   ├── handlers/           # 消息处理器（对应 PeerRequestHandler.java）
│   └── daemon/             # 守护进程
│       ├── connection.rs   # 连接守护进程（对应 peerConnectingThread）
│       ├── discovery.rs    # 发现守护进程（对应 getMorePeersThread）
│       ├── unblacklist.rs  # 黑名单守护进程（对应 peerUnBlacklistingThread）
│       └── transaction.rs  # 交易守护进程（对应 sendTransactionsThread）
```

## 与 NRCS Java 的对应关系

| NRCS Java | Rust 实现 | 说明 |
|-----------|-----------|------|
| Peers.java | P2PManager | 节点管理器单例 |
| Peer.java | Peer | 节点信息 |
| PeerState.java | PeerState | 节点状态枚举 |
| PeerServlet.java | http.rs | HTTP Servlet 处理 |
| PeerWebSocket.java | websocket.rs | WebSocket 处理 |
| PeerRequestHandler.java | Handler trait | 请求处理器接口 |
| Errors.java | ErrorCode | 错误码定义 |
| peerConnectingThread | ConnectionDaemon | 连接守护进程 |
| getMorePeersThread | DiscoveryDaemon | 发现守护进程 |
| peerUnBlacklistingThread | UnblacklistDaemon | 黑名单守护进程 |
| sendTransactionsThread | TransactionDaemon | 交易守护进程 |

## 快速开始

### 1. 添加依赖

```toml
[dependencies]
p2p = { path = "crates/p2p" }
```

### 2. 创建 P2P 服务

```rust
use p2p::{P2PConfig, P2PManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建配置
    let config = P2PConfig::default();

    // 创建管理器
    let mut manager = P2PManager::new(config);
    
    // 初始化并启动
    manager.init().await?;
    manager.start().await?;

    // 运行...
    tokio::signal::ctrl_c().await?;
    
    // 停止
    manager.stop().await;
    
    Ok(())
}
```

## 配置项

所有配置项与 NRCS Java 版本一一对应：

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| `listen_addr` | 0.0.0.0:16974 | 监听地址 |
| `max_connections` | 20 | 最大连接数 |
| `max_inbound_connections` | 100 | 最大入站连接 |
| `max_outbound_connections` | 20 | 最大出站连接 |
| `connect_timeout_ms` | 4000 | 连接超时（毫秒） |
| `read_timeout_ms` | 4000 | 读取超时（毫秒） |
| `blacklisting_period_secs` | 3600 | 黑名单期限（秒） |
| `use_websockets` | true | 使用 WebSocket |
| `max_request_size` | 64MB | 最大请求大小 |
| `max_response_size` | 64MB | 最大响应大小 |

## 守护进程

### ConnectionDaemon（连接守护进程）
- 间隔：5 秒
- 职责：维护出站连接、自动重连、管理入站连接

### DiscoveryDaemon（发现守护进程）
- 间隔：30 秒
- 职责：获取更多节点、广播节点列表

### UnblacklistDaemon（黑名单守护进程）
- 间隔：60 秒
- 职责：检查黑名单过期、解除过期黑名单

### TransactionDaemon（交易守护进程）
- 间隔：30 秒
- 职责：批量发送交易、广播交易

## 消息处理器

已实现的处理器：

| RequestType | Handler | 状态 |
|-------------|---------|------|
| getInfo | GetInfoHandler | ✅ |
| getPeers | GetPeersHandler | ✅ |
| addPeers | AddPeersHandler | ✅ |
| getCumulativeDifficulty | GetCumulativeDifficultyHandler | ✅ |
| getMilestoneBlockIds | GetMilestoneBlockIdsHandler | ✅ |
| getNextBlockIds | GetNextBlockIdsHandler | ✅ |
| getNextBlocks | GetNextBlocksHandler | ✅ |
| getTransactions | GetTransactionsHandler | ✅ |
| processBlock | ProcessBlockHandler | ✅ |
| processTransactions | ProcessTransactionsHandler | ✅ |
| bundlerRate | BundlerRateHandler | ✅ |

## 黑名单管理

```rust
use p2p::BlacklistManager;

let blacklist = BlacklistManager::new();

// 添加到黑名单
blacklist.add_to_blacklist(
    addr,
    "Malicious behavior".to_string(),
    Some(3600), // 1 hour
    false,
).await;

// 检查是否在黑名单
if blacklist.is_blacklisted(&addr).await {
    println!("Peer is blacklisted");
}

// 清理过期条目
blacklist.clean_expired().await;
```

## 协议格式

### 请求格式
```json
{
  "requestType": "getInfo",
  "protocol": 1,
  "version": "1.0.0"
}
```

### 响应格式
```json
{
  "error": null,
  "errorCode": null,
  "version": "1.0.0",
  "platform": "Linux x86_64",
  "application": "NRCS",
  "timestamp": 1234567890
}
```

### 错误码

| 错误码 | 常量 | 说明 |
|--------|------|------|
| 1 | UNSUPPORTED_REQUEST_TYPE | 不支持的请求类型 |
| 2 | UNSUPPORTED_PROTOCOL | 不支持的协议 |
| 3 | UNKNOWN_PEER | 未知节点 |
| 4 | SEQUENCE_ERROR | 序列错误 |
| 5 | MAX_INBOUND_CONNECTIONS | 最大入站连接已满 |
| 6 | DOWNLOADING | 节点正在下载 |
| 7 | LIGHT_CLIENT | 轻客户端 |

## 示例

完整示例见 `examples/p2p_service.rs`

## 测试

```bash
cargo test -p p2p
```

## 许可证

MIT
