//! Block API State
//!
//! 区块 API 的状态管理

use std::sync::Arc;
use orm::{BlockRepository, TransactionRepository};

/// 区块 API 状态
#[derive(Clone)]
pub struct BlockApiState {
    pub block_repo: Arc<dyn BlockRepository>,
    pub tx_repo: Arc<dyn TransactionRepository>,
}

impl BlockApiState {
    pub fn new(
        block_repo: Arc<dyn BlockRepository>,
        tx_repo: Arc<dyn TransactionRepository>,
    ) -> Self {
        Self {
            block_repo,
            tx_repo,
        }
    }
}
