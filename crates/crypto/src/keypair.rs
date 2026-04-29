//! 统一的密钥对类型
//!
//! `KeyPair` 是一个枚举，支持多种签名算法（Ed25519, Curve25519, SM2）。
//! 提供与算法无关的 API：`generate`, `from_seed`, `sign`, `verifying_key`, `secret_key`.

use super::{
    algorithms::SignatureAlgorithm,
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
    /// Curve25519 密钥对（NRCS 兼容）
    Curve25519 {
        public_key: [u8; 32],
        secret_key: [u8; 32],
    },
    /// SM2 密钥对（国密）
    Sm2 {
        public_key: [u8; 65],
        secret_key: [u8; 32],
    },
}

impl KeyPair {
    /// 根据全局配置生成密钥对
    ///
    /// 读取 `CryptoConfig::global().signature` 决定算法。
    pub fn generate() -> Self {
        let cfg = CryptoConfig::global();
        match cfg.signature.as_str() {
            "ed25519" => {
                let kp = SigningKey::generate(&mut rand::thread_rng());
                Self::Ed25519(kp)
            }
            "curve25519" => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let mut seed = [0u8; 32];
                rng.fill(&mut seed);

                let public_key = crate::algorithms::Curve25519::derive_public_key(&seed);

                Self::Curve25519 {
                    public_key,
                    secret_key: seed,
                }
            }
            "sm2" => {
                crate::algorithms::Sm2.generate_keypair()
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

    /// 获取验证密钥（公钥）- 仅 Ed25519
    pub fn verifying_key(&self) -> VerifyingKey {
        match self {
            KeyPair::Ed25519(kp) => kp.verifying_key(),
            KeyPair::Curve25519 { .. } => {
                panic!("Curve25519 does not support verifying_key(), use public_key() instead");
            }
            KeyPair::Sm2 { .. } => {
                panic!("SM2 does not support verifying_key(), use public_key() instead");
            }
        }
    }

    /// 获取公钥
    pub fn public_key(&self) -> PublicKey {
        match self {
            KeyPair::Ed25519(kp) => {
                let bytes = kp.verifying_key().to_bytes();
                PublicKey::Ed25519(bytes)
            }
            KeyPair::Curve25519 { public_key, .. } => {
                PublicKey::Curve25519(*public_key)
            }
            KeyPair::Sm2 { public_key, .. } => {
                PublicKey::Sm2 {
                    public_key: *public_key,
                    distid: None,
                }
            }
        }
    }

    /// 获取私钥
    pub fn secret_key(&self) -> SecretKey {
        match self {
            KeyPair::Ed25519(kp) => {
                let seed = kp.to_bytes();
                let pubkey = kp.verifying_key().to_bytes();
                let mut arr = [0u8; 64];
                arr[..32].copy_from_slice(&seed);
                arr[32..].copy_from_slice(&pubkey);
                SecretKey::Ed25519(arr)
            }
            KeyPair::Curve25519 { secret_key, .. } => {
                SecretKey::Curve25519(*secret_key)
            }
            KeyPair::Sm2 { secret_key, .. } => {
                SecretKey::Sm2 {
                    secret_key: *secret_key,
                    distid: None,
                }
            }
        }
    }

    /// 签名消息
    pub fn sign(&self, message: &[u8]) -> Signature {
        match self {
            KeyPair::Ed25519(kp) => kp.sign(message).to_bytes(),
            KeyPair::Curve25519 { secret_key, .. } => {
                let algo = crate::algorithms::Curve25519;
                algo.sign(&SecretKey::Curve25519(*secret_key), message)
            }
            KeyPair::Sm2 { secret_key, .. } => {
                let algo = crate::algorithms::Sm2;
                algo.sign(&SecretKey::Sm2 { secret_key: *secret_key, distid: None }, message)
            }
        }
    }

    /// 获取算法名称
    pub fn algorithm_name(&self) -> &'static str {
        match self {
            KeyPair::Ed25519(_) => "ed25519",
            KeyPair::Curve25519 { .. } => "curve25519",
            KeyPair::Sm2 { .. } => "sm2",
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
    fn test_keypair_generate_curve25519() {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut seed = [0u8; 32];
        rng.fill(&mut seed);

        let public_key = crate::algorithms::Curve25519::derive_public_key(&seed);
        let kp = KeyPair::Curve25519 {
            public_key,
            secret_key: seed,
        };

        assert!(matches!(kp, KeyPair::Curve25519 { .. }));
        let pk = kp.public_key();
        assert!(matches!(pk, PublicKey::Curve25519(_)));
        let sk = kp.secret_key();
        assert!(matches!(sk, SecretKey::Curve25519(_)));
    }

    #[test]
    fn test_keypair_sign_verify() {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut seed = [0u8; 32];
        rng.fill(&mut seed);

        let public_key = crate::algorithms::Curve25519::derive_public_key(&seed);
        let kp = KeyPair::Curve25519 {
            public_key,
            secret_key: seed,
        };
        let msg = b"test";
        let sig = kp.sign(msg);
        let pk = kp.public_key();
        assert!(crate::verify(&pk, msg, &sig).is_ok());
    }

    #[test]
    fn test_keypair_curve25519_sign_verify() {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut seed = [0u8; 32];
        rng.fill(&mut seed);

        let public_key = crate::algorithms::Curve25519::derive_public_key(&seed);
        let kp = KeyPair::Curve25519 {
            public_key,
            secret_key: seed,
        };

        let msg = b"test message for curve25519";
        let sig = kp.sign(msg);
        let pk = kp.public_key();

        let algo = crate::algorithms::Curve25519;
        assert!(algo.verify(&pk, msg, &sig).is_ok());
    }
}
