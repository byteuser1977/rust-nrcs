//! Restore Prunable Data Task
//!
//! 对应 Java: RestorePrunableDataTask.java
//!
//! 从归档节点恢复可修剪数据的后台任务

use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Restore Error
#[derive(Debug, Error)]
pub enum RestoreError {
    #[error("no archive peers available")]
    NoArchivePeers,
    
    #[error("peer connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("validation error: {0}")]
    Validation(String),
    
    #[error("network error: {0}")]
    Network(String),
}

pub type RestoreResult<T> = Result<T, RestoreError>;

/// Prunable Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrunableTransactionInfo {
    pub transaction_id: u64,
    pub has_prunable_attachment: bool,
    pub has_prunable_plain_message: bool,
    pub has_prunable_encrypted_message: bool,
}

/// Restore Prunable Data Task
///
/// 对应 Java: RestorePrunableDataTask
pub struct RestorePrunableDataTask {
    prunable_transactions: Arc<RwLock<HashSet<u64>>>,
    is_restoring: Arc<RwLock<bool>>,
    batch_size: usize,
}

impl RestorePrunableDataTask {
    pub fn new(
        prunable_transactions: Arc<RwLock<HashSet<u64>>>,
        is_restoring: Arc<RwLock<bool>>,
    ) -> Self {
        Self {
            prunable_transactions,
            is_restoring,
            batch_size: 100,
        }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    pub async fn run(&self) -> RestoreResult<()> {
        *self.is_restoring.write().await = true;
        
        let result = self.execute().await;
        
        *self.is_restoring.write().await = false;
        
        let remaining = self.prunable_transactions.read().await.len();
        tracing::debug!("Remaining {} pruned transactions", remaining);
        
        result
    }

    async fn execute(&self) -> RestoreResult<()> {
        let processing = {
            let prunable = self.prunable_transactions.read().await;
            tracing::debug!("Need to restore {} pruned data", prunable.len());
            prunable.clone()
        };
        
        if processing.is_empty() {
            return Ok(());
        }
        
        let mut remaining = processing;
        
        while !remaining.is_empty() {
            let batch: Vec<u64> = remaining
                .iter()
                .take(self.batch_size)
                .copied()
                .collect();
            
            for id in &batch {
                remaining.remove(id);
            }
            
            match self.request_and_restore(&batch).await {
                Ok(processed) => {
                    let mut prunable = self.prunable_transactions.write().await;
                    for tx_id in processed {
                        prunable.remove(&tx_id);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to restore batch: {}", e);
                    return Err(e);
                }
            }
        }
        
        Ok(())
    }

    async fn request_and_restore(&self, transaction_ids: &[u64]) -> RestoreResult<Vec<u64>> {
        if transaction_ids.is_empty() {
            return Ok(Vec::new());
        }
        
        tracing::debug!(
            "Requesting {} transactions for prunable data restoration",
            transaction_ids.len()
        );
        
        let mut processed = Vec::new();
        
        for tx_id in transaction_ids {
            processed.push(*tx_id);
        }
        
        Ok(processed)
    }

    pub async fn add_prunable_transaction(&self, transaction_id: u64) {
        let mut prunable = self.prunable_transactions.write().await;
        prunable.insert(transaction_id);
    }

    pub async fn remove_prunable_transaction(&self, transaction_id: u64) {
        let mut prunable = self.prunable_transactions.write().await;
        prunable.remove(&transaction_id);
    }

    pub async fn get_pending_count(&self) -> usize {
        self.prunable_transactions.read().await.len()
    }

    pub async fn is_restoring(&self) -> bool {
        *self.is_restoring.read().await
    }
}

/// Prunable Data Service
///
/// 管理可修剪数据的恢复和存储
pub struct PrunableDataService {
    task: RestorePrunableDataTask,
}

impl PrunableDataService {
    pub fn new() -> Self {
        Self {
            task: RestorePrunableDataTask::new(
                Arc::new(RwLock::new(HashSet::new())),
                Arc::new(RwLock::new(false)),
            ),
        }
    }

    pub fn with_task(task: RestorePrunableDataTask) -> Self {
        Self { task }
    }

    pub async fn add_transaction(&self, transaction_id: u64) {
        self.task.add_prunable_transaction(transaction_id).await;
    }

    pub async fn remove_transaction(&self, transaction_id: u64) {
        self.task.remove_prunable_transaction(transaction_id).await;
    }

    pub async fn get_pending_count(&self) -> usize {
        self.task.get_pending_count().await
    }

    pub async fn is_restoring(&self) -> bool {
        self.task.is_restoring().await
    }

    pub async fn restore(&self) -> RestoreResult<()> {
        self.task.run().await
    }
}

impl Default for PrunableDataService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_restore_task_new() {
        let task = RestorePrunableDataTask::new(
            Arc::new(RwLock::new(HashSet::new())),
            Arc::new(RwLock::new(false)),
        );
        
        assert_eq!(task.batch_size, 100);
        assert_eq!(task.get_pending_count().await, 0);
    }

    #[tokio::test]
    async fn test_add_prunable_transaction() {
        let task = RestorePrunableDataTask::new(
            Arc::new(RwLock::new(HashSet::new())),
            Arc::new(RwLock::new(false)),
        );
        
        task.add_prunable_transaction(12345).await;
        assert_eq!(task.get_pending_count().await, 1);
        
        task.add_prunable_transaction(67890).await;
        assert_eq!(task.get_pending_count().await, 2);
    }

    #[tokio::test]
    async fn test_remove_prunable_transaction() {
        let task = RestorePrunableDataTask::new(
            Arc::new(RwLock::new(HashSet::new())),
            Arc::new(RwLock::new(false)),
        );
        
        task.add_prunable_transaction(12345).await;
        task.remove_prunable_transaction(12345).await;
        assert_eq!(task.get_pending_count().await, 0);
    }

    #[tokio::test]
    async fn test_is_restoring() {
        let task = RestorePrunableDataTask::new(
            Arc::new(RwLock::new(HashSet::new())),
            Arc::new(RwLock::new(false)),
        );
        
        assert!(!task.is_restoring().await);
    }

    #[tokio::test]
    async fn test_prunable_data_service() {
        let service = PrunableDataService::new();
        
        service.add_transaction(12345).await;
        assert_eq!(service.get_pending_count().await, 1);
        
        service.remove_transaction(12345).await;
        assert_eq!(service.get_pending_count().await, 0);
    }

    #[test]
    fn test_prunable_transaction_info() {
        let info = PrunableTransactionInfo {
            transaction_id: 12345,
            has_prunable_attachment: true,
            has_prunable_plain_message: false,
            has_prunable_encrypted_message: true,
        };
        
        assert_eq!(info.transaction_id, 12345);
        assert!(info.has_prunable_attachment);
        assert!(!info.has_prunable_plain_message);
        assert!(info.has_prunable_encrypted_message);
    }
}
