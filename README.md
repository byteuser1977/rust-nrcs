# NRCS Rust

NRCS 区块链节点 Rust 实现，从 Java 版本重构，完全兼容 NRCS Java 节点。

## 项目亮点

### 核心成就

- **42 张数据库表** 完整支持，与 Java NRCS 完全兼容
- **65 种交易类型** 全部实现，48 个 Repository 集成
- **双数据库引擎** 支持 SQLite（开发）和 PostgreSQL（生产），通过 ORM 抽象层统一访问
- **P2P 硬编码 SQL 已迁移至 ORM**，切换数据库引擎时上层模块无需改动

### 近期优化（2026-05-05）

| 优化项 | 说明 |
|--------|------|
| **PostgreSQL 测试全量通过** | 32 个 PostgreSQL 集成测试全部通过，ORM 双引擎均已生产就绪 |
| **PostgreSQL 标识符修复** | 修复 pg.rs 中表名/列名大小写引用问题，迁移脚本标识符加引号 |
| **Node PostgreSQL 分支启用** | 取消注释 PostgreSQL 分支，支持直接使用 PostgreSQL 启动节点 |
| **日志系统重构** | info 日志精简（~90 处降级为 debug）、tracing 配置化（RUST_LOG / config / 默认三级优先级）、SQL 查询日志降为 debug |
| **ORM 事务感知方法** | 新增 `insert_tx`/`update_next_block_id_tx`/`delete_by_db_id_tx` 等事务版本方法，p2p crate 不再直接依赖 sqlx |
| **数据库无关类型** | `DbPool`（AnyPool）/ `DbTransaction` 类型别名，屏蔽底层数据库差异 |
| **常量化** | 硬编码值统一使用 `BLOCK_VERSION`、`ONE_NRCS` 等常量 |

## 项目结构

```
rust-nrcs/
├── crates/
│   ├── blockchain-types/    # 核心类型定义、常量、配置
│   ├── crypto/              # 加密算法（Ed25519, Curve25519, SM系列）
│   ├── consensus/           # PoS 共识算法
│   ├── tx-engine/           # 交易处理引擎（65种交易类型 + 48个Repository）
│   ├── http-api/            # REST API 服务
│   │   └── core/middleware/ # API 中间件（密码过滤、请求代理）
│   ├── p2p/                 # P2P 网络层（无硬编码 SQL，纯 ORM 访问）
│   │   ├── daemon/          # 守护进程（同步、发现、连接等）
│   │   ├── handlers/        # 协议处理器
│   │   └── verifier.rs      # 区块验证器（使用 ORM _tx 方法）
│   ├── orm/                 # 数据库 ORM 层
│   │   ├── models/          # 数据模型（Model 层，FromRow 派生）
│   │   ├── repository/      # Repository 接口和实现
│   │   │   ├── traits.rs    # Repository Trait 定义（含事务感知方法）
│   │   │   ├── sqlite.rs    # SQLite 实现（50+ Repository）
│   │   │   └── pg.rs        # PostgreSQL 实现（已启用）
│   │   ├── connection.rs    # 连接管理（DbPool/DbTransaction 类型别名）
│   │   ├── transaction.rs   # DatabaseTransaction 封装
│   │   └── migrations/      # 数据库迁移脚本
│   ├── account/             # 账户管理
│   └── contract/            # 智能合约运行时（WASM）
├── apps/
│   ├── node/                # 节点主程序
│   └── cli/                 # 命令行工具
├── config/                  # 配置文件
│   ├── default.toml         # 默认配置（含 log_level 字段）
│   └── local.toml           # 本地覆盖配置
└── tests/                   # 测试套件
```

## 快速开始

### 环境要求

- Rust 1.70+
- SQLite 3.x（开发环境默认数据库）
- PostgreSQL 14+（生产环境推荐数据库）
- Cargo

### 编译

```bash
# 开发编译
cargo build

# 生产构建
cargo build --release

# 编译特定模块
cargo build -p nrcs-node --release
```

### 运行节点

```bash
# 启动 NRCS 节点（默认 SQLite，info 日志级别）
cargo run -p nrcs-node

# 使用 PostgreSQL 启动节点
DATABASE_URL="postgres://nrcs_user:password@localhost:5432/nrcs_db" cargo run -p nrcs-node

# 使用自定义配置（config/local.toml 已配置 PostgreSQL）
cargo run -p nrcs-node -- --config config/local.toml

# 调试模式（输出所有 debug 日志，包括 SQL 查询和区块同步细节）
RUST_LOG=debug cargo run -p nrcs-node

# 通过配置文件设置日志级别
# 在 config/local.toml 中添加: log_level = "debug"
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 仅单元测试
cargo test --lib

# 运行特定模块
cargo test -p tx-engine
cargo test -p blockchain-types

# 运行 PostgreSQL 集成测试（需先启动 PostgreSQL 并创建数据库）
DATABASE_URL="postgres://nrcs_user:password@localhost:5432/nrcs_db" \
  cargo test -p orm --features postgres -- --ignored --test-threads=1
```

### 代码质量检查

```bash
# Clippy 零警告检查（CI 强制要求）
cargo clippy -- -D warnings

# 格式化代码
cargo fmt

# 完整质量检查流程
cargo fmt && cargo clippy -- -D warnings && cargo test --lib
```

## 日志系统

### 配置方式（优先级从高到低）

1. **环境变量**：`RUST_LOG=debug`（最高优先级，覆盖一切）
2. **配置文件**：`config/default.toml` 或 `config/local.toml` 中的 `log_level` 字段
3. **默认值**：`info`（生产环境合理默认）

### 日志级别策略

| 级别 | 内容 |
|------|------|
| `error!` | 致命错误（数据库连接失败、关键操作失败） |
| `warn!` | 可恢复问题（交易处理异常、节点黑名单） |
| `info!` | 关键事件（节点启动/停止、区块广播、每 5000 高度区块接受） |
| `debug!` | 详细信息（连接事件、逐笔交易详情、守护进程输出、SQL 查询） |

### 示例

```toml
# config/default.toml
log_level = "info"
```

```bash
# 开发调试：查看所有 SQL 查询和区块同步详情
RUST_LOG=debug cargo run -p nrcs-node

# 生产模式：仅输出关键事件
cargo run -p nrcs-node
```

## ORM 数据库抽象层

### 架构设计

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────┐
│  p2p / tx-  │     │  ORM Trait Layer │     │   SQLite /  │
│  engine /   │────▶│  BlockRepository │────▶│ PostgreSQL   │
│  http-api   │     │  TxRepository    │     │   Driver    │
└─────────────┘     │  ...             │     └─────────────┘
                     ├──────────────────┤
                     │  _tx() 方法族    │  ← 事务感知，传入 DbTransaction
                     │  insert_tx()    │
                     │  delete_tx()    │
                     └──────────────────┘
                              ▲
                     ┌────────┴────────┐
                     │  connection.rs  │
                     │  DbPool = AnyPool│
                     │  DbTransaction  │
                     └─────────────────┘
```

### 事务感知方法

```rust
// 标准 Repository 方法（独立执行）
block_repo.insert(&block).await?;

// 事务感知方法（在已有事务内执行）
let mut tx = pool.begin().await?;
block_repo.insert_tx(&block, &mut tx).await?;
tx_repo.insert_tx(&tx_model, &mut tx).await?;
tx.commit().await?;
```

### 42 张数据库表清单

| 类别 | 表数 | 主要表 |
|------|------|--------|
| 核心 | 5 | BLOCK, TRANSACTION, ACCOUNT, ACCOUNT_LEDGER, ACCOUNT_GUARANTEED_BALANCE |
| 资产 | 6 | ASSET, ACCOUNT_ASSET, ASSET_TRANSFER, ASSET_PROPERTY, ASSET_DIVIDEND, ASSET_DELETE |
| 订单/撮合 | 5 | ASK_ORDER, BID_ORDER, TRADE, COIN_ORDER_FXT, COIN_TRADE_FXT |
| 别名/投票 | 4 | ALIAS, ALIAS_OFFER, POLL, VOTE |
| Phasing | 7 | PHASING_POLL, PHASING_VOTE, PHASING_POLL_* (5) |
| 货币 | 5 | CURRENCY, ACCOUNT_CURRENCY, CURRENCY_TRANSFER, CURRENCY_FOUNDER, EXCHANGE_REQUEST |
| 数据存储 | 5 | TAGGED_DATA, TAGGED_DATA_TAG, TAGGED_DATA_EXTEND, TAGGED_TIMESTAMP, PRUNABLE_MESSAGE |
| 隐私 | 3 | SHUFFLING, SHUFFLING_DATA, SHUFFLING_PARTICIPANT |
| 商品 | 3 | GOODS, PURCHASE, PURCHASE_FEEDBACK |
| 其他 | 5+ | HUB, PUBLIC_KEY, CONTRACT_REFERENCE, ACCOUNT_INFO, ACCOUNT_LEASE |

## 配置文件

`config/default.toml`:

```toml
log_level = "info"

[chain]
version = "2.1.0"
is_testnet = false
max_rollback = 720

[p2p]
listen_addr = "0.0.0.0:17974"
max_connections = 20
max_inbound_connections = 100

[api]
host = "127.0.0.1"
port = 8080

[database]
url = "sqlite://nrcs.db"
max_connections = 10
```

## 与 Java 版本对应

| Java 模块 | Rust 模块 | 功能说明 |
|-----------|-----------|----------|
| nrcs-common | blockchain-types | 核心类型、常量、配置 |
| nrcs-crypto | crypto | 加密算法（Ed25519/Curve25519/SM） |
| nrcs-consensus | consensus | PoS 共识算法 |
| nrcs-transaction | tx-engine | 交易处理（65种类型） |
| nrcs-http | http-api | REST API 服务 |
| nrcs-peer | p2p | P2P 网络、区块同步（纯 ORM 访问） |
| nrcs-db | orm | 数据库 ORM（SQLite/PG 双驱动） |
| nrcs-tools | apps/cli | 命令行工具 |

## 开发状态

### 功能完成度

| 功能模块 | 状态 | 备注 |
|---------|------|------|
| 全局常量与配置 | ✅ 完成 | BLOCK_VERSION, ONE_NRCS 等常量化 |
| 区块链处理器 | ✅ 完成 | 含 Genesis 创建 |
| 交易处理器（65种类型） | ✅ 完成 | 48个Repository集成 |
| PoS 共识算法 | ✅ 完成 | 目标计算、出块选择 |
| HTTP API | ✅ 完成 | 密码过滤、请求代理 |
| CLI 工具 | ✅ 完成 | 密钥生成、签名验证 |
| 加密算法 | ✅ 完成 | Ed25519/Curve25519/SM 系列 |
| P2P 网络 | ✅ 完成 | WebSocket 通信，已迁移至 ORM |
| 区块同步（42张表） | ✅ 完成 | 完整数据同步 |
| 账户管理 | ✅ 完成 | 含保证余额 |
| ORM 数据库层 | ✅ 完成 | SQLite + PostgreSQL 双引擎生产就绪 |
| 日志系统 | ✅ 完成 | 三级配置、info 精简 |

### 代码质量指标

- 编译警告: 0 ✅
- Clippy 警告: 0 ✅
- 单元测试: 101 通过 ✅

## 测试结果

| 测试类别 | 用例数 | 通过率 |
|---------|--------|--------|
| account | 3 | 100% |
| blockchain-types | 80 | 100% |
| consensus | 5 | 100% |
| contract | 1 | 100% |
| crypto | 26 | 100% |
| http-api | 23 | 100% |
| orm (SQLite) | 18 | 100% |
| orm (PostgreSQL) | 32 | 100% |
| p2p | 12 | 100% |
| tx-engine | 101 | 100% |
| **总计** | **~300** | **100%** |

## 性能

基于初步测试：

| 指标 | Rust 实现 | Java 实现 | 提升 |
|------|----------|----------|------|
| TPS | 800+ | 500+ | +60% |
| P95 延迟 | <150ms | ~250ms | -40% |
| 内存占用 | ~800MB | ~1.2GB | -33% |
| 启动时间 | ~2s | ~8s | -75% |

## 开发指南

### 快速上手

```bash
git clone <repo-url> && cd rust-nrcs
cargo build
cargo test --lib
cargo run -p nrcs-node
```

### 代码风格

项目遵循 `.trae/rules/develop.md` 规范：

- **命名**: 变量/函数 snake_case，类型 PascalCase，常量 SCREAMING_SNAKE_CASE
- **错误处理**: 禁止 `unwrap()` / `expect()` / `panic!()`，使用 `Result`
- **异步代码**: `#[async_trait]`，所有 trait 必须 `Send + Sync`
- **依赖规则**: 严格单向依赖（上层 → 下层），禁止循环依赖
- **日志规范**: info 仅用于关键事件，debug 用于详细信息

### CI 质量门禁

```bash
cargo fmt && cargo clippy -- -D warnings && cargo test --lib
```

## 更新日志

### v2.6.0 (2026-05-05)

#### 新功能
- **PostgreSQL 测试全量通过**: 32 个 PostgreSQL 集成测试全部通过，覆盖 8 个 Repository 测试文件
- **Node PostgreSQL 分支启用**: 取消注释 PostgreSQL 分支，支持直接使用 PostgreSQL 启动节点
- **config/local.toml 更新**: 默认连接字符串切换为 PostgreSQL

#### 修复
- **PostgreSQL 标识符大小写**: 修复 pg.rs 中所有表名和列名的大小写引用问题（如 `block` → `"BLOCK"`、`timestamp` → `"TIMESTAMP"`）
- **迁移脚本标识符**: 修复 migrations/postgres/0.sql 中 TIMESTAMP、索引、外键约束的标识符引用
- **ASK_ORDER/BID_ORDER INSERT**: 补充缺失的 TRANSACTION_INDEX、TRANSACTION_HEIGHT、CREATION_HEIGHT 列
- **测试文件表名**: 修复所有测试文件中 setup_pg() 函数的表名大小写问题
- **Clippy 警告**: 修复 3 处 needless_borrows_for_generic_args 警告

### v2.5.1 (2026-05-04)

#### 新功能
- **日志系统重构**: info 日志精简（~90 处降级）、tracing 配置化（RUST_LOG/config/默认三级优先级）、SQL 查询日志降为 debug
- **ORM 事务感知方法**: 新增 `_tx()` 方法族，支持在已有事务内执行 CRUD 操作
- **数据库无关类型**: `DbPool`/`DbTransaction` 类型别名，屏蔽 SQLite/PG 差异
- **P2P SQL 迁移**: verifier.rs 中 5 处硬编码 SQL 迁移至 ORM，p2p crate 不再直接依赖 sqlx
- **常量化**: 统一使用 `BLOCK_VERSION`、`ONE_NRCS` 替换硬编码值

#### 修复
- **PRUNABLE_MESSAGE INSERT**: 修正列名（`block_transaction_height` → 正确的 11 列）
- **日志噪音**: 区块接受日志改为每 5000 高度输出一次 info

---

## 许可证

Apache-2.0
