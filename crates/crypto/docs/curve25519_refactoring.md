# Curve25519 重构计划

## 当前状态

Curve25519 签名算法（NRCS 兼容）正在重构中，暂时不可用。

## 背景

NRCS 使用的是自定义的 Curve25519 签名算法，与标准 Ed25519 不同。该算法从 Java NRCS 实现移植而来，但在移植过程中遇到了大量的 Rust 借用检查问题。

## 问题分析

### 1. 借用冲突

Java 实现中大量使用了可变引用和数组操作，这在 Rust 中会导致借用冲突。例如：

```java
// Java 代码
mul(ax, t2, t3);
mul(az, t1, t4);
```

在 Rust 中，如果 `t1`, `t2`, `t3`, `t4` 是同一个数组的元素，就会导致借用冲突。

### 2. 索引类型

Java 使用 `int` 作为索引，而 Rust 需要使用 `usize`，这导致了类型转换问题。

### 3. 可变引用

Java 中的可变参数在 Rust 中需要使用可变引用，但这会导致借用检查器的问题。

## 重构计划

### 阶段 1: 数据结构重构

1. **Long10 结构体**
   - 当前：手动实现 Copy/Clone
   - 改进：使用 `#[derive(Copy, Clone)]` 并添加更多 trait

2. **数组操作**
   - 当前：使用可变引用参数
   - 改进：使用返回值和元组

### 阶段 2: 函数签名重构

1. **辅助函数**
   ```rust
   // 当前
   fn mul(xy: &mut Long10, x: &Long10, y: &Long10);
   
   // 改进
   fn mul(x: &Long10, y: &Long10) -> Long10;
   ```

2. **核心函数**
   ```rust
   // 当前
   fn core(Px: &mut [u8; 32], s: Option<&mut [u8; 32]>, k: &[u8; 32], Gx: Option<&[u8; 32]>);
   
   // 改进
   fn core(k: &[u8; 32], gx: Option<&[u8; 32]>) -> ([u8; 32], Option<[u8; 32]>);
   ```

### 阶段 3: 测试和验证

1. 添加单元测试
2. 添加集成测试
3. 使用 NRCS 测试数据验证

## 临时替代方案

在 Curve25519 重构完成之前，建议使用 Ed25519 作为签名算法。

### 配置

```toml
# config/default.toml
[crypto]
signature = "ed25519"
```

### 使用

```rust
use crypto::{generate_passphrase, validate_passphrase, derive_account_id, derive_public_key};

// 生成助记词
let passphrase = generate_passphrase()?;

// 验证助记词
validate_passphrase(&passphrase)?;

// 派生公钥（使用 Ed25519）
let public_key = derive_public_key(&passphrase)?;

// 派生 Account ID（使用 Ed25519）
let account_id = derive_account_id(&passphrase)?;
```

## 时间估算

- 阶段 1（数据结构重构）：1-2 天
- 阶段 2（函数签名重构）：2-3 天
- 阶段 3（测试和验证）：1-2 天
- **总计**：4-7 天

## 风险评估

### 高风险
- 算法正确性：需要确保重构后的算法与 Java 实现完全一致
- 性能：需要确保重构不会影响性能

### 缓解措施
- 使用 NRCS 测试数据进行完整验证
- 添加性能基准测试
- 保留 Java 实现作为参考

## 参考资源

- [NRCS Java 实现](file:///mnt/d/workspace/git/nrcs/nrcs-crypto/src/main/java/com/bytechain/nrcs/crypto/Curve25519.java)
- [Curve25519 论文](https://cr.yp.to/ecdh/curve25519-20060209.pdf)
- [Ed25519 论文](https://ed25519.cr.yp.to/ed25519-20110926.pdf)

## 更新日志

- **2026-04-23**: 初始计划创建，暂时禁用 Curve25519 实现
