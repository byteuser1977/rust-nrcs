# NRCS 签名验证兼容性实现计划

## 目标

实现与 NRCS Java 完全兼容的 Curve25519 签名和验证算法。

## 问题分析

### 字节序问题

**Java Curve25519 实现分析**：
1. `unpack` 函数使用**小端格式**解析字节数组：
   ```java
   x._0 = ((m[0] & 0xFF)) | ((m[1] & 0xFF)) << 8 | ...
   ```
   这意味着 `m[0]` 是最低有效字节。

2. Java 的 `ORDER` 常量：
   ```java
   ORDER[0] = 237  // 最低有效字节
   ORDER[31] = 16  // 最高有效字节
   ```

3. 当前 Rust 的 `ORDER` 是**大端格式**（错误）：
   ```rust
   ORDER[0] = 16   // 这是最高有效字节，与 Java 不一致！
   ORDER[31] = 237
   ```

### 测试数据

**交易 1** (9344731544105219785):
- Passphrase: `concern entire frozen witch away creak dot drink need season clutch truly`
- Public Key: `2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c`
- Signature: `7f2bb75686349576e634083b1f0a30cb30209cb8db952b9b1b1c2072354cf2076cc640b3ca04d03b82c3cb6f1e44efbf18761f9b334f35ab2bcba98b7e1b06d5`

**交易 2** (2452132941054689657):
- Same Passphrase
- Signature: `eab9a9fd3d73950a372e17a76a38fb206b875a84f9bc4fa80b18307ea683f204fcffc4413d187302a84ccdf89d796130a6e9d161afc30a45fc41e4257af4f0c5`
- Unsigned Transaction Bytes: `001037b138053c002d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c5fe6cbd7bfb374290065cd1d0000000000e1f505000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000007cb1400e278bbd91da7aae30163fc7d52fb0c3d2cd86b77a1a4f32b233735c3108a68d6a525909d4efe887dde`

## 实现计划

### 阶段 1: 修复 ORDER 常量

将 Rust 的 ORDER 改为与 Java 一致的小端格式：

```rust
pub static ORDER: [u8; 32] = [
    237, 211, 245, 92, 26, 99, 18, 88,
    214, 156, 247, 162, 222, 249, 222, 20,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 16,
];

static ORDER_TIMES_8: [u8; 32] = [
    104, 159, 174, 231, 210, 24, 147, 192,
    178, 230, 188, 23, 245, 206, 247, 166,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 128,
];
```

### 阶段 2: 统一字节序处理

**关键发现**：NRCS JSON API 返回的 hex 字符串是**小端格式**的字节数组的十六进制表示。

**Java 的 parseHexString 函数**：
```java
bytes[i] = (byte) ((char1 << 4) + char2);
```
这个函数按顺序解析，不进行字节序转换。

**结论**：
1. hex 字符串的第一个字节对应 bytes[0]
2. 测试代码中的字节反转是**不必要的**
3. 输入数据已经是小端格式，直接传递给核心算法即可

### 阶段 3: 验证签名生成

验证签名生成过程：
1. m = SHA256(unsigned_transaction_bytes)
2. x = SHA256(m || s)
3. Y = keygen(x)
4. h = SHA256(m || Y)
5. v = sign(h, x, s)
6. signature = v || h

### 阶段 4: 验证签名验证

验证签名验证过程：
1. m = SHA256(message)
2. Y = verify(v, h, P)
3. h2 = SHA256(m || Y)
4. 验证 h == h2

### 阶段 5: 集成测试

使用提供的测试数据进行完整验证：
- 交易 1 签名验证
- 交易 2 签名验证
- 自生成签名验证

## 预期结果

- 所有测试向量的签名验证通过
- 与 NRCS Java 实现完全兼容
