//! API Service

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{routing::{get, post}, Router, Server};
use blockchain_types::*;
use orm::{BlockRepository, TransactionRepository};
use tx_engine::TransactionProcessor;
use account::AccountManager;

use super::handlers::{self, ApiState};

pub struct ApiService {
    listen_addr: SocketAddr,
    state: ApiState,
}

impl ApiService {
    pub fn new(
        listen_addr: SocketAddr,
        account_manager: Arc<dyn AccountManager>,
        tx_processor: Arc<dyn TransactionProcessor>,
        block_repo: Arc<dyn BlockRepository>,
        p2p_service: Option<Arc<p2p::P2PService>>,
    ) -> Self {
        Self {
            listen_addr,
            state: ApiState {
                account_manager,
                tx_processor,
                block_repo,
                p2p_service,
            },
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        // 构建路由
        let app = Router::new()
            .route("/health", get(handlers::system::health_check))
            .route("/api/v1/accounts", post(handlers::account::create_account))
            .route("/api/v1/accounts/:address", get(handlers::account::get_account))
            .route("/api/v1/accounts/:id/balance", get(handlers::account::get_balance))
            .route("/api/v1/accounts/transfer", post(handlers::account::transfer))
            .route("/api/v1/transactions", post(handlers::transaction::submit_transaction))
            .route("/api/v1/blocks/latest", get(handlers::block::get_latest_block))
            .route("/api/v1/blocks/:height", get(handlers::block::get_block_by_height))
            .route("/api/v1/node/info", get(handlers::node::get_node_info))
            .with_state(self.state.clone());

        info!("HTTP API listening on http://{}", self.listen_addr);

        // 启动服务器
        Server::bind(&self.listen_addr).serve(app.into_make_service()).await?;

        Ok(())
    }
}