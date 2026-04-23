//! SM4 对称加密实现（CBC 和 GCM 模式）
//!
//! 本模块提供：
//! - CBC 模式（带 PKCS#7 填充）
//! - GCM 模式（认证加密）
//!
//! 接口设计参考 Java 的 AES-CBC/GCM 实现：
//! - 加密返回 `iv || ciphertext`
//! - 解密接受 `iv || ciphertext` 格式

use crate::CryptoError;
use block_cipher_trait::BlockCipher;
use sm4::Sm4;
use zeroize::Zeroize;

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
        return Err(CryptoError::Sm4Error(
            "invalid padding length".into(),
        ));
    }
    if data.len() < pad_len {
        return Err(CryptoError::Sm4Error(
            "data too short for padding".into(),
        ));
    }
    let mut result = data.to_vec();
    result.truncate(data.len() - pad_len);
    Ok(result)
}

/// CBC 加密
///
/// 返回 `iv || ciphertext`
pub fn encrypt_cbc(plaintext: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> Vec<u8> {
    let mut cipher = Sm4::new_from_slice(key).expect("valid key length");
    let mut result = Vec::with_capacity(iv.len() + plaintext.len() + 16);
    result.extend_from_slice(iv);

    let mut prev_block = *iv;
    let padded = pkcs7_pad(plaintext);

    for chunk in padded.chunks(16) {
        // XOR with previous ciphertext block (or IV for first block)
        let mut block = [0u8; 16];
        for (b, &p) in block.iter_mut().zip(chunk) {
            *b = p ^ prev_block[b as usize];
        }
        cipher.encrypt_block(&mut block);
        result.extend_from_slice(&block);
        prev_block = block;
    }

    result
}

/// CBC 解密
///
/// 输入格式：`iv || ciphertext`
pub fn decrypt_cbc(ciphertext_with_iv: &[u8], key: &[u8; 16]) -> Result<Vec<u8>, CryptoError> {
    if ciphertext_with_iv.len() < 16 {
        return Err(CryptoError::Sm4Error(
            "ciphertext too short".into(),
        ));
    }
    if (ciphertext_with_iv.len() - 16) % 16 != 0 {
        return Err(CryptoError::Sm4Error(
            "ciphertext length not multiple of block size".into(),
        ));
    }

    let iv = &ciphertext_with_iv[..16];
    let ciphertext = &ciphertext_with_iv[16:];

    let mut cipher = Sm4::new_from_slice(key).expect("valid key length");
    let mut prev_block = [0u8; 16];
    prev_block.copy_from_slice(iv);

    let mut plaintext = Vec::with_capacity(ciphertext.len());

    for chunk in ciphertext.chunks_exact(16) {
        let mut block = chunk.try_into().unwrap();
        cipher.decrypt_block(&mut block);
        // P_i = D(C_i) XOR C_{i-1}
        for (p, (dec, prev)) in plaintext
            .iter_mut()
            .zip(block.iter().zip(prev_block.iter()))
        {
            *p = dec ^ prev;
        }
        prev_block = [0u8; 16];
        prev_block.copy_from_slice(chunk);
    }

    pkcs7_unpad(&plaintext)
}

/// GCM 模式加密
///
/// # 参数
/// - `plaintext`: 明文
/// - `key`: 16 字节密钥
/// - `nonce`: 96 位 nonce（12 字节）
/// - `aad`: 附加认证数据（如头部信息）
///
/// # 返回
/// `(ciphertext, tag)`，tag 长度为 16 字节（128 位认证标签）
pub fn encrypt_gcm(
    plaintext: &[u8],
    key: &[u8; 16],
    nonce: &[u8; 12],
    aad: &[u8],
) -> Result<(Vec<u8>, [u8; 16]), CryptoError> {
    use sm4_gcm::{
        aead::{Aead, NewAead},
        Sm4Gcm,
        generic_array::GenericArray,
    };

    let key_arr = GenericArray::from_slice(key);
    let cipher = Sm4Gcm::new(key_arr);

    let nonce_arr = GenericArray::from_slice(nonce);
    let aad_arr = GenericArray::from_slice(aad);

    // 加密：返回 ciphertext || tag 组合
    let combined = cipher
        .encrypt(nonce_arr, aad_arr, plaintext)
        .map_err(|_| CryptoError::Sm4Error("GCM encryption failed".into()))?;

    // 分离 tag（最后 16 字节）
    let tag_len = 16;
    if combined.len() < tag_len {
        return Err(CryptoError::Sm4Error(
            "GCM output too short".into(),
        ));
    }
    let (ct_bytes, tag_bytes) = combined.split_at(combined.len() - tag_len);
    let mut tag = [0u8; 16];
    tag.copy_from_slice(tag_bytes);

    Ok((ct_bytes.to_vec(), tag))
}

/// GCM 模式解密
///
/// 输入 `ciphertext`（不含 tag）
/// 返回解密后的明文，并通过 tag 验证完整性
pub fn decrypt_gcm(
    ciphertext: &[u8],
    key: &[u8; 16],
    nonce: &[u8; 12],
    tag: &[u8; 16],
    aad: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    use sm4_gcm::{
        aead::{Aead, NewAead},
        Sm4Gcm,
        generic_array::GenericArray,
    };

    let key_arr = GenericArray::from_slice(key);
    let cipher = Sm4Gcm::new(key_arr);

    let nonce_arr = GenericArray::from_slice(nonce);
    let aad_arr = GenericArray::from_slice(aad);
    let tag_arr = GenericArray::from_slice(tag);

    // 合并 ciphertext 和 tag 用于解密验证
    let mut buffer = ciphertext.to_vec();
    buffer.extend_from_slice(tag);

    cipher
        .decrypt(nonce_arr, aad_arr, &buffer)
        .map_err(|_| CryptoError::Sm4Error(
            "GCM authentication failed or decryption error".into(),
        ))
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
    fn test_sm4_gcm_roundtrip() {
        let key = Sm4Key::random();
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        let aad = b"header data";

        let plaintext = b"SM4-GCM authenticated encryption";
        let (ciphertext, tag) = encrypt_gcm(plaintext, &key, &nonce, aad)
            .expect("encryption success");
        let decrypted = decrypt_gcm(&ciphertext, &key, &nonce, &tag, aad)
            .expect("decryption success");

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_sm4_gcm_wrong_tag() {
        let key = Sm4Key::random();
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);

        let plaintext = b"test message";
        let (ciphertext, tag) = encrypt_gcm(plaintext, &key, &nonce, b"").unwrap();

        // 修改 tag
        let mut bad_tag = tag;
        bad_tag[0] = bad_tag[0].wrapping_add(1);
        let result = decrypt_gcm(&ciphertext, &key, &nonce, &bad_tag, b"");

        assert!(result.is_err());
    }

    #[test]
    fn test_sm4_gcm_wrong_key() {
        let key1 = Sm4Key::random();
        let key2 = Sm4Key::random();
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);

        let plaintext = b"test";
        let (ciphertext, tag) = encrypt_gcm(plaintext, &key1, &nonce, b"").unwrap();

        // Wrong key should fail
        let result = decrypt_gcm(&ciphertext, &key2, &nonce, &tag, b"");
        assert!(result.is_err());
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
