# 创世区块验证逻辑核对计划

## 背景

用户提供了创世区块的JSON数据（来自NRCS WebAPI）和数据库记录，需要核对区块验证逻辑的正确性。

### 创世区块参考数据

**JSON数据 (WebAPI)**:
```json
{
  "block": "3488276486778630462",
  "version": -1,
  "timestamp": 0,
  "height": 0,
  "baseTarget": "153722867",
  "cumulativeDifficulty": "0",
  "generator": "18365787021584764528",
  "generatorPublicKey": "b7f2232ddae77544690e1497f1b58e274039c3b0f99d9b6078f2520926230b26",
  "generationSignature": "0000...0000", // 64个0
  "blockSignature": "47b1aa80...c0201",
  "payloadHash": "8f58dc2f809613424e608586df83b42513056861a864dff3cd00d88baca681ce",
  "payloadLength": 256,
  "totalAmountNQT": "100000000000000000",
  "totalFeeNQT": "0",
  "transactions": [
    {"stringId": "12137079640911009279"},
    {"stringId": "2830446832482296829"}
  ]
}
```

**数据库记录**:
```
(1, 3488276486778630462, -1, 0, NULL, 100000000000000000, 0, 256, NULL, [B@..., 153722867, 3985281431710898053, 0, [B@..., [B@..., [B@..., -80957052124787088)
```

## 问题分析

### 1. Java NRCS Block.getId() 实现分析

```java
// Block.java L184-198
public long getId() {
    byte[] hash = Crypto.getDigest().digest(bytes());
    BigInteger bigInteger = new BigInteger(1, new byte[]{
        hash[7], hash[6], hash[5], hash[4], 
        hash[3], hash[2], hash[1], hash[0]
    });
    id = bigInteger.longValue();
}
```

**关键点**：
- 使用 SHA-256 哈希区块字节
- 取哈希的前8字节，**逆序**构造 BigInteger
- 这相当于：`u64::from_be_bytes(reversed_first_8_bytes)`

### 2. Java NRCS Block.bytes() 实现分析

```java
// Block.java L235-263
ByteBuffer buffer = ByteBuffer.allocate(...);
buffer.order(ByteOrder.LITTLE_ENDIAN);
buffer.putInt(this.getVersion());           // 4 bytes
buffer.putInt(this.getTimestamp());         // 4 bytes
buffer.putLong(this.getPreviousBlockId());  // 8 bytes
buffer.putInt(getTransactions().size());    // 4 bytes
if (this.getVersion() < 3) {
    buffer.putInt((int)(this.getTotalAmount() / Constant.ONE_NRCS)); // 4 bytes
    buffer.putInt((int)(this.getTotalFee() / Constant.ONE_NRCS));    // 4 bytes
} else {
    buffer.putLong(this.getTotalAmount());  // 8 bytes
    buffer.putLong(this.getTotalFee());     // 8 bytes
}
buffer.putInt(this.getPayloadLength());     // 4 bytes
buffer.put(this.getPayloadHash());          // 32 bytes
buffer.put(getGeneratorPublicKey());        // 32 bytes
buffer.put(this.getGenerationSignature());  // 32 or 64 bytes
if (this.getVersion() > 1) {
    buffer.put(this.getPreviousBlockHash()); // 32 bytes
}
if (this.getBlockSignature() != null) {
    buffer.put(this.getBlockSignature());    // 64 bytes
}
```

**关键点**：
- 使用 Little Endian 字节序
- generationSignature：
  - version=1/-1: 写入完整长度（创世区块为64字节）
  - version>1: 写入32字节
- previousBlockHash：仅当 version > 1 时写入

### 3. Rust 实现对比

当前 Rust 实现：
```rust
// block.rs L504-518
pub fn calculate_id(&self) -> Result<u64> {
    let data = self.serialize_for_id();
    let hash = Sha256::digest(&data);
    let mut id_bytes = [0u8; 8];
    id_bytes.copy_from_slice(&hash[..8]);
    id_bytes.reverse();
    Ok(u64::from_be_bytes(id_bytes))
}
```

**问题**：Rust 实现看起来是正确的，但需要验证序列化格式。

### 4. generator_id 存储问题

数据库记录显示 `generator_id = -80957052124787088`，但 JSON 中是 `18365787021584764528`。

计算：
```
18365787021584764528 (无符号)
= 0xFEDCBA9876543210

作为有符号 i64:
= -80957052124787088
```

**结论**：数据库使用有符号 i64 存储，Java 也是这样处理的。这不是错误，但需要在 Rust 中正确处理类型转换。

## 验证计划

### 步骤 1: 验证创世区块序列化格式

创建单元测试，验证创世区块序列化后的字节与 Java NRCS 一致。

**预期序列化长度**：
- version (4) + timestamp (4) + previousBlockId (8) + txCount (4) +
- amount (4, v<3) + fee (4, v<3) + payloadLength (4) + payloadHash (32) +
- generatorPublicKey (32) + generationSignature (64, v=-1) + blockSignature (64)
- = 4+4+8+4+4+4+4+32+32+64+64 = **224 bytes**

**注意**：version=-1 不满足 `version > 1`，所以不写入 previousBlockHash。

### 步骤 2: 验证区块ID计算

使用已知的创世区块数据，验证计算出的区块ID是否为 `3488276486778630462`。

### 步骤 3: 验证 generator_id 计算

验证从公钥计算出的账户ID是否为 `18365787021584764528`。

### 步骤 4: 验证数据库存储

验证 generator_id 在数据库中正确存储（考虑有符号/无符号转换）。

### 步骤 5: 验证创世区块交易

验证创世区块中的两笔交易是否正确处理。

## 实施任务

### 任务 1: 增强单元测试

在 `blockchain-types/src/block.rs` 中增强测试：

1. 添加创世区块完整序列化测试
2. 添加创世区块ID计算测试
3. 添加 generator_id 计算测试
4. 添加交易处理测试

### 任务 2: 修复序列化逻辑

检查并修复以下问题：

1. generationSignature 长度处理（v=1/-1 应为完整长度）
2. previousBlockHash 条件判断（v>1 才写入）
3. amount/fee 类型处理（v<3 时为 int，否则为 long）

### 任务 3: 修复数据库模型

检查 BlockModel 的类型转换：

1. generator_id 使用 i64 存储（与 Java 一致）
2. id 使用 i64 存储
3. 正确处理有符号/无符号转换

### 任务 4: 运行验证测试

1. 删除现有数据库
2. 启动节点同步
3. 验证创世区块数据与参考数据一致
4. 验证后续区块数据与 block_meg.sql 一致

## 预期结果

1. 创世区块ID计算正确：`3488276486778630462`
2. generator_id 计算正确：`18365787021584764528`
3. 数据库存储正确（有符号表示）
4. 所有区块数据与 Java NRCS 一致

## 风险点

1. **字节序问题**：确保所有数值使用 Little Endian
2. **类型转换**：u64/i64 转换需要小心处理
3. **交易序列化**：创世区块的交易需要特殊处理
4. **cumulative_difficulty**：确保序列化格式与 Java BigInteger 一致
