//! HTTP API 核心模块
//!
//! 提供核心 API 框架功能：
//! - 路由管理
//! - 中间件
//! - 错误处理

pub mod error;
pub mod middleware;
pub mod router;

pub use error::ApiError;
pub use router::create_router;
