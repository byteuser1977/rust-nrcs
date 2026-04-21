//! 共识算法测试
//!
//! 测试：
//! - 共识引擎创建
//! - 出块者选择
//! - 难度调整

use consensus::prelude::*;
use blockchain_types::*;

/// 测试 PosEngine 创建和基本操作
#[test]
fn test_pos_engine_creation() {
    let engine = PosEngine::new(15, 1_000_000, 100_000);
    assert_eq!(engine.target_spacing, 15);
    assert_eq!(engine.minimum_balance, 1_000_000);
    assert_eq!(engine.block_reward, 100_000);
}

/// 测试 PoWEngine 创建
#[test]
fn test_pow_engine_creation() {
    let engine = PoWEngine::new(60);
    assert_eq!(engine.target_spacing, 60);
    assert_eq!(engine.initial_difficulty, 1u64 << 32);
}

/// 测试难度调整逻辑
#[test]
fn test_difficulty_adjustment() {
    use consensus::DifficultyAdjustmentParams;
    
    let params = DifficultyAdjustmentParams::default();
    let current = 1_000_000;

    // 出块过快 → 难度提高 → base_target 减小
    let faster = consensus::adjust_difficulty(current, &[12, 13, 12], &params);
    assert!(faster < current);

    // 出块过慢 → 难度降低 → base_target 增大
    let slower = consensus::adjust_difficulty(current, &[20, 22, 21], &params);
    assert!(slower > current);

    let normal = consensus::adjust_difficulty(current, &[14, 15, 16], &params);
    assert_eq!(normal, current);
}

/// 测试出块者选择（基于 PosEngine）
#[test]
fn test_forger_selection() {
    let engine = PosEngine::new(15, 1, 100);
    
    let accounts = vec![
        consensus::AccountSnapshot {
            id: 1,
            balance: 100,
            lease: None,
            has_public_key: true,
        },
        consensus::AccountSnapshot {
            id: 2,
            balance: 200,
            lease: None,
            has_public_key: true,
        },
        consensus::AccountSnapshot {
            id: 3,
            balance: 300,
            lease: None,
            has_public_key: true,
        },
    ];
    
    let state = consensus::BlockchainState::new(
        1,
        Hash256([0u8; 32]),
        1_000_000,
        vec![0u8; 32],
        Hash512([0u8; 64]),
        1000,
        accounts,
    );
    
    let result = engine.select_forger(&state, 1000 + 15);
    assert!(result.is_ok());
    let (forger_id, _) = result.unwrap();
    assert!(vec![1, 2, 3].contains(&forger_id));
}

/// 测试 BlockchainState 总有效余额计算
#[test]
fn test_blockchain_state_total_effective_balance() {
    let accounts = vec![
        consensus::AccountSnapshot {
            id: 1,
            balance: 100,
            lease: None,
            has_public_key: true,
        },
        consensus::AccountSnapshot {
            id: 2,
            balance: 200,
            lease: None,
            has_public_key: true,
        },
        consensus::AccountSnapshot {
            id: 3,
            balance: 300,
            lease: None,
            has_public_key: true,
        },
    ];
    
    let state = consensus::BlockchainState::new(
        1,
        Hash256([0u8; 32]),
        1_000_000,
        vec![0u8; 32],
        Hash512([0u8; 64]),
        1000,
        accounts,
    );
    
    assert_eq!(state.total_effective_balance(), 600);
}
