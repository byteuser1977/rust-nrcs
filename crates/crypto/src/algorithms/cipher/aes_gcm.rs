//! AES-256-GCM encryption algorithm implementation
//!
//! Compatible with Java NRCS (BouncyCastle AES-256-GCM)

use crate::algorithms::GcmAlgorithm;
use crate::{CryptoError, CryptoResult};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::AeadInPlace};

/// AES-256-GCM mode (for Java NRCS compatibility)
#[derive(Debug, Clone, Copy)]
pub struct AesGcm;

impl GcmAlgorithm for AesGcm {
    fn encrypt_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        aad: &[u8],
        plaintext: &[u8],
    ) -> CryptoResult<(Vec<u8>, Vec<u8>)> {
        if key.len() != 32 {
            return Err(CryptoError::AesError("AES-256 key must be 32 bytes".into()));
        }
        if nonce.len() != 12 {
            return Err(CryptoError::AesError("AES-GCM nonce must be 12 bytes".into()));
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| CryptoError::AesError(format!("invalid AES key: {}", e)))?;

        let nonce = Nonce::from_slice(nonce);

        // Use aead_in_place to support AAD
        let mut buffer = plaintext.to_vec();
        let tag = cipher
            .encrypt_in_place_detached(nonce, aad, &mut buffer)
            .map_err(|e| CryptoError::AesError(format!("AES-GCM encrypt error: {}", e)))?;

        Ok((buffer, tag.to_vec()))
    }

    fn decrypt_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        aad: &[u8],
        ciphertext: &[u8],
        tag: &[u8],
    ) -> CryptoResult<Vec<u8>> {
        if key.len() != 32 {
            return Err(CryptoError::AesError("AES-256 key must be 32 bytes".into()));
        }
        if nonce.len() != 12 {
            return Err(CryptoError::AesError("AES-GCM nonce must be 12 bytes".into()));
        }
        if tag.len() != 16 {
            return Err(CryptoError::AesError("AES-GCM tag must be 16 bytes".into()));
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| CryptoError::AesError(format!("invalid AES key: {}", e)))?;

        let nonce = Nonce::from_slice(nonce);
        let tag = aes_gcm::Tag::from_slice(tag);

        let mut buffer = ciphertext.to_vec();
        cipher
            .decrypt_in_place_detached(nonce, aad, &mut buffer, tag)
            .map_err(|e| CryptoError::AesError(format!("AES-GCM decrypt error: {}", e)))?;

        Ok(buffer)
    }

    fn name(&self) -> &'static str {
        "aes-256-gcm"
    }

    fn key_len(&self) -> usize {
        32
    }

    fn nonce_len(&self) -> usize {
        12
    }

    fn tag_len(&self) -> usize {
        16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;
    use rand::RngCore;

    #[test]
    fn test_aes_gcm_roundtrip() {
        let algo = AesGcm;
        let mut key = [0u8; 32];
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut key);
        OsRng.fill_bytes(&mut nonce);
        let aad = b"auth data";

        let plaintext = b"AES-256-GCM test";
        let (ciphertext, tag) = algo.encrypt_gcm(&key, &nonce, aad, plaintext).unwrap();
        let decrypted = algo.decrypt_gcm(&key, &nonce, aad, &ciphertext, &tag).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    #[should_panic(expected = "AES-256 key must be 32 bytes")]
    fn test_aes_gcm_wrong_key_len() {
        let algo = AesGcm;
        let key = [0u8; 16]; // wrong length
        let nonce = [0u8; 12];
        let aad = b"";
        let pt = b"test";
        let _ = algo.encrypt_gcm(&key, &nonce, aad, pt).unwrap();
    }

    #[test]
    fn test_aes_gcm_invalid_tag() {
        let algo = AesGcm;
        let mut key = [0u8; 32];
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut key);
        OsRng.fill_bytes(&mut nonce);
        let aad = b"header";
        let pt = b"test";

        let (ct, tag) = algo.encrypt_gcm(&key, &nonce, aad, pt).unwrap();
        // corrupt tag
        let mut bad_tag = tag.clone();
        bad_tag[0] = bad_tag[0].wrapping_add(1);
        assert!(algo.decrypt_gcm(&key, &nonce, aad, &ct, &bad_tag).is_err());
    }

    #[test]
    fn test_aes_gcm_empty_plaintext() {
        let algo = AesGcm;
        let mut key = [0u8; 32];
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut key);
        OsRng.fill_bytes(&mut nonce);
        let aad = b"header";

        let plaintext = b"";
        let (ciphertext, tag) = algo.encrypt_gcm(&key, &nonce, aad, plaintext).unwrap();
        let decrypted = algo.decrypt_gcm(&key, &nonce, aad, &ciphertext, &tag).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }
}
