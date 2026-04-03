//! 对称加密算法实现
//!
//! 提供 AES-CBC 和 SM4-CBC 的 CipherAlgorithm trait 实现
//! 以及 AES-GCM 和 SM4-GCM 的 GcmAlgorithm trait 实现

use super::algorithms::{CipherAlgorithm, GcmAlgorithm};
use crate::{sm4, CryptoError};
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, generic_array::GenericArray};
use rand::rngs::OsRng;
use sm4::Sm4;

/* 暂时禁用 AES-CBC 实现，避免依赖冲突
/// AES-CBC 模式
///
/// 使用 PKCS#7 填充，返回 `iv || ciphertext`
/// 密钥长度支持 16/24/32 字节（AES-128/192/256）
#[derive(Debug, Clone, Copy)]
pub struct AesCbc {
    key_len: usize,
}

impl AesCbc {
    /// 创建指定密钥长度的 AES-CBC
    pub fn new(key_len: usize) -> Self {
        assert!(key_len == 16 || key_len == 24 || key_len == 32, "AES key must be 16, 24, or 32 bytes");
        Self { key_len }
    }

    /// AES-128-CBC
    pub fn aes128() -> Self {
        Self::new(16)
    }

    /// AES-256-CBC
    pub fn aes256() -> Self {
        Self::new(32)
    }
}

impl CipherAlgorithm for AesCbc {
    fn encrypt_cbc(&self, key: &[u8], iv: &[u8], plaintext: &[u8]) -> Vec<u8> {
        unimplemented!("AES-CBC disabled due to dependency conflict")
    }

    fn decrypt_cbc(&self, key: &[u8], iv_ciphertext: &[u8]) -> CryptoResult<Vec<u8>> {
        unimplemented!("AES-CBC disabled due to dependency conflict")
    }

    fn name(&self) -> &'static str {
        match self.key_len {
            16 => "aes-128-cbc",
            24 => "aes-192-cbc",
            32 => "aes-256-cbc",
            _ => "aes-unknown-cbc",
        }
    }

    fn key_len(&self) -> usize {
        self.key_len
    }
}
*/
/// SM4-CBC 模式
///
/// 使用 PKCS#7 填充，返回 `iv || ciphertext`
/// 密钥长度固定 16 字节
#[derive(Debug, Clone, Copy)]
pub struct Sm4Cbc;

impl CipherAlgorithm for Sm4Cbc {
    fn encrypt_cbc(&self, key: &[u8], iv: &[u8], plaintext: &[u8]) -> Vec<u8> {
        assert_eq!(key.len(), 16, "SM4 key must be 16 bytes");
        assert_eq!(iv.len(), 16, "IV must be 16 bytes");

        let mut cipher = Sm4::new_from_slice(key).unwrap();
        let mut result = Vec::with_capacity(iv.len() + plaintext.len() + 16);
        result.extend_from_slice(iv);

        let mut prev_block = *GenericArray::from_slice(iv).as_slice();
        let padded = {
            let pad_len = 16 - (plaintext.len() % 16);
            let mut p = plaintext.to_vec();
            p.extend(std::iter::repeat(pad_len as u8).take(pad_len));
            p
        };

        for chunk in padded.chunks(16) {
            let mut block = [0u8; 16];
            for (b, &p) in block.iter_mut().zip(chunk) {
                *b = p ^ prev_block[b as usize];
            }
            cipher.encrypt_block(&mut block.into());
            result.extend_from_slice(&block);
            prev_block = block;
        }

        result
    }

    fn decrypt_cbc(&self, key: &[u8], iv_ciphertext: &[u8]) -> CryptoResult<Vec<u8>> {
        assert_eq!(key.len(), 16, "SM4 key must be 16 bytes");

        if iv_ciphertext.len() < 16 {
            return Err(CryptoError::Sm4Error("ciphertext too short".into()));
        }
        if (iv_ciphertext.len() - 16) % 16 != 0 {
            return Err(CryptoError::Sm4Error("ciphertext length not multiple of block size".into()));
        }

        let iv = &iv_ciphertext[..16];
        let ciphertext = &iv_ciphertext[16:];
        let mut cipher = Sm4::new_from_slice(key).unwrap();
        let mut prev_block = [0u8; 16];
        prev_block.copy_from_slice(iv);

        let mut plaintext = Vec::with_capacity(ciphertext.len());

        for chunk in ciphertext.chunks_exact(16) {
            let mut block = GenericArray::clone_from_slice(chunk);
            cipher.decrypt_block(&mut block);

            for (p, (&dec, &prev)) in plaintext.iter_mut().zip(block.iter().zip(prev_block.iter())) {
                *p = dec ^ prev;
            }
            prev_block = *GenericArray::from_slice(chunk).as_slice();
        }

        // PKCS#7 去填充
        if let Some(&pad_len) = plaintext.last() {
            let pad_len = pad_len as usize;
            if pad_len > 0 && pad_len <= 16 && pad_len <= plaintext.len() {
                plaintext.truncate(plaintext.len() - pad_len);
            }
        }

        Ok(plaintext)
    }

    fn name(&self) -> &'static str {
        "sm4-cbc"
    }

    fn key_len(&self) -> usize {
        16
    }
}

/// AES-GCM 模式（使用 `aes-gcm` crate）
#[derive(Debug, Clone, Copy)]
pub struct AesGcm {
    key_len: usize,
}

impl AesGcm {
    pub fn new(key_len: usize) -> Self {
        assert!(key_len == 16 || key_len == 24 || key_len == 32, "AES key must be 16, 24, or 32 bytes");
        Self { key_len }
    }

    pub fn aes256() -> Self {
        Self::new(32)
    }
}

impl GcmAlgorithm for AesGcm {
    fn encrypt_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        aad: &[u8],
        plaintext: &[u8],
    ) -> CryptoResult<(Vec<u8>, Vec<u8>)> {
        assert_eq!(key.len(), self.key_len, "AES key length mismatch");
        assert_eq!(nonce.len(), 12, "GCM nonce must be 12 bytes");

        use aes_gcm::{
            aead::{Aead, NewAead},
            Aes256Gcm, // 简化：固定 AES-256
            generic_array::GenericArray,
        };

        let key_arr = GenericArray::from_slice(key);
        let cipher = Aes256Gcm::new(key_arr);

        let nonce_arr = GenericArray::from_slice(nonce);
        let aad_arr = GenericArray::from_slice(aad);

        let ciphertext = cipher
            .encrypt(nonce_arr, aad_arr, plaintext)
            .map_err(|_| CryptoError::Sm4Error("AES-GCM encryption failed".into()))?;

        // aes-gcm 返回 ciphertext || tag (16 bytes)
        let tag_len = 16;
        let (ct, tag) = ciphertext.split_at(ciphertext.len() - tag_len);
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
        assert_eq!(key.len(), self.key_len, "AES key length mismatch");
        assert_eq!(nonce.len(), 12, "GCM nonce must be 12 bytes");
        assert_eq!(tag.len(), 16, "GCM tag must be 16 bytes");

        use aes_gcm::{
            aead::{Aead, NewAead},
            Aes256Gcm,
            generic_array::GenericArray,
        };

        let key_arr = GenericArray::from_slice(key);
        let cipher = Aes256Gcm::new(key_arr);
        let nonce_arr = GenericArray::from_slice(nonce);
        let aad_arr = GenericArray::from_slice(aad);

        let mut buffer = ciphertext.to_vec();
        buffer.extend_from_slice(tag);

        cipher
            .decrypt(nonce_arr, aad_arr, &buffer)
            .map_err(|_| CryptoError::Sm4Error("AES-GCM authentication failed".into()))
    }

    fn name(&self) -> &'static str {
        match self.key_len {
            16 => "aes-128-gcm",
            24 => "aes-192-gcm",
            32 => "aes-256-gcm",
            _ => "aes-unknown-gcm",
        }
    }

    fn key_len(&self) -> usize {
        self.key_len
    }
}

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
        assert_eq!(key.len(), 16, "SM4 key must be 16 bytes");
        assert_eq!(nonce.len(), 12, "SM4-GCM nonce must be 12 bytes");

        use sm4_gcm::{
            aead::{Aead, NewAead},
            Sm4Gcm as Sm4GcmCipher,
            generic_array::GenericArray,
        };

        let key_arr = GenericArray::from_slice(key);
        let cipher = Sm4GcmCipher::new(key_arr);

        let nonce_arr = GenericArray::from_slice(nonce);
        let aad_arr = GenericArray::from_slice(aad);

        let combined = cipher
            .encrypt(nonce_arr, aad_arr, plaintext)
            .map_err(|_| CryptoError::Sm4Error("SM4-GCM encryption failed".into()))?;

        let tag_len = 16;
        let (ct, tag) = combined.split_at(combined.len() - tag_len);
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
        assert_eq!(key.len(), 16, "SM4 key must be 16 bytes");
        assert_eq!(nonce.len(), 12, "SM4-GCM nonce must be 12 bytes");
        assert_eq!(tag.len(), 16, "SM4-GCM tag must be 16 bytes");

        use sm4_gcm::{
            aead::{Aead, NewAead},
            Sm4Gcm as Sm4GcmCipher,
            generic_array::GenericArray,
        };

        let key_arr = GenericArray::from_slice(key);
        let cipher = Sm4GcmCipher::new(key_arr);
        let nonce_arr = GenericArray::from_slice(nonce);
        let aad_arr = GenericArray::from_slice(aad);

        let mut buffer = ciphertext.to_vec();
        buffer.extend_from_slice(tag);

        cipher
            .decrypt(nonce_arr, aad_arr, &buffer)
            .map_err(|_| CryptoError::Sm4Error("SM4-GCM authentication failed".into()))
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

    #[test]
    fn test_aes_cbc_roundtrip() {
        let algo = AesCbc::aes256();
        let mut key = [0u8; 32];
        let mut iv = [0u8; 16];
        OsRng.fill_bytes(&mut key);
        OsRng.fill_bytes(&mut iv);

        let plaintext = b"AES-CBC test message";
        let ciphertext = algo.encrypt_cbc(&key, &iv, plaintext);
        let decrypted = algo.decrypt_cbc(&key, &ciphertext).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_sm4_cbc_roundtrip() {
        let algo = Sm4Cbc;
        let mut key = [0u8; 16];
        let mut iv = [0u8; 16];
        OsRng.fill_bytes(&mut key);
        OsRng.fill_bytes(&mut iv);

        let plaintext = b"SM4-CBC test";
        let ciphertext = algo.encrypt_cbc(&key, &iv, plaintext);
        let decrypted = algo.decrypt_cbc(&key, &ciphertext).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_aes_gcm_roundtrip() {
        let algo = AesGcm::aes256();
        let mut key = [0u8; 32];
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut key);
        OsRng.fill_bytes(&mut nonce);
        let aad = b"header";

        let plaintext = b"AES-GCM test";
        let (ciphertext, tag) = algo.encrypt_gcm(&key, &nonce, aad, plaintext).unwrap();
        let decrypted = algo.decrypt_gcm(&key, &nonce, aad, &ciphertext, &tag).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

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
    fn test_gcm_wrong_tag() {
        let algo = AesGcm::aes256();
        let mut key = [0u8; 32];
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut key);
        OsRng.fill_bytes(&mut nonce);

        let plaintext = b"test";
        let (ciphertext, tag) = algo.encrypt_gcm(&key, &nonce, b"", plaintext).unwrap();

        let mut bad_tag = tag.clone();
        bad_tag[0] = bad_tag[0].wrapping_add(1);
        assert!(algo.decrypt_gcm(&key, &nonce, b"", &ciphertext, &bad_tag).is_err());
    }

    #[test]
    fn test_cipher_algorithm_names() {
        assert_eq!(AesCbc::aes128().name(), "aes-128-cbc");
        assert_eq!(AesCbc::aes256().name(), "aes-256-cbc");
        assert_eq!(Sm4Cbc.name(), "sm4-cbc");
        assert_eq!(AesGcm::aes256().name(), "aes-256-gcm");
        assert_eq!(Sm4Gcm.name(), "sm4-gcm");
    }

    #[test]
    fn test_key_lengths() {
        assert_eq!(AesCbc::aes128().key_len(), 16);
        assert_eq!(AesCbc::aes256().key_len(), 32);
        assert_eq!(Sm4Cbc.key_len(), 16);
        assert_eq!(AesGcm::aes256().key_len(), 32);
        assert_eq!(Sm4Gcm.key_len(), 16);
    }
}
