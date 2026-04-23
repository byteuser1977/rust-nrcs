//! SM2 椭圆曲线数字签名算法（国密标准 GM/T 0003-2012）
//!
//! SM2 是基于椭圆曲线密码学的数字签名算法，使用 256 位素数域曲线。
//! 类似于 ECDSA，但签名格式不同（r, s 拼接为 64 字节）。
//!
//! ## 使用示例
//! ```
//! use crypto::algorithms::Sm2;
//! use crypto::algorithms::SignatureAlgorithm;
//!
//! // 生成密钥对
//! let keypair = Sm2.generate_keypair();
//!
//! // 签名
//! let msg = b"hello world";
//! let signature = Sm2.sign(&keypair.secret_key(), msg);
//!
//! // 验证
//! assert!(Sm2.verify(&keypair.public_key(), msg, &signature).is_ok());
//! ```

use crate::algorithms::SignatureAlgorithm;
use crate::{CryptoError, CryptoResult, KeyPair, PublicKey, SecretKey, Signature};
use sm2::dsa::{Signature as Sm2Signature, SigningKey, VerifyingKey};
use sm2::elliptic_curve::{Generate, sec1::ToSec1Point};
use sm2::{FieldBytes, SecretKey as Sm2SecretKey};

/// 默认的区分标识符（Distinguishing Identifier）
/// SM2DSA 要求一个用户标识符，通常使用用户 ID 或邮箱
const DEFAULT_DISTID: &str = "nrcs@blockchain.cn";

/// SM2 签名算法标记类型
#[derive(Debug, Clone, Copy)]
pub struct Sm2;

impl Sm2 {
    /// 从 32 字节种子派生密钥对
    pub fn derive_keypair_from_seed(seed: &[u8; 32]) -> CryptoResult<KeyPair> {
        let field_bytes: FieldBytes = (*seed).into();
        let secret = Sm2SecretKey::from_bytes(&field_bytes)
            .map_err(|e| CryptoError::KeyGeneration(format!("invalid seed: {}", e)))?;
        
        let signing_key = SigningKey::new(DEFAULT_DISTID, &secret)
            .map_err(|e| CryptoError::KeyGeneration(format!("failed to create signing key: {}", e)))?;
        
        let verifying_key = signing_key.verifying_key();
        let encoded_point = verifying_key.to_sec1_point(false);
        let public_key_bytes = encoded_point.as_bytes();
        
        let mut pk_bytes = [0u8; 65];
        pk_bytes.copy_from_slice(public_key_bytes);
        
        Ok(KeyPair::Sm2 {
            public_key: pk_bytes,
            secret_key: *seed,
        })
    }
}

impl SignatureAlgorithm for Sm2 {
    fn generate_keypair(&self) -> KeyPair {
        let secret = Sm2SecretKey::generate();
        let secret_bytes: [u8; 32] = secret.to_bytes().into();
        
        let signing_key = SigningKey::new(DEFAULT_DISTID, &secret)
            .expect("valid secret key");
        
        let verifying_key = signing_key.verifying_key();
        let encoded_point = verifying_key.to_sec1_point(false);
        let public_key_bytes = encoded_point.as_bytes();
        
        let mut pk_bytes = [0u8; 65];
        pk_bytes.copy_from_slice(public_key_bytes);
        
        KeyPair::Sm2 {
            public_key: pk_bytes,
            secret_key: secret_bytes,
        }
    }

    fn from_seed(&self, seed: &[u8; 32]) -> KeyPair {
        Self::derive_keypair_from_seed(seed)
            .expect("valid seed")
    }

    fn sign(&self, key: &SecretKey, message: &[u8]) -> Signature {
        match key {
            SecretKey::Sm2 { secret_key, distid } => {
                let field_bytes: FieldBytes = (*secret_key).into();
                let secret = Sm2SecretKey::from_bytes(&field_bytes)
                    .expect("valid secret key");
                
                let distid_str = distid.as_deref().unwrap_or(DEFAULT_DISTID);
                let signing_key = SigningKey::new(distid_str, &secret)
                    .expect("valid signing key");
                
                use sm2::dsa::signature::Signer;
                let sig: Sm2Signature = signing_key.sign(message);
                let sig_bytes = sig.to_bytes();
                
                let mut signature = [0u8; 64];
                signature.copy_from_slice(&sig_bytes);
                signature
            }
            _ => panic!("Invalid key type for SM2"),
        }
    }

    fn verify(&self, public_key: &PublicKey, message: &[u8], signature: &Signature) -> CryptoResult<()> {
        match public_key {
            PublicKey::Sm2 { public_key, distid } => {
                let verifying_key = VerifyingKey::from_sec1_bytes(
                    distid.as_deref().unwrap_or(DEFAULT_DISTID),
                    public_key,
                ).map_err(|_| CryptoError::VerificationFailed)?;
                
                let sig = Sm2Signature::from_bytes(signature.into())
                    .map_err(|_| CryptoError::InvalidSignature(64))?;
                
                use sm2::dsa::signature::Verifier;
                verifying_key.verify(message, &sig)
                    .map_err(|_| CryptoError::VerificationFailed)?;
                
                Ok(())
            }
            _ => Err(CryptoError::VerificationFailed),
        }
    }

    fn name(&self) -> &'static str {
        "sm2"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sm2_keypair_generation() {
        let kp = Sm2.generate_keypair();
        match kp {
            KeyPair::Sm2 { public_key, secret_key } => {
                assert_eq!(public_key.len(), 65);
                assert_eq!(secret_key.len(), 32);
            }
            _ => panic!("Expected Sm2 keypair"),
        }
    }

    #[test]
    fn test_sm2_sign_verify() {
        let kp = Sm2.generate_keypair();
        let msg = b"SM2 test message";
        let sig = Sm2.sign(&kp.secret_key(), msg);
        
        assert!(Sm2.verify(&kp.public_key(), msg, &sig).is_ok());
    }

    #[test]
    fn test_sm2_tampered_message() {
        let kp = Sm2.generate_keypair();
        let msg = b"original message";
        let sig = Sm2.sign(&kp.secret_key(), msg);
        
        let wrong_msg = b"tampered message";
        assert!(Sm2.verify(&kp.public_key(), wrong_msg, &sig).is_err());
    }

    #[test]
    fn test_sm2_from_seed() {
        let seed = [0xaa; 32];
        let kp1 = Sm2.from_seed(&seed);
        let kp2 = Sm2.from_seed(&seed);
        
        match (kp1, kp2) {
            (KeyPair::Sm2 { public_key: pk1, .. }, KeyPair::Sm2 { public_key: pk2, .. }) => {
                assert_eq!(pk1, pk2);
            }
            _ => panic!("Expected Sm2 keypairs"),
        }
    }
}
