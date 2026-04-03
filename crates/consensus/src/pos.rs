//! PoS (Proof of Stake) 共识引擎实现
//!
//! 参考 NRCS 原版 Java 实现，基于账户余额（有效余额）和 `generation_signature` 进行出块者选择。

use super::*;
use blockchain_types::*;
use num_bigint::BigUint;
use num_traits::{Zero, One, ToPrimitive};
use rand::Rng;

/// PoS 共识引擎
pub struct PoSEngine {
    /// 目标区块间隔（秒）
    pub target_spacing: u32,
    /// 最小出块余额要求（NQT）
    pub minimum_balance: Amount,
    /// 出块奖励（NQT）
    pub block_reward: Amount,
    /// 最多租赁次数上限
    pub max_lease_terms: u32,
}

impl PoSEngine {
    pub fn new(target_spacing: u32, minimum_balance: Amount, block_reward: Amount) -> Self {
        Self {
            target_spacing,
            minimum_balance,
            block_reward,
            max_lease_terms: 720, // 按默认每块15秒，720块 ≈ 3 小时
        }
    }

    /// 计算下一个出块者（Forger Selection）
    ///
    /// 算法步骤：
    /// 1. 筛选有效平衡 ≥ minimum_balance 的账户
    /// 2. 如果有多个，根据 `generation_signature` 和当前时间生成随机数
    /// 3. 扫描链中候选者，选择累计难度满足随机数的账户
    fn select_forger_internal(
        &self,
        state: &BlockchainState,
        timestamp: Timestamp,
    ) -> ConsensusResult<(AccountId, Hash512)> {
        let candidates: Vec<&AccountSnapshot> = state.accounts.iter()
            .filter(|a| {
                // 有效余额 ≥ minimum_balance
                a.balance >= self.minimum_balance
            })
            .collect();

        if candidates.is_empty() {
            return Err(ConsensusError::NoValidForger);
        }

        // 1. 计算总有效难度
        // 注意：NRCS 使用 cumulative_difficulty 格式（BigInteger 变长字节）
        let total_effective = candidates.iter()
            .map(|a| BigUint::from(a.balance))
            .sum::<BigUint>();

        // 2. 从上一区块的 generation_signature 和当前时间生成随机数
        // 算法示例：rand = (gen_sig + timestamp) % total_effective
        let mut rand_input = Vec::from(&state.last_generation_signature[..]);
        rand_input.extend_from_slice(&timestamp.to_be_bytes());

        // 简化：使用 SHA-256 生成随机数
        let rand_hash = sha2::Sha256::digest(&rand_input);
        let mut rng_num = BigUint::from_bytes_be(&rand_hash);
        let remainder = rng_num % &total_effective;

        // 3. 扫描候选者
        let mut cumulative = BigUint::zero();
        for candidate in candidates {
            let hit = cumulative + BigUint::from(candidate.balance);
            if &remainder < &hit {
                // 选中该账户
                let gen_sig = self.generate_signature(candidate.id, &state.last_generation_signature);
                return Ok((candidate.id, gen_sig));
            }
            cumulative = hit;
        }

        // 没找到（理论上不应发生）
        Err(ConsensusError::NoValidForger)
    }
}

impl ConsensusEngine for PoSEngine {
    fn verify_difficulty(&self, block: &Block) -> ConsensusResult<()> {
        // PoS 不验证传统难度，改为验证出块时间在 deadline 内
        // 具体逻辑在 PoS 中放在 select_forger 和 deadline 验证
        Ok(())
    }

    fn calculate_next_difficulty(&self, _recent_blocks: &[Block]) -> u64 {
        // PoS 不需要动态调整难度（固定 base_target）
        // 但需要根据总质押量调整难度（可选）
        1_000_000
    }

    fn verify_timestamp(&self, block: &Block, current_time: Timestamp) -> ConsensusResult<()> {
        let max_drift = self.target_spacing * 4;
        if block.timestamp > current_time + max_drift {
            return Err(ConsensusError::InvalidTransaction(
                format!("block timestamp too far in future")
            ));
        }
        Ok(())
    }

    fn verify_block_signature(&self, block: &Block, public_key: &PublicKey) -> ConsensusResult<()> {
        block.verify_signature(public_key)
    }

    fn serialize_header(block: &Block) -> Vec<u8> {
        block.serialize_header()
    }
}

impl PoSEngine for PoSEngine {
    fn select_forger(
        &self,
        blockchain: &BlockchainState,
        timestamp: Timestamp,
    ) -> ConsensusResult<(AccountId, Hash512)> {
        // 检查时间窗口
        if timestamp < blockchain.last_timestamp + self.target_spacing as u32 {
            let remaining = blockchain.last_timestamp + self.target_spacing as u32 - timestamp;
            return Err(ConsensusError::DeadlineExpired { remaining: remaining as u64 });
        }

        self.select_forger_internal(blockchain, timestamp)
    }

    fn calculate_deadline(&self, account_id: AccountId, at_height: Height) -> ConsensusResult<u64> {
        // 简化计算：deadline = (height % 1440) * target_spacing
        // 原始算法更复杂，涉及生成签名
        let base = ((at_height % 1440) as u64).saturating_mul(self.target_spacing as u64);
        Ok(base)
    }

    fn generate_signature(&self, account_id: AccountId, prev_gen_sig: &Hash512) -> Hash512 {
        // 生成签名使用私有密钥（应调用者提供）
        // 这里仅返回占位符
        let mut gen_sig = *prev_gen_sig;
        // 翻转让签名不重复
        gen_sig.reverse();
        gen_sig
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_engine_creation() {
        let engine = PoSEngine::new(15, 500_000_000_000, 150_000_000_000);
        assert_eq!(engine.target_spacing, 15);
        assert_eq!(engine.minimum_balance, 500_000_000_000); // 5000 NRC
        assert_eq!(engine.block_reward, 150_000_000_000);    // 1500 NRC
    }

    #[test]
    fn test_deadline_calculation() {
        let engine = PoSEngine::new(15, 1, 1);
        let deadline = engine.calculate_deadline(123, 100).unwrap();
        // height = 100, 100 % 1440 = 100, deadline = 100 * 15 = 1500
        assert_eq!(deadline, 1500);
    }
}
