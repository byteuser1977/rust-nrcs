//! 区块同步模块
//!
//! 对应 Java: BlockchainProcessor 中的区块下载和同步逻辑
//!
//! 负责:
//! - 获取对等节点的累计难度
//! - 查找共同里程碑区块
//! - 下载区块
//! - 区块导入

use crate::*;
use crate::prelude::Block;
use crate::constants::*;
use num_bigint::BigUint;
use thiserror::Error;
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("peer not available")]
    PeerNotAvailable,
    
    #[error("cumulative difficulty too low")]
    DifficultyTooLow,
    
    #[error("common block not found")]
    CommonBlockNotFound,
    
    #[error("block download failed: {0}")]
    DownloadFailed(String),
    
    #[error("block validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("fork detected at height {0}")]
    ForkDetected(u32),
    
    #[error("rollback limit exceeded")]
    RollbackLimitExceeded,
    
    #[error("network error: {0}")]
    NetworkError(String),
    
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

pub type SyncResult<T> = std::result::Result<T, SyncError>;

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: String,
    pub address: String,
    pub blockchain_height: u32,
    pub cumulative_difficulty: BigUint,
    pub is_connected: bool,
}

impl PeerInfo {
    pub fn new(peer_id: String, address: String) -> Self {
        Self {
            peer_id,
            address,
            blockchain_height: 0,
            cumulative_difficulty: BigUint::ZERO,
            is_connected: false,
        }
    }
    
    pub fn has_better_chain(&self, local_difficulty: &BigUint) -> bool {
        self.cumulative_difficulty > *local_difficulty
    }
}

#[derive(Debug, Clone, Default)]
pub struct SyncState {
    pub is_syncing: bool,
    pub is_downloading: bool,
    pub last_feeder: Option<String>,
    pub last_feeder_height: u32,
    pub blocks_downloaded: u64,
    pub start_time: u64,
}

pub const MAX_ROLLBACK: u32 = 720;
pub const SEGMENT_SIZE: usize = 36;
pub const MAX_BLOCK_IDS: usize = 1440;
pub const MAX_MILESTONE_IDS: usize = 10;

pub struct BlockSyncer {
    max_rollback: u32,
    fork_confirmations: u32,
    state: SyncState,
}

impl BlockSyncer {
    pub fn new() -> Self {
        Self {
            max_rollback: MAX_ROLLBACK,
            fork_confirmations: 1,
            state: SyncState::default(),
        }
    }
    
    pub fn with_params(max_rollback: u32, fork_confirmations: u32) -> Self {
        Self {
            max_rollback,
            fork_confirmations,
            state: SyncState::default(),
        }
    }
    
    pub fn get_state(&self) -> &SyncState {
        &self.state
    }
    
    pub fn is_syncing(&self) -> bool {
        self.state.is_syncing
    }
    
    pub fn start_sync(&mut self) {
        self.state.is_syncing = true;
        self.state.is_downloading = true;
        self.state.start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
    
    pub fn stop_sync(&mut self) {
        self.state.is_syncing = false;
        self.state.is_downloading = false;
    }
    
    pub fn select_best_peer<'a>(
        &self,
        peers: &'a [PeerInfo],
        local_difficulty: &BigUint,
    ) -> Option<&'a PeerInfo> {
        peers.iter()
            .filter(|p| p.is_connected && p.has_better_chain(local_difficulty))
            .max_by(|a, b| {
                a.cumulative_difficulty.cmp(&b.cumulative_difficulty)
            })
    }
    
    pub fn find_common_milestone_block(
        &self,
        local_block_ids: &[i64],
        peer_block_ids: &[i64],
    ) -> SyncResult<i64> {
        for block_id in peer_block_ids {
            if local_block_ids.contains(block_id) {
                return Ok(*block_id);
            }
        }
        
        Err(SyncError::CommonBlockNotFound)
    }
    
    pub fn get_block_ids_after_common(
        &self,
        common_block_id: i64,
        peer_block_ids: &[i64],
    ) -> Vec<i64> {
        let mut result = Vec::new();
        let mut found = false;
        
        for &block_id in peer_block_ids {
            if found {
                result.push(block_id);
                if result.len() >= MAX_BLOCK_IDS {
                    break;
                }
            } else if block_id == common_block_id {
                found = true;
            }
        }
        
        result
    }
    
    pub fn validate_download_range(
        &self,
        common_height: u32,
        current_height: u32,
    ) -> SyncResult<()> {
        let rollback_distance = current_height.saturating_sub(common_height);
        
        if rollback_distance >= self.max_rollback {
            return Err(SyncError::RollbackLimitExceeded);
        }
        
        Ok(())
    }
    
    pub fn calculate_download_progress(
        &self,
        current_height: u32,
        target_height: u32,
    ) -> f64 {
        if target_height == 0 {
            return 0.0;
        }
        
        (current_height as f64 / target_height as f64) * 100.0
    }
    
    pub fn estimate_remaining_time(
        &self,
        blocks_remaining: u32,
        blocks_per_second: f64,
    ) -> u64 {
        if blocks_per_second <= 0.0 {
            return 0;
        }
        
        (blocks_remaining as f64 / blocks_per_second) as u64
    }
    
    pub fn update_download_stats(&mut self, blocks_count: u64) {
        self.state.blocks_downloaded += blocks_count;
    }
    
    pub fn set_last_feeder(&mut self, peer_id: String, height: u32) {
        self.state.last_feeder = Some(peer_id);
        self.state.last_feeder_height = height;
    }
    
    pub fn verify_cumulative_difficulty(
        &self,
        peer_difficulty: &BigUint,
        local_difficulty: &BigUint,
    ) -> bool {
        peer_difficulty > local_difficulty
    }
}

impl Default for BlockSyncer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BlockDownloader {
    batch_size: usize,
    timeout_ms: u64,
}

impl BlockDownloader {
    pub fn new() -> Self {
        Self {
            batch_size: SEGMENT_SIZE,
            timeout_ms: 30000,
        }
    }
    
    pub fn with_params(batch_size: usize, timeout_ms: u64) -> Self {
        Self {
            batch_size,
            timeout_ms,
        }
    }
    
    pub fn download_blocks(
        &self,
        _peer: &PeerInfo,
        _from_height: u32,
        _count: usize,
    ) -> SyncResult<Vec<Block>> {
        Ok(Vec::new())
    }
    
    pub fn get_next_blocks_request(
        &self,
        block_ids: &[i64],
        offset: usize,
    ) -> Vec<i64> {
        block_ids.iter()
            .skip(offset)
            .take(self.batch_size)
            .copied()
            .collect()
    }
    
    pub fn validate_downloaded_blocks(
        &self,
        blocks: &[Block],
        expected_prev_hash: &Hash256,
    ) -> SyncResult<()> {
        if blocks.is_empty() {
            return Err(SyncError::DownloadFailed("no blocks received".to_string()));
        }
        
        let first_block = &blocks[0];
        if first_block.previous_block_hash != *expected_prev_hash {
            return Err(SyncError::ValidationFailed(
                "first block previous hash mismatch".to_string()
            ));
        }
        
        for i in 1..blocks.len() {
            let prev_block = &blocks[i - 1];
            let curr_block = &blocks[i];
            
            if curr_block.height != prev_block.height + 1 {
                return Err(SyncError::ValidationFailed(
                    format!("block height gap at index {}", i)
                ));
            }
            
            let prev_hash = prev_block.compute_hash()
                .map_err(|e| SyncError::ValidationFailed(format!("hash error: {:?}", e)))?;
            
            if curr_block.previous_block_hash != prev_hash {
                return Err(SyncError::ValidationFailed(
                    format!("block chain broken at height {}", curr_block.height)
                ));
            }
        }
        
        Ok(())
    }
}

impl Default for BlockDownloader {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BlockImporter {
    verify_signatures: bool,
    verify_timestamps: bool,
}

impl BlockImporter {
    pub fn new() -> Self {
        Self {
            verify_signatures: true,
            verify_timestamps: true,
        }
    }
    
    pub fn import_block(
        &self,
        block: &Block,
        prev_block: &Block,
    ) -> SyncResult<()> {
        self.verify_block(block, prev_block)?;
        Ok(())
    }
    
    fn verify_block(&self, block: &Block, prev_block: &Block) -> SyncResult<()> {
        if block.version != BLOCK_VERSION {
            return Err(SyncError::ValidationFailed(
                format!("invalid block version: {}", block.version)
            ));
        }
        
        if block.height != prev_block.height + 1 {
            return Err(SyncError::ValidationFailed(
                format!("invalid block height: expected {}, got {}", 
                    prev_block.height + 1, block.height)
            ));
        }
        
        if self.verify_timestamps && block.timestamp <= prev_block.timestamp {
            return Err(SyncError::ValidationFailed(
                "block timestamp not greater than previous".to_string()
            ));
        }
        
        let expected_prev_hash = prev_block.compute_hash()
            .map_err(|e| SyncError::ValidationFailed(format!("hash error: {:?}", e)))?;
        
        if block.previous_block_hash != expected_prev_hash {
            return Err(SyncError::ValidationFailed(
                "previous block hash mismatch".to_string()
            ));
        }
        
        Ok(())
    }
    
    pub fn import_batch(
        &self,
        blocks: &[Block],
        prev_block: &Block,
    ) -> SyncResult<u32> {
        let mut last_block = prev_block.clone();
        let mut imported = 0u32;
        
        for block in blocks {
            self.import_block(block, &last_block)?;
            last_block = block.clone();
            imported += 1;
        }
        
        Ok(imported)
    }
}

impl Default for BlockImporter {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SyncSegment {
    pub start: usize,
    pub stop: usize,
    pub request_count: usize,
    pub blocks: Vec<Block>,
}

impl SyncSegment {
    pub fn new(chain_block_ids: &[i64], start: usize, stop: usize) -> Self {
        Self {
            start,
            stop,
            request_count: 0,
            blocks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DownloadStats {
    pub total_blocks: u64,
    pub total_time_ms: u64,
    pub blocks_per_second: f64,
}

impl Default for DownloadStats {
    fn default() -> Self {
        Self {
            total_blocks: 0,
            total_time_ms: 0,
            blocks_per_second: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_state_default() {
        let state = SyncState::default();
        assert!(!state.is_syncing);
        assert!(!state.is_downloading);
    }

    #[test]
    fn test_block_syncer_creation() {
        let syncer = BlockSyncer::new();
        assert!(!syncer.is_syncing());
    }

    #[test]
    fn test_sync_start_stop() {
        let mut syncer = BlockSyncer::new();
        
        syncer.start_sync();
        assert!(syncer.is_syncing());
        
        syncer.stop_sync();
        assert!(!syncer.is_syncing());
    }

    #[test]
    fn test_peer_info_better_chain() {
        let peer = PeerInfo::new("peer1".to_string(), "127.0.0.1:8080".to_string());
        let local_diff = BigUint::from(1000u64);
        
        assert!(!peer.has_better_chain(&local_diff));
        
        let mut better_peer = peer.clone();
        better_peer.cumulative_difficulty = BigUint::from(2000u64);
        assert!(better_peer.has_better_chain(&local_diff));
    }

    #[test]
    fn test_download_progress() {
        let syncer = BlockSyncer::new();
        
        let progress = syncer.calculate_download_progress(500, 1000);
        assert!((progress - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_block_downloader_creation() {
        let downloader = BlockDownloader::new();
        assert_eq!(downloader.batch_size, SEGMENT_SIZE);
    }
    
    #[test]
    fn test_find_common_milestone_block() {
        let syncer = BlockSyncer::new();
        let local_ids = vec![1, 2, 3, 4, 5];
        let peer_ids = vec![6, 5, 4, 3];
        
        let result = syncer.find_common_milestone_block(&local_ids, &peer_ids);
        assert_eq!(result.unwrap(), 5);
    }
    
    #[test]
    fn test_get_block_ids_after_common() {
        let syncer = BlockSyncer::new();
        let peer_ids = vec![1, 2, 3, 4, 5, 6, 7];
        
        let result = syncer.get_block_ids_after_common(3, &peer_ids);
        assert_eq!(result, vec![4, 5, 6, 7]);
    }
    
    #[test]
    fn test_validate_download_range() {
        let syncer = BlockSyncer::new();
        
        assert!(syncer.validate_download_range(100, 200).is_ok());
        assert!(syncer.validate_download_range(100, 900).is_err());
    }
}
