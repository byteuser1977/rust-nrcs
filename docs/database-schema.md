# NRCS 数据库设计（完整版）

## ⚠️ 重要更新

**源数据来源**：完整 schema 已在 `nrcs-sql/src/main/resources/sql-scripts-h2/0.sql` 中找到（49KB，55+ 表）。

**本次设计**：基于 H2 schema 转换为 PostgreSQL（兼容 Rust/SQLx），并确保与原 Java 版本的数据结构完全一致。

## 📊 表结构总览

根据 `0.sql`，共定义 **55+ 个表**，包括：

### 核心区块链表（必实现）

| 表名 | 说明 | 关键字段 |
|------|------|---------|
| `ACCOUNT` | 账户 | ID, BALANCE, HEIGHT, LATEST |
| `ASSET` | 资产 | ID, ACCOUNT_ID, QUANTITY, DECIMALS |
| `TRANSACTION` | 交易 | ID, TYPE, HEIGHT, TIMESTAMP |
| `BLOCK` | 区块 | ID, HEIGHT, GENERATOR_ID, PAYLOAD_HASH |
| `PUBLIC_KEY` | 公钥 | ACCOUNT_ID, PUBLIC_KEY |
| `UNCONFIRMED_TRANSACTION` | 未确认交易 | ID, TRANSACTION_BYTES |

### 资产与交易相关

| 表名 | 说明 | 关键字段 |
|------|------|---------|
| `ACCOUNT_ASSET` | 账户资产关联 | ACCOUNT_ID, ASSET_ID, QUANTITY |
| `ASSET_TRANSFER` | 资产转移 | ID, SENDER_ID, RECIPIENT_ID, ASSET_ID |
| `ASK_ORDER` | 卖单 | ID, ACCOUNT_ID, ASSET_ID, PRICE, QUANTITY |
| `BID_ORDER` | 买单 | ID, ACCOUNT_ID, ASSET_ID, PRICE |
| `SELL_OFFER` | 出售报价 | ID, ACCOUNT_ID, ASSET_ID, PRICE |
| `BUY_OFFER` | 购买报价 | ID, ACCOUNT_ID, ASSET_ID, PRICE |
| `TRADE` | 交易记录 | ASK_ORDER_ID, BID_ORDER_ID, PRICE, QUANTITY |

### 账户扩展

| 表名 | 说明 | 关键字段 |
|------|------|---------|
| `ACCOUNT_INFO` | 账户信息 | ACCOUNT_ID, NAME, DESCRIPTION |
| `ACCOUNT_LEASE` | 账户租赁 | LESSOR_ID, LESSEE_ID, HEIGHT_FROM/TO |
| `ACCOUNT_CURRENCY` | 账户货币余额 | ACCOUNT_ID, CURRENCY_ID, UNITS |
| `ACCOUNT_PROPERTY` | 账户属性 | ID, PROPERTY, VALUE |
| `ACCOUNT_LEDGER` | 账户账本 | ACCOUNT_ID, EVENT_TYPE, EVENT_ID, CHANGE |
| `ACCOUNT_CONTROL_PHASING` | 账户控制分阶段 | ACCOUNT_ID, VOTING_MODEL, QUORUM |

### 别名系统

| 表名 | 说明 | 关键字段 |
|------|------|---------|
| `ALIAS` | 别名 | ID, ACCOUNT_ID, ALIAS_NAME, ALIAS_URI |
| `ALIAS_OFFER` | 别名报价 | ID, PRICE, BUYER_ID |

### 货币与经济

| 表名 | 说明 | 关键字段 |
|------|------|---------|
| `CURRENCY` | 货币定义 | ID, NAME, CODE, DECIMALS |
| `CURRENCY_MINT` | 货币铸造 | CURRENCY_ID, ACCOUNT_ID, AMOUNT |
| `CURRENCY_SUPPLY` | 货币供应量 | CURRENCY_ID, CURRENT_SUPPLY |
| `CURRENCY_TRANSFER` | 货币转账 | ID, SENDER_ID, RECIPIENT_ID, CURRENCY_ID, UNITS |

### 治理与投票

| 表名 | 说明 | 关键字段 |
|------|------|---------|
| `PHASING_POLL` | 分阶段投票 | ID, VOTING_MODEL, QUORUM, MIN_BALANCE |
| `PHASING_VOTE` | 投票 | ACCOUNT_ID, POLL_ID, VOTE |
| `POLL` | 投票池 | ID, NAME, DESCRIPTION |
| `POLL_RESULT` | 投票结果 | POLL_ID, ACCOUNT_ID, VOTE_VALUE |

### 其他功能

| 表名 | 说明 | 关键字段 |
|------|------|---------|
| `PRUNABLE_MESSAGE` | 可修剪消息 | ID, MESSAGE, PRUNABLE |
| `TAGGED_DATA` | 标签数据 | ID, TAGGED_DATA, SIGNATURE |
| `EXCHANGE` | 交易所 | ID, NAME |
| `EXCHANGE_REQUEST` | 交易所请求 | ID, EXCHANGE_ID, ACCOUNT_ID |
| `GOODS` | 商品 | ID, NAME, DESCRIPTION, PRICE, QUANTITY |
| `PURCHASE` | 采购 | ID, GOODS_ID, BUYER_ID, QUANTITY |
| `SHUFFLING` | 混币交易 | ID, SHUFFLING_STATE |

**总计 55+ 表**。完整列表见 `crates/orm/SCHEMA_FULL.txt`（待生成）。

## 🗃 PostgreSQL 迁移脚本

已生成初始迁移：`crates/orm/migrations/001_initial.sql`

**转换规则**（H2 → PostgreSQL）：
- `BIGINT AUTO_INCREMENT` → `BIGSERIAL`
- `BOOLEAN DEFAULT FALSE` → `BOOLEAN DEFAULT FALSE`
- `INTEGER` → `INTEGER`
- `VARCHAR(2147483647)` → `VARCHAR` (无长度限制)
- `BINARY(2147483647)` → `BYTEA` (PostgreSQL 二进制)
- `BIT VARYING` → `BIT VARYING`
- `NUMERIC` → `NUMERIC` (保持)
- 索引：`CREATE INDEX` 语法兼容

**注意**：H2 的 `IF NOT EXISTS` 在 PostgreSQL 也支持，但注意依赖顺序（外键需后建）。

当前 `001_initial.sql` 已包含核心表（ACCOUNT, ASSET, TRANSACTION, BLOCK 等 20+ 表），剩余表待补充。

## 📦 ORM 模型实现

### 核心模型（优先级 1）

```rust
// crates/orm/src/models.rs

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct Account {
    pub db_id: i64,
    pub id: i64,
    pub balance: i64,
    pub unconfirmed_balance: i64,
    pub forged_balance: i64,
    pub active_lessee_id: Option<i64>,
    pub has_control_phasing: bool,
    pub height: i32,
    pub latest: bool,
}

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct Asset {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub quantity: i64,
    pub decimals: i16,
    pub initial_quantity: i64,
    pub has_control_phasing: bool,
    pub height: i32,
    pub latest: bool,
}

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct Transaction {
    pub db_id: i64,
    pub id: i64,
    pub type_: i16,
    pub account_id: Option<i64>,
    // ... 其他字段根据 schema 补充
    pub height: i32,
    pub latest: bool,
}

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct Block {
    pub db_id: i64,
    pub id: i64,
    pub version: i32,
    pub timestamp_: i32,
    pub previous_block_id: Option<i64>,
    pub total_flat_fee: i64,
    pub total_payload_length: i32,
    pub base_target: i64,
    pub generation_signature: String,
    pub generator_id: i64,
    pub generator_public_key: String,
    pub height: i32,
    pub cumulative_difficulty: i64, // 可能需要 Decimal
    pub payload_hash: String,
    pub transaction_count: i32,
    // ... 更多字段
}
```

### Repository Trait

```rust
// crates/orm/src/repository.rs
#[async_trait]
pub trait Repository<T: FromRow + Send + Sync> {
    async fn find_by_id(&self, id: i64, height: Option<i32>) -> Result<Option<T>>;
    async fn find_latest(&self, id: i64) -> Result<Option<T>>;
    async fn insert(&self, model: &T) -> Result<i64>;
    async fn update(&self, model: &T) -> Result<()>;
    async fn delete(&self, id: i64, height: i32) -> Result<()>;
    async fn list(&self, filter: Filter) -> Result<Vec<T>>;
}
```

## 🔄 数据迁移策略

### 从 H2 迁移到 PostgreSQL

**步骤**：
1. **导出 H2 数据**（Java 版本运行）：
   ```bash
   java -jar org.h2.tools.Script -url jdbc:h2:~/nrcs -user sa -script nrcs_export.sql
   ```

2. **转换 SQL**（工具待编写）：
   - 替换 `AUTO_INCREMENT` → `SERIAL`
   - 移除 H2 特有语法

3. **导入 PostgreSQL**：
   ```bash
   psql -U nr_superadmin -d nrcs_prod -f nrcs_converted.sql
   ```

4. **验证**：
   - 表记录数比对
   - 抽样数据比对

**备用方案**：过渡期同时支持双数据库（通过配置切换）。

## ⚠️ 关键注意事项

1. **时间旅行查询**：
   - 原 H2 使用 `(ID, HEIGHT DESC)` 唯一索引 + `LATEST` 标志
   - ORM 默认查询 `WHERE latest = true`
   - 历史查询：`WHERE id = ? AND height <= ?`

2. **主键策略**：
   - `DB_ID` 是物理主键（自增）
   - `ID` 是业务主键（如账户ID，可能重复在不同高度）
   - 查询业务记录时通常按 `(ID, HEIGHT)` 查询最新

3. **数据类型差异**：
   - `BIT` → 检查是否为布尔
   - `NUMERIC` → 注意精度
   - `BYTEA` 用于二进制（签名、公钥、payload）

4. **外键约束**：
   - 原 Java 可能应用层保证完整性
   - PostgreSQL 是否加外键：建议初期不加，后期再补

## 📋 实现清单（阶段2-3）

- [ ] 补充完整 `001_initial.sql`（所有 55+ 表）
- [ ] 实现 4 个核心模型（Account, Asset, Transaction, Block）
- [ ] 实现 Repository trait 基础 CRUD
- [ ] 单元测试覆盖 CRUD（≥80%）
- [ ] 补充剩余模型（根据业务优先级）
- [ ] 编写数据迁移工具（H2 → PostgreSQL）
- [ ] 验证双库数据一致性

## 📚 参考文件

- 原 H2 schema: `nrcs-sql/src/main/resources/sql-scripts-h2/0.sql`
- 当前迁移: `crates/orm/migrations/001_initial.sql`
- 待编写工具: `tools/migration/convert_h2_to_pg.py`, `tools/migration/h2-to-pg.rs`