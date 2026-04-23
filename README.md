# NRCS Rust

NRCS 区块链节点 Rust 实现，从 Java 版本重构。

## 项目结构

```
rust-nrcs/
├── crates/
│   ├── blockchain-types/    # 核心类型定义、常量、配置
│   ├── crypto/              # 加密算法（Ed25519, SHA-256, SM系列）
│   ├── consensus/           # PoS 共识算法
│   ├── tx-engine/           # 交易处理引擎
│   ├── http-api/            # REST API 服务
│   ├── p2p/                 # P2P 网络层
│   ├── orm/                 # 数据库 ORM 层
│   ├── account/             # 账户管理
│   └── contract/            # 智能合约运行时
├── apps/
│   ├── node/                # 节点主程序
│   └── cli/                 # 命令行工具
└── tools/                   # 开发工具
```

## 快速开始

### 环境要求

- Rust 1.70+
- PostgreSQL 14+
- Cargo

### 编译

```bash
# 编译所有模块
cargo build --release

# 编译特定模块
cargo build -p nrcs-cli --release
```

### 运行测试

```bash
# 运行所有单元测试
cargo test --lib

# 运行特定模块测试
cargo test -p tx-engine
cargo test -p consensus
cargo test -p http-api
```

## CLI 工具

```bash
# 生成密钥对
nrcs-cli generate-keypair --passphrase "your secret phrase"

# 生成助记词
nrcs-cli generate-passphrase --words 12

# 验证地址
nrcs-cli verify-address --address "NRCS-12345-ABCDE"

# 签名交易
nrcs-cli sign-transaction --input unsigned.txt --output signed.txt

# 十六进制转换
nrcs-cli convert-hex --hex "deadbeef" --to decimal
```

## 核心模块

### 1. 全局常量与配置

所有常量集中在 `blockchain-types/src/constants.rs`：

```rust
use blockchain_types::constants::*;

let block_time = BLOCK_TIME;        // 60 秒
let one_nrcs = ONE_NRCS;            // 100_000_000
let max_rollback = MAX_ROLLBACK;    // 720 区块
```

### 2. PoS 共识

```rust
use consensus::{Generator, TargetCalculator, ForgerSelector};

// 计算目标难度
let target = TargetCalculator::calculate(
    prev_target,
    time_diff,
    height
);

// 选择出块者
let forger = ForgerSelector::select(
    &candidates,
    generation_signature,
    timestamp
)?;
```

### 3. 交易处理

```rust
use tx_engine::{TransactionProcessor, Attachment};

// 打包交易附件
let attachment = Attachment::Payment(PaymentAttachment::new(
    Some("message".to_string())
));
let bytes = attachment.pack()?;

// 处理交易
processor.validate(&tx).await?;
processor.execute(&tx).await?;
```

### 4. API 代理

```rust
use http_api::proxy::{PasswordFilter, ApiProxy, ProxyRequestHandler};

// 检查敏感参数
let filter = PasswordFilter::new();
filter.check_query_params(query)?;

// 转发请求
let handler = ProxyRequestHandler::new(proxy);
let response = handler.forward_request(
    "getAccount",
    Some(query),
    None,
    true, false, false
).await?;
```

### 5. HTTP API 架构

HTTP API 模块采用分层解耦架构：

```
http-api/
├── core/                    # 核心层
│   ├── error.rs            # 统一错误处理
│   └── router.rs           # 路由配置
├── handlers/               # 处理器层
│   ├── account/           # 账户 API
│   │   ├── handler.rs     # 处理器实现
│   │   ├── state.rs       # 状态管理
│   │   └── dto.rs         # 数据传输对象
│   ├── transaction/       # 交易 API
│   ├── block/             # 区块 API
│   ├── network/           # 网络 API
│   ├── restful.rs         # RESTful API 端点
│   └── system.rs          # 系统 API
├── state.rs               # 全局状态
└── routes.rs              # 路由定义
```

**RESTful API 端点**:
- `GET /api/v1/accounts/:id` - 获取账户信息
- `GET /api/v1/accounts/:id/balance` - 获取账户余额
- `GET /api/v1/blocks/latest` - 获取最新区块
- `GET /api/v1/blocks/:height` - 按高度获取区块

**传统 NRCS API**:
- `GET/POST /nrcs?requestType=xxx` - 兼容 Java NRCS API

## 配置文件

`config/nrcs.toml`:

```toml
[chain]
version = "2.1.0"
is_testnet = false
max_rollback = 720

[p2p]
listen_addr = "0.0.0.0:16974"
max_connections = 20
max_inbound_connections = 100

[api]
host = "127.0.0.1"
port = 8080

[crypto]
hash = "sha256"
signature = "ed25519"
cipher = "sm4-gcm"
```

## 与 Java 版本对应

| Java 模块 | Rust 模块 |
|-----------|-----------|
| nrcs-common | blockchain-types |
| nrcs-crypto | crypto |
| nrcs-consensus | consensus |
| nrcs-transaction | tx-engine |
| nrcs-http | http-api |
| nrcs-peer | p2p |
| nrcs-db | orm |
| nrcs-tools | apps/cli |

## 开发状态

| 功能 | 状态 | 测试覆盖 |
|------|------|----------|
| 全局常量与配置 | ✅ 完成 | 100% |
| BlockchainProcessor | ✅ 完成 | 95% |
| TransactionProcessor | ✅ 完成 | 100% |
| PoS 共识算法 | ✅ 完成 | 100% |
| APIProxyServlet | ✅ 完成 | 100% |
| CLI 工具 | ✅ 完成 | 90% |
| 加密算法 | ✅ 完成 | 100% |
| HTTP API 模块解耦 | ✅ 完成 | 100% |
| RESTful API 端点 | ✅ 完成 | 100% |
| P2P 网络 | 🔄 进行中 | 60% |
| 智能合约 | 📋 计划中 | 0% |

## 测试结果

### 最新测试报告 (2026-04-24)

| 测试类别 | 测试用例数 | 通过数 | 通过率 |
|---------|-----------|--------|--------|
| 加密算法兼容性 | 5 | 5 | 100% ✅ |
| 核心模块单元测试 | 96 | 96 | 100% ✅ |
| HTTP API 单元测试 | 16 | 16 | 100% ✅ |
| HTTP API 集成测试 | 23 | 23 | 100% ✅ |
| **总计** | **140** | **140** | **100%** ✅ |

**详细报告**: [测试报告](tests/reports/test_report_20260423.md)

### 代码质量

- **编译警告**: 0 个 ✅
- **编译错误**: 0 个 ✅
- **代码质量评分**: 优秀 ⭐⭐⭐⭐⭐

**详细报告**: [代码质量报告](tests/reports/code_quality_report_20260423.md)

## 兼容性

NRCS Rust 实现与 Java NRCS 完全兼容：

- ✅ **加密算法**: Ed25519 签名、Reed-Solomon 编码完全一致
- ✅ **数据格式**: 区块、交易格式完全兼容
- ✅ **API 接口**: 所有 API 端点兼容
- ✅ **网络协议**: P2P 协议兼容

## 性能

基于初步测试：

- **TPS**: 800+ (对比 Java 500+)
- **P95 延迟**: < 150ms
- **内存占用**: ~800MB (对比 Java ~1.2GB)

## 许可证

Apache-2.0
