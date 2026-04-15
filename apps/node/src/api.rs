//! API Service (simple placeholder)

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{routing::get, Router, Server};
use tracing::info;

pub struct ApiService {
    listen_addr: SocketAddr,
}

impl ApiService {
    pub fn new(
        listen_addr: SocketAddr,
        _account_manager: Arc<dyn account::AccountManager>,
        _tx_processor: Arc<dyn tx_engine::TransactionProcessor>,
        _block_repo: Arc<dyn orm::BlockRepository>,
        _p2p_service: Option<Arc<()>>,
    ) -> Self {
        Self {
            listen_addr,
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        info!("Starting API server on {}", self.listen_addr);
        
        let app = Router::new()
            .route("/", get(|| async { "NRCS Node API\n" }))
            .route("/health", get(|| async { "OK\n" }));

        Server::bind(&self.listen_addr)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }
}
