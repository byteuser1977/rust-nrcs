//! 账户与资产相关数据结构

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Utc;

/// 账户结构体
///
/// 存储账户的基本信息、余额和相关属性。
/// 账户 ID 通常由公钥的 SHA-256 哈希得出。
///
/// 参考 Java: `BaseAccount`
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Account {
    /// 账户 ID（唯一标识，64 位整数）
    pub id: AccountId,
    /// 账户地址（Base58 编码，Rust 中可变，便于显示）
    /// 数据库存储时可能不包含，由 ID 实时计算
    #[serde(skip)]
    pub address: Option<String>,
    /// 可用余额（已确认，NQT）
    pub balance: Amount,
    /// 未确认余额（Pending，正在处理中的交易）
    pub unconfirmed_balance: Amount,
    /// 保留余额（Reserved，用于租赁等锁定用途）
    pub reserved_balance: Amount,
    /// 担保余额（Guaranteed，用于特定担保场景）
    pub guaranteed_balance: Amount,
    /// 资产列表（持有的资产 ID -> 数量）
    pub assets: HashMap<AssetId, Amount>,
    /// 账户属性（键值对，如 "name", "description"）
    pub properties: HashMap<String, String>,
    /// 账户租赁信息（出块权租赁）
    pub lease: Option<AccountLease>,
    /// 创建时间（Unix 时间戳）
    pub created_at: Timestamp,
    /// 最后更新时间
    pub last_updated: Timestamp,
    /// 当前块高度（用于确认余额）
    pub current_height: Height,
}

impl Account {
    /// 创建新账户
    pub fn new(id: AccountId, initial_balance: Amount) -> Self {
        let now = chrono::Utc::now().timestamp() as u32;
        Self {
            id,
            address: None,
            balance: initial_balance,
            unconfirmed_balance: initial_balance,
            reserved_balance: 0,
            guaranteed_balance: 0,
            assets: HashMap::new(),
            properties: HashMap::new(),
            lease: None,
            created_at: now,
            last_updated: now,
            current_height: 0,
        }
    }

    /// 获取账户有效余额（可用于交易）
    /// 有效余额 = balance - reserved_balance
    pub fn effective_balance(&self) -> Amount {
        self.balance.saturating_sub(self.reserved_balance)
    }

    /// 检查是否有足够余额
    pub fn has_balance(&self, required: Amount) -> bool {
        self.effective_balance() >= required
    }

    /// 增加余额
    pub fn credit(&mut self, amount: Amount) {
        self.balance = self.balance.saturating_add(amount);
        self.unconfirmed_balance = self.unconfirmed_balance.saturating_add(amount);
    }

    /// 减少余额
    pub fn debit(&mut self, amount: Amount) -> Result<()> {
        if !self.has_balance(amount) {
            return Err(BlockchainError::InsufficientBalance {
                have: self.effective_balance(),
                need: amount,
            });
        }
        self.balance = self.balance.saturating_sub(amount);
        self.unconfirmed_balance = self.unconfirmed_balance.saturating_sub(amount);
        Ok(())
    }

    /// 持有某资产的数量
    pub fn asset_quantity(&self, asset_id: AssetId) -> Amount {
        self.assets.get(&asset_id).copied().unwrap_or(0)
    }

    /// 增加资产持有
    pub fn add_asset(&mut self, asset_id: AssetId, amount: Amount) {
        let current = self.asset_quantity(asset_id);
        if current == 0 && amount > 0 {
            self.assets.insert(asset_id, amount);
        } else {
            *self.assets.entry(asset_id).or_insert(0) = current.saturating_add(amount);
        }
    }

    /// 减少资产持有
    pub fn remove_asset(&mut self, asset_id: AssetId, amount: Amount) -> Result<()> {
        let current = self.asset_quantity(asset_id);
        if current < amount {
            return Err(BlockchainError::InvalidTransaction(
                format!("insufficient asset {}: have {}, need {}", asset_id, current, amount)
            ));
        }
        if current == amount {
            self.assets.remove(&asset_id);
        } else {
            *self.assets.get_mut(&asset_id).unwrap() = current.saturating_sub(amount);
        }
        Ok(())
    }

    /// 计算账户的权益权重（用于 PoS 出块选择）
    /// 权重 = effective_balance * lease_factor (if leased)
    pub fn forging_weight(&self, at_height: Height) -> Amount {
        let mut weight = self.effective_balance();

        // 如果当前处于租赁期，则租赁余额不计入
        if let Some(lease) = &self.lease {
            if at_height >= lease.start_height && at_height <= lease.end_height {
                weight = self.balance.saturating_sub(lease.amount);
            }
        }

        weight
    }

    /// 设置账户地址（Base58）
    pub fn set_address(&mut self, address: String) {
        self.address = Some(address);
    }

    /// 获取 Base58 地址
    pub fn get_address(&self) -> Option<&str> {
        self.address.as_deref()
    }
}

/// 账户租赁信息
///
/// 账户可以将其余额租赁给他人，让他人获得出块权。
/// 原版 Java: `BaseAccountLease`
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AccountLease {
    /// 被租赁者账户 ID
    pub lessee_id: AccountId,
    /// 租赁金额（NQT）
    pub amount: Amount,
    /// 租赁开始区块高度
    pub start_height: Height,
    /// 租赁结束区块高度（1440 个区块 ≈ 1 天）
    pub end_height: Height,
}

impl AccountLease {
    pub fn new(lessee_id: AccountId, amount: Amount, start_height: Height, duration: Height) -> Self {
        Self {
            lessee_id,
            amount,
            start_height,
            end_height: start_height + duration,
        }
    }

    /// 是否在有效期内
    pub fn is_active(&self, current_height: Height) -> bool {
        current_height >= self.start_height && current_height <= self.end_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_balance() {
        let mut account = Account::new(1234567890, 1000);
        assert!(account.has_balance(500));
        assert!(account.debit(300).is_ok());
        assert_eq!(account.balance, 700);
        assert!(account.debit(800).is_err());
    }

    #[test]
    fn test_account_assets() {
        let mut account = Account::new(1234567890, 0);
        account.add_asset(100, 1000);
        assert_eq!(account.asset_quantity(100), 1000);
        account.add_asset(100, 500);
        assert_eq!(account.asset_quantity(100), 1500);
        assert!(account.remove_asset(100, 1200).is_ok());
        assert_eq!(account.asset_quantity(100), 300);
    }

    #[test]
    fn test_forging_weight() {
        let account = Account::new(1234567890, 1000);
        let weight = account.forging_weight(100);
        assert_eq!(weight, 1000);
    }
}
