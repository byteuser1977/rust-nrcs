//! 系统处理器（健康检查、metrics等）

use axum::{http::StatusCode, Json};
use serde::Serialize;

use crate::error::{ApiResult, response::ApiResponse};

/// 健康检查响应
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: u128,
    pub uptime_seconds: u64,
}

/// 健康检查端点
pub async fn health_check() -> ApiResult<Json<HealthResponse>> {
    let response = HealthResponse {
        status: "ok".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
        uptime_seconds: 0, // TODO: 记录启动时间
    };

    Ok(Json(response))
}

/// Prometheus metrics 端点（可选）
pub async fn metrics() -> &'static str {
    // TODO: 实现 metrics 收集
    "# HELP nrcs_node_info Node information\n# TYPE nrcs_node_info gauge\nnrcs_node_info{version=\"0.1.0\"} 1\n"
}