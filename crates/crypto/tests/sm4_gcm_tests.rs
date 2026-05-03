//! SM4-GCM 加密/解密测试
//!
//! 验证 SM4-GCM 认证加密算法的正确性。

use crypto::algorithms::{Sm4Gcm, GcmAlgorithm};
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
    let plaintext = b"SM4-GCM encryption test";

    let (ciphertext, tag) = algo.encrypt_gcm(&key, &nonce, aad, plaintext).unwrap();
    let decrypted = algo.decrypt_gcm(&key, &nonce, aad, &ciphertext, &tag).unwrap();
    assert_eq!(plaintext, decrypted.as_slice());
}

#[test]
fn test_sm4_gcm_empty_plaintext() {
    let algo = Sm4Gcm;
    let mut key = [0u8; 16];
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut key);
    OsRng.fill_bytes(&mut nonce);

    let (ciphertext, tag) = algo.encrypt_gcm(&key, &nonce, b"", b"").unwrap();
    let decrypted = algo.decrypt_gcm(&key, &nonce, b"", &ciphertext, &tag).unwrap();
    assert_eq!(b"", decrypted.as_slice());
}

#[test]
fn test_sm4_gcm_invalid_tag() {
    let algo = Sm4Gcm;
    let mut key = [0u8; 16];
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut key);
    OsRng.fill_bytes(&mut nonce);

    let (ct, tag) = algo.encrypt_gcm(&key, &nonce, b"aad", b"test").unwrap();
    let mut bad_tag = tag.clone();
    bad_tag[0] = bad_tag[0].wrapping_add(1);
    assert!(algo.decrypt_gcm(&key, &nonce, b"aad", &ct, &bad_tag).is_err());
}

#[test]
fn test_sm4_gcm_invalid_aad() {
    let algo = Sm4Gcm;
    let mut key = [0u8; 16];
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut key);
    OsRng.fill_bytes(&mut nonce);

    let (ct, tag) = algo.encrypt_gcm(&key, &nonce, b"correct_aad", b"test").unwrap();
    assert!(algo.decrypt_gcm(&key, &nonce, b"wrong_aad", &ct, &tag).is_err());
}

#[test]
fn test_sm4_gcm_wrong_key() {
    let algo = Sm4Gcm;
    let mut key1 = [0u8; 16];
    let mut key2 = [0u8; 16];
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut key1);
    OsRng.fill_bytes(&mut key2);
    OsRng.fill_bytes(&mut nonce);

    let (ct, tag) = algo.encrypt_gcm(&key1, &nonce, b"aad", b"test").unwrap();
    assert!(algo.decrypt_gcm(&key2, &nonce, b"aad", &ct, &tag).is_err());
}

#[test]
fn test_sm4_gcm_wrong_key_length() {
    let algo = Sm4Gcm;
    let key = [0u8; 32];
    let nonce = [0u8; 12];
    let result = algo.encrypt_gcm(&key, &nonce, b"", b"test");
    assert!(result.is_err());
}

#[test]
fn test_sm4_gcm_wrong_nonce_length() {
    let algo = Sm4Gcm;
    let key = [0u8; 16];
    let nonce = [0u8; 8];
    let result = algo.encrypt_gcm(&key, &nonce, b"", b"test");
    assert!(result.is_err());
}

#[test]
fn test_sm4_gcm_algorithm_properties() {
    let algo = Sm4Gcm;
    assert_eq!(algo.name(), "sm4-gcm");
    assert_eq!(algo.key_len(), 16);
    assert_eq!(algo.nonce_len(), 12);
    assert_eq!(algo.tag_len(), 16);
}

#[test]
fn test_sm4_gcm_large_plaintext() {
    let algo = Sm4Gcm;
    let mut key = [0u8; 16];
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut key);
    OsRng.fill_bytes(&mut nonce);

    let plaintext = vec![0x55u8; 10000];
    let (ct, tag) = algo.encrypt_gcm(&key, &nonce, b"aad", &plaintext).unwrap();
    assert_ne!(ct, plaintext);
    let decrypted = algo.decrypt_gcm(&key, &nonce, b"aad", &ct, &tag).unwrap();
    assert_eq!(plaintext, decrypted);
}
