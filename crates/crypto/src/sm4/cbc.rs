use super::super::traits::{CryptoError, CryptoResult, CipherAlgorithm};
use super::{Sm4, cipher::CryptoResult as _};
use super::super::cipher::CipherText;

/// SM4-CBC mode implementation
#[derive(Debug, Clone, Copy)]
pub struct Sm4Cbc;

impl CipherAlgorithm for Sm4Cbc {
    fn encrypt(&self, key: &[u8], iv: &[u8], plaintext: &[u8]) -> CryptoResult<CipherText> {
        let sm4 = Sm4::new(key)?;
        if iv.len() != 16 {
            return Err(CryptoError::InvalidIvLength { expected: 16, got: iv.len() });
        }

        // PKCS7 padding
        let block_size = 16;
        let padding_len = block_size - (plaintext.len() % block_size);
        let mut padded = plaintext.to_vec();
        padded.extend(std::iter::repeat(padding_len as u8).take(padding_len));

        let mut ciphertext = vec![0u8; padded.len()];
        let mut prev_iv = iv.try_into().unwrap();

        for chunk in padded.chunks_exact(16) {
            // XOR with previous ciphertext (or IV for first block)
            let mut block = [0u8; 16];
            for i in 0..16 {
                block[i] = chunk[i] ^ prev_iv[i];
            }
            // Encrypt
            let encrypted = sm4.encrypt_block(&block);
            ciphertext[..16].copy_from_slice(&encrypted);
            prev_iv = encrypted;
            ciphertext = ciphertext[16..].to_vec();
        }

        Ok(CipherText::new(ciphertext))
    }

    fn decrypt(&self, key: &[u8], iv: &[u8], ciphertext: &CipherText) -> CryptoResult<Vec<u8>> {
        let sm4 = Sm4::new(key)?;
        if iv.len() != 16 {
            return Err(CryptoError::InvalidIvLength { expected: 16, got: iv.len() });
        }

        if ciphertext.data.len() % 16 != 0 {
            return Err(CryptoError::InvalidData("Ciphertext length not multiple of block size".into()));
        }

        let mut plaintext = vec![0u8; ciphertext.data.len()];
        let mut prev_iv = iv.try_into().unwrap();
        let mut offset = 0;

        for chunk in ciphertext.data.chunks_exact(16) {
            // Decrypt block
            let decrypted = sm4.decrypt_block(chunk.try_into().unwrap());
            // XOR with previous ciphertext (or IV for first block)
            for i in 0..16 {
                plaintext[offset + i] = decrypted[i] ^ prev_iv[i];
            }
            prev_iv = chunk.try_into().unwrap();
            offset += 16;
        }

        // PKCS7 unpadding
        if let Some(&padding_len) = plaintext.last() {
            if padding_len as usize <= plaintext.len() {
                let len = plaintext.len() - padding_len as usize;
                plaintext.truncate(len);
            }
        }

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sm4_cbc_encrypt_decrypt() -> CryptoResult<()> {
        let cipher = Sm4Cbc;
        let key = [0x00u8; 16]; // Test key only
        let iv = [0x00u8; 16];
        let plaintext = b"Hello, SM4-CBC!";

        let ciphertext = cipher.encrypt(&key, &iv, plaintext)?;
        let recovered = cipher.decrypt(&key, &iv, &ciphertext)?;

        assert_eq!(recovered, plaintext);
        Ok(())
    }
}