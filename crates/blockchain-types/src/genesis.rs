//! Genesis block creation and initial state setup.

use crate::block::Block;
use crate::account::Account;
use crate::{AccountId, Hash256, Hash512, Amount, Timestamp, Height};
use chrono::Utc;

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
            generator_account_id: 1, // 创世账户
            generator_public_key: vec![0; 32], // 占位，需要替换为真实公钥
            initial_accounts: vec![
                (1, 100_000_000_000_000), // 1 billion NQT 给创世账户
                (2, 50_000_000_000_000),  // 预分配第二个账户
            ],
            base_target: 1_000_000,
        }
    }
}

/// Create the genesis block and initial accounts.
/// Returns the genesis block and a list of initial accounts.
pub fn create_genesis(config: GenesisConfig) -> (Block, Vec<Account>) {
    let height: Height = 1;
    let previous_hash = Hash256([0u8; 32]);
    let payload_hash = Hash256([0u8; 32]); // No transactions yet

    // Create dummy generation signature and block signature (in production, sign with private key)
    let generation_signature = Hash256([0u8; 32]);
    let block_signature = Hash512([0u8; 64]);

    // Build Block
    let block = Block {
        version: 1,
        timestamp: config.timestamp as Timestamp,
        height,
        previous_block_id: 0,
        previous_block_hash: previous_hash,
        payload_hash,
        generator_id: config.generator_account_id,
        generator_public_key: Some([0u8; 32]),
        nonce: 0,
        base_target: config.base_target,
        cumulative_difficulty: vec![],
        total_amount: 0,
        total_fee: 0,
        payload_length: 0,
        generation_signature,
        block_signature,
        transactions: vec![],
    };

    // Create initial accounts with genesis balances
    let accounts: Vec<Account> = config
        .initial_accounts
        .into_iter()
        .map(|(account_id, balance)| {
            Account::new(account_id, balance as Amount)
        })
        .collect();

    (block, accounts)
}

/// Default genesis block for testing
pub fn default_genesis() -> (Block, Vec<Account>) {
    create_genesis(GenesisConfig::default())
}
