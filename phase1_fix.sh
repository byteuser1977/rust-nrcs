#!/usr/bin/env bash
set -e
cd /mnt/d/workspace/git/rust-nrcs

echo 'Fixing account.rs (8 fields)...'
cat > crates/blockchain-types/src/account.rs <<'ACCOUNT'
//! 账户数据结构（严格对齐 ORM AccountModel）
use super::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub id: AccountId,
    pub balance: Amount,
    pub unconfirmed_balance: Amount,
    pub forged_balance: Amount,
    pub active_lessee_id: Option<AccountId>,
    pub has_control_phasing: bool,
    pub height: Height,
    pub latest: bool,
}

impl Account {
    pub fn new(id: AccountId, initial_balance: Amount) -> Self {
        Self {
            id,
            balance: initial_balance,
            unconfirmed_balance: initial_balance,
            forged_balance: 0,
            active_lessee_id: None,
            has_control_phasing: false,
            height: 0,
            latest: true,
        }
    }
    pub fn effective_balance(&self) -> Amount { self.balance }
    pub fn has_balance(&self, required: Amount) -> bool { self.balance >= required }
    pub fn credit(&mut self, amount: Amount) {
        self.balance = self.balance.saturating_add(amount);
        self.unconfirmed_balance = self.unconfirmed_balance.saturating_add(amount);
    }
    pub fn debit(&mut self, amount: Amount) -> Result<()> {
        if !self.has_balance(amount) {
            return Err(BlockchainError::InsufficientBalance { have: self.balance, need: amount });
        }
        self.balance = self.balance.saturating_sub(amount);
        self.unconfirmed_balance = self.unconfirmed_balance.saturating_sub(amount);
        Ok(())
    }
}
ACCOUNT

echo 'Fixing block.rs (16 fields, Serialize only, strict to BLOCK table)...'
cat > crates/blockchain-types/src/block.rs <<'BLOCK'
//! 区块数据结构（严格对齐 BLOCK 表）
use super::*;
use serde::Serialize;
use chrono::Utc;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Block {
    pub version: u32,
    pub timestamp: Timestamp,
    pub height: Height,
    pub previous_block_hash: Hash256,
    pub payload_hash: Hash256,
    pub generator_id: AccountId,
    pub base_target: u64,
    pub cumulative_difficulty: Vec<u8>,
    pub total_amount: Amount,
    pub total_fee: Amount,
    pub payload_length: u32,
    pub id: i64,
    #[serde(skip)]
    pub generation_signature: Option<Hash512>,
    #[serde(skip)]
    pub block_signature: Option<Hash512>,
    pub previous_block_id: Option<i64>,
    pub next_block_id: Option<i64>,
}

impl Block {
    pub fn new(height: Height, previous_block_hash: Hash256, generator_id: AccountId) -> Self {
        Self {
            version: super::BLOCK_VERSION,
            timestamp: 0,
            height,
            previous_block_hash,
            payload_hash: [0u8; 32],
            generator_id,
            base_target: 1_000_000,
            cumulative_difficulty: vec![],
            total_amount: 0,
            total_fee: 0,
            payload_length: 0,
            id: 0,
            generation_signature: None,
            block_signature: None,
            previous_block_id: None,
            next_block_id: None,
        }
    }

    pub fn compute_hash(&self) -> Result<Hash256> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.serialize_header());
        Ok(hasher.finalize().try_into().unwrap())
    }

    fn serialize_header(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.version.to_be_bytes());
        buf.extend_from_slice(&self.timestamp.to_be_bytes());
        buf.extend_from_slice(&self.height.to_be_bytes());
        buf.extend_from_slice(&self.previous_block_hash);
        buf.extend_from_slice(&self.payload_hash);
        buf.extend_from_slice(&self.generator_id.to_be_bytes());
        buf.extend_from_slice(&self.base_target.to_be_bytes());
        buf.extend_from_slice(&(self.cumulative_difficulty.len() as u32).to_be_bytes());
        buf.extend_from_slice(&self.cumulative_difficulty);
        buf.extend_from_slice(&self.total_amount.to_be_bytes());
        buf.extend_from_slice(&self.total_fee.to_be_bytes());
        buf.extend_from_slice(&self.payload_length.to_be_bytes());
        buf
    }

    pub fn validate_basic(&self) -> Result<()> {
        if self.version != super::BLOCK_VERSION {
            return Err(BlockchainError::InvalidTransaction(format!("unsupported block version: {}", self.version)));
        }
        if self.base_target == 0 {
            return Err(BlockchainError::InvalidTransaction("base_target cannot be zero".into()));
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| BlockchainError::Serialization(Box::new(e)))
    }
}
BLOCK

echo 'Verifying field counts...'
acc_fields=$(grep -cE 'pub\s+[a-zA-Z_][a-zA-Z0-9_]*\s*:' crates/blockchain-types/src/account.rs)
block_fields=$(grep -cE 'pub\s+[a-zA-Z_][a-zA-Z0-9_]*\s*:' crates/blockchain-types/src/block.rs)
echo "Account fields: $acc_fields (target 8)"
echo "Block fields: $block_fields (target 16)"

if [ "$acc_fields" -ne 8 ] || [ "$block_fields" -ne 16 ]; then
    echo 'ERROR: Field count mismatch!'
    exit 1
fi

echo 'Running cargo check --all-targets (may take a while)...'
cargo check --all-targets

echo 'Creating PHASE1_SUMMARY.md...'
cat > PHASE1_SUMMARY.md <<'SUMMARY'
# Phase 1 Summary: blockchain-types 与 ORM 对齐

## 修改概览

| 文件 | 状态 | 字段数 |
|------|------|--------|
| crates/blockchain-types/src/account.rs | ✅ 重构 | 8 |
| crates/blockchain-types/src/block.rs   | ✅ 重构 | 16 |
| crates/orm/src/models.rs              | ⚠️ 待手动修复转换 | - |

## Account 字段（8个，对应 ACCOUNT 表）
1. id (AccountId)
2. balance (Amount)
3. unconfirmed_balance (Amount)
4. forged_balance (Amount)
5. active_lessee_id (Option<AccountId>)
6. has_control_phasing (bool)
7. height (Height)
8. latest (bool)

已删除：address, reserved_balance, guaranteed_balance, assets, properties, lease, created_at, last_updated, current_height

## Block 字段（16个，对应 BLOCK 表业务字段，不含 db_id)
1. version (u32)
2. timestamp (Timestamp)
3. height (Height)
4. previous_block_hash (Hash256)
5. payload_hash (Hash256)
6. generator_id (AccountId)
7. base_target (u64)
8. cumulative_difficulty (Vec<u8>)
9. total_amount (Amount)
10. total_fee (Amount)
11. payload_length (u32)
12. id (i64)
13. generation_signature: Option<Hash512> (skipped)
14. block_signature: Option<Hash512> (skipped)
15. previous_block_id: Option<i64>
16. next_block_id: Option<i64>

已删除：nonce, transactions

## 验证
- ✅ Account 字段数：8
- ✅ Block 字段数：16
- ✅ 编译通过：cargo check --all-targets

## 待办
- 修复 orm/src/models.rs 中的 AccountModel::to_domain/from_domain
- 可后续实现 Block Deserialize（当前仅 Serialize）
- 编写一致性测试（可选）
SUMMARY

echo 'Phase 1 fix completed!'