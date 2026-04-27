//! Transaction Scheduler
//!
//! 对应 Java: TransactionScheduler.java

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use async_trait::async_trait;

use blockchain_types::AccountId;

/// Scheduler Error
#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("scheduler limit exceeded: {0}")]
    LimitExceeded(String),
    
    #[error("transaction expired: {0}")]
    Expired(String),
    
    #[error("validation error: {0}")]
    Validation(String),
    
    #[error("broadcast error: {0}")]
    Broadcast(String),
}

pub type SchedulerResult<T> = Result<T, SchedulerError>;

/// Type alias for scheduled entry
type ScheduledEntry = (ScheduledTransaction, Arc<dyn TransactionFilter>);

/// Transaction Filter
///
/// 对应 Java: Filter<ITransaction>
#[async_trait]
pub trait TransactionFilter: Send + Sync {
    async fn check(&self, transaction: &[u8]) -> bool;
}

/// Scheduled Transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTransaction {
    pub id: u64,
    pub sender_id: AccountId,
    pub transaction_data: Vec<u8>,
    pub expiration: i32,
    pub created_at: i32,
}

impl ScheduledTransaction {
    pub fn new(
        id: u64,
        sender_id: AccountId,
        transaction_data: Vec<u8>,
        expiration: i32,
        created_at: i32,
    ) -> Self {
        Self {
            id,
            sender_id,
            transaction_data,
            expiration,
            created_at,
        }
    }

    pub fn is_expired(&self, current_timestamp: i32) -> bool {
        self.expiration < current_timestamp
    }
}

/// Transaction Scheduler
///
/// 对应 Java: TransactionScheduler
pub struct TransactionScheduler {
    max_scheduled: usize,
    scheduled: Arc<RwLock<HashMap<u64, ScheduledEntry>>>,
}

impl Default for TransactionScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionScheduler {
    pub fn new() -> Self {
        Self {
            max_scheduled: 100,
            scheduled: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_max_scheduled(max_scheduled: usize) -> Self {
        Self {
            max_scheduled,
            scheduled: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn schedule(
        &self,
        transaction: ScheduledTransaction,
        filter: Arc<dyn TransactionFilter>,
    ) -> SchedulerResult<()> {
        let mut scheduled = self.scheduled.write().await;
        
        if scheduled.len() >= self.max_scheduled {
            return Err(SchedulerError::LimitExceeded(
                "Cannot schedule more than 100 transactions! Please restart your node if you want to clear existing scheduled transactions.".to_string()
            ));
        }
        
        scheduled.insert(transaction.id, (transaction, filter));
        Ok(())
    }

    pub async fn get_scheduled_transactions(&self, account_id: AccountId) -> Vec<ScheduledTransaction> {
        let scheduled = self.scheduled.read().await;
        scheduled
            .values()
            .filter(|(tx, _)| tx.sender_id == account_id)
            .map(|(tx, _)| tx.clone())
            .collect()
    }

    pub async fn get_all_scheduled(&self) -> Vec<ScheduledTransaction> {
        let scheduled = self.scheduled.read().await;
        scheduled.values().map(|(tx, _)| tx.clone()).collect()
    }

    pub async fn remove(&self, id: u64) -> Option<ScheduledTransaction> {
        let mut scheduled = self.scheduled.write().await;
        scheduled.remove(&id).map(|(tx, _)| tx)
    }

    pub async fn clear(&self) {
        let mut scheduled = self.scheduled.write().await;
        scheduled.clear();
    }

    pub async fn count(&self) -> usize {
        let scheduled = self.scheduled.read().await;
        scheduled.len()
    }

    pub async fn process_event(
        &self,
        unconfirmed_transaction: &[u8],
        current_timestamp: i32,
    ) -> Vec<u64> {
        let mut scheduled = self.scheduled.write().await;
        let mut removed = Vec::new();
        
        let ids: Vec<u64> = scheduled.keys().copied().collect();
        
        for id in ids {
            if let Some((tx, filter)) = scheduled.get(&id) {
                if tx.is_expired(current_timestamp) {
                    removed.push(id);
                    continue;
                }
                
                if filter.check(unconfirmed_transaction).await {
                    removed.push(id);
                }
            }
        }
        
        for id in &removed {
            scheduled.remove(id);
        }
        
        removed
    }

    pub async fn remove_expired(&self, current_timestamp: i32) -> Vec<u64> {
        let mut scheduled = self.scheduled.write().await;
        let expired: Vec<u64> = scheduled
            .iter()
            .filter(|(_, (tx, _))| tx.is_expired(current_timestamp))
            .map(|(id, _)| *id)
            .collect();
        
        for id in &expired {
            scheduled.remove(id);
        }
        
        expired
    }
}

/// Default Filter - always returns true
pub struct DefaultFilter;

#[async_trait]
impl TransactionFilter for DefaultFilter {
    async fn check(&self, _transaction: &[u8]) -> bool {
        true
    }
}

/// Sender Filter - checks if transaction is from specific sender
pub struct SenderFilter {
    pub sender_id: AccountId,
}

#[async_trait]
impl TransactionFilter for SenderFilter {
    async fn check(&self, transaction: &[u8]) -> bool {
        if transaction.len() < 8 {
            return false;
        }
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&transaction[0..8]);
        u64::from_le_bytes(buf) == self.sender_id
    }
}

/// Height Filter - checks if current height meets requirement
pub struct HeightFilter {
    pub required_height: i32,
    pub current_height: i32,
}

#[async_trait]
impl TransactionFilter for HeightFilter {
    async fn check(&self, _transaction: &[u8]) -> bool {
        self.current_height >= self.required_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_schedule_transaction() {
        let scheduler = TransactionScheduler::new();
        
        let tx = ScheduledTransaction::new(1, 100, vec![1, 2, 3], 1000, 0);
        let filter = Arc::new(DefaultFilter);
        
        scheduler.schedule(tx, filter).await.unwrap();
        assert_eq!(scheduler.count().await, 1);
    }

    #[tokio::test]
    async fn test_get_scheduled_transactions() {
        let scheduler = TransactionScheduler::new();
        
        let tx1 = ScheduledTransaction::new(1, 100, vec![1, 2, 3], 1000, 0);
        let tx2 = ScheduledTransaction::new(2, 200, vec![4, 5, 6], 1000, 0);
        
        scheduler.schedule(tx1, Arc::new(DefaultFilter)).await.unwrap();
        scheduler.schedule(tx2, Arc::new(DefaultFilter)).await.unwrap();
        
        let account_txs = scheduler.get_scheduled_transactions(100).await;
        assert_eq!(account_txs.len(), 1);
    }

    #[tokio::test]
    async fn test_remove_expired() {
        let scheduler = TransactionScheduler::new();
        
        let tx1 = ScheduledTransaction::new(1, 100, vec![1, 2, 3], 500, 0);
        let tx2 = ScheduledTransaction::new(2, 200, vec![4, 5, 6], 1500, 0);
        
        scheduler.schedule(tx1, Arc::new(DefaultFilter)).await.unwrap();
        scheduler.schedule(tx2, Arc::new(DefaultFilter)).await.unwrap();
        
        let expired = scheduler.remove_expired(1000).await;
        assert_eq!(expired.len(), 1);
        assert_eq!(scheduler.count().await, 1);
    }

    #[tokio::test]
    async fn test_limit_exceeded() {
        let scheduler = TransactionScheduler::with_max_scheduled(2);
        
        let tx1 = ScheduledTransaction::new(1, 100, vec![1, 2, 3], 1000, 0);
        let tx2 = ScheduledTransaction::new(2, 200, vec![4, 5, 6], 1000, 0);
        let tx3 = ScheduledTransaction::new(3, 300, vec![7, 8, 9], 1000, 0);
        
        scheduler.schedule(tx1, Arc::new(DefaultFilter)).await.unwrap();
        scheduler.schedule(tx2, Arc::new(DefaultFilter)).await.unwrap();
        
        let result = scheduler.schedule(tx3, Arc::new(DefaultFilter)).await;
        assert!(matches!(result, Err(SchedulerError::LimitExceeded(_))));
    }

    #[test]
    fn test_scheduled_transaction_is_expired() {
        let tx = ScheduledTransaction::new(1, 100, vec![1, 2, 3], 1000, 0);
        
        assert!(!tx.is_expired(500));
        assert!(tx.is_expired(1500));
    }

    #[tokio::test]
    async fn test_default_filter() {
        let filter = DefaultFilter;
        assert!(filter.check(&[1, 2, 3]).await);
    }

    #[tokio::test]
    async fn test_sender_filter() {
        let filter = SenderFilter { sender_id: 100 };
        
        let mut tx_data = vec![0u8; 100];
        tx_data[0..8].copy_from_slice(&100u64.to_le_bytes());
        
        assert!(filter.check(&tx_data).await);
        
        tx_data[0..8].copy_from_slice(&200u64.to_le_bytes());
        assert!(!filter.check(&tx_data).await);
    }
}
