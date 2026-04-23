//! 交易 API 处理器
//!
//! 提供交易相关的 API 端点：
//! - sendMoney
//! - getTransaction
//! - getUnconfirmedTransactions

pub mod handler;
pub mod state;
pub mod dto;

pub use handler::*;
pub use state::TransactionApiState;
