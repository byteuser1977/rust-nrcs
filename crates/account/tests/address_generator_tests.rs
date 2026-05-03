//! 地址生成测试
//!
//! 测试 Account ID 派生、RS 地址编码和地址验证。

use account::crypto::{generate_keypair, derive_account_id, derive_address, verify_address, AddressGenerator};

#[test]
fn test_generate_keypair() {
    let kp = generate_keypair();
    assert_eq!(kp.public_key().len(), 32, "Public key should be 32 bytes");
}

#[test]
fn test_generate_keypair_unique() {
    let kp1 = generate_keypair();
    let kp2 = generate_keypair();
    assert_ne!(kp1.public_key(), kp2.public_key(), "Two keypairs should be different");
}

#[test]
fn test_derive_account_id() {
    let kp = generate_keypair();
    let account_id = derive_account_id(&kp.public_key());
    assert_ne!(account_id, 0, "Account ID should be non-zero");
}

#[test]
fn test_derive_account_id_deterministic() {
    let kp = generate_keypair();
    let id1 = derive_account_id(&kp.public_key());
    let id2 = derive_account_id(&kp.public_key());
    assert_eq!(id1, id2, "Same public key should produce same account ID");
}

#[test]
fn test_derive_address() {
    let kp = generate_keypair();
    let account_id = derive_account_id(&kp.public_key());
    let address = derive_address(account_id);
    assert!(!address.is_empty(), "Address should not be empty");
    assert!(address.len() > 15, "Address should be reasonably long");
}

#[test]
fn test_verify_address_valid() {
    let kp = generate_keypair();
    let account_id = derive_account_id(&kp.public_key());
    let address = derive_address(account_id);
    assert!(verify_address(account_id, &address), "Address should verify against its account ID");
}

#[test]
fn test_verify_address_invalid() {
    let kp = generate_keypair();
    let account_id = derive_account_id(&kp.public_key());
    assert!(!verify_address(account_id, "invalid_address"), "Invalid address should not verify");
}

#[test]
fn test_address_generator_trait() {
    let generator = ();
    let (kp, account_id, address) = generator.generate_account();
    assert_eq!(kp.public_key().len(), 32);
    assert_ne!(account_id, 0);
    assert!(!address.is_empty());
    assert!(generator.verify_address(account_id, &address));
}

#[test]
fn test_different_public_keys_different_ids() {
    let kp1 = generate_keypair();
    let kp2 = generate_keypair();
    let id1 = derive_account_id(&kp1.public_key());
    let id2 = derive_account_id(&kp2.public_key());
    assert_ne!(id1, id2, "Different public keys should produce different account IDs");
}

#[test]
fn test_address_from_known_account_id() {
    let account_id: u64 = 1234567890;
    let address = derive_address(account_id);
    assert!(verify_address(account_id, &address));
    assert!(!verify_address(account_id + 1, &address));
}
