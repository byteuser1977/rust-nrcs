//! Shared API state

use std::sync::Arc;

use blockchain_types::*;
use orm::{BlockRepository, TransactionRepository, AssetRepository, AccountAssetRepository};
use tx_engine::TransactionProcessor;
use ::account::AccountManager;
use p2p::P2PManager;

/// Global API state (shared across all handlers)
#[derive(Clone)]
pub struct ApiState {
    pub account_manager: Arc<dyn AccountManager>,
    pub tx_processor: Arc<dyn TransactionProcessor>,
    pub block_repo: Arc<dyn BlockRepository>,
    pub tx_repo: Arc<dyn TransactionRepository>,
    pub asset_repo: Arc<dyn AssetRepository>,
    pub account_asset_repo: Arc<dyn AccountAssetRepository>,
    pub p2p_manager: Option<Arc<P2PManager>>,
}
