# 区块同步测试报告（10区块请求测试）

**测试日期**：2026-04-25
**测试目标**：验证Rust NRCS节点从Java NRCS节点同步区块，请求数量限制为10个
**测试地址**：192.168.110.87
**数据库**：SQLite

## 测试配置修改

1. **SEGMENT_SIZE**: 从36改为10（每次请求10个区块）
2. **黑名单逻辑修改**: 将节点因返回过多区块而列入黑名单改为警告继续处理

## 测试结果

### 成功项
- ✅ Rust节点成功启动
- ✅ 成功连接到Java节点（192.168.110.87:17974）
- ✅ P2P握手成功
- ✅ 获取了791个区块ID
- ✅ 从Java节点下载了791个区块

### 问题项
- ❌ 所有区块高度都显示为0，而不是从1开始
- ❌ 数据库约束错误：UNIQUE constraint failed: BLOCK.HEIGHT（因为所有区块高度都是0）

## 日志分析

### 区块请求
```
Downloading 719 blocks from peer
```

### 区块处理
```
Processing downloaded block: height=0
```

### 错误信息
```
Block verification/processing failed: database error: database error: error returned from database: (code: 2067) UNIQUE constraint failed: BLOCK.HEIGHT
```

## 问题分析

### 1. 区块高度解析问题
从日志中可以看到，所有区块的height都显示为0，而不是正确的从1开始的值。这可能是因为：
- Java节点返回的区块数据中缺少height字段
- 或者height字段的解析逻辑有问题

### 2. 数据库约束错误
由于所有区块的高度都是0，尝试插入多个高度为0的区块时违反了BLOCK.HEIGHT的唯一性约束。

## 待解决问题

1. **区块高度解析**：需要检查Java节点返回的区块数据格式，确保height字段被正确解析
2. **区块同步逻辑**：需要确保区块按照正确的顺序和高度插入数据库
3. **数据库约束处理**：在插入区块前需要检查是否已存在，或使用UPSERT逻辑

## 建议

1. 检查Java节点返回的GetNextBlocks响应的完整内容
2. 添加更多调试日志，显示原始JSON数据
3. 验证区块高度是否符合预期（从1开始）
