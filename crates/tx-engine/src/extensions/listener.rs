//! Transaction Listener Module
//!
//! 对应 Java: TransactionProcessor.addListener(), notifyListeners()
//!
//! 事件监听机制

use std::sync::{Arc, RwLock};
use blockchain_types::prelude::Transaction;

/// 交易事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionEvent {
    Added,
    Removed,
    Processed,
    Broadcasted,
    Confirmed,
    Expired,
}

/// 交易事件
#[derive(Debug, Clone)]
pub struct TransactionEventData {
    pub event: TransactionEvent,
    pub transactions: Vec<Transaction>,
    pub timestamp: u64,
}

impl TransactionEventData {
    pub fn new(event: TransactionEvent, transactions: Vec<Transaction>) -> Self {
        Self {
            event,
            transactions,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }
}

/// 交易监听器 Trait
pub trait TransactionListener: Send + Sync {
    fn notify(&self, event: &TransactionEventData);
}

/// 监听器函数类型
type ListenerCallback = Box<dyn Fn(&TransactionEventData) + Send + Sync>;

/// 事件分发器
pub struct EventDispatcher {
    listeners: RwLock<Vec<(TransactionEvent, ListenerCallback)>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            listeners: RwLock::new(Vec::new()),
        }
    }

    pub fn add_listener<F>(&self, event: TransactionEvent, callback: F)
    where
        F: Fn(&TransactionEventData) + Send + Sync + 'static,
    {
        self.listeners.write().unwrap().push((event, Box::new(callback)));
    }

    pub fn notify(&self, event_data: &TransactionEventData) {
        let listeners = self.listeners.read().unwrap();
        for (event_type, callback) in listeners.iter() {
            if *event_type == event_data.event {
                callback(event_data);
            }
        }
    }

    pub fn remove_all_listeners(&self) {
        self.listeners.write().unwrap().clear();
    }

    pub fn listener_count(&self) -> usize {
        self.listeners.read().unwrap().len()
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局事件分发器
static EVENT_DISPATCHER: std::sync::OnceLock<Arc<EventDispatcher>> = 
    std::sync::OnceLock::new();

pub fn add_listener<F>(event: TransactionEvent, callback: F)
where
    F: Fn(&TransactionEventData) + Send + Sync + 'static,
{
    EVENT_DISPATCHER.get_or_init(|| Arc::new(EventDispatcher::new())).add_listener(event, callback);
}

pub fn notify_listeners(event_data: &TransactionEventData) {
    if let Some(dispatcher) = EVENT_DISPATCHER.get() {
        dispatcher.notify(event_data);
    }
}

pub fn get_dispatcher() -> Arc<EventDispatcher> {
    Arc::clone(EVENT_DISPATCHER.get_or_init(|| Arc::new(EventDispatcher::new())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_event_dispatcher() {
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
    fn test_multiple_listeners() {
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
}
