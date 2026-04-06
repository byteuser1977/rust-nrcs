//! 签名算法实现
//!
//! 为 Ed25519 和 SM2 实现 `SignatureAlgorithm` trait。
//! 每个算法是无状态零大小类型（marker），算法实现通过返回 `crate::keypair::KeyPair` 枚举来支持多态。

use crate::algorithms::SignatureAlgorithm;
use crate::{CryptoError, CryptoResult, KeyPair, PublicKey, SecretKey, Signature};
use ed25519_dalek::{self, Signature as EdSignature, Signer, Verifier};
use rand::rngs::OsRng;

/// Ed25519 签名算法标记类型
#[derive(Debug, Clone, Copy)]
pub struct Ed25519;

impl SignatureAlgorithm for Ed25519 {
    fn generate_keypair(&self) -> KeyPair {
        let kp = ed25519_dalek::SigningKey::generate(&mut OsRng);
        KeyPair::Ed25519(kp)
    }

    fn from_seed(&self, seed: &[u8; 32]) -> KeyPair {
        let kp = ed25519_dalek::SigningKey::from_bytes(seed);
        KeyPair::Ed25519(kp)
    }

    fn sign(&self, key: &SecretKey, message: &[u8]) -> Signature {
        match key {
            SecretKey::Ed25519(bytes) => {
                let mut seed = [0u8; 32];
                seed.copy_from_slice(&bytes[0..32]);
                let secret = ed25519_dalek::SigningKey::from_bytes(&seed);
                secret.sign(message).to_bytes()
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
                let pk = ed25519_dalek::VerifyingKey::from_bytes(bytes)
                    .map_err(|_| CryptoError::InvalidPublicKey(32))?;
                
                let sig_array: [u8; 64] = *signature;
                let sig = EdSignature::from(sig_array);
                
                pk.verify(message, &sig)
                    .map_err(|_| CryptoError::VerificationFailed)?;
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "ed25519"
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
    fn test_algorithm_names() {
        assert_eq!(Ed25519.name(), "ed25519");
    }
}
