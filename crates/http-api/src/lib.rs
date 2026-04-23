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
pub mod core;
pub mod dto;
pub mod error;
pub mod handlers;
pub mod nrcs_handler;
pub mod proxy;
pub mod request_handler;
pub mod response;
pub mod routes;
pub mod services;
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

pub async fn run_from_config(_config: ApiConfig) -> anyhow::Result<()> {
    panic!("run_from_config not implemented yet; use apps/node");
}

// TODO: 重新实现 API 注册机制
// 旧的 init_api_handlers 函数已被注释掉，因为使用了不存在的处理器
// 需要根据新的架构重新实现 API 注册机制
/*
pub fn init_api_handlers() {
    use std::sync::Arc;
    
    // Account APIs
    register_api("getAccountAssets", Arc::new(GetAccountAssetsHandler::new()));
    register_api("getAccountCurrencies", Arc::new(GetAccountCurrenciesHandler::new()));
    register_api("getAccountProperties", Arc::new(GetAccountPropertiesHandler::new()));
    register_api("getAccountLessors", Arc::new(GetAccountLessorsHandler::new()));
    register_api("getBalance", Arc::new(GetBalanceHandler::new()));
    register_api("getBalances", Arc::new(GetBalancesHandler::new()));
    register_api("getEffectiveBalance", Arc::new(GetEffectiveBalanceHandler::new()));
    register_api("getGuaranteedBalance", Arc::new(GetGuaranteedBalanceHandler::new()));
    register_api("setAccountInfo", Arc::new(SetAccountInfoHandler::new()));
    register_api("setAccountProperty", Arc::new(SetAccountPropertyHandler::new()));
    register_api("getAccountAssetCount", Arc::new(GetAccountAssetCountHandler::new()));
    register_api("getAccountCurrencyCount", Arc::new(GetAccountCurrencyCountHandler::new()));
    
    // ... 其他所有 API 注册 ...
    
    register_api("getCoinExchangeOrderIds", Arc::new(GetCoinExchangeOrderIdsHandler::new()));
    register_api("getCoinExchangeOrders", Arc::new(GetCoinExchangeOrdersHandler::new()));
    register_api("getCoinExchangeTrades", Arc::new(GetCoinExchangeTradesHandler::new()));
    register_api("exchangeCoins", Arc::new(ExchangeCoinsHandler::new()));
    register_api("cancelCoinExchangeOrder", Arc::new(CancelCoinExchangeOrderHandler::new()));
    register_api("simulateCoinExchange", Arc::new(SimulateCoinExchangeHandler::new()));
}
*/
