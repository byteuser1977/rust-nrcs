# P2P 连接问题解决方案

## 当前状态

### ✅ 成功部分
- Rust节点成功启动并监听在 `172.31.75.249:17974`
- SQLite数据库连接成功
- WebSocket连接成功建立到Java节点（`192.168.2.164:17974`）
- Java NRCS节点正常运行（区块高度：1367035）

### ❌ 问题
Java NRCS节点将Rust节点列入黑名单：
- 原因：`java.net.SocketTimeoutException: Connect Timeout`
- 黑名单周期：10分钟（600000毫秒）

## 解决方案

### 方案1：修改Java NRCS配置（推荐）

在Java NRCS节点的配置文件 `nrcs.properties` 中添加或修改以下配置：

```properties
# 将黑名单周期设置为1分钟（60000毫秒）
nrcs.blacklistingPeriod=60000

# 或者完全禁用黑名单（仅用于测试）
# nrcs.blacklistingPeriod=0
```

然后重启Java NRCS节点。

### 方案2：重启Java NRCS节点

重启Java NRCS节点会清除所有黑名单：

```bash
# 停止Java NRCS节点
# 然后重新启动
```

### 方案3：等待黑名单自动解除

等待10分钟后，黑名单会自动解除。

## 验证步骤

1. 应用上述任一方案后，重新启动Rust节点：

```bash
cd /mnt/d/workspace/git/rust-nrcs
cargo run --release --bin nrcs-node
```

2. 检查日志中是否出现以下成功信息：
   - `Handshake successful`
   - `Peer connected`
   - 开始接收区块数据

3. 验证P2P连接：

```bash
# 检查Rust节点的对等节点
curl http://127.0.0.1:17976/nrcs?requestType=getPeers

# 检查区块链状态
curl http://127.0.0.1:17976/nrcs?requestType=getBlockchainStatus
```

## 配置文件位置

### Rust节点配置
文件：`/mnt/d/workspace/git/rust-nrcs/config/local.toml`

```toml
[p2p]
listen_addr = "0.0.0.0:17974"
external_addr = "172.31.75.249:17974"
bootstrap_nodes = ["192.168.2.164:17974"]
max_connections = 100
connection_ttl_secs = 600
protocol_id = "NRCS"
```

### Java NRCS配置
文件：`<java-nrcs-home>/conf/nrcs.properties`

关键配置：
- `nrcs.peerServerPort=17974` - P2P端口
- `nrcs.blacklistingPeriod=600000` - 黑名单周期（毫秒）
- `nrcs.myAddress=` - 外部地址（如果需要）

## 网络拓扑

```
Rust Node (172.31.75.249:17974)  <--->  Java Node (192.168.2.164:17974)
           |                                    |
           v                                    v
   SQLite Database                      Java Database
   (nrcs_test.db)                    (PostgreSQL/H2)
```

## 下一步

1. 选择上述解决方案之一
2. 应用配置更改
3. 重新测试P2P连接
4. 验证区块同步功能
5. 测试数据一致性

## 测试命令

```bash
# 运行自动化测试脚本
bash /mnt/d/workspace/git/rust-nrcs/tests/scripts/test_p2p_connection.sh

# 或者手动测试
cargo run --release --bin nrcs-node 2>&1 | grep -E "(INFO|WARN|ERROR|blacklist|connected)"
```
