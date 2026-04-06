//! # Cryptography Module
//!
//! 加密原语模块，提供区块链所需的核心加密功能：
//! - 数字签名（Ed25519）
//! - 哈希算法（SHA-256, BLAKE3, SM3）
//! - **可插拔算法抽象层**：通过配置文件动态选择算法
//!
//! ## 架构
//! - `algorithms.rs`: 定义核心 trait (HashAlgorithm, SignatureAlgorithm)
//! - `impls/`: 各具体算法实现 (Sha256, Sm3, Ed25519)
//! - `config.rs`: 配置管理（从 `config/default.toml` 读取）
//! - `keypair.rs`: 统一密钥对类型（枚举，支持多算法）
//! - `crypto.rs`: `Crypto` 结构体（根据配置组合算法）、向后兼容的便捷函数
//!
//! ## 使用示例
//!
//! ### 便捷函数（兼容原有 API）
//! ```
//! use crypto::{hash, verify, generate_keypair, sha256, sm3};
//!
//! let data = b"hello world";
//! let hash = hash(data);
//! let specific = sha256(data);
//!
//! let kp = generate_keypair();
//! let sig = sign(&kp.secret_key(), data);
//! assert!(verify(&kp.public_key(), data, &sig).is_ok());
//! ```
//!
//! ### 直接使用 `Crypto` 结构体（显式控制）
//! ```
//! use crypto::{Crypto, config::CryptoConfig};
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
//! signature = "ed25519"  # "ed25519"
//! ```
//!
//! 默认配置：Ed25519 + SHA-256

/// 固定大小的 SHA-256 哈希（32 字节）
pub type Hash256 = [u8; 32];

/// 签名类型（64 字节）
pub type Signature = [u8; 64];

/// 公钥类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PublicKey {
    /// Ed25519 公钥（32 字节）
    Ed25519([u8; 32]),
}

impl PublicKey {
    /// 获取公钥长度（字节）
    pub fn len(&self) -> usize {
        match self {
            PublicKey::Ed25519(bytes) => bytes.len(),
        }
    }

    /// 转换为字节 slice
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            PublicKey::Ed25519(bytes) => bytes,
        }
    }
}

/// 私钥类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SecretKey {
    /// Ed25519 私钥（64 字节）
    Ed25519([u8; 64]),
}

impl SecretKey {
    /// 获取私钥长度
    pub fn len(&self) -> usize {
        match self {
            SecretKey::Ed25519(bytes) => bytes.len(),
        }
    }

    /// 转换为字节 slice
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            SecretKey::Ed25519(bytes) => bytes,
        }
    }
}

// 模块声明
mod algorithms;
mod config;
mod crypto;
mod impls;
mod keypair;

// 重新导出算法 trait（方便用户自定义算法）
pub use algorithms::{
    HashAlgorithm, SignatureAlgorithm,
};

// 重新导出具体算法（供选择和测试）
pub use impls::{
    Ed25519, Sha256, Sm3,
};

// 导出配置类型
pub use config::CryptoConfig;

// 导出统一服务
pub use crypto::Crypto;

// 导出核心类型
pub use keypair::KeyPair;

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

// 其他工具函数
pub use crypto::{random_32, zeroize_keypair};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let crypto = Crypto::global().unwrap();
        assert_eq!(crypto.hash_algorithm_name(), "sha256");
        assert_eq!(crypto.signature_algorithm_name(), "ed25519");
    }

    #[test]
    fn test_hash_compatibility() {
        let data = b"compatibility test";
        let h1 = hash(data);
        let crypto = Crypto::global().unwrap();
        let h2 = crypto.hash(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_signature_compatibility() {
        let kp = generate_keypair();
        let msg = b"test";
        let sig = sign(&kp.secret_key(), msg);
        assert!(verify(&kp.public_key(), msg, &sig).is_ok());
    }

    #[test]
    fn test_old_sha256_still_works() {
        let data = b"test";
        let hash = sha256(data);
        // 验证 SHA-256 正确性
        let mut expected = [0u8; 32];
        expected.copy_from_slice(&[
            0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14,
            0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24,
            0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c,
            0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55,
        ]);
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_config_switching_hash() {
        // 测试切换到 SM3
        let cfg_sm3 = CryptoConfig { hash: "sm3".into(), ..Default::default() };
        let crypto_sm3 = Crypto::new(&cfg_sm3).unwrap();
        let data = b"hello";
        let hash_sm3 = crypto_sm3.hash(data);
        let expected = sm3(data);
        assert_eq!(hash_sm3, expected);

        // 回到 SHA256
        let cfg_sha = CryptoConfig { hash: "sha256".into(), ..Default::default() };
        let crypto_sha = Crypto::new(&cfg_sha).unwrap();
        let hash_sha = crypto_sha.hash(data);
        let expected = sha256(data);
        assert_eq!(hash_sha, expected);
    }
}
