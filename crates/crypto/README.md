# Crypto 模块说明文档

## 概述

`crypto` 模块是 NRCS 区块链的核心加密原语库，提供以下功能：

- **数字签名**：Ed25519、Curve25519（NRCS 兼容）、SM2（国密）
- **哈希算法**：SHA-256、SM3（国密）
- **对称加密**：SM4-GCM（国密）
- **助记词管理**：NRCS 标准 12 词助记词生成与验证
- **账户系统**：账户 ID 派生、Reed-Solomon 编码

## 模块架构

```
crates/crypto/src/
├── lib.rs              # 模块入口，类型定义，便捷函数导出
├── config.rs           # 算法配置管理
├── crypto.rs           # Crypto 统一服务结构体
├── keypair.rs          # 统一密钥对类型（KeyPair 枚举）
├── passphrase/         # 助记词模块
│   ├── mod.rs          # 助记词生成、验证、转换
│   └── words.rs        # NRCS 词表（2048 词）
├── reed_solomon.rs     # Reed-Solomon 编码（账户地址）
└── algorithms/         # 算法实现
    ├── mod.rs          # Trait 定义
    ├── hash/           # 哈希算法
    │   ├── sha256.rs
    │   └── sm3.rs
    ├── signature/      # 签名算法
    │   ├── ed25519.rs
    │   └── curve25519/
    │       ├── mod.rs
    │       └── core.rs # EC-KCDSA 核心实现
    └── cipher/         # 加密算法
        └── sm4_gcm.rs
```

## 核心类型

### 公钥与私钥

```rust
/// 公钥类型
pub enum PublicKey {
    Ed25519([u8; 32]),      // Ed25519 公钥（32 字节）
    Curve25519([u8; 32]),   // Curve25519 公钥（32 字节）
}

/// 私钥类型
pub enum SecretKey {
    Ed25519([u8; 64]),      // Ed25519 私钥（64 字节：seed + public）
    Curve25519([u8; 32]),   // Curve25519 私钥（32 字节）
}

/// 签名类型（64 字节）
pub type Signature = [u8; 64];

/// 哈希类型（32 字节）
pub type Hash256 = [u8; 32];
```

### 密钥对

```rust
/// 统一密钥对类型
pub enum KeyPair {
    Ed25519(SigningKey),
    Curve25519 {
        public_key: [u8; 32],
        secret_key: [u8; 32],
    },
}
```

## 算法配置

### 配置结构

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoConfig {
    pub hash: String,       // "sha256" 或 "sm3"
    pub signature: String,  // "ed25519" 或 "curve25519"
    pub cipher: String,     // "sm4-gcm"
}
```

### 默认配置

```rust
impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            hash: "sha256".to_string(),
            signature: "ed25519".to_string(),
            cipher: "sm4-gcm".to_string(),
        }
    }
}
```

### 使用配置

```rust
use crypto::{Crypto, CryptoConfig};

// 使用默认配置
let crypto = Crypto::new(&CryptoConfig::default()).unwrap();

// 使用自定义配置
let config = CryptoConfig {
    hash: "sm3".to_string(),
    signature: "curve25519".to_string(),
    cipher: "sm4-gcm".to_string(),
};
let crypto = Crypto::new(&config).unwrap();
```

## API 使用指南

### 1. 哈希计算

```rust
use crypto::{sha256, hash, Hash256};

// 直接使用 SHA-256
let data = b"hello world";
let hash: Hash256 = sha256(data);

// 使用配置的哈希算法
let hash: Hash256 = hash(data);
```

### 2. 密钥对生成

```rust
use crypto::{generate_keypair, keypair_from_seed, KeyPair};

// 生成随机密钥对（使用配置的签名算法）
let kp: KeyPair = generate_keypair();

// 从种子派生密钥对
let seed = [0u8; 32];
let kp: KeyPair = keypair_from_seed(&seed);

// 获取公钥和私钥
let public_key = kp.public_key();
let secret_key = kp.secret_key();
```

### 3. 签名与验证

```rust
use crypto::{sign, verify, generate_keypair};

let kp = generate_keypair();
let message = b"test message";

// 签名
let signature = sign(&kp.secret_key(), message);

// 验证
let result = verify(&kp.public_key(), message, &signature);
assert!(result.is_ok());
```

### 4. NRCS 助记词

```rust
use crypto::{
    generate_passphrase, validate_passphrase,
    passphrase_to_keypair, derive_account_id,
};

// 生成 12 词助记词
let passphrase = generate_passphrase().unwrap();

// 验证助记词
validate_passphrase(&passphrase).unwrap();

// 从助记词派生密钥对
let keypair = passphrase_to_keypair(&passphrase).unwrap();

// 派生账户 ID（NRCS-XXXX-XXXX-XXXX-XXXXX 格式）
let account_id = derive_account_id(&passphrase).unwrap();
```

### 5. GCM 加密

```rust
use crypto::{encrypt_gcm, decrypt_gcm, random_32};

let key = random_32();
let nonce = &random_32()[..12];
let aad = b"additional authenticated data";
let plaintext = b"secret message";

// 加密
let (ciphertext, tag) = encrypt_gcm(&key, nonce, aad, plaintext).unwrap();

// 解密
let decrypted = decrypt_gcm(&key, nonce, aad, &ciphertext, &tag).unwrap();
assert_eq!(decrypted, plaintext);
```

## 签名算法对比

| 特性 | Ed25519 | Curve25519 (NRCS) | SM2 (国密) |
|------|---------|-------------------|------------|
| 密钥长度 | 32 字节种子 + 32 字节公钥 | 32 字节种子 | 32 字节种子 |
| 公钥长度 | 32 字节 | 32 字节 | 65 字节（未压缩） |
| 签名长度 | 64 字节 | 64 字节 | 64 字节 |
| 签名算法 | EdDSA | EC-KCDSA 变体 | SM2DSA |
| 性能 | 高 | 中 | 中 |
| NRCS 兼容 | 否 | **是** | 否 |
| 国密标准 | 否 | 否 | **是 (GM/T 0003-2012)** |
| 标准库 | ed25519-dalek | x25519-dalek + 自研 | sm2 crate |

### Curve25519 签名流程

NRCS 使用 Curve25519 的 EC-KCDSA 变体：

1. **密钥派生**：
   - `seed = SHA256(passphrase)`
   - `public_key = x25519_scalar_mult(seed, base_point)`

2. **签名生成**：
   - `m = SHA256(message)`
   - `x = SHA256(m || signing_key)`
   - `Y = keygen(x)` (临时公钥)
   - `h = SHA256(m || Y)`
   - `v = (x - h) * signing_key mod ORDER`
   - `signature = v || h`

3. **签名验证**：
   - `m = SHA256(message)`
   - `Y = verify(v, h, public_key)`
   - `h2 = SHA256(m || Y)`
   - 验证 `h == h2`

## NRCS 兼容性

### 测试向量

```rust
// 已验证的测试数据
const PASSPHRASE: &str = "concern entire frozen witch away creak dot drink need season clutch truly";
const PUBLIC_KEY: &str = "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c";
const ACCOUNT_ID: &str = "NRCS-LVB4-2ATD-4KZZ-2BXNV";
```

### 交易签名验证

```rust
// 交易签名验证通过
let unsigned_bytes = hex::decode(UNSIGNED_TRANSACTION_BYTES).unwrap();
let signature = nrcs_sign(&unsigned_bytes, PASSPHRASE);
let result = nrcs_verify(&signature, &unsigned_bytes, &public_key);
assert!(result);
```

## 错误处理

```rust
pub enum CryptoError {
    InvalidPublicKey(usize),
    InvalidSignature(usize),
    VerificationFailed,
    KeyGeneration(String),
    Sm2Error(String),
    Sm3Error(String),
    Sm4Error(String),
    ConfigurationError(String),
    CipherError(String),
}

pub type CryptoResult<T> = Result<T, CryptoError>;
```

## 依赖库

| 库 | 用途 |
|---|------|
| ed25519-dalek | Ed25519 签名 |
| x25519-dalek | Curve25519 密钥交换 |
| sha2 | SHA-256 哈希 |
| blake3 | BLAKE3 哈希 |
| rand | 随机数生成 |
| num-bigint | 大整数运算（助记词） |
| libsm | 国密算法（SM3, SM4） |
| once_cell | 全局配置单例 |
| thiserror | 错误类型定义 |

## 安全注意事项

1. **私钥保护**：私钥应存储在安全位置，使用后应清零内存
2. **随机数**：使用 `rand::thread_rng()` 生成密码学安全随机数
3. **签名验证**：始终验证签名，不要跳过验证步骤
4. **密钥派生**：使用标准算法派生密钥，不要自行实现

## 测试

```bash
# 运行所有测试
cargo test --package crypto

# 运行特定测试
cargo test --package crypto --test nrcs_signature_verify

# 运行 Curve25519 相关测试
cargo test --package crypto --lib -- curve25519
```

## 版本历史

- **v0.1.0**: 初始版本，支持 Ed25519、SHA-256
- **v0.2.0**: 添加 Curve25519 签名（NRCS 兼容）
- **v0.3.0**: 添加国密算法（SM3, SM4-GCM）
- **v0.4.0**: 完整 SM2 签名支持（GM/T 0003-2012）
- **v0.5.0**: 完整 SM4 分组密码支持（CBC, ECB, GCM 模式）
- **当前**: 完整 NRCS 兼容，国密算法完整实现
