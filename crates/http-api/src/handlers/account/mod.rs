//! 账户 API 处理器
//!
//! 提供账户相关的 API 端点：
//! - getAccount
//! - getBalance
//! - getAccountPublicKey

pub mod handler;
pub mod state;
pub mod dto;

pub use handler::*;
pub use state::AccountApiState;
