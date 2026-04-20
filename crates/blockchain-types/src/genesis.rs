//! Genesis block creation and initial state setup.

use super::*;
use crate::orm::models::{BlockModel, AccountModel};
use sqlx::PgPool;
use chrono::Utc;
use ed25519_dalek::Keypair;

/// Genesis configuration - can be loaded from external file
#[derive(Debug, Clone)]
pub struct GenesisConfig {
    pub timestamp: i64,
    pub generator_account_id: AccountId,
    pub generator_public_key: Vec<u8>,
    pub initial_accounts: Vec<(AccountId, i64)>, // (account_id, initial_balance)
    pub base_target: u64,
}

impl Default for GenesisConfig {
    fn default() -> Self {
        // Hardcoded genesis for testnet/mainnet
        Self {
            timestamp: Utc::now().timestamp(),
            generator_account_id: AccountId(1), // 创世账户
            generator_public_key: vec![0; 32], // 占位，需要替换为真实公钥
            initial_accounts: vec![
                (AccountId(1), 100_000_000_000_000), // 1 billion NQT 给创世账户
                (AccountId(2), 50_000_000_000_000),  // 预分配第二个账户
            ],
            base_target: 1_000_000,
        }
    }
}

/// Create the genesis block if the database is empty (height = 0).
pub async fn ensure_genesis(pool: &PgPool, config: GenesisConfig) -> sqlx::Result<()> {
    // Check if any block exists
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM block").fetch_one(pool).await?;
    if count.0 > 0 {
        return Ok(()); // Already initialized
    }

    info!(target: "genesis", "Creating genesis block...");

    let height = 1i32;
    let previous_hash = Hash256([0u8; 32]);
    let payload_hash = Hash256([0u8; 32]); // No transactions yet

    // Create dummy generation signature and block signature (in production, sign with private key)
    let generation_signature = Hash512([0u8; 64]);
    let block_signature = Hash512([0u8; 64]);

    let block_id = 1i64; // First block ID

    // Build BlockModel
    let block = BlockModel {
        db_id: None,
        id: block_id,
        version: 1,
        timestamp: config.timestamp,
        previous_block_id: None,
        total_amount: 0,
        total_fee: 0,
        payload_length: 0,
        previous_block_hash: Some(previous_hash.0.to_vec()),
        cumulative_difficulty: vec![], // TODO: compute proper difficulty encoding
        base_target: config.base_target as i64,
        next_block_id: None,
        height,
        generation_signature: generation_signature.0.to_vec(),
        block_signature: block_signature.0.to_vec(),
        payload_hash: payload_hash.0.to_vec(),
        generator_id: config.generator_account_id.0,
    };

    // Insert block
    sqlx::query!(
        r#"
        INSERT INTO block (
            id, version, timestamp, previous_block_id, total_amount,
            total_fee, payload_length, previous_block_hash, cumulative_difficulty,
            base_target, next_block_id, height, generation_signature,
            block_signature, payload_hash, generator_id
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16
        )
        "#,
        block.id,
        block.version,
        block.timestamp,
        block.previous_block_id,
        block.total_amount,
        block.total_fee,
        block.payload_length,
        block.previous_block_hash,
        block.cumulative_difficulty.as_slice(),
        block.base_target,
        block.next_block_id,
        block.height,
        block.generation_signature.as_slice(),
        block.block_signature.as_slice(),
        block.payload_hash.as_slice(),
        block.generator_id,
    )
    .execute(pool)
    .await?;

    // Create initial accounts with genesis balances
    for (account_id, balance) in config.initial_accounts {
        sqlx::query!(
            r#"
            INSERT INTO account (
                id, balance, unconfirmed_balance, forged_balance,
                active_lessee_id, has_control_phasing, height, latest
            ) VALUES ($1, $2, $2, 0, NULL, FALSE, $3, TRUE)
            "#,
            account_id.0,
            balance as i64,
            height
        )
        .execute(pool)
        .await?;

        // Also create account ledger entries for full balance
        sqlx::query!(
            r#"
            INSERT INTO account_ledger (
                account_id, event_type, event_id, holding_type, holding_id,
                "change", balance, block_id, height, timestamp
            ) VALUES ($1, 0, 0, 0, NULL, $2, $2, $3, $3, $3)
            "#,
            account_id.0,
            balance as i64,
            block_id
        )
        .execute(pool)
        .await?;
    }

    info!(target: "genesis", "Genesis block created at height {} with {} initial accounts", height, config.initial_accounts.len());
    Ok(())
}
