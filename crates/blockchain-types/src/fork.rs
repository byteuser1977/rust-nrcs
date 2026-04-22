//! 分叉处理模块
//!
//! 对应 Java: BlockchainProcessor 中的分叉检测和处理逻辑
//!
//! 负责:
//! - 检测区块链分叉
//! - 比较累计难度选择最佳链
//! - 区块回滚
//! - 分叉解决

use crate::*;
use crate::prelude::Block;
use crate::constants::*;
use num_bigint::BigUint;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ForkError {
    #[error("fork depth exceeds maximum rollback: {0}")]
    MaxRollbackExceeded(u32),
    
    #[error("rollback failed: {0}")]
    RollbackFailed(String),
    
    #[error("invalid fork: {0}")]
    InvalidFork(String),
    
    #[error("no common ancestor found")]
    NoCommonAncestor,
}

pub type ForkResult<T> = std::result::Result<T, ForkError>;

#[derive(Debug, Clone, PartialEq)]
pub enum ForkType {
    None,
    Soft,
    Hard,
}

#[derive(Debug, Clone)]
pub struct ForkInfo {
    pub fork_type: ForkType,
    pub fork_height: u32,
    pub common_ancestor_id: BlockId,
    pub local_chain_length: u32,
    pub peer_chain_length: u32,
    pub local_difficulty: BigUint,
    pub peer_difficulty: BigUint,
}

impl ForkInfo {
    pub fn new() -> Self {
        Self {
            fork_type: ForkType::None,
            fork_height: 0,
            common_ancestor_id: 0,
            local_chain_length: 0,
            peer_chain_length: 0,
            local_difficulty: BigUint::ZERO,
            peer_difficulty: BigUint::ZERO,
        }
    }
    
    pub fn should_switch(&self) -> bool {
        self.peer_difficulty > self.local_difficulty
    }
    
    pub fn rollback_depth(&self) -> u32 {
        self.local_chain_length.saturating_sub(self.fork_height)
    }
}

impl Default for ForkInfo {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ForkDetector {
    max_rollback: u32,
    fork_confirmations: u32,
}

impl ForkDetector {
    pub fn new() -> Self {
        Self {
            max_rollback: MAX_ROLLBACK,
            fork_confirmations: 1,
        }
    }
    
    pub fn with_params(max_rollback: u32, fork_confirmations: u32) -> Self {
        Self {
            max_rollback,
            fork_confirmations,
        }
    }
    
    pub fn detect_fork(
        &self,
        local_blocks: &[Block],
        peer_block_ids: &[BlockId],
    ) -> ForkResult<ForkInfo> {
        let mut fork_info = ForkInfo::new();
        
        if local_blocks.is_empty() || peer_block_ids.is_empty() {
            return Ok(fork_info);
        }
        
        let local_ids: Vec<BlockId> = local_blocks.iter()
            .filter_map(|b| b.compute_hash().ok())
            .map(|h| u64::from_be_bytes(h.0[..8].try_into().unwrap_or([0u8; 8])))
            .collect();
        
        let mut common_idx = None;
        for (i, &peer_id) in peer_block_ids.iter().enumerate() {
            if let Some(local_pos) = local_ids.iter().position(|&id| id == peer_id) {
                common_idx = Some((local_pos, i));
                break;
            }
        }
        
        match common_idx {
            Some((local_pos, _peer_pos)) => {
                let common_block = &local_blocks[local_pos];
                fork_info.common_ancestor_id = local_ids[local_pos];
                fork_info.fork_height = common_block.height;
                fork_info.local_chain_length = local_blocks.len() as u32;
                fork_info.peer_chain_length = peer_block_ids.len() as u32;
                
                let rollback_depth = local_blocks.len() - local_pos - 1;
                if rollback_depth as u32 > self.max_rollback {
                    fork_info.fork_type = ForkType::Hard;
                    return Err(ForkError::MaxRollbackExceeded(rollback_depth as u32));
                }
                
                fork_info.fork_type = ForkType::Soft;
                
                let local_diff: BigUint = local_blocks.last()
                    .map(|b| BigUint::from_bytes_be(&b.cumulative_difficulty))
                    .unwrap_or_default();
                fork_info.local_difficulty = local_diff;
                
                Ok(fork_info)
            }
            None => {
                fork_info.fork_type = ForkType::Hard;
                Err(ForkError::NoCommonAncestor)
            }
        }
    }
    
    pub fn compare_chains(
        &self,
        local_difficulty: &BigUint,
        peer_difficulty: &BigUint,
    ) -> std::cmp::Ordering {
        peer_difficulty.cmp(local_difficulty)
    }
    
    pub fn is_valid_fork(
        &self,
        fork_info: &ForkInfo,
        current_height: u32,
    ) -> bool {
        if fork_info.fork_type == ForkType::Hard {
            return false;
        }
        
        let rollback_depth = current_height.saturating_sub(fork_info.fork_height);
        rollback_depth < self.max_rollback
    }
}

impl Default for ForkDetector {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ForkResolver {
    max_rollback: u32,
}

impl ForkResolver {
    pub fn new() -> Self {
        Self {
            max_rollback: MAX_ROLLBACK,
        }
    }
    
    pub fn resolve_fork(
        &self,
        fork_info: &ForkInfo,
        local_blocks: &[Block],
        peer_blocks: &[Block],
    ) -> ForkResult<ForkResolution> {
        if !fork_info.should_switch() {
            return Ok(ForkResolution::KeepLocal);
        }
        
        if fork_info.rollback_depth() > self.max_rollback {
            return Err(ForkError::MaxRollbackExceeded(fork_info.rollback_depth()));
        }
        
        let rollback_blocks = self.get_blocks_to_rollback(local_blocks, fork_info.fork_height)?;
        let new_blocks = self.get_blocks_to_add(peer_blocks, fork_info.common_ancestor_id)?;
        
        Ok(ForkResolution::Switch {
            rollback_to: fork_info.fork_height,
            rollback_blocks,
            new_blocks,
        })
    }
    
    fn get_blocks_to_rollback(
        &self,
        blocks: &[Block],
        to_height: u32,
    ) -> ForkResult<Vec<Block>> {
        let mut rollback = Vec::new();
        
        for block in blocks.iter().rev() {
            if block.height <= to_height {
                break;
            }
            rollback.push(block.clone());
        }
        
        Ok(rollback)
    }
    
    fn get_blocks_to_add(
        &self,
        blocks: &[Block],
        after_id: BlockId,
    ) -> ForkResult<Vec<Block>> {
        let mut add_blocks = Vec::new();
        let mut found = false;
        
        for block in blocks {
            if found {
                add_blocks.push(block.clone());
            } else {
                let block_id = block.compute_hash()
                    .map_err(|e| ForkError::InvalidFork(format!("hash error: {:?}", e)))?;
                let id = u64::from_be_bytes(block_id.0[..8].try_into().unwrap_or([0u8; 8]));
                if id == after_id {
                    found = true;
                }
            }
        }
        
        Ok(add_blocks)
    }
}

impl Default for ForkResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum ForkResolution {
    KeepLocal,
    Switch {
        rollback_to: u32,
        rollback_blocks: Vec<Block>,
        new_blocks: Vec<Block>,
    },
}

pub struct BlockRollback {
    max_rollback: u32,
}

impl BlockRollback {
    pub fn new() -> Self {
        Self {
            max_rollback: MAX_ROLLBACK,
        }
    }
    
    pub fn rollback_to_height(
        &self,
        current_height: u32,
        target_height: u32,
    ) -> ForkResult<u32> {
        if current_height < target_height {
            return Err(ForkError::RollbackFailed(
                "target height greater than current".to_string()
            ));
        }
        
        let rollback_count = current_height - target_height;
        if rollback_count > self.max_rollback {
            return Err(ForkError::MaxRollbackExceeded(rollback_count));
        }
        
        Ok(rollback_count)
    }
    
    pub fn rollback_blocks(
        &self,
        blocks: &[Block],
        count: u32,
    ) -> ForkResult<Vec<Block>> {
        if count as usize > blocks.len() {
            return Err(ForkError::RollbackFailed(
                "rollback count exceeds available blocks".to_string()
            ));
        }
        
        let rollback: Vec<Block> = blocks.iter()
            .rev()
            .take(count as usize)
            .cloned()
            .collect();
        
        Ok(rollback)
    }
    
    pub fn verify_rollback(
        &self,
        blocks: &[Block],
        target_height: u32,
    ) -> ForkResult<bool> {
        if blocks.is_empty() {
            return Ok(true);
        }
        
        let last_block = blocks.last().unwrap();
        if last_block.height != target_height {
            return Err(ForkError::RollbackFailed(
                format!("rollback ended at wrong height: expected {}, got {}",
                    target_height, last_block.height)
            ));
        }
        
        Ok(true)
    }
}

impl Default for BlockRollback {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fork_info_creation() {
        let info = ForkInfo::new();
        assert_eq!(info.fork_type, ForkType::None);
        assert!(!info.should_switch());
    }

    #[test]
    fn test_fork_detector_creation() {
        let detector = ForkDetector::new();
        assert!(detector.max_rollback > 0);
    }

    #[test]
    fn test_fork_detector_no_fork() {
        let detector = ForkDetector::new();
        
        let local_blocks = vec![];
        let peer_block_ids = vec![];
        
        let result = detector.detect_fork(&local_blocks, &peer_block_ids);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().fork_type, ForkType::None);
    }

    #[test]
    fn test_fork_resolver_creation() {
        let resolver = ForkResolver::new();
        assert!(resolver.max_rollback > 0);
    }

    #[test]
    fn test_fork_resolution_keep_local() {
        let resolver = ForkResolver::new();
        
        let mut fork_info = ForkInfo::new();
        fork_info.local_difficulty = BigUint::from(2000u64);
        fork_info.peer_difficulty = BigUint::from(1000u64);
        
        let result = resolver.resolve_fork(&fork_info, &[], &[]);
        assert!(matches!(result, Ok(ForkResolution::KeepLocal)));
    }

    #[test]
    fn test_block_rollback_creation() {
        let rollback = BlockRollback::new();
        assert!(rollback.max_rollback > 0);
    }

    #[test]
    fn test_rollback_height_calculation() {
        let rollback = BlockRollback::new();
        
        let result = rollback.rollback_to_height(100, 90);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10);
    }

    #[test]
    fn test_rollback_exceeds_limit() {
        let rollback = BlockRollback::new();
        
        let result = rollback.rollback_to_height(1000, 100);
        assert!(matches!(result, Err(ForkError::MaxRollbackExceeded(_))));
    }
}
