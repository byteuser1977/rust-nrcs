//! # Consensus Layer
//!
//! 共识算法模块，提供 PoW（工作量证明）和 PoS（权益证明）实现。
//!
//! 该模块设计为 trait 化，便于未来扩展其他共识算法。
//!
//! ## PoS 算法说明
//! 参考 NRCS 原版 Java 实现，PoS 核心逻辑：
//! 1. 根据 `generation_signature` 和当前时间戳计算随机数
//! 2. 扫描链上候选出块者（账户余额 > 0）
//! 3. 累计难度 `cumulative_difficulty` 选择下个出块者
//! 4. 计算目标时间（deadline），在时间窗口内出块得到奖励

pub mod pow;
pub mod pos;

pub mod prelude {
    pub use crate::{Consensus, ConsensusEngine, PoSEngine};
}

use super::blockchain_types::*;
use thiserror::Error;

/// 共识错误类型
#[derive(Debug, Error)]
pub enum ConsensusError {
    #[error("invalid difficulty: {0}")]
    InvalidDifficulty(String),

    #[error("block not meet difficulty requirement")]
    NotMeetDifficulty,

    #[error("no valid forger selected")]
    NoValidForger,

    #[error("deadline expired: expected in {remaining} seconds")]
    DeadlineExpired { remaining: u64 },

    #[error("invalid block signature")]
    InvalidSignature,
}

pub type ConsensusResult<T> = std::result::Result<T, ConsensusError>;

/// 共识引擎 trait
///
/// 定义了区块链共识的核心操作，适用 PoW、PoS 或其他算法。
pub trait ConsensusEngine: Send + Sync {
    /// 验证区块是否满足难度要求
    fn verify_difficulty(&self, block: &Block) -> ConsensusResult<()>;

    /// 计算/调整下一个区块的难度（目标值）
    /// 基于最近 N 个区块的时间戳和难度
    fn calculate_next_difficulty(&self, recent_blocks: &[Block]) -> u64;

    /// 验证区块时间有效性（不早于上一区块，不晚于当前时间+容差）
    fn verify_timestamp(&self, block: &Block, current_time: Timestamp) -> ConsensusResult<()>;

    /// 验证区块签名（对应出块者）
    fn verify_block_signature(&self, block: &Block, public_key: &PublicKey) -> ConsensusResult<()>;

    /// 序列化区块头用于哈希计算
    fn serialize_header(block: &Block) -> Vec<u8>;
}

/// PoS 共识引擎扩展 trait
///
/// PoS 独有的接口：选块者、deadline 计算、生成签名。
pub trait PoSEngine: ConsensusEngine {
    /// 选择下一个出块者（Forger）
    ///
    /// 根据链上状态（高度，候选账户）计算下一个出块者
    /// 返回：出块者 AccountId 和对应的签名 `generation_signature`
    fn select_forger(
        &self,
        blockchain: &BlockchainState,
        timestamp: Timestamp,
    ) -> ConsensusResult<(AccountId, Hash512)>;

    /// 计算指定账户的 deadline（距离可出块的时间）
    /// 返回：deadline（秒），0 表示可立即出块
    fn calculate_deadline(&self, account_id: AccountId, at_height: Height) -> ConsensusResult<u64>;

    /// 生成出块签名（用于区块头部）
    /// 输入：上一区块的 generation_signature，当前账户私钥
    fn generate_signature(&self, account_id: AccountId, prev_gen_sig: &Hash512) -> Hash512;
}

/// 区块链状态只读视图（供共识引擎使用）
///
/// 只暴露共识所需的最小集合，避免循环依赖。
pub struct BlockchainState {
    /// 当前高度
    pub height: Height,
    /// 最新区块哈希
    pub last_block_hash: Hash256,
    /// 最新区块基值
    pub last_base_target: u64,
    /// 累计难度
    pub cumulative_difficulty: Vec<u8>,
    /// 出块者签名
    pub last_generation_signature: Hash512,
    /// 时间戳
    pub last_timestamp: Timestamp,
    /// 账户快照（仅包含余额和租赁信息，用于选块）
    pub accounts: Vec<AccountSnapshot>,
}

/// 账户快照（只读）
pub struct AccountSnapshot {
    pub id: AccountId,
    pub balance: Amount,
    pub lease: Option<AccountLease>,
    pub has_public_key: bool,
}

impl BlockchainState {
    pub fn new(
        height: Height,
        last_block_hash: Hash256,
        last_base_target: u64,
        cumulative_difficulty: Vec<u8>,
        last_generation_signature: Hash512,
        last_timestamp: Timestamp,
        accounts: Vec<AccountSnapshot>,
    ) -> Self {
        Self {
            height,
            last_block_hash,
            last_base_target,
            cumulative_difficulty,
            last_generation_signature,
            last_timestamp,
            accounts,
        }
    }

    /// 获取有效余额总和（总抵押）
    pub fn total_effective_balance(&self) -> u64 {
        self.accounts.iter()
            .map(|a| a.balance)
            .sum()
    }
}

/// 难度调整参数
pub struct DifficultyAdjustmentParams {
    /// 调整周期（每 N 个区块调整一次）
    pub interval: u32,
    /// 目标区块间隔（秒）
    pub target_spacing: u32,
    /// 调整幅度限制（±%）
    pub max_adjustment: f64,
}

impl Default for DifficultyAdjustmentParams {
    fn default() -> Self {
        Self {
            interval: 10,    // 每 10 个区块调整
            target_spacing: 15, // 15 秒出块
            max_adjustment: 0.5, // ±50%
        }
    }
}

/// 计算难度调整公式（来自 NRCS PoS）
///
/// - 如果平均区块时间 < target_spacing * 0.9，降低难度
/// - 如果平均区块时间 > target_spacing * 1.1，增加难度
/// - 否则不变
pub fn adjust_difficulty(
    current_target: u64,
    actual_times: &[u32], // 最近 N 个区块时间差
    params: &DifficultyAdjustmentParams,
) -> u64 {
    if actual_times.len() < 2 {
        return current_target;
    }

    let sum: u64 = actual_times.iter().sum();
    let avg = sum as f64 / actual_times.len() as f64;
    let target = params.target_spacing as f64;

    let ratio = avg / target;

    // ratio < 1 表示出块过快，应增加难度（base_target 增大）
    // ratio > 1 表示出块过慢，应降低难度（base_target 减小）
    let adjustment = if ratio < 0.9 {
        // 需要降低难度（base_target 乘以 factor < 1）
        ratio
    } else if ratio > 1.1 {
        // 需要增加难度（base_target 乘以 factor > 1）
        ratio
    } else {
        return current_target; // 在容差范围内，不调整
    };

    let new_target = (current_target as f64 * adjustment) as u64;

    // 限制调整幅度
    let max_factor = 1.0 + params.max_adjustment;
    let min_factor = 1.0 - params.max_adjustment;

    if new_target > current_target {
        let limited = (current_target as f64 * max_factor) as u64;
        new_target.min(limited)
    } else {
        let limited = (current_target as f64 * min_factor) as u64;
        new_target.max(limited)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_adjustment() {
        let params = DifficultyAdjustmentParams::default();
        let current = 1_000_000;

        // 区块间隔过快（12秒 vs 15秒 target）
        let faster = adjust_difficulty(current, &[12, 13, 12], &params);
        assert!(faster > current); // 增加难度

        // 区块间隔过慢（20秒）
        let slower = adjust_difficulty(current, &[20, 22, 21], &params);
        assert!(slower < current); // 降低难度

        // 正常（14-16秒）
        let normal = adjust_difficulty(current, &[14, 15, 16], &params);
        assert_eq!(normal, current);
    }
}
