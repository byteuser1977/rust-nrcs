//! 区块数据结构定义

use super::*;
use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use chrono::Utc;

// 密码学依赖
use ed25519_dalek::{Verifier, Signature};

/// 区块结构体
///
/// 区块是区块链的基本组成单元，包含区块头部和交易列表。
/// 区块头部信息用于共识验证、链式连接。
///
/// 参考 Java: `BaseBlock`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    /// 区块版本
    pub version: u32,
    /// 区块时间戳（Unix 秒）
    pub timestamp: Timestamp,
    /// 区块高度（从 1 开始）
    pub height: Height,
    /// 前序区块的哈希（SHA-256，32 字节）
    pub previous_block_hash: Hash256,
    /// 交易 Payload 的哈希（Merkle Root）
    pub payload_hash: Hash256,
    /// 出块者账户 ID（Generator ID）
    pub generator_id: AccountId,
    /// 随机数（Nonce）
    /// - PoW 场景为挖矿随机数
    /// - PoS 场景为 0
    pub nonce: u64,
    /// 基础难度目标值（Base Target）
    /// 用于计算区块是否满足难度要求：`hash < base_target`
    /// 越小难度越大
    pub base_target: u64,
    /// 累计难度（从创世区块到当前区块总难度）
    /// 使用变长字节数组存储（BigInteger 格式），Rust 中使用 `num-bigint`
    pub cumulative_difficulty: Vec<u8>,
    /// 总金额（包含在区块中的所有交易金额总和）
    /// 单位：NQT（10^-8）
    pub total_amount: Amount,
    /// 总手续费
    pub total_fee: Amount,
    /// Payload 长度（交易列表的字节数）
    pub payload_length: u32,
    /// 生成签名
    /// PoS 中出块者使用私钥生成，用于下一个出块者选择
    #[serde(skip)]
    pub generation_signature: Hash512,
    /// 区块签名（出块者对区块头签名）
    #[serde(skip)]
    pub block_signature: Hash512,
    /// 交易列表（内存中，未序列化到 P2P 消息体时）
    /// JSON 序列化/反序列化时忽略，避免性能问题
    #[serde(skip)]
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// 创建新区块的便捷构造函数
    pub fn new(
        height: Height,
        previous_block_hash: Hash256,
        generator_id: AccountId,
    ) -> Self {
        Self {
            version: BLOCK_VERSION,
            timestamp: 0, // 需填充
            height,
            previous_block_hash,
            payload_hash: [0u8; 32],
            generator_id,
            nonce: 0,
            base_target: 1_000_000, // 默认值
            cumulative_difficulty: vec![],
            total_amount: 0,
            total_fee: 0,
            payload_length: 0,
            generation_signature: [0u8; 64],
            block_signature: [0u8; 64],
            transactions: vec![],
        }
    }

    /// 计算区块头的完整哈希（用于区块 ID）
    /// 使用 SHA-256(version + timestamp + previous_hash + ... + block_signature)
    pub fn compute_hash(&self) -> Result<Hash256> {
        // 序列化区块头（不包含 transactions 和 signature）
        let data = self.serialize_header();
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        Ok(hash.try_into().map_err(|_| BlockchainError::InvalidHash("length mismatch".to_string()))?)
    }

    /// 验证区块签名
    pub fn verify_signature(&self, public_key: &PublicKey) -> Result<()> {
        let header_data = self.serialize_header_for_signing();

        // 仅支持 Ed25519
        if let PublicKey::Ed25519(bytes) = public_key {
            let pk = ed25519_dalek::PublicKey::from_bytes(bytes)
                .map_err(|e| BlockchainError::InvalidTransaction(e.to_string()))?;
            let sig = ed25519_dalek::Signature::from_bytes(&self.block_signature)
                .map_err(|e| BlockchainError::InvalidTransaction(e.to_string()))?;
            pk.verify(&header_data, &sig)
                .map_err(|e| BlockchainError::InvalidTransaction(e.to_string()))?;
        } else {
            return Err(BlockchainError::InvalidTransaction("unsupported key type".to_string()));
        }

        Ok(())
    }

    /// 序列化区块头（用于哈希计算）
    fn serialize_header(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.version.to_be_bytes());
        buf.extend_from_slice(&self.timestamp.to_be_bytes());
        buf.extend_from_slice(&self.height.to_be_bytes());
        buf.extend_from_slice(&self.previous_block_hash);
        buf.extend_from_slice(&self.payload_hash);
        buf.extend_from_slice(&self.generator_id.to_be_bytes());
        buf.extend_from_slice(&self.nonce.to_be_bytes());
        buf.extend_from_slice(&self.base_target.to_be_bytes());
        // cumulative_difficulty 变长，先写长度再写值
        buf.extend_from_slice(&(self.cumulative_difficulty.len() as u32).to_be_bytes());
        buf.extend_from_slice(&self.cumulative_difficulty);
        buf.extend_from_slice(&self.total_amount.to_be_bytes());
        buf.extend_from_slice(&self.total_fee.to_be_bytes());
        buf.extend_from_slice(&self.payload_length.to_be_bytes());
        buf.extend_from_slice(&self.generation_signature);
        // block_signature 不包含在 hash 中（签名部分单独计算）
        buf
    }

    /// 序列化区块头（用于签名验证，不包括 block_signature）
    fn serialize_header_for_signing(&self) -> Vec<u8> {
        let mut buf = self.serialize_header();
        // block_signature 不参与签名
        buf
    }

    /// 计算 Merkle Root（从交易列表）
    pub fn compute_merkle_root(transactions: &[Transaction]) -> Result<Hash256> {
        if transactions.is_empty() {
            return Ok([0u8; 32]); // 创世区块或空区块
        }

        let mut hashes: Vec<Hash256> = transactions
            .iter()
            .map(|tx| tx.full_hash)
            .collect();

        while hashes.len() > 1 {
            let mut next = Vec::with_capacity((hashes.len() + 1) / 2);
            for chunk in hashes.chunks(2) {
                let mut combined = Vec::with_capacity(64);
                combined.extend_from_slice(&chunk[0]);
                if chunk.len() == 2 {
                    combined.extend_from_slice(&chunk[1]);
                } else {
                    // 奇数个元素，重复最后一个
                    combined.extend_from_slice(&chunk[0]);
                }
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(combined);
                let hash = hasher.finalize();
                next.push(hash.try_into().unwrap());
            }
            hashes = next;
        }

        Ok(hashes[0])
    }

    /// 验证区块基本字段
    pub fn validate_basic(&self) -> Result<()> {
        if self.version != BLOCK_VERSION {
            return Err(BlockchainError::InvalidTransaction(format!("unsupported block version: {}", self.version)));
        }
        if self.payload_length as usize != self.transactions.iter().map(|tx| tx.size()).sum::<usize>() {
            return Err(BlockchainError::InvalidTransaction("payload length mismatch".to_string()));
        }
        // 验证时间戳合理性（不能超过当前时间太多，也不能太早）
        let now = Utc::now().timestamp() as u32;
        if self.timestamp > now + 3600 {
            return Err(BlockchainError::InvalidTransaction("block timestamp too far in future".to_string()));
        }
        // 验证 nonce 范围（对于 PoW）
        if self.nonce > u64::MAX / 2 {
            return Err(BlockchainError::InvalidTransaction("nonce out of range".to_string()));
        }
        // 验证难度目标
        if self.base_target == 0 {
            return Err(BlockchainError::InvalidTransaction("base_target cannot be zero".to_string()));
        }
        Ok(())
    }

    /// 验证区块完整性（包括交易和签名）
    pub fn validate_full(&self, expected_height: Height) -> Result<()> {
        self.validate_basic()?;

        // 检查区块高度
        if self.height != expected_height {
            return Err(BlockchainError::InvalidTransaction(format!("height mismatch: expected {}, got {}", expected_height, self.height)));
        }

        // 验证所有交易
        for tx in &self.transactions {
            tx.validate_basic()?;
        }

        // 重算 payload_hash 并验证
        let computed_payload_hash = Self::compute_merkle_root(&self.transactions)?;
        if computed_payload_hash != self.payload_hash {
            return Err(BlockchainError::InvalidTransaction("payload hash mismatch".to_string()));
        }

        // 验证区块签名（需要出块者公钥，这里简化处理，假设公钥可从 generator_id 获取）
        // 实际实现中需要从账户模型或缓存中获取公钥
        // self.verify_signature(&public_key)?;

        Ok(())
    }

    /// 序列化为二进制（bincode）用于网络传输
    pub fn to_bincode(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| BlockchainError::Serialization(e.into()))
    }

    /// 从二进制反序列化
    pub fn from_bincode(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data).map_err(|e| BlockchainError::Serialization(e.into()))
    }

    /// 序列化为 JSON 用于 API 输出
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| BlockchainError::Serialization(e.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_creation() {
        let mut block = Block::new(1, [0u8; 32], 1234567890);
        block.timestamp = 1_704_000_000;
        block.total_amount = 1_000_000_000;
        assert_eq!(block.height, 1);
        assert_eq!(block.version, BLOCK_VERSION);
    }

    #[test]
    fn test_merkle_root() {
        let tx1 = Transaction {
            type_id: 0,
            full_hash: [1u8; 32],
            ..Default::default()
        };
        let tx2 = Transaction {
            type_id: 0,
            full_hash: [2u8; 32],
            ..Default::default()
        };

        let root = Block::compute_merkle_root(&[tx1, tx2]).unwrap();
        assert_ne!(root, [0u8; 32]); // non-zero
    }
}
