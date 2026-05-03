# Rust-NRCS 测试覆盖实现计划

基于 NRCS (Java) 项目 `nrcs-test` 模块的测试实现，为 rust-nrcs 新增对应的测试覆盖内容。

---

## 一、现状分析

### Java NRCS 测试覆盖范围（nrcs-test 模块）

| 功能域 | 测试文件数 | 覆盖的 API |
|--------|-----------|-----------|
| 账户信息与属性 | 7 | setAccountInfo, setAccountProperty, setAccountLongValueProperty, setPhasingOnlyControl, approveTransaction |
| 资产发行/转账/属性/分红 | 4 | issueAsset, transferAsset, setAssetProperty, dividendPayment, placeBidOrder, placeAskOrder |
| 交易打包器 | 2 | startBundler, stopBundler, bundleTransactions |
| 加密算法 | 1 | SM2 加密/解密 |
| 货币发行/兑换/铸造/储备/删除 | 6 | issueCurrency, currencyBuy, currencySell, publishExchangeOffer, currencyMint, currencyReserveIncrease, deleteCurrency |
| 区块生成/锻造 | 2 | Generator.getHit, Generator.startForging, generateBlock |
| 消息发送 | 1 | sendMessage, readMessage |
| NRCS 转账/余额租赁 | 2 | sendMoney, leaseBalance, signTransaction, broadcastTransaction |
| 投票系统 | 3 | createPoll, castVote, getPollResult, getPolls |

### Rust-NRCS 当前测试状态

| 模块 | 规范要求覆盖率 | 当前状态 |
|------|---------------|---------|
| crypto | 100% | 中等 - 签名验证充分，SM2/SM3/SM4/AES-GCM 独立测试少 |
| consensus | 100% | 偏低 - 缺少 generation_signature、target 计算独立测试 |
| tx-engine | 95% | 偏低 - 缺少 processor/mempool/ledger/phasing/shuffler/bundler/monitor 测试 |
| p2p | 90% | 中等 - Peer/BlacklistManager 充分，缺少 WebSocket/Handler/守护进程测试 |
| http-api | 90% | 偏低 - 仅覆盖基础路由，缺少 NRCS 兼容 API 测试 |
| orm | - | 偏低 - 缺少 Repository CRUD 集成测试 |
| account | - | 缺失 - 完全没有测试 |
| node | - | 缺失 - 无测试 |
| contract | - | 偏低 - 仅为框架测试 |

---

## 二、测试架构设计

### 2.1 测试分层

```
┌─────────────────────────────────────────────────┐
│  Layer 4: 端到端集成测试 (tests/integration/)     │  ← 需要完整节点运行
├─────────────────────────────────────────────────┤
│  Layer 3: API 集成测试 (crates/http-api/tests/)  │  ← Mock + Axum oneshot
├─────────────────────────────────────────────────┤
│  Layer 2: 业务逻辑测试 (各 crate/tests/)          │  ← 内存数据库 + 真实业务逻辑
├─────────────────────────────────────────────────┤
│  Layer 1: 单元测试 (各 crate/src/ #[cfg(test)])   │  ← 纯函数/逻辑验证
└─────────────────────────────────────────────────┘
```

### 2.2 测试基础设施

参照 Java NRCS 的 `AbstractBlockchainTest` → `BlockchainTest` → 具体测试类层次，在 Rust 中创建：

```
crates/test-utils/           ← 新增共享测试工具 crate
├── src/
│   ├── lib.rs               ← 导出所有公共测试工具
│   ├── fixtures.rs          ← 测试账户、常量、配置
│   ├── db_helper.rs         ← 内存数据库创建、Schema 初始化
│   ├── api_client.rs        ← HTTP API 测试客户端
│   ├── assertions.rs        ← 自定义断言宏和辅助函数
│   └── mock_helpers.rs      ← Mock 实现工厂
```

### 2.3 测试账户体系

参照 Java NRCS 的 6 个标准测试账户（FORGY/ALICE/BOB/CHUCK/DAVE/RIKER），在 Rust 中定义：

```rust
pub struct TestAccount {
    pub name: &'static str,
    pub passphrase: &'static str,
    pub secret_key: [u8; 32],
    pub public_key: [u8; 32],
    pub account_id: u64,
    pub rs_address: String,
}

pub const FORGY: TestAccount = TestAccount { ... };  // 出块者
pub const ALICE: TestAccount = TestAccount { ... };   // 发送方
pub const BOB: TestAccount = TestAccount { ... };     // 接收方
pub const CHUCK: TestAccount = TestAccount { ... };   // 第三方
pub const DAVE: TestAccount = TestAccount { ... };    // 第四方
pub const RIKER: TestAccount = TestAccount { ... };   // 资金提供者
```

---

## 三、分阶段实施计划

### 阶段 1：测试基础设施搭建（优先级：高）

#### 任务 1.1：创建 test-utils crate

- 创建 `crates/test-utils/` crate
- 在 workspace Cargo.toml 中添加成员
- 实现核心模块：
  - `fixtures.rs`：6 个标准测试账户定义、NRCS 常量（ONE_NRCS 等）
  - `db_helper.rs`：`setup_in_memory_db()` 函数，创建 SQLite 内存数据库 + 全量 Schema
  - `assertions.rs`：`assert_json_success()`、`assert_json_error()`、`assert_balance_diff()` 等断言宏
  - `lib.rs`：统一导出

#### 任务 1.2：实现内存数据库辅助

- 参照 `orm/src/genesis.rs` 中的 `setup_repos()` 模式
- 创建 `setup_full_db()` 函数，创建所有表（使用内联 DDL，与 migration 一致）
- 返回所有 Repository 实例的元组
- 支持创世区块初始化

#### 任务 1.3：实现 API 测试客户端

- 参照 Java NRCS 的 `APICall.Builder` 模式
- 实现 `NrcsApiClient` 结构体，支持：
  - 构建请求参数（HashMap<String, String>）
  - 发送 GET/POST 请求
  - 解析 JSON 响应
  - 错误码断言
- 支持两种模式：
  - 进程内模式：直接调用 Axum Router（`tower::ServiceExt::oneshot`）
  - 远程模式：HTTP 请求到运行中的节点

### 阶段 2：核心模块单元测试补充（优先级：高）

#### 任务 2.1：crypto 模块测试补充

**目标：达到 100% 覆盖率**

新增测试文件：
- `crates/crypto/tests/sm2_tests.rs` — SM2 签名/验签/加密/解密测试
- `crates/crypto/tests/sm3_tests.rs` — SM3 哈希算法测试
- `crates/crypto/tests/sm4_gcm_tests.rs` — SM4-GCM 加密/解密测试
- `crates/crypto/tests/aes_gcm_tests.rs` — AES-GCM 加密/解密测试
- `crates/crypto/tests/reed_solomon_tests.rs` — Reed-Solomon 编码/解码完整测试
- `crates/crypto/tests/passphrase_tests.rs` — 助记词生成和验证测试

参照 Java `TestSM2.java` 的测试模式，验证 SM2 加密/解密功能。

#### 任务 2.2：consensus 模块测试补充

**目标：达到 100% 覆盖率**

新增测试文件：
- `crates/consensus/tests/generation_signature_tests.rs` — 生成签名计算/验证测试
- `crates/consensus/tests/target_tests.rs` — 目标值计算测试
- `crates/consensus/tests/selection_tests.rs` — 出块者选择算法完整测试
- `crates/consensus/tests/block_gen_tests.rs` — 区块生成模板测试
- `crates/consensus/tests/generator_tests.rs` — 锻造计算测试（参照 Java `GeneratorTest.java`）

参照 Java `GeneratorTest.java`：
- 验证 hit 值和 hitTime 计算
- 验证在 deadline 之前不能锻造
- 验证在 hitTime 之后可以锻造
- 验证生成签名的正确性

#### 任务 2.3：account 模块测试补充

**目标：从 0 到基本覆盖**

新增测试文件：
- `crates/account/tests/account_manager_tests.rs` — AccountManager 核心功能测试
- `crates/account/tests/address_generator_tests.rs` — 地址生成测试
- `crates/account/tests/crypto_tests.rs` — 密钥对生成测试

测试内容：
- 创建账户（初始余额/无余额）
- 注册账户（公钥）
- 获取余额
- 转账（正常/余额不足）
- 信用/借记操作
- Nonce 递增
- 公钥查询
- RS 地址生成

### 阶段 3：ORM Repository 集成测试（优先级：高）

#### 任务 3.1：核心 Repository CRUD 测试

新增测试文件：
- `crates/orm/tests/block_repository_tests.rs` — BlockRepository 全量方法测试
- `crates/orm/tests/transaction_repository_tests.rs` — TransactionRepository 全量方法测试
- `crates/orm/tests/account_repository_tests.rs` — AccountRepository 全量方法测试
- `crates/orm/tests/asset_repository_tests.rs` — AssetRepository 全量方法测试
- `crates/orm/tests/alias_repository_tests.rs` — AliasRepository 全量方法测试

每个 Repository 测试覆盖：
- `insert()` — 插入记录
- `find_by_id()` — 按 DB_ID 查询
- `find_by_id_column()` — 按业务 ID 查询
- `find_all()` — 分页查询
- `count()` — 计数
- `update()` — 更新记录
- `delete()` — 删除记录
- 扩展方法（如 BlockRepository 的 `find_by_height`、`find_latest`、`find_range` 等）

#### 任务 3.2：扩展 Repository 测试

- `crates/orm/tests/currency_repository_tests.rs`
- `crates/orm/tests/order_repository_tests.rs`
- `crates/orm/tests/poll_repository_tests.rs`
- `crates/orm/tests/peer_repository_tests.rs`
- `crates/orm/tests/phasing_repository_tests.rs`
- `crates/orm/tests/goods_repository_tests.rs`
- `crates/orm/tests/tagged_data_repository_tests.rs`
- `crates/orm/tests/shuffling_repository_tests.rs`

### 阶段 4：交易引擎业务逻辑测试（优先级：高）

#### 任务 4.1：转账交易测试

新增测试文件：`crates/tx-engine/tests/send_money_tests.rs`

参照 Java `SendMoneyTest.java`：
- `test_send_money()` — 正常转账，验证发送者/接收者/锻造者余额变化
- `test_send_too_much_money()` — 余额不足应失败
- `test_send_and_return()` — 双向转账，验证双方余额净变化
- `test_sign_and_broadcast_bytes()` — 离线签名 + 广播（字节格式）
- `test_sign_and_broadcast_json()` — 离线签名 + 广播（JSON 格式）

#### 任务 4.2：消息交易测试

新增测试文件：`crates/tx-engine/tests/send_message_tests.rs`

参照 Java `SendMessageTest.java`：
- `test_send_message()` — 明文消息发送与读取
- `test_send_encrypted_message()` — 服务端加密消息
- `test_send_client_encrypted_message()` — 客户端加密消息
- `test_send_encrypted_message_to_self()` — 给自己加密的消息
- `test_public_key_announcement()` — 公钥公告创建新账户
- `test_send_from_not_existing_account()` — 不存在账户发送消息

#### 任务 4.3：资产交易测试

新增测试文件：`crates/tx-engine/tests/asset_tests.rs`

参照 Java `AssetTest.java` 和 `AssetExchangeTest.java`：
- `test_issue_asset()` — 发行资产
- `test_transfer_asset()` — 转移资产
- `test_nrcs_dividend()` — NRCS 货币分红
- `test_asset_dividend()` — 资产分红
- `test_currency_dividend()` — 货币分红

新增测试文件：`crates/tx-engine/tests/asset_property_tests.rs`

参照 Java `AssetPropertyTest.java` 和 `AssetLongValuePropertyTest.java`：
- `test_set_and_get_property()` — 设置和获取资产属性
- `test_delete_property()` — 删除属性
- `test_other_account_delete_rejected()` — 其他账户删除被拒绝
- `test_empty_value_property()` — 空值属性
- `test_multi_account_same_property()` — 多账户设置同名属性

#### 任务 4.4：货币交易测试

新增测试文件：`crates/tx-engine/tests/currency_tests.rs`

参照 Java `TestCurrencyIssuance.java`、`TestCurrencyExchange.java`、`TestCurrencyMint.java`、`TestCurrencyReserveAndClaim.java`、`TestDeleteCurrency.java`：
- `test_issue_currency()` — 发行货币
- `test_issue_currency_no_broadcast()` — 不广播发行
- `test_issue_multiple_currencies()` — 批量发行
- `test_currency_buy()` — 买入货币
- `test_currency_sell()` — 卖出货币
- `test_currency_mint()` — 铸造货币
- `test_currency_reserve_increase()` — 储备增加
- `test_currency_reserve_claim()` — 众筹分配
- `test_delete_currency()` — 删除货币

新增测试文件：`crates/tx-engine/tests/mint_calculation_tests.rs`

参照 Java `TestMintCalculations.java`：
- `test_target_calculation()` — 目标值计算
- `test_hash_calculation()` — 哈希计算（Keccak25）
- `test_sha256_hash()` — SHA256 哈希验证
- `test_sha3_hash()` — SHA3 哈希验证
- `test_scrypt_hash()` — Scrypt 哈希验证

#### 任务 4.5：Phasing 交易测试

新增测试文件：`crates/tx-engine/tests/phasing_tests.rs`

参照 Java `PhasingOnlyTest.java` 和 `CompositePhasingOnlyTest.java`：
- `test_set_and_get()` — 设置和获取 Phasing 控制
- `test_account_voting()` — 账户投票模式
- `test_extra_restrictions()` — 额外限制（maxFees/minDuration/maxDuration）
- `test_rejecting_pending_transaction()` — 拒绝待处理交易
- `test_balance_voting()` — 余额投票模式
- `test_asset_voting()` — 资产投票模式
- `test_currency_voting()` — 货币投票模式
- `test_property_voting()` — 属性投票模式
- `test_zero_max_fees()` — 零最大费用场景

#### 任务 4.6：Bundler 测试

新增测试文件：`crates/tx-engine/tests/bundler_tests.rs`

参照 Java `AssetBundlerTest.java`：
- `test_asset_transfer_bundler()` — 资产转账打包器
- `test_asset_bid_bundler()` — 资产订单打包器
- `test_asset_bundler_with_quota()` — 带配额的打包器

#### 任务 4.7：投票系统测试

新增测试文件：`crates/tx-engine/tests/voting_tests.rs`

参照 Java `TestCreatePoll.java`、`TestCastVote.java`、`TestGetPolls.java`：
- `test_create_poll()` — 创建有效投票
- `test_create_invalid_poll()` — 创建无效投票
- `test_cast_vote()` — 投票
- `test_cast_invalid_vote()` — 无效投票
- `test_get_polls()` — 查询投票列表

#### 任务 4.8：余额租赁测试

新增测试文件：`crates/tx-engine/tests/lease_tests.rs`

参照 Java `LeaseTest.java`：
- `test_lease_balance()` — 余额租赁生效和到期

#### 任务 4.9：账户属性测试

新增测试文件：`crates/tx-engine/tests/account_property_tests.rs`

参照 Java `AccountInfoTest.java`、`AccountPropertiesTest.java`、`AccountLongPropertiesTest.java`：
- `test_set_account_info()` — 设置账户信息（名称长度限制）
- `test_set_account_property()` — 设置账户属性
- `test_set_account_long_value_property()` — 设置长值属性
- `test_property_value_length_limit()` — 属性值长度限制
- `test_property_name_length_limit()` — 属性名长度限制

### 阶段 5：HTTP API 集成测试扩展（优先级：中）

#### 任务 5.1：NRCS 兼容 API 测试

新增测试文件：`crates/http-api/tests/nrcs_api_tests.rs`

覆盖 NRCS 兼容 API 端点：
- `/nrcs?requestType=sendMoney` — 转账
- `/nrcs?requestType=sendMessage` — 发送消息
- `/nrcs?requestType=getAccount` — 获取账户
- `/nrcs?requestType=getBalance` — 获取余额
- `/nrcs?requestType=getBlock` — 获取区块
- `/nrcs?requestType=getBlockchainStatus` — 区块链状态
- `/nrcs?requestType=issueAsset` — 发行资产
- `/nrcs?requestType=transferAsset` — 转移资产
- `/nrcs?requestType=issueCurrency` — 发行货币
- `/nrcs?requestType=createPoll` — 创建投票
- `/nrcs?requestType=castVote` — 投票
- `/nrcs?requestType=leaseBalance` — 余额租赁

#### 任务 5.2：RESTful v1 API 测试

新增测试文件：`crates/http-api/tests/v1_api_tests.rs`

覆盖 RESTful v1 API 端点：
- 账户相关（创建/查询/余额）
- 区块相关（查询/列表）
- 交易相关（创建/查询/签名/广播）
- 资产相关
- 货币相关
- 投票相关

#### 任务 5.3：API 错误处理测试

新增测试文件：`crates/http-api/tests/api_error_tests.rs`

- 无效参数错误
- 余额不足错误
- 账户不存在错误
- 重复交易错误
- 签名验证失败错误

### 阶段 6：P2P 网络测试补充（优先级：中）

#### 任务 6.1：P2P 协议测试

新增测试文件：`crates/p2p/tests/protocol_tests.rs`

- 帧结构编解码
- 请求类型序列化/反序列化
- GetInfo/GetPeers/GetCumulativeDifficulty 请求构建
- ProcessBlock/ProcessTransactions 请求构建

#### 任务 6.2：P2P Handler 测试

新增测试文件：`crates/p2p/tests/handler_tests.rs`

- GetInfoHandler 处理逻辑
- GetPeersHandler 处理逻辑
- GetCumulativeDifficultyHandler 处理逻辑
- GetMilestoneBlockIdsHandler 处理逻辑
- GetNextBlockIdsHandler 处理逻辑
- GetNextBlocksHandler 处理逻辑
- ProcessBlockHandler 处理逻辑
- ProcessTransactionsHandler 处理逻辑

#### 任务 6.3：区块同步测试

新增测试文件：`crates/p2p/tests/sync_tests.rs`

- 区块链同步守护进程逻辑
- 连接发现逻辑
- 黑名单过期清理逻辑
- 交易广播逻辑

### 阶段 7：端到端集成测试（优先级：低）

#### 任务 7.1：节点启动集成测试

新增测试文件：`tests/integration/node_tests.rs`

- 节点启动和初始化
- 创世区块加载
- API 服务可用性
- P2P 服务可用性

#### 任务 7.2：区块链操作集成测试

新增测试文件：`tests/integration/blockchain_tests.rs`

- 完整的转账流程（创建→签名→广播→出块→确认）
- 完整的资产发行→转账→分红流程
- 完整的货币发行→兑换→铸造流程
- 完整的投票创建→投票→结果查询流程

---

## 四、实施优先级与依赖关系

```
阶段 1（基础设施）
  ↓
阶段 2（核心模块单元测试）+ 阶段 3（ORM 集成测试）
  ↓
阶段 4（交易引擎业务逻辑测试）
  ↓
阶段 5（HTTP API 集成测试）+ 阶段 6（P2P 网络测试）
  ↓
阶段 7（端到端集成测试）
```

---

## 五、测试文件清单

### 新增测试文件（共约 35 个）

| # | 文件路径 | 对应 Java 测试 | 优先级 |
|---|---------|---------------|--------|
| 1 | `crates/test-utils/src/lib.rs` | - | P0 |
| 2 | `crates/test-utils/src/fixtures.rs` | BlockchainTest/Tester | P0 |
| 3 | `crates/test-utils/src/db_helper.rs` | AbstractBlockchainTest | P0 |
| 4 | `crates/test-utils/src/api_client.rs` | APICall.Builder | P0 |
| 5 | `crates/test-utils/src/assertions.rs` | JSONAssert | P0 |
| 6 | `crates/test-utils/src/mock_helpers.rs` | - | P0 |
| 7 | `crates/crypto/tests/sm2_tests.rs` | TestSM2 | P1 |
| 8 | `crates/crypto/tests/sm3_tests.rs` | - | P1 |
| 9 | `crates/crypto/tests/sm4_gcm_tests.rs` | - | P1 |
| 10 | `crates/crypto/tests/aes_gcm_tests.rs` | - | P1 |
| 11 | `crates/crypto/tests/reed_solomon_tests.rs` | - | P1 |
| 12 | `crates/crypto/tests/passphrase_tests.rs` | - | P1 |
| 13 | `crates/consensus/tests/generation_signature_tests.rs` | - | P1 |
| 14 | `crates/consensus/tests/target_tests.rs` | - | P1 |
| 15 | `crates/consensus/tests/selection_tests.rs` | - | P1 |
| 16 | `crates/consensus/tests/block_gen_tests.rs` | - | P1 |
| 17 | `crates/consensus/tests/generator_tests.rs` | GeneratorTest | P1 |
| 18 | `crates/account/tests/account_manager_tests.rs` | - | P1 |
| 19 | `crates/account/tests/address_generator_tests.rs` | - | P1 |
| 20 | `crates/orm/tests/block_repository_tests.rs` | - | P1 |
| 21 | `crates/orm/tests/transaction_repository_tests.rs` | - | P1 |
| 22 | `crates/orm/tests/account_repository_tests.rs` | - | P1 |
| 23 | `crates/orm/tests/asset_repository_tests.rs` | - | P2 |
| 24 | `crates/orm/tests/currency_repository_tests.rs` | - | P2 |
| 25 | `crates/tx-engine/tests/send_money_tests.rs` | SendMoneyTest | P2 |
| 26 | `crates/tx-engine/tests/send_message_tests.rs` | SendMessageTest | P2 |
| 27 | `crates/tx-engine/tests/asset_tests.rs` | AssetTest/AssetExchangeTest | P2 |
| 28 | `crates/tx-engine/tests/asset_property_tests.rs` | AssetPropertyTest | P2 |
| 29 | `crates/tx-engine/tests/currency_tests.rs` | TestCurrencyIssuance/Exchange/Mint/Reserve/Delete | P2 |
| 30 | `crates/tx-engine/tests/mint_calculation_tests.rs` | TestMintCalculations | P2 |
| 31 | `crates/tx-engine/tests/phasing_tests.rs` | PhasingOnlyTest/CompositePhasingOnlyTest | P2 |
| 32 | `crates/tx-engine/tests/bundler_tests.rs` | AssetBundlerTest | P2 |
| 33 | `crates/tx-engine/tests/voting_tests.rs` | TestCreatePoll/CastVote/GetPolls | P2 |
| 34 | `crates/tx-engine/tests/lease_tests.rs` | LeaseTest | P2 |
| 35 | `crates/tx-engine/tests/account_property_tests.rs` | AccountInfoTest/PropertiesTest | P2 |
| 36 | `crates/http-api/tests/nrcs_api_tests.rs` | - | P3 |
| 37 | `crates/http-api/tests/v1_api_tests.rs` | - | P3 |
| 38 | `crates/http-api/tests/api_error_tests.rs` | - | P3 |
| 39 | `crates/p2p/tests/protocol_tests.rs` | - | P3 |
| 40 | `crates/p2p/tests/handler_tests.rs` | - | P3 |
| 41 | `crates/p2p/tests/sync_tests.rs` | - | P3 |

---

## 六、关键技术决策

### 6.1 测试数据库策略

- **单元测试/业务逻辑测试**：使用 `SqlitePool::connect("sqlite::memory:")` + 内联 DDL
- **API 集成测试**：使用 Mock 实现（参照现有 `api_integration_test.rs` 模式）
- **端到端测试**：使用文件数据库（`sqlite:file:...`），测试后清理

### 6.2 出块机制

Java NRCS 使用 `enableFakeForging=true` + `fakeForgingAccount` 实现假锻造。Rust 中需要：
- 在测试配置中支持假锻造模式
- 提供 `generate_block()` 辅助方法
- 提供区块回滚机制（`pop_off_to()`）

### 6.3 测试隔离

- 每个测试函数使用独立的内存数据库实例
- 测试账户使用固定密钥短语，确保确定性
- 余额差值断言模式（操作前后余额差值比较）

### 6.4 Mock 策略

- 优先使用真实实现（内存数据库 + 真实 Repository）
- 仅在无法使用真实实现时使用 Mock（如 P2P 网络层）
- Mock 实现集中在 `test-utils` crate 中，避免重复

---

## 七、验收标准

1. **所有新增测试通过**：`cargo test --workspace` 无失败
2. **代码质量**：`cargo clippy -- -D warnings` 无警告
3. **覆盖率目标**：
   - crypto: 100%
   - consensus: 100%
   - tx-engine: 95%
   - p2p: 90%
   - http-api: 90%
   - orm: 核心表 Repository 100%
   - account: 基本功能 100%
4. **与 Java NRCS 兼容性**：测试用例覆盖 Java `nrcs-test` 模块的所有测试场景
