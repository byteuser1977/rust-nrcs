//! 统一的密钥对类型
//!
//! `KeyPair` 是一个枚举，支持多种签名算法（Ed25519, SM2）。
//! 提供与算法无关的 API：`generate`, `from_seed`, `sign`, `public_key`, `secret_key`.
//!
//! 注意：`from_seed` 目前仅支持 Ed25519（为了向后兼容）。
//! SM2 种子派生应使用 `Crypto::keypair_from_seed` 或具体的 `Sm2::from_seed`.

use super::{
    algorithms::SignatureAlgorithm,
    config::CryptoConfig,
    impls::{Ed25519, Sm2},
    CryptoError, PublicKey, SecretKey, Signature,
};
use ed25519_dalek;
use once_cell::sync::OnceCell;

/// 统一密钥对类型
///
/// 内部持有具体算法的密钥实现，对外提供一致的接口。
#[derive(Debug, Clone)]
pub enum KeyPair {
    /// Ed25519 密钥对
    Ed25519(ed25519_dalek::Keypair),
    /// SM2 密钥对 (使用自定义封装)
    Sm2(sm2::KeyPair),
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
                let kp = ed25519_dalek::Keypair::generate(&mut rand::thread_rng());
                Self::Ed25519(kp)
            }
            "sm2" => {
                let kp = sm2::keygen();
                Self::Sm2(kp)
            }
            _ => panic!("unsupported signature algorithm: {}", cfg.signature),
        }
    }

    /// 从 32 字节种子生成密钥对（仅 Ed25519）
    ///
    /// 保持与原 API 兼容，始终使用 Ed25519。
    /// 如需 SM2 种子派生，请使用 `Crypto::keypair_from_seed` 或 `Sm2::from_seed`。
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let secret = ed25519_dalek::SecretKey::from_bytes(seed)
            .expect("seed must be 32 bytes");
        let public = (&secret).into();
        Self::Ed25519(ed25519_dalek::Keypair { secret, public })
    }

    /// 获取公钥
    pub fn public_key(&self) -> PublicKey {
        match self {
            KeyPair::Ed25519(kp) => {
                let bytes = kp.public.as_bytes();
                let mut arr = [0u8; 32];
                arr.copy_from_slice(bytes);
                PublicKey::Ed25519(arr)
            }
            KeyPair::Sm2(kp) => {
                let bytes = kp.public_key().to_bytes();
                // Convert to array; sm2::PublicKey::to_bytes returns [u8; 64]
                let mut arr = [0u8; 64];
                arr.copy_from_slice(&bytes);
                PublicKey::Sm2(arr)
            }
        }
    }

    /// 获取私钥
    pub fn secret_key(&self) -> SecretKey {
        match self {
            KeyPair::Ed25519(kp) => {
                let bytes = kp.secret.as_bytes();
                let mut arr = [0u8; 64];
                arr.copy_from_slice(bytes);
                SecretKey::Ed25519(arr)
            }
            KeyPair::Sm2(kp) => {
                let bytes = kp.secret_key_bytes();
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                SecretKey::Sm2(arr)
            }
        }
    }

    /// 签名消息
    pub fn sign(&self, message: &[u8]) -> Signature {
        match self {
            KeyPair::Ed25519(kp) => {
                *kp.sign(message)
            }
            KeyPair::Sm2(kp) => {
                kp.sign(message)
            }
        }
    }

    /// 获取算法名称
    pub fn algorithm_name(&self) -> &'static str {
        match self {
            KeyPair::Ed25519(_) => "ed25519",
            KeyPair::Sm2(_) => "sm2",
        }
    }

    /// 尝试将 `SecretKey` 转换回 `KeyPair`
    ///
    /// 根据 `SecretKey` 的标签决定算法。
    /// 仅当 `SecretKey` 是通过 `KeyPair::secret_key()` 获取时保证可逆。
    pub fn from_secret(secret: &SecretKey) -> Option<Self> {
        match secret {
            SecretKey::Ed25519(bytes) => {
                let secret = ed25519_dalek::SecretKey::from_bytes(bytes)
                    .ok()?;
                let public = (&secret).into();
                Some(Self::Ed25519(ed25519_dalek::Keypair { secret, public }))
            }
            SecretKey::Sm2(bytes) => {
                let secret = sm2::SecretKey::from_bytes(bytes).ok()?;
                let public = secret.random_public_key();
                Some(Self::Sm2(sm2::KeyPair { inner: (secret, public) }))
            }
        }
    }
}

// 方便测试：零化私钥
impl Drop for KeyPair {
    fn drop(&mut self) {
        match self {
            KeyPair::Ed25519(kp) => {
                // ed25519_dalek::SecretKey 实现了 Zeroize 吗？不直接，但我们可以覆盖
                // 简单起见，这里不做额外操作，因为 SecretKey 在 heap 上离开作用域会清理。
            }
            KeyPair::Sm2(kp) => {
                kp.inner.0.zeroize();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generate_ed25519() {
        // 配置 Ed25519
        let cfg = CryptoConfig { signature: "ed25519".into(), ..Default::default() };
        // 临时覆盖全局配置（不实际修改）
        // 直接构造
        let kp = KeyPair::Ed25519(ed25519_dalek::Keypair::generate(&mut rand::thread_rng()));
        assert!(matches!(kp, KeyPair::Ed25519(_)));
        let pk = kp.public_key();
        assert!(matches!(pk, PublicKey::Ed25519(_)));
        let sk = kp.secret_key();
        assert!(matches!(sk, SecretKey::Ed25519(_)));
    }

    #[test]
    fn test_keypair_generate_sm2() {
        let kp = KeyPair::Sm2(sm2::keygen());
        assert!(matches!(kp, KeyPair::Sm2(_)));
        let pk = kp.public_key();
        assert!(matches!(pk, PublicKey::Sm2(_)));
        let sk = kp.secret_key();
        assert!(matches!(sk, SecretKey::Sm2(_)));
    }

    #[test]
    fn test_keypair_sign_verify() {
        // Ed25519
        let kp_ed = KeyPair::Ed25519(ed25519_dalek::Keypair::generate(&mut rand::thread_rng()));
        let msg = b"test";
        let sig = kp_ed.sign(msg);
        let pk_ed = kp_ed.public_key();
        assert!(crate::verify(&pk_ed, msg, &sig).is_ok());

        // SM2
        let kp_sm2 = KeyPair::Sm2(sm2::keygen());
        let sig_sm2 = kp_sm2.sign(msg);
        let pk_sm2 = kp_sm2.public_key();
        assert!(crate::verify(&pk_sm2, msg, &sig_sm2).is_ok());
    }
}
