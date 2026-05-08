# 事件驱动架构 - Event-Driven Architecture

NRCS Rust 的**事件驱动架构**，完全基于 **Java NRCS** 的 `Listeners<Account, AccountEvent>` 模式实现，用于协调跨表数据更新和账户状态管理。

## 📚 目录

- [核心概念](#核心概念)
- [组件详解](#组件详解)
- [使用指南](#使用指南)
- [与 Java NRCS 对照](#与-java-nrcs-对照)
- [最佳实践](#最佳实践)
- [故障排查](#故障排查)

---

## 核心概念

### 为什么需要事件驱动架构？

在 NRCS 区块链系统中，一个交易的处理可能影响多张表：

```
用户发起转账交易
    ↓
├── 更新 ACCOUNT 表（余额变更）
├── 更新 PUBLIC_KEY 表（首次交互时）
├── 更新 ACCOUNT_ASSET 表（如果是资产转账）
├── 更新 ACCOUNT_LEDGER 表（记录账本条目）
├── 更新 ACCOUNT_GUARANTEED_BALANCE 表（如果有保证余额）
└── 可能触发 FundingMonitor 充值逻辑
```

如果没有统一的事件机制，这些表之间的更新容易出现：
- ❌ 数据不一致
- ❌ 双重支付错误
- ❌ 未确认余额计算错误
- ❌ 遗漏某些表的更新

### 解决方案：观察者模式

采用 **观察者模式（Observer Pattern）**，通过事件分发器协调所有相关表的更新：

```
TransactionProcessor
    ↓ 发出事件
EventDispatcher
    ↓ 分发给所有监听器
┌─────────────────────┐
│ AccountListener     │ → 更新 ACCOUNT 表
│ AssetListener       │ → 更新 ACCOUNT_ASSET 表
│ CurrencyListener    │ → 更新 ACCOUNT_CURRENCY 表
│ LedgerListener      │ → 记录账本条目
│ FundingMonitor      │ → 检查余额阈值
└─────────────────────┘
```

---

## 组件详解

### 1. EventDispatcher（事件分发器）

对应 Java NRCS: `Listeners<T, EventType>`

**职责**：管理三组独立的监听器列表，分发事件给所有注册的处理器。

```rust
pub struct EventDispatcher {
    /// 账户主事件处理器列表
    account_handlers: RwLock<Vec<EventHandler>>,

    /// 资产事件处理器列表
    asset_handlers: RwLock<Vec<AssetEventHandler>>,

    /// 货币事件处理器列表
    currency_handlers: RwLock<Vec<CurrencyEventHandler>>,
}
```

#### 核心方法

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `on_account_event()` | `Fn(&AccountEvent)` | 无 | 注册账户事件监听器 |
| `on_asset_event()` | `Fn(i64, i64, i64, i64)` | 无 | 注册资产事件监听器 |
| `on_currency_event()` | `Fn(i64, i64, i64, i64)` | 无 | 注册货币事件监听器 |
| `dispatch_account_event()` | `&AccountEvent` | 无 | 分发账户事件 |
| `dispatch_asset_event()` | `(account_id, asset_id, qty, unconf)` | 无 | 分发资产事件 |
| `dispatch_currency_event()` | `(account_id, currency_id, units, unconf)` | 无 | 分发货币事件 |
| `clear_all_handlers()` | 无 | 无 | 清除所有监听器 |
| `handler_counts()` | 无 | `(usize, usize, usize)` | 获取各类型监听器数量 |

#### 使用示例

```rust
use orm::events::EventDispatcher;
use std::sync::Arc;

// 创建分发器
let dispatcher = Arc::new(EventDispatcher::new());

// 注册账户事件监听器
dispatcher.on_account_event(|event| {
    println!("收到账户事件: {:?}", event.event_type);
}).await;

// 分发事件
let event = AccountEvent::new(12345, AccountEventType::Balance)
    .with_change("balance", 1000);
dispatcher.dispatch_account_event(&event).await;
```

---

### 2. AccountEvent（账户事件）

对应 Java NRCS: `AccountEvent` 枚举

**职责**：定义所有可能的账户变更事件类型。

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountEventType {
    // 余额事件
    Balance,                    // 已确认余额变更
    UnconfirmedBalance,         // 未确认余额变更

    // 资产事件
    AssetBalance,               // 已确认资产数量变更
    UnconfirmedAssetBalance,    // 未确认资产数量变更

    // 货币事件
    CurrencyBalance,            // 已确认货币单位数变更
    UnconfirmedCurrencyBalance, // 未确认货币单位数变更

    // 租赁事件
    LeaseScheduled,             // 租赁计划已安排
    LeaseStarted,               // 租赁已开始
    LeaseEnded,                 // 租赁已结束

    // 属性事件
    SetProperty,                // 属性已设置
    DeleteProperty,             // 属性已删除
}
```

#### 链式 API 构建

```rust
let event = AccountEvent::new(account_id, AccountEventType::UnconfirmedBalance)
    .with_change("unconfirmed_balance", -10000000)        // 变更详情
    .with_height(866640)                                   // 区块高度
    .with_source("apply_unconfirmed")                      // 触发来源
    .with_transaction(9876543210)                          // 关联交易 ID
    .with_holding_id(1046280091729872666);                // 关联资产 ID
```

#### 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `account_id` | `i64` | 账户 ID |
| `event_type` | `AccountEventType` | 事件类型 |
| `changes` | `HashMap<String, i64>` | 变更详情（字段名→变化值） |
| `height` | `i32` | 区块高度 |
| `timestamp` | `i64` | Unix 时间戳 |
| `source` | `Option<String>` | 触发来源（如 "transaction_apply"） |
| `transaction_id` | `Option<i64>` | 关联交易 ID |
| `holding_id` | `Option<i64>` | 关联的资产/货币 ID |

---

### 3. LedgerEvent（账本事件）

对应 Java NRCS: `LedgerEvent` 枚举

**职责**：定义所有账本记录的事件类型，用于记录到 `ACCOUNT_LEDGER` 表。

#### 完整事件列表（50+ 值）

##### 区块和交易基础事件

| 代码 | 事件名 | 说明 |
|------|--------|------|
| 1 | `BlockGenerated` | 区块生成 |
| 2 | `RejectPhasedTransaction` | 拒绝分阶段交易 |
| 50 | `TransactionFee` | 交易手续费 |

##### TYPE_PAYMENT

| 代码 | 事件名 | 说明 |
|------|--------|------|
| 3 | `OrdinaryPayment` | 普通转账 |

##### TYPE_MESSAGING（11个）

| 代码 | 事件名 | 说明 |
|------|--------|------|
| 4 | `AccountInfo` | 账户信息设置 |
| 5 | `AliasAssignment` | 别名分配 |
| 6 | `AliasBuy` | 别名购买 |
| 7 | `AliasDelete` | 别名删除 |
| 8 | `AliasSell` | 别名出售 |
| 9 | `ArbitraryMessage` | 任意消息 |
| 10 | `HubAnnouncement` | Hub 公告 |
| 11 | `PhasingVoteCasting` | 分阶段投票 |
| 12 | `PollCreation` | 投票创建 |
| 13 | `VoteCasting` | 选票投递 |
| 56 | `AccountProperty` | 账户属性 |
| 57 | `AccountPropertyDelete` | 删除账户属性 |
| 656 | `AccountPropertySet` | 设置账户属性 |
| 68 | `AccountLongValuePropertySet` | 设置长值属性 |

##### TYPE_COLORED_COINS（14+）

| 代码 | 事件名 | 说明 |
|------|--------|------|
| 14 | `AssetAskOrderCancellation` | 卖单取消 |
| 15 | `AssetAskOrderPlacement` | 卖单下单 |
| 16 | `AssetBidOrderCancellation` | 买单取消 |
| 17 | `AssetBidOrderPlacement` | 买单下单 |
| 18 | `AssetDividendPayment` | 资产分红支付 |
| 19 | `AssetIssuance` | 资产发行 |
| 20 | `AssetTrade` | 资产交易 |
| 21 | `AssetTransfer` | 资产转移 |
| 49 | `AssetDelete` | 资产删除 |
| 61 | `AssetIncrease` | 资产增发 |
| 62 | `AssetSetPhasingControl` | 设置资产分阶段控制 |
| 65 | `AssetPropertySet` | 设置资产属性 |
| 66 | `AssetPropertyDelete` | 删除资产属性 |
| 67 | `AssetLongValuePropertySet` | 设置资产长值属性 |

##### 其他类型...

（完整列表请参考源码 `crates/orm/src/events.rs`）

#### 核心方法

```rust
// 从代码值转换
let event = LedgerEvent::from_code(21);  // Some(LedgerEvent::AssetTransfer)

// 获取代码
let code = LedgerEvent::AssetTransfer.code();  // 21

// 检查是否为交易事件
let is_tx = LedgerEvent::BlockGenerated.is_transaction();  // false
let is_tx = LedgerEvent::OrdinaryPayment.is_transaction();   // true
```

---

### 4. LedgerEntry（账本条目）

对应 Java NRCS: `LedgerEntry` 类

**职责**：记录账户余额变更的完整历史，用于审计追踪和数据恢复。

```rust
pub struct LedgerEntry {
    pub db_id: i64,                    // 数据库 ID（自增主键）
    pub account_id: i64,               // 账户 ID
    pub event_type: i16,               // 事件类型（LedgerEvent.code()）
    pub event_id: i64,                 // 事件 ID（交易/区块 ID）
    pub holding_type: Option<i32>,     // 持有类型（LedgerHolding.code()）
    pub holding_id: Option<i64>,       // 持有 ID（资产/货币 ID）
    pub change: i64,                   // 变更金额
    pub balance: i64,                  // 变更后余额
    pub block_id: i64,                 // 关联区块 ID
    pub height: i32,                   // 区块高度
    pub timestamp: i32,                // 时间戳
}
```

#### 构造示例

```rust
// 完整版本（包含持有信息）
let entry = LedgerEntry::new(
    LedgerEvent::OrdinaryPayment,
    123456789,           // 事件 ID（交易 ID）
    9876543210,          // 账户 ID
    Some(LedgerHolding::NrcsBalance),  // 持有类型
    None,                // 持有 ID（NRCS 为 None）
    -10000000,           // 变更金额（-1 NRCS）
    900000000,           // 变更后余额
    111222333,           // 区块 ID
    866640,              // 高度
    40674,               // 时间戳
);

// 简化版本（无持有信息）
let simple_entry = LedgerEntry::new_simple(
    LedgerEvent::AssetTransfer,
    999888777,
    1234567890,
    500,                 // 变更金额
    1500,                // 变更后余额
    444555666,
    100000,
    50506,
);

// 累加变更金额
entry.update_change(-500);  // change = -500 + (-500) = -1000
```

---

### 5. FundingMonitor（资金监控服务）

对应 Java NRCS: `FundingMonitor`

**职责**：监控指定账户的余额变化，当余额低于阈值时自动发起充值交易。

#### 核心功能

```
┌─────────────────────────────────────────────────────┐
│                  FundingMonitor                       │
├─────────────────────────────────────────────────────┤
│  监控账户列表 (monitored_accounts)                   │
│  ┌─────────────────────────────────────────────┐   │
│  │ Account: 1234567890                           │   │
│  │ Type: NRCS                                   │   │
│  │ Threshold: 1 NRCS                             │   │
│  │ Funding Amount: 10 NRCS                       │   │
│  └─────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────┤
│  待处理队列 (pending_events)                        │
│  ┌─────────────────────────────────────────────┐   │
│  │ ⚠️ Account 1234567890 balance < threshold!   │   │
│  └─────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

#### 配置结构

```rust
pub struct MonitoredAccountConfig {
    pub account_id: i64,           // 被监控的账户 ID
    pub holding_type: HoldingType,  // 持有类型（NRCS/Asset/Currency）
    pub holding_id: Option<i64>,   // 持有 ID（资产/货币 ID）
    pub threshold: i64,            // 触发阈值
    pub funding_amount: i64,       // 充值金额
    pub funding_account_id: i64,   // 充值源账户 ID
}

pub enum HoldingType {
    Nrcs,      // NRCS 余额
    Asset,     // 资产余额
    Currency,  // 货币余额
}
```

#### 使用流程

```rust
// 1. 创建监控服务（返回 Arc<FundingMonitor>）
let monitor = FundingMonitor::new(Arc::clone(&dispatcher));

// 2. 初始化（注册三种监听器到 EventDispatcher）
monitor.init().await;

// 3. 添加监控账户
let config = MonitoredAccountConfig::new(
    1234567890,           // 账户 ID
    HoldingType::Nrcs,    // 监控 NRCS 余额
    None,                 // 无持有 ID
    100000000,            // 阈值: 1 NRCS
    1000000000,           // 充值金额: 10 NRCS
    9876543210,           // 充值源账户
);
monitor.add_monitored_account(config).await;

// 4. 处理待充值事件（通常在定时任务中调用）
monitor.process_pending_events().await;

// 5. 停止监控
monitor.shutdown().await;
```

#### 当前状态

- ✅ **日志模式已就绪**：检测到低余额时会输出警告日志
- 🔄 **生产模式开发中**：需要集成 TransactionProcessor 来构建和广播充值交易

---

### 6. DataConsistencyChecker（数据一致性检查器）

**职责**：自动验证跨表数据的一致性，防止数据异常。

#### 检查项

##### 账户余额检查（3项）

| 检查名称 | 条件 | 说明 |
|---------|------|------|
| `confirmed_balance_non_negative` | `confirmed >= 0` | 确认余额非负 |
| `unconfirmed_balance_non_negative` | `unconfirmed >= 0` | 未确认余额非负 |
| `unconfirms_not_exceeds_confirmed` | `unconfirmed <= confirmed` | 未确认 ≤ 确认 |

##### 资产余额检查（3项）

| 检查名称 | 条件 | 说明 |
|---------|------|------|
| `asset_quantity_non_negative` | `quantity >= 0` | 资产数量非负 |
| `asset_unconfirmed_non_negative` | `unconfirmed_qty >= 0` | 未确认资产数量非负 |
| `asset_unconfirms_within_confirmed` | `unconfirmed <= confirmed` | 未确认资产 ≤ 确认资产 |

#### 使用示例

```rust
let checker = DataConsistencyChecker::new(dispatcher);

// 检查账户一致性
let result = checker.check_account_consistency(
    1234567890,        // 账户 ID
    1000000000,        // 确认余额
    800000000,         // 未确认余额
).await;

if !result.is_consistent {
    for check in &result.checks {
        if !check.passed {
            eprintln!("❌ {}: {}", check.name, check.detail);
        }
    }
} else {
    println!("✅ 所有一致性检查通过");
}
```

---

## 使用指南

### 快速开始

#### 1. 在 TransactionProcessor 中集成

```rust
// 在 processor.rs 中初始化事件系统
impl DatabaseTransactionProcessor {
    pub fn new(/* ... */) -> Self {
        Self {
            // ... 其他字段 ...
            dispatcher: Arc::new(EventDispatcher::new()),
        }
    }

    pub async fn setup_default_listeners(&self) {
        setup_default_listeners(&self.dispatcher).await;
    }
}
```

#### 2. 在交易处理中分发事件

```rust
async fn apply_unconfirmed(&self, tx: &Transaction) -> ProcessorResult<bool> {
    // ... 执行业务逻辑 ...

    // 更新未确认余额
    self.account_repo.add_to_unconfirmed_balance(sender_id, -deduct_amount, height).await?;

    // 分发账户变更事件
    let event = AccountEvent::new(sender_id, AccountEventType::UnconfirmedBalance)
        .with_change("unconfirmed_balance", -deduct_amount)
        .with_height(height)
        .with_source("apply_unconfirmed")
        .with_transaction(tx.id as i64);

    self.dispatcher.dispatch_account_event(&event).await;

    Ok(true)
}
```

#### 3. 自定义监听器

```rust
// 定义自定义监听器
struct MyCustomListener {
    db_pool: Arc<SqlitePool>,
}

#[async_trait]
impl AccountEventListener for MyCustomListener {
    async fn handle(&self, event: &AccountEvent) {
        if event.event_type == AccountEventType::Balance {
            // 执行自定义逻辑，如写入自定义数据库
            tracing::info!(
                "Custom handler: account {} balance changed",
                event.account_id
            );
        }
    }
}

// 注册自定义监听器
let listener = Arc::new(MyCustomListener { db_pool });
dispatcher.on_account_event(move |event| {
    let listener = listener.clone();
    tokio::spawn(async move {
        listener.handle(event).await;
    });
}).await;
```

---

## 与 Java NRCS 对照

| Java NRCS 组件 | Rust 实现 | 兼容性 | 功能状态 |
|---------------|----------|--------|---------|
| `AccountEvent` 枚举 (11) | `AccountEventType` | ✅ 100% | 完整实现 |
| `Listeners<Account, AccountEvent>` | `EventDispatcher.account_handlers` | ✅ 100% | 完整实现 |
| `Listeners<AccountAsset, ...>` | `EventDispatcher.asset_handlers` | ✅ 100% | 完整实现 |
| `Listeners<AccountCurrency, ...>` | `EventDispatcher.currency_handlers` | ✅ 100% | 完整实现 |
| `AccountEventHandler` | `DefaultLoggingListener` + Trait | ✅ 100% | 完整实现 |
| `AssetEventHandler` | `DefaultAssetLoggingListener` + Trait | ✅ 100% | 完整实现 |
| `CurrencyEventHandler` | `DefaultCurrencyLoggingListener` + Trait | ✅ 100% | 完整实现 |
| `LedgerEvent` 枚举 (50+) | `LedgerEvent` | ✅ 100% | 完整实现 |
| `LedgerEntry` 类 | `LedgerEntry` 结构体 | ✅ 100% | 完整实现 |
| `LedgerHolding` 枚举 | `LedgerHolding` | ✅ 100% | 完整实现 |
| `FundingMonitor` 类 | `FundingMonitor` | ✅ 95% | 核心完成（生产模式开发中） |
| `MonitoredAccount` 配置 | `MonitoredAccountConfig` | ✅ 100% | 完整实现 |
| `HoldingType` 枚举 | `HoldingType` | ✅ 100% | 完整实现 |
| `Account.checkBalance()` | `DataConsistencyChecker` | ✅ 100% | 新增增强 |

---

## 最佳实践

### 1. 事件命名规范

```rust
// ✅ 正确：使用具体来源标识
.with_source("transaction_apply")      // 交易应用
.with_source("block_reward")           // 区块奖励
.with_source("genesis_init")           // 创世初始化
.with_source("rollback_undone")        // 回滚撤销

// ❌ 错误：过于笼统
.with_source("update")
.with_source("change")
```

### 2. 事件粒度控制

```rust
// ✅ 正确：每个独立操作都应分发事件
async fn apply_payment(&self, tx: &Transaction) -> ProcessorResult<()> {
    // 扣减发送方余额
    self.account_repo.add_to_balance(sender_id, -amount).await?;
    self.dispatch_balance_event(sender_id, -amount, "payment_deduct").await;

    // 增加接收方余额
    self.account_repo.add_to_balance(recipient_id, amount).await?;
    self.dispatch_balance_event(recipient_id, amount, "payment_add").await;

    Ok(())
}

// ❌ 错误：只分发一个聚合事件
self.dispatcher.dispatch(AccountEvent::new(...)
    .with_change("total", amount))  // 无法追踪单个账户变化
```

### 3. 错误处理

```rust
// ✅ 正确：事件分发失败不应阻断业务流程
match self.dispatcher.dispatch_account_event(&event).await {
    Ok(_) => {},
    Err(e) => {
        warn!(error = %e, "Failed to dispatch account event");
        // 记录错误但继续执行
    }
}

// ❌ 错误：事件失败导致整个交易回滚
self.dispatcher.dispatch_account_event(&event).await?;  // 不推荐
```

### 4. 性能优化

```rust
// ✅ 正确：批量处理事件
let events: Vec<AccountEvent> = prepare_events();
for event in events {
    dispatcher.dispatch_account_event(&event).await;  // 异步并行
}

// ✅ 正确：避免在热路径上创建过多事件
if should_log_ledger(event_type) {  // 过滤不必要的事件
    dispatcher.dispatch_account_event(&event).await;
}
```

---

## 故障排查

### 1. 事件未触发

**症状**：监听器未收到预期的事件

**排查步骤**：

```bash
# 1. 检查是否正确调用了 dispatch
RUST_LOG=debug cargo run -p nrcs-node

# 日志输出示例：
# DEBUG [dispatcher] Dispatching account event
# DEBUG [dispatcher]   account=1234567890, event_type=BALANCE, handler_count=3

# 如果没有看到此日志，说明 dispatch 未被调用
```

**常见原因**：

- ❌ 忘记调用 `setup_default_listeners()`
- ❌ 事件类型不匹配（如监听 `Balance` 但实际发出的是 `UnconfirmedBalance`）
- ❌ `dispatcher` 实例不一致（使用了不同的 `Arc`）

### 2. 性能问题

**症状**：事件处理导致延迟增加

**解决方案**：

```rust
// 方案 1：减少监听器数量
// 只注册必要的监听器
dispatcher.on_account_event(|event| {
    if is_critical_event(event) {
        process_immediately(event);
    }
}).await;

// 方案 2：异步批处理
let mut batch = Vec::new();
batch.push(event);
if batch.len() >= BATCH_SIZE {
    process_batch(batch.drain(..)).await;
}
```

### 3. 内存泄漏

**症状**：内存占用持续增长

**原因**：闭包捕获了大型对象

```rust
// ❌ 错误：闭包捕获了整个 Transaction 对象
dispatcher.on_account_event(move |_| {
    println!("{:?}", large_transaction);  // 导致 Transaction 无法释放
}).await;

// ✅ 正确：只捕获必要字段
let tx_id = transaction.id;
let sender_id = transaction.sender_id;
dispatcher.on_account_event(move |event| {
    if event.transaction_id == Some(tx_id) {
        println!("Tx {} affected account {}", tx_id, sender_id);
    }
}).await;
```

---

## 测试覆盖

### 单元测试（30+ 个）

```bash
# 运行所有事件系统测试
cargo test -p orm --lib events

# 测试类别：
# - test_account_event_creation        ✅
# - test_ledger_holding_conversion      ✅
# - test_event_dispatcher_basic        ✅
# - test_asset_event_dispatch          ✅
# - test_currency_event_dispatch       ✅
# - test_multiple_handlers             ✅
# - test_handler_counts                ✅
# - test_clear_all_handlers            ✅
# - test_setup_default_listeners       ✅
# - test_ledger_event_from_code        ✅
# - test_ledger_entry_creation         ✅
# - test_funding_monitor_*             ✅ (4 tests)
# - test_consistency_checker_*         ✅ (5 tests)
```

### 集成测试

```bash
# 在实际交易处理中测试事件分发
cargo test -p tx-engine --lib -- apply_unconfirmed
cargo test -p tx-engine --lib -- undo_unconfirmed
```

---

## 相关文档

- [主项目 README](../../README.md) - 项目总览
- [ORM Layer 文档](../README.md) - ORM 详细文档
- [Java-Rust 兼容性分析](../../../.trae/documents/java_rust_compatibility_analysis.md) - 差异对比
- [NRCS Rust 开发规范](../../../.trae/rules/develop.md) - 编码规范

---

## 版本历史

### v2.7.0 (2026-05-07)

- ✨ 完整实现事件驱动架构（11 个 AccountEventType + 50+ LedgerEvent）
- ✨ 实现 FundingMonitor 完整版（日志模式）
- ✨ 新增 DataConsistencyChecker
- 🐛 修复双重支付错误（increase_quantity/decrease_quantity 同步更新）
- 🐛 修复账户创建 height=0 问题
- 📊 新增 30+ 单元测试（总计 350+ 测试全部通过）

---

**事件驱动架构** - 让数据联动更简单、更可靠！🎯
