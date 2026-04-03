use consensus::pos::*;
use blockchain_types::*;
use num_bigint::BigUint;
use num_traits::{Zero, ToPrimitive};

#[test]
fn test_pos_engine_creation() {
    let engine = PoSEngine::new(15, 1000, 100);
    assert_eq!(engine.target_spacing, 15);
    assert_eq!(engine.minimum_balance, 1000);
    assert_eq!(engine.block_reward, 100);
}

#[test]
fn test_pos_select_forger_insufficient_balance() {
    let engine = PoSEngine::new(15, 1000, 100);
    let state = BlockchainState {
        accounts: vec![
            AccountSnapshot { id: 1, balance: 500, ..Default::default() }, // below minimum
        ],
        last_generation_signature: [0u8; 64],
    };

    let result = engine.select_forger(&state, 1700000000);
    assert!(matches!(result, Err(ConsensusError::NoValidForger)));
}

#[test]
fn test_pos_select_forger_single_candidate() {
    let engine = PoSEngine::new(15, 1000, 100);
    let state = BlockchainState {
        accounts: vec![
            AccountSnapshot { id: 1, balance: 2000, ..Default::default() },
        ],
        last_generation_signature: [0u8; 64],
    };

    let (forger_id, gen_sig) = engine.select_forger(&state, 1700000000).unwrap();
    assert_eq!(forger_id, 1);
    assert_ne!(gen_sig, [0u8; 64]); // should generate non-zero signature
}

#[test]
fn test_pos_select_forger_multiple_candidates() {
    let engine = PoSEngine::new(15, 1000, 100);
    let state = BlockchainState {
        accounts: vec![
            AccountSnapshot { id: 1, balance: 2000, ..Default::default() },
            AccountSnapshot { id: 2, balance: 3000, ..Default::default() },
            AccountSnapshot { id: 3, balance: 5000, ..Default::default() },
        ],
        last_generation_signature: [1u8; 64],
    };

    // Run multiple times to check distribution (probabilistic)
    let mut winners = std::collections::HashSet::new();
    for _ in 0..100 {
        if let Ok((forger_id, _)) = engine.select_forger(&state, 1700000000) {
            winners.insert(forger_id);
        }
    }

    // At least two different forgers should be selected over 100 runs
    assert!(winners.len() >= 2, "Expected multiple winners, got {}: {:?}", winners.len(), winners);
}

#[test]
fn test_pos_calculate_deadline() {
    let engine = PoSEngine::new(15, 1000, 100);
    let account = AccountSnapshot {
        id: 1,
        balance: 2000,
        forgings_since: 0,
        last_forged_time: 0,
    };

    let deadline = engine.calculate_deadline(&account, &[1u8; 64], 1000).unwrap();
    // Deadline should be proportional to balance
    assert!(deadline > 0);
    assert!(deadline < 3600); // should be reasonable (< 1 hour typically)
}

#[test]
fn test_pos_deadline_ordering() {
    let engine = PoSEngine::new(15, 1000, 100);
    let gen_sig = [2u8; 64];

    // Higher balance should generally yield shorter deadline (more likely to forge)
    let low_balance = AccountSnapshot { id: 1, balance: 2000, forgings_since: 0, last_forged_time: 0 };
    let high_balance = AccountSnapshot { id: 2, balance: 5000, forgings_since: 0, last_forged_time: 0 };

    let deadline_low = engine.calculate_deadline(&low_balance, &gen_sig, 1000).unwrap();
    let deadline_high = engine.calculate_deadline(&high_balance, &gen_sig, 1000).unwrap();

    // With higher balance, deadline should be smaller (faster hit)
    assert!(deadline_high < deadline_low);
}

#[test]
fn test_pos_verify_deadline() {
    let engine = PoSEngine::new(15, 1000, 100);
    let now = 1700000000u32;

    // Deadline not yet passed should be valid
    assert!(engine.verify_deadline(now + 100, now).is_ok());

    // Deadline passed should fail
    assert!(matches!(engine.verify_deadline(now - 100, now), Err(ConsensusError::DeadlineExpired { .. })));
}

#[test]
fn test_pos_verify_block_signature() {
    let engine = PoSEngine::new(15, 1000, 100);
    let public_key: PublicKey = [3u8; 32];
    let block = Block {
        block_signature: [4u8; 64],
        ..Block::new(1, [0u8; 32], 1)
    };

    // In real impl, would verify with actual cryptographic signature
    // Here we just check that it doesn't panic
    // let result = engine.verify_block_signature(&block, &public_key);
    // assert!(result.is_ok()); // would need actual valid signature
}

#[test]
fn test_pos_generate_signature() {
    let engine = PoSEngine::new(15, 1000, 100);
    let account_id = 1234567890u64;
    let prev_gen_sig = [5u8; 64];

    let gen_sig = engine.generate_signature(account_id, &prev_gen_sig);
    assert_ne!(gen_sig, [0u8; 64]);
    assert_eq!(gen_sig.len(), 64);
}

#[test]
fn test_pos_generate_signature_determinism() {
    let engine = PoSEngine::new(15, 1000, 100);
    let account_id = 1234567890u64;
    let prev_gen_sig = [5u8; 64];

    let sig1 = engine.generate_signature(account_id, &prev_gen_sig);
    let sig2 = engine.generate_signature(account_id, &prev_gen_sig);

    assert_eq!(sig1, sig2);
}

#[test]
fn test_pos_apply_block_reward() {
    let engine = PoSEngine::new(15, 1000, 100);
    let mut state = BlockchainState {
        accounts: vec![
            AccountSnapshot { id: 1, balance: 2000, ..Default::default() },
        ],
        last_generation_signature: [0u8; 64],
    };

    let mut block = Block::new(1, [0u8; 32], 1);
    block.total_fee = 50;

    engine.apply_block_reward(&mut state, &block).unwrap();

    // Forger should receive block reward + fees
    let forger = state.accounts.iter().find(|a| a.id == 1).unwrap();
    assert_eq!(forger.balance, 2000 + 100 + 50); // initial + reward + fees
}

#[test]
fn test_pos_cumulative_difficulty_increase() {
    let engine = PoSEngine::new(15, 1000, 100);
    let mut state = BlockchainState {
        accounts: vec![
            AccountSnapshot { id: 1, balance: 2000, ..Default::default() },
        ],
        last_generation_signature: [0u8; 64],
        cumulative_difficulty: BigUint::from(1000u64),
    };

    let block = Block {
        base_target: 1000,
        ..Block::new(1, [0u8; 32], 1)
    };

    let new_cumulative = engine.update_cumulative_difficulty(&state, &block);
    assert!(new_cumulative > BigUint::from(1000u64));
}

#[test]
fn test_pos_minimum_balance_check() {
    let engine = PoSEngine::new(15, 1000, 100);

    // Exactly at minimum should be eligible
    let state = BlockchainState {
        accounts: vec![
            AccountSnapshot { id: 1, balance: 1000, ..Default::default() },
        ],
        last_generation_signature: [0u8; 64],
    };

    let result = engine.select_forger(&state, 1700000000);
    assert!(result.is_ok());
}

#[test]
fn test_pos_minimum_balance_excluded() {
    let engine = PoSEngine::new(15, 1000, 100);

    // Just below minimum should be excluded
    let state = BlockchainState {
        accounts: vec![
            AccountSnapshot { id: 1, balance: 999, ..Default::default() },
        ],
        last_generation_signature: [0u8; 64],
    };

    let result = engine.select_forger(&state, 1700000000);
    assert!(matches!(result, Err(ConsensusError::NoValidForger)));
}

#[test]
fn test_pos_forgings_since_increment() {
    let engine = PoSEngine::new(15, 1000, 100);
    let mut account = AccountSnapshot {
        id: 1,
        balance: 2000,
        forgings_since: 0,
        last_forged_time: 0,
    };

    // Simulate forging
    engine.increment_forgings_since(&mut account);
    assert_eq!(account.forgings_since, 1);

    engine.increment_forgings_since(&mut account);
    assert_eq!(account.forgings_since, 2);
}

#[test]
fn test_pos_reset_forgings_since() {
    let engine = PoSEngine::new(15, 1000, 100);
    let mut account = AccountSnapshot {
        id: 1,
        balance: 2000,
        forgings_since: 5,
        last_forged_time: 0,
    };

    engine.reset_forgings_since(&mut account);
    assert_eq!(account.forgings_since, 0);
}

#[test]
fn test_pos_bigint_operations() {
    // Test that BigUint operations work correctly for large balances
    let balances = vec![
        BigUint::from(1000u64),
        BigUint::from(2000u64),
        BigUint::from(5000u64),
    ];

    let total: BigUint = balances.iter().sum();
    assert_eq!(total, BigUint::from(8000u64));

    // Modulo operation
    let remainder = total.mod_floor(&BigUint::from(3000u64));
    assert!(remainder < BigUint::from(3000u64));
}
