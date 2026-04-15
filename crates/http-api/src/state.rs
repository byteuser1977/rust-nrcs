//! Shared API state

use std::sync::Arc;

use blockchain_types::*;
use orm::BlockRepository;
use tx_engine::DatabaseTransactionProcessor;
use ::account::AccountManager;

/// Global API state (shared across all handlers)
#[derive(Clone)]
pub struct ApiState {
    pub account_manager: Arc<dyn AccountManager>,
    pub tx_processor: Arc<DatabaseTransactionProcessor>,
    pub block_repo: Arc<dyn BlockRepository>,
}