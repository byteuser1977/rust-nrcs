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
