//! 签名算法实现
//!
//! 为 Ed25519 和 SM2 实现 `SignatureAlgorithm` trait。
//! 每个算法是无状态零大小类型（marker），算法实现通过返回 `crate::keypair::KeyPair` 枚举来支持多态。

use super::algorithms::SignatureAlgorithm;
use crate::{CryptoError, Hash256, KeyPair, PublicKey, SecretKey, Signature};
use ed25519_dalek::{self, Signature as EdSignature, Signer, Verifier};
use rand::rngs::OsRng;
use sm2::{self, Signature as Sm2Signature, SecretKey as Sm2Secret, PublicKey as Sm2Public};

/// Ed25519 签名算法标记类型
#[derive(Debug, Clone, Copy)]
pub struct Ed25519;

impl SignatureAlgorithm for Ed25519 {
    fn generate_keypair(&self) -> KeyPair {
        let kp = ed25519_dalek::Keypair::generate(&mut OsRng);
        KeyPair::Ed25519(kp)
    }

    fn from_seed(&self, seed: &[u8; 32]) -> KeyPair {
        let secret = ed25519_dalek::SecretKey::from_bytes(seed)
            .expect("seed must be 32 bytes");
        let public = (&secret).into();
        KeyPair::Ed25519(ed25519_dalek::Keypair { secret, public })
    }

    fn sign(&self, key: &SecretKey, message: &[u8]) -> Signature {
        match key {
            SecretKey::Ed25519(bytes) => {
                let secret = ed25519_dalek::SecretKey::from_bytes(bytes)
                    .expect("valid Ed25519 secret key");
                *secret.sign(message)
            }
            SecretKey::Sm2(_) => {
                panic!("Ed25519 algorithm cannot sign with SM2 secret key");
            }
        }
    }

    fn verify(
        &self,
        public_key: &PublicKey,
        message: &[u8],
        signature: &Signature,
    ) -> CryptoResult<()> {
        match public_key {
            PublicKey::Ed25519(bytes) => {
                let pk = ed25519_dalek::PublicKey::from_bytes(bytes)
                    .map_err(|_| CryptoError::InvalidPublicKey(32))?;
                let sig = EdSignature::from_bytes(signature)
                    .map_err(|_| CryptoError::InvalidSignature(64))?;
                pk.verify(message, &sig)
                    .map_err(|_| CryptoError::VerificationFailed)?;
            }
            PublicKey::Sm2(_) => {
                panic!("Ed25519 algorithm cannot verify SM2 public key");
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "ed25519"
    }
}

/// SM2 国密签名算法标记类型
#[derive(Debug, Clone, Copy)]
pub struct Sm2;

impl SignatureAlgorithm for Sm2 {
    fn generate_keypair(&self) -> KeyPair {
        let (secret, public) = sm2::keygen();
        KeyPair::Sm2(sm2::KeyPair { inner: (secret, public) })
    }

    fn from_seed(&self, seed: &[u8; 32]) -> KeyPair {
        let secret_bytes = sm3::sm3(seed); // SM2 key derivation uses SM3 of seed
        let secret = Sm2Secret::from_bytes(&secret_bytes)
            .expect("derived secret is valid");
        let public = secret.random_public_key();
        KeyPair::Sm2(sm2::KeyPair { inner: (secret, public) })
    }

    fn sign(&self, key: &SecretKey, message: &[u8]) -> Signature {
        match key {
            SecretKey::Sm2(bytes) => {
                let secret = Sm2Secret::from_bytes(bytes)
                    .expect("valid SM2 secret key");
                secret.sign(message).to_bytes().into()
            }
            SecretKey::Ed25519(_) => {
                panic!("SM2 algorithm cannot sign with Ed25519 secret key");
            }
        }
    }

    fn verify(
        &self,
        public_key: &PublicKey,
        message: &[u8],
        signature: &Signature,
    ) -> CryptoResult<()> {
        match public_key {
            PublicKey::Sm2(bytes) => {
                let mut arr = [0u8; 64];
                arr.copy_from_slice(bytes);
                let pk = Sm2Public::from_bytes(&arr)
                    .map_err(|_| CryptoError::InvalidPublicKey(64))?;
                let mut sig_arr = [0u8; 64];
                sig_arr.copy_from_slice(signature);
                let sig = Sm2Signature::from_bytes(&sig_arr)
                    .map_err(|_| CryptoError::InvalidSignature(64))?;
                sm2::verify(&pk, message, &sig)
                    .map_err(|_| CryptoError::VerificationFailed)?;
            }
            PublicKey::Ed25519(_) => {
                panic!("SM2 algorithm cannot verify Ed25519 public key");
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "sm2"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_sign_verify() {
        let algo = Ed25519;
        let kp = algo.generate_keypair();
        let msg = b"test";
        let sig = algo.sign(&kp.secret_key(), msg);
        assert!(algo.verify(&kp.public_key(), msg, &sig).is_ok());
    }

    #[test]
    fn test_sm2_sign_verify() {
        let algo = Sm2;
        let kp = algo.generate_keypair();
        let msg = b"test";
        let sig = algo.sign(&kp.secret_key(), msg);
        assert!(algo.verify(&kp.public_key(), msg, &sig).is_ok());
    }

    #[test]
    fn test_algorithm_names() {
        assert_eq!(Ed25519.name(), "ed25519");
        assert_eq!(Sm2.name(), "sm2");
    }
}
