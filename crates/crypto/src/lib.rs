//! # Cryptography Module
//!
//! 加密原语模块，提供区块链所需的核心加密功能：
//! - 数字签名（Ed25519, Curve25519）
//! - 哈希算法（SHA-256, BLAKE3, SM3）
//! - **可插拔算法抽象层**：通过配置文件动态选择算法
//!
//! ## 架构
//! - `algorithms/`: 定义核心 trait (HashAlgorithm, SignatureAlgorithm, GcmAlgorithm) 及具体实现
//! - `config.rs`: 配置管理（从 `config/default.toml` 读取）
//! - `keypair.rs`: 统一密钥对类型（枚举，支持多算法）
//! - `crypto.rs`: `Crypto` 结构体（根据配置组合算法）、向后兼容的便捷函数
//!
//! ## 使用示例
//!
//! ### 便捷函数（兼容原有 API）
//! ```
//! use crypto::{generate_keypair, sha256};
//!
//! let data = b"hello world";
//! let hash = sha256(data);
//!
//! let kp = generate_keypair();
//! let sig = kp.sign(data);
//! assert!(crypto::verify(&kp.public_key(), data, &sig).is_ok());
//! ```
//!
//! ### 直接使用 `Crypto` 结构体（显式控制）
//! ```
//! use crypto::{Crypto, CryptoConfig};
//!
//! let cfg = CryptoConfig::default();
//! let crypto = Crypto::new(&cfg).unwrap();
//!
//! let hash = crypto.hash(b"data");
//! let kp = crypto.generate_keypair();
//! let sig = crypto.sign(&kp.secret_key(), b"msg");
//! ```
//!
//! ## 算法配置
//!
//! 在 `config/default.toml` 中设置：
//! ```toml
//! [crypto]
//! hash = "sha256"        # "sha256" 或 "sm3"
//! signature = "ed25519"  # "ed25519" 或 "curve25519"
//! ```
//!
//! 默认配置：Ed25519 + SHA-256

/// 固定大小的 SHA-256 哈希（32 字节）
pub type Hash256 = [u8; 32];

/// 签名类型（64 字节）
pub type Signature = [u8; 64];

/// 公钥类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicKey {
    /// Ed25519 公钥（32 字节）
    Ed25519([u8; 32]),
    /// Curve25519 公钥（32 字节）- NRCS 兼容
    Curve25519([u8; 32]),
    /// SM2 公钥（65 字节，未压缩格式）+ 可选区分标识符
    Sm2 {
        public_key: [u8; 65],
        distid: Option<String>,
    },
}

impl PublicKey {
    /// 获取公钥长度（字节）
    pub fn len(&self) -> usize {
        match self {
            PublicKey::Ed25519(bytes) => bytes.len(),
            PublicKey::Curve25519(bytes) => bytes.len(),
            PublicKey::Sm2 { public_key, .. } => public_key.len(),
        }
    }

    /// 公钥是否为空（始终返回 false，因为公钥长度固定）
    pub fn is_empty(&self) -> bool {
        false
    }

    /// 转换为字节 slice
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            PublicKey::Ed25519(bytes) => bytes,
            PublicKey::Curve25519(bytes) => bytes,
            PublicKey::Sm2 { public_key, .. } => public_key,
        }
    }
}

/// 私钥类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretKey {
    /// Ed25519 私钥（64 字节）
    Ed25519([u8; 64]),
    /// Curve25519 私钥（32 字节）- NRCS 兼容
    Curve25519([u8; 32]),
    /// SM2 私钥（32 字节）+ 可选区分标识符
    Sm2 {
        secret_key: [u8; 32],
        distid: Option<String>,
    },
}

impl SecretKey {
    /// 获取私钥长度
    pub fn len(&self) -> usize {
        match self {
            SecretKey::Ed25519(bytes) => bytes.len(),
            SecretKey::Curve25519(bytes) => bytes.len(),
            SecretKey::Sm2 { secret_key, .. } => secret_key.len(),
        }
    }

    /// 私钥是否为空（始终返回 false，因为私钥长度固定）
    pub fn is_empty(&self) -> bool {
        false
    }

    /// 转换为字节 slice
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            SecretKey::Ed25519(bytes) => bytes,
            SecretKey::Curve25519(bytes) => bytes,
            SecretKey::Sm2 { secret_key, .. } => secret_key,
        }
    }
}

// 模块声明
pub mod algorithms;
pub mod config;
pub mod crypto;
pub mod keypair;
pub mod passphrase;
pub mod reed_solomon;

// 重新导出 trait（方便用户直接 use）
pub use algorithms::{HashAlgorithm, SignatureAlgorithm, GcmAlgorithm};

// 重新导出核心类型
pub use crypto::Crypto;
pub use keypair::KeyPair;
pub use config::CryptoConfig;

// 重新导出具体算法（供选择和测试）
pub use algorithms::{Ed25519, Curve25519, Sm2, Sha256, Sm3, Sm4Gcm};

// 错误类型与结果
use thiserror::Error;

/// 加密错误类型
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("invalid public key length: {0}")]
    InvalidPublicKey(usize),

    #[error("invalid signature length: {0}")]
    InvalidSignature(usize),

    #[error("signature verification failed")]
    VerificationFailed,

    #[error("key generation error: {0}")]
    KeyGeneration(String),

    // 国密算法错误
    #[error("SM2 error: {0}")]
    Sm2Error(String),

    #[error("SM3 error: {0}")]
    Sm3Error(String),

    #[error("SM4 error: {0}")]
    Sm4Error(String),

    #[error("configuration error: {0}")]
    ConfigurationError(String),

    #[error("cipher error: {0}")]
    CipherError(String),
}

pub type CryptoResult<T> = std::result::Result<T, CryptoError>;

// ============================================================================
// 原有 API 保持一致（通过内部模块转发）
// ============================================================================

// Hash 函数
pub use crypto::{blake3, hash, sha256, sm3};

// Signature 函数
pub use crypto::{generate_keypair, keypair_from_seed, sign, verify};

// CLI 工具函数
pub use crypto::{
    generate_keypair_from_passphrase, derive_account_id,
    derive_public_key, validate_account_address, sign_transaction_bytes,
};

// Passphrase 模块函数
pub use passphrase::{
    generate_passphrase, passphrase_to_keypair, validate_passphrase,
    secret_to_number, number_to_secret, is_12_words_secret,
    NRCS_WORDS, WORD_COUNT,
};

// 其他工具函数
pub use crypto::{random_32, zeroize_keypair};
