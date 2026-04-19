//! API 路由配置

use axum::{
    Router,
    routing::{get, post},
};

use crate::{state::ApiState, handlers::system, handlers::account, handlers::transaction, handlers::block, handlers::contract, handlers::node};

/// 创建所有路由
pub fn create_router(state: ApiState) -> Router {
    Router::new()
        // 健康检查
        .route("/health", get(system::health_check))
        // 账户相关
        .route("/api/v1/accounts", post(account::create_account))
        .route("/api/v1/accounts/:id", get(account::get_account))
        .route("/api/v1/accounts/:id/balance", get(account::get_balance))
        .route("/api/v1/accounts", get(account::list_accounts))
        .route("/api/v1/accounts/transfer", post(account::transfer))
        // 交易相关
        .route("/api/v1/transactions", post(transaction::submit_transaction))
        .route("/api/v1/transactions/:hash", get(transaction::get_transaction))
        .route("/api/v1/transactions", get(transaction::list_transactions))
        // 区块相关
        .route("/api/v1/blocks/latest", get(block::get_latest_block))
        .route("/api/v1/blocks/:height", get(block::get_block_by_height))
        .route("/api/v1/blocks/hash/:hash", get(block::get_block_by_hash))
        .route("/api/v1/blocks", get(block::list_blocks))
        // 合约相关
        .route("/api/v1/contracts/deploy", post(contract::deploy_contract))
        .route("/api/v1/contracts/:address/call", post(contract::call_contract))
        .route("/api/v1/contracts/:address", get(contract::get_contract))
        // 节点相关
        .route("/api/v1/node/info", get(node::get_node_info))
        .route("/api/v1/node/peers", get(node::list_peers))
        // metrics
        .route("/metrics", get(system::metrics))
        .with_state(state)
}