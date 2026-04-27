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

## 5.5 ORM 实现规范

### 5.5.1 数据库 Schema 规范

**必须严格遵循 NRCS Java 项目的数据库 Schema**：

- Schema 定义位置：`/mnt/d/workspace/git/nrcs/nrcs-main/resources/db/migration/`
- Rust 项目迁移脚本：`crates/orm/migrations/001_initial.sql`
- 字段名使用 `SCREAMING_SNAKE_CASE`（如 `GENERATOR_ID`, `PREVIOUS_BLOCK_ID`）
- 表名使用大写（如 `BLOCK`, `TRANSACTION`, `ACCOUNT`）

### 5.5.2 Model 层规范

**Model 结构体定义**：

```rust
#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct BlockModel {
    pub db_id: i64,
    pub id: i64,                              // 区块 ID（有符号 i64）
    pub version: i32,
    pub timestamp: i32,
    pub previous_block_id: Option<i64>,
    pub total_amount: i64,
    pub total_fee: i64,
    pub payload_length: i32,
    pub previous_block_hash: Option<Vec<u8>>,
    pub cumulative_difficulty: Vec<u8>,
    pub base_target: i64,
    pub next_block_id: Option<i64>,
    pub height: i32,
    pub generation_signature: Vec<u8>,
    pub block_signature: Vec<u8>,
    pub payload_hash: Vec<u8>,
    pub generator_id: i64,                    // 生成者 ID（有符号 i64）
}
```

**关键规则**：
1. 使用 `#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]` 映射数据库字段
2. 所有 `BIGINT` 类型映射为 `i64`（有符号）
3. `BYTEA` 类型映射为 `Vec<u8>` 或 `Option<Vec<u8>>`
4. 可空字段使用 `Option<T>`

### 5.5.3 数据类型转换规范

**u64 与 i64 转换**：

```rust
// Java NRCS 使用有符号 long 存储 ID，Rust 需要正确转换
// u64 -> i64（存储到数据库）
let signed_id = unsigned_id as i64;

// i64 -> u64（从数据库读取）
let unsigned_id = signed_id as u64;

// 示例：generator_id 转换
let generator_id: u64 = 18365787021584764528;
let db_value: i64 = generator_id as i64;  // = -80957052124787088
```

**Hash 类型转换**：

```rust
// Hash256/Hash512 -> Vec<u8>（存储）
let hash_bytes: Vec<u8> = hash.0.to_vec();

// Vec<u8> -> Hash256/Hash512（读取）
let hash = Hash256(bytes.as_slice().try_into()
    .map_err(|_| BlockchainError::InvalidHash("length mismatch".to_string()))?);
```

**Domain 与 Model 转换**：

```rust
impl BlockModel {
    pub fn from_domain(block: &Block) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: block.get_id() as i64,
            generator_id: block.get_generator_id() as i64,
            previous_block_id: block.previous_block_id
                .filter(|&id| id != 0)
                .map(|id| id as i64),
            // ... 其他字段
        })
    }

    pub fn to_domain(&self) -> Result<Block> {
        Ok(Block {
            id: Some(self.id as u64),
            generator_id: Some(self.generator_id as u64),
            previous_block_id: self.previous_block_id.map(|id| id as u64),
            // ... 其他字段
        })
    }
}
```

### 5.5.4 Repository Trait 规范

**基础 Repository Trait**：

```rust
#[async_trait]
pub trait Repository<T>: Send + Sync {
    async fn insert(&self, item: &T) -> RepositoryResult<()>;
    async fn find_by_id(&self, db_id: i64) -> RepositoryResult<Option<T>>;
    async fn update(&self, item: &T) -> RepositoryResult<()>;
    async fn delete(&self, db_id: i64) -> RepositoryResult<()>;
    async fn find_all(&self, limit: Option<i64>, offset: Option<i64>) -> RepositoryResult<Vec<T>>;
    async fn count(&self) -> RepositoryResult<i64>;
}
```

**扩展 Repository Trait**：

```rust
#[async_trait]
pub trait BlockRepository: Repository<BlockModel> {
    async fn find_by_height(&self, height: i32) -> RepositoryResult<Option<BlockModel>>;
    async fn find_by_id_column(&self, id: i64) -> RepositoryResult<Option<BlockModel>>;
    async fn find_by_hash(&self, hash: &[u8]) -> RepositoryResult<Option<BlockModel>>;
    async fn find_latest(&self) -> RepositoryResult<Option<BlockModel>>;
    async fn find_range(&self, start_height: i32, end_height: i32) -> RepositoryResult<Vec<BlockModel>>;
    async fn find_by_generator(&self, generator_id: i64) -> RepositoryResult<Vec<BlockModel>>;
    async fn get_height(&self) -> RepositoryResult<i32>;
    async fn get_block_id_at_height(&self, height: i32) -> RepositoryResult<Option<i64>>;
    async fn has_block(&self, id: i64) -> RepositoryResult<bool>;
    async fn get_ids_after(&self, block_id: i64, limit: i32) -> RepositoryResult<Vec<i64>>;
    async fn update_next_block_id(&self, previous_block_id: i64, next_block_id: i64) -> RepositoryResult<()>;
}
```

### 5.5.5 SQLite 实现规范

```rust
pub struct SqliteBlockRepository {
    pool: SqlitePool,
}

#[async_trait]
impl BlockRepository for SqliteBlockRepository {
    async fn find_by_height(&self, height: i32) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as::<_, BlockModel>(
            "SELECT * FROM block WHERE height = ?"
        )
        .bind(height)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }

    async fn find_latest(&self) -> RepositoryResult<Option<BlockModel>> {
        let record = sqlx::query_as::<_, BlockModel>(
            "SELECT * FROM block ORDER BY height DESC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;
        Ok(record)
    }
}
```

### 5.5.6 错误处理规范

```rust
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("duplicate key: {0}")]
    DuplicateKey(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("blockchain error: {0}")]
    Blockchain(#[from] blockchain_types::BlockchainError),
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;
```

### 5.5.7 JSON 序列化规范（P2P 同步）

**从 Java NRCS 接收的 JSON 格式**：

```json
{
    "blockSignature": "hex_string",
    "generationSignature": "hex_string",
    "generatorPublicKey": "hex_string",
    "payloadHash": "hex_string",
    "previousBlock": "12345678901234567890",
    "timestamp": 40674,
    "totalAmountNQT": 0,
    "totalFeeNQT": 0,
    "transactions": [],
    "version": 3
}
```

**字段名转换规则**：

| Java JSON 字段 | Rust 字段 | 转换规则 |
|---------------|----------|---------|
| `generatorPublicKey` | `generator_public_key` | camelCase → snake_case |
| `previousBlock` | `previous_block_id` | 字符串转 u64 |
| `totalAmountNQT` | `total_amount` | camelCase + 去掉 NQT 后缀 |
| `blockSignature` | `block_signature` | hex string → Hash512 |
| `generatorPublicKey` | `generator_id` | hex string → SHA256 → 前 8 字节 → u64 |

**generator_id 计算方法**（参考 Java `Account.getId(byte[] publicKey)`）：

```rust
pub fn account_id_from_public_key(public_key: &[u8; 32]) -> AccountId {
    use sha2::{Sha256, Digest};
    let hash = Sha256::digest(public_key);
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&hash[..8]);
    u64::from_le_bytes(buf)  // 小端序
}
```

### 5.5.8 Hash 类型序列化规范

**支持 hex string 和 byte array 双向反序列化**：

```rust
mod hex_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
    
    pub fn serialize<const N: usize, S>(arr: &[u8; N], serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(&hex::encode(arr))
    }
    
    pub fn deserialize<'de, const N: usize, D>(deserializer: D) -> Result<[u8; N], D::Error>
    where D: Deserializer<'de> {
        struct HexVisitor<const N: usize>;
        
        impl<'de, const N: usize> de::Visitor<'de> for HexVisitor<N> {
            type Value = [u8; N];
            
            fn visit_str<E>(self, value: &str) -> Result<[u8; N], E>
            where E: de::Error {
                let bytes = hex::decode(value).map_err(|e| de::Error::custom(format!("invalid hex: {}", e)))?;
                if bytes.len() != N {
                    return Err(de::Error::custom(format!("expected {} bytes, got {}", N, bytes.len())));
                }
                let mut arr = [0u8; N];
                arr.copy_from_slice(&bytes);
                Ok(arr)
            }
            
            fn visit_seq<A>(self, mut seq: A) -> Result<[u8; N], A::Error>
            where A: de::SeqAccess<'de> {
                let mut arr = [0u8; N];
                for i in 0..N {
                    arr[i] = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(i, &self))?;
                }
                Ok(arr)
            }
        }
        deserializer.deserialize_any(HexVisitor::<N>)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hash256(pub [u8; 32]);

impl Serialize for Hash256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        hex_serde::serialize::<32, S>(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for Hash256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        Ok(Hash256(hex_serde::deserialize::<32, D>(deserializer)?))
    }
}
```

### 5.5.9 开发注意事项

1. **严格参考 Java 实现**：所有数据库操作逻辑必须参考 NRCS Java 项目
2. **Schema 一致性**：不得修改数据库 Schema，必须与 Java 版本完全一致
3. **类型转换**：注意 u64/i64 的正确转换，Java 使用有符号 long
4. **字段映射**：使用 `#[serde(alias = "camelCase")]` 支持 JSON 字段名映射
5. **可选字段**：使用 `Option<T>` 和 `#[serde(default)]` 处理缺失字段
6. **测试覆盖**：Model 转换必须有单元测试覆盖

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

**版本**：v2.4 | **日期**：2026-04-25
