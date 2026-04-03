use blockchain_types::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_transaction_verify_signature_valid() {
    use ed25519_dalek::{Keypair, Signer};

    let keypair = Keypair::generate(&mut rand::rngs::OsRng);
    let public_key = keypair.public.as_bytes();

    let mut tx = Transaction::new(
        TransactionType::Payment,
        1234567890,
        Some(9876543210),
        1_000_000_000,
        100_000,
        1700000000,
        32767,
    );
    tx.full_hash = tx.compute_hash().unwrap();
    let data_to_sign = tx.serialize_for_signing();
    tx.signature = keypair.sign(&data_to_sign).to_bytes();

    assert!(tx.verify_signature(public_key).is_ok());
}

#[test]
fn test_transaction_verify_signature_invalid() {
    use ed25519_dalek::{Keypair, Signer};

    let keypair1 = Keypair::generate(&mut rand::rngs::OsRng);
    let keypair2 = Keypair::generate(&mut rand::rngs::OsRng);
    let public_key2 = keypair2.public.as_bytes();

    let mut tx = Transaction::new(
        TransactionType::Payment,
        1234567890,
        Some(9876543210),
        1_000_000_000,
        100_000,
        1700000000,
        32767,
    );
    tx.full_hash = tx.compute_hash().unwrap();
    let data_to_sign = tx.serialize_for_signing();
    // Sign with keypair1
    tx.signature = keypair1.sign(&data_to_sign).to_bytes();
    // But verify with keypair2's public key
    let result = tx.verify_signature(public_key2);
    assert!(result.is_err());
}

#[test]
fn test_block_generate_and_verify_signature() {
    use ed25519_dalek::{Keypair, Signer};

    let keypair = Keypair::generate(&mut rand::rngs::OsRng);
    let public_key = keypair.public.as_bytes();

    let mut block = Block::new(1, [0u8; 32], 1234567890);
    block.timestamp = 1700000000;
    block.total_amount = 1000;
    block.payload_length = 0;
    block.payload_hash = [0u8; 32];
    block.generation_signature = [0u8; 64];
    block.block_signature = [0u8; 64];

    // The block_signature should be set by the generator
    let header_data = block.serialize_header_for_signing();
    block.block_signature = keypair.sign(&header_data).to_bytes();

    assert!(block.verify_signature(public_key).is_ok());
}

#[test]
fn test_transaction_hash_consistency_across_serde() {
    let tx_original = Transaction {
        version: 1,
        type_id: TransactionType::Payment,
        subtype: 0,
        timestamp: 1700000000,
        deadline: 32767,
        sender_id: 1234567890,
        recipient_id: Some(9876543210),
        amount: 1_000_000_000,
        fee: 100_000,
        height: 0,
        block_id: 0,
        signature: [1u8; 64],
        full_hash: [0u8; 32], // will be computed
        attachment_bytes: vec![],
        phased: false,
        has_message: false,
        has_encrypted_message: false,
        has_public_key_announcement: false,
        has_prunable_attachment: false,
        ec_block_height: None,
        ec_block_id: None,
        has_encrypttoself_message: false,
        has_prunable_encrypted_message: false,
    };

    let hash1 = tx_original.compute_hash().unwrap();

    // Serialize and deserialize
    let json = serde_json::to_string(&tx_original).unwrap();
    let tx_deserialized: Transaction = serde_json::from_str(&json).unwrap();

    let hash2 = tx_deserialized.compute_hash().unwrap();

    assert_eq!(hash1, hash2);
}

#[test]
fn test_account_balance_operations_underflow() {
    let mut account = Account::new(1234567890, 100);
    // Debit more than available should fail
    assert!(account.debit(200).is_err());
    // Balance should remain unchanged
    assert_eq!(account.balance, 100);
    assert_eq!(account.unconfirmed_balance, 100);
}

#[test]
fn test_asset_operations_edge_cases() {
    let mut account = Account::new(1234567890, 0);

    // Remove asset with zero quantity should be no-op
    let result = account.remove_asset(100, 0);
    assert!(result.is_err()); // should be error

    // After adding, removing exactly the amount should delete entry
    account.add_asset(100, 100);
    assert!(account.remove_asset(100, 100).is_ok());
    assert!(!account.assets.contains_key(&100));
}

#[test]
fn test_transaction_type_range() {
    // Test all defined transaction types
    let types = [
        TransactionType::Payment,
        TransactionType::AssetTransfer,
        TransactionType::AssetIssuance,
        TransactionType::ContractInvocation,
        TransactionType::ContractDeployment,
        TransactionType::Lease,
        TransactionType::SetProperty,
    ];

    for ty in types.iter() {
        let u8_val = u8::from(*ty);
        let recovered = TransactionType::from(u8_val);
        assert_eq!(*ty, recovered);
    }
}

#[test]
fn test_block_version_constant() {
    assert_eq!(BLOCK_VERSION, 1);
}

#[test]
fn test_transaction_version_constant() {
    assert_eq!(TRANSACTION_VERSION, 1);
}

#[test]
fn test_transaction_subtype_ignored_in_hash() {
    // Changing subtype should not affect hash? Actually it does because it's in serialize_for_signing
    let base_tx = Transaction {
        version: 1,
        type_id: TransactionType::Payment,
        subtype: 0,
        timestamp: 1700000000,
        deadline: 32767,
        sender_id: 1234567890,
        recipient_id: Some(9876543210),
        amount: 1_000_000_000,
        fee: 100_000,
        height: 0,
        block_id: 0,
        signature: [0u8; 64],
        full_hash: [0u8; 32],
        attachment_bytes: vec![],
        phased: false,
        has_message: false,
        has_encrypted_message: false,
        has_public_key_announcement: false,
        has_prunable_attachment: false,
        ec_block_height: None,
        ec_block_id: None,
        has_encrypttoself_message: false,
        has_prunable_encrypted_message: false,
    };

    let hash1 = base_tx.compute_hash().unwrap();

    let mut tx2 = base_tx.clone();
    tx2.subtype = 1;
    let hash2 = tx2.compute_hash().unwrap();

    // Subtype is included in hash, so they should differ
    assert_ne!(hash1, hash2);
}

#[test]
fn test_transaction_phased_flag() {
    let mut tx = Transaction::new(
        TransactionType::Payment,
        1234567890,
        Some(9876543210),
        1_000_000_000,
        100_000,
        1700000000,
        32767,
    );
    assert!(!tx.phased);

    tx.phased = true;
    assert!(tx.phased);
}

#[test]
fn test_merkle_root_three_transactions() {
    let mut hashes = vec![];

    // Create 3 transactions with distinct hashes
    for i in 1..=3 {
        let tx = Transaction {
            full_hash: [i as u8; 32],
            ..Default::default()
        };
        hashes.push(tx);
    }

    let root = Block::compute_merkle_root(&hashes).unwrap();
    assert_ne!(root, [0u8; 32]);

    // Determinism: same order should give same root
    let root2 = Block::compute_merkle_root(&hashes).unwrap();
    assert_eq!(root, root2);
}

#[test]
fn test_merkle_root_odd_count() {
    // Test with 3 transactions (odd count)
    let tx1 = Transaction { full_hash: [1u8; 32], ..Default::default() };
    let tx2 = Transaction { full_hash: [2u8; 32], ..Default::default() };
    let tx3 = Transaction { full_hash: [3u8; 32], ..Default::default() };

    let root = Block::compute_merkle_root(&[tx1, tx2, tx3]).unwrap();
    assert_ne!(root, [0u8; 32]);
}

#[test]
fn test_timestamp_validation_boundary() {
    // Test timestamp exactly 1 hour in future (should fail)
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let mut tx = Transaction::new(
        TransactionType::Payment,
        1234567890,
        Some(9876543210),
        1_000_000_000,
        100_000,
        now + 3601, // 1 hour + 1 second
        32767,
    );
    tx.full_hash = tx.compute_hash().unwrap();

    let result = tx.validate_basic();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("timestamp"));
}

#[test]
fn test_timestamp_validation_one_hour_ago_ok() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let mut tx = Transaction::new(
        TransactionType::Payment,
        1234567890,
        Some(9876543210),
        1_000_000_000,
        100_000,
        now - 3600, // exactly 1 hour ago (edge)
        32767,
    );
    tx.full_hash = tx.compute_hash().unwrap();

    // Should pass (timestamp not more than 1 hour in future)
    assert!(tx.validate_basic().is_ok());
}
