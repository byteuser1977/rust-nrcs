# NRCS 测试套件

NRCS Rust 实现的完整测试套件，用于验证 42 张数据库表交易同步、核心功能正确性以及与 Java NRCS 的兼容性。

**最后更新**: 2026-05-03  
**测试通过率**: **276/276 (100%)** ✅

## 🎯 测试概览

### 核心单元测试（Cargo Test）

| 模块 | 测试数 | 通过率 | 状态 |
|------|--------|--------|------|
| blockchain-types | 80 | 100% | ✅ |
| tx-engine | 101 | 100% | ✅ |
| orm | 18 | 100% | ✅ （含 Genesis 4个）|
| http-api | 23 | 100% | ✅ |
| crypto | 26 | 100% | ✅ |
| p2p | 12 | 100% | ✅ |
| consensus | 5 | 100% | ✅ |
| account | 3 | 100% | ✅ |
| contract | 1 | 100% | ✅ |
| 集成测试 | 7 | 100% | ✅ |
| **总计** | **276** | **100%** | ✅ |

### 42 张数据库表验证

✅ 所有核心业务表的 CRUD 操作已验证：
- **核心表** (5张): BLOCK, TRANSACTION, ACCOUNT, ACCOUNT_LEDGER, ACCOUNT_GUARANTEED_BALANCE
- **资产表** (7张): ASSET, ASSET_TRANSFER, ASSET_PROPERTY, ASSET_DIVIDEND, ASSET_DELETE, ASSET_HISTORY, ACCOUNT_ASSET
- **订单交易** (5张): ASK_ORDER, BID_ORDER, TRADE, COIN_ORDER_FXT, COIN_TRADE_FXT
- **Phasing 表** (7张): PHASING_POLL/VOTE 及子表
- **其他表** (18张): CURRENCY, TAGGED_DATA, SHUFFLING 等

## 📁 目录结构

```
tests/
├── README.md                          # 本文档
├── integration/                       # 集成测试脚本
│   ├── api/
│   │   └── test_api_compatibility.sh  # HTTP API 兼容性测试
│   ├── consensus/
│   │   └── test_consensus.sh          # PoS 共识算法测试
│   ├── p2p/
│   │   ├── test_block_sync.sh         # 区块同步测试（42张表）
│   │   └── test_node_discovery.sh     # 节点发现与握手测试
│   └── run_all.sh                     # 运行所有集成测试
├── crypto/
│   └── test_crypto_compatibility.sh   # 加密算法兼容性测试
├── performance/                       # 性能测试工具
│   ├── README.md                      # 性能测试说明
│   ├── benchmark_config.json          # k6 基准配置
│   ├── load_test.k6.js                # k6 负载测试脚本
│   └── tps_test.js                    # TPS 性能测试脚本
├── data/                              # 测试数据文件
│   ├── sample_blocks.json             # 示例区块数据
│   ├── sample_transactions.json       # 示例交易数据
│   └── test_vectors.json              # 加密算法测试向量
└── scripts/                           # 辅助工具脚本
    ├── setup_wsl_port_forwarding.ps1  # WSL 网络端口转发配置
    ├── start_test_env.sh              # 启动完整测试环境
    ├── stop_test_env.sh               # 停止测试环境
    └── test_p2p_connection.sh         # P2P 连接快速验证
```

**文件统计**: 18 个文件，6 个子目录

## ⚡ 快速开始

### 方式一：运行单元测试（推荐）

```bash
# 运行所有单元测试
cargo test

# 仅运行特定模块
cargo test -p blockchain-types      # 核心类型测试（80个）
cargo test -p tx-engine             # 交易引擎测试（101个）
cargo test -p orm --lib genesis     # Genesis 区块创建测试（4个）

# 完整质量检查流程
cargo fmt && cargo clippy -- -D warnings && cargo test
```

### 方式二：运行集成测试

#### 1. 环境准备

确保已安装：
- Rust 1.70+
- SQLite 3.x 或 PostgreSQL 14+
- curl, jq（JSON 处理）
- k6（可选，性能测试）

#### 2. 配置环境变量

```bash
# 数据库配置（SQLite 或 PostgreSQL 二选一）
export DB_URL="sqlite://test_nrcs.db"
# export DB_URL="postgres://user:password@localhost:5432/nrcs_test"

# 节点配置
export RUST_NRCS_HOST="localhost"
export RUST_NRCS_API_PORT="8080"
export RUST_NRCS_P2P_PORT="17974"
```

#### 3. 启动测试环境

```bash
# 启动数据库和节点
./tests/scripts/start_test_env.sh
```

#### 4. 运行集成测试

```bash
# 运行所有集成测试
./tests/integration/run_all.sh

# 或单独运行
./tests/crypto/test_crypto_compatibility.sh        # 加密算法
./tests/integration/p2p/test_node_discovery.sh      # P2P 节点发现
./tests/integration/p2p/test_block_sync.sh         # 区块同步（42张表）
./tests/integration/api/test_api_compatibility.sh  # HTTP API
./tests/integration/consensus/test_consensus.sh     # PoS 共识
```

#### 5. 停止环境

```bash
./tests/scripts/stop_test_env.sh
```

### 方式三：运行性能测试

```bash
# 使用 k6 进行 TPS 测试
k6 run tests/performance/tps_test.js

# 负载测试
k6 run tests/performance/load_test.k6.js
```

## 🧪 测试套件详细说明

### 1️⃣ 单元测试（Cargo Test）

位于各 crate 的 `tests/` 或 `src/` 目录：

#### blockchain-types（80 tests）
- 区块创建和序列化
- 交易类型和附件处理
- Hash 计算（SHA-256）
- Account ID 推导
- 配置常量验证

#### tx-engine（101 tests）
- 65 种交易类型验证
- 42 张表的 Repository 操作
- TransactionProcessor 集成测试
- 附件序列化/反序列化

#### orm（18 tests）
- Model 转换（Domain ↔ DB）
- BlockModel/TransactionModel/AccountModel
- **Genesis 区块创建**（4 tests）:
  - `test_genesis_creates_initial_state` - 初始状态创建
  - `test_genesis_block_height_is_zero` - 创世区块高度为 0
  - `test_genesis_base_target` - 初始基础目标值
  - `test_block_model_from_domain_genesis` - Domain→Model 转换

#### http-api（23 tests + 7 integration）
- PasswordFilter 安全过滤
- ApiProxy 请求转发
- RESTful API 端点兼容性

#### crypto（26 tests）
- Ed25519/Curve25519 签名验证
- NRCS 特有签名格式
- SM2/SM3/SM4 国密算法
- Reed-Solomon 编码

#### p2p（12 tests + 6 peers tests）
- Peer 连接管理
- BlockchainVerifier 验证
- WebSocket 通信协议
- 区块同步逻辑

### 2️⃣ 集成测试（Shell Scripts）

#### 加密算法兼容性测试 (P0)

**脚本**: `crypto/test_crypto_compatibility.sh`

**验证内容**:
- ✅ Ed25519 签名/验签与 Java 一致
- ✅ NRCS passphrase → 密钥对推导
- ✅ X25519 ECDH 密钥交换
- ✅ SM2/SM3/SM4 国密算法正确性

**依赖**: 无需外部服务，纯算法对比

---

#### P2P 网络互通测试 (P0)

**脚本**: 
- `integration/p2p/test_node_discovery.sh` - 节点发现
- `integration/p2p/test_block_sync.sh` - 区块同步

**验证内容**:
- ✅ 节点握手和身份交换
- ✅ 42 张表数据完整同步
- ✅ 区块传播顺序正确
- ✅ 交易广播机制

**前置条件**: 需要运行中的 Java/Rust 节点

---

#### HTTP API 兼容性测试 (P0)

**脚本**: `integration/api/test_api_compatibility.sh`

**验证端点**:
```
GET  /api/v1/accounts/:address           # 账户查询
GET  /api/v1/blocks/height               # 当前高度
GET  /api/v1/blocks/:height              # 按高度查询区块
POST /api/v1/transactions                # 提交交易
GET  /api/v1/state                      # 节点状态
```

**验证内容**:
- ✅ JSON 格式与 Java NRCS 兼容
- ✅ 字段名称映射正确（camelCase ↔ snake_case）
- ✅ 错误响应格式统一

---

#### PoS 共识一致性测试 (P1)

**脚本**: `integration/consensus/test_consensus.sh`

**验证内容**:
- ✅ 出块者选择算法（Slot 时间 × 有效余额）
- ✅ 基础目标值计算公式
- ✅ 累积难度更新规则
- ✅ Generation Signature 验证

---

#### 性能基准测试 (P2)

**脚本**: `performance/tps_test.js`, `performance/load_test.k6.js`

**指标**:
- **TPS**: 目标 800+（对比 Java 500+）
- **P95 延迟**: < 150ms
- **内存占用**: < 1GB

**使用方法**:
```bash
# 安装 k6
curl -s https://github.com/grafana/k6/releases/download/v0.47.0/k6-v0.47.0-linux-amd64.tar.gz | tar xz
sudo mv k6-v0.47.0-linux-amd64/k6 /usr/local/bin/

# 运行 TPS 测试
k6 run tests/performance/tps_test.js --vus 10 --duration 60s
```

## 📊 测试数据

### 测试向量 (`data/test_vectors.json`)

包含标准化的加密算法测试用例：
- Ed25519 签名/验签向量（来自 RFC 8032）
- NRCS 特有签名格式示例
- SM2/SM3/SM4 国密测试向量

### 示例数据 (`data/sample_blocks.json` & `sample_transactions.json`)

包含从真实网络捕获的示例数据：
- Genesis 区块及后续 10 个区块
- 各类交易样本（Payment, AssetIssuance, Shuffling 等）
- 用于验证序列化/反序列化正确性

## 🔧 工具脚本说明

### 环境管理

| 脚本 | 功能 | 用法 |
|------|------|------|
| `scripts/start_test_env.sh` | 启动完整测试环境（DB + Node） | `./start_test_env.sh` |
| `scripts/stop_test_env.sh` | 停止所有测试进程 | `./stop_test_env.sh` |
| `scripts/setup_wsl_port_forwarding.ps1` | WSL2 网络端口转发（Windows） | `powershell -File setup_wsl_port_forwarding.ps1` |
| `scripts/test_p2p_connection.sh` | 快速验证 P2P 连接 | `./test_p2p_connection.sh` |

### WSL 环境配置（Windows 用户）

如果在 WSL2 中开发，需要端口转发才能让 Windows 访问 Linux 节点：

```powershell
# 在 PowerShell（管理员）中执行
cd \path\to\rust-nrcs\tests\scripts
.\setup_wsl_port_forwarding.ps1
```

这将配置：
- 17974 → WSL P2P 端口
- 17976 → WSL HTTP API 端口
- 8080 → WSL API 代理端口

## ✅ 验证清单

每次提交前应确保：

- [ ] `cargo test` 全部通过（276 tests）
- [ ] `cargo clippy -- -D warnings` 零警告
- [ ] `cargo fmt` 代码格式化正确
- [ ] Genesis 测试通过（`cargo test -p orm --lib genesis`）
- [ ] 至少运行一次集成测试（`./tests/integration/run_all.sh`）

## 🐛 故障排查

### 常见问题

#### 1. 单元测试失败

**症状**: `cargo test` 报错

**解决方案**:
```bash
# 清理并重新编译
cargo clean && cargo test

# 查看详细错误信息
cargo test -- --nocapture

# 仅运行失败的测试
cargo test failed_test_name
```

#### 2. Genesis 测试失败："near "," syntax error"

**原因**: SQL INSERT 语句参数占位符缺失

**解决方案**:
```bash
# 检查 ORM 的 SQL 实现
grep -r "VALUES (" crates/orm/src/repository/

# 应该看到正确的占位符：
# SQLite: VALUES (?, ?, ?)
# PostgreSQL: VALUES ($1, $2, $3)
```

#### 3. 集成测试连接失败

**症状**: 无法连接到节点或数据库

**检查项**:
```bash
# 1. 检查节点是否运行
curl http://localhost:8080/api/v1/state

# 2. 检查数据库连接
sqlite3 nrcs.db "SELECT COUNT(*) FROM block;"

# 3. 检查端口占用
netstat -tlnp | grep -E "17974|8080|5432"
```

#### 4. P2P 同步超时

**症状**: `test_block_sync.sh` 超时

**解决方案**:
```bash
# 增加 timeout（默认 300 秒）
export BLOCK_SYNC_TIMEOUT=600

# 检查 Java 节点是否有足够区块
curl http://192.168.2.164:7876/api/v1/blocks/height

# 检查网络延迟
ping 192.168.2.164
```

#### 5. 性能测试 k6 找不到

**症状**: `k6: command not found`

**安装方法**:
```bash
# Linux/macOS
curl -s https://github.com/grafana/k6/releases/download/v0.47.0/k6-v0.47.0-linux-amd64.tar.gz | \
  tar xz --strip-components=1 && sudo mv k6 /usr/local/bin/

# 验证安装
k6 version
```

## 📈 CI/CD 集成

### GitHub Actions 示例

```yaml
name: Tests

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run unit tests
        run: cargo test
        
      - name: Run clippy
        run: cargo clippy -- -D warnings
        
  integration-tests:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_USER: nrcs
          POSTGRES_PASSWORD: password
          POSTGRES_DB: nrcs_test
        ports:
          - 5432:5432
          
    steps:
      - uses: actions/checkout@v2
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y curl jq
          
      - name: Run integration tests
        run: |
          ./tests/scripts/start_test_env.sh
          ./tests/integration/run_all.sh
          ./tests/scripts/stop_test_env.sh
        env:
          DB_URL: postgres://nrcs:password@localhost:5432/nrcs_test
```

## 📝 开发指南

### 添加新测试

#### 1. 添加单元测试

在对应 crate 的 `tests/` 目录或模块内：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_feature() {
        // 准备测试数据
        let input = create_test_data();
        
        // 执行待测功能
        let result = function_to_test(&input).await;
        
        // 验证结果
        assert!(result.is_ok());
        assert_eq!(result.unwrap().expected_field, 42);
    }
}
```

#### 2. 添加集成测试

在 `tests/integration/` 创建新脚本：

```bash
#!/bin/bash
set -e  # 遇到错误立即退出

echo "=== Running New Integration Test ==="

# 1. 准备环境
setup_test_environment

# 2. 执行测试
run_test_cases

# 3. 清理
cleanup_test_environment

echo "=== All Tests Passed ==="
exit 0
```

#### 3. 更新文档

添加测试后请更新：
- 本 README 的"测试概览"表格
- 相关测试脚本的注释说明
- 如涉及新表，更新 ORM README 的表清单

### 测试命名规范

- **单元测试**: `test_<功能>_<场景>` (如 `test_block_creation_with_valid_data`)
- **集成测试**: `test_<模块>_<功能>.sh` (如 `test_block_sync.sh`)
- **性能测试**: `<指标>_test.js` (如 `tps_test.js`)

## 🎯 当前测试覆盖重点（v2.5.0）

### 已完成验证

- ✅ **42 张数据库表**的完整 CRUD 操作
- ✅ **Genesis 区块**创建和初始化（4 个专项测试）
- ✅ **65 种交易类型**的处理逻辑
- ✅ **48 个 Repository** 的注入和使用
- ✅ **双数据库引擎**（SQLite / PostgreSQL）
- ✅ **Java NRCS 兼容性**（签名、API、数据格式）

### 待增强

- [ ] 更多边界条件测试（极端输入值）
- [ ] 并发安全性测试（多线程操作同一表）
- [ ] 大数据量性能回归测试
- [ ] 故障恢复测试（数据库中断后重启）

## 📚 相关文档

- [主项目 README](../README.md) - 项目总览和架构
- [ORM 模块文档](../crates/orm/README.md) - 42 张表详细说明
- [开发规范](../.trae/rules/develop.md) - NRCS Rust 编码规范
- [42 张表修复计划](../.trae/plan/42-tables-full-fix-plan.md) - 完整修复方案

---

**NRCS 测试套件** - 确保 42 张表交易同步的正确性和可靠性 ✅

*最后更新: 2026-05-03 | 测试总数: 276 | 通过率: 100%*
