//! 区块验证器模块
//!
//! 对应 Java: BlockchainProcessor 中的区块验证逻辑
//!
//! 负责:
//! - 基础区块验证
//! - 区块签名验证
//! - 时间戳验证
//! - 难度验证
//! - Payload哈希验证

use crate::*;
use crate::prelude::{Block, Transaction};
use crate::constants::*;
use sha2::{Sha256, Digest};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("invalid block version: {0}")]
    InvalidVersion(u32),
    
    #[error("invalid block height: {0}")]
    InvalidHeight(u32),
    
    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(String),
    
    #[error("previous block hash mismatch")]
    PreviousHashMismatch,
    
    #[error("payload hash mismatch")]
    PayloadHashMismatch,
    
    #[error("invalid base target: {0}")]
    InvalidBaseTarget(u64),
    
    #[error("invalid block signature")]
    InvalidSignature,
    
    #[error("too many transactions: {0}")]
    TooManyTransactions(usize),
    
    #[error("payload too large: {0} > {1}")]
    PayloadTooLarge(usize, usize),
    
    #[error("invalid generator")]
    InvalidGenerator,
    
    #[error("generation signature mismatch")]
    GenerationSignatureMismatch,
}

pub type ValidationResult<T> = std::result::Result<T, ValidationError>;

pub struct BlockValidator {
    max_transactions: usize,
    max_payload_length: usize,
    max_timedrift: u32,
}

impl BlockValidator {
    pub fn new() -> Self {
        Self {
            max_transactions: MAX_NUMBER_OF_TRANSACTIONS,
            max_payload_length: MAX_PAYLOAD_LENGTH,
            max_timedrift: MAX_TIMEDRIFT,
        }
    }
    
    pub fn validate_basic(&self, block: &Block) -> ValidationResult<()> {
        if block.version != BLOCK_VERSION {
            return Err(ValidationError::InvalidVersion(block.version));
        }
        
        if block.height == 0 {
            return Err(ValidationError::InvalidHeight(0));
        }
        
        Ok(())
    }
    
    pub fn validate_timestamp(
        &self,
        block: &Block,
        prev_block: &Block,
        current_time: u32,
    ) -> ValidationResult<()> {
        if block.timestamp <= prev_block.timestamp {
            return Err(ValidationError::InvalidTimestamp(
                "block timestamp not greater than previous".to_string()
            ));
        }
        
        if block.timestamp > current_time + self.max_timedrift {
            return Err(ValidationError::InvalidTimestamp(
                format!("block timestamp {} too far in future (current: {})", 
                    block.timestamp, current_time)
            ));
        }
        
        Ok(())
    }
    
    pub fn validate_previous_hash(
        &self,
        block: &Block,
        prev_block: &Block,
    ) -> ValidationResult<()> {
        let expected_hash = prev_block.compute_hash()
            .map_err(|_| ValidationError::PreviousHashMismatch)?;
        
        if block.previous_block_hash != expected_hash {
            return Err(ValidationError::PreviousHashMismatch);
        }
        
        Ok(())
    }
    
    pub fn validate_height(
        &self,
        block: &Block,
        prev_block: &Block,
    ) -> ValidationResult<()> {
        if block.height != prev_block.height + 1 {
            return Err(ValidationError::InvalidHeight(block.height));
        }
        
        Ok(())
    }
    
    pub fn validate_payload(&self, block: &Block) -> ValidationResult<()> {
        if block.transactions.len() > self.max_transactions {
            return Err(ValidationError::TooManyTransactions(block.transactions.len()));
        }
        
        let payload_length = self.calculate_payload_length(&block.transactions);
        if payload_length > self.max_payload_length {
            return Err(ValidationError::PayloadTooLarge(
                payload_length,
                self.max_payload_length
            ));
        }
        
        let computed_hash = self.compute_payload_hash(&block.transactions);
        if computed_hash != block.payload_hash {
            return Err(ValidationError::PayloadHashMismatch);
        }
        
        Ok(())
    }
    
    pub fn validate_base_target(
        &self,
        block: &Block,
        prev_block: &Block,
        prev_prev_block: Option<&Block>,
    ) -> ValidationResult<()> {
        let expected_target = self.calculate_base_target(prev_block, prev_prev_block);
        
        if block.base_target != expected_target {
            return Err(ValidationError::InvalidBaseTarget(block.base_target));
        }
        
        Ok(())
    }
    
    pub fn validate_signature(
        &self,
        block: &Block,
        generator_public_key: &[u8],
    ) -> ValidationResult<()> {
        let header_data = self.serialize_block_header(block);
        
        let mut hasher = Sha256::new();
        hasher.update(&header_data);
        hasher.update(generator_public_key);
        let expected_sig = hasher.finalize();
        
        let sig_valid = block.block_signature.0[..32] == expected_sig[..];
        
        if !sig_valid {
            return Err(ValidationError::InvalidSignature);
        }
        
        Ok(())
    }
    
    pub fn validate_full(
        &self,
        block: &Block,
        prev_block: &Block,
        prev_prev_block: Option<&Block>,
        current_time: u32,
        generator_public_key: Option<&[u8]>,
    ) -> ValidationResult<()> {
        self.validate_basic(block)?;
        self.validate_height(block, prev_block)?;
        self.validate_timestamp(block, prev_block, current_time)?;
        self.validate_previous_hash(block, prev_block)?;
        self.validate_payload(block)?;
        self.validate_base_target(block, prev_block, prev_prev_block)?;
        
        if let Some(pk) = generator_public_key {
            self.validate_signature(block, pk)?;
        }
        
        Ok(())
    }
    
    fn calculate_payload_length(&self, transactions: &[Transaction]) -> usize {
        transactions.iter()
            .map(|tx| MIN_TRANSACTION_SIZE + tx.attachment_bytes.len())
            .sum()
    }
    
    fn compute_payload_hash(&self, transactions: &[Transaction]) -> Hash256 {
        let mut hasher = Sha256::new();
        
        for tx in transactions {
            if let Ok(tx_hash) = tx.compute_hash() {
                hasher.update(&tx_hash.0);
            }
        }
        
        let hash = hasher.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hash);
        Hash256(arr)
    }
    
    fn calculate_base_target(
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
                let adjust_factor = (target_block_time * BASE_TARGET_GAMMA as i64) 
                    / (block_time * BASE_TARGET_GAMMA as i64);
                prev_base_target / adjust_factor.max(1) as u64
            } else if block_time > target_block_time * MAX_BLOCKTIME_LIMIT as i64 / BLOCK_TIME as i64 {
                let adjust_factor = (block_time * BASE_TARGET_GAMMA as i64) 
                    / (target_block_time * BASE_TARGET_GAMMA as i64);
                prev_base_target * adjust_factor.max(1) as u64
            } else {
                prev_base_target
            };
            
            new_target.clamp(MIN_BASE_TARGET, MAX_BASE_TARGET)
        } else {
            prev_base_target
        }
    }
    
    fn serialize_block_header(&self, block: &Block) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&block.version.to_be_bytes());
        buf.extend_from_slice(&block.timestamp.to_be_bytes());
        buf.extend_from_slice(&block.height.to_be_bytes());
        buf.extend_from_slice(&block.previous_block_hash.0);
        buf.extend_from_slice(&block.payload_hash.0);
        buf.extend_from_slice(&block.generator_id.to_be_bytes());
        buf.extend_from_slice(&block.nonce.to_be_bytes());
        buf.extend_from_slice(&block.base_target.to_be_bytes());
        buf.extend_from_slice(&(block.cumulative_difficulty.len() as u32).to_be_bytes());
        buf.extend_from_slice(&block.cumulative_difficulty);
        buf.extend_from_slice(&block.total_amount.to_be_bytes());
        buf.extend_from_slice(&block.total_fee.to_be_bytes());
        buf.extend_from_slice(&block.payload_length.to_be_bytes());
        buf.extend_from_slice(&block.generation_signature.0);
        buf
    }
}

impl Default for BlockValidator {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TransactionValidator {
    min_fee: u64,
    max_amount: u64,
}

impl TransactionValidator {
    pub fn new() -> Self {
        Self {
            min_fee: 0,
            max_amount: MAX_BALANCE_NQT,
        }
    }
    
    pub fn validate_basic(&self, tx: &Transaction) -> ValidationResult<()> {
        if tx.amount > self.max_amount {
            return Err(ValidationError::InvalidTimestamp(
                format!("amount {} exceeds maximum", tx.amount)
            ));
        }
        
        Ok(())
    }
    
    pub fn validate_fee(&self, tx: &Transaction, min_fee: u64) -> ValidationResult<()> {
        if tx.fee < min_fee {
            return Err(ValidationError::InvalidTimestamp(
                format!("fee {} below minimum {}", tx.fee, min_fee)
            ));
        }
        
        Ok(())
    }
    
    pub fn validate_timestamp(&self, tx: &Transaction, current_time: u32) -> ValidationResult<()> {
        if tx.timestamp > current_time + MAX_TIMEDRIFT {
            return Err(ValidationError::InvalidTimestamp(
                format!("transaction timestamp {} too far in future", tx.timestamp)
            ));
        }
        
        Ok(())
    }
}

impl Default for TransactionValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_block(version: u32, height: u32) -> Block {
        Block {
            version,
            timestamp: 0,
            height,
            previous_block_hash: Hash256([0u8; 32]),
            payload_hash: Hash256([0u8; 32]),
            generator_id: 0,
            nonce: 0,
            base_target: INITIAL_BASE_TARGET,
            cumulative_difficulty: vec![],
            total_amount: 0,
            total_fee: 0,
            payload_length: 0,
            generation_signature: Hash512([0u8; 64]),
            block_signature: Hash512([0u8; 64]),
            transactions: vec![],
        }
    }

    #[test]
    fn test_block_validator_creation() {
        let validator = BlockValidator::new();
        assert!(validator.max_transactions > 0);
        assert!(validator.max_payload_length > 0);
    }

    #[test]
    fn test_validate_basic() {
        let validator = BlockValidator::new();
        
        let block = create_test_block(BLOCK_VERSION, 1);
        
        assert!(validator.validate_basic(&block).is_ok());
    }

    #[test]
    fn test_validate_basic_invalid_version() {
        let validator = BlockValidator::new();
        
        let block = create_test_block(99, 1);
        
        assert!(matches!(
            validator.validate_basic(&block),
            Err(ValidationError::InvalidVersion(99))
        ));
    }

    #[test]
    fn test_validate_height() {
        let validator = BlockValidator::new();
        
        let prev_block = create_test_block(BLOCK_VERSION, 10);
        let block = create_test_block(BLOCK_VERSION, 11);
        
        assert!(validator.validate_height(&block, &prev_block).is_ok());
    }

    #[test]
    fn test_validate_height_mismatch() {
        let validator = BlockValidator::new();
        
        let prev_block = create_test_block(BLOCK_VERSION, 10);
        let block = create_test_block(BLOCK_VERSION, 12);
        
        assert!(matches!(
            validator.validate_height(&block, &prev_block),
            Err(ValidationError::InvalidHeight(12))
        ));
    }

    #[test]
    fn test_transaction_validator_creation() {
        let validator = TransactionValidator::new();
        assert_eq!(validator.min_fee, 0);
    }
}
