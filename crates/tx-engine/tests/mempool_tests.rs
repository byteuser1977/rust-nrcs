//! 交易引擎（tx-engine）测试模块
//!
//! 测试核心功能：
//! - 交易池管理（插入、移除、查询）
//! - 交易验证（签名、余额、格式）
//! - 交易排序（Gas 价格优先）
//! - 双重支付检测
//! - 性能基准测试准备

use crate::*;
use blockchain_types::*;
use std::sync::Arc;

/// 测试交易池创建
#[test]
fn test_mempool_creation() {
    let mempool = MemPool::new(1000); // 容量 1000
    assert_eq!(mempool.size(), 0);
}

/// 测试交易添加
#[tokio::test]
async fn test_mempool_add_transaction() {
    let mempool = MemPool::new(100);
    
    let tx = Transaction::new_transfer(1, 2, 100, 1);
    let signed_tx = SignedTransaction::new(tx, &[1u8; 64]);
    
    assert!(mempool.add_transaction(signed_tx.clone()).await.is_ok());
    assert_eq!(mempool.size(), 1);
}

/// 测试交易排序（按 Gas 价格）
#[tokio::test]
async fn test_mempool_ordering() {
    let mempool = MemPool::new(100);
    
    let tx1 = Transaction::new_transfer(1, 2, 100, 1);
    let tx2 = Transaction::new_transfer(3, 4, 200, 2); // 更高的 Gas 价格
    let tx3 = Transaction::new_transfer(5, 6, 150, 1);
    
    let signed1 = SignedTransaction::new(tx1, &[1u8; 64]);
    let signed2 = SignedTransaction::new(tx2, &[2u8; 64]);
    let signed3 = SignedTransaction::new(tx3, &[3u8; 64]);
    
    mempool.add_transaction(signed1.clone()).await.unwrap();
    mempool.add_transaction(signed2.clone()).await.unwrap();
    mempool.add_transaction(signed3.clone()).await.unwrap();
    
    // 取出的优先级应排序：tx2 (gas=2) > tx3 (gas=1) > tx1 (gas=1 but lower nonce)
    let txs = mempool.get_transactions_for_block(2).await;
    assert_eq!(txs.len(), 2);
    assert_eq!(txs[0].tx.gas_price, 2);
    assert_eq!(txs[1].tx.gas_price, 1);
}

/// 测试重复交易检测
#[tokio::test]
async fn test_mempool_double_spend_detection() {
    let mempool = MemPool::new(100);
    
    let tx1 = Transaction::new_transfer(1, 2, 100, 1);
    let tx2 = Transaction::new_transfer(1, 2, 100, 1); // 重复的 nonce
    
    let signed1 = SignedTransaction::new(tx1, &[1u8; 64]);
    let signed2 = SignedTransaction::new(tx2, &[1u8; 64]);
    
    mempool.add_transaction(signed1.clone()).await.unwrap();
    // 重复交易应该被拒绝
    assert!(mempool.add_transaction(signed2).await.is_err());
}

/// 测试交易移除
#[tokio::test]
async fn test_mempool_remove_transaction() {
    let mempool = MemPool::new(100);
    
    let tx = Transaction::new_transfer(1, 2, 100, 1);
    let signed_tx = SignedTransaction::new(tx, &[1u8; 64]);
    let tx_hash = signed_tx.hash();
    
    mempool.add_transaction(signed_tx.clone()).await.unwrap();
    assert!(mempool.size() == 1);
    
    mempool.remove_transaction(&tx_hash).await;
    assert_eq!(mempool.size(), 0);
}

/// 测试交易验证 - 有效签名
#[tokio::test]
async fn test_transaction_validation_valid_signature() {
    let processor = TransactionProcessor::new();
    
    let from_pubkey = [1u8; 32];
    let tx = Transaction::new_transfer(
        AccountId::from_pubkey(&from_pubkey),
        2,
        100,
        1
    );
    
    let signature = ed25519_dalek::Signer::sign(&from_pubkey.into(), &tx.hash_bytes());
    let signed_tx = SignedTransaction::new(tx, &signature.to_bytes());
    
    assert!(processor.validate_transaction(&signed_tx).await.is_ok());
}

/// 测试交易验证 - 无效签名
#[tokio::test]
async fn test_transaction_validation_invalid_signature() {
    let processor = TransactionProcessor::new();
    
    let tx = Transaction::new_transfer(1, 2, 100, 1);
    let invalid_signature = [0u8; 64];
    let signed_tx = SignedTransaction::new(tx, &invalid_signature);
    
    assert!(processor.validate_transaction(&signed_tx).await.is_err());
}

/// 测试交易验证 - 重复 nonce
#[tokio::test]
async fn test_transaction_validation_double_spend() {
    let processor = TransactionProcessor::new();
    
    let account_id = 1u64;
    let tx1 = Transaction::new_transfer(account_id, 2, 100, 1);
    let tx2 = Transaction::new_transfer(account_id, 3, 200, 1); // 相同 nonce
    
    // 模拟第一条交易已被处理
    processor.record_processed_transaction(account_id, 1).await.unwrap();
    
    assert!(processor.validate_transaction(&SignedTransaction::new(tx1, &[1u8; 64])).await.is_err());
    assert!(processor.validate_transaction(&SignedTransaction::new(tx2, &[2u8; 64])).await.is_err());
}

/// 测试交易处理流水线
#[tokio::test]
async fn test_transaction_processing_pipeline() {
    let mempool = Arc::new(MemPool::new(100));
    let processor = TransactionProcessor::new();
    
    // 准备测试交易
    let tx = Transaction::new_transfer(1, 2, 100, 1);
    let signed_tx = SignedTransaction::new(tx, &[1u8; 64]);
    
    // 提交到交易池
    mempool.add_transaction(signed_tx.clone()).await.unwrap();
    
    // 从交易池获取待处理交易
    let pending_txs = mempool.get_transactions_for_block(10).await;
    
    // 处理交易（需要一个完整的区块链状态，这里只验证流程）
    assert!(!pending_txs.is_empty());
    
    let result = processor.process_transaction(signed_tx).await;
    assert!(result.is_ok());
}

/// 测试批量交易处理
#[tokio::test]
async fn test_batch_transaction_processing() {
    let processor = TransactionProcessor::new();
    
    let mut txs = vec![];
    for i in 0..10 {
        let tx = Transaction::new_transfer(i as u64, (i + 1) as u64, 10, 1);
        let signed = SignedTransaction::new(tx, &[i as u8; 64]);
        txs.push(signed);
    }
    
    let results = processor.process_batch(txs).await;
    assert_eq!(results.len(), 10);
    // 所有交易都应该成功（没有依赖冲突）
    assert!(results.iter().all(|r| r.is_ok()));
}