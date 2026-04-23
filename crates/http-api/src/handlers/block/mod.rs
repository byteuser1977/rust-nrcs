//! 区块 API 处理器
//!
//! 提供区块相关的 API 端点：
//! - getBlock
//! - getBlocks
//! - getBlockchainStatus

pub mod handler;
pub mod state;
pub mod dto;

pub use handler::*;
pub use state::BlockApiState;
