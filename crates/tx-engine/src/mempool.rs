//! Transaction Mempool
//!
//! 管理未确认交易的内存池：
//! - 去重（防止双花）
//! - 优先级排序（gas price 高的优先）
//! 内存限制+驱逐策略（可选）
//! - 持久化（可选的 Redis 或数据库）

use std::time::{Duration, Instant};

use dashmap::DashMap;
use parking_lot::RwLock;
use tracing::debug;

use blockchain_types::*;
use blockchain_types::prelude::Transaction;
use crate::TxPriority;
// use hex; // 暂时注释，等待 Cargo.toml 添加依赖

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
    /// 等待队列（延迟处理的交易）
    waiting_transactions: RwLock<Vec<(Transaction, Instant)>>,
    /// 已广播交易缓存
    broadcasted_transactions: DashMap<Hash256, Transaction>,
}

impl Mempool {
    pub fn new(config: MempoolConfig) -> Self {
        Self {
            pool: DashMap::new(),
            nonce_tracker: DashMap::new(),
            config,
            memory_usage: RwLock::new(0),
            waiting_transactions: RwLock::new(Vec::new()),
            broadcasted_transactions: DashMap::new(),
        }
    }

    /// 添加交易到内存池
    pub fn add(&self, tx: Transaction) -> std::result::Result<(), MempoolError> {
        let full_hash = tx.full_hash;

        // 1. 检查是否已存在
        if self.pool.contains_key(&full_hash) {
            return Err(MempoolError::DuplicateTransaction);
        }

        // 2. 检查发送者 nonce（防重放）
        // 注意：Transaction 中可能没有 nonce 字段，需要调整
        // 暂时跳过 nonce 检查，等待 Transaction 结构完善

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
        debug!("tx added to mempool: hash={:?}", full_hash);

        // 6. 更新 nonce tracker
        // self.nonce_tracker.insert(tx.sender_id, tx.nonce + 1);

        Ok(())
    }

    /// 移除交易
    pub fn remove(&self, hash: &Hash256) -> Option<Transaction> {
        if let Some((tx, _priority, _time)) = self.pool.remove(hash).map(|(_, value)| value) {
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
        self.pool.get(hash).map(|entry| entry.value().0.clone())
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
            .filter(|entry| entry.value().0.sender_id == sender_id)
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
    fn evict_one(&self) -> std::result::Result<(), MempoolError> {
        // 收集所有交易并按优先级排序
        let mut entries: Vec<_> = self.pool.iter().map(|entry| {
            let (_tx, priority, _time) = entry.value();
            (priority.clone(), entry.key().clone())
        }).collect();
        
        if entries.is_empty() {
            return Ok(());
        }
        
        // 按优先级排序（最低的先驱逐）
        entries.sort_by_key(|(p, _)| *p);
        
        let (_priority, hash) = entries.first().unwrap();
        if let Some(_tx) = self.remove(hash) {
            debug!("evicted tx from mempool: hash={:?}", hash);
        }
        
        Ok(())
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

    /// 重新排队所有未确认交易
    /// 对应 Java: requeueAllUnconfirmedTransactions()
    pub fn requeue_all(&self) {
        let mut waiting = self.waiting_transactions.write();
        for entry in self.pool.iter() {
            let (tx, _priority, time) = entry.value();
            waiting.push((tx.clone(), *time));
        }
        self.pool.clear();
        *self.memory_usage.write() = 0;
        debug!("Requeued all unconfirmed transactions to waiting queue");
    }

    /// 添加交易到等待队列
    /// 对应 Java: processLater()
    pub fn process_later(&self, transactions: Vec<Transaction>) {
        let mut waiting = self.waiting_transactions.write();
        let now = Instant::now();
        let count = transactions.len();
        for tx in transactions {
            waiting.push((tx, now));
        }
        debug!("Added {} transactions to waiting queue", count);
    }

    /// 处理等待队列中的交易
    /// 对应 Java: processWaitingTransactions()
    pub fn process_waiting(&self) -> Vec<Transaction> {
        let mut waiting = self.waiting_transactions.write();
        let mut processed = Vec::new();
        let now = Instant::now();
        
        waiting.retain(|(tx, time)| {
            let elapsed = now.duration_since(*time).as_secs();
            if elapsed > self.config.tx_ttl_seconds {
                debug!("Expired waiting transaction removed");
                return false;
            }
            
            match self.add(tx.clone()) {
                Ok(()) => {
                    processed.push(tx.clone());
                    false
                }
                Err(MempoolError::DuplicateTransaction) => false,
                Err(_) => true,
            }
        });
        
        debug!("Processed {} waiting transactions", processed.len());
        processed
    }

    /// 获取等待队列中的交易
    pub fn get_waiting(&self) -> Vec<Transaction> {
        self.waiting_transactions.read()
            .iter()
            .map(|(tx, _)| tx.clone())
            .collect()
    }

    /// 获取等待队列大小
    pub fn waiting_len(&self) -> usize {
        self.waiting_transactions.read().len()
    }

    /// 添加到已广播缓存
    pub fn add_broadcasted(&self, tx: &Transaction) {
        self.broadcasted_transactions.insert(tx.full_hash, tx.clone());
    }

    /// 检查是否已广播
    pub fn is_broadcasted(&self, hash: &Hash256) -> bool {
        self.broadcasted_transactions.contains_key(hash)
    }

    /// 从已广播缓存移除
    pub fn remove_broadcasted(&self, hash: &Hash256) -> Option<Transaction> {
        self.broadcasted_transactions.remove(hash).map(|(_, tx)| tx)
    }

    /// 获取所有已广播交易
    pub fn get_all_broadcasted(&self) -> Vec<Transaction> {
        self.broadcasted_transactions.iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// 获取缓存的未确认交易（排除指定列表）
    /// 对应 Java: getCachedUnconfirmedTransactions(exclude)
    pub fn get_cached(&self, exclude: &[Hash256]) -> Vec<Transaction> {
        self.pool.iter()
            .filter(|entry| !exclude.contains(entry.key()))
            .map(|entry| entry.value().0.clone())
            .collect()
    }

    /// 获取所有未确认交易ID
    pub fn get_all_ids(&self) -> Vec<Hash256> {
        self.pool.iter().map(|entry| *entry.key()).collect()
    }

    /// 根据ID获取交易
    pub fn get_by_id(&self, id: u64) -> Option<Transaction> {
        self.pool.iter()
            .find(|entry| entry.value().0.id == id)
            .map(|entry| entry.value().0.clone())
    }
}

/// 内存池错误
#[derive(Debug, thiserror::Error)]
pub enum MempoolError {
    #[error("duplicate transaction")]
    DuplicateTransaction,

    #[error("invalid nonce")]
    InvalidNonce,

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