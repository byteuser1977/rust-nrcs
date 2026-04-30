//! 区块生成模块
//!
//! 对应 Java: BlockchainProcessor.generateBlock()
//!
//! 负责:
//! - 收集交易并构建新区块
//! - 计算区块哈希和签名
//! - 区块验证

use blockchain_types::*;
use blockchain_types::prelude::{Block, Transaction};
use sha2::{Sha256, Digest};

use crate::target::TargetCalculator;
use crate::generation_signature::calculate_generation_signature;

#[derive(Debug, Clone)]
pub struct BlockTemplate {
    pub version: i32,
    pub timestamp: u32,
    pub height: u32,
    pub previous_block_hash: Hash256,
    pub generator_id: AccountId,
    pub base_target: u64,
    pub cumulative_difficulty: Vec<u8>,
    pub generation_signature: Vec<u8>,
    pub transactions: Vec<Transaction>,
}

impl BlockTemplate {
    pub fn new(
        height: u32,
        previous_block_hash: Hash256,
        generator_id: AccountId,
    ) -> Self {
        Self {
            version: BLOCK_VERSION,
            timestamp: 0,
            height,
            previous_block_hash,
            generator_id,
            base_target: INITIAL_BASE_TARGET,
            cumulative_difficulty: vec![],
            generation_signature: vec![0u8; 32],
            transactions: Vec::new(),
        }
    }
}

pub struct BlockGenerator {
    target_calculator: TargetCalculator,
    max_payload_length: usize,
    max_transactions: usize,
}

impl BlockGenerator {
    pub fn new() -> Self {
        Self {
            target_calculator: TargetCalculator::new(),
            max_payload_length: MAX_PAYLOAD_LENGTH,
            max_transactions: MAX_NUMBER_OF_TRANSACTIONS,
        }
    }
    
    pub fn create_block_template(
        &self,
        prev_block: &Block,
        generator_id: AccountId,
        timestamp: u32,
        transactions: Vec<Transaction>,
    ) -> BlockTemplate {
        let target_info = self.target_calculator.calculate_target_info(
            prev_block,
            None,
        );
        
        let generation_signature = calculate_generation_signature(
            &prev_block.generation_signature,
            generator_id,
        );
        
        let mut template = BlockTemplate::new(
            prev_block.height + 1,
            prev_block.compute_hash().unwrap_or(Hash256([0u8; 32])),
            generator_id,
        );
        
        template.timestamp = timestamp;
        template.base_target = target_info.base_target;
        template.cumulative_difficulty = target_info.cumulative_difficulty.to_bytes_be();
        template.generation_signature = generation_signature;
        template.transactions = self.filter_transactions(transactions);
        
        template
    }
    
    fn filter_transactions(&self, mut transactions: Vec<Transaction>) -> Vec<Transaction> {
        // 对应 Java: 按 fee 降序排列，优先打包高 fee 交易
        transactions.sort_by(|a, b| b.fee.cmp(&a.fee));

        let mut result = Vec::new();
        let mut total_size = 0usize;

        for tx in transactions {
            let tx_size = self.estimate_transaction_size(&tx);

            if total_size + tx_size > self.max_payload_length {
                break;
            }

            if result.len() >= self.max_transactions {
                break;
            }

            total_size += tx_size;
            result.push(tx);
        }

        result
    }
    
    fn estimate_transaction_size(&self, tx: &Transaction) -> usize {
        MIN_TRANSACTION_SIZE + tx.attachment_bytes.len()
    }
    
    pub fn build_block(
        &self,
        template: BlockTemplate,
        secret_phrase: &str,
    ) -> std::result::Result<Block, String> {
        let payload_hash = self.calculate_payload_hash(&template.transactions);
        
        let total_amount: u64 = template.transactions.iter()
            .map(|tx| tx.amount)
            .sum();
        
        let total_fee: u64 = template.transactions.iter()
            .map(|tx| tx.fee)
            .sum();
        
        let payload_length = self.calculate_payload_length(&template.transactions);
        
        let mut block = Block {
            id: None,
            version: template.version,
            timestamp: template.timestamp,
            height: template.height,
            previous_block_id: Some(0),
            previous_block_hash: template.previous_block_hash,
            payload_hash,
            generator_id: Some(template.generator_id),
            generator_public_key: None,
            nonce: 0,
            base_target: template.base_target,
            cumulative_difficulty: template.cumulative_difficulty,
            total_amount,
            total_fee,
            payload_length: payload_length as u32,
            generation_signature: template.generation_signature,
            block_signature: Hash512([0u8; 64]),
            transactions: template.transactions,
        };
        
        let signature = self.sign_block(&block, secret_phrase)?;
        block.block_signature = signature;
        
        Ok(block)
    }
    
    fn calculate_payload_hash(&self, transactions: &[Transaction]) -> Hash256 {
        let mut hasher = Sha256::new();

        for tx in transactions {
            hasher.update(tx.get_bytes());
        }

        let hash = hasher.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hash);
        Hash256(arr)
    }
    
    fn calculate_payload_length(&self, transactions: &[Transaction]) -> usize {
        transactions.iter()
            .map(|tx| MIN_TRANSACTION_SIZE + tx.attachment_bytes.len())
            .sum()
    }
    
    fn sign_block(&self, block: &Block, secret_phrase: &str) -> std::result::Result<Hash512, String> {
        // 对应 Java: Crypto.sign(block.getHeader(), secretPhrase)
        let header_data = Self::serialize_block_header(block);
        let seed = crypto::sha256(secret_phrase.as_bytes());
        let keypair = crypto::keypair_from_seed(&seed);
        let sig = keypair.sign(&header_data);
        Ok(Hash512(sig))
    }
    
    pub fn serialize_block_header(block: &Block) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&block.version.to_be_bytes());
        buf.extend_from_slice(&block.timestamp.to_be_bytes());
        buf.extend_from_slice(&block.height.to_be_bytes());
        buf.extend_from_slice(&block.previous_block_hash.0);
        buf.extend_from_slice(&block.payload_hash.0);
        buf.extend_from_slice(&block.get_generator_id().to_be_bytes());
        buf.extend_from_slice(&block.nonce.to_be_bytes());
        buf.extend_from_slice(&block.base_target.to_be_bytes());
        buf.extend_from_slice(&(block.cumulative_difficulty.len() as u32).to_be_bytes());
        buf.extend_from_slice(&block.cumulative_difficulty);
        buf.extend_from_slice(&block.total_amount.to_be_bytes());
        buf.extend_from_slice(&block.total_fee.to_be_bytes());
        buf.extend_from_slice(&block.payload_length.to_be_bytes());
        buf.extend_from_slice(&block.generation_signature);
        buf
    }
    
    pub fn verify_block(&self, block: &Block, prev_block: &Block) -> std::result::Result<(), String> {
        if block.version != BLOCK_VERSION {
            return Err(format!("Invalid block version: {}", block.version));
        }
        
        if block.height != prev_block.height + 1 {
            return Err(format!("Invalid block height: {}", block.height));
        }
        
        if block.timestamp <= prev_block.timestamp {
            return Err("Block timestamp must be greater than previous block".to_string());
        }
        
        let expected_prev_hash = prev_block.compute_hash()
            .map_err(|e| format!("Failed to compute previous block hash: {:?}", e))?;
        if block.previous_block_hash != expected_prev_hash {
            return Err("Previous block hash mismatch".to_string());
        }
        
        if !self.target_calculator.verify_base_target(block.base_target) {
            return Err(format!("Invalid base target: {}", block.base_target));
        }
        
        if block.transactions.len() > self.max_transactions {
            return Err(format!("Too many transactions: {}", block.transactions.len()));
        }
        
        Ok(())
    }
    
    pub fn calculate_block_reward(&self, height: u32) -> u64 {
        if height == 0 {
            return 0;
        }
        
        let initial_reward = 1000 * ONE_NRCS;
        let halving_interval = 1_000_000u32;
        let halvings = height / halving_interval;
        
        if halvings >= 64 {
            return 0;
        }
        
        initial_reward >> halvings
    }
}

impl Default for BlockGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_generator_creation() {
        let generator = BlockGenerator::new();
        assert!(generator.max_payload_length > 0);
        assert!(generator.max_transactions > 0);
    }

    #[test]
    fn test_block_template_creation() {
        let template = BlockTemplate::new(1, Hash256([0u8; 32]), 123);
        assert_eq!(template.height, 1);
        assert_eq!(template.generator_id, 123);
    }

    #[test]
    fn test_block_reward_calculation() {
        let generator = BlockGenerator::new();
        
        let reward_1 = generator.calculate_block_reward(1);
        assert!(reward_1 > 0);
        
        let reward_100 = generator.calculate_block_reward(100);
        assert!(reward_100 > 0);
        
        let reward_high = generator.calculate_block_reward(10_000_000);
        assert!(reward_high < reward_1);
    }
}
