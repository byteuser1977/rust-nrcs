//! 账户查询服务模块
//!
//! 对应 NRCS Java: `com.bytechain.nrcs.service.account.AccountService`
//!
//! 功能：
//! - 账户区块统计和列表查询
//! - 账户交易历史查询
//! - 账户资产余额查询
//! - 账户租赁信息查询

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 账户查询服务 Trait
///
/// 定义账户相关的高级查询接口
#[async_trait::async_trait]
pub trait AccountQueryServiceApi: Send + Sync {
    /// 获取账户产生的区块数量
    ///
    /// 对应 NRCS Java: `Account.getBlockCount()`
    async fn get_account_block_count(&self, account_id: u64) -> i64;

    /// 获取账户产生的区块 ID 列表
    ///
    /// 对应 NRCS Java: `Account.getBlockIds()`
    async fn get_account_block_ids(
        &self,
        account_id: u64,
        first_index: i32,
        last_index: i32,
    ) -> Vec<i64>;

    /// 获取账户产生的区块详情列表
    ///
    /// 对应 NRCS Java: `Account.getBlocks()`
    async fn get_account_blocks(
        &self,
        account_id: u64,
        first_index: i32,
        last_index: i32,
    ) -> Vec<serde_json::Value>;

    /// 获取账户的交易数量
    async fn get_account_transaction_count(&self, account_id: u64) -> i64;

    /// 获取账户的交易 ID 列表
    async fn get_account_transaction_ids(
        &self,
        account_id: u64,
        first_index: i32,
        last_index: i32,
    ) -> Vec<String>;

    /// 获取账户的资产余额列表
    async fn get_account_asset_balances(
        &self,
        account_id: u64,
        first_index: i32,
        last_index: i32,
    ) -> Vec<serde_json::Value>;

    /// 获取账户的货币余额列表
    async fn get_account_currency_balances(
        &self,
        account_id: u64,
        first_index: i32,
        last_index: i32,
    ) -> Vec<serde_json::Value>;

    /// 获取账户的租赁者列表
    async fn get_account_lessors(
        &self,
        account_id: u64,
    ) -> (Vec<u64>, Vec<String>);
}

/// 基于内存的实现（用于测试和开发）
///
/// 生产环境应替换为基于 ORM 的实现
pub struct MemoryAccountQueryService {
    block_stats: Arc<RwLock<HashMap<u64, i64>>>,
    block_ids_cache: Arc<RwLock<HashMap<u64, Vec<i64>>>>,
}

impl Default for MemoryAccountQueryService {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryAccountQueryService {
    pub fn new() -> Self {
        Self {
            block_stats: Arc::new(RwLock::new(HashMap::new())),
            block_ids_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 分页处理辅助函数
    fn paginate<T: Clone>(items: &[T], first_index: i32, last_index: i32) -> Vec<T> {
        let start = if first_index < 0 { 0 } else { first_index as usize };
        let end = if last_index < 0 {
            items.len()
        } else {
            (last_index as usize + 1).min(items.len())
        };

        if start >= items.len() || start > end {
            return vec![];
        }

        items[start..end].to_vec()
    }
}

#[async_trait::async_trait]
impl AccountQueryServiceApi for MemoryAccountQueryService {
    async fn get_account_block_count(&self, account_id: u64) -> i64 {
        let stats = self.block_stats.read().await;
        stats.get(&account_id).copied().unwrap_or(0)
    }

    async fn get_account_block_ids(
        &self,
        account_id: u64,
        first_index: i32,
        last_index: i32,
    ) -> Vec<i64> {
        let cache = self.block_ids_cache.read().await;
        
        match cache.get(&account_id) {
            Some(ids) => Self::paginate(ids, first_index, last_index),
            None => vec![],
        }
    }

    async fn get_account_blocks(
        &self,
        _account_id: u64,
        _first_index: i32,
        _last_index: i32,
    ) -> Vec<serde_json::Value> {
        // 实际实现中需要从 BlockRepository 查询
        // 这里返回空数组，等待后续集成
        vec![]
    }

    async fn get_account_transaction_count(&self, _account_id: u64) -> i64 {
        // 实际实现中需要从 TransactionRepository 统计
        0
    }

    async fn get_account_transaction_ids(
        &self,
        _account_id: u64,
        _first_index: i32,
        _last_index: i32,
    ) -> Vec<String> {
        // 实际实现中需要从 TransactionRepository 查询
        vec![]
    }

    async fn get_account_asset_balances(
        &self,
        _account_id: u64,
        _first_index: i32,
        _last_index: i32,
    ) -> Vec<serde_json::Value> {
        // 实际实现中需要从 AccountAssetRepository 查询
        vec![]
    }

    async fn get_account_currency_balances(
        &self,
        _account_id: u64,
        _first_index: i32,
        _last_index: i32,
    ) -> Vec<serde_json::Value> {
        // 实际实现中需要从 Currency 相关 Repository 查询
        vec![]
    }

    async fn get_account_lessors(&self, _account_id: u64) -> (Vec<u64>, Vec<String>) {
        // 实际实现中需要从 Account 表查询租赁关系
        (vec![], vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_account_query_service() {
        let service = MemoryAccountQueryService::new();
        
        // 测试默认值
        let count = service.get_account_block_count(12345).await;
        assert_eq!(count, 0);

        let ids = service.get_account_block_ids(12345, 0, -1).await;
        assert!(ids.is_empty());
    }
}
