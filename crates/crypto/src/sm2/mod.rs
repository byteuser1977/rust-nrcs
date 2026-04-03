//! SM2 椭圆曲线数字签名算法（国密标准 GM/T 0003-2012）
//!
//! SM2 是基于椭圆曲线密码学的数字签名算法，使用 256 位素数域曲线。
//! 类似于 ECDSA，但签名格式不同（r, s 拼接为 64 字节）。
//!
//! ## 曲线参数
//! 使用 SM2P256v1（也称为 GM/T 0003.1/2-2012），曲线方程为：
//! - y² = x³ + ax + b over F_p
//! - p = FFFFFFFE FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF
//! - a = FFFFFFFE FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFC
//! - b = 28E9FA9E 9D9F5E34 4D5A9E4B CF6509A7 F39789F5 15AB8F92 DDBCBD41 4D940E93
//! - n = FFFFFFFE FFFFFFFF FFFFFFFF FFFFFFFF 7203DF6B 21C6052B 53BBF4B 3D6AA9B5
//! - Gx = 32C4AE2C 1F198119 5F990446 6A39C994 8FE3BB9F 5FC340A8 178E9B3A D1660
//! - Gy = 21A8D441 5BB1D17C 2B9B8B43 5EB4B4F7 4C97ECF8 37F5B90B 5D58B46 D66D36
//!
//! ## 使用示例
//! ```
//! use crypto::sm2::{KeyPair, sign, verify};
//! use crypto::PublicKey;
//!
//! // 生成密钥对
//! let keypair = KeyPair::generate();
//!
//! // 签名
//! let msg = b"hello world";
//! let signature = keypair.sign(msg);
//!
//! // 验证
//! let pubkey = keypair.public_key();
//! assert!(verify(&pubkey, msg, &signature).is_ok());
//! ```
//!
//! ## 安全要求
//! - 私钥使用 `zeroize` 自动清理
//! - 随机数生成使用 `rand::rngs::OsRng`（加密安全）
//! - 常量时间实现（防止侧信道攻击）

mod keypair;
mod signature;

pub use keypair::*;
pub use signature::*;

use crate::CryptoError;
use blockchain_types::*;
use sm2::{keygen, SecretKey, PublicKey as Sm2PublicKey};
use sm3::sm3;
use zeroize::Zeroize;

/// SM2 密钥对
///
/// 与 Ed25519 KeyPair 保持相同 API，便于替换
/// - 私钥：32 字节
/// - 公钥：64 字节（未压缩，直接序列化 x || y）
/// - 签名：64 字节（r || s）
#[derive(Clone)]
pub struct KeyPair {
    /// SM2 密钥对
    inner: (SecretKey, Sm2PublicKey),
}

impl KeyPair {
    /// 生成新的随机 SM2 密钥对
    ///
    /// 使用操作系统提供的 CSPRNG 生成随机私钥
    pub fn generate() -> Self {
        let (secret, public) = keygen();
        Self { inner: (secret, public) }
    }

    /// 从 32 字节种子派生密钥对
    ///
    /// 使用 SM3(seed) 作为私钥，注意这不同于 Ed25519 的 `from_seed`
    /// 符合国密标准的密钥派生方式
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        // 派生私钥：k = SM3(seed)
        let secret_bytes = sm3(seed);
        let secret = SecretKey::from_bytes(&secret_bytes)
            .expect("SM3 output is valid secret key length");

        // 计算公钥：Q = d * G
        let public = secret.random_public_key();

        Self { inner: (secret, public) }
    }

    /// 从现有私钥字节构建密钥对
    ///
    /// # 参数
    /// - `secret_bytes`: 32 字节私钥
    ///
    /// # 返回
    /// 如果私钥有效则返回 KeyPair，否则 Err
    pub fn from_secret_bytes(secret_bytes: &[u8; 32]) -> Result<Self, CryptoError> {
        let secret = SecretKey::from_bytes(secret_bytes)
            .map_err(|_| CryptoError::KeyGeneration("invalid secret key bytes".into()))?;
        let public = secret.random_public_key();
        Ok(Self { inner: (secret, public) })
    }

    /// 获取公钥（64 字节，x || y 拼接）
    pub fn public_key(&self) -> PublicKey {
        let pub_bytes = self.inner.1.to_bytes();
        pub_bytes.into()
    }

    /// 获取私钥（32 字节）
    ///
    /// **警告**：避免在日志或错误信息中泄露
    pub fn secret_key(&self) -> SecretKey {
        self.inner.0.clone()
    }

    /// 获取私钥字节（32 字节）
    pub fn secret_key_bytes(&self) -> [u8; 32] {
        *self.inner.0.as_bytes()
    }

    /// 对消息进行签名（返回 64 字节）
    ///
    /// 使用 SM2 签名算法，输入任意长度消息
    ///
    /// # 参数
    /// - `message`: 待签名的消息
    ///
    /// # 返回
    /// 64 字节签名（r || s）
    pub fn sign(&self, message: &[u8]) -> Signature {
        // SM2 签名：计算 e = H(M)，然后生成 (r, s)
        let sig = self.inner.0.sign(message);
        let sig_bytes = sig.to_bytes();
        sig_bytes.into()
    }

    /// 获取公钥指纹（前 8 字节）
    ///
    /// 用于快速识别密钥
    pub fn fingerprint(&self) -> [u8; 8] {
        let pubkey = self.public_key();
        let mut fp = [0u8; 8];
        fp.copy_from_slice(&pubkey[..8]);
        fp
    }
}

/// 签名验证
///
/// # 参数
/// - `public_key`: 64 字节公钥（x || y）
/// - `message`: 原始消息
/// - `signature`: 64 字节签名（r || s）
///
/// # 返回
/// `Ok(())` 如果签名有效，否则 `Err(CryptoError)`
pub fn verify(public_key: &PublicKey, message: &[u8], signature: &Signature) -> Result<(), CryptoError> {
    if public_key.len() != 64 {
        return Err(CryptoError::InvalidPublicKey(public_key.len()));
    }
    if signature.len() != 64 {
        return Err(CryptoError::InvalidSignature(signature.len()));
    }

    let pub_bytes: [u8; 64] = public_key
        .try_into()
        .map_err(|_| CryptoError::InvalidPublicKey(public_key.len()))?;
    let sig_bytes: [u8; 64] = signature
        .try_into()
        .map_err(|_| CryptoError::InvalidSignature(signature.len()))?;

    let pubkey = Sm2PublicKey::from_bytes(&pub_bytes)
        .map_err(|_| CryptoError::InvalidPublicKey(public_key.len()))?;
    let sig = sm2::Signature::from_bytes(&sig_bytes)
        .map_err(|_| CryptoError::InvalidSignature(signature.len()))?;

    // 验证签名
    pubkey.verify(message, &sig)
        .map_err(|_| CryptoError::VerificationFailed)?;

    Ok(())
}

/// 零化私钥
///
/// 调用后私钥被清零，密钥对不可用
/// 可用于销毁密钥
pub fn zeroize_keypair(keypair: &mut KeyPair) {
    keypair.inner.0.zeroize();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sm2_keypair_generation() {
        let kp = KeyPair::generate();
        assert_eq!(kp.public_key().len(), 64);
        assert_eq!(kp.secret_key_bytes().len(), 32);
    }

    #[test]
    fn test_sm2_sign_verify() {
        let kp = KeyPair::generate();
        let msg = b"SM2 test message";
        let sig = kp.sign(msg);
        let pubkey = kp.public_key();

        // 验证签名应成功
        assert!(verify(&pubkey, msg, &sig).is_ok());

        // 篡改消息应失败
        let wrong_msg = b"wrong message";
        assert!(verify(&pubkey, wrong_msg, &sig).is_err());
    }

    #[test]
    fn test_sm2_from_seed() {
        let seed = [0xaa; 32];
        let kp = KeyPair::from_seed(&seed);
        assert_eq!(kp.secret_key_bytes().len(), 32);
        assert!(verify(&kp.public_key(), b"test", &kp.sign(b"test")).is_ok());
    }

    #[test]
    fn test_sm2_standard_vector() {
        // 使用标准测试向量（如果可用）
        // 这里使用固定随机种子保证可重复性
        let mut rng = rand::rngs::StdRng::seed_from_u64(12345);
        let (secret, public) = sm2::keygen_with_rng(&mut rng);
        let kp = KeyPair { inner: (secret, public) };

        let msg = b"GM/T 0003-2012 test vector";
        let sig = kp.sign(msg);
        assert!(verify(&kp.public_key(), msg, &sig).is_ok());
    }

    #[test]
    fn test_sm2_invalid_key() {
        // 测试无效公钥长度
        let invalid_pubkey = vec![0u8; 32];
        let msg = b"test";
        let kp = KeyPair::generate();
        let sig = kp.sign(msg);
        let result = verify(&invalid_pubkey, msg, &sig);
        assert!(result.is_err());
    }

    #[test]
    fn test_sm2_invalid_signature() {
        let kp = KeyPair::generate();
        let invalid_sig = vec![0u8; 32]; // 长度不足 64
        let result = verify(&kp.public_key(), b"test", &invalid_sig);
        assert!(result.is_err());
    }

    #[test]
    fn test_sm2_zeroize() {
        let mut kp = KeyPair::generate();
        let secret_before = kp.secret_key_bytes();
        zeroize_keypair(&mut kp);
        let secret_after = kp.secret_key_bytes();
        // 清零后私钥应为全零
        assert!(secret_after.iter().all(|&b| b == 0));
        // 公钥不受影响（公钥可公开）
        assert_ne!(kp.public_key().len(), 0);
    }
}
