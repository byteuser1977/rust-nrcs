//! API 路由配置
//!
//! 支持传统路由和 RESTful 路由

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::services::ServeDir;

use crate::{state::ApiState, nrcs_handler, test_page};

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/nrcs", get(nrcs_handler::handle_nrcs_get))
        .route("/nrcs", post(nrcs_handler::handle_nrcs_post))
        .route("/test", get(test_page::api_test_page))
        .route("/test-proxy", get(test_page::api_test_page_proxy))
        .route("/health", get(crate::handlers::system::health_check))
        .route("/api/v1/accounts", post(crate::handlers::restful::create_account))
        .route("/api/v1/accounts/:id", get(crate::handlers::restful::get_account))
        .route("/api/v1/accounts/:id/balance", get(crate::handlers::restful::get_balance))
        .route("/api/v1/blocks/latest", get(crate::handlers::restful::get_latest_block))
        .route("/api/v1/blocks/:height", get(crate::handlers::restful::get_block_by_height))
        .route("/metrics", get(crate::handlers::system::metrics))
        .fallback_service(ServeDir::new("crates/http-api/ui"))
        .with_state(state)
}
