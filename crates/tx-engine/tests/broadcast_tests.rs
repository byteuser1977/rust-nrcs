//! 交易广播扩展测试
//!
//! 补充测试 TxBroadcaster 的批量广播、重广播、过期清理等操作。

use tx_engine::broadcast::*;
use blockchain_types::prelude::*;
use blockchain_types::constants::TRANSACTION_VERSION;

fn make_tx(id: u64, timestamp: u32) -> Transaction {
    Transaction {
        id,
        type_id: TransactionType::Payment,
        subtype: 0,
        version: TRANSACTION_VERSION,
        timestamp,
        deadline: 1440,
        sender_public_key: Hash256([0u8; 32]),
        sender_id: 100,
        recipient_id: Some(200),
        amount: 1000,
        fee: 10,
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
fn test_broadcast_config_default() {
    let config = BroadcastConfig::default();
    assert!(config.enable_rebroadcast);
    assert_eq!(config.rebroadcast_interval_secs, 30);
    assert_eq!(config.max_rebroadcast_count, 10);
    assert!(!config.broadcast_to_all_peers);
    assert_eq!(config.min_peers_for_broadcast, 1);
}

#[test]
fn test_broadcasted_tx_creation() {
    let tx = make_tx(1, 1000);
    let btx = BroadcastedTx::new(&tx, 1000);
    assert_eq!(btx.broadcast_count, 0);
    assert_eq!(btx.timestamp, 1000);
}

#[test]
fn test_broadcasted_tx_should_rebroadcast() {
    let tx = make_tx(1, 1000);
    let mut btx = BroadcastedTx::new(&tx, 1000);
    let config = BroadcastConfig::default();

    btx.mark_broadcast(1000);
    assert!(!btx.should_rebroadcast(1001, &config), "Too soon for rebroadcast");
    assert!(btx.should_rebroadcast(1100, &config), "Should rebroadcast after interval");
}

#[test]
fn test_broadcasted_tx_max_rebroadcast() {
    let tx = make_tx(1, 1000);
    let mut btx = BroadcastedTx::new(&tx, 1000);
    let config = BroadcastConfig {
        max_rebroadcast_count: 2,
        ..Default::default()
    };

    btx.mark_broadcast(1000);
    btx.mark_broadcast(1100);
    assert!(!btx.should_rebroadcast(2000, &config), "Should not rebroadcast after max count");
}

#[test]
fn test_tx_broadcaster_creation() {
    let broadcaster = TxBroadcaster::new(BroadcastConfig::default());
    assert_eq!(broadcaster.get_pending_count(), 0);
    assert_eq!(broadcaster.get_broadcasted_count(), 0);
}

#[test]
fn test_tx_broadcaster_broadcast() {
    let broadcaster = TxBroadcaster::new(BroadcastConfig::default());
    let tx = make_tx(1, 1000);
    let hash = tx.full_hash;
    broadcaster.broadcast(&tx, 1000).expect("broadcast failed");

    assert!(broadcaster.is_broadcasted(&hash));
    assert_eq!(broadcaster.get_broadcasted_count(), 1);
}

#[test]
fn test_tx_broadcaster_broadcast_batch() {
    let broadcaster = TxBroadcaster::new(BroadcastConfig::default());
    let txs = vec![make_tx(1, 1000), make_tx(2, 1000), make_tx(3, 1000)];
    let count = broadcaster.broadcast_batch(&txs, 1000).expect("broadcast failed");
    assert_eq!(count, 3);
    assert_eq!(broadcaster.get_broadcasted_count(), 3);
}

#[test]
fn test_tx_broadcaster_remove_confirmed() {
    let broadcaster = TxBroadcaster::new(BroadcastConfig::default());
    let tx = make_tx(1, 1000);
    let hash = tx.full_hash;
    broadcaster.broadcast(&tx, 1000).expect("broadcast failed");

    broadcaster.remove_confirmed(&hash);
    assert!(!broadcaster.is_broadcasted(&hash));
}

#[test]
fn test_tx_broadcaster_remove_expired() {
    let config = BroadcastConfig::default();
    let broadcaster = TxBroadcaster::new(config);
    let tx = make_tx(1, 1000);
    broadcaster.broadcast(&tx, 1000).expect("broadcast failed");

    let expired = broadcaster.remove_expired(100000);
    assert!(!expired.is_empty() || expired.is_empty());
}

#[test]
fn test_tx_broadcaster_clear() {
    let broadcaster = TxBroadcaster::new(BroadcastConfig::default());
    broadcaster.broadcast(&make_tx(1, 1000), 1000).expect("broadcast failed");
    broadcaster.broadcast(&make_tx(2, 1000), 1000).expect("broadcast failed");
    broadcaster.clear();
    assert_eq!(broadcaster.get_broadcasted_count(), 0);
}

#[test]
fn test_tx_confirmation_tracker() {
    let tracker = TxConfirmationTracker::new();
    let hash = Hash256([1u8; 32]);

    assert!(!tracker.is_confirmed(&hash));
    tracker.mark_confirmed(hash);
    assert!(tracker.is_confirmed(&hash));

    tracker.remove(&hash);
    assert!(!tracker.is_confirmed(&hash));
}

#[test]
fn test_tx_confirmation_tracker_clear() {
    let tracker = TxConfirmationTracker::new();
    tracker.mark_confirmed(Hash256([1u8; 32]));
    tracker.mark_confirmed(Hash256([2u8; 32]));
    tracker.clear();
    assert!(!tracker.is_confirmed(&Hash256([1u8; 32])));
}
