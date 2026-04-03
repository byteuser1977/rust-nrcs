//! HTTP REST API Server
//!
//! 提供区块链节点的 RESTful API 接口，使用 Axum 框架。
//! 支持：
//! - 账户查询与创建
//! - 交易提交与查询
//! - 区块查询
//! - 合约部署与调用
//! - 节点状态信息
//! - 健康检查

pub mod config;
pub mod error;
pub mod response;
pub mod routes;
pub mod handlers;
pub mod middleware;
pub mod state;

use axum::Router;
use config::ApiConfig;
use routes::create_router;
use state::ApiState;
use std::net::SocketAddr;

/// 启动 HTTP API 服务器（需要预先创建服务和状态）
pub async fn run_server(state: ApiState, addr: SocketAddr) -> anyhow::Result<()> {
    // 创建路由器
    let app = create_router(state);

    println!("🚀 HTTP API server listening on http://{}", addr);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 便捷函数：从配置启动服务器（会初始化所有服务）
pub async fn run_from_config(config: ApiConfig) -> anyhow::Result<()> {
    // 这里需要初始化所有服务（数据库、账户管理器等）
    // 由于依赖较多，暂不实现完整逻辑，建议在 apps/node 中完成初始化

    panic!("run_from_config not implemented yet; use apps/node");
}