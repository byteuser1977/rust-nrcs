//! 账户模块集成测试

use crate::AccountManager;
use blockchain_types::*;
use std::sync::Arc;

/// 测试账户创建
#[tokio::test]
async fn test_create_account() {
    let manager = Arc::new(crate::account::InMemoryAccountManager::new());
    
    let pubkey = [1u8; 32];
    let account_id = manager.create_account(pubkey).await.unwrap();
    
    assert!(account_id > 0);
    
    let account = manager.get_account(account_id).await.unwrap();
    assert_eq!(account.pubkey, pubkey);
    assert_eq!(account.balance, 0);
}

/// 测试余额查询
#[tokio::test]
async fn test_balance_query() {
    let manager = Arc::new(crate::account::InMemoryAccountManager::new());
    
    let pubkey = [2u8; 32];
    let account_id = manager.create_account(pubkey).await.unwrap();
    
    // 初始余额应为0
    assert_eq!(manager.get_balance(account_id).await.unwrap(), 0);
}

/// 测试转账功能
#[tokio::test]
async fn test_transfer() {
    let manager = Arc::new(crate::account::InMemoryAccountManager::new());
    
    let pubkey1 = [3u8; 32];
    let pubkey2 = [4u8; 32];
    
    let from_id = manager.create_account(pubkey1).await.unwrap();
    let to_id = manager.create_account(pubkey2).await.unwrap();
    
    // 为发送者充值
    manager.deposit_balance(from_id, 1000).await.unwrap();
    
    // 执行转账
    let tx = Transaction::new_transfer(from_id, to_id, 500, 1);
    manager.transfer(tx).await.unwrap();
    
    // 验证余额
    assert_eq!(manager.get_balance(from_id).await.unwrap(), 500);
    assert_eq!(manager.get_balance(to_id).await.unwrap(), 500);
}

/// 测试重复转账
#[tokio::test]
#[should_panic(expected = "InsufficientBalance")]
async fn test_transfer_insufficient_balance() {
    let manager = Arc::new(crate::account::InMemoryAccountManager::new());
    
    let pubkey1 = [5u8; 32];
    let pubkey1_id = manager.create_account(pubkey1).await.unwrap();
    
    manager.deposit_balance(pubkey1_id, 100).await.unwrap();
    
    let pubkey2 = [6u8; 32];
    let pubkey2_id = manager.create_account(pubkey2).await.unwrap();
    
    let tx = Transaction::new_transfer(pubkey1_id, pubkey2_id, 200, 1);
    manager.transfer(tx).await.unwrap();
}

/// 测试并发余额更新
#[tokio::test]
async fn test_concurrent_balance_updates() {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    let manager = Arc::new(crate::account::InMemoryAccountManager::new());
    let pubkey = [7u8; 32];
    let account_id = manager.create_account(pubkey).await.unwrap();
    
    // 并发存款
    let mut tasks = vec![];
    for i in 0..10 {
        let manager_clone = manager.clone();
        let task = tokio::spawn(async move {
            manager_clone.deposit_balance(account_id, 100).await.unwrap();
        });
        tasks.push(task);
    }
    
    for task in tasks {
        task.await.unwrap();
    }
    
    // 总余额应为 1000
    assert_eq!(manager.get_balance(account_id).await.unwrap(), 1000);
}