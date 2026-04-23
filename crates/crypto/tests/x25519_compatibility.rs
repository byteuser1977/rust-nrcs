//! 测试 x25519-dalek 标准库与 NRCS 的一致性

use sha2::{Digest, Sha256};
use x25519_dalek::StaticSecret;

/// 测试数据来自 NRCS Java 实现
/// PassPhrase: confusion flirt teeth story crawl dear shove screw decay flood cover warrior
/// PublicKey: 5aafe59365b988d73aa15424a1b83c24fcada07e7cd73fcfade612c5bf32fc71
const TEST_PASSPHRASE: &str = "confusion flirt teeth story crawl dear shove screw decay flood cover warrior";
const EXPECTED_PUBLIC_KEY: &str = "5aafe59365b988d73aa15424a1b83c24fcada07e7cd73fcfade612c5bf32fc71";

#[test]
fn test_x25519_dalek_public_key_derivation() {
    let expected_pk_bytes = hex::decode(EXPECTED_PUBLIC_KEY).expect("Invalid hex");
    let expected_pk: [u8; 32] = expected_pk_bytes.try_into().expect("Invalid length");

    let seed = Sha256::digest(TEST_PASSPHRASE.as_bytes());
    let seed_array: [u8; 32] = seed.into();

    let secret = StaticSecret::from(seed_array);
    let public = x25519_dalek::PublicKey::from(&secret);
    let derived_pk: [u8; 32] = *public.as_bytes();

    println!("Expected public key: {}", hex::encode(expected_pk));
    println!("Derived public key:  {}", hex::encode(derived_pk));

    assert_eq!(derived_pk, expected_pk, "x25519-dalek public key does not match NRCS expected value");
}

#[test]
fn test_x25519_dalek_key_exchange() {
    let seed1: [u8; 32] = Sha256::digest(b"alice secret phrase").into();
    let seed2: [u8; 32] = Sha256::digest(b"bob secret phrase").into();

    let secret1 = StaticSecret::from(seed1);
    let secret2 = StaticSecret::from(seed2);

    let public1 = x25519_dalek::PublicKey::from(&secret1);
    let public2 = x25519_dalek::PublicKey::from(&secret2);

    let shared1 = secret1.diffie_hellman(&public2);
    let shared2 = secret2.diffie_hellman(&public1);

    println!("Shared secret 1: {}", hex::encode(shared1.as_bytes()));
    println!("Shared secret 2: {}", hex::encode(shared2.as_bytes()));

    assert_eq!(shared1.as_bytes(), shared2.as_bytes(), "DH key exchange failed");
}

#[test]
fn test_another_passphrase() {
    let passphrase = "like just love know never want time out there make look eye";
    let seed: [u8; 32] = Sha256::digest(passphrase.as_bytes()).into();
    let secret = StaticSecret::from(seed);
    let public = x25519_dalek::PublicKey::from(&secret);

    println!("Passphrase: {}", passphrase);
    println!("Public key: {}", hex::encode(public.as_bytes()));
}
