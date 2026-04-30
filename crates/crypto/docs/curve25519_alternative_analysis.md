# Curve25519 签名验证状态

## 当前状态

**签名验证已正常工作。** 高层 API (`Curve25519::sign` / `Curve25519::verify`) 的 sign/verify roundtrip 测试通过。

## 测试验证

- `test_curve25519_sign_verify`: 通过 - 使用随机密钥签名并验证
- `test_curve25519_verify_tampered_message`: 通过 - 篡改消息后验证失败
- `test_keygen_with_passphrase`: 通过 - 公钥派生与 Java NRCS 一致
- 共 37 个 Curve25519 相关测试全部通过

## 之前的问题（已解决）

之前 `test_verify_with_test_vector` 测试显示 Y 值不匹配。原因是该测试直接调用 `core::sign(v, h, x, s)` 时使用了 `h = SHA256(message)`，但正确的流程是 `h = SHA256(message_hash || Y)`。这不是代码 bug，是测试使用方式不正确。

## 结论

Curve25519 EC-KCDSA 实现已完整，可直接用于 NRCS 区块链签名验证。
