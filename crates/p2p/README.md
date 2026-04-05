# P2P Protocol Implementation

Rust 实现的 P2P 网络协议，与 Java NRCs 客户端完全兼容。

## 协议概述

本实现基于 Java NRCs 的 `PeerWebSocket.java` 和 `PeerServlet.java`，支持：

- **WebSocket** 二进制帧协议
- **HTTP POST** `/peer` 端点
- **11 种 RPC 方法**：GetInfo, GetPeers, AddPeers, GetNextBlocks, ProcessBlock, ProcessTransactions, GetTransactions, GetUnconfirmedTransactions, GetCumulativeDifficulty, GetMilestoneBlockIds, GetNextBlockIds, BundlerRate

## 消息格式

### 二进制帧（WebSocket）

```
+----------------+----------------+----------------+----------------+----------------+
| Magic "P2P\0"  | Version (int)  | RequestID (long) | Flags (int)  | Length (int)   |
+----------------+----------------+----------------+----------------+----------------+
| Body (bytes, length bytes, optional GZIP compressed)                         |
+------------------------------------------------------------------------------+
```

- **Magic**: `0x50 0x32 0x50 0x00` ("P2P\\0")
- **Version**: 协议版本（当前为 1）
- **RequestID**: 客户端生成的唯一请求 ID（响应需原值返回）
- **Flags**: 位标志（bit 0 = 1 表示 GZIP 压缩）
- **Length**: Body 长度（字节）
- **Body**: JSON 编码的请求/响应（UTF-8）

### JSON 请求

```json
{
  "requestType": "getInfo",
  "protocol": 1,
  // 其他字段（根据 requestType 而定）
}
```

### JSON 响应

```json
{
  "error": "ERROR_CODE",  // 可选，成功时省略
  // 或成功数据字段
}
```

## 快速开始

### 1. 添加依赖

```toml
[dependencies]
p2p = { path = "../crates/p2p" }
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
```

### 2. 启动服务器

```rust
use p2p::{websocket::WebsocketServer, http::HttpConfig, serve};
use std::sync::Arc;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // 初始化节点管理器
    let peers = Arc::new(p2p::peer::Peers::new(
        p2p::peer::Peer::new("127.0.0.1:7070".parse().unwrap(), false)
    ));

    // 创建处理器
    let handler = Arc::new(p2p::handlers::Handler::new(Arc::clone(&peers)));

    // 启动 WebSocket 服务器（端口由配置 p2p.listen 指定）
    let ws_config = p2p::websocket::WebsocketConfig {
        listen_addr: "127.0.0.1:7070".parse().unwrap(),
        max_connections: 100,
    };
    let ws_server = p2p::websocket::WebsocketServer::new(ws_config, Arc::clone(&peers), Arc::clone(&handler));
    tokio::spawn(async move {
        ws_server.serve().await.expect("WebSocket server failed");
    });

    // 启动 HTTP 服务器（可选，端口 7071）
    let http_config = HttpConfig {
        listen_addr: "127.0.0.1:7071".parse().unwrap(),
    };
    tokio::spawn(async move {
        serve(http_config, peers, handler).await.expect("HTTP server failed");
    });

    // 保持运行
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
```

## 兼容性说明

- 所有 JSON 字段名与 Java 端完全一致（使用 `snake_case` 或 `camelCase` 如 `requestType`）
- `requestType` 值使用小写驼峰（如 `getInfo`, `processBlock`）
- `protocol` 必须是整数（1 或 2）
- 整数类型：Java 可能使用 `long`/`int`，Rust 使用 `i64`/`i32`/`u64`，JSON 数字在解析时会自动转换

## 待实现功能

- [ ] 区块存储与读取（对接数据库/链状态）
- [ ] 交易验证与内存池
- [ ] 节点发现与自动连接
- [ ] 黑名单持久化
- [ ] Hallmark 身份验证
- [ ] 请求流控与限速
- [ ] TLS/SSL 支持
- [ ] 更多 RPC 实现（GetCumulativeDifficulty, GetMilestoneBlockIds, GetNextBlockIds, BundlerRate）

## 测试

与 Java NRCs 节点互通测试请参考 `docs/p2p-protocol-implementation-report.md`。

## 许可证

MIT