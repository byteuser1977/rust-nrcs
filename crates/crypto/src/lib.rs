//! # Cryptography Module
//!
//! 加密原语模块，提供区块链所需的核心加密功能：
//! - 数字签名（Ed25519）
//! - 哈希算法（SHA-256, BLAKE3）
//! - 密钥派生
//! - 安全内存管理
//!
//! ## 使用示例
//! ```
//! use crypto::{KeyPair, sign, verify};
//! use blockchain_types::PublicKey;
//!
//! let kp = KeyPair::generate();
//! let msg = b"hello world";
//! let sig = kp.sign(msg);
//! let pubkey: PublicKey = kp.public_key().into();
//! assert!(verify(&pubkey, msg, &sig).is_ok());
//! ```

mod hash;
mod keypair;
mod signature;

pub use hash::*;
pub use keypair::*;
pub use signature::*;

use blockchain_types::*;
use ed25519_dalek::{Signature as EdSignature, Signer, Verifier};
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
}

pub type CryptoResult<T> = std::result::Result<T, CryptoError>;

/// 密钥对（私钥 + 公钥）
///
/// 使用 Ed25519 算法（Curve25519）
///
/// 注意：SecretKey 包含 32 字节种子，公钥 32 字节
/// 为了安全性，建议使用 `zeroize` 自动清理内存
#[derive(Clone)]
pub struct KeyPair {
    /// Ed25519 密钥对
    inner: ed25519_dalek::Keypair,
}

impl KeyPair {
    /// 生成新的随机密钥对
    pub fn generate() -> Self {
        let secret = ed25519_dalek::SecretKey::generate(&mut rand::thread_rng());
        let public = (&secret).into();
        Self {
            inner: ed25519_dalek::Keypair { secret, public },
        }
    }

    /// 从 32 字节种子生成密钥对（BIP-39 风格）
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let secret = ed25519_dalek::SecretKey::from_bytes(seed)
            .expect("seed must be 32 bytes");
        let public = (&secret).into();
        Self {
            inner: ed25519_dalek::Keypair { secret, public },
        }
    }

    /// 获取公钥（32 字节）
    pub fn public_key(&self) -> PublicKey {
        (*self.inner.public.as_bytes()).into()
    }

    /// 获取私钥（64 字节，secret key + public key）
    /// **注意**：生产环境避免暴露私钥
    pub fn secret_key(&self) -> SecretKey {
        self.inner.secret.as_bytes().into()
    }

    /// 签名消息（返回 64 字节 Ed25519 签名）
    pub fn sign(&self, message: &[u8]) -> Signature {
        *self.inner.sign(message)
    }
}

/// 签名验证（静态方法）
pub fn verify(public_key: &PublicKey, message: &[u8], signature: &Signature) -> CryptoResult<()> {
    let pk = ed25519_dalek::PublicKey::from_bytes(public_key)
        .map_err(|_| CryptoError::InvalidPublicKey(public_key.len()))?;
    let sig = EdSignature::from_bytes(signature)
        .map_err(|_| CryptoError::InvalidSignature(signature.len()))?;

    pk.verify(message, &sig)
        .map_err(|_| CryptoError::VerificationFailed)?;
    Ok(())
}

/// 计算 SHA-256 哈希
pub fn sha256(data: &[u8]) -> Hash256 {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// 计算 BLAKE3 哈希（速度更快）
pub fn blake3(data: &[u8]) -> Hash256 {
    use blake3::Hasher;
    let mut hasher = blake3::Hasher::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.into()
}

/// 双 SHA-256 哈希（用于 PoW）
pub fn sha256d(data: &[u8]) -> Hash256 {
    let first = sha256(data);
    sha256(&first)
}

/// 生成随机 32 字节（用于 nonce、密钥等）
pub fn random_32() -> [u8; 32] {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill(&mut buf);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let kp = KeyPair::generate();
        assert_eq!(kp.public_key().len(), 32);
        assert_eq!(kp.secret_key().len(), 64);
    }

    #[test]
    fn test_sign_verify() {
        let kp = KeyPair::generate();
        let msg = b"test message";
        let sig = kp.sign(msg);
        let pubkey = kp.public_key();
        assert!(verify(&pubkey, msg, &sig).is_ok());

        // 篡改消息应失败
        let wrong_msg = b"wrong message";
        assert!(verify(&pubkey, wrong_msg, &sig).is_err());
    }

    #[test]
    fn test_hash_functions() {
        let data = b"hello world";
        let hash1 = sha256(data);
        let hash2 = blake3(data);
        assert_ne!(hash1, hash2); // 不同算法，不同输出
        assert_eq!(hash1.len(), 32);
        assert_eq!(hash2.len(), 32);
    }

    #[test]
    fn test_sha256d() {
        let data = b"test";
        let double = sha256d(data);
        let single = sha256(data);
        let double_manual = sha256(&single);
        assert_eq!(double, double_manual);
    }
}
