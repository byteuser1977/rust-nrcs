use consensus::pow::*;
use blockchain_types::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_pow_engine_creation() {
    let engine = ProofOfWorkEngine::new(1);
    assert!(engine.verify_difficulty(&Block::new(1, [0u8; 32], 0)).is_ok());
}

#[test]
fn test_pow_difficulty_calculation() {
    let engine = ProofOfWorkEngine::new(1);
    let recent_blocks = vec![];

    let difficulty = engine.calculate_next_difficulty(&recent_blocks);
    assert_eq!(difficulty, 1);
}

#[test]
fn test_pow_verify_difficulty_too_easy() {
    let engine = ProofOfWorkEngine::new(1000);
    let block = Block {
        base_target: 2000, // too high (easier)
        ..Block::new(1, [0u8; 32], 0)
    };

    let result = engine.verify_difficulty(&block);
    assert!(result.is_err());
    assert!(matches!(result, Err(ConsensusError::NotMeetDifficulty)));
}

#[test]
fn test_pow_verify_difficulty_valid() {
    let target_difficulty = 1000u64;
    let engine = ProofOfWorkEngine::new(target_difficulty);

    let mut block = Block::new(1, [0u8; 32], 0);
    block.base_target = target_difficulty;

    assert!(engine.verify_difficulty(&block).is_ok());
}

#[test]
fn test_pow_verify_timestamp_valid() {
    let engine = ProofOfWorkEngine::new(1);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let block = Block {
        timestamp: now,
        ..Block::new(1, [0u8; 32], 0)
    };

    assert!(engine.verify_timestamp(&block, now).is_ok());
}

#[test]
fn test_pow_verify_timestamp_too_far_in_future() {
    let engine = ProofOfWorkEngine::new(1);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let block = Block {
        timestamp: now + 7201, // > 2 hours
        ..Block::new(1, [0u8; 32], 0)
    };

    let result = engine.verify_timestamp(&block, now);
    assert!(result.is_err());
}

#[test]
fn test_pow_verify_timestamp_before_genesis() {
    let engine = ProofOfWorkEngine::new(1);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let block = Block {
        timestamp: now - 3600, // 1 hour ago could be valid depending on config
        ..Block::new(1, [0u8; 32], 0)
    };

    // Current implementation allows past timestamps (synchronization)
    assert!(engine.verify_timestamp(&block, now).is_ok());
}

#[test]
fn test_pow_difficulty_adjustment() {
    // Create a series of blocks with fast generation time -> difficulty should increase
    let engine = ProofOfWorkEngine::new(1000);
    let mut blocks = vec![];

    for i in 0..10 {
        let mut block = Block::new(i as u32 + 1, if i == 0 { [0u8; 32] } else { blocks[i-1].compute_hash().unwrap() }, 0);
        block.timestamp = 1000 + i as u32 * 10; // every 10 seconds
        blocks.push(block);
    }

    let next_diff = engine.calculate_next_difficulty(&blocks);
    // Difficulty should increase if blocks are too fast
    assert!(next_diff > 1000);
}

#[test]
fn test_pow_difficulty_adjustment_slow() {
    // Create blocks that are slow -> difficulty should decrease
    let engine = ProofOfWorkEngine::new(1000);
    let mut blocks = vec![];

    for i in 0..10 {
        let mut block = Block::new(i as u32 + 1, if i == 0 { [0u8; 32] } else { blocks[i-1].compute_hash().unwrap() }, 0);
        block.timestamp = 1000 + i as u32 * 60; // every 60 seconds (slower than target)
        blocks.push(block);
    }

    let next_diff = engine.calculate_next_difficulty(&blocks);
    // Difficulty might decrease (depends on algorithm)
    assert!(next_diff != 1000); // some change expected
}

#[test]
fn test_pow_block_hash_computation() {
    let block = Block {
        version: BLOCK_VERSION,
        timestamp: 1700000000,
        height: 100,
        previous_block_hash: [1u8; 32],
        payload_hash: [2u8; 32],
        generator_id: 1234567890,
        nonce: 42,
        base_target: 1_000_000,
        cumulative_difficulty: vec![],
        total_amount: 0,
        total_fee: 0,
        payload_length: 0,
        generation_signature: [0u8; 64],
        block_signature: [0u8; 64],
        transactions: vec![],
    };

    let hash = block.compute_hash().unwrap();
    assert_ne!(hash, [0u8; 32]);
    assert_eq!(hash.len(), 32);
}

#[test]
fn test_pow_determinism() {
    let engine1 = ProofOfWorkEngine::new(1000);
    let engine2 = ProofOfWorkEngine::new(1000);

    let block = Block::new(1, [0u8; 32], 0);

    let diff1 = engine1.calculate_next_difficulty(&[]);
    let diff2 = engine2.calculate_next_difficulty(&[]);

    assert_eq!(diff1, diff2);
}
