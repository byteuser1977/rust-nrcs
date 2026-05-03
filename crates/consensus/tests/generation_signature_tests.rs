//! 生成签名计算/验证测试
//!
//! 参照 Java NRCS 的 GeneratorTest.java，验证生成签名的计算和验证逻辑。

use consensus::generation_signature::*;
use blockchain_types::AccountId;

#[test]
fn test_calculate_generation_signature_basic() {
    let prev_sig = vec![1u8; 32];
    let account_id: AccountId = 12345;
    let sig = calculate_generation_signature(&prev_sig, account_id);
    assert_eq!(sig.len(), 32, "Generation signature should be 32 bytes");
    assert!(is_valid_generation_signature(&sig));
}

#[test]
fn test_calculate_generation_signature_deterministic() {
    let prev_sig = vec![42u8; 32];
    let account_id: AccountId = 98765;
    let sig1 = calculate_generation_signature(&prev_sig, account_id);
    let sig2 = calculate_generation_signature(&prev_sig, account_id);
    assert_eq!(sig1, sig2, "Same inputs should produce same generation signature");
}

#[test]
fn test_calculate_generation_signature_different_account() {
    let prev_sig = vec![1u8; 32];
    let sig1 = calculate_generation_signature(&prev_sig, 111);
    let sig2 = calculate_generation_signature(&prev_sig, 222);
    assert_ne!(sig1, sig2, "Different accounts should produce different signatures");
}

#[test]
fn test_calculate_generation_signature_different_prev_sig() {
    let prev_sig1 = vec![1u8; 32];
    let prev_sig2 = vec![2u8; 32];
    let account_id: AccountId = 12345;
    let sig1 = calculate_generation_signature(&prev_sig1, account_id);
    let sig2 = calculate_generation_signature(&prev_sig2, account_id);
    assert_ne!(sig1, sig2, "Different prev signatures should produce different results");
}

#[test]
fn test_verify_generation_signature_valid() {
    let prev_sig = vec![1u8; 32];
    let account_id: AccountId = 12345;
    let sig = calculate_generation_signature(&prev_sig, account_id);
    assert!(verify_generation_signature(&sig, &prev_sig, account_id));
}

#[test]
fn test_verify_generation_signature_invalid() {
    let prev_sig = vec![1u8; 32];
    let account_id: AccountId = 12345;
    let sig = calculate_generation_signature(&prev_sig, account_id);
    assert!(!verify_generation_signature(&sig, &prev_sig, 99999));
}

#[test]
fn test_calculate_generation_signature_with_public_key() {
    let prev_sig = vec![1u8; 32];
    let public_key = vec![0xABu8; 32];
    let sig = calculate_generation_signature_with_public_key(&prev_sig, &public_key);
    assert_eq!(sig.len(), 32);
    assert!(is_valid_generation_signature(&sig));
}

#[test]
fn test_calculate_generation_signature_hash() {
    let gen_sig = vec![1u8; 32];
    let public_key = vec![2u8; 32];
    let hash = calculate_generation_signature_hash(&gen_sig, &public_key);
    assert_eq!(hash.len(), 32);
}

#[test]
fn test_derive_hit_from_signature() {
    let gen_sig = vec![1u8; 32];
    let public_key = vec![2u8; 32];
    let hit = derive_hit_from_signature(&gen_sig, &public_key);
    assert_ne!(hit, 0, "Hit should be non-zero for non-zero inputs");
}

#[test]
fn test_derive_hit_deterministic() {
    let gen_sig = vec![1u8; 32];
    let public_key = vec![2u8; 32];
    let hit1 = derive_hit_from_signature(&gen_sig, &public_key);
    let hit2 = derive_hit_from_signature(&gen_sig, &public_key);
    assert_eq!(hit1, hit2, "Same inputs should produce same hit");
}

#[test]
fn test_derive_hit_different_keys() {
    let gen_sig = vec![1u8; 32];
    let pk1 = vec![1u8; 32];
    let pk2 = vec![2u8; 32];
    let hit1 = derive_hit_from_signature(&gen_sig, &pk1);
    let hit2 = derive_hit_from_signature(&gen_sig, &pk2);
    assert_ne!(hit1, hit2, "Different public keys should produce different hits");
}

#[test]
fn test_is_valid_generation_signature_all_zeros() {
    let zero_sig = vec![0u8; 32];
    assert!(!is_valid_generation_signature(&zero_sig), "All-zero signature should be invalid");
}

#[test]
fn test_is_valid_generation_signature_nonzero() {
    let sig = vec![1u8; 32];
    assert!(is_valid_generation_signature(&sig));
}

#[test]
fn test_next_generation_signature_chain() {
    let gen_sig1 = vec![1u8; 32];
    let pk1 = vec![0xAAu8; 32];
    let gen_sig2 = calculate_next_generation_signature(&gen_sig1, &pk1);
    assert_eq!(gen_sig2.len(), 32);
    assert_ne!(gen_sig1, gen_sig2);
    let pk2 = vec![0xBBu8; 32];
    let gen_sig3 = calculate_next_generation_signature(&gen_sig2, &pk2);
    assert_ne!(gen_sig2, gen_sig3);
}
