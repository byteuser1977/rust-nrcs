//! Transaction Listener Module
//!
//! 对照 Java NRCS TransactionProcessor + Listeners + TransactionProcessEvent 实现。
//!
//! Java NRCS 事件类型:
//! - REMOVED_UNCONFIRMED_TRANSACTIONS: 未确认交易被移除
//! - ADDED_UNCONFIRMED_TRANSACTIONS: 新增未确认交易
//! - ADDED_CONFIRMED_TRANSACTIONS: 交易被确认（上链）
//! - RELEASE_PHASED_TRANSACTION: 分阶段交易释放
//! - REJECT_PHASED_TRANSACTION: 分阶段交易被拒绝
//!
//! 参考:
//! - /mnt/d/workspace/git/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/transaction/TransactionProcessor.java
//! - /mnt/d/workspace/git/nrcs/nrcs-service-common/src/main/java/com/bytechain/nrcs/service/common/enums/transaction/TransactionProcessEvent.java
//! - /mnt/d/workspace/git/nrcs/nrcs-utils/src/main/java/com/bytechain/nrcs/utils/block/Listeners.java

use std::sync::{Arc, RwLock};
use blockchain_types::prelude::Transaction;

/// 交易处理事件类型
///
/// 对照 Java NRCS TransactionProcessEvent 枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransactionProcessEvent {
    RemovedUnconfirmedTransactions,
    AddedUnconfirmedTransactions,
    AddedConfirmedTransactions,
    ReleasePhasedTransaction,
    RejectPhasedTransaction,
}

impl TransactionProcessEvent {
    pub fn name(&self) -> &'static str {
        match self {
            TransactionProcessEvent::RemovedUnconfirmedTransactions => "REMOVED_UNCONFIRMED_TRANSACTIONS",
            TransactionProcessEvent::AddedUnconfirmedTransactions => "ADDED_UNCONFIRMED_TRANSACTIONS",
            TransactionProcessEvent::AddedConfirmedTransactions => "ADDED_CONFIRMED_TRANSACTIONS",
            TransactionProcessEvent::ReleasePhasedTransaction => "RELEASE_PHASED_TRANSACTION",
            TransactionProcessEvent::RejectPhasedTransaction => "REJECT_PHASED_TRANSACTION",
        }
    }
}

/// 交易事件类型（兼容旧版）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionEvent {
    Added,
    Removed,
    Processed,
    Broadcasted,
    Confirmed,
    Expired,
}

impl From<TransactionProcessEvent> for TransactionEvent {
    fn from(e: TransactionProcessEvent) -> Self {
        match e {
            TransactionProcessEvent::AddedUnconfirmedTransactions => TransactionEvent::Added,
            TransactionProcessEvent::AddedConfirmedTransactions => TransactionEvent::Confirmed,
            TransactionProcessEvent::RemovedUnconfirmedTransactions => TransactionEvent::Removed,
            TransactionProcessEvent::ReleasePhasedTransaction => TransactionEvent::Processed,
            TransactionProcessEvent::RejectPhasedTransaction => TransactionEvent::Expired,
        }
    }
}

/// 交易事件数据
#[derive(Debug, Clone)]
pub struct TransactionEventData {
    pub event: TransactionEvent,
    pub process_event: Option<TransactionProcessEvent>,
    pub transactions: Vec<Transaction>,
    pub timestamp: u64,
}

impl TransactionEventData {
    pub fn new(event: TransactionEvent, transactions: Vec<Transaction>) -> Self {
        Self {
            event,
            process_event: None,
            transactions,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    pub fn from_process_event(process_event: TransactionProcessEvent, transactions: Vec<Transaction>) -> Self {
        Self {
            event: process_event.into(),
            process_event: Some(process_event),
            transactions,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }
}

/// 交易监听器 Trait
///
/// 对照 Java NRCS Listener<T> 接口
pub trait TransactionListener: Send + Sync {
    fn notify(&self, event: &TransactionEventData);
}

/// 监听器函数类型
type ListenerCallback = Box<dyn Fn(&TransactionEventData) + Send + Sync>;

/// 事件分发器
///
/// 对照 Java NRCS Listeners<T, E> 类，使用 ConcurrentHashMap<EventType, CopyOnWriteArrayList<Listener>>
/// Rust 实现使用 RwLock<Vec> 替代
pub struct EventDispatcher {
    listeners: RwLock<Vec<(TransactionEvent, ListenerCallback)>>,
    process_listeners: RwLock<Vec<(TransactionProcessEvent, ListenerCallback)>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            listeners: RwLock::new(Vec::new()),
            process_listeners: RwLock::new(Vec::new()),
        }
    }

    pub fn add_listener<F>(&self, event: TransactionEvent, callback: F)
    where
        F: Fn(&TransactionEventData) + Send + Sync + 'static,
    {
        self.listeners.write().unwrap().push((event, Box::new(callback)));
    }

    /// 注册 TransactionProcessEvent 类型的监听器
    ///
    /// 对照 Java NRCS TransactionProcessor.addListener(listener, TransactionProcessEvent)
    pub fn add_process_listener<F>(&self, event: TransactionProcessEvent, callback: F)
    where
        F: Fn(&TransactionEventData) + Send + Sync + 'static,
    {
        self.process_listeners.write().unwrap().push((event, Box::new(callback)));
    }

    pub fn notify(&self, event_data: &TransactionEventData) {
        let listeners = self.listeners.read().unwrap();
        for (event_type, callback) in listeners.iter() {
            if *event_type == event_data.event {
                callback(event_data);
            }
        }
        drop(listeners);

        if let Some(process_event) = event_data.process_event {
            let process_listeners = self.process_listeners.read().unwrap();
            for (event_type, callback) in process_listeners.iter() {
                if *event_type == process_event {
                    callback(event_data);
                }
            }
        }
    }

    /// 通知 TransactionProcessEvent 事件
    ///
    /// 对照 Java NRCS TransactionProcessor.notifyListeners(transactions, TransactionProcessEvent)
    pub fn notify_process_event(&self, process_event: TransactionProcessEvent, transactions: Vec<Transaction>) {
        let event_data = TransactionEventData::from_process_event(process_event, transactions);
        self.notify(&event_data);
    }

    pub fn remove_all_listeners(&self) {
        self.listeners.write().unwrap().clear();
        self.process_listeners.write().unwrap().clear();
    }

    pub fn listener_count(&self) -> usize {
        self.listeners.read().unwrap().len() + self.process_listeners.read().unwrap().len()
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

static EVENT_DISPATCHER: std::sync::OnceLock<Arc<EventDispatcher>> =
    std::sync::OnceLock::new();

pub fn add_listener<F>(event: TransactionEvent, callback: F)
where
    F: Fn(&TransactionEventData) + Send + Sync + 'static,
{
    EVENT_DISPATCHER.get_or_init(|| Arc::new(EventDispatcher::new())).add_listener(event, callback);
}

/// 注册 TransactionProcessEvent 类型的监听器
pub fn add_process_listener<F>(event: TransactionProcessEvent, callback: F)
where
    F: Fn(&TransactionEventData) + Send + Sync + 'static,
{
    EVENT_DISPATCHER.get_or_init(|| Arc::new(EventDispatcher::new())).add_process_listener(event, callback);
}

pub fn notify_listeners(event_data: &TransactionEventData) {
    if let Some(dispatcher) = EVENT_DISPATCHER.get() {
        dispatcher.notify(event_data);
    }
}

/// 通知 TransactionProcessEvent 事件
pub fn notify_process_event(process_event: TransactionProcessEvent, transactions: Vec<Transaction>) {
    if let Some(dispatcher) = EVENT_DISPATCHER.get() {
        dispatcher.notify_process_event(process_event, transactions);
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

    #[test]
    fn test_process_event_listener() {
        let dispatcher = EventDispatcher::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let c1 = counter.clone();
        dispatcher.add_process_listener(TransactionProcessEvent::AddedUnconfirmedTransactions, move |_| {
            c1.fetch_add(1, Ordering::SeqCst);
        });

        let c2 = counter.clone();
        dispatcher.add_process_listener(TransactionProcessEvent::AddedConfirmedTransactions, move |_| {
            c2.fetch_add(10, Ordering::SeqCst);
        });

        dispatcher.notify_process_event(TransactionProcessEvent::AddedUnconfirmedTransactions, vec![]);
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        dispatcher.notify_process_event(TransactionProcessEvent::AddedConfirmedTransactions, vec![]);
        assert_eq!(counter.load(Ordering::SeqCst), 11);
    }

    #[test]
    fn test_process_event_name() {
        assert_eq!(TransactionProcessEvent::AddedUnconfirmedTransactions.name(), "ADDED_UNCONFIRMED_TRANSACTIONS");
        assert_eq!(TransactionProcessEvent::RemovedUnconfirmedTransactions.name(), "REMOVED_UNCONFIRMED_TRANSACTIONS");
        assert_eq!(TransactionProcessEvent::AddedConfirmedTransactions.name(), "ADDED_CONFIRMED_TRANSACTIONS");
        assert_eq!(TransactionProcessEvent::ReleasePhasedTransaction.name(), "RELEASE_PHASED_TRANSACTION");
        assert_eq!(TransactionProcessEvent::RejectPhasedTransaction.name(), "REJECT_PHASED_TRANSACTION");
    }
}
