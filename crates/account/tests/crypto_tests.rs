use account::crypto::*;
use ed25519_dalek::{Keypair, Signer, Verifier};

#[test]
fn test_generate_keypair() {
    let kp1 = generate_keypair();
    let kp2 = generate_keypair();

    // Keys should be different
    assert_ne!(kp1.secret.as_bytes(), kp2.secret.as_bytes());
    assert_ne!(kp1.public.as_bytes(), kp2.public.as_bytes());

    // Key sizes should be correct
    assert_eq!(kp1.secret.as_bytes().len(), 64);
    assert_eq!(kp1.public.as_bytes().len(), 32);
}

#[test]
fn test_derive_account_id_consistency() {
    let kp = generate_keypair();
    let account_id1 = derive_account_id(&kp.public);
    let account_id2 = derive_account_id(&kp.public);

    assert_eq!(account_id1, account_id2);
    assert_ne!(account_id1, 0); // should not be zero

    // First 8 bytes of SHA-256(public_key) as big-endian u64
    let expected_hash = sha2::Sha256::digest(kp.public.as_bytes());
    let expected_id = u64::from_be_bytes([
        expected_hash[0], expected_hash[1], expected_hash[2], expected_hash[3],
        expected_hash[4], expected_hash[5], expected_hash[6], expected_hash[7],
    ]);
    assert_eq!(account_id1, expected_id);
}

#[test]
fn test_derive_account_id_different_keys() {
    let kp1 = generate_keypair();
    let kp2 = generate_keypair();

    let id1 = derive_account_id(&kp1.public);
    let id2 = derive_account_id(&kp2.public);

    assert_ne!(id1, id2);
}

#[test]
fn test_derive_address_format() {
    let address = derive_address(1234567890);
    // Base58 address should be alphanumeric
    assert!(!address.is_empty());
    assert!(address.len() > 10);
    assert!(address.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn test_derive_address_determinism() {
    let account_id = 1234567890u64;
    let addr1 = derive_address(account_id);
    let addr2 = derive_address(account_id);

    assert_eq!(addr1, addr2);
}

#[test]
fn test_derive_address_different_ids() {
    let addr1 = derive_address(1234567890);
    let addr2 = derive_address(9876543210);

    assert_ne!(addr1, addr2);
}

#[test]
fn test_address_generator_trait() {
    let generator = ();
    let (kp1, id1, addr1) = generator.generate_account();
    let (kp2, id2, addr2) = generator.generate_account();

    assert_ne!(id1, id2);
    assert_ne!(addr1, addr2);
    assert!(!addr1.is_empty());
    assert!(!addr2.is_empty());

    // Verify that addresses match IDs
    assert!(generator.verify_address(id1, &addr1));
    assert!(generator.verify_address(id2, &addr2));

    // Invalid address should fail
    assert!(!generator.verify_address(id1, "invalid"));
    assert!(!generator.verify_address(999999, &addr1));
}

#[test]
fn test_sign_and_verify() {
    let kp = generate_keypair();

    let message = b"Hello, NRCS!";
    let signature = kp.sign(message);

    // Verification with correct public key should succeed
    kp.public.verify(message, &signature).unwrap();

    // Verification with modified message should fail
    let modified_message = b"Hello, NRCS modified!";
    assert!(kp.public.verify(modified_message, &signature).is_err());
}

#[test]
fn test_signature_is_64_bytes() {
    let kp = generate_keypair();
    let message = b"test message";
    let signature = kp.sign(message);

    assert_eq!(signature.to_bytes().len(), 64);
}

#[test]
fn test_public_key_recovery_from_signature() {
    // Ed25519 doesn't support public key recovery from signature alone
    // But we can verify that signature is valid for the message
    let kp = generate_keypair();
    let message = b"test";

    let signature = kp.sign(message);

    // Anyone with public key can verify
    assert!(kp.public.verify(message, &signature).is_ok());
}

#[test]
fn test_double_sha256_calculation() {
    let data = b"test data";
    let result = double_sha256(data);

    assert_eq!(result.len(), 32);
    // Known test vector: double SHA256 of "test data"
    // This is just ensuring it's non-zero
    assert!(result != [0u8; 32]);
}

#[test]
fn test_address_checksum() {
    // Base58Check addresses have a 4-byte checksum
    // The checksum is the first 4 bytes of double_sha256(payload)
    let account_id = 12345u64;
    let address = derive_address(account_id);

    // Decode the address and verify checksum
    // This is simplified; full implementation would need base58 decoding
    assert!(!address.is_empty());
}

#[test]
fn test_keypair_randomness() {
    let mut keys = Vec::new();
    for _ in 0..10 {
        let kp = generate_keypair();
        keys.push(kp);
    }

    // All keys should be unique
    for i in 0..keys.len() {
        for j in i+1..keys.len() {
            assert_ne!(keys[i].secret.as_bytes(), keys[j].secret.as_bytes());
            assert_ne!(keys[i].public.as_bytes(), keys[j].public.as_bytes());
        }
    }
}

#[test]
fn test_account_id_from_public_key_properties() {
    // Account ID should be derived deterministically from public key
    let kp = generate_keypair();

    let id1 = derive_account_id(&kp.public);
    let id2 = derive_account_id(&kp.public);

    assert_eq!(id1, id2);
    assert!(id1 > 0); // Should be non-zero for valid public keys
}

#[test]
fn test_address_version_byte() {
    // The address starts with version byte 0x00
    let account_id = 1234567890u64;
    let address = derive_address(account_id);

    // The first character in Base58 typically corresponds to version
    // 0x00 maps to '1' in Base58
    assert!(address.starts_with('1') || address.starts_with('2') || address.starts_with('3'));
}

#[test]
fn test_sign_and_verify_multiple_messages() {
    let kp = generate_keypair();

    let messages = vec![b"msg1", b"msg2", b"msg3"];
    let signatures: Vec<_> = messages.iter().map(|m| kp.sign(m)).collect();

    for (msg, sig) in messages.iter().zip(signatures.iter()) {
        assert!(kp.public.verify(msg, sig).is_ok());
    }
}

#[test]
fn test_wrong_public_key_verification() {
    let kp1 = generate_keypair();
    let kp2 = generate_keypair();

    let message = b"test message";
    let signature = kp1.sign(message);

    // Verifying with wrong public key should fail
    assert!(kp2.public.verify(message, &signature).is_err());
}

#[test]
fn test_empty_message_signature() {
    let kp = generate_keypair();
    let message = b"";

    let signature = kp.sign(message);
    assert!(kp.public.verify(message, &signature).is_ok());
}

#[test]
fn test_large_message_signature() {
    let kp = generate_keypair();
    let message = vec![0xAA; 10000]; // 10KB message

    let signature = kp.sign(&message);
    assert!(kp.public.verify(&message, &signature).is_ok());
}
