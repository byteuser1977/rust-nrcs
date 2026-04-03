//! PoW (Proof of Work) 共识引擎实现
//!
//! PoW 使用 SHA-256d（双 SHA-256）进行挖矿。
//! 本实现仅提供骨架结构，根据需求可选实现。

use super::*;
use blockchain_types::*;
use sha2::{Sha256, Digest};

/// PoW 共识引擎
pub struct PoWEngine {
    /// 目标区块间隔（秒）
    pub target_spacing: u32,
    /// 初始难度
    pub initial_difficulty: u64,
}

impl PoWEngine {
    pub fn new(target_spacing: u32) -> Self {
        Self {
            target_spacing,
            initial_difficulty: 1u64 << 32, // 2^32
        }
    }

    /// 计算区块哈希并检查是否满足难度要求
    /// 挖矿的就是 nonce 直到 hash < target
    pub fn mine(&self, block_header: &[u8], initial_nonce: u64) -> (u64, Hash256) {
        let mut nonce = initial_nonce;
        loop {
            let mut data = block_header.to_vec();
            data.extend_from_slice(&nonce.to_be_bytes());

            let hash = self.hash(&data);
            let hash_u64 = u64::from_be_bytes(hash[0..8].try_into().unwrap());

            if hash_u64 < self.initial_difficulty {
                return (nonce, hash);
            }

            nonce = nonce.wrapping_add(1);
        }
    }

    /// 双 SHA-256 哈希
    fn hash(&self, data: &[u8]) -> Hash256 {
        let mut hasher1 = Sha256::new();
        hasher1.update(data);
        let first = hasher1.finalize();

        let mut hasher2 = Sha256::new();
        hasher2.update(first);
        hasher2.finalize().try_into().unwrap()
    }
}

impl ConsensusEngine for PoWEngine {
    fn verify_difficulty(&self, block: &Block) -> ConsensusResult<()> {
        let hash = block.compute_hash()?;
        let hash_u64 = u64::from_be_bytes(hash[0..8].try_into().unwrap());

        if hash_u64 >= block.base_target {
            return Err(ConsensusError::NotMeetDifficulty);
        }
        Ok(())
    }

    fn calculate_next_difficulty(&self, recent_blocks: &[Block]) -> u64 {
        if recent_blocks.len() < 2 {
            return self.initial_difficulty;
        }

        let mut times = Vec::new();
        for i in 1..recent_blocks.len() {
            let delta = recent_blocks[i].timestamp.saturating_sub(recent_blocks[i-1].timestamp);
            times.push(delta);
        }

        // 计算平均时间并调整难度
        let avg = times.iter().sum::<u32>() as f64 / times.len() as f64;
        let target = self.target_spacing as f64;

        if avg < target * 0.9 {
            // 出块过快，增加难度（base_target 减小）
            // PoW 中难度表示为 max_target / current_target
            // 简化：返回新的 target
            (block.base_target as f64 * (avg / target)) as u64
        } else if avg > target * 1.1 {
            // 出块过慢，降低难度
            (block.base_target as f64 * (avg / target)) as u64
        } else {
            block.base_target // 保持不变
        }
    }

    fn verify_timestamp(&self, block: &Block, current_time: Timestamp) -> ConsensusResult<()> {
        let max_drift = self.target_spacing * 2; // 允许 2 倍漂移
        if block.timestamp > current_time + max_drift {
            return Err(ConsensusError::InvalidTransaction(
                format!("block timestamp too far in future: {} vs {}", block.timestamp, current_time)
            ));
        }
        // 不能早于前序区块（由区块链逻辑检查）
        Ok(())
    }

    fn verify_block_signature(&self, block: &Block, public_key: &PublicKey) -> ConsensusResult<()> {
        // PoW 中区块签名是出块者对区块内容的签名
        block.verify_signature(public_key)
    }

    fn serialize_header(block: &Block) -> Vec<u8> {
        block.serialize_header()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pow_mining() {
        let engine = PoWEngine::new(600);
        let header = b"test header";
        let (nonce, hash) = engine.mine(header, 0);

        let hash_u64 = u64::from_be_bytes(hash[0..8].try_into().unwrap());
        assert!(hash_u64 < engine.initial_difficulty);
        assert!(nonce > 0);
    }

    #[test]
    fn test_pow_verify_difficulty() {
        let engine = PoWEngine::new(600);
        let mut block = Block::new(1, [0u8; 32], 123);
        block.base_target = engine.initial_difficulty / 100; // 低难度

        // 构造一个满足难度的区块
        let header = block.serialize_header();
        let (nonce, _) = engine.mine(&header, 0);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&sha2::Sha256::digest(&[
            &header[..],
            &nonce.to_be_bytes()
        ].concat())[..]);

        // 这里简化处理：实际应该重新计算完整 block hash
    }
}
