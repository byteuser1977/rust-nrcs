//! 统一的密钥对类型
//!
//! `KeyPair` 是一个枚举，支持多种签名算法（Ed25519, SM2）。
//! 提供与算法无关的 API：`generate`, `from_seed`, `sign`, `verifying_key`, `secret_key`.
//!
//! 注意：`from_seed` 目前仅支持 Ed25519（为了向后兼容）。
//! SM2 种子派生应使用 `Crypto::keypair_from_seed` 或具体的 `Sm2::from_seed`.

use super::{
    config::CryptoConfig,
    PublicKey, SecretKey, Signature,
};
use ed25519_dalek::{self, SigningKey, VerifyingKey, Signer};

/// 统一密钥对类型
///
/// 内部持有具体算法的密钥实现，对外提供一致的接口。
#[derive(Debug, Clone)]
pub enum KeyPair {
    /// Ed25519 密钥对
    Ed25519(SigningKey),
}

impl KeyPair {
    /// 根据全局配置生成密钥对
    ///
    /// 读取 `CryptoConfig::global().signature` 决定算法。
    /// 常用：`KeyPair::generate()`
    pub fn generate() -> Self {
        let cfg = CryptoConfig::global();
        match cfg.signature.as_str() {
            "ed25519" => {
                let kp = SigningKey::generate(&mut rand::thread_rng());
                Self::Ed25519(kp)
            }
            _ => panic!("unsupported signature algorithm: {}", cfg.signature),
        }
    }

    /// 从 32 字节种子生成密钥对（仅 Ed25519）
    ///
    /// 保持与原 API 兼容，始终使用 Ed25519。
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let kp = SigningKey::from_bytes(seed);
        Self::Ed25519(kp)
    }

    /// 获取验证密钥（公钥）
    pub fn verifying_key(&self) -> VerifyingKey {
        match self {
            KeyPair::Ed25519(kp) => kp.verifying_key(),
        }
    }

    /// 获取公钥（兼容旧 API）
    pub fn public_key(&self) -> PublicKey {
        match self {
            KeyPair::Ed25519(kp) => {
                let bytes = kp.verifying_key().to_bytes();
                PublicKey::Ed25519(bytes)
            }
        }
    }

    /// 获取私钥
    pub fn secret_key(&self) -> SecretKey {
        match self {
            KeyPair::Ed25519(kp) => {
                let bytes = kp.to_bytes();
                let mut arr = [0u8; 64];
                arr.copy_from_slice(&bytes);
                SecretKey::Ed25519(arr)
            }
        }
    }

    /// 签名消息
    pub fn sign(&self, message: &[u8]) -> Signature {
        match self {
            KeyPair::Ed25519(kp) => kp.sign(message).to_bytes(),
        }
    }

    /// 获取算法名称
    pub fn algorithm_name(&self) -> &'static str {
        match self {
            KeyPair::Ed25519(_) => "ed25519",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generate_ed25519() {
        let kp = KeyPair::Ed25519(SigningKey::generate(&mut rand::thread_rng()));
        assert!(matches!(kp, KeyPair::Ed25519(_)));
        let pk = kp.public_key();
        assert!(matches!(pk, PublicKey::Ed25519(_)));
        let sk = kp.secret_key();
        assert!(matches!(sk, SecretKey::Ed25519(_)));
    }

    #[test]
    fn test_keypair_sign_verify() {
        // Ed25519
        let kp_ed = KeyPair::Ed25519(SigningKey::generate(&mut rand::thread_rng()));
        let msg = b"test";
        let sig = kp_ed.sign(msg);
        let pk_ed = kp_ed.public_key();
        assert!(crate::verify(&pk_ed, msg, &sig).is_ok());
    }
}
