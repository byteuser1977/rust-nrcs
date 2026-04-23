//! SM4 分组密码算法（国密标准 GM/T 0002-2012）
//!
//! SM4 是一个 128 位分组、128 位密钥的对称密码算法。
//! 算法特点：
//! - 分组长度：128 位（16 字节）
//! - 密钥长度：128 位（16 字节）
//! - 轮数：32 轮
//! - 结构：Feistel 网络
//!
//! ## 使用示例
//! ```
//! use crypto::algorithms::cipher::sm4::{encrypt_cbc, decrypt_cbc, Sm4Key};
//!
//! let key = Sm4Key::random();
//! let iv = [0u8; 16];
//! let plaintext = b"Hello, SM4!";
//!
//! // CBC 加密
//! let ciphertext = encrypt_cbc(plaintext, &key, &iv);
//!
//! // CBC 解密
//! let decrypted = decrypt_cbc(&ciphertext, &key).unwrap();
//! assert_eq!(plaintext, decrypted.as_slice());
//! ```

use crate::CryptoError;
use rand::RngCore;
use sm4::cipher::{BlockCipherDecBackend, BlockCipherEncBackend, KeyInit};
use sm4::Sm4;
use zeroize::Zeroize;

/// SM4 密钥（16 字节）
#[derive(Debug, Clone)]
pub struct Sm4Key([u8; 16]);

impl Zeroize for Sm4Key {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl Drop for Sm4Key {
    fn drop(&mut self) {
        self.zeroize();
    }
}

impl Sm4Key {
    /// 生成随机密钥
    pub fn random() -> Self {
        let mut key = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut key);
        Self(key)
    }

    /// 从字节切片创建密钥
    pub fn from_bytes(bytes: &[u8; 16]) -> Self {
        Self(*bytes)
    }

    /// 从密码派生密钥（使用 SM3 哈希）
    pub fn derive_from(input: &[u8]) -> Self {
        use sm3::{Digest, Sm3};
        let mut hasher = Sm3::new();
        hasher.update(input);
        let result = hasher.finalize();
        let mut key = [0u8; 16];
        key.copy_from_slice(&result[..16]);
        Self(key)
    }

    /// 获取密钥字节
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl AsRef<[u8]> for Sm4Key {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 16]> for Sm4Key {
    fn from(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

/// PKCS#7 填充
fn pkcs7_pad(data: &[u8]) -> Vec<u8> {
    let block_size = 16;
    let pad_len = block_size - (data.len() % block_size);
    let mut padded = data.to_vec();
    padded.extend(std::iter::repeat(pad_len as u8).take(pad_len));
    padded
}

/// PKCS#7 去填充
fn pkcs7_unpad(data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if data.is_empty() {
        return Err(CryptoError::Sm4Error("empty input".into()));
    }
    let pad_len = *data.last().unwrap() as usize;
    if pad_len == 0 || pad_len > 16 {
        return Err(CryptoError::Sm4Error("invalid padding length".into()));
    }
    if data.len() < pad_len {
        return Err(CryptoError::Sm4Error("data too short for padding".into()));
    }
    // 验证填充字节
    for i in 0..pad_len {
        if data[data.len() - 1 - i] != pad_len as u8 {
            return Err(CryptoError::Sm4Error("invalid padding bytes".into()));
        }
    }
    let mut result = data.to_vec();
    result.truncate(data.len() - pad_len);
    Ok(result)
}

/// CBC 模式加密
///
/// 返回 `iv || ciphertext`
pub fn encrypt_cbc(plaintext: &[u8], key: &Sm4Key, iv: &[u8; 16]) -> Vec<u8> {
    let cipher = Sm4::new_from_slice(key.as_bytes()).expect("valid key length");
    let mut result = Vec::with_capacity(iv.len() + plaintext.len() + 16);
    result.extend_from_slice(iv);

    let mut prev_block = *iv;
    let padded = pkcs7_pad(plaintext);

    for chunk in padded.chunks(16) {
        let mut block = [0u8; 16];
        block[..chunk.len()].copy_from_slice(chunk);
        
        // XOR with previous ciphertext block (or IV for first block)
        for (b, p) in block.iter_mut().zip(prev_block.iter()) {
            *b ^= p;
        }
        
        cipher.encrypt_block_inplace((&mut block).into());
        result.extend_from_slice(&block);
        prev_block = block;
    }

    result
}

/// CBC 模式解密
///
/// 输入格式：`iv || ciphertext`
pub fn decrypt_cbc(ciphertext_with_iv: &[u8], key: &Sm4Key) -> Result<Vec<u8>, CryptoError> {
    if ciphertext_with_iv.len() < 32 {
        return Err(CryptoError::Sm4Error("ciphertext too short".into()));
    }
    if (ciphertext_with_iv.len() - 16) % 16 != 0 {
        return Err(CryptoError::Sm4Error("ciphertext length not multiple of block size".into()));
    }

    let iv = &ciphertext_with_iv[..16];
    let ciphertext = &ciphertext_with_iv[16..];

    let cipher = Sm4::new_from_slice(key.as_bytes()).expect("valid key length");
    let mut prev_block = [0u8; 16];
    prev_block.copy_from_slice(iv);

    let mut plaintext = Vec::with_capacity(ciphertext.len());

    for chunk in ciphertext.chunks_exact(16) {
        let mut block: [u8; 16] = chunk.try_into().unwrap();
        cipher.decrypt_block_inplace((&mut block).into());
        
        // P_i = D(C_i) XOR C_{i-1}
        for (dec, prev) in block.iter().zip(prev_block.iter()) {
            plaintext.push(dec ^ prev);
        }
        
        prev_block.copy_from_slice(chunk);
    }

    pkcs7_unpad(&plaintext)
}

/// ECB 模式加密（无填充，输入必须是 16 字节的倍数）
pub fn encrypt_ecb(plaintext: &[u8], key: &Sm4Key) -> Result<Vec<u8>, CryptoError> {
    if plaintext.len() % 16 != 0 {
        return Err(CryptoError::Sm4Error("plaintext length must be multiple of 16".into()));
    }
    
    let cipher = Sm4::new_from_slice(key.as_bytes()).expect("valid key length");
    let mut result = Vec::with_capacity(plaintext.len());
    
    for chunk in plaintext.chunks(16) {
        let mut block: [u8; 16] = chunk.try_into().unwrap();
        cipher.encrypt_block_inplace((&mut block).into());
        result.extend_from_slice(&block);
    }
    
    Ok(result)
}

/// ECB 模式解密（无填充，输入必须是 16 字节的倍数）
pub fn decrypt_ecb(ciphertext: &[u8], key: &Sm4Key) -> Result<Vec<u8>, CryptoError> {
    if ciphertext.len() % 16 != 0 {
        return Err(CryptoError::Sm4Error("ciphertext length must be multiple of 16".into()));
    }
    
    let cipher = Sm4::new_from_slice(key.as_bytes()).expect("valid key length");
    let mut result = Vec::with_capacity(ciphertext.len());
    
    for chunk in ciphertext.chunks(16) {
        let mut block: [u8; 16] = chunk.try_into().unwrap();
        cipher.decrypt_block_inplace((&mut block).into());
        result.extend_from_slice(&block);
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sm4_key_derivation() {
        let input = b"password";
        let key = Sm4Key::derive_from(input);
        assert_eq!(key.as_bytes().len(), 16);

        // 可重复性
        let key2 = Sm4Key::derive_from(input);
        assert_eq!(key.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_sm4_cbc_roundtrip() {
        let key = Sm4Key::random();
        let mut iv = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut iv);

        let plaintext = b"SM4 CBC encryption test data!";
        let ciphertext = encrypt_cbc(plaintext, &key, &iv);
        let decrypted = decrypt_cbc(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_sm4_cbc_with_long_message() {
        let key = Sm4Key::random();
        let mut iv = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut iv);

        let plaintext = vec![0x41; 100]; // 100 字节 'A'
        let ciphertext = encrypt_cbc(&plaintext, &key, &iv);
        let decrypted = decrypt_cbc(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_sm4_ecb_roundtrip() {
        let key = Sm4Key::random();
        let plaintext = [0x42u8; 32]; // 32 字节 'B'
        
        let ciphertext = encrypt_ecb(&plaintext, &key).unwrap();
        let decrypted = decrypt_ecb(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_sm4_ecb_wrong_length() {
        let key = Sm4Key::random();
        let plaintext = [0u8; 15]; // 15 字节，不是 16 的倍数
        
        assert!(encrypt_ecb(&plaintext, &key).is_err());
    }

    #[test]
    fn test_sm4_cbc_decrypt_wrong_padding() {
        let key = Sm4Key::random();
        let mut iv = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut iv);

        let plaintext = b"12345";
        let ciphertext = encrypt_cbc(plaintext, &key, &iv);
        let mut corrupted = ciphertext.clone();
        // Corrupt last byte to break padding
        *corrupted.last_mut().unwrap() = corrupted.last().unwrap().wrapping_add(1);

        let result = decrypt_cbc(&corrupted, &key);
        assert!(result.is_err());
    }
}
