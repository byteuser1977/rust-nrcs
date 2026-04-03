//! 交易处理器（TransactionProcessor）测试

use crate::*;
use blockchain_types::*;

/// 测试处理器创建
#[test]
fn test_processor_creation() {
    let processor = TransactionProcessor::new();
    assert!(processor.is_ok());
}

/// 测试交易执行 - 成功场景
#[tokio::test]
async fn test_process_transaction_success() {
    let processor = TransactionProcessor::new();
    
    let tx = Transaction::new_transfer(
        1, // from
        2, // to
        100, // amount
        1  // nonce
    );
    
    let signature = [1u8; 64]; // mock signature
    let signed_tx = SignedTransaction::new(tx, &signature);
    
    let result = processor.process_transaction(signed_tx).await;
    assert!(result.is_ok());
}

/// 测试交易执行 - 无效签名
#[tokio::test]
async fn test_process_transaction_invalid_signature() {
    let processor = TransactionProcessor::new();
    
    let tx = Transaction::new_transfer(1, 2, 100, 1);
    let invalid_signature = [0u8; 64];
    let signed_tx = SignedTransaction::new(tx, &invalid_signature);
    
    let result = processor.process_transaction(signed_tx).await;
    assert!(result.is_err());
}

/// 测试交易执行 - 余额不足
#[tokio::test]
async fn test_process_transaction_insufficient_balance() {
    let processor = TransactionProcessor::new();
    
    let tx = Transaction::new_transfer(1, 2, 1000, 1);
    let signature = [1u8; 64];
    let signed_tx = SignedTransaction::new(tx, &signature);
    
    let result = processor.process_transaction(signed_tx).await;
    assert!(result.is_err());
}

/// 测试交易去重
#[tokio::test]
async fn test_transaction_deduplication() {
    let processor = TransactionProcessor::new();
    
    let tx = Transaction::new_transfer(1, 2, 100, 1);
    let signature = [1u8; 64];
    let signed_tx = SignedTransaction::new(tx, &signature);
    
    // 第一次处理成功
    let result1 = processor.process_transaction(signed_tx.clone()).await;
    assert!(result1.is_ok());
    
    // 第二次同样的交易应该失败（重复）
    let result2 = processor.process_transaction(signed_tx).await;
    assert!(result2.is_err());
}

/// 测试 nonce 顺序验证
#[tokio::test]
async fn test_nonce_ordering() {
    let processor = TransactionProcessor::new();
    
    // 先发送 nonce=1
    let tx1 = Transaction::new_transfer(1, 2, 100, 1);
    let signed1 = SignedTransaction::new(tx1, &[1u8; 64]);
    assert!(processor.process_transaction(signed1.clone()).await.is_ok());
    
    // 再发送 nonce=1 应该失败
    let tx2 = Transaction::new_transfer(1, 3, 100, 1);
    let signed2 = SignedTransaction::new(tx2, &[2u8; 64]);
    assert!(processor.process_transaction(signed2).await.is_err());
    
    // 发送 nonce=2 应该成功
    let tx3 = Transaction::new_transfer(1, 4, 100, 2);
    let signed3 = SignedTransaction::new(tx3, &[3u8; 64]);
    assert!(processor.process_transaction(signed3).await.is_ok());
}