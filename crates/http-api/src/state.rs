//! Shared API state

use std::sync::Arc;

use blockchain_types::*;
use orm::BlockRepository;
use tx_engine::TransactionProcessor;
use account::AccountManager;
use p2p::P2PService;

/// Global API state (shared across all handlers)
#[derive(Clone)]
pub struct ApiState {
    pub account_manager: Arc<dyn AccountManager>,
    pub tx_processor: Arc<dyn TransactionProcessor>,
    pub block_repo: Arc<dyn BlockRepository>,
    pub p2p_service: Option<Arc<P2PService>>,
}