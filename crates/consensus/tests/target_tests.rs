//! Target 计算和难度调整测试
//!
//! 参照 Java NRCS 的 Block.java 中的 baseTarget 和 cumulativeDifficulty 计算。

use consensus::target::*;
use blockchain_types::prelude::*;
use blockchain_types::constants::{INITIAL_BASE_TARGET, MAX_BASE_TARGET, MIN_BASE_TARGET, BLOCK_TIME};
use num_bigint::BigUint;
use num_traits::One;

fn make_test_block(height: u32, timestamp: u32, base_target: u64) -> Block {
    Block {
        version: 3,
        timestamp,
        height,
        base_target,
        cumulative_difficulty: vec![],
        generation_signature: vec![0u8; 32],
        generator_id: Some(1),
        generator_public_key: None,
        nonce: 0,
        previous_block_id: None,
        previous_block_hash: Hash256([0u8; 32]),
        transactions: vec![],
        block_signature: Hash512([0u8; 64]),
        payload_hash: Hash256([0u8; 32]),
        payload_length: 0,
        total_amount: 0,
        total_fee: 0,
        id: None,
    }
}

#[test]
fn test_target_calculator_default() {
    let calc = TargetCalculator::new();
    assert_eq!(calc.get_initial_base_target(), INITIAL_BASE_TARGET);
    assert_eq!(calc.get_max_base_target(), MAX_BASE_TARGET);
    assert_eq!(calc.get_min_base_target(), MIN_BASE_TARGET);
}

#[test]
fn test_target_calculator_custom_params() {
    let calc = TargetCalculator::with_params(100, 10000, 10, 64);
    assert_eq!(calc.get_initial_base_target(), 100);
    assert_eq!(calc.get_max_base_target(), 10000);
    assert_eq!(calc.get_min_base_target(), 10);
}

#[test]
fn test_base_target_no_change_when_normal_block_time() {
    let calc = TargetCalculator::new();
    let prev_block = make_test_block(100, 1000, INITIAL_BASE_TARGET);
    let prev_prev_block = make_test_block(99, 1000 - BLOCK_TIME, INITIAL_BASE_TARGET);
    let new_target = calc.calculate_base_target(&prev_block, Some(&prev_prev_block));
    assert_eq!(new_target, INITIAL_BASE_TARGET);
}

#[test]
fn test_base_target_increases_when_slow_blocks() {
    let calc = TargetCalculator::new();
    let prev_block = make_test_block(100, 2000, INITIAL_BASE_TARGET);
    let prev_prev_block = make_test_block(99, 1000, INITIAL_BASE_TARGET);
    let new_target = calc.calculate_base_target(&prev_block, Some(&prev_prev_block));
    assert!(new_target > INITIAL_BASE_TARGET, "Slow blocks should increase base target");
}

#[test]
fn test_base_target_decreases_when_fast_blocks() {
    let calc = TargetCalculator::new();
    let prev_block = make_test_block(100, 1010, INITIAL_BASE_TARGET);
    let prev_prev_block = make_test_block(99, 1000, INITIAL_BASE_TARGET);
    let new_target = calc.calculate_base_target(&prev_block, Some(&prev_prev_block));
    assert!(new_target < INITIAL_BASE_TARGET, "Fast blocks should decrease base target");
}

#[test]
fn test_base_target_clamped_to_max() {
    let calc = TargetCalculator::with_params(100, 1000, 10, 64);
    let prev_block = make_test_block(100, 100000, 900);
    let prev_prev_block = make_test_block(99, 0, 900);
    let new_target = calc.calculate_base_target(&prev_block, Some(&prev_prev_block));
    assert!(new_target <= 1000, "Base target should not exceed max");
}

#[test]
fn test_base_target_clamped_to_min() {
    let calc = TargetCalculator::with_params(100, 10000, 10, 64);
    let prev_block = make_test_block(100, 1, 100);
    let prev_prev_block = make_test_block(99, 0, 100);
    let new_target = calc.calculate_base_target(&prev_block, Some(&prev_prev_block));
    assert!(new_target >= 10, "Base target should not go below min");
}

#[test]
fn test_base_target_no_prev_prev_block() {
    let calc = TargetCalculator::new();
    let prev_block = make_test_block(0, 0, INITIAL_BASE_TARGET);
    let new_target = calc.calculate_base_target(&prev_block, None);
    assert_eq!(new_target, INITIAL_BASE_TARGET);
}

#[test]
fn test_cumulative_difficulty_increases() {
    let calc = TargetCalculator::new();
    let prev_diff = BigUint::from(1000u64);
    let prev_bytes = biguint_to_bytes(&prev_diff);
    let new_diff = calc.calculate_cumulative_difficulty(&prev_bytes, 100_000);
    assert!(new_diff > prev_diff, "Cumulative difficulty should increase");
}

#[test]
fn test_cumulative_difficulty_empty_prev() {
    let calc = TargetCalculator::new();
    let new_diff = calc.calculate_cumulative_difficulty(&[], 100_000);
    assert!(new_diff > BigUint::one(), "Should add difficulty even with empty prev");
}

#[test]
fn test_cumulative_difficulty_zero_base_target() {
    let calc = TargetCalculator::new();
    let prev_diff = BigUint::from(1000u64);
    let prev_bytes = biguint_to_bytes(&prev_diff);
    let new_diff = calc.calculate_cumulative_difficulty(&prev_bytes, 0);
    assert_eq!(new_diff, prev_diff, "Zero base target should not change difficulty");
}

#[test]
fn test_calculate_target_info() {
    let calc = TargetCalculator::new();
    let prev_block = make_test_block(100, 2000, INITIAL_BASE_TARGET);
    let prev_prev_block = make_test_block(99, 1000, INITIAL_BASE_TARGET);
    let info = calc.calculate_target_info(&prev_block, Some(&prev_prev_block));
    assert!(info.base_target >= calc.get_min_base_target());
    assert!(info.base_target <= calc.get_max_base_target());
}

#[test]
fn test_calculate_hit() {
    let pk = vec![1u8; 32];
    let gen_sig = vec![2u8; 32];
    let hit = calculate_hit(&pk, &gen_sig);
    assert!(hit > BigUint::from(0u64), "Hit should be positive");
}

#[test]
fn test_calculate_deadline_normal() {
    let hit = BigUint::from(1_000_000_000_000u64);
    let effective_balance = 1_000u64;
    let base_target = 1_000u64;
    let deadline = calculate_deadline(&hit, effective_balance, base_target);
    assert!(deadline > 0);
    assert!(deadline < u64::MAX);
}

#[test]
fn test_calculate_deadline_zero_balance() {
    let hit = BigUint::from(1_000_000_000_000u64);
    let deadline = calculate_deadline(&hit, 0, 1000);
    assert_eq!(deadline, u64::MAX, "Zero balance should return max deadline");
}

#[test]
fn test_calculate_deadline_zero_base_target() {
    let hit = BigUint::from(1_000_000_000_000u64);
    let deadline = calculate_deadline(&hit, 1000, 0);
    assert_eq!(deadline, u64::MAX, "Zero base target should return max deadline");
}

#[test]
fn test_verify_hit_valid() {
    let hit = BigUint::from(5_000_000u64);
    let effective_balance = 100u64;
    let base_target = 100u64;
    assert!(verify_hit(&hit, effective_balance, base_target, 1000));
}

#[test]
fn test_verify_hit_zero_elapsed() {
    let hit = BigUint::from(5_000_000u64);
    assert!(!verify_hit(&hit, 100, 100, 0), "Zero elapsed time should fail");
}

#[test]
fn test_difficulty_ratio() {
    let calc = TargetCalculator::new();
    let ratio = calc.calculate_difficulty_ratio(INITIAL_BASE_TARGET);
    assert!(ratio > BigUint::from(0u64));
}

#[test]
fn test_verify_base_target() {
    let calc = TargetCalculator::new();
    assert!(calc.verify_base_target(INITIAL_BASE_TARGET));
    assert!(calc.verify_base_target(MAX_BASE_TARGET));
    assert!(calc.verify_base_target(MIN_BASE_TARGET));
    assert!(!calc.verify_base_target(0));
    assert!(!calc.verify_base_target(u64::MAX));
}

#[test]
fn test_average_block_time() {
    let blocks = vec![
        make_test_block(0, 0, INITIAL_BASE_TARGET),
        make_test_block(1, 60, INITIAL_BASE_TARGET),
        make_test_block(2, 120, INITIAL_BASE_TARGET),
    ];
    let avg = TargetCalculator::calculate_average_block_time(&blocks);
    assert_eq!(avg, 60);
}

#[test]
fn test_average_block_time_single_block() {
    let blocks = vec![make_test_block(0, 0, INITIAL_BASE_TARGET)];
    let avg = TargetCalculator::calculate_average_block_time(&blocks);
    assert_eq!(avg, BLOCK_TIME as u64);
}

#[test]
fn test_biguint_bytes_roundtrip() {
    let value = BigUint::from(123456789u64);
    let bytes = biguint_to_bytes(&value);
    let recovered = bytes_to_biguint(&bytes);
    assert_eq!(value, recovered);
}
