//! API 路由配置

use axum::{
    Router,
    routing::{get, post},
};

use crate::{handlers::*, state::ApiState};

/// 创建所有路由
pub fn create_router(state: ApiState) -> Router {
    Router::new()
        // 健康检查
        .route("/health", get(health_check))
        // 账户相关
        .route("/api/v1/accounts", post(handlers::account::create_account))
        .route("/api/v1/accounts/:address", get(handlers::account::get_account))
        .route("/api/v1/accounts/:id/balance", get(handlers::account::get_balance))
        .route("/api/v1/accounts/transfer", post(handlers::account::transfer))
        // 交易相关
        .route("/api/v1/transactions", post(handlers::transaction::submit_transaction))
        .route("/api/v1/transactions/:hash", get(handlers::transaction::get_transaction))
        .route("/api/v1/transactions", get(handlers::transaction::list_transactions))
        // 区块相关
        .route("/api/v1/blocks/latest", get(handlers::block::get_latest_block))
        .route("/api/v1/blocks/:height", get(handlers::block::get_block_by_height))
        .route("/api/v1/blocks/hash/:hash", get(handlers::block::get_block_by_hash))
        .route("/api/v1/blocks", get(handlers::block::list_blocks))
        // 合约相关
        .route("/api/v1/contracts/deploy", post(handlers::contract::deploy_contract))
        .route("/api/v1/contracts/:address/call", post(handlers::contract::call_contract))
        .route("/api/v1/contracts/:address", get(handlers::contract::get_contract))
        // 节点相关
        .route("/api/v1/node/info", get(handlers::node::get_node_info))
        .route("/api/v1/node/peers", get(handlers::node::list_peers))
        // metrics
        .route("/metrics", get(handlers::system::metrics))
        .with_state(state)
}