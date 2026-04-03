//! Blockchain Types 基础类型测试

use blockchain_types::*;

/// 测试地址生成
#[test]
fn test_address_generation() {
    let pubkey = [1u8; 32];
    let address = Address::from_pubkey(&pubkey);
    assert_eq!(address.0.len(), 32); // 32 bytes
}

/// 测试交易哈希
#[test]
fn test_transaction_hash() {
    let tx = Transaction::new_transfer(1, 2, 100, 1);
    let hash1 = tx.hash();
    let hash2 = tx.hash();
    
    assert_eq!(hash1, hash2);
    assert_eq!(hash1.0.len(), 32);
}

/// 测试不同交易的哈希唯一性
#[test]
fn test_transaction_hash_uniqueness() {
    let tx1 = Transaction::new_transfer(1, 2, 100, 1);
    let tx2 = Transaction::new_transfer(1, 2, 200, 1); // 金额不同
    
    assert_ne!(tx1.hash(), tx2.hash());
}

/// 测试区块哈希计算
#[test]
fn test_block_hash_calculation() {
    let mut block = Block::new(
        1,
        [0u8; 32], // previous hash
        0,
        vec![],
    );
    
    let hash1 = block.calculate_hash();
    block.timestamp += 1;
    let hash2 = block.calculate_hash();
    
    assert_ne!(hash1, hash2);
}

/// 测试序列化和反序列化
#[test]
fn test_serialization() {
    let original = Transaction::new_transfer(1, 2, 100, 1);
    
    let encoded = bincode::serialize(&original).unwrap();
    let decoded: Transaction = bincode::deserialize(&encoded).unwrap();
    
    assert_eq!(original.from, decoded.from);
    assert_eq!(original.to, decoded.to);
    assert_eq!(original.amount, decoded.amount);
}

/// 测试时间戳类型
#[test]
fn test_timestamp() {
    let ts = Timestamp::now();
    assert!(ts.0 > 0);
    
    let ts2 = Timestamp::from(1000);
    assert_eq!(ts2.0, 1000);
}

/// 测试哈希256类型
#[test]
fn test_hash256() {
    let bytes = [1u8; 32];
    let hash = Hash256::new(bytes);
    assert_eq!(hash.0, bytes);
    
    let zero = Hash256::zero();
    assert_eq!(zero.0, [0u8; 32]);
}