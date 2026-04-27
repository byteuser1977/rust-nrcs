//! 锻造者选择模块
//!
//! 对应 Java: Generator.getNextGenerators() 和 ActiveGenerator
//!
//! 负责:
//! - 选择下一个区块的锻造者
//! - 管理活跃锻造者列表
//! - 计算锻造者排序

use blockchain_types::*;
use blockchain_types::prelude::Block;

use crate::target::{calculate_hit, calculate_deadline, verify_hit};

#[derive(Debug, Clone)]
pub struct ForgerInfo {
    pub account_id: AccountId,
    pub hit_time: u64,
    pub effective_balance: u64,
    pub deadline: u64,
}

impl ForgerInfo {
    pub fn new(account_id: AccountId, hit_time: u64, effective_balance: u64, deadline: u64) -> Self {
        Self {
            account_id,
            hit_time,
            effective_balance,
            deadline,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ForgerSelection {
    pub forger_id: AccountId,
    pub hit_time: u64,
    pub deadline: u64,
    pub effective_balance: u64,
    pub generation_signature: Vec<u8>,
}

impl ForgerSelection {
    pub fn new(
        forger_id: AccountId,
        hit_time: u64,
        deadline: u64,
        effective_balance: u64,
        generation_signature: Vec<u8>,
    ) -> Self {
        Self {
            forger_id,
            hit_time,
            deadline,
            effective_balance,
            generation_signature,
        }
    }
}

pub struct ForgerSelector {
    min_forging_balance: u64,
    #[allow(dead_code)]
    max_forgers: usize,
}

impl ForgerSelector {
    pub fn new() -> Self {
        Self {
            min_forging_balance: MIN_FORGING_BALANCE_NQT,
            max_forgers: 100,
        }
    }
    
    pub fn with_params(min_balance: u64, max_forgers: usize) -> Self {
        Self {
            min_forging_balance: min_balance,
            max_forgers,
        }
    }
    
    pub fn select_forger(
        &self,
        block: &Block,
        candidates: &[ForgerCandidate],
    ) -> Option<ForgerSelection> {
        let valid_candidates: Vec<_> = candidates.iter()
            .filter(|c| c.effective_balance >= self.min_forging_balance)
            .collect();
        
        if valid_candidates.is_empty() {
            return None;
        }
        
        let mut forgers: Vec<_> = valid_candidates.into_iter()
            .map(|c| {
                let hit = calculate_hit(&c.public_key, &block.generation_signature);
                let deadline = calculate_deadline(&hit, c.effective_balance, block.base_target);
                let hit_time = block.timestamp as u64 + deadline;
                
                ForgerInfo::new(c.account_id, hit_time, c.effective_balance, deadline)
            })
            .collect();
        
        forgers.sort_by_key(|f| f.hit_time);
        
        forgers.into_iter().next().map(|f| {
            ForgerSelection::new(
                f.account_id,
                f.hit_time,
                f.deadline,
                f.effective_balance,
                block.generation_signature.clone(),
            )
        })
    }
    
    pub fn get_next_forgers(
        &self,
        block: &Block,
        candidates: &[ForgerCandidate],
        count: usize,
    ) -> Vec<ForgerInfo> {
        let valid_candidates: Vec<_> = candidates.iter()
            .filter(|c| c.effective_balance >= self.min_forging_balance)
            .collect();
        
        if valid_candidates.is_empty() {
            return Vec::new();
        }
        
        let mut forgers: Vec<_> = valid_candidates.into_iter()
            .map(|c| {
                let hit = calculate_hit(&c.public_key, &block.generation_signature);
                let deadline = calculate_deadline(&hit, c.effective_balance, block.base_target);
                let hit_time = block.timestamp as u64 + deadline;
                
                ForgerInfo::new(c.account_id, hit_time, c.effective_balance, deadline)
            })
            .collect();
        
        forgers.sort_by_key(|f| f.hit_time);
        forgers.into_iter().take(count).collect()
    }
    
    pub fn verify_forger(
        &self,
        block: &Block,
        _forger_id: AccountId,
        public_key: &[u8],
        effective_balance: u64,
        timestamp: u32,
    ) -> bool {
        if effective_balance < self.min_forging_balance {
            return false;
        }
        
        let hit = calculate_hit(public_key, &block.generation_signature);
        let elapsed_time = timestamp as u64 - block.timestamp as u64;
        
        if elapsed_time == 0 {
            return false;
        }
        
        verify_hit(&hit, effective_balance, block.base_target, elapsed_time)
    }
    
    pub fn calculate_generation_deadline(
        &self,
        block: &Block,
        public_key: &[u8],
        effective_balance: u64,
    ) -> u64 {
        if effective_balance < self.min_forging_balance {
            return u64::MAX;
        }
        
        let hit = calculate_hit(public_key, &block.generation_signature);
        calculate_deadline(&hit, effective_balance, block.base_target)
    }
}

impl Default for ForgerSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ForgerCandidate {
    pub account_id: AccountId,
    pub public_key: Vec<u8>,
    pub effective_balance: u64,
}

impl ForgerCandidate {
    pub fn new(account_id: AccountId, public_key: Vec<u8>, effective_balance: u64) -> Self {
        Self {
            account_id,
            public_key,
            effective_balance,
        }
    }
}

pub struct ActiveGeneratorList {
    generators: Vec<ActiveGeneratorEntry>,
    initialized: bool,
    last_block_id: u64,
}

#[derive(Debug, Clone)]
struct ActiveGeneratorEntry {
    account_id: AccountId,
    hit_time: u64,
    effective_balance: u64,
    public_key: Option<Vec<u8>>,
}

impl ActiveGeneratorList {
    pub fn new() -> Self {
        Self {
            generators: Vec::new(),
            initialized: false,
            last_block_id: 0,
        }
    }
    
    pub fn initialize(&mut self, generator_ids: &[AccountId]) {
        if self.initialized {
            return;
        }
        
        self.generators = generator_ids.iter()
            .map(|&id| ActiveGeneratorEntry {
                account_id: id,
                hit_time: u64::MAX,
                effective_balance: 0,
                public_key: None,
            })
            .collect();
        
        self.initialized = true;
    }
    
    pub fn add_generator(&mut self, account_id: AccountId) {
        if !self.generators.iter().any(|g| g.account_id == account_id) {
            self.generators.push(ActiveGeneratorEntry {
                account_id,
                hit_time: u64::MAX,
                effective_balance: 0,
                public_key: None,
            });
        }
    }
    
    pub fn update_for_block(
        &mut self,
        block: &Block,
        get_balance: impl Fn(AccountId) -> (u64, Option<Vec<u8>>),
    ) {
        if block.height as u64 == self.last_block_id {
            return;
        }
        
        self.last_block_id = block.height as u64;
        
        for entry in &mut self.generators {
            let (balance, pk) = get_balance(entry.account_id);
            entry.effective_balance = balance;
            entry.public_key = pk;
            
            if entry.public_key.is_none() || entry.effective_balance == 0 {
                entry.hit_time = u64::MAX;
                continue;
            }
            
            if let Some(pk) = &entry.public_key {
                let hit = calculate_hit(pk, &block.generation_signature);
                let deadline = calculate_deadline(&hit, entry.effective_balance, block.base_target);
                entry.hit_time = block.timestamp as u64 + deadline;
            }
        }
        
        self.generators.sort_by_key(|g| g.hit_time);
    }
    
    pub fn get_sorted_generators(&self) -> Vec<ForgerInfo> {
        self.generators.iter()
            .filter(|g| g.hit_time < u64::MAX)
            .map(|g| ForgerInfo::new(
                g.account_id,
                g.hit_time,
                g.effective_balance,
                0,
            ))
            .collect()
    }
    
    pub fn get_next_hit_time(&self, current_time: u64, delay: u64) -> u64 {
        self.generators.iter()
            .filter(|g| g.hit_time >= current_time - delay)
            .map(|g| g.hit_time)
            .next()
            .unwrap_or(0)
    }
    
    pub fn len(&self) -> usize {
        self.generators.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.generators.is_empty()
    }
}

impl Default for ActiveGeneratorList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forger_selector_creation() {
        let selector = ForgerSelector::new();
        assert!(selector.min_forging_balance > 0);
    }

    #[test]
    fn test_forger_candidate() {
        let candidate = ForgerCandidate::new(123, vec![1, 2, 3], 1000);
        assert_eq!(candidate.account_id, 123);
        assert_eq!(candidate.effective_balance, 1000);
    }

    #[test]
    fn test_active_generator_list() {
        let mut list = ActiveGeneratorList::new();
        
        list.initialize(&[1, 2, 3]);
        assert_eq!(list.len(), 3);
        
        list.add_generator(4);
        assert_eq!(list.len(), 4);
        
        list.add_generator(1);
        assert_eq!(list.len(), 4);
    }

    #[test]
    fn test_forger_info_sorting() {
        let mut forgers = vec![
            ForgerInfo::new(1, 100, 1000, 0),
            ForgerInfo::new(2, 50, 1000, 0),
            ForgerInfo::new(3, 200, 1000, 0),
        ];
        
        forgers.sort_by_key(|f| f.hit_time);
        
        assert_eq!(forgers[0].account_id, 2);
        assert_eq!(forgers[1].account_id, 1);
        assert_eq!(forgers[2].account_id, 3);
    }
}
