# 区块同步和交易同步修复计划

## 背景

当前区块同步存在问题：
1. 每次同步都从高度 0 开始，而不是从最新的本地区块高度继续
2. 交易同步逻辑不完整，只有创世区块的交易被写入
3. 缺少错误级别日志
4. 数据库路径需要恢复到项目目录

## 任务清单

### 任务 1: 修复同步逻辑，从最新的本地区块高度继续

**问题分析**：
- 当前 `get_common_milestone_block_id` 函数在找到本地区块后返回，但 `get_block_ids_after_common` 函数仍然从创世区块开始下载
- 需要修改逻辑，使同步从最新的本地区块高度继续

**修复方案**：
1. 修改 `blockchain_sync.rs` 中的同步逻辑
2. 使用 `get_last_block_id` 获取最新的本地区块ID
3. 从最新区块的下一个区块开始下载

**涉及文件**：
- `/mnt/d/workspace/git/rust-nrcs/crates/p2p/src/daemon/blockchain_sync.rs`

### 任务 2: 完整移植交易同步逻辑

**问题分析**：
- 当前交易同步逻辑不完整，只有创世区块的交易被写入
- 创世区块交易数据需要修正
- 需要参考 Java NRCS 的 `BlockService.saveBlock()` 方法实现

**创世区块交易数据（修正后）**：
```
交易1: ID=-6309664432798542337, RECIPIENT_ID=2794603741293765856, AMOUNT=99999999900000000, FEE=100000000
交易2: ID=2830446832482296829, RECIPIENT_ID=-891382425467438890, AMOUNT=100000000, FEE=100000000
```

**Java NRCS 参考代码**：
```java
// BlockService.java
public static void saveBlock(Block block) {
    // 保存区块
    blockDb.saveBlock(block);
    // 保存交易
    for (Transaction transaction : block.getTransactions()) {
        transactionDb.saveTransaction(transaction);
    }
}
```

**修复方案**：
1. 修正 `genesis.rs` 中的创世区块交易数据
2. 在 `verifier.rs` 的 `insert_block` 方法中添加交易插入逻辑
3. 确保 `transaction` 表名使用双引号转义（SQLite 关键字）
4. 实现 `TransactionRepository` 的 `insert` 方法

**涉及文件**：
- `/mnt/d/workspace/git/rust-nrcs/crates/orm/src/genesis.rs`
- `/mnt/d/workspace/git/rust-nrcs/crates/p2p/src/verifier.rs`
- `/mnt/d/workspace/git/rust-nrcs/crates/orm/src/repository/sqlite.rs`
- `/mnt/d/workspace/git/rust-nrcs/crates/orm/src/models/transaction.rs`

### 任务 3: 添加错误级别日志

**修复方案**：
1. 在同步失败时使用 `error!` 宏记录错误日志
2. 在交易处理失败时记录错误日志
3. 在数据库操作失败时记录错误日志

**涉及文件**：
- `/mnt/d/workspace/git/rust-nrcs/crates/p2p/src/daemon/blockchain_sync.rs`
- `/mnt/d/workspace/git/rust-nrcs/crates/p2p/src/verifier.rs`

### 任务 4: 恢复数据库路径到项目目录

**修复方案**：
1. 修改 `config/default.toml` 中的数据库路径
2. 从 `sqlite:///tmp/nrcs_sync.db?mode=rwc` 改为 `sqlite:///mnt/d/workspace/git/rust-nrcs/nrcs.db?mode=rwc`

**涉及文件**：
- `/mnt/d/workspace/git/rust-nrcs/config/default.toml`

### 任务 5: 测试验证

**测试步骤**：
1. 删除现有数据库
2. 启动节点进行同步
3. 验证区块数据正确写入
4. 验证交易数据正确写入
5. 重启节点，验证同步从最新高度继续

### 任务 6: ORM 模块封装（后续任务）

**目标**：
- 将 `sqlx::query` 的实现逻辑通过 orm 模块封装后统一访问
- 提供统一的数据库访问接口

## 预期结果

1. 区块同步从最新的本地区块高度继续
2. 交易数据正确写入数据库
3. 错误日志正确记录
4. 数据库路径恢复到项目目录
5. 同步测试成功

## 风险点

1. **SQLite 关键字处理**：确保 `transaction` 表名使用双引号转义
2. **交易 ID 计算**：确保交易 ID 计算与 Java NRCS 一致
3. **并发安全**：确保交易插入不会重复
