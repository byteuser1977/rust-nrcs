//! 交易广播模块
//!
//! 对应 Java: TransactionProcessor 中的交易广播和重广播逻辑
//!
//! 负责:
//! - 广播交易到网络
//! - 重广播未确认交易
//! - 跟踪已广播交易

use blockchain_types::*;
use blockchain_types::prelude::Transaction;
use std::collections::HashSet;
use parking_lot::RwLock;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BroadcastError {
    #[error("no peers available")]
    NoPeersAvailable,
    
    #[error("broadcast failed: {0}")]
    BroadcastFailed(String),
    
    #[error("transaction expired")]
    TransactionExpired,
    
    #[error("transaction already confirmed")]
    AlreadyConfirmed,
}

pub type BroadcastResult<T> = std::result::Result<T, BroadcastError>;

#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    pub enable_rebroadcast: bool,
    pub rebroadcast_interval_secs: u64,
    pub max_rebroadcast_count: u32,
    pub broadcast_to_all_peers: bool,
    pub min_peers_for_broadcast: usize,
}

impl Default for BroadcastConfig {
    fn default() -> Self {
        Self {
            enable_rebroadcast: true,
            rebroadcast_interval_secs: 30,
            max_rebroadcast_count: 10,
            broadcast_to_all_peers: false,
            min_peers_for_broadcast: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BroadcastedTx {
    pub tx_hash: Hash256,
    pub timestamp: u32,
    pub broadcast_count: u32,
    pub last_broadcast: u32,
    pub expiration: u32,
}

impl BroadcastedTx {
    pub fn new(tx: &Transaction, _current_time: u32) -> Self {
        Self {
            tx_hash: tx.full_hash,
            timestamp: tx.timestamp,
            broadcast_count: 0,
            last_broadcast: 0,
            expiration: tx.deadline as u32,
        }
    }
    
    pub fn should_rebroadcast(&self, current_time: u32, config: &BroadcastConfig) -> bool {
        if !config.enable_rebroadcast {
            return false;
        }
        
        if self.broadcast_count >= config.max_rebroadcast_count {
            return false;
        }
        
        if self.expiration > 0 && current_time > self.expiration {
            return false;
        }
        
        let time_since_last = current_time.saturating_sub(self.last_broadcast);
        time_since_last >= config.rebroadcast_interval_secs as u32
    }
    
    pub fn mark_broadcast(&mut self, current_time: u32) {
        self.broadcast_count += 1;
        self.last_broadcast = current_time;
    }
}

pub struct TxBroadcaster {
    config: BroadcastConfig,
    broadcasted: RwLock<HashSet<Hash256>>,
    pending_rebroadcast: RwLock<Vec<BroadcastedTx>>,
}

impl TxBroadcaster {
    pub fn new(config: BroadcastConfig) -> Self {
        Self {
            config,
            broadcasted: RwLock::new(HashSet::new()),
            pending_rebroadcast: RwLock::new(Vec::new()),
        }
    }
    
    pub fn broadcast(&self, tx: &Transaction, current_time: u32) -> BroadcastResult<()> {
        let tx_hash = tx.full_hash;
        
        {
            let broadcasted = self.broadcasted.read();
            if broadcasted.contains(&tx_hash) {
                return Ok(());
            }
        }
        
        // TODO: 实际的网络广播逻辑
        // let peers = self.get_connected_peers()?;
        // if peers.len() < self.config.min_peers_for_broadcast {
        //     return Err(BroadcastError::NoPeersAvailable);
        // }
        // 
        // for peer in peers {
        //     peer.send_transaction(tx)?;
        // }
        
        {
            let mut broadcasted = self.broadcasted.write();
            broadcasted.insert(tx_hash);
        }
        
        let mut pending = self.pending_rebroadcast.write();
        let mut broadcasted_tx = BroadcastedTx::new(tx, current_time);
        broadcasted_tx.mark_broadcast(current_time);
        pending.push(broadcasted_tx);
        
        Ok(())
    }
    
    pub fn broadcast_batch(&self, txs: &[Transaction], current_time: u32) -> BroadcastResult<usize> {
        let mut count = 0;
        for tx in txs {
            match self.broadcast(tx, current_time) {
                Ok(()) => count += 1,
                Err(_) => continue,
            }
        }
        Ok(count)
    }
    
    pub fn rebroadcast_pending(&self, current_time: u32) -> BroadcastResult<Vec<Hash256>> {
        if !self.config.enable_rebroadcast {
            return Ok(Vec::new());
        }
        
        let mut rebroadcasted = Vec::new();
        let mut pending = self.pending_rebroadcast.write();
        
        pending.retain(|tx| {
            if tx.should_rebroadcast(current_time, &self.config) {
                rebroadcasted.push(tx.tx_hash);
                true
            } else {
                !(tx.expiration > 0 && current_time > tx.expiration)
            }
        });
        
        Ok(rebroadcasted)
    }
    
    pub fn remove_confirmed(&self, tx_hash: &Hash256) {
        {
            let mut broadcasted = self.broadcasted.write();
            broadcasted.remove(tx_hash);
        }
        
        {
            let mut pending = self.pending_rebroadcast.write();
            pending.retain(|tx| &tx.tx_hash != tx_hash);
        }
    }
    
    pub fn remove_expired(&self, current_time: u32) -> Vec<Hash256> {
        let mut expired = Vec::new();
        
        {
            let mut broadcasted = self.broadcasted.write();
            broadcasted.retain(|hash| {
                let pending = self.pending_rebroadcast.read();
                let found = pending.iter().find(|tx| &tx.tx_hash == hash);
                match found {
                    Some(tx) => tx.expiration == 0 || current_time <= tx.expiration,
                    None => true,
                }
            });
        }
        
        {
            let mut pending = self.pending_rebroadcast.write();
            pending.retain(|tx| {
                if tx.expiration > 0 && current_time > tx.expiration {
                    expired.push(tx.tx_hash);
                    false
                } else {
                    true
                }
            });
        }
        
        expired
    }
    
    pub fn is_broadcasted(&self, tx_hash: &Hash256) -> bool {
        self.broadcasted.read().contains(tx_hash)
    }
    
    pub fn get_pending_count(&self) -> usize {
        self.pending_rebroadcast.read().len()
    }
    
    pub fn get_broadcasted_count(&self) -> usize {
        self.broadcasted.read().len()
    }
    
    pub fn clear(&self) {
        self.broadcasted.write().clear();
        self.pending_rebroadcast.write().clear();
    }
}

impl Default for TxBroadcaster {
    fn default() -> Self {
        Self::new(BroadcastConfig::default())
    }
}

pub struct TxConfirmationTracker {
    confirmed: RwLock<HashSet<Hash256>>,
}

impl TxConfirmationTracker {
    pub fn new() -> Self {
        Self {
            confirmed: RwLock::new(HashSet::new()),
        }
    }
    
    pub fn mark_confirmed(&self, tx_hash: Hash256) {
        self.confirmed.write().insert(tx_hash);
    }
    
    pub fn is_confirmed(&self, tx_hash: &Hash256) -> bool {
        self.confirmed.read().contains(tx_hash)
    }
    
    pub fn remove(&self, tx_hash: &Hash256) {
        self.confirmed.write().remove(tx_hash);
    }
    
    pub fn clear(&self) {
        self.confirmed.write().clear();
    }
}

impl Default for TxConfirmationTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_transaction() -> Transaction {
        Transaction {
            id: 0,
            version: TRANSACTION_VERSION,
            type_id: TransactionType::Payment,
            subtype: 0,
            timestamp: 1000,
            deadline: 2000,
            sender_public_key: Hash256([0u8; 32]),
            sender_id: 123,
            recipient_id: Some(456),
            amount: 1000,
            fee: 10,
            height: 0,
            block_id: 0,
            block_timestamp: 0,
            transaction_index: 0,
            signature: Signature([0u8; 64]),
            full_hash: Hash256([1u8; 32]),
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
        assert!(config.rebroadcast_interval_secs > 0);
    }

    #[test]
    fn test_broadcasted_tx_creation() {
        let tx = create_test_transaction();
        let broadcasted = BroadcastedTx::new(&tx, 1000);
        
        assert_eq!(broadcasted.tx_hash, tx.full_hash);
        assert_eq!(broadcasted.broadcast_count, 0);
    }

    #[test]
    fn test_broadcasted_tx_rebroadcast_check() {
        let tx = create_test_transaction();
        let config = BroadcastConfig::default();
        
        let mut broadcasted = BroadcastedTx::new(&tx, 1000);
        broadcasted.mark_broadcast(1000);
        
        assert!(!broadcasted.should_rebroadcast(1000, &config));
        assert!(broadcasted.should_rebroadcast(1100, &config));
    }

    #[test]
    fn test_tx_broadcaster_creation() {
        let broadcaster = TxBroadcaster::default();
        assert_eq!(broadcaster.get_broadcasted_count(), 0);
    }

    #[test]
    fn test_tx_broadcaster_broadcast() {
        let broadcaster = TxBroadcaster::default();
        let tx = create_test_transaction();
        
        let result = broadcaster.broadcast(&tx, 1000);
        assert!(result.is_ok());
        assert!(broadcaster.is_broadcasted(&tx.full_hash));
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
}
