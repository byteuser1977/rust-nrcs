//! Common types for transaction processing

use std::sync::Arc;

use blockchain_types::*;
use serde::{Deserialize, Serialize};

/// 交易状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxStatus {
    /// 待处理（在 mempool 中）
    Pending,
    /// 已确认（已打包进区块）
    Confirmed,
    /// 执行成功
    Success,
    /// 执行失败
    Failed,
}

/// 交易收据信息（返回给 API）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TxReceiptInfo {
    pub transaction_id: TransactionId,
    pub status: TxStatus,
    pub block_height: Option<Height>,
    pub gas_used: u64,
    pub logs: Vec<String>,
    pub contract_address: Option<[u8; 20]>,
    pub executed_at: Timestamp,
}

impl TxReceiptInfo {
    pub fn from_domain(receipt: &TxReceipt, block_height: Option<Height>) -> Self {
        let status = match receipt.status {
            0 => TxStatus::Pending,
            1 => TxStatus::Success,
            2 => TxStatus::Failed,
            _ => TxStatus::Failed,
        };

        // 解析 logs JSON 数组
        let logs_vec: Vec<String> = serde_json::from_str(&receipt.logs).unwrap_or_default();

        Self {
            transaction_id: receipt.transaction_id,
            status,
            block_height,
            gas_used: receipt.gas_used,
            logs: logs_vec,
            contract_address: receipt.contract_address,
            executed_at: receipt.executed_at,
        }
    }
}

/// 交易优先级（用于 mempool 排序）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TxPriority {
    /// gas 价格（每单位 gas 的费用）
    pub gas_price: Amount,
    /// 交易创建时间戳
    pub timestamp: Timestamp,
    /// 交易大小（字节）
    pub size: usize,
}

impl TxPriority {
    /// 计算得分：gas_price * 1000 - timestamp（越早得分越高）
    /// 返回值越大优先级越高
    pub fn score(&self) -> i64 {
        (self.gas_price as i64) * 1000 - (self.timestamp as i64)
    }
}

impl Ord for TxPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score().cmp(&other.score()).reverse() // 降序，高分优先
    }
}

impl PartialOrd for TxPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}