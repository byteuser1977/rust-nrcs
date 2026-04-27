//! Referenced Transaction Module
//!
//! 对应 Java: ReferencedTransaction.java
//!
//! 处理引用的交易（用于 Phasing 等功能）

use blockchain_types::prelude::Hash256;

/// 引用交易
#[derive(Debug, Clone)]
pub struct ReferencedTransaction {
    pub transaction_id: u64,
    pub referenced_transaction_id: u64,
    pub referenced_transaction_full_hash: Hash256,
    pub height: i32,
    pub latest: bool,
}

impl ReferencedTransaction {
    pub fn new(
        transaction_id: u64,
        referenced_transaction_id: u64,
        referenced_transaction_full_hash: Hash256,
        height: i32,
    ) -> Self {
        Self {
            transaction_id,
            referenced_transaction_id,
            referenced_transaction_full_hash,
            height,
            latest: true,
        }
    }

    pub fn get_referenced_transaction_id(&self) -> u64 {
        self.referenced_transaction_id
    }

    pub fn get_referenced_transaction_full_hash(&self) -> &Hash256 {
        &self.referenced_transaction_full_hash
    }

    pub fn verify_hash(&self, expected_hash: &Hash256) -> bool {
        &self.referenced_transaction_full_hash == expected_hash
    }
}

/// 引用交易存储
pub struct ReferencedTransactionStore {
    transactions: Vec<ReferencedTransaction>,
}

impl ReferencedTransactionStore {
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
        }
    }

    pub fn add(&mut self, rt: ReferencedTransaction) {
        self.transactions.push(rt);
    }

    pub fn get_by_transaction_id(&self, transaction_id: u64) -> Option<&ReferencedTransaction> {
        self.transactions
            .iter()
            .find(|rt| rt.transaction_id == transaction_id && rt.latest)
    }

    pub fn get_all_referenced(&self, transaction_id: u64) -> Vec<&ReferencedTransaction> {
        self.transactions
            .iter()
            .filter(|rt| rt.transaction_id == transaction_id)
            .collect()
    }

    pub fn remove(&mut self, transaction_id: u64) {
        self.transactions.retain(|rt| rt.transaction_id != transaction_id);
    }
}

impl Default for ReferencedTransactionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_referenced_transaction_creation() {
        let hash = Hash256([1u8; 32]);
        let rt = ReferencedTransaction::new(12345, 67890, hash, 100);

        assert_eq!(rt.get_referenced_transaction_id(), 67890);
        assert!(rt.verify_hash(&Hash256([1u8; 32])));
    }

    #[test]
    fn test_referenced_transaction_store() {
        let mut store = ReferencedTransactionStore::new();
        let hash = Hash256([1u8; 32]);
        
        store.add(ReferencedTransaction::new(1, 100, hash, 10));
        store.add(ReferencedTransaction::new(2, 200, hash, 20));

        assert!(store.get_by_transaction_id(1).is_some());
        assert!(store.get_by_transaction_id(2).is_some());
        assert!(store.get_by_transaction_id(3).is_none());
    }
}
