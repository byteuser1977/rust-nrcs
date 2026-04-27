//! Hub Module
//!
//! 对应 Java: Hub.java
//!
//! 处理 Hub 节点公告和选择

use std::sync::RwLock;

/// Hub 节点
#[derive(Debug, Clone)]
pub struct Hub {
    pub account_id: u64,
    pub min_fee_per_byte: u64,
    pub uris: Vec<String>,
    pub height: i32,
}

impl Hub {
    pub fn new(account_id: u64, min_fee_per_byte: u64, uris: Vec<String>, height: i32) -> Self {
        Self {
            account_id,
            min_fee_per_byte,
            uris,
            height,
        }
    }

    pub fn get_account_id(&self) -> u64 {
        self.account_id
    }

    pub fn get_min_fee_per_byte(&self) -> u64 {
        self.min_fee_per_byte
    }

    pub fn get_uris(&self) -> &[String] {
        &self.uris
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }
}

/// Hub Hit（用于选择 Hub）
#[derive(Debug, Clone)]
pub struct HubHit {
    pub hub: Hub,
    pub hit_time: u64,
}

impl HubHit {
    pub fn new(hub: Hub, hit_time: u64) -> Self {
        Self { hub, hit_time }
    }

    pub fn compare(&self, other: &HubHit) -> std::cmp::Ordering {
        if self.hit_time < other.hit_time {
            std::cmp::Ordering::Less
        } else if self.hit_time > other.hit_time {
            std::cmp::Ordering::Greater
        } else {
            self.hub.account_id.cmp(&other.hub.account_id)
        }
    }
}

/// Hub 存储
pub struct HubStore {
    hubs: RwLock<Vec<Hub>>,
    last_block_id: RwLock<u64>,
    last_hits: RwLock<Vec<HubHit>>,
}

impl HubStore {
    pub fn new() -> Self {
        Self {
            hubs: RwLock::new(Vec::new()),
            last_block_id: RwLock::new(0),
            last_hits: RwLock::new(Vec::new()),
        }
    }

    pub fn add_or_update(&self, hub: Hub) {
        let mut hubs = self.hubs.write().unwrap();
        
        if let Some(existing) = hubs.iter_mut().find(|h| h.account_id == hub.account_id) {
            *existing = hub;
        } else {
            hubs.push(hub);
        }
    }

    pub fn get_all(&self) -> Vec<Hub> {
        self.hubs.read().unwrap().clone()
    }

    pub fn get_by_account(&self, account_id: u64) -> Option<Hub> {
        self.hubs
            .read()
            .unwrap()
            .iter()
            .find(|h| h.account_id == account_id)
            .cloned()
    }

    pub fn get_hub_hits(&self, block_id: u64, min_effective_balance: u64) -> Vec<HubHit> {
        let last_block_id = *self.last_block_id.read().unwrap();
        
        if block_id == last_block_id {
            return self.last_hits.read().unwrap().clone();
        }

        let mut hits: Vec<HubHit> = self.hubs
            .read()
            .unwrap()
            .iter()
            .map(|hub| {
                let hit_time = Self::calculate_hit_time(hub.account_id, min_effective_balance);
                HubHit::new(hub.clone(), hit_time)
            })
            .collect();

        hits.sort_by(|a, b| a.compare(b));

        *self.last_block_id.write().unwrap() = block_id;
        *self.last_hits.write().unwrap() = hits.clone();

        hits
    }

    fn calculate_hit_time(account_id: u64, effective_balance: u64) -> u64 {
        if effective_balance == 0 {
            return u64::MAX;
        }
        account_id.wrapping_mul(1000) / effective_balance
    }

    pub fn remove(&self, account_id: u64) {
        self.hubs.write().unwrap().retain(|h| h.account_id != account_id);
    }
}

impl Default for HubStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hub_creation() {
        let hub = Hub::new(12345, 100, vec!["http://hub.example.com".to_string()], 100);
        
        assert_eq!(hub.get_account_id(), 12345);
        assert_eq!(hub.get_min_fee_per_byte(), 100);
        assert_eq!(hub.get_uris().len(), 1);
    }

    #[test]
    fn test_hub_store() {
        let store = HubStore::new();
        
        store.add_or_update(Hub::new(1, 100, vec![], 10));
        store.add_or_update(Hub::new(2, 200, vec![], 20));

        assert_eq!(store.get_all().len(), 2);
        assert!(store.get_by_account(1).is_some());
    }
}
