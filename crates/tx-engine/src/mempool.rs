//! Transaction Mempool
//!
//! 管理未确认交易的内存池：
//! - 去重（防止双花）
//! - 优先级排序（gas price 高的优先）
//! 内存限制+驱逐策略（可选）
//! - 持久化（可选的 Redis 或数据库）

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use parking_lot::RwLock;
use tracing::{debug, warn};

use blockchain_types::*;
use crate::{Transaction, TxPriority};

/// Mempool 配置
#[derive(Debug, Clone)]
pub struct MempoolConfig {
    /// 最大交易数量
    pub max_txs: usize,
    /// 最大内存占用（字节）
    pub max_memory_bytes: usize,
    /// 交易过期时间（秒）
    pub tx_ttl_seconds: u64,
    /// 是否启用持久化（写入数据库或 Redis）
    pub persistent: bool,
    /// 驱逐策略：true=按费用最低驱逐，false=FIFO
    pub evict_by_fee: bool,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        Self {
            max_txs: 100_000,
            max_memory_bytes: 100 * 1024 * 1024, // 100 MB
            tx_ttl_seconds: 86400 * 3, // 3 days
            persistent: false,
            evict_by_fee: true,
        }
    }
}

/// Mempool 结构
pub struct Mempool {
    /// 交易池（tx_hash -> (Transaction, priority, timestamp)）
    pool: DashMap<Hash256, (Transaction, TxPriority, Instant)>,
    /// 发送者 nonce 跟踪（防止重放攻击）
    nonce_tracker: DashMap<AccountId, u64>,
    /// 配置
    config: MempoolConfig,
    /// 当前内存占用（字节，估算值）
    memory_usage: RwLock<usize>,
}

impl Mempool {
    pub fn new(config: MempoolConfig) -> Self {
        Self {
            pool: DashMap::new(),
            nonce_tracker: DashMap::new(),
            config,
            memory_usage: RwLock::new(0),
        }
    }

    /// 添加交易到内存池
    pub fn add(&self, tx: Transaction) -> Result<(), MempoolError> {
        let full_hash = tx.full_hash;

        // 1. 检查是否已存在
        if self.pool.contains_key(&full_hash) {
            return Err(MempoolError::DuplicateTransaction);
        }

        // 2. 检查发送者 nonce（防重放）
        let expected_nonce = self.get_next_nonce(tx.sender_id);
        // 注意：Transaction 中的 deadline 字段实际用于表示 nonce 吗？
        // 在原版 NXT 中，deadline 是交易过期时间，nonce 是单独的字段
        // 这里假设我们使用一个字段来追踪 nonce，简化处理
        // 实际上需要调整 Transaction 结构或使用扩展字段

        // 3. 计算优先级
        let priority = TxPriority {
            gas_price: tx.fee, // 简化：fee 即 gas_price
            timestamp: tx.timestamp,
            size: tx.size(),
        };

        // 4. 检查内存限制
        let tx_size = priority.size;
        {
            let mut usage = self.memory_usage.write();
            *usage += tx_size;
            if *usage > self.config.max_memory_bytes {
                // 需要驱逐交易
                self.evict_one()?;
            }
        }

        // 5. 插入交易池
        self.pool.insert(full_hash, (tx, priority, Instant::now()));
        debug!("tx added to mempool: hash={:x}", hex::encode(full_hash));

        // 6. 更新 nonce tracker
        // self.nonce_tracker.insert(tx.sender_id, tx.nonce + 1);

        Ok(())
    }

    /// 移除交易
    pub fn remove(&self, hash: &Hash256) -> Option<Transaction> {
        if let Some((tx, _priority, _time)) = self.pool.remove(hash) {
            // 更新内存占用
            let size = tx.size();
            *self.memory_usage.write() = self.memory_usage.read().saturating_sub(size);
            Some(tx)
        } else {
            None
        }
    }

    /// 获取交易
    pub fn get(&self, hash: &Hash256) -> Option<Transaction> {
        self.pool.get(hash).map(|(tx, _p, _t)| tx.clone())
    }

    /// 获取所有交易（按优先级排序）
    pub fn get_all_sorted(&self) -> Vec<Transaction> {
        let mut vec: Vec<(Transaction, TxPriority, Instant)> = self.pool.iter().map(|v| v.value().clone()).collect();
        vec.sort_by(|a, b| a.1.cmp(&b.1));
        vec.into_iter().map(|(tx, _p, _t)| tx).collect()
    }

    /// 获取指定发送者的交易
    pub fn get_by_sender(&self, sender_id: AccountId) -> Vec<Transaction> {
        self.pool
            .iter()
            .filter(|(_, (tx, _, _))| tx.sender_id == sender_id)
            .map(|v| v.value().0.clone())
            .collect()
    }

    /// 批量移除交易
    pub fn remove_many(&self, hashes: &[Hash256]) -> Vec<Transaction> {
        let mut removed = Vec::new();
        for hash in hashes {
            if let Some(tx) = self.remove(hash) {
                removed.push(tx);
            }
        }
        removed
    }

    /// 清理过期交易
    pub fn cleanup_expired(&self) -> Vec<Hash256> {
        let now = Instant::now();
        let mut expired_hashes = Vec::new();

        for entry in self.pool.iter() {
            let hash = *entry.key();
            let (_, _, time) = entry.value();
            if now.duration_since(*time) > Duration::from_secs(self.config.tx_ttl_seconds) {
                self.remove(&hash);
                expired_hashes.push(hash);
            }
        }

        expired_hashes
    }

    /// 获取池大小
    pub fn len(&self) -> usize {
        self.pool.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }

    /// 清除所有交易
    pub fn clear(&self) {
        self.pool.clear();
        *self.memory_usage.write() = 0;
        self.nonce_tracker.clear();
    }

    /// 检查是否存在某交易
    pub fn contains(&self, hash: &Hash256) -> bool {
        self.pool.contains_key(hash)
    }

    /// 获取下一个 nonce（防重放）
    fn get_next_nonce(&self, sender_id: AccountId) -> u64 {
        self.nonce_tracker
            .entry(sender_id)
            .and_modify(|n| *n += 1)
            .or_insert(0)
            .clone()
    }

    /// 驱逐一个最不优先的交易
    fn evict_one(&self) -> Result<(), MempoolError> {
        if let Some((hash, (tx, priority, _time))) = self.pool.iter().next() {
            // 如果 evict_by_fee 为 true，按 gas price 最低的驱逐
            // 否则按最早插入的驱逐（当前迭代顺序不保证，需要收集排序）
            // 简化：移除第一个
            self.remove(hash);
            debug!("evicted tx from mempool: hash={:x}", hex::encode(*hash));
            Ok(())
        } else {
            Ok(())
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> MempoolStats {
        let memory = *self.memory_usage.read();
        MempoolStats {
            transaction_count: self.pool.len(),
            memory_usage_bytes: memory,
            senders_count: self.nonce_tracker.len(),
        }
    }
}

/// 内存池错误
#[derive(Debug, thiserror::Error)]
pub enum MempoolError {
    #[error("duplicate transaction")]
    DuplicateTransaction,

    #[error("mempool full")]
    Full,

    #[error("insufficient balance")]
    InsufficientBalance,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// 内存池统计
#[derive(Debug, Clone)]
pub struct MempoolStats {
    pub transaction_count: usize,
    pub memory_usage_bytes: usize,
    pub senders_count: usize,
}