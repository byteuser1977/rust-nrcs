# NRCS Rust 开发规范

---

## 1. 技术栈与工具

### 1.1 核心依赖

| 组件 | 选型 | 用途 |
|-----|------|------|
| Web 框架 | Axum 0.7+ | HTTP API 服务 |
| 异步运行时 | Tokio 1.x | 异步任务调度 |
| 数据库 | SQLx 0.7+ | SQLite/PostgreSQL |
| 加密签名 | ed25519-dalek 2.x | Ed25519 签名 |
| 序列化 | serde + bincode 1.x | JSON/二进制序列化 |
| 日志 | tracing 0.1+ | 结构化日志 |
| 错误处理 | thiserror 1.x | 错误类型定义 |

### 1.2 核心功能约束

项目功能约束：
- 项目基于 NRCS（Java）项目的重构项目，要求完全兼容 NRCS 项目的全部功能，包括：PEER 的区块通讯、API 接口、公式算法、加密算法、数据库结构、智能合约（这部分的开发放在第二步实现）
- NRCS 源码地址：NRCS 的参考源码在：/mnt/d/workspace/git/nrcs,在开发及测试过程中遇到的代码问题，需要参考 NRCS 项目的代码，不要自己实现
- orm 的实现要完全基于 NRCS 项目的数据库 schema 实现（migrations 目录中的 schema 是 nrcs 全量脚本），不能有任何差异
- 全量覆盖实现 orm 的数据库访问的方法
- 系统代码实现要有效的模块化，避免单一源码文件内容过多

### 1.3 开发命令

```bash
cargo build --release          # 生产构建
cargo test --lib               # 单元测试
cargo clippy -- -D warnings    # 代码检查
cargo run -p nrcs-node         # 启动节点
```

---

## 2. 模块架构

### 2.1 模块职责

| 模块 | 职责 | 依赖 |
|-----|------|------|
| blockchain-types | 核心类型定义、常量 | 无 |
| crypto | 签名、哈希、加密 | blockchain-types |
| consensus | PoS 共识算法 | blockchain-types, crypto |
| orm | 数据库模型、Repository | blockchain-types |
| account | 账户创建、余额管理 | blockchain-types, crypto, orm |
| tx-engine | 交易验证、执行、内存池 | blockchain-types, orm |
| p2p | P2P 网络、区块同步 | blockchain-types, orm, tx-engine |
| http-api | REST API 服务 | blockchain-types, orm, account, tx-engine |
| node | 节点主程序 | 所有模块 |

### 2.2 依赖规则

```
依赖方向：上层 → 下层（严格单向）

node (应用层)
  ↓
http-api / p2p / contract (服务层)
  ↓
account / tx-engine / consensus (业务层)
  ↓
orm / crypto (基础层)
  ↓
blockchain-types (核心层)

禁止：循环依赖、下层依赖上层
```

---

## 3. 代码风格

### 3.1 命名规范

| 类型 | 风格 | 示例 |
|-----|------|------|
| 变量、函数 | snake_case | `get_block_by_height` |
| 类型、Trait | PascalCase | `BlockRepository` |
| 常量 | SCREAMING_SNAKE_CASE | `MAX_BLOCK_SIZE` |

### 3.2 错误处理

```rust
#[derive(Debug, Error)]
pub enum BlockchainError {
    #[error("invalid block hash: {0}")]
    InvalidHash(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

**禁止**：`unwrap()`、`expect()`、`panic!`

### 3.3 异步代码

```rust
#[async_trait]
pub trait BlockRepository: Send + Sync {
    async fn find_by_height(&self, height: i32) -> RepositoryResult<Option<BlockModel>>;
}
```

---

## 4. 功能设计参照

### 4.1 核心类型

```rust
pub struct Block {
    pub version: i32,
    pub timestamp: Timestamp,
    pub height: Height,
    pub previous_block_id: Option<u64>,
    pub generator_id: AccountId,
    pub base_target: u64,
    pub cumulative_difficulty: Vec<u8>,
    pub generation_signature: Hash256,
    pub block_signature: Hash512,
    pub transactions: Vec<Transaction>,
}

pub struct Transaction {
    pub id: u64,
    pub type_id: TransactionType,
    pub sender_id: AccountId,
    pub recipient_id: Option<AccountId>,
    pub amount: Amount,
    pub fee: Amount,
    pub signature: Signature,
    pub full_hash: Hash256,
}
```

### 4.2 P2P 协议

```
帧结构：Version(4) + RequestID(8) + Flags(4) + Length(4) + Body(JSON)
```

请求类型：GetInfo, GetPeers, GetCumulativeDifficulty, GetMilestoneBlockIds, GetNextBlockIds, GetNextBlocks, ProcessBlock, ProcessTransactions

### 4.3 API 响应格式

```json
{
  "account": "1234567890",
  "balanceNQT": "100000000000",
  "publicKey": "abc123..."
}
```

---

## 5. 数据库设计

### 5.1 核心表

- `block`：区块数据
- `transaction`：交易数据
- `account`：账户数据

### 5.2 Repository Trait

```rust
#[async_trait]
pub trait BlockRepository: Send + Sync {
    async fn insert(&self, block: &BlockModel) -> RepositoryResult<()>;
    async fn find_by_height(&self, height: i32) -> RepositoryResult<Option<BlockModel>>;
    async fn find_latest(&self) -> RepositoryResult<Option<BlockModel>>;
}
```

---

## 6. 测试规范

| 模块 | 最低覆盖率 |
|-----|-----------|
| crypto | 100% |
| consensus | 100% |
| tx-engine | 95% |
| p2p | 90% |
| http-api | 90% |

---

## 7. 代码审查清单

- [ ] `cargo clippy` 无警告
- [ ] `cargo test` 全部通过
- [ ] 公共 API 有文档注释
- [ ] 错误处理使用 `Result`
- [ ] 无循环依赖

---

**版本**：v2.2 | **日期**：2026-04-25
