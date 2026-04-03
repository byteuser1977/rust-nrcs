use crypto::*;
use ed25519_dalek::{Keypair, Signer};

#[test]
fn test_generate_keypair_random() {
    let kp1 = generate_keypair();
    let kp2 = generate_keypair();

    // Keys should be different
    assert_ne!(kp1.secret.as_bytes(), kp2.secret.as_bytes());
    assert_ne!(kp1.public.as_bytes(), kp2.public.as_bytes());
}

#[test]
fn test_generate_keypair_sizes() {
    let kp = generate_keypair();
    assert_eq!(kp.secret.as_bytes().len(), 64); // Ed25519 secret key is 64 bytes
    assert_eq!(kp.public.as_bytes().len(), 32); // Ed25519 public key is 32 bytes
}

#[test]
fn test_derive_public_key_from_secret() {
    let kp = generate_keypair();
    let derived_pub = derive_public_key(&kp.secret);

    assert_eq!(derived_pub.as_bytes(), kp.public.as_bytes());
}

#[test]
fn test_derive_account_id() {
    let kp = generate_keypair();
    let account_id = derive_account_id(&kp.public);

    // Account ID should be non-zero
    assert_ne!(account_id, 0);

    // Deriving from same public key should give same ID
    let account_id2 = derive_account_id(&kp.public);
    assert_eq!(account_id, account_id2);
}

#[test]
fn test_derive_address_from_account_id() {
    let account_id = 1234567890u64;
    let address = derive_address(account_id);

    assert!(!address.is_empty());
    assert!(address.len() > 10);
    // Base58 addresses are alphanumeric
    assert!(address.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn test_address_derivation_consistency() {
    let kp = generate_keypair();
    let account_id = derive_account_id(&kp.public);
    let address = derive_address(account_id);

    // Verify address can be derived consistently
    let address2 = derive_address(account_id);
    assert_eq!(address, address2);
}

#[test]
fn test_private_key_encryption_decryption() {
    // If encryption functions are implemented
    let kp = generate_keypair();
    let encrypted = encrypt_private_key(&kp.secret, b"password123").unwrap();
    let decrypted = decrypt_private_key(&encrypted, b"password123").unwrap();

    assert_eq!(decrypted.as_bytes(), kp.secret.as_bytes());
}

#[test]
fn test_encryption_wrong_password() {
    let kp = generate_keypair();
    let encrypted = encrypt_private_key(&kp.secret, b"password123").unwrap();

    let result = decrypt_private_key(&encrypted, b"wrongpass");
    assert!(result.is_err());
}

#[test]
fn test_generate_multiple_keys_unique() {
    let mut keys = Vec::new();
    for _ in 0..10 {
        let kp = generate_keypair();
        keys.push(kp);
    }

    // All should be unique
    for i in 0..keys.len() {
        for j in i+1..keys.len() {
            assert_ne!(keys[i].secret.as_bytes(), keys[j].secret.as_bytes());
            assert_ne!(keys[i].public.as_bytes(), keys[j].public.as_bytes());
        }
    }
}

#[test]
fn test_sign_with_generated_key() {
    let kp = generate_keypair();
    let message = b"Test message for generated key";

    let signature = sign_ed25519(message, &kp.secret);
    assert!(verify_ed25519(message, &signature, &kp.public).is_ok());
}

#[test]
fn test_public_key_derivation_consistency() {
    let kp = generate_keypair();
    let derived_pub1 = derive_public_key(&kp.secret);
    let derived_pub2 = derive_public_key(&kp.secret);

    assert_eq!(derived_pub1.as_bytes(), derived_pub2.as_bytes());
    assert_eq!(derived_pub1.as_bytes(), kp.public.as_bytes());
}

#[test]
fn test_account_id_uniqueness() {
    // Generate many keypairs and ensure account IDs are unique
    let mut ids = Vec::new();
    for _ in 0..100 {
        let kp = generate_keypair();
        let id = derive_account_id(&kp.public);
        assert!(!ids.contains(&id), "Duplicate account ID found: {}", id);
        ids.push(id);
    }
}
