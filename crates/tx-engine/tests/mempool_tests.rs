//! Mempool 集成测试
//!
//! 测试 Mempool 的核心操作：添加、查询、排序等。
//! 参照 Java NRCS 的 TransactionProcessorImpl 中未确认交易内存池逻辑。

use tx_engine::mempool::{Mempool, MempoolConfig, MempoolError};
use blockchain_types::prelude::*;
use blockchain_types::constants::TRANSACTION_VERSION;

fn make_config() -> MempoolConfig {
    MempoolConfig {
        max_txs: 100,
        max_memory_bytes: 10 * 1024 * 1024,
        tx_ttl_seconds: 86400,
        persistent: false,
        evict_by_fee: true,
    }
}

fn make_transaction(id: u64, sender_id: u64, recipient_id: u64, amount: u64, fee: u64) -> Transaction {
    Transaction {
        id,
        type_id: TransactionType::Payment,
        subtype: 0,
        version: TRANSACTION_VERSION,
        timestamp: 1000,
        deadline: 1440,
        sender_public_key: Hash256([0u8; 32]),
        sender_id,
        recipient_id: Some(recipient_id),
        amount,
        fee,
        height: 0,
        block_id: 0,
        block_timestamp: 0,
        transaction_index: 0,
        signature: Signature([0u8; 64]),
        full_hash: Hash256({
            let mut h = [0u8; 32];
            h[0..8].copy_from_slice(&id.to_le_bytes());
            h
        }),
        referenced_transaction_full_hash: None,
        attachment_bytes: vec![],
        pruned_attachment_bytes: 0,
        attachment_json: None,
        phased: false,
        has_message: false,
        has_encrypted_message: false,
        has_public_key_announcement: false,
        has_prunable_message: false,
        has_prunable_attachment: false,
        ec_block_height: None,
        ec_block_id: None,
        has_encrypttoself_message: false,
        has_prunable_encrypted_message: false,
    }
}

#[test]
fn test_mempool_creation() {
    let pool = Mempool::new(make_config());
    assert!(pool.get_all_sorted().is_empty());
}

#[test]
fn test_mempool_add_and_get() {
    let pool = Mempool::new(make_config());
    let tx = make_transaction(1, 100, 200, 1000, 10);
    pool.add(tx.clone()).expect("add failed");

    let retrieved = pool.get(&tx.full_hash);
    assert!(retrieved.is_some());
}

#[test]
fn test_mempool_add_duplicate() {
    let pool = Mempool::new(make_config());
    let tx = make_transaction(1, 100, 200, 1000, 10);
    pool.add(tx.clone()).expect("add failed");
    let result = pool.add(tx);
    assert!(result.is_err(), "Duplicate should be rejected");
}

#[test]
fn test_mempool_get_by_id() {
    let pool = Mempool::new(make_config());
    let tx = make_transaction(42, 100, 200, 1000, 10);
    pool.add(tx).expect("add failed");

    let found = pool.get_by_id(42);
    assert!(found.is_some());

    let not_found = pool.get_by_id(999);
    assert!(not_found.is_none());
}

#[test]
fn test_mempool_get_all_sorted() {
    let pool = Mempool::new(make_config());
    pool.add(make_transaction(1, 100, 200, 1000, 50)).expect("add failed");
    pool.add(make_transaction(2, 100, 200, 1000, 100)).expect("add failed");
    pool.add(make_transaction(3, 100, 200, 1000, 10)).expect("add failed");

    let sorted = pool.get_all_sorted();
    assert_eq!(sorted.len(), 3);
    assert!(sorted[0].fee >= sorted[1].fee);
}

#[test]
fn test_mempool_get_by_sender() {
    let pool = Mempool::new(make_config());
    pool.add(make_transaction(1, 100, 200, 1000, 10)).expect("add failed");
    pool.add(make_transaction(2, 100, 300, 2000, 20)).expect("add failed");
    pool.add(make_transaction(3, 400, 500, 3000, 30)).expect("add failed");

    let sender_txs = pool.get_by_sender(100);
    assert_eq!(sender_txs.len(), 2);
}

#[test]
fn test_mempool_clear() {
    let pool = Mempool::new(make_config());
    pool.add(make_transaction(1, 100, 200, 1000, 10)).expect("add failed");
    pool.add(make_transaction(2, 100, 300, 2000, 20)).expect("add failed");

    pool.clear();
    assert!(pool.get_all_sorted().is_empty());
}

#[test]
fn test_mempool_get_all_ids() {
    let pool = Mempool::new(make_config());
    pool.add(make_transaction(1, 100, 200, 1000, 10)).expect("add failed");
    pool.add(make_transaction(2, 100, 300, 2000, 20)).expect("add failed");

    let ids = pool.get_all_ids();
    assert_eq!(ids.len(), 2);
}

#[test]
fn test_mempool_broadcast_tracking() {
    let pool = Mempool::new(make_config());
    let tx = make_transaction(1, 100, 200, 1000, 10);
    let hash = tx.full_hash;
    pool.add(tx.clone()).expect("add failed");

    assert!(!pool.is_broadcasted(&hash));
    pool.add_broadcasted(&tx);
    assert!(pool.is_broadcasted(&hash));
}

#[test]
fn test_mempool_waiting_queue() {
    let pool = Mempool::new(make_config());
    let tx = make_transaction(1, 100, 200, 1000, 10);

    pool.process_later(vec![tx]);
    assert!(!pool.get_waiting().is_empty());

    pool.process_waiting();
    assert!(pool.get_waiting().is_empty());
}

#[test]
fn test_mempool_get_cached() {
    let pool = Mempool::new(make_config());
    pool.add(make_transaction(1, 100, 200, 1000, 10)).expect("add failed");
    pool.add(make_transaction(2, 100, 300, 2000, 20)).expect("add failed");

    let cached = pool.get_cached(&[]);
    assert_eq!(cached.len(), 2);

    let exclude_hash = make_transaction(1, 100, 200, 1000, 10).full_hash;
    let cached = pool.get_cached(&[exclude_hash]);
    assert_eq!(cached.len(), 1);
}

#[test]
fn test_mempool_get_all_broadcasted() {
    let pool = Mempool::new(make_config());
    let tx = make_transaction(1, 100, 200, 1000, 10);
    pool.add(tx.clone()).expect("add failed");
    pool.add_broadcasted(&tx);

    let broadcasted = pool.get_all_broadcasted();
    assert_eq!(broadcasted.len(), 1);
}
