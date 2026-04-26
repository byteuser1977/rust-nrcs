//! Block-related database models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use blockchain_types::{AccountId, Amount, BlockchainError, Height, Result, Timestamp};
use blockchain_types::prelude::*;

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct BlockModel {
    pub db_id: i64,
    pub id: i64,
    pub version: i32,
    pub timestamp: i32,
    pub previous_block_id: Option<i64>,
    pub total_amount: i64,
    pub total_fee: i64,
    pub payload_length: i32,
    pub previous_block_hash: Option<Vec<u8>>,
    pub cumulative_difficulty: Vec<u8>,
    pub base_target: i64,
    pub next_block_id: Option<i64>,
    pub height: i32,
    pub generation_signature: Vec<u8>,
    pub block_signature: Vec<u8>,
    pub payload_hash: Vec<u8>,
    pub generator_id: i64,
}

impl BlockModel {
    pub fn to_domain(&self) -> Result<Block> {
        let block = Block {
            id: Some(self.id as u64),
            version: self.version,
            timestamp: self.timestamp as Timestamp,
            height: self.height as Height,
            previous_block_id: self.previous_block_id.map(|id| id as u64),
            previous_block_hash: self
                .previous_block_hash
                .as_ref()
                .and_then(|h| h.as_slice().try_into().ok())
                .map(Hash256)
                .unwrap_or(Hash256([0u8; 32])),
            payload_hash: self.payload_hash.as_slice().try_into().map(Hash256).map_err(|_| {
                BlockchainError::InvalidHash("payload_hash length mismatch".to_string())
            })?,
            generator_id: Some(self.generator_id as AccountId),
            generator_public_key: None,
            nonce: 0,
            base_target: self.base_target as u64,
            cumulative_difficulty: self.cumulative_difficulty.clone(),
            total_amount: self.total_amount as Amount,
            total_fee: self.total_fee as Amount,
            payload_length: self.payload_length as u32,
            generation_signature: self.generation_signature.clone(),
            block_signature: self
                .block_signature
                .as_slice()
                .try_into()
                .map(Hash512)
                .unwrap_or(Hash512([0u8; 64])),
            transactions: vec![],
        };
        Ok(block)
    }

    pub fn from_domain(block: &Block) -> Result<Self> {
        let id = block.get_id() as i64;
        
        let previous_block_id = block.previous_block_id
            .filter(|&id| id != 0)
            .map(|id| id as i64);
        
        let previous_block_hash = if block.previous_block_hash.0 == [0u8; 32] {
            None
        } else {
            Some(block.previous_block_hash.0.to_vec())
        };
        
        Ok(Self {
            db_id: 0,
            id,
            version: block.version,
            timestamp: block.timestamp as i32,
            previous_block_id,
            total_amount: block.total_amount as i64,
            total_fee: block.total_fee as i64,
            payload_length: block.payload_length as i32,
            previous_block_hash,
            cumulative_difficulty: block.cumulative_difficulty.clone(),
            base_target: block.base_target as i64,
            next_block_id: None,
            height: block.height as i32,
            generation_signature: block.generation_signature.clone(),
            block_signature: block.block_signature.0.to_vec(),
            payload_hash: block.payload_hash.0.to_vec(),
            generator_id: block.get_generator_id() as i64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blockchain_types::{Hash256, Hash512, INITIAL_BASE_TARGET};
    use blockchain_types::prelude::Transaction;

    #[test]
    fn test_generator_id_conversion() {
        let unsigned_id: u64 = 18365787021584764528;
        let signed_id: i64 = unsigned_id as i64;
        
        assert_eq!(signed_id, -80957052124787088, "Generator ID should convert to signed value");
        
        let back_to_unsigned: u64 = signed_id as u64;
        assert_eq!(back_to_unsigned, unsigned_id, "Conversion back should be lossless");
    }

    #[test]
    fn test_block_model_from_domain_genesis() {
        let block = Block {
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
            transactions: vec![Transaction::default(), Transaction::default()],
        };

        let model = BlockModel::from_domain(&block).unwrap();
        
        assert_eq!(model.id, 3488276486778630462_i64, "Block ID should match");
        assert_eq!(model.generator_id, -80957052124787088_i64, "Generator ID should be signed value");
        assert_eq!(model.version, -1);
        assert_eq!(model.timestamp, 0);
        assert_eq!(model.height, 0);
        assert_eq!(model.total_amount, 100000000000000000_i64);
        assert_eq!(model.base_target, 153722867_i64);
    }

    #[test]
    fn test_block_model_roundtrip() {
        let original = Block {
            id: Some(3985281431710898053),
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
            transactions: vec![],
        };

        let model = BlockModel::from_domain(&original).unwrap();
        let restored = model.to_domain().unwrap();

        assert_eq!(restored.get_id(), original.get_id());
        assert_eq!(restored.get_generator_id(), original.get_generator_id());
        assert_eq!(restored.version, original.version);
        assert_eq!(restored.timestamp, original.timestamp);
        assert_eq!(restored.height, original.height);
        assert_eq!(restored.total_amount, original.total_amount);
        assert_eq!(restored.base_target, original.base_target);
    }
}
