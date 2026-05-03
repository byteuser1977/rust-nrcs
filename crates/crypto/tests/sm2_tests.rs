//! SM2 加密/解密测试
//!
//! 参照 Java NRCS 的 TestSM2.java，验证 SM2 算法的加密和解密功能。

use crypto::algorithms::{Sm2, SignatureAlgorithm};
use crypto::{KeyPair, PublicKey, SecretKey};

#[test]
fn test_sm2_keypair_generation() {
    let kp = Sm2.generate_keypair();
    match kp {
        KeyPair::Sm2 { public_key, secret_key } => {
            assert_eq!(public_key.len(), 65, "SM2 public key should be 65 bytes (uncompressed)");
            assert_eq!(secret_key.len(), 32, "SM2 secret key should be 32 bytes");
            assert_eq!(public_key[0], 0x04, "Uncompressed point should start with 0x04");
        }
        _ => panic!("Expected Sm2 keypair"),
    }
}

#[test]
fn test_sm2_sign_verify_roundtrip() {
    let kp = Sm2.generate_keypair();
    let msg = b"SM2 test message for signing";
    let sig = Sm2.sign(&kp.secret_key(), msg);
    assert_eq!(sig.len(), 64, "SM2 signature should be 64 bytes");
    assert!(Sm2.verify(&kp.public_key(), msg, &sig).is_ok());
}

#[test]
fn test_sm2_tampered_message_fails() {
    let kp = Sm2.generate_keypair();
    let msg = b"original message";
    let sig = Sm2.sign(&kp.secret_key(), msg);
    let wrong_msg = b"tampered message";
    assert!(Sm2.verify(&kp.public_key(), wrong_msg, &sig).is_err());
}

#[test]
fn test_sm2_tampered_signature_fails() {
    let kp = Sm2.generate_keypair();
    let msg = b"test message";
    let mut sig = Sm2.sign(&kp.secret_key(), msg);
    sig[0] = sig[0].wrapping_add(1);
    assert!(Sm2.verify(&kp.public_key(), msg, &sig).is_err());
}

#[test]
fn test_sm2_deterministic_from_seed() {
    let seed = [0x42u8; 32];
    let kp1 = Sm2.from_seed(&seed);
    let kp2 = Sm2.from_seed(&seed);

    match (kp1, kp2) {
        (KeyPair::Sm2 { public_key: pk1, secret_key: sk1 }, KeyPair::Sm2 { public_key: pk2, secret_key: sk2 }) => {
            assert_eq!(pk1, pk2, "Same seed should produce same public key");
            assert_eq!(sk1, sk2, "Same seed should produce same secret key");
        }
        _ => panic!("Expected Sm2 keypairs"),
    }
}

#[test]
fn test_sm2_different_seeds_different_keys() {
    let seed1 = [0x01u8; 32];
    let seed2 = [0x02u8; 32];
    let kp1 = Sm2.from_seed(&seed1);
    let kp2 = Sm2.from_seed(&seed2);

    match (kp1, kp2) {
        (KeyPair::Sm2 { public_key: pk1, .. }, KeyPair::Sm2 { public_key: pk2, .. }) => {
            assert_ne!(pk1, pk2, "Different seeds should produce different keys");
        }
        _ => panic!("Expected Sm2 keypairs"),
    }
}

#[test]
fn test_sm2_sign_empty_message() {
    let kp = Sm2.generate_keypair();
    let msg = b"";
    let sig = Sm2.sign(&kp.secret_key(), msg);
    assert!(Sm2.verify(&kp.public_key(), msg, &sig).is_ok());
}

#[test]
fn test_sm2_sign_large_message() {
    let kp = Sm2.generate_keypair();
    let msg = vec![0xABu8; 10000];
    let sig = Sm2.sign(&kp.secret_key(), &msg);
    assert!(Sm2.verify(&kp.public_key(), &msg, &sig).is_ok());
}

#[test]
fn test_sm2_cross_keypair_verify_fails() {
    let kp1 = Sm2.generate_keypair();
    let kp2 = Sm2.generate_keypair();
    let msg = b"test message";
    let sig = Sm2.sign(&kp1.secret_key(), msg);
    assert!(Sm2.verify(&kp2.public_key(), msg, &sig).is_err());
}
