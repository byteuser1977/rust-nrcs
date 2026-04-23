//! Transaction API State
//!
//! 交易 API 的状态管理

use std::sync::Arc;
use tx_engine::TransactionProcessor;
use orm::{TransactionRepository, AccountRepository};

/// 交易 API 状态
#[derive(Clone)]
pub struct TransactionApiState {
    pub tx_processor: Arc<dyn TransactionProcessor>,
    pub tx_repo: Arc<dyn TransactionRepository>,
    pub account_repo: Arc<dyn AccountRepository>,
}

impl TransactionApiState {
    pub fn new(
        tx_processor: Arc<dyn TransactionProcessor>,
        tx_repo: Arc<dyn TransactionRepository>,
        account_repo: Arc<dyn AccountRepository>,
    ) -> Self {
        Self {
            tx_processor,
            tx_repo,
            account_repo,
        }
    }
}
