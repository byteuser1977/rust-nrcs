//! 出块者选择算法测试
//!
//! 参照 Java NRCS 的 Generator.getNextGenerators() 逻辑。

use consensus::selection::*;
use blockchain_types::prelude::*;
use blockchain_types::constants::INITIAL_BASE_TARGET;

fn make_test_block_with_gen_sig(height: u32, timestamp: u32, base_target: u64) -> Block {
    Block {
        version: 3,
        timestamp,
        height,
        base_target,
        cumulative_difficulty: vec![],
        generation_signature: vec![1u8; 32],
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
fn test_forger_selector_creation() {
    let selector = ForgerSelector::new();
    let block = make_test_block_with_gen_sig(100, 1000, INITIAL_BASE_TARGET);
    let candidates = vec![ForgerCandidate::new(1, vec![1u8; 32], 1_000_000_000_000)];
    let result = selector.select_forger(&block, &candidates);
    assert!(result.is_some(), "ForgerSelector should work with default params");
}

#[test]
fn test_forger_selector_custom_params() {
    let selector = ForgerSelector::with_params(1000, 50);
    let block = make_test_block_with_gen_sig(100, 1000, INITIAL_BASE_TARGET);
    let candidates = vec![ForgerCandidate::new(1, vec![1u8; 32], 100)];
    let result = selector.select_forger(&block, &candidates);
    assert!(result.is_none(), "Balance below min should be rejected");
}

#[test]
fn test_select_forger_no_candidates() {
    let selector = ForgerSelector::new();
    let block = make_test_block_with_gen_sig(100, 1000, INITIAL_BASE_TARGET);
    let result = selector.select_forger(&block, &[]);
    assert!(result.is_none(), "No candidates should return None");
}

#[test]
fn test_select_forger_insufficient_balance() {
    let selector = ForgerSelector::with_params(1000, 10);
    let block = make_test_block_with_gen_sig(100, 1000, INITIAL_BASE_TARGET);
    let candidates = vec![
        ForgerCandidate::new(1, vec![1u8; 32], 100),
    ];
    let result = selector.select_forger(&block, &candidates);
    assert!(result.is_none(), "Candidate with insufficient balance should not be selected");
}

#[test]
fn test_select_forger_single_candidate() {
    let selector = ForgerSelector::with_params(100, 10);
    let block = make_test_block_with_gen_sig(100, 1000, INITIAL_BASE_TARGET);
    let candidates = vec![
        ForgerCandidate::new(1, vec![1u8; 32], 1_000_000),
    ];
    let result = selector.select_forger(&block, &candidates);
    assert!(result.is_some());
    let selection = result.unwrap();
    assert_eq!(selection.forger_id, 1);
    assert!(selection.deadline > 0);
}

#[test]
fn test_select_forger_multiple_candidates() {
    let selector = ForgerSelector::with_params(100, 10);
    let block = make_test_block_with_gen_sig(100, 1000, INITIAL_BASE_TARGET);
    let candidates = vec![
        ForgerCandidate::new(1, vec![1u8; 32], 1_000_000),
        ForgerCandidate::new(2, vec![2u8; 32], 1_000_000),
        ForgerCandidate::new(3, vec![3u8; 32], 1_000_000),
    ];
    let result = selector.select_forger(&block, &candidates);
    assert!(result.is_some());
    let selection = result.unwrap();
    assert!([1u64, 2, 3].contains(&selection.forger_id));
}

#[test]
fn test_get_next_forgers() {
    let selector = ForgerSelector::with_params(100, 10);
    let block = make_test_block_with_gen_sig(100, 1000, INITIAL_BASE_TARGET);
    let candidates = vec![
        ForgerCandidate::new(1, vec![1u8; 32], 1_000_000),
        ForgerCandidate::new(2, vec![2u8; 32], 1_000_000),
        ForgerCandidate::new(3, vec![3u8; 32], 1_000_000),
    ];
    let forgers = selector.get_next_forgers(&block, &candidates, 2);
    assert!(forgers.len() <= 2);
    if forgers.len() >= 2 {
        assert!(forgers[0].hit_time <= forgers[1].hit_time, "Forgers should be sorted by hit_time");
    }
}

#[test]
fn test_calculate_generation_deadline() {
    let selector = ForgerSelector::with_params(100, 10);
    let block = make_test_block_with_gen_sig(100, 1000, INITIAL_BASE_TARGET);
    let public_key = vec![1u8; 32];
    let deadline = selector.calculate_generation_deadline(&block, &public_key, 1_000_000);
    assert!(deadline > 0);
    assert!(deadline < u64::MAX);
}

#[test]
fn test_calculate_generation_deadline_insufficient_balance() {
    let selector = ForgerSelector::with_params(1000, 10);
    let block = make_test_block_with_gen_sig(100, 1000, INITIAL_BASE_TARGET);
    let public_key = vec![1u8; 32];
    let deadline = selector.calculate_generation_deadline(&block, &public_key, 100);
    assert_eq!(deadline, u64::MAX);
}

#[test]
fn test_active_generator_list_creation() {
    let mut list = ActiveGeneratorList::new();
    assert!(list.is_empty());
    list.initialize(&[1, 2, 3]);
    assert_eq!(list.len(), 3);
}

#[test]
fn test_active_generator_list_add_duplicate() {
    let mut list = ActiveGeneratorList::new();
    list.initialize(&[1, 2, 3]);
    list.add_generator(1);
    assert_eq!(list.len(), 3, "Duplicate should not be added");
}

#[test]
fn test_active_generator_list_add_new() {
    let mut list = ActiveGeneratorList::new();
    list.initialize(&[1, 2, 3]);
    list.add_generator(4);
    assert_eq!(list.len(), 4);
}

#[test]
fn test_active_generator_list_update_for_block() {
    let mut list = ActiveGeneratorList::new();
    list.initialize(&[1, 2]);
    let block = make_test_block_with_gen_sig(100, 1000, INITIAL_BASE_TARGET);
    list.update_for_block(&block, |id| {
        (1_000_000, Some(vec![id as u8; 32]))
    });
    let generators = list.get_sorted_generators();
    assert!(!generators.is_empty());
}

#[test]
fn test_forger_candidate_creation() {
    let candidate = ForgerCandidate::new(123, vec![1u8; 32], 1000);
    assert_eq!(candidate.account_id, 123);
    assert_eq!(candidate.effective_balance, 1000);
    assert_eq!(candidate.public_key, vec![1u8; 32]);
}

#[test]
fn test_forger_info_sorting() {
    let mut forgers = vec![
        ForgerInfo::new(1, 300, 1000, 0),
        ForgerInfo::new(2, 100, 1000, 0),
        ForgerInfo::new(3, 200, 1000, 0),
    ];
    forgers.sort_by_key(|f| f.hit_time);
    assert_eq!(forgers[0].account_id, 2);
    assert_eq!(forgers[1].account_id, 3);
    assert_eq!(forgers[2].account_id, 1);
}
