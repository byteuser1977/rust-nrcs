//! 算法实现模块
//!
//! 提供多种加密算法的实现，包括：
//! - 哈希算法：SHA-256, SM3
//! - 签名算法：Ed25519, Curve25519, SM2
//! - 加密算法：SM4, SM4-GCM

use crate::{CryptoResult, Hash256, PublicKey, SecretKey, Signature, keypair::KeyPair};

/// 哈希算法 trait
///
/// 所有哈希算法输出固定 256 位（32 字节）
pub trait HashAlgorithm: Send + Sync + std::fmt::Debug + 'static {
    /// 计算哈希值
    fn hash(&self, data: &[u8]) -> Hash256;
    /// 获取算法名称（用于配置和调试）
    fn name(&self) -> &'static str;
}

/// 签名算法 trait
///
/// 支持密钥对生成、签名和验证
pub trait SignatureAlgorithm: Send + Sync + std::fmt::Debug + 'static {
    /// 生成新的随机密钥对
    fn generate_keypair(&self) -> KeyPair;

    /// 从 32 字节种子派生密钥对（用于确定性生成）
    fn from_seed(&self, seed: &[u8; 32]) -> KeyPair;

    /// 签名消息
    ///
    /// # 参数
    /// - `key`: 私钥（32 或 64 字节，取决于算法）
    /// - `message`: 待签名消息
    ///
    /// # 返回
    /// 签名字节（长度由算法决定，通常 64 字节）
    fn sign(&self, key: &SecretKey, message: &[u8]) -> Signature;

    /// 验证签名
    ///
    /// # 参数
    /// - `public_key`: 公钥
    /// - `message`: 原始消息
    /// - `signature`: 签名
    ///
    /// # 返回
    /// `Ok(())` 验证通过，`Err` 验证失败或参数错误
    fn verify(&self, public_key: &PublicKey, message: &[u8], signature: &Signature)
        -> CryptoResult<()>;

    /// 获取算法名称（用于配置和调试）
    fn name(&self) -> &'static str;
}

/// 对称加密算法（CBC 模式）
///
/// 提供块加密的加密和解密操作
pub trait CipherAlgorithm: Send + Sync + std::fmt::Debug + 'static {
    /// 加密（返回 iv || ciphertext）
    fn encrypt_cbc(&self, key: &[u8], iv: &[u8], plaintext: &[u8]) -> CryptoResult<Vec<u8>>;
    /// 解密（输入需包含 iv）
    fn decrypt_cbc(&self, key: &[u8], iv_ciphertext: &[u8]) -> CryptoResult<Vec<u8>>;
    /// 算法名称
    fn name(&self) -> &'static str;
    /// 密钥长度（字节）
    fn key_len(&self) -> usize;
}

/// 认证加密算法（GCM 模式）
///
/// 提供机密性+完整性的 AEAD 操作
pub trait GcmAlgorithm: Send + Sync + std::fmt::Debug + 'static {
    /// 加密（返回 ciphertext, tag）
    fn encrypt_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        aad: &[u8],
        plaintext: &[u8],
    ) -> CryptoResult<(Vec<u8>, Vec<u8>)>;
    /// 解密（接受独立的 ciphertext 和 tag）
    fn decrypt_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        aad: &[u8],
        ciphertext: &[u8],
        tag: &[u8],
    ) -> CryptoResult<Vec<u8>>;
    /// 算法名称
    fn name(&self) -> &'static str;
    /// 密钥长度（字节）
    fn key_len(&self) -> usize;
    /// nonce/IV 长度（字节）
    fn nonce_len(&self) -> usize;
    /// 认证标签长度（字节）
    fn tag_len(&self) -> usize;
}

pub mod hash;
pub mod signature;
pub mod cipher;

// 重新导出哈希算法
pub use hash::{Sha256, Sm3};

// 重新导出签名算法
pub use signature::{Ed25519, Curve25519};

// 重新导出加密算法
pub use cipher::Sm4Gcm;
