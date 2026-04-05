use super::super::traits::{CryptoError, CryptoResult, SignerAlgorithm};
use super::{PublicKey, SecretKey, Signature};

/// Ed25519 signature algorithm implementation
#[derive(Debug, Clone, Copy)]
pub struct Ed25519;

impl SignerAlgorithm for Ed25519 {
    fn sign(&self, key: &SecretKey, message: &[u8]) -> Signature {
        let secret_key = match key {
            SecretKey::Ed25519(sk) => sk,
            _ => panic!("Ed25519 sign called with non-Ed25519 key"),
        };
        let signature = secret_key.sign(message);
        Signature::Ed25519(signature)
    }

    fn verify(&self, key: &PublicKey, message: &[u8], signature: &Signature) -> CryptoResult<()> {
        let public_key = match key {
            PublicKey::Ed25519(pk) => pk,
            _ => return Err(CryptoError::InvalidData("Ed25519 verify called with non-Ed25519 key".into())),
        };
        let sig = match signature {
            Signature::Ed25519(s) => s,
            _ => return Err(CryptoError::InvalidData("Signature type mismatch".into())),
        };

        public_key
            .verify(message, sig)
            .map_err(|_| CryptoError::VerificationFailed)?;
        Ok(())
    }

    fn generate_keypair(&self) -> CryptoResult<super::KeyPair> {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let keypair = ed25519_dalek::Keypair::generate(&mut rng);
        Ok(super::KeyPair::Ed25519(keypair))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_sign_verify() -> CryptoResult<()> {
        let algo = Ed25519;
        let keypair = algo.generate_keypair()?;
        let message = b"hello world";

        let signature = algo.sign(keypair.secret_key(), message);
        algo.verify(keypair.public_key(), message, &signature)?;

        Ok(())
    }

    #[test]
    fn test_ed25519_verify_invalid_signature() -> CryptoResult<()> {
        let algo = Ed25519;
        let keypair = algo.generate_keypair()?;
        let message = b"hello world";
        let wrong_message = b"goodbye world";

        let signature = algo.sign(keypair.secret_key(), message);
        let result = algo.verify(keypair.public_key(), wrong_message, &signature);
        assert!(result.is_err());

        Ok(())
    }
}