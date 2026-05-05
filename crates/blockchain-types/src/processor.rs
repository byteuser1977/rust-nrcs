//! Blockchain state processor for blocks and transactions.
//!
//! ⚠️ **已废弃**: 此模块包含硬编码 SQL，违反架构规范（核心层不应有数据库依赖）
//! 实际区块处理逻辑已迁移至 p2p/src/verifier.rs，使用 ORM Repository 访问数据库
//! 此模块仅保留作为参考，不应在新代码中使用
//!
//! 原因：
//! 1. blockchain-types 是核心层模块，按依赖规则不应直接访问数据库
//! 2. SQL 使用小写表名/列名，与 NRCS Schema 大写格式不一致
//! 3. INSERT 语句中存在疑似逻辑错误（$1 绑定 block.height 而非 block.id）
//! 4. 功能已被 p2p::BlockchainVerifier 完全替代

// 以下代码已屏蔽，功能已迁移至 p2p/src/verifier.rs
// 如需参考原始实现，请查看 git 历史

// #[cfg(feature = "database")]
// use crate::prelude::*;
// #[cfg(feature = "database")]
// use crate::block::Block;
// #[cfg(feature = "database")]
// use crate::transaction::Transaction;
// #[cfg(feature = "database")]
// use sqlx::PgPool;
// #[cfg(feature = "database")]
// use tracing::{info};
//
// #[cfg(feature = "database")]
// pub struct BlockchainProcessor {
//     pool: PgPool,
// }
//
// #[cfg(feature = "database")]
// impl BlockchainProcessor {
//     pub fn new(pool: PgPool) -> Self {
//         Self { pool }
//     }
//
//     pub async fn process_block(&self, block: &Block) -> Result<Height> {
//         self.validate_basic(block)?;
//         let prev_height = block.height.checked_sub(1).ok_or_else(||
//             BlockchainError::InvalidTransaction("block height underflow".to_string())
//         )?;
//         if prev_height > 0 {
//             let prev_exists = self.block_exists(prev_height).await?;
//             if !prev_exists {
//                 return Err(BlockchainError::InvalidBlock(format!(
//                     "previous block at height {} not found", prev_height
//                 )));
//             }
//         }
//         let computed_payload_hash = self.compute_payload_hash(&block.transactions)?;
//         if computed_payload_hash != block.payload_hash {
//             return Err(BlockchainError::InvalidBlock(
//                 "payload hash mismatch".to_string()
//             ));
//         }
//         self.insert_block(block).await?;
//         info!(target: "blockchain", "Accepted block: height={}, id={}, generator={}, txs={}",
//               block.height, block.compute_hash()?, block.get_generator_id(), block.transactions.len());
//         Ok(block.height)
//     }
//
//     fn validate_basic(&self, block: &Block) -> Result<()> {
//         if block.version != BLOCK_VERSION {
//             return Err(BlockchainError::InvalidBlock(
//                 format!("unsupported block version: {}", block.version)
//             ));
//         }
//         if block.height == 0 {
//             return Err(BlockchainError::InvalidBlock("height cannot be zero".to_string()));
//         }
//         Ok(())
//     }
//
//     async fn block_exists(&self, height: Height) -> Result<bool> {
//         let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM block WHERE height = $1")
//             .bind(height as i32)
//             .fetch_one(&self.pool)
//             .await
//             .map_err(|e| BlockchainError::Database(e.to_string()))?;
//         Ok(count.0 > 0)
//     }
//
//     fn compute_payload_hash(&self, txs: &[Transaction]) -> Result<Hash256> {
//         use sha2::{Digest, Sha256};
//         let mut hasher = Sha256::new();
//         for tx in txs {
//             hasher.update(tx.get_bytes());
//         }
//         let hash = hasher.finalize();
//         let arr: [u8; 32] = hash.try_into().unwrap();
//         Ok(Hash256(arr))
//     }
//
//     async fn insert_block(&self, block: &Block) -> Result<()> {
//         let previous_block_hash = Some(block.previous_block_hash.0.to_vec());
//         let cumulative_difficulty = block.cumulative_difficulty.clone();
//         let generation_signature = block.generation_signature.clone();
//         let block_signature = block.block_signature.0.to_vec();
//         let payload_hash = block.payload_hash.0.to_vec();
//         sqlx::query(
//             r#"
//             INSERT INTO block (
//                 id, version, timestamp, previous_block_id, total_amount,
//                 total_fee, payload_length, previous_block_hash, cumulative_difficulty,
//                 base_target, next_block_id, height, generation_signature,
//                 block_signature, payload_hash, generator_id
//             ) VALUES (
//                 $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16
//             )
//             "#
//         )
//         .bind(block.height as i64)
//         .bind(block.version as i32)
//         .bind(block.timestamp as i32)
//         .bind(None::<i64>)
//         .bind(block.total_amount as i64)
//         .bind(block.total_fee as i64)
//         .bind(block.payload_length as i32)
//         .bind(previous_block_hash.as_deref())
//         .bind(cumulative_difficulty.as_slice())
//         .bind(block.base_target as i64)
//         .bind(None::<i64>)
//         .bind(block.height as i32)
//         .bind(generation_signature.as_slice())
//         .bind(block_signature.as_slice())
//         .bind(payload_hash.as_slice())
//         .bind(block.get_generator_id() as i64)
//         .execute(&self.pool)
//         .await
//         .map_err(|e| BlockchainError::Database(e.to_string()))?;
//         Ok(())
//     }
// }
