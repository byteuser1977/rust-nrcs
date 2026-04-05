use super::super::traits::{CryptoError, CryptoResult, SignerAlgorithm};
use super::{PublicKey, SecretKey, Signature};

/// SM2 signature algorithm implementation (national cryptography standard)
///
/// GM/T 0003-2012: SM2椭圆曲线公钥密码算法
#[derive(Debug, Clone, Copy)]
pub struct Sm2;

impl SignerAlgorithm for Sm2 {
    fn sign(&self, key: &SecretKey, message: &[u8]) -> Signature {
        let secret_key = match key {
            SecretKey::Sm2(sk) => sk,
            _ => panic!("SM2 sign called with non-SM2 key"),
        };
        let signature = sm2::sign(secret_key, message);
        Signature::Sm2(signature)
    }

    fn verify(&self, key: &PublicKey, message: &[u8], signature: &Signature) -> CryptoResult<()> {
        let public_key = match key {
            PublicKey::Sm2(pk) => pk,
            _ => return Err(CryptoError::InvalidData("SM2 verify called with non-SM2 key".into())),
        };
        let sig = match signature {
            Signature::Sm2(s) => s,
            _ => return Err(CryptoError::InvalidData("Signature type mismatch".into())),
        };

        sm2::verify(public_key, message, sig)
            .map_err(|_| CryptoError::VerificationFailed)?;
        Ok(())
    }

    fn generate_keypair(&self) -> CryptoResult<super::KeyPair> {
        // SM2 uses a deterministic key generation based on random number
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let keypair = sm2::KeyPair::generate(&mut rng);
        Ok(super::KeyPair::Sm2(keypair))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sm2_sign_verify() -> CryptoResult<()> {
        let algo = Sm2;
        let keypair = algo.generate_keypair()?;
        let message = b"hello world";

        let signature = algo.sign(keypair.secret_key(), message);
        algo.verify(keypair.public_key(), message, &signature)?;

        Ok(())
    }

    #[test]
    fn test_sm2_verify_invalid_signature() -> CryptoResult<()> {
        let algo = Sm2;
        let keypair = algo.generate_keypair()?;
        let message = b"hello world";
        let wrong_message = b"goodbye world";

        let signature = algo.sign(keypair.secret_key(), message);
        let result = algo.verify(keypair.public_key(), wrong_message, &signature);
        assert!(result.is_err());

        Ok(())
    }
}