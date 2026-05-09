//! Shared API state

use std::sync::Arc;

use crate::account_query_service::AccountQueryServiceApi;
use crate::bundler_service::BundlerServiceApi;
use crate::dgs_service::DGSServiceApi;
use orm::{BlockRepository, TransactionRepository, AssetRepository, AccountAssetRepository, DbPool};
use tx_engine::TransactionProcessor;
use ::account::AccountManager;
use p2p::P2PManager;

/// Forging service API trait
///
/// 定义 forging 相关的 API 接口，由 node 层的 ForgingService 实现。
/// 避免 http-api 对 node 的循环依赖。
#[async_trait::async_trait]
pub trait ForgingApi: Send + Sync {
    /// 注册锻造者
    async fn start_forging(&self, secret_phrase: &str) -> std::result::Result<(), String>;
    /// 注销锻造者
    async fn stop_forging(&self, secret_phrase: &str) -> std::result::Result<(), String>;
    /// 获取所有锻造者信息
    fn get_forgers(&self) -> Vec<ForgingInfo>;
    /// 获取锻造者数量
    fn get_forger_count(&self) -> usize;
}

/// 锻造者信息（API 层）
#[derive(Debug, Clone, serde::Serialize)]
pub struct ForgingInfo {
    pub account_id: u64,
    pub hit_time: u64,
    pub effective_balance: String,
    pub deadline: u64,
}

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
    pub forging_service: Option<Arc<dyn ForgingApi>>,
    /// Bundler 服务（用于交易打包管理）
    pub bundler_service: Arc<dyn BundlerServiceApi>,
    /// DGS 商城服务（用于商品和购买记录查询）
    pub dgs_service: Arc<dyn DGSServiceApi>,
    /// 账户查询服务（用于高级账户查询）
    pub account_query_service: Arc<dyn AccountQueryServiceApi>,
    pub db_pool: Option<DbPool>,
    /// 允许访问 API 的 IP 白名单 (对应 Java: API.allowedBotHosts)
    /// None 或 Vec::empty() 表示允许所有 (默认)
    pub allowed_bot_hosts: Vec<String>,
}
