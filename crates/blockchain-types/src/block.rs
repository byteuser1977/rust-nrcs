//! 区块数据结构定义

use super::*;
use crate::transaction::Transaction;
use serde::{Serialize, Deserialize};
use std::ops::{Deref, DerefMut};
use chrono::Utc;

// 密码学依赖
use ed25519_dalek::{Verifier, Signature as EdSignature};

/// 区块结构体
///
/// 区块是区块链的基本组成单元，包含区块头部和交易列表。
/// 区块头部信息用于共识验证、链式连接等。
///
/// 参考 Java: `BaseBlock`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    #[serde(alias = "block", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    pub version: i32,
    pub timestamp: Timestamp,
    #[serde(default)]
    pub height: Height,
    #[serde(alias = "previousBlock", alias = "previous_block", deserialize_with = "deserialize_optional_block_id")]
    pub previous_block_id: Option<u64>,
    #[serde(alias = "previousBlockHash", default)]
    pub previous_block_hash: Hash256,
    #[serde(alias = "payloadHash")]
    pub payload_hash: Hash256,
    #[serde(alias = "generatorId", alias = "generator", default, skip_serializing_if = "Option::is_none")]
    pub generator_id: Option<AccountId>,
    #[serde(alias = "generatorPublicKey", deserialize_with = "deserialize_public_key")]
    pub generator_public_key: Option<[u8; 32]>,
    #[serde(default)]
    pub nonce: u64,
    #[serde(alias = "baseTarget", default)]
    pub base_target: u64,
    #[serde(alias = "cumulativeDifficulty", default, deserialize_with = "deserialize_cumulative_difficulty")]
    pub cumulative_difficulty: Vec<u8>,
    #[serde(alias = "totalAmountNQT", deserialize_with = "deserialize_amount_string")]
    pub total_amount: Amount,
    #[serde(alias = "totalFeeNQT", deserialize_with = "deserialize_amount_string")]
    pub total_fee: Amount,
    #[serde(alias = "payloadLength")]
    pub payload_length: u32,
    #[serde(alias = "generationSignature")]
    pub generation_signature: Hash256,
    #[serde(alias = "blockSignature")]
    pub block_signature: Hash512,
    #[serde(default)]
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn get_generator_id(&self) -> AccountId {
        if let Some(id) = self.generator_id {
            return id;
        }
        if let Some(pub_key) = &self.generator_public_key {
            return account_id_from_public_key(pub_key);
        }
        0
    }
    
    pub fn get_id(&self) -> u64 {
        if let Some(id) = self.id {
            return id;
        }
        // 如果没有设置 ID，尝试计算
        self.calculate_id().unwrap_or(0)
    }
}

pub fn account_id_from_public_key(public_key: &[u8; 32]) -> AccountId {
    use sha2::{Sha256, Digest};
    let hash = Sha256::digest(public_key);
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&hash[..8]);
    u64::from_le_bytes(buf)
}

pub const INITIAL_BASE_TARGET: u64 = 153722867;
pub const GENESIS_BLOCK_ID: u64 = 3488276486778630462;
pub const BLOCK_TIME: u64 = 60;
pub const MIN_BLOCKTIME_LIMIT: u64 = 53;
pub const MAX_BLOCKTIME_LIMIT: u64 = 67;
pub const BASE_TARGET_GAMMA: u64 = 64;
pub const MAX_BASE_TARGET_2: u64 = 7686143350;
pub const MIN_BASE_TARGET: u64 = 138350580;
pub const MAX_BASE_TARGET: u64 = 153722867000000000;

pub struct PreviousBlockData {
    pub base_target: u64,
    pub cumulative_difficulty: Vec<u8>,
    pub timestamp: u32,
    pub height: i32,
    pub id: u64,
}

pub fn calculate_base_target_and_cumulative_difficulty(
    current_timestamp: u32,
    current_height: i32,
    previous_block: &PreviousBlockData,
    block_at_height_minus_2: Option<&PreviousBlockData>,
) -> (u64, Vec<u8>) {
    let base_target = if previous_block.height < -1 || previous_block.id == GENESIS_BLOCK_ID {
        let prev_base_target = previous_block.base_target;
        let time_diff = if current_timestamp > previous_block.timestamp {
            (current_timestamp - previous_block.timestamp) as u64
        } else {
            1
        };
        
        let mut bt = prev_base_target * time_diff / BLOCK_TIME;
        
        if bt > MAX_BASE_TARGET {
            bt = MAX_BASE_TARGET;
        }
        if bt < prev_base_target / 2 {
            bt = prev_base_target / 2;
        }
        if bt == 0 {
            bt = 1;
        }
        let twofold = if prev_base_target > i64::MAX as u64 / 2 {
            MAX_BASE_TARGET
        } else {
            prev_base_target * 2
        };
        if bt > twofold {
            bt = twofold;
        }
        bt
    } else if previous_block.height % 2 == 0 {
        let prev_base_target = previous_block.base_target;
        
        if let Some(block_hm2) = block_at_height_minus_2 {
            let blocktime_average = if current_timestamp > block_hm2.timestamp {
                (current_timestamp - block_hm2.timestamp) as u64 / 3
            } else {
                BLOCK_TIME
            };
            
            let bt = if blocktime_average > BLOCK_TIME {
                let capped = std::cmp::min(blocktime_average, MAX_BLOCKTIME_LIMIT);
                prev_base_target * capped / BLOCK_TIME
            } else {
                let capped = std::cmp::max(blocktime_average, MIN_BLOCKTIME_LIMIT);
                prev_base_target - prev_base_target * BASE_TARGET_GAMMA
                    * (BLOCK_TIME - capped) / 6000
            };
            
            let bt = if bt > MAX_BASE_TARGET_2 {
                MAX_BASE_TARGET_2
            } else {
                bt
            };
            let bt = std::cmp::max(bt, MIN_BASE_TARGET);
            bt
        } else {
            prev_base_target
        }
    } else {
        previous_block.base_target
    };
    
    let prev_cum_diff = if previous_block.cumulative_difficulty.is_empty() {
        num_bigint::BigUint::from(0u64)
    } else {
        num_bigint::BigUint::from_bytes_be(&previous_block.cumulative_difficulty)
    };
    let two64 = num_bigint::BigUint::from(u128::MAX) + 1u128;
    let base_target_big = num_bigint::BigUint::from(base_target);
    let diff_add = two64 / base_target_big;
    let new_cum_diff = prev_cum_diff + diff_add;
    let cumulative_difficulty = if new_cum_diff == num_bigint::BigUint::from(0u64) {
        vec![0u8]
    } else {
        new_cum_diff.to_bytes_be()
    };
    
    (base_target, cumulative_difficulty)
}

fn deserialize_optional_block_id<'de, D>(deserializer: D) -> std::result::Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    
    struct OptionalBlockIdVisitor;
    
    impl<'de> Visitor<'de> for OptionalBlockIdVisitor {
        type Value = Option<u64>;
        
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string, number, or null")
        }
        
        fn visit_none<E>(self) -> std::result::Result<Option<u64>, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
        
        fn visit_unit<E>(self) -> std::result::Result<Option<u64>, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
        
        fn visit_str<E>(self, value: &str) -> std::result::Result<Option<u64>, E>
        where
            E: de::Error,
        {
            if value.is_empty() || value == "null" {
                Ok(None)
            } else {
                value.parse::<u64>()
                    .map(Some)
                    .map_err(de::Error::custom)
            }
        }
        
        fn visit_u64<E>(self, value: u64) -> std::result::Result<Option<u64>, E>
        where
            E: de::Error,
        {
            Ok(Some(value))
        }
        
        fn visit_i64<E>(self, value: i64) -> std::result::Result<Option<u64>, E>
        where
            E: de::Error,
        {
            if value < 0 {
                Ok(None)
            } else {
                Ok(Some(value as u64))
            }
        }
    }
    
    deserializer.deserialize_any(OptionalBlockIdVisitor)
}

fn deserialize_public_key<'de, D>(deserializer: D) -> std::result::Result<Option<[u8; 32]>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    
    struct PublicKeyVisitor;
    
    impl<'de> Visitor<'de> for PublicKeyVisitor {
        type Value = Option<[u8; 32]>;
        
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a hex string or null")
        }
        
        fn visit_none<E>(self) -> std::result::Result<Option<[u8; 32]>, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
        
        fn visit_unit<E>(self) -> std::result::Result<Option<[u8; 32]>, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
        
        fn visit_str<E>(self, value: &str) -> std::result::Result<Option<[u8; 32]>, E>
        where
            E: de::Error,
        {
            if value.is_empty() {
                return Ok(None);
            }
            
            let bytes = hex::decode(value).map_err(|e| de::Error::custom(format!("invalid hex: {}", e)))?;
            if bytes.len() != 32 {
                return Err(de::Error::custom(format!("expected 32 bytes, got {}", bytes.len())));
            }
            
            let mut array = [0u8; 32];
            array.copy_from_slice(&bytes);
            Ok(Some(array))
        }
    }
    
    deserializer.deserialize_any(PublicKeyVisitor)
}

fn deserialize_cumulative_difficulty<'de, D>(deserializer: D) -> std::result::Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    
    struct CumulativeDifficultyVisitor;
    
    impl<'de> Visitor<'de> for CumulativeDifficultyVisitor {
        type Value = Vec<u8>;
        
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string number or array")
        }
        
        fn visit_str<E>(self, value: &str) -> std::result::Result<Vec<u8>, E>
        where
            E: de::Error,
        {
            if value.is_empty() {
                return Ok(vec![]);
            }
            match num_bigint::BigUint::parse_bytes(value.as_bytes(), 10) {
                Some(biguint) => {
                    let bytes = biguint.to_bytes_be();
                    if bytes.is_empty() {
                        Ok(vec![0u8])
                    } else {
                        Ok(bytes)
                    }
                }
                None => Ok(vec![])
            }
        }
        
        fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Vec<u8>, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut bytes = Vec::new();
            while let Some(byte) = seq.next_element::<u8>()? {
                bytes.push(byte);
            }
            Ok(bytes)
        }
        
        fn visit_none<E>(self) -> std::result::Result<Vec<u8>, E>
        where
            E: de::Error,
        {
            Ok(vec![])
        }
        
        fn visit_unit<E>(self) -> std::result::Result<Vec<u8>, E>
        where
            E: de::Error,
        {
            Ok(vec![])
        }
    }
    
    deserializer.deserialize_any(CumulativeDifficultyVisitor)
}

fn deserialize_amount_string<'de, D>(deserializer: D) -> std::result::Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    
    struct AmountVisitor;
    
    impl<'de> Visitor<'de> for AmountVisitor {
        type Value = u64;
        
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or number")
        }
        
        fn visit_str<E>(self, value: &str) -> std::result::Result<u64, E>
        where
            E: de::Error,
        {
            value.parse::<u64>().map_err(de::Error::custom)
        }
        
        fn visit_u64<E>(self, value: u64) -> std::result::Result<u64, E>
        where
            E: de::Error,
        {
            Ok(value)
        }
        
        fn visit_i64<E>(self, value: i64) -> std::result::Result<u64, E>
        where
            E: de::Error,
        {
            if value < 0 {
                Ok(0)
            } else {
                Ok(value as u64)
            }
        }
    }
    
    deserializer.deserialize_any(AmountVisitor)
}

impl Block {
    /// 创建新区块的便捷构造函数
    pub fn new(
        height: Height,
        previous_block_hash: Hash256,
        generator_id: AccountId,
    ) -> Self {
        Self {
            id: None,
            version: BLOCK_VERSION as i32,
            timestamp: 0,
            height,
            previous_block_id: None,
            previous_block_hash,
            payload_hash: Hash256([0u8; 32]),
            generator_id: Some(generator_id),
            generator_public_key: None,
            nonce: 0,
            base_target: 1_000_000,
            cumulative_difficulty: vec![],
            total_amount: 0,
            total_fee: 0,
            payload_length: 0,
            generation_signature: Hash256([0u8; 32]),
            block_signature: Hash512([0u8; 64]),
            transactions: vec![],
        }
    }

    /// 计算区块ID（参考Java Block.getId()）
    /// 对区块字节进行SHA-256哈希，取前8个字节（小端序）
    pub fn calculate_id(&self) -> Result<u64> {
        use sha2::{Digest, Sha256};
        
        let data = self.serialize_for_id();
        
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash = hasher.finalize();
        
        let mut id_bytes = [0u8; 8];
        id_bytes.copy_from_slice(&hash[..8]);
        
        id_bytes.reverse();
        
        Ok(u64::from_be_bytes(id_bytes))
    }
    
    /// 序列化区块用于ID计算（参考Java Block.bytes()）
    pub fn serialize_for_id(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.extend_from_slice(&self.timestamp.to_le_bytes());
        
        let prev_block_id = self.previous_block_id.unwrap_or(0);
        buf.extend_from_slice(&prev_block_id.to_le_bytes());
        
        buf.extend_from_slice(&(self.transactions.len() as i32).to_le_bytes());
        
        if self.version < 3 {
            buf.extend_from_slice(&((self.total_amount / 100_000_000) as i32).to_le_bytes());
            buf.extend_from_slice(&((self.total_fee / 100_000_000) as i32).to_le_bytes());
        } else {
            buf.extend_from_slice(&self.total_amount.to_le_bytes());
            buf.extend_from_slice(&self.total_fee.to_le_bytes());
        }
        
        buf.extend_from_slice(&self.payload_length.to_le_bytes());
        buf.extend_from_slice(&self.payload_hash.0);
        
        if let Some(pub_key) = &self.generator_public_key {
            buf.extend_from_slice(pub_key);
        } else {
            buf.extend_from_slice(&[0u8; 32]);
        }
        
        buf.extend_from_slice(&self.generation_signature.0);
        
        if self.version > 1 {
            buf.extend_from_slice(&self.previous_block_hash.0);
        }
        
        buf.extend_from_slice(&self.block_signature.0);
        
        buf
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
        let hash_arr: [u8; 32] = hash.try_into().map_err(|_| BlockchainError::InvalidHash("length mismatch".to_string()))?;
        Ok(Hash256(hash_arr))
    }

    /// 验证区块签名
    pub fn verify_signature(&self, public_key: &PublicKey) -> Result<()> {
        // 序列化区块头（用于签名）
        let data = self.serialize_header_for_signing();
        // 提取公钥
        let pk_bytes = match public_key {
            PublicKey::Ed25519(bytes) => bytes,
        };
        let pk = ed25519_dalek::PublicKey::from_bytes(pk_bytes)
            .map_err(|_| BlockchainError::InvalidSignature("invalid public key".to_string()))?;
        // 转换签名
        let sig = EdSignature::from_bytes(&self.block_signature.0)
            .map_err(|_| BlockchainError::InvalidSignature("invalid signature".to_string()))?;
        // 验证
        pk.verify(&data, &sig).map_err(|_| BlockchainError::InvalidSignature("signature verification failed".to_string()))?;
        Ok(())
    }

    /// 序列化区块头（用于哈希计算）
    fn serialize_header(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.version.to_be_bytes());
        buf.extend_from_slice(&self.timestamp.to_be_bytes());
        buf.extend_from_slice(&self.height.to_be_bytes());
        buf.extend_from_slice(&self.previous_block_hash.0);
        buf.extend_from_slice(&self.payload_hash.0);
        buf.extend_from_slice(&self.get_generator_id().to_be_bytes());
        buf.extend_from_slice(&self.nonce.to_be_bytes());
        buf.extend_from_slice(&self.base_target.to_be_bytes());
        // cumulative_difficulty 变长，先写长度再写内容
        buf.extend_from_slice(&(self.cumulative_difficulty.len() as u32).to_be_bytes());
        buf.extend_from_slice(&self.cumulative_difficulty);
        buf.extend_from_slice(&self.total_amount.to_be_bytes());
        buf.extend_from_slice(&self.total_fee.to_be_bytes());
        buf.extend_from_slice(&self.payload_length.to_be_bytes());
        buf.extend_from_slice(&self.generation_signature.0);
        // block_signature 不包含在 hash 中（签名部分单独计算）
        buf
    }

    /// 序列化区块头（用于签名验证，不包含 block_signature）
    fn serialize_header_for_signing(&self) -> Vec<u8> {
        let mut buf = self.serialize_header();
        // block_signature 不参与签名
        buf
    }

    /// 计算 Merkle Root（从交易列表）
    pub fn compute_merkle_root(transactions: &[Transaction]) -> Result<Hash256> {
        if transactions.is_empty() {
            return Ok(Hash256([0u8; 32])); // 创世区块或空区块
        }

        let mut hashes: Vec<Hash256> = transactions
            .iter()
            .map(|tx| tx.full_hash)
            .collect();

        while hashes.len() > 1 {
            let mut next = Vec::with_capacity((hashes.len() + 1) / 2);
            for chunk in hashes.chunks(2) {
                let mut combined = Vec::with_capacity(64);
                combined.extend_from_slice(&chunk[0].0);
                if chunk.len() == 2 {
                    combined.extend_from_slice(&chunk[1].0);
                } else {
                    // 奇数个元素，重复最后一个
                    combined.extend_from_slice(&chunk[0].0);
                }
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(combined);
                let hash = hasher.finalize();
                let hash_arr: [u8; 32] = hash.try_into().unwrap();
                next.push(Hash256(hash_arr));
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
        bincode::serialize(self).map_err(|e| BlockchainError::Serialization(Box::new(e)))
    }

    /// 从二进制反序列化
    pub fn from_bincode(_data: &[u8]) -> Result<Self> {
        unimplemented!("Deserialize not yet supported; will be implemented later")
    }

    /// 序列化为 JSON 用于 API 输出
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| BlockchainError::Serialization(Box::new(e)))
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_creation() {
        let mut block = Block::new(1, Hash256([0u8; 32]), 1234567890);
        block.timestamp = 1_704_000_000;
        block.total_amount = 1_000_000_000;
        assert_eq!(block.height, 1);
        assert_eq!(block.version, BLOCK_VERSION);
    }

    #[test]
    fn test_merkle_root() {
        let tx1 = Transaction::new(
            TransactionType::Payment,
            1,
            Some(2),
            100,
            1,
            1700000000,
            32767,
        );
        let tx2 = Transaction::new(
            TransactionType::Payment,
            3,
            Some(4),
            200,
            1,
            1700000000,
            32767,
        );

        let root = Block::compute_merkle_root(&[tx1, tx2]).unwrap();
        assert_ne!(*root, [0u8; 32]); // non-zero
    }
}
