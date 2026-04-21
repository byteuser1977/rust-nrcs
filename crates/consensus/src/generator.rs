//! Generator 模块 - 锻造者管理
//!
//! 对应 Java: com.bytechain.nrcs.service.generator.Generator
//!
//! 负责管理锻造者账户、计算hit time、deadline等

use blockchain_types::*;
use blockchain_types::prelude::Block;
use blockchain_types::constants::*;
use num_bigint::BigUint;
use num_traits::{Zero, ToPrimitive};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use parking_lot::RwLock;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("max forgers exceeded: {0}")]
    MaxForgersExceeded(usize),
    
    #[error("invalid secret phrase")]
    InvalidSecretPhrase,
    
    #[error("effective balance is zero")]
    ZeroEffectiveBalance,
    
    #[error("generator not found")]
    NotFound,
}

pub type GeneratorResult<T> = std::result::Result<T, GeneratorError>;

#[derive(Debug, Clone)]
pub struct GeneratorInfo {
    pub account_id: AccountId,
    pub public_key: Vec<u8>,
    pub hit_time: u64,
    pub hit: BigUint,
    pub effective_balance: BigUint,
    pub deadline: u64,
}

impl GeneratorInfo {
    pub fn new(account_id: AccountId, public_key: Vec<u8>) -> Self {
        Self {
            account_id,
            public_key,
            hit_time: 0,
            hit: BigUint::ZERO,
            effective_balance: BigUint::ZERO,
            deadline: 0,
        }
    }
}

pub struct Generator {
    account_id: AccountId,
    secret_phrase: String,
    public_key: Vec<u8>,
    hit_time: u64,
    hit: BigUint,
    effective_balance: BigUint,
    deadline: u64,
}

impl Generator {
    pub fn new(secret_phrase: &str) -> GeneratorResult<Self> {
        let public_key = Self::derive_public_key(secret_phrase)?;
        let account_id = Self::derive_account_id(&public_key);
        
        Ok(Self {
            account_id,
            secret_phrase: secret_phrase.to_string(),
            public_key,
            hit_time: 0,
            hit: BigUint::ZERO,
            effective_balance: BigUint::ZERO,
            deadline: 0,
        })
    }
    
    fn derive_public_key(secret_phrase: &str) -> GeneratorResult<Vec<u8>> {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(secret_phrase.as_bytes());
        let hash = hasher.finalize();
        Ok(hash.to_vec())
    }
    
    fn derive_account_id(public_key: &[u8]) -> AccountId {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        let hash = hasher.finalize();
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&hash[..8]);
        u64::from_le_bytes(arr)
    }
    
    pub fn get_account_id(&self) -> AccountId {
        self.account_id
    }
    
    pub fn get_public_key(&self) -> &[u8] {
        &self.public_key
    }
    
    pub fn get_hit_time(&self) -> u64 {
        self.hit_time
    }
    
    pub fn get_deadline(&self) -> u64 {
        self.deadline
    }
    
    pub fn get_effective_balance(&self) -> &BigUint {
        &self.effective_balance
    }
    
    pub fn set_last_block(&mut self, last_block: &Block, effective_balance: u64) {
        self.effective_balance = BigUint::from(effective_balance);
        
        if effective_balance == 0 {
            self.hit_time = 0;
            self.hit = BigUint::ZERO;
            return;
        }
        
        self.hit = Self::calculate_hit(&self.public_key, last_block);
        self.hit_time = Self::calculate_hit_time(&self.effective_balance, &self.hit, last_block);
        self.deadline = self.hit_time.saturating_sub(last_block.timestamp as u64);
    }
    
    pub fn calculate_hit(public_key: &[u8], block: &Block) -> BigUint {
        let mut hasher = Sha256::new();
        hasher.update(&block.generation_signature.0);
        hasher.update(public_key);
        let hash = hasher.finalize();
        
        let mut hit_bytes = [0u8; 8];
        hit_bytes.copy_from_slice(&hash[..8]);
        hit_bytes.reverse();
        
        BigUint::from(u64::from_be_bytes(hit_bytes))
    }
    
    pub fn calculate_hit_time(effective_balance: &BigUint, hit: &BigUint, block: &Block) -> u64 {
        if effective_balance.is_zero() {
            return 0;
        }
        
        let base_target = BigUint::from(block.base_target);
        let divisor = base_target * effective_balance;
        
        if divisor.is_zero() {
            return 0;
        }
        
        block.timestamp as u64 + (hit / divisor).to_u64().unwrap_or(0)
    }
    
    pub fn verify_hit(&self, block: &Block, timestamp: u32) -> bool {
        let elapsed_time = timestamp as i64 - block.timestamp as i64;
        if elapsed_time <= 0 {
            return false;
        }
        
        let effective_base_target = BigUint::from(block.base_target) * &self.effective_balance;
        let prev_target = &effective_base_target * BigUint::from(elapsed_time as u64 - 1);
        let target = &prev_target + &effective_base_target;
        
        self.hit < target && (self.hit >= prev_target || elapsed_time as u64 > 600)
    }
    
    pub fn get_timestamp(&self, generation_limit: u64) -> u32 {
        if generation_limit > self.hit_time + 3600 {
            generation_limit as u32
        } else {
            self.hit_time as u32 + 1
        }
    }
}

impl PartialEq for Generator {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
    }
}

impl Eq for Generator {}

impl PartialOrd for Generator {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Generator {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_product = &self.hit * &other.effective_balance;
        let other_product = &other.hit * &self.effective_balance;
        
        match self_product.cmp(&other_product) {
            std::cmp::Ordering::Equal => self.account_id.cmp(&other.account_id),
            ord => ord,
        }
    }
}

pub struct ActiveGenerator {
    account_id: AccountId,
    hit_time: u64,
    effective_balance: u64,
    public_key: Option<Vec<u8>>,
}

impl ActiveGenerator {
    pub fn new(account_id: AccountId) -> Self {
        Self {
            account_id,
            hit_time: u64::MAX,
            effective_balance: 0,
            public_key: None,
        }
    }
    
    pub fn get_account_id(&self) -> AccountId {
        self.account_id
    }
    
    pub fn get_hit_time(&self) -> u64 {
        self.hit_time
    }
    
    pub fn get_effective_balance(&self) -> u64 {
        self.effective_balance
    }
    
    pub fn set_last_block(&mut self, block: &Block, public_key: Option<Vec<u8>>, effective_balance: u64) {
        self.public_key = public_key;
        self.effective_balance = effective_balance;
        
        if self.public_key.is_none() || effective_balance == 0 {
            self.hit_time = u64::MAX;
            return;
        }
        
        if let Some(pk) = &self.public_key {
            let hit = Generator::calculate_hit(pk, block);
            let effective_balance_big = BigUint::from(effective_balance);
            self.hit_time = Generator::calculate_hit_time(&effective_balance_big, &hit, block);
        }
    }
}

impl PartialEq for ActiveGenerator {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
    }
}

impl Eq for ActiveGenerator {}

impl PartialOrd for ActiveGenerator {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ActiveGenerator {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hit_time.cmp(&other.hit_time)
    }
}

pub struct GeneratorRegistry {
    generators: RwLock<HashMap<String, Generator>>,
    max_forgers: usize,
}

impl GeneratorRegistry {
    pub fn new() -> Self {
        Self {
            generators: RwLock::new(HashMap::new()),
            max_forgers: 100,
        }
    }
    
    pub fn with_max_forgers(max_forgers: usize) -> Self {
        Self {
            generators: RwLock::new(HashMap::new()),
            max_forgers,
        }
    }
    
    pub fn start_forging(&self, secret_phrase: &str) -> GeneratorResult<()> {
        let mut generators = self.generators.write();
        
        if generators.len() >= self.max_forgers {
            return Err(GeneratorError::MaxForgersExceeded(self.max_forgers));
        }
        
        let generator = Generator::new(secret_phrase)?;
        generators.insert(secret_phrase.to_string(), generator);
        
        Ok(())
    }
    
    pub fn stop_forging(&self, secret_phrase: &str) -> GeneratorResult<()> {
        let mut generators = self.generators.write();
        generators.remove(secret_phrase)
            .ok_or(GeneratorError::NotFound)?;
        Ok(())
    }
    
    pub fn stop_all_forging(&self) -> usize {
        let mut generators = self.generators.write();
        let count = generators.len();
        generators.clear();
        count
    }
    
    pub fn get_generator(&self, secret_phrase: &str) -> Option<GeneratorInfo> {
        let generators = self.generators.read();
        generators.get(secret_phrase).map(|g| GeneratorInfo {
            account_id: g.account_id,
            public_key: g.public_key.clone(),
            hit_time: g.hit_time,
            hit: g.hit.clone(),
            effective_balance: g.effective_balance.clone(),
            deadline: g.deadline,
        })
    }
    
    pub fn get_all_generators(&self) -> Vec<GeneratorInfo> {
        let generators = self.generators.read();
        generators.values().map(|g| GeneratorInfo {
            account_id: g.account_id,
            public_key: g.public_key.clone(),
            hit_time: g.hit_time,
            hit: g.hit.clone(),
            effective_balance: g.effective_balance.clone(),
            deadline: g.deadline,
        }).collect()
    }
    
    pub fn get_generator_count(&self) -> usize {
        self.generators.read().len()
    }
    
    pub fn get_sorted_forgers(&self) -> Vec<GeneratorInfo> {
        let generators = self.generators.read();
        let mut forgers: Vec<_> = generators.values()
            .filter(|g| !g.effective_balance.is_zero())
            .collect();
        forgers.sort();
        
        forgers.into_iter().map(|g| GeneratorInfo {
            account_id: g.account_id,
            public_key: g.public_key.clone(),
            hit_time: g.hit_time,
            hit: g.hit.clone(),
            effective_balance: g.effective_balance.clone(),
            deadline: g.deadline,
        }).collect()
    }
    
    pub fn update_generator(&self, secret_phrase: &str, block: &Block, effective_balance: u64) -> GeneratorResult<()> {
        let mut generators = self.generators.write();
        let generator = generators.get_mut(secret_phrase)
            .ok_or(GeneratorError::NotFound)?;
        generator.set_last_block(block, effective_balance);
        Ok(())
    }
}

impl Default for GeneratorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_creation() {
        let generator = Generator::new("test_secret_phrase").unwrap();
        assert!(generator.get_account_id() > 0);
        assert!(!generator.get_public_key().is_empty());
    }

    #[test]
    fn test_generator_registry() {
        let registry = GeneratorRegistry::new();
        
        registry.start_forging("secret1").unwrap();
        registry.start_forging("secret2").unwrap();
        
        assert_eq!(registry.get_generator_count(), 2);
        
        registry.stop_forging("secret1").unwrap();
        assert_eq!(registry.get_generator_count(), 1);
        
        let count = registry.stop_all_forging();
        assert_eq!(count, 1);
        assert_eq!(registry.get_generator_count(), 0);
    }

    #[test]
    fn test_max_forgers_limit() {
        let registry = GeneratorRegistry::with_max_forgers(2);
        
        registry.start_forging("secret1").unwrap();
        registry.start_forging("secret2").unwrap();
        
        let result = registry.start_forging("secret3");
        assert!(matches!(result, Err(GeneratorError::MaxForgersExceeded(2))));
    }
}
