//! 共识算法测试（基于PoS或类似机制）
//!
//! 测试：
//! - 出块者选择
//! - 区块验证
//! - 分叉处理
//! - 拜占庭容错

use consensus::*;
use blockchain_types::*;

/// 测试出块者选择（基于质押权重的随机选择）
#[test]
fn test_block_producer_selection() {
    let mut consensus = PosConsensus::new(12345); // 固定随机种子
    
    let stakes = vec![
        (1u64, 100u64), // (validator_id, stake)
        (2u64, 200u64),
        (3u64, 300u64),
    ];
    
    let selected = consensus.select_block_producer(&stakes, Height(1));
    
    assert!(selected.is_some());
    let (validator_id, _) = selected.unwrap();
    assert!(vec![1, 2, 3].contains(&validator_id));
}

/// 测试区块签名验证
#[test]
fn test_block_signature_verification() {
    let block = Block::new(
        1,
        [1u8; 32],
        1000,
        vec![],
    );
    
    let private_key = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
    let signature = private_key.sign(&block.hash_bytes());
    
    let public_key = private_key.verifying_key();
    
    assert!(block.verify_signature(&public_key, &signature).is_ok());
}

/// 测试无效签名
#[test]
fn test_invalid_signature() {
    let block = Block::new(1, [1u8; 32], 1000, vec![]);
    
    let dummy_pubkey = ed25519_dalek::VerifyingKey::from_bytes(&[0u8; 32]).unwrap();
    let dummy_signature = ed25519_dalek::Signature::from_bytes(&[0u8; 64]).unwrap();
    
    assert!(block.verify_signature(&dummy_pubkey, &dummy_signature).is_err());
}

/// 测试区块哈希唯一性（不同内容产生不同哈希）
#[test]
fn test_block_hash_uniqueness() {
    let block1 = Block::new(1, [0u8; 32], 1000, vec![]);
    let block2 = Block::new(1, [0u8; 32], 1001, vec![]); // 时间戳不同
    
    assert_ne!(block1.hash(), block2.hash());
}

/// 测试 PoS 出块概率与质押量成正比
#[test]
fn test_stake_weighted_probability() {
    let mut consensus = PosConsensus::new(12345);
    
    let stakes = vec![
        (1u64, 100u64),
        (2u64, 200u64), // 2倍质押
        (3u64, 300u64), // 3倍质押
    ];
    
    let mut counts = std::collections::HashMap::new();
    for _ in 0..1000 {
        if let Some((validator, _)) = consensus.select_block_producer(&stakes, Height(1)) {
            *counts.entry(validator).or_insert(0) += 1;
        }
    }
    
    // 验证者2和3应该被选中更多次（大致成比例）
    let v1 = counts.get(&1u64).unwrap_or(&0);
    let v2 = counts.get(&2u64).unwrap_or(&0);
    let v3 = counts.get(&3u64).unwrap_or(&0);
    
    // v2 应该比 v1 多，v3 应该比 v2 多
    assert!(*v2 > *v1);
    assert!(*v3 > *v2);
}

/// 测试最终确定性（finality）
#[tokio::test]
async fn test_finality_after_checkpoints() {
    let mut chain = Vec::new();
    let mut consensus = PosConsensus::new(12345);
    
    // 模拟生成一系列区块
    for height in 1..=10 {
        let prev_hash = if height == 1 { [0u8; 32] } else { chain.last().unwrap().hash().0 };
        let block = Block::new(height, prev_hash, 1000, vec![]);
        chain.push(block);
    }
    
    // 检查点机制：每 N 个区块确认
    let finalized_height = consensus.get_finalized_height(&chain, CheckpointInterval(5));
    assert_eq!(finalized_height.0, 5); // 前5个区块已最终确定
}

/// 测试分叉选择规则（最长的链获胜）
#[test]
fn test_fork_choice_longest_chain() {
    let mut consensus = Consensus::new();
    
    // 链A：高度1-3
    let mut chain_a = vec![];
    for i in 1..=3 {
        let prev = if i == 1 { [0u8; 32] } else { chain_a.last().unwrap().hash().0 };
        chain_a.push(Block::new(i, prev, 1000, vec![]));
    }
    
    // 链B：高度1-2（较短）
    let mut chain_b = vec![];
    for i in 1..=2 {
        let prev = if i == 1 { [0u8; 32] } else { chain_b.last().unwrap().hash().0 };
        chain_b.push(Block::new(i, prev, 1000, vec![]));
    }
    
    let chosen = consensus.select_best_chain(chain_a.clone(), chain_b.clone());
    assert_eq!(chosen.len(), 3); // 选择长链
}

/// 测试无效区块拒绝
#[test]
fn test_invalid_block_rejection() {
    let mut consensus = Consensus::new();
    
    let mut block = Block::new(1, [0u8; 32], 1000, vec![]);
    block.timestamp = 0; // 无效的时间戳（早于创世区块）
    
    assert!(consensus.validate_block(&block).is_err());
}