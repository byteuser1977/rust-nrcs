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

| 功能 | 状态 |
|------|------|
| 全局常量与配置 | ✅ 完成 |
| BlockchainProcessor | ✅ 完成 |
| TransactionProcessor | ✅ 完成 |
| PoS 共识算法 | ✅ 完成 |
| APIProxyServlet | ✅ 完成 |
| CLI 工具 | ✅ 完成 |
| P2P 网络 | 🔄 进行中 |
| 智能合约 | 📋 计划中 |

## 许可证

Apache-2.0
