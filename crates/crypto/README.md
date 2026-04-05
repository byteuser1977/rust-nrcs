# ZeroClaw Crypto

可插拔国密算法实现，支持国际算法与国密算法的无缝切换。

## 特性

- **哈希算法**: SHA-256 (默认) 或 SM3
- **签名算法**: Ed25519 (默认) 或 SM2
- **对称加密**: AES-CBC/GCM (默认) 或 SM4-CBC/SM4-GCM

## 配置

通过配置文件选择算法：

```toml
# config/default.toml
[crypto]
hash = "sm3"          # "sha256" (default) or "sm3"
signature = "sm2"    # "ed25519" (default) or "sm2"
cipher = "sm4"       # "aes" (default) or "sm4"
cipher_mode = "gcm"  # "cbc" or "gcm" (default: gcm)
```

或通过环境变量覆盖：
```
CONFIG_CRYPTO_HASH=sm3
CONFIG_CRYPTO_SIGNATURE=sm2
CONFIG_CRYPTO_CIPHER=sm4
CONFIG_CRYPTO_CIPHER_MODE=gcm
```

## 使用示例

```rust
use zeroclaw_crypto::{sha256, sign, verify, KeyPair, generate_keypair};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 哈希
    let data = b"hello world";
    let hash = sha256(data);
    println!("Hash: {}", hex::encode(hash));

    // 生成密钥对
    let kp = generate_keypair()?;
    println!("Public key ({} bytes): {}", kp.public_key().len(), hex::encode(kp.public_key().as_bytes()));

    // 签名
    let message = b"important message";
    let signature = sign(kp.secret_key(), message);
    println!("Signature: {}", hex::encode(signature.as_bytes()));

    // 验证
    verify(kp.public_key(), message, &signature)?;
    println!("Signature verified!");

    // 对称加密（如果配置了cipher）
    if let Some(cipher) = &zeroclaw_crypto::CRYPTO_CONFIG.cipher {
        let key = [0x00u8; 16];
        let iv = [0x00u8; 12];
        let plaintext = b"secret data";
        let ciphertext = cipher.encrypt(&key, &iv, plaintext)?;
        let recovered = cipher.decrypt(&key, &iv, &ciphertext)?;
        assert_eq!(recovered, plaintext);
    }

    Ok(())
}
```

## 架构

### 核心 Trait

```rust
pub trait HashAlgorithm {
    fn hash(&self, data: &[u8]) -> Hash256;
}

pub trait SignerAlgorithm {
    fn sign(&self, key: &SecretKey, message: &[u8]) -> Signature;
    fn verify(&self, key: &PublicKey, message: &[u8], signature: &Signature) -> CryptoResult<()>;
    fn generate_keypair(&self) -> CryptoResult<KeyPair>;
}

pub trait CipherAlgorithm {
    fn encrypt(&self, key: &[u8], iv: &[u8], plaintext: &[u8]) -> CryptoResult<CipherText>;
    fn decrypt(&self, key: &[u8], iv: &[u8], ciphertext: &CipherText) -> CryptoResult<Vec<u8>>;
}
```

### 全局配置

```rust
pub static CRYPTO_CONFIG: Lazy<CryptoConfig> = Lazy::new(load_config);
```

`CryptoConfig` 包含三个算法实例：`hash`, `signer`, `cipher`。便捷函数 `sha256()`, `sign()`, `verify()` 等自动使用配置中的算法。

### 类型系统

- `Hash256 = [u8; 32]`
- `PublicKey` 枚举：`Ed25519(ed25519_dalek::PublicKey)` | `Sm2(sm2::PublicKey)`
- `SecretKey` 枚举：`Ed25519(ed25519_dalek::SecretKey)` | `Sm2(sm2::SecretKey)`
- `Signature` 枚举：`Ed25519(...)` | `Sm2(...)`
- `KeyPair` 枚举：`Ed25519(...)` | `Sm2(...)`

## 算法细节

### 哈希
- **SHA-256**: 使用 `sha2` crate，符合 FIPS 180-4。
- **SM3**: 使用 `sm3` crate，符合 GM/T 0003-2012。

### 签名
- **Ed25519**: 使用 `ed25519-dalek`，高速抗侧信道。
- **SM2**: 使用 `sm2` crate，基于 NIST P-256 曲线，符合 GM/T 0003-2012。

### 对称加密
- **AES**: 支持 128/192/256 位密钥
  - CBC 模式：PKCS7 填充
  - GCM 模式：认证加密，128 位标签
- **SM4**:
  - CBC 模式：PKCS7 填充
  - GCM 模式：目前在实现中回退到 CBC（待完整实现）

## 测试

每个算法包含标准向量测试：

```bash
cargo test -p zeroclaw-crypto
```

### 已知测试向量

- SHA-256：FIPS PUB 180-4 向量
- SM3：GM/T 0003-2012 附录 A
- AES-GCM：NIST SP 800-38D

## 错误处理

返回 `CryptoResult<T>`，错误类型 `CryptoError` 包括：
- `InvalidKeyLength`
- `InvalidSignatureLength`
- `InvalidIvLength`
- `InvalidData`
- `VerificationFailed`
- `CipherNotConfigured`
- `UnsupportedAlgorithm`
- `ConfigError`

## 迁移指南

### 现有代码使用 `sha2::Sha256` 直接调用

替换为：

```rust
// Before
use sha2::{Digest, Sha256};
let hash = Sha256::digest(data);

// After
use zeroclaw_crypto::sha256;
let hash = sha256(data);
```

类似地，将 `ed25519_dalek` 签名代码改为调用 `sign()` 和 `verify()`。

通过配置可无缝切换算法，无需修改业务逻辑。

## 依赖项

```toml
[dependencies]
sha2 = "0.10"
ed25519-dalek = "2.0"
aes = "0.8"
aes-gcm = "0.11.0-rc.3"
cbc = "0.2.0-rc.4"
sm2 = "0.14.0-rc.8"
sm3 = "0.5.0"
sm4 = "0.6.0-rc.3"
config = "0.13"
once_cell = "1.18"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
rand = "0.10"
```

## 许可证

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 贡献

见 [CONTRIBUTING.md](../CONTRIBUTING.md)。
