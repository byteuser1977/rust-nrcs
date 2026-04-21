//! HTTP REST API Server
//!
//! 提供区块链节点的 RESTful API 接口，使用 Axum 框架。
//! 与 Java 版本 NRCS API 完全对齐。
//!
//! 支持：
//! - 传统路由模式: /nrcs?requestType=getAccount
//! - RESTful 模式: /api/v1/accounts/:id
//! - API 测试页面: /test

pub mod api_tag;
pub mod api_registry;
pub mod config;
pub mod dto;
pub mod error;
pub mod nrcs_handler;
pub mod request_handler;
pub mod response;
pub mod routes;
pub mod handlers;
pub mod state;
pub mod test_page;

use axum::Router;
use config::ApiConfig;
use routes::create_router;
use state::ApiState;
use std::net::SocketAddr;

pub use api_tag::ApiTag;
pub use api_registry::{register_api, get_api_handler, get_all_handlers, API_REGISTRY};
pub use request_handler::{ApiRequest, RequestHandler, HandlerPtr, RsResp, RsRespWithData, RsRespBuilder};
pub use error::ApiError;

pub async fn run_server(state: ApiState, addr: SocketAddr) -> anyhow::Result<()> {
    let app = create_router(state);

    println!("🚀 HTTP API server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

pub async fn run_from_config(config: ApiConfig) -> anyhow::Result<()> {
    panic!("run_from_config not implemented yet; use apps/node");
}

pub fn init_api_handlers() {
    use std::sync::Arc;
    use crate::handlers::v1::*;
    
    register_api("getAccount", Arc::new(GetAccountHandler::new()));
    register_api("getBlock", Arc::new(GetBlockHandler::new()));
    register_api("getBlockchainStatus", Arc::new(GetBlockchainStatusHandler::new()));
    register_api("getTime", Arc::new(GetTimeHandler::new()));
    register_api("getBalance", Arc::new(GetBalanceHandler::new()));
    register_api("getAccountId", Arc::new(GetAccountIdHandler::new()));
    register_api("getTransaction", Arc::new(GetTransactionHandler::new()));
}
