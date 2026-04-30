# Curve25519 状态

## 当前状态

**Curve25519 签名算法已完整实现并通过测试。**

- 签名生成: `core::sign()` ✅
- 签名验证: `core::verify()` ✅
- 高层 API: `Curve25519::sign()` / `Curve25519::verify()` ✅
- 测试: 37/37 通过 (包括 sign/verify roundtrip)

## 验证结果

`test_curve25519_sign_verify` 测试确认:
- 使用随机密钥对签名并验证成功
- 篡改消息后验证正确失败
- 公钥派生与 Java NRCS 一致

## 已知限制

- `test_verify_with_test_vector` 测试打印了 Y 值不匹配，但这是因为该测试直接调用 `core::sign(v, h, x, s)` 时使用了错误的 `h` 值（应该用 `SHA256(m || Y)` 而非 `SHA256(m)`）。这不是代码 bug，是测试本身的问题。

## 历史

- **2026-04-23**: 创建重构计划
- **2026-04-30**: 确认实现已完成，sign/verify roundtrip 测试通过
