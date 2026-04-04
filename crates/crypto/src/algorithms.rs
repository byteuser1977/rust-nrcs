//! 加密算法抽象层
//!
//! 定义统一的算法 trait，允许在运行时根据配置动态选择具体实现。
//! 设计目标：
//! - 可插拔：各算法实现独立，互不依赖
//! - 向后兼容：保持原有 API 不变
//! - 配置驱动：通过配置文件选择算法

use crate::{CryptoError, CryptoResult, Hash256, PublicKey, SecretKey, Signature, keypair::KeyPair};

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
