//! SM4-GCM 加密算法实现

use crate::algorithms::GcmAlgorithm;
use crate::{CryptoError, CryptoResult};
use sm4_gcm::{sm4_gcm_aad_decrypt, sm4_gcm_aad_encrypt, Sm4Key};

/// SM4-GCM 模式（使用 `sm4-gcm` crate）
#[derive(Debug, Clone, Copy)]
pub struct Sm4Gcm;

impl GcmAlgorithm for Sm4Gcm {
    fn encrypt_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        aad: &[u8],
        plaintext: &[u8],
    ) -> CryptoResult<(Vec<u8>, Vec<u8>)> {
        if key.len() != 16 {
            return Err(CryptoError::Sm4Error("SM4 key must be 16 bytes".into()));
        }
        if nonce.len() != 12 {
            return Err(CryptoError::Sm4Error("SM4-GCM nonce must be 12 bytes".into()));
        }

        let sm4_key = Sm4Key::from_slice(key)
            .map_err(|_| CryptoError::Sm4Error("invalid SM4 key".into()))?;

        let combined = sm4_gcm_aad_encrypt(&sm4_key, nonce, aad, plaintext);

        // 分离 ciphertext 和 tag (tag 16 字节)
        let tag_len = 16;
        if combined.len() < tag_len {
            return Err(CryptoError::Sm4Error("encrypted output too short".into()));
        }
        let ct = &combined[..combined.len() - tag_len];
        let tag = &combined[combined.len() - tag_len..];
        Ok((ct.to_vec(), tag.to_vec()))
    }

    fn decrypt_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        aad: &[u8],
        ciphertext: &[u8],
        tag: &[u8],
    ) -> CryptoResult<Vec<u8>> {
        if key.len() != 16 {
            return Err(CryptoError::Sm4Error("SM4 key must be 16 bytes".into()));
        }
        if nonce.len() != 12 {
            return Err(CryptoError::Sm4Error("SM4-GCM nonce must be 12 bytes".into()));
        }
        if tag.len() != 16 {
            return Err(CryptoError::Sm4Error("SM4-GCM tag must be 16 bytes".into()));
        }

        let sm4_key = Sm4Key::from_slice(key)
            .map_err(|_| CryptoError::Sm4Error("invalid SM4 key".into()))?;

        let mut combined = ciphertext.to_vec();
        combined.extend_from_slice(tag);

        sm4_gcm_aad_decrypt(&sm4_key, nonce, aad, &combined)
            .map_err(|e| CryptoError::Sm4Error(format!("SM4-GCM decrypt error: {}", e)))
    }

    fn name(&self) -> &'static str {
        "sm4-gcm"
    }

    fn key_len(&self) -> usize {
        16
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
    fn test_sm4_gcm_roundtrip() {
        let algo = Sm4Gcm;
        let mut key = [0u8; 16];
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut key);
        OsRng.fill_bytes(&mut nonce);
        let aad = b"auth data";

        let plaintext = b"SM4-GCM test";
        let (ciphertext, tag) = algo.encrypt_gcm(&key, &nonce, aad, plaintext).unwrap();
        let decrypted = algo.decrypt_gcm(&key, &nonce, aad, &ciphertext, &tag).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    #[should_panic(expected = "SM4 key must be 16 bytes")]
    fn test_sm4_gcm_wrong_key_len() {
        let algo = Sm4Gcm;
        let key = [0u8; 32]; // wrong length
        let nonce = [0u8; 12];
        let aad = b"";
        let pt = b"test";
        let _ = algo.encrypt_gcm(&key, &nonce, aad, pt).unwrap();
    }

    #[test]
    fn test_sm4_gcm_invalid_tag() {
        let algo = Sm4Gcm;
        let mut key = [0u8; 16];
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
}
