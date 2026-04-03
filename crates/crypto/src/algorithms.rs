//! 加密算法抽象层
//!
//! 定义统一的算法 trait，允许在运行时根据配置动态选择具体实现。
//! 设计目标：
//! - 可插拔：各算法实现独立，互不依赖
//! - 向后兼容：保持原有 API 不变
//! - 配置驱动：通过配置文件选择算法

use crate::{CryptoError, CryptoResult, Hash256, PublicKey, SecretKey, Signature};

/// 哈希算法 trait
///
/// 所有哈希算法输出固定 256 位（32 字节）
pub trait HashAlgorithm: Send + Sync + 'static {
    /// 计算哈希值
    fn hash(&self, data: &[u8]) -> Hash256;
}

/// 签名算法 trait
///
/// 支持密钥对生成、签名和验证
pub trait SignatureAlgorithm: Send + Sync + 'static {
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

/// 对称加密算法 trait（CBC 模式）
///
/// 接口风格参考 Java `Crypto.aesEncrypt`：
/// - 加密返回 `iv || ciphertext`
/// - 解密接受 `iv || ciphertext` 格式
pub trait CipherAlgorithm: Send + Sync + 'static {
    /// 加密（CBC 模式）
    ///
    /// # 参数
    /// - `key`: 密钥（长度由算法决定，如 16/24/32 字节）
    /// - `iv`: 初始化向量（必须是 16 字节）
    /// - `plaintext`: 明文（任意长度，会自动填充）
    ///
    /// # 返回
    /// `iv || ciphertext` 组合
    fn encrypt_cbc(&self, key: &[u8], iv: &[u8], plaintext: &[u8]) -> Vec<u8>;

    /// 解密（CBC 模式）
    ///
    /// # 参数
    /// - `key`: 密钥
    /// - `iv_ciphertext`: `iv || ciphertext` 格式的密文
    ///
    /// # 返回
    /// 解密后的明文（填充已移除）
    fn decrypt_cbc(&self, key: &[u8], iv_ciphertext: &[u8]) -> CryptoResult<Vec<u8>>;

    /// 获取算法名称
    fn name(&self) -> &'static str;

    /// 密钥长度（字节）
    fn key_len(&self) -> usize;

    /// 块大小（字节，通常 16）
    fn block_size(&self) -> usize {
        16
    }
}

/// GCM 模式 trait（认证加密）
///
/// 如果需要认证加密，实现此 trait
pub trait GcmAlgorithm: Send + Sync + 'static {
    /// GCM 加密
    ///
    /// # 参数
    /// - `key`: 密钥
    /// - `nonce`: 96 位（12 字节）nonce
    /// - `aad`: 附加认证数据
    /// - `plaintext`: 明文
    ///
    /// # 返回
    /// `(ciphertext, tag)` 元组，tag 通常 16 字节
    fn encrypt_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        aad: &[u8],
        plaintext: &[u8],
    ) -> CryptoResult<(Vec<u8>, Vec<u8>)>;

    /// GCM 解密
    ///
    /// # 参数
    /// - `key`: 密钥
    /// - `nonce`: nonce（必须与加密时相同）
    /// - `aad`: 附加认证数据（必须与加密时相同）
    /// - `ciphertext`: 密文
    /// - `tag`: 认证标签
    ///
    /// # 返回
    /// 解密后的明文，认证失败返回 `Err`
    fn decrypt_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        aad: &[u8],
        ciphertext: &[u8],
        tag: &[u8],
    ) -> CryptoResult<Vec<u8>>;

    /// 获取算法名称
    fn name(&self) -> &'static str;

    /// 密钥长度
    fn key_len(&self) -> usize;

    /// 推荐 nonce 长度
    fn nonce_len(&self) -> usize {
        12
    }

    /// 认证标签长度
    fn tag_len(&self) -> usize {
        16
    }
}
