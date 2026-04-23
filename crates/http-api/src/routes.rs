//! API 路由配置
//!
//! 支持传统路由和 RESTful 路由

use axum::{
    Router,
    routing::{get, post},
    body::Body,
    http::{header, Response, StatusCode},
};
use tower_http::services::ServeDir;

use crate::{state::ApiState, nrcs_handler, test_page};

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/nrcs", get(nrcs_handler::handle_nrcs_get))
        .route("/nrcs", post(nrcs_handler::handle_nrcs_post))
        .route("/test", get(test_page::api_test_page))
        .route("/health", get(crate::handlers::system::health_check))
        // TODO: 重新启用 RESTful API 路由（需要调整处理器以使用 ApiState）
        // .route("/api/v1/accounts/:id", get(crate::handlers::account::get_account))
        // .route("/api/v1/accounts/:id/balance", get(crate::handlers::account::get_balance))
        // .route("/api/v1/blocks/latest", get(crate::handlers::block::get_latest_block))
        // .route("/api/v1/blocks/:height", get(crate::handlers::block::get_block_by_height))
        // .route("/api/v1/transactions/:hash", get(crate::handlers::transaction::get_transaction))
        // .route("/api/v1/node/info", get(crate::handlers::node::get_node_info))
        .route("/metrics", get(crate::handlers::system::metrics))
        .fallback_service(ServeDir::new("crates/http-api/ui"))
        .with_state(state)
}
