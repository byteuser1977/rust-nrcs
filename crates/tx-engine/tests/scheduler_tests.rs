//! Scheduler 外部测试
//!
//! 测试 TransactionScheduler、ScheduledTransaction、TransactionFilter 等调度器功能

use tx_engine::scheduler::{
    TransactionScheduler, SchedulerError, ScheduledTransaction,
};
use tx_engine::scheduler::transaction_scheduler::{
    DefaultFilter, SenderFilter, HeightFilter, TransactionFilter,
};
use std::sync::Arc;

fn make_scheduled_tx(id: u64, sender_id: u64, expiration: i32) -> ScheduledTransaction {
    ScheduledTransaction::new(id, sender_id, vec![1, 2, 3], expiration, 0)
}

#[test]
fn test_scheduled_transaction_new() {
    let tx = make_scheduled_tx(1, 100, 1000);
    assert_eq!(tx.id, 1);
    assert_eq!(tx.sender_id, 100);
    assert_eq!(tx.expiration, 1000);
    assert_eq!(tx.created_at, 0);
    assert_eq!(tx.transaction_data, vec![1, 2, 3]);
}

#[test]
fn test_scheduled_transaction_is_expired() {
    let tx = make_scheduled_tx(1, 100, 1000);
    assert!(!tx.is_expired(500), "Not expired yet");
    assert!(!tx.is_expired(1000), "At expiration boundary");
    assert!(tx.is_expired(1001), "Past expiration");
}

#[test]
fn test_scheduled_transaction_is_expired_zero() {
    let tx = make_scheduled_tx(1, 100, 0);
    assert!(tx.is_expired(1), "Zero expiration means already expired");
}

#[tokio::test]
async fn test_scheduler_new() {
    let scheduler = TransactionScheduler::new();
    assert_eq!(scheduler.count().await, 0);
}

#[tokio::test]
async fn test_scheduler_default() {
    let scheduler = TransactionScheduler::default();
    assert_eq!(scheduler.count().await, 0);
}

#[tokio::test]
async fn test_scheduler_schedule() {
    let scheduler = TransactionScheduler::new();
    let tx = make_scheduled_tx(1, 100, 1000);
    let filter = Arc::new(DefaultFilter);

    scheduler.schedule(tx, filter).await.unwrap();
    assert_eq!(scheduler.count().await, 1);
}

#[tokio::test]
async fn test_scheduler_schedule_multiple() {
    let scheduler = TransactionScheduler::new();

    for i in 1..=5 {
        let tx = make_scheduled_tx(i, 100, 1000);
        scheduler.schedule(tx, Arc::new(DefaultFilter)).await.unwrap();
    }

    assert_eq!(scheduler.count().await, 5);
}

#[tokio::test]
async fn test_scheduler_limit_exceeded() {
    let scheduler = TransactionScheduler::with_max_scheduled(2);

    let tx1 = make_scheduled_tx(1, 100, 1000);
    let tx2 = make_scheduled_tx(2, 200, 1000);
    let tx3 = make_scheduled_tx(3, 300, 1000);

    scheduler.schedule(tx1, Arc::new(DefaultFilter)).await.unwrap();
    scheduler.schedule(tx2, Arc::new(DefaultFilter)).await.unwrap();

    let result = scheduler.schedule(tx3, Arc::new(DefaultFilter)).await;
    assert!(matches!(result, Err(SchedulerError::LimitExceeded(_))));
}

#[tokio::test]
async fn test_scheduler_get_scheduled_transactions() {
    let scheduler = TransactionScheduler::new();

    let tx1 = make_scheduled_tx(1, 100, 1000);
    let tx2 = make_scheduled_tx(2, 200, 1000);
    let tx3 = make_scheduled_tx(3, 100, 1000);

    scheduler.schedule(tx1, Arc::new(DefaultFilter)).await.unwrap();
    scheduler.schedule(tx2, Arc::new(DefaultFilter)).await.unwrap();
    scheduler.schedule(tx3, Arc::new(DefaultFilter)).await.unwrap();

    let account_txs = scheduler.get_scheduled_transactions(100).await;
    assert_eq!(account_txs.len(), 2);

    let account_txs = scheduler.get_scheduled_transactions(200).await;
    assert_eq!(account_txs.len(), 1);

    let account_txs = scheduler.get_scheduled_transactions(999).await;
    assert_eq!(account_txs.len(), 0);
}

#[tokio::test]
async fn test_scheduler_get_all_scheduled() {
    let scheduler = TransactionScheduler::new();

    scheduler.schedule(make_scheduled_tx(1, 100, 1000), Arc::new(DefaultFilter)).await.unwrap();
    scheduler.schedule(make_scheduled_tx(2, 200, 1000), Arc::new(DefaultFilter)).await.unwrap();

    let all = scheduler.get_all_scheduled().await;
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn test_scheduler_remove() {
    let scheduler = TransactionScheduler::new();

    scheduler.schedule(make_scheduled_tx(1, 100, 1000), Arc::new(DefaultFilter)).await.unwrap();
    scheduler.schedule(make_scheduled_tx(2, 200, 1000), Arc::new(DefaultFilter)).await.unwrap();

    let removed = scheduler.remove(1).await;
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().id, 1);
    assert_eq!(scheduler.count().await, 1);

    let removed = scheduler.remove(999).await;
    assert!(removed.is_none());
}

#[tokio::test]
async fn test_scheduler_clear() {
    let scheduler = TransactionScheduler::new();

    for i in 1..=5 {
        scheduler.schedule(make_scheduled_tx(i, 100, 1000), Arc::new(DefaultFilter)).await.unwrap();
    }

    scheduler.clear().await;
    assert_eq!(scheduler.count().await, 0);
}

#[tokio::test]
async fn test_scheduler_remove_expired() {
    let scheduler = TransactionScheduler::new();

    scheduler.schedule(make_scheduled_tx(1, 100, 500), Arc::new(DefaultFilter)).await.unwrap();
    scheduler.schedule(make_scheduled_tx(2, 200, 1500), Arc::new(DefaultFilter)).await.unwrap();
    scheduler.schedule(make_scheduled_tx(3, 300, 800), Arc::new(DefaultFilter)).await.unwrap();

    let expired = scheduler.remove_expired(1000).await;
    assert_eq!(expired.len(), 2);
    assert!(expired.contains(&1));
    assert!(expired.contains(&3));
    assert_eq!(scheduler.count().await, 1);
}

#[tokio::test]
async fn test_scheduler_remove_expired_none() {
    let scheduler = TransactionScheduler::new();

    scheduler.schedule(make_scheduled_tx(1, 100, 5000), Arc::new(DefaultFilter)).await.unwrap();

    let expired = scheduler.remove_expired(1000).await;
    assert!(expired.is_empty());
    assert_eq!(scheduler.count().await, 1);
}

#[tokio::test]
async fn test_scheduler_process_event_expired() {
    let scheduler = TransactionScheduler::new();

    scheduler.schedule(make_scheduled_tx(1, 100, 500), Arc::new(DefaultFilter)).await.unwrap();
    scheduler.schedule(make_scheduled_tx(2, 200, 5000), Arc::new(DefaultFilter)).await.unwrap();

    let removed = scheduler.process_event(&[], 1000).await;
    assert_eq!(removed.len(), 2, "Both should be removed: 1 expired, 2 filter match");
}

#[tokio::test]
async fn test_scheduler_process_event_filter_match() {
    let scheduler = TransactionScheduler::new();

    scheduler.schedule(make_scheduled_tx(1, 100, 5000), Arc::new(DefaultFilter)).await.unwrap();

    let removed = scheduler.process_event(&[1, 2, 3], 1000).await;
    assert_eq!(removed.len(), 1, "DefaultFilter always matches");
}

#[tokio::test]
async fn test_scheduler_sender_filter() {
    let filter = SenderFilter { sender_id: 100 };

    let mut tx_data = vec![0u8; 100];
    tx_data[0..8].copy_from_slice(&100u64.to_le_bytes());
    assert!(filter.check(&tx_data).await);

    tx_data[0..8].copy_from_slice(&200u64.to_le_bytes());
    assert!(!filter.check(&tx_data).await);
}

#[tokio::test]
async fn test_scheduler_sender_filter_short_data() {
    let filter = SenderFilter { sender_id: 100 };
    assert!(!filter.check(&[1, 2, 3]).await);
}

#[tokio::test]
async fn test_scheduler_height_filter() {
    let filter = HeightFilter {
        required_height: 100,
        current_height: 50,
    };
    assert!(!filter.check(&[]).await);

    let filter = HeightFilter {
        required_height: 100,
        current_height: 150,
    };
    assert!(filter.check(&[]).await);
}

#[tokio::test]
async fn test_scheduler_replace_same_id() {
    let scheduler = TransactionScheduler::new();

    let tx1 = make_scheduled_tx(1, 100, 1000);
    scheduler.schedule(tx1, Arc::new(DefaultFilter)).await.unwrap();
    assert_eq!(scheduler.count().await, 1);

    let tx2 = make_scheduled_tx(1, 200, 2000);
    scheduler.schedule(tx2, Arc::new(DefaultFilter)).await.unwrap();
    assert_eq!(scheduler.count().await, 1, "Same ID should replace");

    let all = scheduler.get_all_scheduled().await;
    assert_eq!(all[0].sender_id, 200);
    assert_eq!(all[0].expiration, 2000);
}
