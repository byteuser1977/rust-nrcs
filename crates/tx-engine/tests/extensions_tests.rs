//! Extensions 外部测试
//!
//! 测试 Hub、PrunableTransaction、ReferencedTransaction、PurchaseFeedback 等扩展功能

use tx_engine::extensions::Hub;
use tx_engine::extensions::hub::{HubStore, HubHit};
use tx_engine::extensions::prunable::{PrunableTransaction, PrunableData, PrunableDataType};
use tx_engine::extensions::referenced::ReferencedTransaction;
use tx_engine::extensions::feedback::{PurchaseFeedback, PurchasePublicFeedback, EncryptedData, FeedbackStore};
use tx_engine::extensions::listener::{EventDispatcher, TransactionEventData, TransactionEvent};
use blockchain_types::prelude::*;

#[test]
fn test_hub_creation() {
    let hub = Hub::new(12345, 100, vec!["http://hub.example.com".to_string()], 100);
    assert_eq!(hub.get_account_id(), 12345);
    assert_eq!(hub.get_min_fee_per_byte(), 100);
    assert_eq!(hub.get_uris().len(), 1);
    assert_eq!(hub.get_height(), 100);
}

#[test]
fn test_hub_store_add() {
    let store = HubStore::new();
    store.add_or_update(Hub::new(1, 100, vec![], 10));
    store.add_or_update(Hub::new(2, 200, vec![], 20));

    assert_eq!(store.get_all().len(), 2);
}

#[test]
fn test_hub_store_update() {
    let store = HubStore::new();
    store.add_or_update(Hub::new(1, 100, vec![], 10));
    store.add_or_update(Hub::new(1, 200, vec![], 20));

    assert_eq!(store.get_all().len(), 1, "Should update existing");
    let hub = store.get_by_account(1).unwrap();
    assert_eq!(hub.get_min_fee_per_byte(), 200);
}

#[test]
fn test_hub_store_get_by_account() {
    let store = HubStore::new();
    store.add_or_update(Hub::new(1, 100, vec![], 10));

    assert!(store.get_by_account(1).is_some());
    assert!(store.get_by_account(999).is_none());
}

#[test]
fn test_hub_store_remove() {
    let store = HubStore::new();
    store.add_or_update(Hub::new(1, 100, vec![], 10));
    store.add_or_update(Hub::new(2, 200, vec![], 20));

    store.remove(1);
    assert_eq!(store.get_all().len(), 1);
    assert!(store.get_by_account(1).is_none());
    assert!(store.get_by_account(2).is_some());
}

#[test]
fn test_hub_hit_compare() {
    let hub1 = Hub::new(1, 100, vec![], 10);
    let hub2 = Hub::new(2, 100, vec![], 10);

    let hit1 = HubHit::new(hub1, 100);
    let hit2 = HubHit::new(hub2, 200);

    assert_eq!(hit1.compare(&hit2), std::cmp::Ordering::Less);
    assert_eq!(hit2.compare(&hit1), std::cmp::Ordering::Greater);
}

#[test]
fn test_hub_hit_compare_same_time() {
    let hub1 = Hub::new(1, 100, vec![], 10);
    let hub2 = Hub::new(2, 100, vec![], 10);

    let hit1 = HubHit::new(hub1, 100);
    let hit2 = HubHit::new(hub2, 100);

    assert_eq!(hit1.compare(&hit2), std::cmp::Ordering::Less, "Lower account_id first");
}

#[test]
fn test_hub_store_get_hub_hits() {
    let store = HubStore::new();
    store.add_or_update(Hub::new(1, 100, vec![], 10));
    store.add_or_update(Hub::new(2, 200, vec![], 20));

    let hits = store.get_hub_hits(1, 1000);
    assert_eq!(hits.len(), 2);

    let cached_hits = store.get_hub_hits(1, 1000);
    assert_eq!(cached_hits.len(), 2, "Should return cached hits for same block");
}

#[test]
fn test_prunable_transaction_creation() {
    let pt = PrunableTransaction::new(12345, TransactionType::Messaging, true, false, true);
    assert_eq!(pt.get_id(), 12345);
    assert_eq!(pt.get_transaction_type(), TransactionType::Messaging);
    assert!(pt.has_prunable_attachment());
    assert!(!pt.has_prunable_plain_message());
    assert!(pt.has_prunable_encrypted_message());
}

#[test]
fn test_prunable_transaction_is_pruned() {
    let pt = PrunableTransaction::new(1, TransactionType::Payment, true, false, false);
    assert!(pt.is_pruned());

    let pt = PrunableTransaction::new(2, TransactionType::Payment, false, true, false);
    assert!(pt.is_pruned());

    let pt = PrunableTransaction::new(3, TransactionType::Payment, false, false, false);
    assert!(!pt.is_pruned());
}

#[test]
fn test_prunable_data_creation() {
    let data = PrunableData::new(12345, PrunableDataType::PlainMessage, vec![1, 2, 3], Some(vec![4, 5, 6]), 100);
    assert_eq!(data.transaction_id, 12345);
    assert_eq!(data.data_type, PrunableDataType::PlainMessage);
    assert_eq!(data.data, vec![1, 2, 3]);
    assert_eq!(data.nonce, Some(vec![4, 5, 6]));
    assert_eq!(data.height, 100);
}

#[test]
fn test_prunable_data_is_expired() {
    let data = PrunableData::new(1, PrunableDataType::PlainMessage, vec![], None, 100);
    assert!(data.is_expired(200, 50), "200-100=100 > 50");
    assert!(!data.is_expired(120, 50), "120-100=20 <= 50");
}

#[test]
fn test_prunable_data_not_expired() {
    let data = PrunableData::new(1, PrunableDataType::EncryptedMessage, vec![], None, 100);
    assert!(!data.is_expired(149, 50), "149-100=49 <= 50");
    assert!(data.is_expired(151, 50), "151-100=51 > 50");
}

#[test]
fn test_referenced_transaction_creation() {
    let hash = Hash256([1u8; 32]);
    let rt = ReferencedTransaction::new(12345, 67890, hash, 100);

    assert_eq!(rt.get_referenced_transaction_id(), 67890);
    assert!(rt.verify_hash(&Hash256([1u8; 32])));
    assert!(!rt.verify_hash(&Hash256([2u8; 32])));
}

#[test]
fn test_purchase_feedback() {
    let encrypted = EncryptedData::new(vec![1, 2, 3], vec![4, 5, 6]);
    let feedback = PurchaseFeedback::new(12345, encrypted, 100);

    assert_eq!(feedback.get_purchase_id(), 12345);
    assert_eq!(feedback.get_height(), 100);
    let enc = feedback.get_encrypted_data();
    assert_eq!(enc.data, vec![1, 2, 3]);
    assert_eq!(enc.nonce, vec![4, 5, 6]);
}

#[test]
fn test_public_feedback() {
    let feedback = PurchasePublicFeedback::new(12345, vec![1, 2, 3, 4], 100);

    assert_eq!(feedback.get_purchase_id(), 12345);
    assert_eq!(feedback.get_public_feedback(), &[1, 2, 3, 4]);
    assert_eq!(feedback.get_height(), 100);
}

#[test]
fn test_feedback_store() {
    let mut store = FeedbackStore::new();

    let encrypted = EncryptedData::new(vec![1, 2, 3], vec![4, 5, 6]);
    store.add_feedback(PurchaseFeedback::new(1, encrypted.clone(), 100));
    store.add_feedback(PurchaseFeedback::new(2, encrypted, 200));

    store.add_public_feedback(PurchasePublicFeedback::new(1, vec![10, 20], 100));

    assert!(store.get_feedback(1).is_some());
    assert!(store.get_feedback(2).is_some());
    assert!(store.get_feedback(3).is_none());

    assert!(store.get_public_feedback(1).is_some());
    assert!(store.get_public_feedback(2).is_none());
}

#[test]
fn test_event_dispatcher() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let dispatcher = EventDispatcher::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    dispatcher.add_listener(TransactionEvent::Added, move |_| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    let event_data = TransactionEventData::new(TransactionEvent::Added, vec![]);
    dispatcher.notify(&event_data);

    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_event_dispatcher_multiple_listeners() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let dispatcher = EventDispatcher::new();
    let counter = Arc::new(AtomicUsize::new(0));

    let c1 = counter.clone();
    dispatcher.add_listener(TransactionEvent::Added, move |_| {
        c1.fetch_add(1, Ordering::SeqCst);
    });

    let c2 = counter.clone();
    dispatcher.add_listener(TransactionEvent::Added, move |_| {
        c2.fetch_add(10, Ordering::SeqCst);
    });

    let event_data = TransactionEventData::new(TransactionEvent::Added, vec![]);
    dispatcher.notify(&event_data);

    assert_eq!(counter.load(Ordering::SeqCst), 11);
}

#[test]
fn test_event_dispatcher_selective_notify() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let dispatcher = EventDispatcher::new();
    let counter = Arc::new(AtomicUsize::new(0));

    let c1 = counter.clone();
    dispatcher.add_listener(TransactionEvent::Added, move |_| {
        c1.fetch_add(1, Ordering::SeqCst);
    });

    let c2 = counter.clone();
    dispatcher.add_listener(TransactionEvent::Removed, move |_| {
        c2.fetch_add(10, Ordering::SeqCst);
    });

    let event_data = TransactionEventData::new(TransactionEvent::Added, vec![]);
    dispatcher.notify(&event_data);

    assert_eq!(counter.load(Ordering::SeqCst), 1, "Only Added listener should fire");
}

#[test]
fn test_event_dispatcher_remove_all() {
    let dispatcher = EventDispatcher::new();
    dispatcher.add_listener(TransactionEvent::Added, |_| {});
    dispatcher.add_listener(TransactionEvent::Removed, |_| {});

    assert_eq!(dispatcher.listener_count(), 2);
    dispatcher.remove_all_listeners();
    assert_eq!(dispatcher.listener_count(), 0);
}

#[test]
fn test_event_dispatcher_listener_count() {
    let dispatcher = EventDispatcher::new();
    assert_eq!(dispatcher.listener_count(), 0);

    dispatcher.add_listener(TransactionEvent::Added, |_| {});
    assert_eq!(dispatcher.listener_count(), 1);

    dispatcher.add_listener(TransactionEvent::Confirmed, |_| {});
    assert_eq!(dispatcher.listener_count(), 2);
}
