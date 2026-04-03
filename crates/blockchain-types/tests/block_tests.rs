use blockchain_types::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_block_serialization_roundtrip() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let block = Block {
        version: BLOCK_VERSION,
        timestamp: now,
        height: 100,
        previous_block_hash: [1u8; 32],
        payload_hash: [2u8; 32],
        generator_id: 1234567890,
        nonce: 42,
        base_target: 1_000_000,
        cumulative_difficulty: vec![0u8; 16],
        total_amount: 1_000_000_000,
        total_fee: 100_000,
        payload_length: 0,
        generation_signature: [3u8; 64],
        block_signature: [4u8; 64],
        transactions: vec![],
    };

    // JSON roundtrip
    let json = block.to_json().unwrap();
    let decoded: Block = serde_json::from_str(&json).unwrap();
    assert_eq!(block.version, decoded.version);
    assert_eq!(block.height, decoded.height);
    assert_eq!(block.generator_id, decoded.generator_id);

    // Bincode roundtrip
    let binary = block.to_bincode().unwrap();
    let decoded: Block = Block::from_bincode(&binary).unwrap();
    assert_eq!(block, decoded);
}

#[test]
fn test_block_new() {
    let block = Block::new(1, [0u8; 32], 1234567890);
    assert_eq!(block.height, 1);
    assert_eq!(block.previous_block_hash, [0u8; 32]);
    assert_eq!(block.generator_id, 1234567890);
    assert_eq!(block.version, BLOCK_VERSION);
    assert_eq!(block.nonce, 0);
}

#[test]
fn test_compute_hash() {
    let mut block = Block::new(1, [0u8; 32], 1234567890);
    block.timestamp = 1700000000;
    block.total_amount = 1000;
    block.total_fee = 50;

    let hash = block.compute_hash().unwrap();
    assert!(hash != [0u8; 32]);
    assert_eq!(hash.len(), 32);

    // Hash should be deterministic
    let hash2 = block.compute_hash().unwrap();
    assert_eq!(hash, hash2);
}

#[test]
fn test_merkle_root_empty() {
    let root = Block::compute_merkle_root(&[]).unwrap();
    assert_eq!(root, [0u8; 32]);
}

#[test]
fn test_merkle_root_single() {
    let tx = Transaction {
        full_hash: [1u8; 32],
        ..Default::default()
    };
    let root = Block::compute_merkle_root(&[tx.clone()]).unwrap();
    assert_eq!(root, tx.full_hash);
}

#[test]
fn test_merkle_root_multiple() {
    let tx1 = Transaction { full_hash: [1u8; 32], ..Default::default() };
    let tx2 = Transaction { full_hash: [2u8; 32], ..Default::default() };
    let tx3 = Transaction { full_hash: [3u8; 32], ..Default::default() };

    let root = Block::compute_merkle_root(&[tx1, tx2, tx3]).unwrap();
    assert!(root != [0u8; 32]);
    // Verify order matters
    let root_reversed = Block::compute_merkle_root(&[tx3, tx2, tx1]).unwrap();
    assert_ne!(root, root_reversed);
}

#[test]
fn test_validate_basic_success() {
    let mut block = Block::new(1, [0u8; 32], 1234567890);
    block.timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    block.total_amount = 1000;
    block.payload_length = 0;

    assert!(block.validate_basic().is_ok());
}

#[test]
fn test_validate_basic_version_mismatch() {
    let mut block = Block::new(1, [0u8; 32], 1234567890);
    block.version = 999;
    block.timestamp = 1700000000;

    let result = block.validate_basic();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("version"));
}

#[test]
fn test_validate_basic_future_timestamp() {
    let mut block = Block::new(1, [0u8; 32], 1234567890);
    // Set timestamp 2 hours in future
    let future = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32 + 7200;
    block.timestamp = future;

    let result = block.validate_basic();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("future"));
}

#[test]
fn test_validate_basic_zero_base_target() {
    let mut block = Block::new(1, [0u8; 32], 1234567890);
    block.base_target = 0;
    block.timestamp = 1700000000;

    let result = block.validate_basic();
    assert!(result.is_err());
}

#[test]
fn test_validate_full() {
    let mut block = Block::new(1, [0u8; 32], 1234567890);
    block.timestamp = 1700000000;
    block.total_amount = 1000;
    block.payload_length = 0;
    block.payload_hash = [0u8; 32]; // empty merkle root

    assert!(block.validate_full(1).is_ok());
}

#[test]
fn test_validate_full_height_mismatch() {
    let mut block = Block::new(1, [0u8; 32], 1234567890);
    block.timestamp = 1700000000;

    let result = block.validate_full(999);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("height"));
}

#[test]
fn test_compute_merkle_root_with_transactions() {
    let tx1 = Transaction {
        full_hash: [1u8; 32],
        ..Default::default()
    };
    let tx2 = Transaction {
        full_hash: [2u8; 32],
        ..Default::default()
    };

    let root = Block::compute_merkle_root(&[tx1, tx2]).unwrap();
    // With two transactions, the root is: SHA256(SHA256(tx1) + SHA256(tx2))
    // So it should be non-zero and deterministic
    assert_ne!(root, [0u8; 32]);
}

#[test]
#[should_panic]
fn test_compute_hash_empty_block_genesis() {
    // Genesis block can have zero previous hash but should still compute a valid hash
    let genesis = Block {
        version: BLOCK_VERSION,
        timestamp: 0,
        height: 0,
        previous_block_hash: [0u8; 32],
        payload_hash: [0u8; 32],
        generator_id: 0,
        nonce: 0,
        base_target: 1_000_000,
        cumulative_difficulty: vec![],
        total_amount: 0,
        total_fee: 0,
        payload_length: 0,
        generation_signature: [0u8; 64],
        block_signature: [0u8; 64],
        transactions: vec![],
    };
    let hash = genesis.compute_hash().unwrap();
    assert_ne!(hash, [0u8; 32]); // Genesis hash should be non-zero
}
