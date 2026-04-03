# P2P 互通测试计划

## 目标
验证 Rust P2P 节点与 Java NRCs 节点的协议兼容性。

## 前提条件

### 环境准备
1. 启动 Java NRCs 节点（本地）
   - 确保 `nrcs-main` 项目编译完成
   - 配置文件 `nrcs.properties` 中启用 WebSocket 和 HTTP：
     ```
     nrcs.enableWebSocket=true
     peers.maxConnections=50
     ```
   - 启动：`java -jar nrcs.jar`
   - 默认端口：HTTP 7070, WebSocket 7071

2. 启动 Rust P2P 节点
   - 修改 `config/local.toml`：
     ```toml
     [p2p]
     listen = "/ip4/0.0.0.0/tcp/7071"  # 与 Java 一致端口
     use_websocket = true
     ```
   - 运行：`cargo run --bin node`
   - 确保日志显示 WebSocket 监听端口

3. 网络检查
   ```bash
   netstat -an | findstr :7071
   # 应显示 LISTENING（Rust 和 Java 都能绑）
   ```

---

## 测试用例

### TC-01: HTTP POST getInfo 基础交换

**步骤**：
1. 发送请求到 Rust 节点的 `/peer` 端点：
   ```bash
   curl -X POST http://localhost:7071/peer \
     -H "Content-Type: application/json" \
     -d '{"requestType":"getInfo","protocol":1}'
   ```
2. 预期响应：JSON 包含 `version`, `application`, `platform`, `services`, `shareAddress`, `apiPort`, `apiSSLPort`, `hallmark` 等字段
3. 同样测试 Java 节点：`curl -X POST http://localhost:7070/peer -d '{"requestType":"getInfo","protocol":1}'`

**验证**：
- Rust 响应格式与 Java 一致
- 无 `"error"` 字段
- `services` 字段与 Java 的 `Peers.MY_SERVICES` 匹配（可能需要硬编码对照）

---

### TC-02: WebSocket getInfo 交换

**步骤**：
1. 使用 WebSocket 客户端连接 Rust 节点的 `ws://localhost:7071/peer`
   - 发送二进制帧（按 Java 格式）：
     - 版本(4) = 1 (big-endian)
     - requestId(8) = 随机 long (如 12345)
     - flags(4) = 0
     - length(4) = JSON 体长度
     - body = `{"requestType":"getInfo","protocol":1}`
2. 预期响应：二进制帧包含 JSON 响应（gzip 可选）
3. 连接 Java 节点做相同测试

**验证**：
- 帧头解析正确
- 响应 JSON 字段完整
- 支持压缩（可测 flags & 0x1 情况）

---

### TC-03: 添加互为 peer（addPeers）

**步骤**：
1. 获取 Rust 节点的公网地址或本地地址：`announcedAddress` 从 getInfo 中获取
2. 向 Java 节点发送 `addPeers` 请求，包含 Rust 节点地址：
   ```bash
   curl -X POST http://localhost:7070/peer \
     -d '{"requestType":"addPeers","protocol":1,"peers":[{"address":"192.168.1.100","port":7071,"services":"0"}]}'
   ```
3. 在 Java 节点日志查看是否记录新 peer
4. 反向：向 Rust 节点添加 Java 节点地址

**验证**：
- `peers` 数组应反映新节点
- 双方日志显示连接尝试

---

### TC-04: 区块同步 - getNextBlocks

**准备**：
- Java 节点应已挖出至少 2 个区块（高度 > 1）
- Rust 节点处于同步模式（blockchain_state=some value）

**步骤**：
1. 从 Rust 节点向 Java 节点发送 `getNextBlocks`：
   ```bash
   curl -X POST http://localhost:7070/peer \
     -d '{"requestType":"getNextBlocks","protocol":1,"blockId":"1","limit":10}'
   ```
2. 预期响应：`{"nextBlocks":[{block JSON},...]}`

**验证**：
- 返回区块数量 ≤ limit
- 每个区块 JSON 字段完整：`version`, `timestamp`, `previousBlock`, `previousBlockHash`, `totalAmountNQT`, `totalFeeNQT`, `payloadLength`, `payloadHash`, `generatorPublicKey`, `generationSignature`, `blockSignature`, `transactions[]`
- 区块高度递增，链连续（可选验证）

---

### TC-05: 区块广播 - processBlock

**步骤**：
1. 从 Java 节点向 Rust 节点发送一个测试区块（或反之）
   ```bash
   curl -X POST http://localhost:7071/peer \
     -d @sample_block.json
   ```
   `sample_block.json` 是一个完整的区块 JSON（从 Java 节点 `/getNextBlocks` 获取）
2. 预期响应：`{}`（空对象）

**验证**：
- 检查 Rust 节点日志：是否收到区块，是否验证通过
- 检查 Rust 节点数据库：区块是否存入 `block` 表
- 发送重复区块应返回错误（可选）

---

### TC-06: 交易传播 - processTransactions / getUnconfirmedTransactions

**步骤**：
1. 发送交易到 Rust 节点的 `processTransactions`：
   ```bash
   curl -X POST http://localhost:7071/peer \
     -d '{"requestType":"processTransactions","protocol":1,"transactions":[{tx JSON}]}'
   ```
2. 查询未确认交易：`getUnconfirmedTransactions`

**验证**:
- 交易进入内存池（可通过 Java 节点查询）
- 交易格式与 Java 一致

---

## 错误场景测试

- **协议版本过高**: send `protocol: 3` → expect `{"error":"UNSUPPORTED_PROTOCOL"}`
- **请求类型未实现**: send `requestType:"bogus"` → `{"error":"UNSUPPORTED_REQUEST_TYPE"}`
- **未先 getInfo 就 processBlock**: 在 Java 中模拟 → `{"error":"SEQUENCE_ERROR"}`
- **节点黑名单**: 模拟恶意请求（过大 payload）→ Java/Rust 返回 `BLACKLISTED`

---

## 自动化测试脚本

### 01_healthcheck.sh

```bash
#!/bin/bash
set -e

JAVA_HOST="localhost"
JAVA_PORT=7070
RUST_HOST="localhost"
RUST_PORT=7071

echo "=== TC-01: HTTP getInfo health check ==="
curl -s -X POST http://$RUST_HOST:$RUST_PORT/peer \
  -H "Content-Type: application/json" \
  -d '{"requestType":"getInfo","protocol":1}' | jq .

curl -s -X POST http://$JAVA_HOST:$JAVA_PORT/peer \
  -H "Content-Type: application/json" \
  -d '{"requestType":"getInfo","protocol":1}' | jq .

echo "=== TC-02: Add peers (mutual) ==="
# Extract announcedAddress from Rust response
RUST_INFO=$(curl -s -X POST http://$RUST_HOST:$RUST_PORT/peer \
  -H "Content-Type: application/json" \
  -d '{"requestType":"getInfo","protocol":1}')
RUST_ADDR=$(echo $RUST_INFO | jq -r '.announcedAddress // "localhost"')

# Add Rust to Java
curl -s -X POST http://$JAVA_HOST:$JAVA_PORT/peer \
  -H "Content-Type: application/json" \
  -d "{\"requestType\":\"addPeers\",\"protocol\":1,\"peers\":[{\"address\":\"$RUST_ADDR\",\"port\":7071,\"services\":\"0\"}]}" | jq .

# Add Java to Rust
JAVA_INFO=$(curl -s -X POST http://$JAVA_HOST:$JAVA_PORT/peer \
  -H "Content-Type: application/json" \
  -d '{"requestType":"getInfo","protocol":1}')
JAVA_ADDR=$(echo $JAVA_INFO | jq -r '.announcedAddress // "localhost"')

curl -s -X POST http://$RUST_HOST:$RUST_PORT/peer \
  -H "Content-Type: application/json" \
  -d "{\"requestType\":\"addPeers\",\"protocol\":1,\"peers\":[{\"address\":\"$JAVA_ADDR\",\"port\":7070,\"services\":\"0\"}]}" | jq .

echo "All health checks passed!"
```

### 02_block_sync.sh

```bash
#!/bin/bash
set -e

# 假设 Java 节点已知至少 1 个区块高度
# 获取最新的区块 ID（从 Java 的 getNextBlocks 接口用特殊 blockId=0）

echo "=== Fetching blocks from Java node ==="
curl -s -X POST http://localhost:7070/peer \
  -H "Content-Type: application/json" \
  -d '{"requestType":"getNextBlocks","protocol":1,"blockId":"0","limit":10}' \
  | jq '.nextBlocks | length'


echo "Block sync test complete."
```

---

## 测试环境配置建议

```bash
# 保存脚本到 tests/p2p-integration/
chmod +x tests/p2p-integration/*.sh
cd tests/p2p-integration
./01_healthcheck.sh
./02_block_sync.sh
```

---

## 报告与问题追踪

测试完成后，请填写：
- ✅ 所有 TC 通过 / ❌ 失败用例
- 性能指标（RTT、吞吐）
- 不兼容字段（如果有）
- 建议的协议修正

报告输出到：`docs/p2p-interop-report.md`

---

**祝测试顺利！** 如果发现不兼容，优先检查：
- 二进制帧字节序（Java 使用 big-endian, Rust 使用 `i32::from_be_bytes()`）
- JSON 字段名大小写（camelCase）
- 整数类型长度（Java `long` = 64 位, Rust `i64`）