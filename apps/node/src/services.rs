//! Node services coordination

use std::sync::Arc;

use anyhow::Result;

use nrcs_node::config::NodeConfig;
use orm::PgPool;

/// Main node service that coordinates all components
pub struct NodeService {
    pub db_pool: PgPool,
    pub account_manager: Arc<dyn account::AccountManager>,
    pub tx_processor: Arc<dyn tx_engine::TransactionProcessor>,
    pub p2p_service: Option<Arc<p2p::P2PService>>,
    pub chain_service: Arc<chain::ChainService>,
    pub api_service: Option<Arc<api::ApiService>>,
}

impl NodeService {
    /// Initialize all services
    pub async fn init(config: NodeConfig) -> Result<(Arc<p2p::P2PService>, Arc<chain::ChainService>, Arc<api::ApiService>)> {
        // 1. 初始化数据库连接池
        let db_pool = PgPool::connect(&config.database.url).await?;

        // 2. 初始化数据仓库
        let account_repo = Arc::new(orm::PgAccountRepository::new(db_pool.clone())) as Arc<dyn orm::AccountRepository>;
        let account_asset_repo = Arc::new(orm::PgAccountAssetRepository::new(db_pool.clone())) as Arc<dyn orm::AccountAssetRepository>;
        let tx_repo = Arc::new(orm::PgTransactionRepository::new(db_pool.clone())) as Arc<dyn orm::TransactionRepository>;
        let receipt_repo = Arc::new(orm::PgTransactionReceiptRepository::new(db_pool.clone())) as Arc<dyn orm::TransactionReceiptRepository>;
        let block_repo = Arc::new(orm::PgBlockRepository::new(db_pool.clone())) as Arc<dyn orm::BlockRepository>;

        // 3. 初始化账户存储
        let account_store = Arc::new(account::repository::PgAccountStore::new(account_repo.clone()));

        // 4. 初始化账户管理器
        let account_config = account::AccountConfig {
            enable_address: true,
            initial_balance: 0,
            admin_account_id: None,
        };
        let account_manager = Arc::new(account::manager::DatabaseAccountManager::new(
            account_store,
            account_repo.clone(),
            account_asset_repo.clone(),
            account_config,
        ));

        // 5. 初始化交易处理器
        let tx_processor = Arc::new(tx_engine::processor::DatabaseTransactionProcessor::new(
            account_repo.clone(),
            account_asset_repo.clone(),
            tx_repo.clone(),
            receipt_repo,
        ));

        // 6. 初始化链服务
        let chain_service = Arc::new(chain::ChainService::new(
            block_repo.clone(),
            tx_repo.clone(),
            tx_processor.clone(),
            account_manager.clone(),
        ));

        // 7. 初始化 P2P 服务（可选）
        let p2p_service = if config.p2p.enabled {
            let p2p = Arc::new(p2p::P2PService::new(
                config.p2p.listen_addr,
                config.p2p.seed_nodes,
                tx_processor.clone(),
                chain_service.clone(),
            ));
            p2p.start().await?;
            Some(p2p)
        } else {
            None
        };

        // 8. 初始化 API 服务
        let api_service = if config.api.enabled {
            let api = Arc::new(api::ApiService::new(
                config.api.listen_addr,
                account_manager.clone(),
                tx_processor.clone(),
                block_repo.clone(),
                p2p_service.clone(),
            ));
            Some(api)
        } else {
            None
        };

        Ok((p2p_service.unwrap(), chain_service, api_service.unwrap()))
    }

    /// Start all running services
    pub async fn start(
        p2p_service: Arc<p2p::P2PService>,
        chain_service: Arc<chain::ChainService>,
        api_service: Arc<api::ApiService>,
    ) -> Result<()> {
        // 启动链同步服务
        chain_service.start_sync().await?;

        // 启动 API 服务器（会阻塞）
        api_service.start().await?;

        Ok(())
    }
}