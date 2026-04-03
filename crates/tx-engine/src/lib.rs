//! Transaction Processing Engine
//!
//! 负责交易的生命周期管理：
//! - Validation: 验证交易签名、余额、nonce 等
//! - Execution: 应用交易到账户状态（余额、资产、合约）
//! - Mempool: 内存池管理（去重、优先级排序、持久化可选）
//! - Receipt: 生成交易收据（成功/失败、gas 消耗、日志）

pub mod processor;
pub mod mempool;
pub mod types;

pub use processor::{TransactionProcessor, DatabaseTransactionProcessor};
pub use mempool::{Mempool, MempoolConfig, MempoolStats};
pub use types::{TxStatus, TxReceiptInfo, TxPriority};