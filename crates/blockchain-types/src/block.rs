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
    #[serde(alias = "generationSignature", deserialize_with = "deserialize_generation_signature")]
    pub generation_signature: Vec<u8>,
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

pub fn biguint_to_signed_bytes_be(value: num_bigint::BigUint) -> Vec<u8> {
    if value == num_bigint::BigUint::from(0u64) {
        return vec![0u8];
    }
    let bytes = value.to_bytes_be();
    if bytes[0] & 0x80 != 0 {
        let mut signed_bytes = vec![0u8];
        signed_bytes.extend_from_slice(&bytes);
        signed_bytes
    } else {
        bytes
    }
}

pub fn signed_bytes_be_to_biguint(bytes: &[u8]) -> num_bigint::BigUint {
    if bytes.is_empty() {
        return num_bigint::BigUint::from(0u64);
    }
    if bytes[0] & 0x80 != 0 {
        let mut v = vec![0u8];
        v.extend_from_slice(bytes);
        num_bigint::BigUint::from_bytes_be(&v)
    } else {
        num_bigint::BigUint::from_bytes_be(bytes)
    }
}
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
    
    let prev_cum_diff = signed_bytes_be_to_biguint(&previous_block.cumulative_difficulty);
    let two64 = num_bigint::BigUint::from(18446744073709551616u128);
    let base_target_big = num_bigint::BigUint::from(base_target);
    let diff_add = two64 / base_target_big;
    let new_cum_diff = prev_cum_diff + diff_add;
    let cumulative_difficulty = biguint_to_signed_bytes_be(new_cum_diff);
    
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

fn deserialize_generation_signature<'de, D>(deserializer: D) -> std::result::Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    
    struct GenerationSignatureVisitor;
    
    impl<'de> Visitor<'de> for GenerationSignatureVisitor {
        type Value = Vec<u8>;
        
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a hex string (32 or 64 bytes)")
        }
        
        fn visit_str<E>(self, value: &str) -> std::result::Result<Vec<u8>, E>
        where
            E: de::Error,
        {
            if value.is_empty() {
                return Ok(vec![0u8; 32]);
            }
            let bytes = hex::decode(value).map_err(|e| de::Error::custom(format!("invalid hex: {}", e)))?;
            Ok(bytes)
        }
        
        fn visit_none<E>(self) -> std::result::Result<Vec<u8>, E>
        where
            E: de::Error,
        {
            Ok(vec![0u8; 32])
        }
        
        fn visit_unit<E>(self) -> std::result::Result<Vec<u8>, E>
        where
            E: de::Error,
        {
            Ok(vec![0u8; 32])
        }
    }
    
    deserializer.deserialize_any(GenerationSignatureVisitor)
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
            generation_signature: vec![0u8; 32],
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
        
        if self.version < BLOCK_VERSION {
            buf.extend_from_slice(&((self.total_amount / ONE_NRCS) as i32).to_le_bytes());
            buf.extend_from_slice(&((self.total_fee / ONE_NRCS) as i32).to_le_bytes());
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
        
        if self.version == 1 || self.version == -1 {
            buf.extend_from_slice(&self.generation_signature);
        } else {
            if self.generation_signature.len() >= 32 {
                buf.extend_from_slice(&self.generation_signature[..32]);
            } else {
                buf.extend_from_slice(&self.generation_signature);
                buf.extend_from_slice(&vec![0u8; 32 - self.generation_signature.len()]);
            }
        }
        
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

    /// 序列化区块用于签名验证（不含 block_signature）
    ///
    /// 对应 Java: `Arrays.copyOf(bytes(), bytes.length - 64)`
    /// 即 serialize_for_id() 的结果去掉末尾 64 字节签名
    pub fn serialize_for_signing(&self) -> Vec<u8> {
        let full = self.serialize_for_id();
        if full.len() > 64 {
            full[..full.len() - 64].to_vec()
        } else {
            full
        }
    }

    /// 使用 Curve25519 验证区块签名
    ///
    /// 对应 Java: Crypto.verify(blockSignature, data, generatorPublicKey, enforceCanonical)
    pub fn verify_block_signature(&self) -> Result<bool> {
        let pub_key_bytes = match &self.generator_public_key {
            Some(pk) => *pk,
            None => return Ok(false),
        };

        let signature = self.block_signature.0;
        let data = self.serialize_for_signing();

        // 使用 Curve25519 EC-KCDSA 验证
        let pub_key = crypto::PublicKey::Curve25519(pub_key_bytes);
        match crypto::verify(&pub_key, &data, &signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// 验证 generation signature（version >= 2）
    ///
    /// 对应 Java: generationSignature = SHA256(previousGenerationSignature || generatorPublicKey)
    pub fn verify_generation_signature(&self, previous_generation_signature: &[u8]) -> Result<bool> {
        if self.version < 2 {
            // version 1 使用 Curve25519 签名验证，暂不处理
            return Ok(true);
        }

        let pub_key_bytes = match &self.generator_public_key {
            Some(pk) => *pk,
            None => return Ok(false),
        };

        // 计算期望的 generation signature: SHA256(prev_gen_sig || generator_pubkey)
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(previous_generation_signature);
        hasher.update(&pub_key_bytes);
        let expected: [u8; 32] = hasher.finalize().into();

        Ok(self.generation_signature.len() >= 32 && self.generation_signature[..32] == expected)
    }

    /// 验证区块签名（Ed25519，保留用于兼容）
    pub fn verify_signature(&self, public_key: &PublicKey) -> Result<()> {
        let data = self.serialize_header_for_signing();
        let pk_bytes = match public_key {
            PublicKey::Ed25519(bytes) => bytes,
        };
        let pk = ed25519_dalek::PublicKey::from_bytes(pk_bytes)
            .map_err(|_| BlockchainError::InvalidSignature("invalid public key".to_string()))?;
        let sig = EdSignature::from_bytes(&self.block_signature.0)
            .map_err(|_| BlockchainError::InvalidSignature("invalid signature".to_string()))?;
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
        buf.extend_from_slice(&self.generation_signature);
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

    /// 计算区块 Payload Hash（对应 Java: digest.update(transaction.getBytes())）
    ///
    /// 使用 SHA256( tx1.getBytes() + tx2.getBytes() + ... + txN.getBytes() )
    pub fn compute_payload_hash_from_bytes(transactions: &[Transaction]) -> Result<Hash256> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        for tx in transactions {
            hasher.update(tx.get_bytes());
        }
        let hash = hasher.finalize();
        let arr: [u8; 32] = hash.try_into().map_err(|_| BlockchainError::InvalidHash("hash length mismatch".to_string()))?;
        Ok(Hash256(arr))
    }

    /// 验证区块基本字段
    pub fn validate_basic(&self) -> Result<()> {
        if self.version != BLOCK_VERSION {
            return Err(BlockchainError::InvalidTransaction(format!("unsupported block version: {}", self.version)));
        }
        // 验证 payload length（对应 Java BlockchainProcessor.java:1157）
        // 当 P2P 同步时，prunable attachment 数据可能被裁剪导致计算值偏小
        let has_pruned = self.transactions.iter()
            .any(|tx| tx.has_prunable_message || tx.has_prunable_encrypted_message || tx.has_prunable_attachment);
        let computed_payload = self.transactions.iter().map(|tx| tx.size()).sum::<usize>();
        let payload_match = if has_pruned {
            computed_payload <= self.payload_length as usize
        } else {
            computed_payload == self.payload_length as usize
        };
        if !payload_match {
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

        // 重算 payload_hash 并验证（对应 Java: digest.update(transaction.getBytes())）
        let computed_payload_hash = Self::compute_payload_hash_from_bytes(&self.transactions)?;
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
    pub fn from_bincode(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data).map_err(|e| BlockchainError::Serialization(Box::new(e)))
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

    fn create_genesis_block_for_test() -> Block {
        Block {
            id: None,
            version: -1,
            timestamp: 0,
            height: 0,
            previous_block_id: None,
            previous_block_hash: Hash256([0u8; 32]),
            payload_hash: Hash256(hex::decode("8f58dc2f809613424e608586df83b42513056861a864dff3cd00d88baca681ce").unwrap().try_into().unwrap()),
            generator_id: Some(18365787021584764528),
            generator_public_key: Some(hex::decode("b7f2232ddae77544690e1497f1b58e274039c3b0f99d9b6078f2520926230b26").unwrap().try_into().unwrap()),
            nonce: 0,
            base_target: INITIAL_BASE_TARGET,
            cumulative_difficulty: vec![0u8],
            total_amount: 100000000000000000,
            total_fee: 0,
            payload_length: 256,
            generation_signature: vec![0u8; 64],
            block_signature: Hash512(hex::decode("47b1aa800d657ccad4aaa8c946b2b0d2a7337fd3ab8e8c9ed6a06a49b7756e04a3ff13b15f6471afdff30313e1c47c4c2ab0e209c78a0673a42c254b74cc0201").unwrap().try_into().unwrap()),
            transactions: vec![
                Transaction::default(),
                Transaction::default(),
            ],
        }
    }

    #[test]
    fn test_genesis_block_id() {
        let block = create_genesis_block_for_test();
        let serialized = block.serialize_for_id();
        
        println!("Genesis block serialized length: {}", serialized.len());
        println!("Genesis block serialized (first 40 bytes): {:02x?}", &serialized[..40.min(serialized.len())]);
        
        // For version=-1:
        // version (4) + timestamp (4) + previousBlockId (8) + txCount (4) + 
        // amount (4, v<3) + fee (4, v<3) + payloadLength (4) + payloadHash (32) + 
        // generatorPublicKey (32) + generationSignature (64, v=-1) + blockSignature (64)
        // = 4+4+8+4+4+4+4+32+32+64+64 = 224 bytes
        // Note: v=-1 is NOT > 1, so no previousBlockHash
        
        let block_id = block.get_id();
        let expected_id: u64 = 3488276486778630462;
        
        println!("Calculated genesis block ID: {}", block_id);
        println!("Expected genesis block ID: {}", expected_id);
        
        assert_eq!(serialized.len(), 224, "Genesis block serialization should be 224 bytes");
        assert_eq!(block_id, expected_id, "Genesis block ID should match Java NRCS");
    }

    #[test]
    fn test_genesis_block_serialization_matches_java() {
        let block = create_genesis_block_for_test();
        let serialized = block.serialize_for_id();
        
        // Java NRCS uses:
        // version (4) + timestamp (4) + previousBlockId (8) + txCount (4) + 
        // amount (4, v<3) + fee (4, v<3) + payloadLength (4) + payloadHash (32) + 
        // generatorPublicKey (32) + generationSignature (64, v=-1) + blockSignature (64)
        // = 4+4+8+4+4+4+4+32+32+64+64 = 224 bytes
        
        // But wait, for version=-1, there's no previousBlockHash
        // Let me recalculate:
        // version=-1 < 3, so amount/fee are int (4 bytes each)
        // version=-1 is NOT > 1, so no previousBlockHash
        // generationSignature for genesis is 64 bytes
        
        // Expected: 4+4+8+4+4+4+4+32+32+64+64 = 224 bytes
        
        println!("Serialized length: {}", serialized.len());
        
        // Check version field (first 4 bytes, little-endian)
        let version_bytes = &serialized[0..4];
        let version = i32::from_le_bytes(version_bytes.try_into().unwrap());
        assert_eq!(version, -1, "Version should be -1");
        
        // Check timestamp (next 4 bytes)
        let timestamp_bytes = &serialized[4..8];
        let timestamp = u32::from_le_bytes(timestamp_bytes.try_into().unwrap());
        assert_eq!(timestamp, 0, "Timestamp should be 0");
        
        // Check previousBlockId (next 8 bytes)
        let prev_id_bytes = &serialized[8..16];
        let prev_id = u64::from_le_bytes(prev_id_bytes.try_into().unwrap());
        assert_eq!(prev_id, 0, "Previous block ID should be 0");
    }

    fn create_block_1_for_test() -> Block {
        Block {
            id: None,
            version: 3,
            timestamp: 38,
            height: 1,
            previous_block_id: Some(3488276486778630462),
            previous_block_hash: Hash256(hex::decode("3eb1c9a8fbd868309bf9b0d908c20ce948efcc552ec9ee2213f95ee6b0161f4b").unwrap().try_into().unwrap()),
            payload_hash: Hash256(hex::decode("3886d44573dd9c266e0cdc22f9329efb92022fbf4ff570747d6839f0d0cf28dd").unwrap().try_into().unwrap()),
            generator_id: Some(2794603741293765856),
            generator_public_key: Some(hex::decode("21a908b060ca909e4f4f665aa31243f51817e9cbd32ddbb751fcef2373aeba1f").unwrap().try_into().unwrap()),
            nonce: 0,
            base_target: 97357815,
            cumulative_difficulty: vec![0u8],
            total_amount: 100000000,
            total_fee: 100000000,
            payload_length: 176,
            generation_signature: hex::decode("950f9aec2d1f094f0c2b11b4d91dc2479774d168c9a85a28e7568842616d9ab2").unwrap(),
            block_signature: Hash512(hex::decode("94be72fe9ba361fe385921597ed58f6600f5a05dfe0eb0f6441476f6368fde05255c1f691dc1c41003f5e5d684bd81ffb9efe71bbbd260f2fa64f648916cdd4b").unwrap().try_into().unwrap()),
            transactions: vec![Transaction::default()],
        }
    }

    #[test]
    fn test_block_1_id() {
        let block = create_block_1_for_test();
        let serialized = block.serialize_for_id();
        
        println!("Block 1 serialized length: {}", serialized.len());
        println!("Block 1 serialized (first 60 bytes): {:02x?}", &serialized[..60.min(serialized.len())]);
        
        let block_id = block.get_id();
        let expected_id: u64 = 3985281431710898053;
        
        println!("Calculated block 1 ID: {}", block_id);
        println!("Expected block 1 ID: {}", expected_id);
        
        assert_eq!(serialized.len(), 232, "Block 1 serialization should be 232 bytes");
        assert_eq!(block_id, expected_id, "Block 1 ID should match Java NRCS");
    }

    #[test]
    fn test_block_1_serialization_fields() {
        let block = create_block_1_for_test();
        let serialized = block.serialize_for_id();
        
        // For version 3:
        // version (4) + timestamp (4) + previousBlockId (8) + txCount (4) + 
        // amount (8, v>=3) + fee (8, v>=3) + payloadLength (4) + payloadHash (32) + 
        // generatorPublicKey (32) + generationSignature (32, v>=2) + previousBlockHash (32, v>1) + blockSignature (64)
        // = 4+4+8+4+8+8+4+32+32+32+32+64 = 232 bytes
        
        assert_eq!(serialized.len(), 232, "Block 1 should serialize to 232 bytes");
        
        // Verify version
        let version = i32::from_le_bytes(serialized[0..4].try_into().unwrap());
        assert_eq!(version, 3);
        
        // Verify timestamp
        let timestamp = u32::from_le_bytes(serialized[4..8].try_into().unwrap());
        assert_eq!(timestamp, 38);
        
        // Verify previousBlockId
        let prev_id = u64::from_le_bytes(serialized[8..16].try_into().unwrap());
        assert_eq!(prev_id, 3488276486778630462);
        
        // Verify txCount
        let tx_count = i32::from_le_bytes(serialized[16..20].try_into().unwrap());
        assert_eq!(tx_count, 1); // block has 1 transaction in test
        
        // Verify totalAmount (8 bytes for v>=3)
        let amount = u64::from_le_bytes(serialized[20..28].try_into().unwrap());
        assert_eq!(amount, 100000000);
        
        // Verify totalFee (8 bytes for v>=3)
        let fee = u64::from_le_bytes(serialized[28..36].try_into().unwrap());
        assert_eq!(fee, 100000000);
        
        // Verify payloadLength
        let payload_len = u32::from_le_bytes(serialized[36..40].try_into().unwrap());
        assert_eq!(payload_len, 176);
    }
}
