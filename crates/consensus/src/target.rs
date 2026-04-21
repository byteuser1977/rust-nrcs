//! Target 计算模块
//!
//! 对应 Java: Block.java 中的 baseTarget 和 cumulativeDifficulty 计算
//!
//! 负责:
//! - 计算基础目标值 (base_target)
//! - 计算累计难度 (cumulative_difficulty)
//! - 难度调整算法

use blockchain_types::*;
use blockchain_types::prelude::Block;
use blockchain_types::constants::*;
use num_bigint::BigUint;
use num_traits::{ToPrimitive, One, Zero};

#[derive(Debug, Clone)]
pub struct TargetInfo {
    pub base_target: u64,
    pub cumulative_difficulty: BigUint,
}

impl TargetInfo {
    pub fn new(base_target: u64, cumulative_difficulty: BigUint) -> Self {
        Self {
            base_target,
            cumulative_difficulty,
        }
    }
}

pub struct TargetCalculator {
    initial_base_target: u64,
    max_base_target: u64,
    min_base_target: u64,
    gamma: u32,
}

impl TargetCalculator {
    pub fn new() -> Self {
        Self {
            initial_base_target: INITIAL_BASE_TARGET,
            max_base_target: MAX_BASE_TARGET,
            min_base_target: MIN_BASE_TARGET,
            gamma: BASE_TARGET_GAMMA,
        }
    }
    
    pub fn with_params(initial: u64, max: u64, min: u64, gamma: u32) -> Self {
        Self {
            initial_base_target: initial,
            max_base_target: max,
            min_base_target: min,
            gamma,
        }
    }
    
    pub fn calculate_base_target(
        &self,
        prev_block: &Block,
        prev_prev_block: Option<&Block>,
    ) -> u64 {
        let prev_base_target = prev_block.base_target;
        
        if let Some(prev_prev) = prev_prev_block {
            let block_time = prev_block.timestamp as i64 - prev_prev.timestamp as i64;
            
            if block_time <= 0 {
                return prev_base_target;
            }
            
            let target_block_time = BLOCK_TIME as i64;
            
            let new_target = if block_time < target_block_time * MIN_BLOCKTIME_LIMIT as i64 / BLOCK_TIME as i64 {
                let adjust_factor = (target_block_time * self.gamma as i64) / (block_time * self.gamma as i64);
                prev_base_target / adjust_factor.max(1) as u64
            } else if block_time > target_block_time * MAX_BLOCKTIME_LIMIT as i64 / BLOCK_TIME as i64 {
                let adjust_factor = (block_time * self.gamma as i64) / (target_block_time * self.gamma as i64);
                prev_base_target * adjust_factor.max(1) as u64
            } else {
                prev_base_target
            };
            
            new_target.clamp(self.min_base_target, self.max_base_target)
        } else {
            prev_base_target
        }
    }
    
    pub fn calculate_cumulative_difficulty(
        &self,
        prev_cumulative_difficulty: &[u8],
        base_target: u64,
    ) -> BigUint {
        let prev_difficulty = if prev_cumulative_difficulty.is_empty() {
            BigUint::one()
        } else {
            BigUint::from_bytes_be(prev_cumulative_difficulty)
        };
        
        if base_target == 0 {
            return prev_difficulty;
        }
        
        let difficulty_add = BigUint::from(self.max_base_target / base_target);
        prev_difficulty + difficulty_add
    }
    
    pub fn calculate_target_info(
        &self,
        prev_block: &Block,
        prev_prev_block: Option<&Block>,
    ) -> TargetInfo {
        let base_target = self.calculate_base_target(prev_block, prev_prev_block);
        let cumulative_difficulty = self.calculate_cumulative_difficulty(
            &prev_block.cumulative_difficulty,
            base_target,
        );
        
        TargetInfo::new(base_target, cumulative_difficulty)
    }
    
    pub fn calculate_average_block_time(blocks: &[Block]) -> u64 {
        if blocks.len() < 2 {
            return BLOCK_TIME as u64;
        }
        
        let total_time: u64 = blocks.windows(2)
            .map(|w| {
                let diff = w[1].timestamp as i64 - w[0].timestamp as i64;
                diff.max(0) as u64
            })
            .sum();
        
        total_time / (blocks.len() - 1) as u64
    }
    
    pub fn calculate_difficulty_ratio(&self, base_target: u64) -> BigUint {
        if base_target == 0 {
            return BigUint::one();
        }
        BigUint::from(self.max_base_target / base_target)
    }
    
    pub fn verify_base_target(&self, base_target: u64) -> bool {
        base_target >= self.min_base_target && base_target <= self.max_base_target
    }
    
    pub fn get_initial_base_target(&self) -> u64 {
        self.initial_base_target
    }
    
    pub fn get_max_base_target(&self) -> u64 {
        self.max_base_target
    }
    
    pub fn get_min_base_target(&self) -> u64 {
        self.min_base_target
    }
}

impl Default for TargetCalculator {
    fn default() -> Self {
        Self::new()
    }
}

pub fn biguint_to_bytes(value: &BigUint) -> Vec<u8> {
    value.to_bytes_be()
}

pub fn bytes_to_biguint(bytes: &[u8]) -> BigUint {
    if bytes.is_empty() {
        BigUint::one()
    } else {
        BigUint::from_bytes_be(bytes)
    }
}

pub fn calculate_hit(public_key: &[u8], generation_signature: &[u8; 64]) -> BigUint {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(generation_signature);
    hasher.update(public_key);
    let hash = hasher.finalize();
    
    let mut hit_bytes = [0u8; 8];
    hit_bytes.copy_from_slice(&hash[..8]);
    hit_bytes.reverse();
    
    BigUint::from(u64::from_be_bytes(hit_bytes))
}

pub fn calculate_deadline(
    hit: &BigUint,
    effective_balance: u64,
    base_target: u64,
) -> u64 {
    if effective_balance == 0 || base_target == 0 {
        return u64::MAX;
    }
    
    let effective_base_target = BigUint::from(base_target) * BigUint::from(effective_balance);
    
    if effective_base_target.is_zero() {
        return u64::MAX;
    }
    
    (hit / effective_base_target).to_u64().unwrap_or(u64::MAX)
}

pub fn verify_hit(
    hit: &BigUint,
    effective_balance: u64,
    base_target: u64,
    elapsed_time: u64,
) -> bool {
    if elapsed_time == 0 {
        return false;
    }
    
    let effective_base_target = BigUint::from(base_target) * BigUint::from(effective_balance);
    
    let prev_target = &effective_base_target * BigUint::from(elapsed_time - 1);
    let target = &prev_target + &effective_base_target;
    
    hit < &target && (hit >= &prev_target || elapsed_time > 600)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_calculator_creation() {
        let calc = TargetCalculator::new();
        assert_eq!(calc.get_initial_base_target(), INITIAL_BASE_TARGET);
        assert_eq!(calc.get_max_base_target(), MAX_BASE_TARGET);
        assert_eq!(calc.get_min_base_target(), MIN_BASE_TARGET);
    }

    #[test]
    fn test_cumulative_difficulty() {
        let calc = TargetCalculator::new();
        
        let prev_diff = BigUint::from(1000u64);
        let prev_bytes = biguint_to_bytes(&prev_diff);
        
        let new_diff = calc.calculate_cumulative_difficulty(&prev_bytes, 100_000);
        
        assert!(new_diff > prev_diff);
    }

    #[test]
    fn test_base_target_bounds() {
        let calc = TargetCalculator::new();
        
        assert!(calc.verify_base_target(INITIAL_BASE_TARGET));
        assert!(calc.verify_base_target(MAX_BASE_TARGET));
        assert!(calc.verify_base_target(MIN_BASE_TARGET));
        assert!(!calc.verify_base_target(0));
        assert!(!calc.verify_base_target(u64::MAX));
    }

    #[test]
    fn test_hit_calculation() {
        let public_key = [1u8; 32];
        let gen_sig = [2u8; 64];
        
        let hit = calculate_hit(&public_key, &gen_sig);
        assert!(hit > BigUint::zero());
    }

    #[test]
    fn test_deadline_calculation() {
        let hit = BigUint::from(1_000_000_000_000u64);
        let effective_balance = 1_000u64;
        let base_target = 1_000u64;
        
        let deadline = calculate_deadline(&hit, effective_balance, base_target);
        assert!(deadline > 0);
        assert!(deadline < u64::MAX);
    }

    #[test]
    fn test_verify_hit() {
        let hit = BigUint::from(5_000_000u64);
        let effective_balance = 100u64;
        let base_target = 100u64;
        
        assert!(verify_hit(&hit, effective_balance, base_target, 1000));
        assert!(!verify_hit(&hit, effective_balance, base_target, 0));
    }
}
