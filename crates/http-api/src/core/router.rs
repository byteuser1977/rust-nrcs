//! API Router
//!
//! 路由配置（新架构，暂时未启用）
//!
//! 注意：此路由器暂时未启用，当前使用 handlers/ 目录下的旧路由结构。
//! 新架构需要调整处理器以使用 ApiState，而不是独立的状态。

use axum::{
    routing::get,
    Router,
};

/// 创建主路由（新架构）
/// 注意：此路由器暂时未启用，需要调整处理器以使用 ApiState
pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
}

/// 健康检查
async fn health_check() -> &'static str {
    "OK"
}
