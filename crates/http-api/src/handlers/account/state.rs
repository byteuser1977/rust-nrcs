//! Account API State
//!
//! 账户 API 的状态管理

use std::sync::Arc;
use account::AccountManager;
use orm::{AccountRepository, AccountAssetRepository};

/// 账户 API 状态
#[derive(Clone)]
pub struct AccountApiState {
    pub account_manager: Arc<dyn AccountManager>,
    pub account_repo: Arc<dyn AccountRepository>,
    pub account_asset_repo: Arc<dyn AccountAssetRepository>,
}

impl AccountApiState {
    pub fn new(
        account_manager: Arc<dyn AccountManager>,
        account_repo: Arc<dyn AccountRepository>,
        account_asset_repo: Arc<dyn AccountAssetRepository>,
    ) -> Self {
        Self {
            account_manager,
            account_repo,
            account_asset_repo,
        }
    }
}
