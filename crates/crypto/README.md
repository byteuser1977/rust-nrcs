# Crypto Crate - 加密原语

提供 NRCs 区块链所需的核心加密功能，包括：

- **Ed25519**：数字签名（原有）
- **SHA-256 / BLAKE3**：哈希计算（原有）
- **SM2**：椭圆曲线数字签名（国密）
- **SM3**：密码杂凑算法（国密）
- **SM4**：分组密码（国密），支持 CBC 和 GCM 模式

---

## 快速开始

### 签名（Ed25519 或 SM2）

```rust
use crypto::{KeyPair, verify};

// Ed25519（原有）
let kp_ed = KeyPair::generate();
let msg = b"hello";
let sig = kp_ed.sign(msg);
assert!(verify(&kp_ed.public_key(), msg, &sig).is_ok());

// SM2（新增）
use crypto::sm2::KeyPair as Sm2KeyPair;
let kp_sm2 = Sm2KeyPair::generate();
let sig_sm2 = kp_sm2.sign(msg);
assert!(crypto::sm2::verify(&kp_sm2.public_key(), msg, &sig_sm2).is_ok());
```

### 哈希（SM3）

```rust
use crypto::sm3;

let hash = sm3(b"hello world");
assert_eq!(hash.len(), 32);
```

### 对称加密（SM4）

```rust
use crypto::sm4::{Sm4Key, encrypt_cbc, decrypt_cbc};

let key = Sm4Key::random();
let mut iv = [0u8; 16];
rand::thread_rng().fill_bytes(&mut iv);

let plaintext = b"Secret message";
let ciphertext = encrypt_cbc(plaintext, &key, &iv);
let decrypted = decrypt_cbc(&ciphertext, &key).unwrap();

assert_eq!(plaintext, decrypted.as_slice());
```

#### GCM 模式

```rust
use crypto::sm4::{encrypt_gcm, decrypt_gcm, Sm4Key};

let key = Sm4Key::random();
let mut nonce = [0u8; 12];
rand::thread_rng().fill_bytes(&mut nonce);
let aad = b"header";

let (ciphertext, tag) = encrypt_gcm(b"data", &key, &nonce, aad).unwrap();
let plaintext = decrypt_gcm(&ciphertext, &key, &nonce, &tag, aad).unwrap();
```

---

## 国密算法实现细节

| 算法  |  crate   | 输出长度 | 密钥长度 | 模式/用途           |
|-------|----------|----------|----------|---------------------|
| SM2   | sm2      | 64 字节  | 32 字节  | 签名/验证（非加密） |
| SM3   | sm3      | 32 字节  | N/A      | 哈希、KDF           |
| SM4   | sm4      | 变长     | 16 字节  | CBC（填充）、GCM    |

详细技术文档见 `docs/crypto-gm-implementation-report.md`。

---

## API 兼容性

- 新 SM2 `KeyPair` 保持与 Ed25519 `KeyPair` 相同的方法名（`generate`, `from_seed`, `sign`, `public_key`）。
- SM4 加密函数与 Java `Crypto.aesEncrypt` 风格一致：返回 `iv || ciphertext` 格式，便于跨语言迁移。

---

## 安全建议

- 使用 `Sm4Key::derive_from` 从密码派生密钥，而非硬编码。
- 私钥和密钥材料使用后应显式 `zeroize`（已自动处理）。
- GCM 模式推荐使用 12 字节随机 nonce；勿重复使用同一 nonce+key 组合。
- CBC 模式适用于无认证需求的场景；优先使用 GCM 认证加密。

---

## 测试

运行测试：

```bash
cargo test -p crypto
```

测试覆盖：
- 标准向量验证（GM/T）
- 加解密往返
- 错误注入（错误密钥、错误 tag、填充错误）
- 零化内存检查

---

## 许可证

Apache-2.0

---

## 参考

- GM/T 0003-2012: SM2 椭圆曲线数字签名算法
- GM/T 0004-2012: SM3 密码杂凑算法
- GM/T 0002-2012: SM4 分组密码算法
- Java 参考实现: `nrcs-crypto/src/main/java/com/bytechain/nrcs/crypto/Crypto.java`
