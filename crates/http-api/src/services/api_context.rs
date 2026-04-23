//! API Context
//!
//! API 上下文管理

use std::sync::Arc;
use crate::handlers::{
    account::AccountApiState,
    transaction::TransactionApiState,
    block::BlockApiState,
    network::NetworkApiState,
};

/// API 上下文（用于依赖注入）
pub struct ApiContext {
    pub account_api: AccountApiState,
    pub transaction_api: TransactionApiState,
    pub block_api: BlockApiState,
    pub network_api: NetworkApiState,
}

impl ApiContext {
    pub fn new(
        account_api: AccountApiState,
        transaction_api: TransactionApiState,
        block_api: BlockApiState,
        network_api: NetworkApiState,
    ) -> Self {
        Self {
            account_api,
            transaction_api,
            block_api,
            network_api,
        }
    }
}
