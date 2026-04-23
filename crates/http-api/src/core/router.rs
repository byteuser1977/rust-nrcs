//! API Router
//!
//! 路由配置（新架构，暂时未启用）

use axum::{
    routing::{get, post},
    Router,
};

/// 创建主路由（新架构）
/// 注意：此路由器暂时未启用，需要调整处理器以使用 ApiState
pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        // TODO: 启用新的路由结构
        // .nest("/nrcs", api_routes())
}

/// API 路由
fn _api_routes() -> Router {
    Router::new()
        .merge(_account_routes())
        .merge(_transaction_routes())
        .merge(_block_routes())
        .merge(_network_routes())
}

/// 账户相关路由
fn _account_routes() -> Router {
    Router::new()
        .route("/getAccount", post(crate::handlers::account::get_account))
        .route("/getBalance", post(crate::handlers::account::get_balance))
        .route("/getAccountPublicKey", post(crate::handlers::account::get_account_public_key))
}

/// 交易相关路由
fn _transaction_routes() -> Router {
    Router::new()
        .route("/sendMoney", post(crate::handlers::transaction::send_money))
        .route("/getTransaction", post(crate::handlers::transaction::get_transaction))
        .route("/getUnconfirmedTransactions", post(crate::handlers::transaction::get_unconfirmed_transactions))
}

/// 区块相关路由
fn _block_routes() -> Router {
    Router::new()
        .route("/getBlock", post(crate::handlers::block::get_block))
        .route("/getBlocks", post(crate::handlers::block::get_blocks))
        .route("/getBlockchainStatus", post(crate::handlers::block::get_blockchain_status))
}

/// 网络相关路由
fn _network_routes() -> Router {
    Router::new()
        .route("/getPeers", post(crate::handlers::network::get_peers))
        .route("/getPeer", post(crate::handlers::network::get_peer))
}

/// 健康检查
async fn health_check() -> &'static str {
    "OK"
}
