# 区块同步测试分析报告

**测试日期**：2026-04-25
**测试目标**：分析Rust NRCS与Java NRCS的区块同步问题

## 当前状态

- **Java NRCS区块总数**：1,368,736
- **Rust本地数据库区块数**：691
- **Java NRCS状态**：DOWNLOADING
- **Java NRCS cumulativeDifficulty**：70800403529522349

## 发现的问题

### 1. 区块高度计算问题（已修复）

**问题**：Java NRCS的GetNextBlocks API返回的区块数据中**不包含height字段**

Java Block.java的getJSONObject()方法：
```java
public JSONObject getJSONObject() {
    JSONObject json = new JSONObject();
    json.put("blockSignature", Convert.toHexString(getBlockSignature()));
    json.put("generationSignature", Convert.toHexString(getGenerationSignature()));
    json.put("generatorPublicKey", Convert.toHexString(getPublicKey()));
    json.put("payloadHash", Convert.toHexString(getPayloadHash()));
    json.put("payloadLength", getPayloadLength());
    json.put("previousBlock", Long.toUnsignedString(getPreviousBlockId()));
    json.put("previousBlockHash", Convert.toHexString(getPreviousBlockHash()));
    json.put("timestamp", getTimestamp());
    json.put("totalAmountNQT", getTotalAmountNQT());
    json.put("totalFeeNQT", getTotalFeeNQT());
    json.put("transactions", transactions);
    json.put("version", getVersion());
    // 注意：没有包含 height 字段
    return json;
}
```

**修复**：根据区块在列表中的位置计算高度
```rust
let base_height = start_idx as u32 + 1;
for (block_idx, block_data) in next_blocks.iter().enumerate() {
    Self::process_downloaded_block(&block_data, base_height + block_idx as u32, ...)
}
```

### 2. Java NRCS GetNextBlocks 限制

Java GetNextBlocks.java:
```java
if (stringList.size() > 36) {
    return TOO_MANY_BLOCKS_REQUESTED;
}
```

Java每次最多返回36个区块，即使请求更多。

### 3. 分批下载逻辑

Java downloadBlockchain():
```java
int segSize = 36;
int stop = chainBlockIds.size() - 1;
for (int start = 0; start < stop; start += segSize) {
    getList.add(new GetNextBlocks(chainBlockIds, start, Math.min(start + segSize, stop)));
}
```

## 数据库现状

```
691|0|720  // 691个区块，高度0到720
```

但高度不连续：
```
0, 4, 5, 6, 7, 8, 9, 10, 11, 12, ... 720
```

缺少高度1, 2, 3等。

## 待解决问题

1. **创世区块插入**：创世区块没有正确插入到数据库
2. **高度不连续**：有些区块插入失败（UNIQUE约束错误）
3. **区块验证逻辑**：需要确保按照正确的链顺序插入区块

## 下一步

1. 清理数据库，重新初始化
2. 确保创世区块正确插入
3. 实现正确的区块验证和插入逻辑
