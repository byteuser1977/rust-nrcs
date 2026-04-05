use std::ops::Range;

use super::super::traits::{CryptoError, CryptoResult, CipherAlgorithm};
use super::CipherText;
use aes::cipher::{generic_array::GenericArray, BlockEncrypt, BlockDecrypt, KeyInit};
use cbc::{Encryptor as CbcEncryptor, Decryptor as CbcDecryptor};
use aes_gcm::AesGcm;

/// AES-CBC mode implementation using the `cbc` crate
#[derive(Debug, Clone, Copy)]
pub struct AesCbc;

impl CipherAlgorithm for AesCbc {
    fn encrypt(&self, key: &[u8], iv: &[u8], plaintext: &[u8]) -> CryptoResult<CipherText> {
        if iv.len() != 16 {
            return Err(CryptoError::InvalidIvLength { expected: 16, got: iv.len() });
        }

        // Select AES variant based on key length
        let (encryptor, key_len) = match key.len() {
            16 => (CbcEncryptor::<aes::Aes128>::new(key.into(), iv.into()), 16),
            24 => (CbcEncryptor::<aes::Aes192>::new(key.into(), iv.into()), 24),
            32 => (CbcEncryptor::<aes::Aes256>::new(key.into(), iv.into()), 32),
            _ => return Err(CryptoError::InvalidKeyLength { expected: 16, got: key.len() }),
        };

        // PKCS7 padding
        let block_size = 16usize;
        let padding_len = block_size - (plaintext.len() % block_size);
        let mut padded = plaintext.to_vec();
        padded.extend(std::iter::repeat(padding_len as u8).take(padding_len));

        // Allocate ciphertext
        let mut ciphertext = vec![0u8; padded.len()];
        encryptor.encrypt(padded.as_slice().into(), &mut ciphertext.into());

        Ok(CipherText::new(ciphertext))
    }

    fn decrypt(&self, key: &[u8], iv: &[u8], ciphertext: &CipherText) -> CryptoResult<Vec<u8>> {
        if iv.len() != 16 {
            return Err(CryptoError::InvalidIvLength { expected: 16, got: iv.len() });
        }

        // Select AES variant
        let (decryptor, key_len) = match key.len() {
            16 => (CbcDecryptor::<aes::Aes128>::new(key.into(), iv.into()), 16),
            24 => (CbcDecryptor::<aes::Aes192>::new(key.into(), iv.into()), 24),
            32 => (CbcDecryptor::<aes::Aes256>::new(key.into(), iv.into()), 32),
            _ => return Err(CryptoError::InvalidKeyLength { expected: 16, got: key.len() }),
        };

        // Decrypt all blocks
        let mut decrypted = vec![0u8; ciphertext.data.len()];
        decryptor.decrypt(ciphertext.data.as_slice().into(), &mut decrypted.into());

        // PKCS7 unpadding
        if let Some(&padding_len) = decrypted.last() {
            if padding_len as usize <= decrypted.len() {
                let len = decrypted.len() - padding_len as usize;
                decrypted.truncate(len);
            }
        }

        Ok(decrypted)
    }
}

/// AES-GCM mode implementation (AEAD) using the `aes-gcm` crate
#[derive(Debug, Clone, Copy)]
pub struct AesGcm;

impl CipherAlgorithm for AesGcm {
    fn encrypt(&self, key: &[u8], iv: &[u8], plaintext: &[u8]) -> CryptoResult<CipherText> {
        // AES-GCM supports 128, 192, 256-bit keys
        let cipher = match key.len() {
            16 => AesGcm::<aes::Aes128>::new_from_slice(key),
            24 => AesGcm::<aes::Aes192>::new_from_slice(key),
            32 => AesGcm::<aes::Aes256>::new_from_slice(key),
            _ => return Err(CryptoError::InvalidKeyLength { expected: 16|24|32, got: key.len() }),
        }
        .map_err(|_| CryptoError::InvalidData("Invalid AES key for GCM".into()))?;

        if iv.len() != 12 {
            return Err(CryptoError::InvalidIvLength { expected: 12, got: iv.len() });
        }

        let mut ciphertext = vec![0u8; plaintext.len()];
        let mut tag = [0u8; 16];
        cipher
            .encrypt_in_slice(GenericArray::from(iv), plaintext, &mut ciphertext, &mut tag)
            .map_err(|_| CryptoError::InvalidData("AES-GCM encryption failed".into()))?;

        Ok(CipherText::with_tag(ciphertext, tag))
    }

    fn decrypt(&self, key: &[u8], iv: &[u8], ciphertext: &CipherText) -> CryptoResult<Vec<u8>> {
        let cipher = match key.len() {
            16 => AesGcm::<aes::Aes128>::new_from_slice(key),
            24 => AesGcm::<aes::Aes192>::new_from_slice(key),
            32 => AesGcm::<aes::Aes256>::new_from_slice(key),
            _ => return Err(CryptoError::InvalidKeyLength { expected: 16|24|32, got: key.len() }),
        }
        .map_err(|_| CryptoError::InvalidData("Invalid AES key for GCM".into()))?;

        if iv.len() != 12 {
            return Err(CryptoError::InvalidIvLength { expected: 12, got: iv.len() });
        }
        let Some(tag) = ciphertext.tag else {
            return Err(CryptoError::InvalidData("Missing authentication tag for AES-GCM".into()));
        };

        let mut plaintext = vec![0u8; ciphertext.data.len()];
        cipher
            .decrypt_in_slice(
                GenericArray::from(iv),
                &ciphertext.data,
                GenericArray::from(tag),
                &mut plaintext,
            )
            .map_err(|_| CryptoError::VerificationFailed)?;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_cbc_encrypt_decrypt() -> CryptoResult<()> {
        let cipher = AesCbc;
        let key = [0x00u8; 16];
        let iv = [0x00u8; 16];
        let plaintext = b"Hello, AES-CBC!";

        let ciphertext = cipher.encrypt(&key, &iv, plaintext)?;
        let recovered = cipher.decrypt(&key, &iv, &ciphertext)?;

        assert_eq!(recovered, plaintext);
        Ok(())
    }

    #[test]
    fn test_aes_gcm_encrypt_decrypt() -> CryptoResult<()> {
        let cipher = AesGcm;
        let key = [0x00u8; 16];
        let iv = [0x00u8; 12];
        let plaintext = b"Hello, AES-GCM!";

        let ciphertext = cipher.encrypt(&key, &iv, plaintext)?;
        assert!(ciphertext.tag.is_some());

        let recovered = cipher.decrypt(&key, &iv, &ciphertext)?;
        assert_eq!(recovered, plaintext);
        Ok(())
    }

    #[test]
    fn test_aes_gcm_tampered() {
        let cipher = AesGcm;
        let key = [0x00u8; 16];
        let iv = [0x00u8; 12];
        let plaintext = b"Hello, AES-GCM!";

        let ciphertext = cipher.encrypt(&key, &iv, plaintext).unwrap();
        // Tamper with ciphertext
        let mut tampered = ciphertext.clone();
        tampered.data[0] ^= 1;

        let result = cipher.decrypt(&key, &iv, &tampered);
        assert!(result.is_err());
    }
}