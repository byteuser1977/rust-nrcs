//! AES-256-GCM 加密/解密测试
//!
//! 验证 AES-256-GCM 认证加密算法的正确性，确保与 Java NRCS (BouncyCastle) 兼容。

use crypto::algorithms::{AesGcm, GcmAlgorithm};
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
    let plaintext = b"AES-256-GCM encryption test";

    let (ciphertext, tag) = algo.encrypt_gcm(&key, &nonce, aad, plaintext).unwrap();
    let decrypted = algo.decrypt_gcm(&key, &nonce, aad, &ciphertext, &tag).unwrap();
    assert_eq!(plaintext, decrypted.as_slice());
}

#[test]
fn test_aes_gcm_empty_plaintext() {
    let algo = AesGcm;
    let mut key = [0u8; 32];
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut key);
    OsRng.fill_bytes(&mut nonce);

    let (ct, tag) = algo.encrypt_gcm(&key, &nonce, b"aad", b"").unwrap();
    let decrypted = algo.decrypt_gcm(&key, &nonce, b"aad", &ct, &tag).unwrap();
    assert!(decrypted.is_empty());
}

#[test]
fn test_aes_gcm_invalid_tag() {
    let algo = AesGcm;
    let mut key = [0u8; 32];
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut key);
    OsRng.fill_bytes(&mut nonce);

    let (ct, tag) = algo.encrypt_gcm(&key, &nonce, b"aad", b"test").unwrap();
    let mut bad_tag = tag.clone();
    bad_tag[0] = bad_tag[0].wrapping_add(1);
    assert!(algo.decrypt_gcm(&key, &nonce, b"aad", &ct, &bad_tag).is_err());
}

#[test]
fn test_aes_gcm_invalid_aad() {
    let algo = AesGcm;
    let mut key = [0u8; 32];
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut key);
    OsRng.fill_bytes(&mut nonce);

    let (ct, tag) = algo.encrypt_gcm(&key, &nonce, b"correct_aad", b"test").unwrap();
    assert!(algo.decrypt_gcm(&key, &nonce, b"wrong_aad", &ct, &tag).is_err());
}

#[test]
fn test_aes_gcm_wrong_key() {
    let algo = AesGcm;
    let mut key1 = [0u8; 32];
    let mut key2 = [0u8; 32];
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut key1);
    OsRng.fill_bytes(&mut key2);
    OsRng.fill_bytes(&mut nonce);

    let (ct, tag) = algo.encrypt_gcm(&key1, &nonce, b"aad", b"test").unwrap();
    assert!(algo.decrypt_gcm(&key2, &nonce, b"aad", &ct, &tag).is_err());
}

#[test]
fn test_aes_gcm_wrong_key_length() {
    let algo = AesGcm;
    let key = [0u8; 16];
    let nonce = [0u8; 12];
    let result = algo.encrypt_gcm(&key, &nonce, b"", b"test");
    assert!(result.is_err());
}

#[test]
fn test_aes_gcm_algorithm_properties() {
    let algo = AesGcm;
    assert_eq!(algo.name(), "aes-256-gcm");
    assert_eq!(algo.key_len(), 32);
    assert_eq!(algo.nonce_len(), 12);
    assert_eq!(algo.tag_len(), 16);
}

#[test]
fn test_aes_gcm_deterministic_with_same_inputs() {
    let algo = AesGcm;
    let key = [0x42u8; 32];
    let nonce = [0x13u8; 12];
    let aad = b"aad";
    let plaintext = b"deterministic test";

    let (ct1, tag1) = algo.encrypt_gcm(&key, &nonce, aad, plaintext).unwrap();
    let (ct2, tag2) = algo.encrypt_gcm(&key, &nonce, aad, plaintext).unwrap();
    assert_eq!(ct1, ct2, "Same inputs should produce same ciphertext");
    assert_eq!(tag1, tag2, "Same inputs should produce same tag");
}

#[test]
fn test_aes_gcm_ciphertext_differs_from_plaintext() {
    let algo = AesGcm;
    let mut key = [0u8; 32];
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut key);
    OsRng.fill_bytes(&mut nonce);

    let plaintext = b"plaintext data that should be encrypted";
    let (ct, _) = algo.encrypt_gcm(&key, &nonce, b"", plaintext).unwrap();
    assert_ne!(ct.as_slice(), plaintext);
}
