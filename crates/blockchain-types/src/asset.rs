//! 资产数据结构定义

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use chrono::Utc;

/// 资产结构体
///
/// 资产是区块链上可交易的可量化物品（代币、NFT、票据等）。
/// 支持整数数量，可设置小数精度。
///
/// 参考 Java: `BaseAsset`
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Asset {
    /// 资产 ID（全局唯一）
    pub id: AssetId,
    /// 所有者账户 ID
    pub owner_id: AccountId,
    /// 资产名称（UTF-8 字符串，最长 100 字符）
    pub name: String,
    /// 资产描述（可选，最长 1000 字符）
    pub description: String,
    /// 总发行量（单位：最小面额，如 decimals=2 表示 0.01 的单位）
    pub quantity: Amount,
    /// 小数位数（0-8，默认 0）
    pub decimals: u8,
    /// 是否可铸造（发行后是否允许增发）
    pub mintable: bool,
    /// 是否可转让（资产是否可以交易）
    pub transferable: bool,
    /// 附加数据（可选，用于存储图标、JSON 元数据等）
    /// 可存储 IPFS 哈希、图片数据、JSON 结构
    pub data: Vec<u8>,
    /// 创建时间
    pub created_at: Timestamp,
    /// 最后修改时间
    pub last_updated: Timestamp,
    /// 是否已删除（软删除）
    pub deleted: bool,
}

impl Asset {
    /// 创建新资产
    pub fn new(
        owner_id: AccountId,
        name: String,
        description: String,
        quantity: Amount,
        decimals: u8,
    ) -> Self {
        let now = chrono::Utc::now().timestamp() as u32;
        Self {
            id: 0, // 需要后续分配（从数据库自增或链上生成）
            owner_id,
            name,
            description,
            quantity,
            decimals,
            mintable: false,
            transferable: true,
            data: vec![],
            created_at: now,
            last_updated: now,
            deleted: false,
        }
    }

    /// 获取显示数量（带小数转换）
    /// display_quantity = quantity / 10^decimals
    pub fn display_quantity(&self) -> f64 {
        let divisor = 10u64.pow(self.decimals as u32) as f64;
        self.quantity as f64 / divisor
    }

    /// 设置显示数量（从浮点数转回整数）
    pub fn set_display_quantity(&mut self, display: f64) {
        let divisor = 10u64.pow(self.decimals as u32) as f64;
        self.quantity = (display * divisor).round() as u64;
    }

    /// 资产是否可交易（未删除且可转让）
    pub fn is_tradable(&self) -> bool {
        !self.deleted && self.transferable
    }

    /// 更新数据（IPFS 哈希等）
    pub fn update_data(&mut self, data: Vec<u8>) {
        self.data = data;
        self.last_updated = chrono::Utc::now().timestamp() as u32;
    }

    /// 销毁资产（软删除）
    pub fn delete(&mut self) {
        self.deleted = true;
        self.last_updated = chrono::Utc::now().timestamp() as u32;
    }
}

/// 账户资产持仓记录
///
/// 快速查询账户持有某资产的数量，避免 HashMap 重复计算
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AccountAsset {
    /// 账户 ID
    pub account_id: AccountId,
    /// 资产 ID
    pub asset_id: AssetId,
    /// 持有数量
    pub quantity: Amount,
    /// 最后更新时间
    pub last_updated: Timestamp,
}

impl AccountAsset {
    pub fn new(account_id: AccountId, asset_id: AssetId, quantity: Amount) -> Self {
        let now = chrono::Utc::now().timestamp() as u32;
        Self { account_id, asset_id, quantity, last_updated: now }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_display_quantity() {
        let mut asset = Asset::new(1234567890, "TestCoin".to_string(), "desc".to_string(), 100_000_000, 8);
        assert_eq!(asset.display_quantity(), 1.0); // 100,000,000 * 10^-8 = 1

        asset.decimals = 2;
        asset.quantity = 12345;
        assert_eq!(asset.display_quantity(), 123.45); // 12345 * 10^-2 = 123.45
    }

    #[test]
    fn test_asset_tradable() {
        let mut asset = Asset::new(1234567890, "NFT".to_string(), "test".to_string(), 1, 0);
        assert!(asset.is_tradable());
        asset.deleted = true;
        assert!(!asset.is_tradable());
        asset.deleted = false;
        asset.transferable = false;
        assert!(!asset.is_tradable());
    }

    #[test]
    fn test_account_asset() {
        let aa = AccountAsset::new(123, 456, 1000);
        assert_eq!(aa.account_id, 123);
        assert_eq!(aa.asset_id, 456);
        assert_eq!(aa.quantity, 1000);
    }
}
