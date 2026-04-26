# 区块验证逻辑核对计划

## 背景

当前 Rust-NRCS 项目在区块同步时，区块 ID 计算与 Java NRCS 不一致。创世区块 ID 正确（3488276486778630462），但区块 1 的 ID 不正确（实际：4540710371265965680，期望：3985281431710898053）。

## 参考数据

### Java NRCS Web API 创世区块 JSON
```json
{
    "baseTarget": "153722867",
    "block": "3488276486778630462",
    "blockSignature": "47b1aa800d657ccad4aaa8c946b2b0d2a7337fd3ab8e8c9ed6a06a49b7756e04a3ff13b15f6471afdff30313e1c47c4c2ab0e209c78a0673a42c254b74cc0201",
    "cumulativeDifficulty": "0",
    "generationSignature": "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "generator": "18365787021584764528",
    "generatorPublicKey": "b7f2232ddae77544690e1497f1b58e274039c3b0f99d9b6078f2520926230b26",
    "height": 0,
    "nextBlock": "3985281431710898053",
    "numberOfTransactions": 2,
    "payloadHash": "8f58dc2f809613424e608586df83b42513056861a864dff3cd00d88baca681ce",
    "payloadLength": 256,
    "timestamp": 0,
    "totalAmountNQT": "100000000000000000",
    "totalFeeNQT": "0",
    "version": -1
}
```

### 数据库记录格式
```
(DB_ID, ID, VERSION, TIMESTAMP, PREVIOUS_BLOCK_ID, TOTAL_AMOUNT, TOTAL_FEE, PAYLOAD_LENGTH, PREVIOUS_BLOCK_HASH, CUMULATIVE_DIFFICULTY, BASE_TARGET, NEXT_BLOCK_ID, HEIGHT, GENERATION_SIGNATURE, BLOCK_SIGNATURE, PAYLOAD_HASH, GENERATOR_ID)
(1, 3488276486778630462, -1, 0, NULL, 100000000000000000, 0, 256, NULL, [B@4f2218c3, 153722867, 3985281431710898053, 0, [B@751d647d, [B@7ac871c2, [B@25a46cfe, -80957052124787088)
```

## 问题分析

### 已确认正确的部分
1. 创世区块 ID = 3488276486778630462 ✅
2. 创世区块所有字段值正确 ✅
3. 区块 1 的非 ID 字段正确（TIMESTAMP=38, TOTAL_AMOUNT=100000000, BASE_TARGET=97357815 等）✅

### 问题点
1. 区块 1 的 ID 计算不正确
2. 需要核对 `serialize_for_id` 方法与 Java `Block.bytes()` 的一致性

## 实施步骤

### 步骤 1：创建单元测试验证区块 ID 计算
- 创建测试用例，使用创世区块和区块 1 的已知数据
- 验证 `serialize_for_id` 输出的字节序列
- 验证 `calculate_id` 计算的 ID 值

### 步骤 2：核对序列化逻辑
对比 Java `Block.bytes()` 和 Rust `serialize_for_id`：

| 字段 | Java 类型 | Rust 类型 | 序列化方式 |
|------|-----------|-----------|------------|
| version | int | i32 | `<i` (LE) |
| timestamp | int | u32 | `<I` (LE) |
| previousBlockId | long | Option<u64> | `<q` (LE) |
| txCount | int | usize | `<i` (LE) |
| totalAmount (v<3) | int | u64/4 | `<i` (LE) |
| totalAmount (v>=3) | long | u64 | `<q` (LE) |
| totalFee (v<3) | int | u64/4 | `<i` (LE) |
| totalFee (v>=3) | long | u64 | `<q` (LE) |
| payloadLength | int | u32 | `<i` (LE) |
| payloadHash | byte[32] | Hash256 | 直接写入 |
| generatorPublicKey | byte[32] | Option<[u8;32]> | 直接写入 |
| generationSignature (v=1/-1) | byte[64] | Vec<u8> | 直接写入 |
| generationSignature (v>=2) | byte[32] | Vec<u8> | 写入前32字节 |
| previousBlockHash (v>1) | byte[32] | Hash256 | 直接写入 |
| blockSignature | byte[64] | Hash512 | 直接写入 |

### 步骤 3：修复发现的问题
根据测试结果修复代码

### 步骤 4：调整数据库路径
将 SQLite 数据库路径从 `/tmp/nrcs.db` 改回 `rust-nrcs/nrcs.db`

### 步骤 5：重新测试验证
删除数据库，重新同步，验证区块数据正确性

## 测试验证方法

### Python 验证脚本
```python
import hashlib
import struct

# 创世区块数据
version = -1
timestamp = 0
previous_block_id = 0
tx_count = 2
total_amount = 100000000000000000 // 100000000  # v<3 除以 10^8
total_fee = 0
payload_length = 256

payload_hash = bytes.fromhex("8f58dc2f809613424e608586df83b42513056861a864dff3cd00d88baca681ce")
generator_public_key = bytes.fromhex("b7f2232ddae77544690e1497f1b58e274039c3b0f99d9b6078f2520926230b26")
generation_signature = bytes.fromhex("00" * 64)  # 64 字节全零
block_signature = bytes.fromhex("47b1aa800d657ccad4aaa8c946b2b0d2a7337fd3ab8e8c9ed6a06a49b7756e04a3ff13b15f6471afdff30313e1c47c4c2ab0e209c78a0673a42c254b74cc0201")

# 序列化
buf = b''
buf += struct.pack('<i', version)
buf += struct.pack('<i', timestamp)
buf += struct.pack('<q', previous_block_id)
buf += struct.pack('<i', tx_count)
buf += struct.pack('<i', total_amount)
buf += struct.pack('<i', total_fee)
buf += struct.pack('<i', payload_length)
buf += payload_hash
buf += generator_public_key
buf += generation_signature  # v=-1 写入全部 64 字节
# v=-1 不写入 previousBlockHash
buf += block_signature

# 计算 ID
hash_result = hashlib.sha256(buf).digest()
id_bytes = hash_result[:8]
reversed_bytes = bytes(reversed(id_bytes))
block_id = int.from_bytes(reversed_bytes, 'big')

print(f"Genesis block ID: {block_id}")
print(f"Expected: 3488276486778630462")
```

## 预期结果
- 创世区块 ID = 3488276486778630462 ✅
- 区块 1 ID = 3985281431710898053
- 区块 2 ID = 826792334011190233
- 所有区块数据与 Java NRCS 一致
