//! System API Handlers
//!
//! 系统相关的 API 处理器

use axum::Json;
use serde_json::json;

/// 健康检查
pub async fn health_check() -> &'static str {
    "OK"
}

/// 指标端点
pub async fn metrics() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
