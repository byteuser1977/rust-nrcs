//! NRCS Node - Main entry point
//!
//! 启动完整的区块链节点，包括：
//! - HTTP REST API 服务器
//! - P2P 网络
//! - 共识引擎
//! - 交易处理
//! - 账户管理

use anyhow::{Context, Result};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use p2p::{
    peer::{Peer, PeerState, Peers},
    websocket::{self, WebsocketServer, WebsocketConfig as WsServerConfig},
    Handler,
    daemon::BlockchainSyncDaemon,
    config::P2PConfig as GlobalP2PConfig,
    block_apply::BlockRewardApplicator,
};

use config::{Config, File, Environment};
use sqlx::SqlitePool;

use http_api::state::ApiState;
use account::{AccountManager, AccountConfig, DatabaseAccountManager, AccountStore, PgAccountStore};
use tx_engine::{TransactionProcessor, DatabaseTransactionProcessor};
use orm::{BlockRepository, TransactionRepository, AssetRepository, AssetTransferRepository, AccountAssetRepository, AccountRepository, PublicKeyRepository,
         AccountGuaranteedBalanceRepository, AccountLedgerRepository,
         TaggedDataExtendRepository, TaggedTimestampRepository,
         PhasingPollRepository, PhasingVoteRepository, AccountControlPhasingRepository,
         AccountInfoRepository,
         // 新增导入
         AliasRepository, AliasOfferRepository,
         PollRepository, VoteRepository,
         TaggedDataRepository, TaggedDataTagRepository,
         ContractReferenceRepository,
         AskOrderRepository, BidOrderRepository,
         CurrencyRepository, AccountCurrencyRepository, CurrencyTransferRepository,
         AssetPropertyRepository, AccountPropertyRepository,
         // Digital Goods
         GoodsRepository, PurchaseRepository,
         // Shuffling
         ShufflingRepository,
         // Account Lease
         AccountLeaseRepository};
use orm::repository::sqlite::SqliteAccountGuaranteedBalanceRepository;

use p2p::BlockchainVerifier;

mod forging;

enum DatabaseType {
    PostgreSQL,
    SQLite,
}

impl DatabaseType {
    fn from_url(url: &str) -> Self {
        if url.starts_with("sqlite://") {
            DatabaseType::SQLite
        } else {
            DatabaseType::PostgreSQL
        }
    }
}

/// 节点配置结构（与 TOML 映射）
#[derive(Debug, Clone, serde::Deserialize)]
struct NodeConfig {
    p2p: P2PConfig,
    api: APIConfig,
    websocket: WsAppConfig,
    database: DatabaseConfig,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
struct P2PConfig {
    listen_addr: String,
    external_addr: Option<String>,
    #[serde(default)]
    bootstrap_nodes: Vec<String>,
    max_connections: usize,
    connection_ttl_secs: u64,
    protocol_id: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct APIConfig {
    host: String,
    port: u16,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
struct WsAppConfig {
    enabled: bool,
    port: u16,
}

#[derive(Debug, Clone, Deserialize)]
struct DatabaseConfig {
    url: String,
    #[allow(dead_code)]
    max_connections: u32,
}

/// 解析 multiaddr 或 `host:port` 为 SocketAddr
fn parse_socket_addr(addr: &str) -> anyhow::Result<SocketAddr> {
    // 如果是 multiaddr 格式: /ip4/192.168.2.164/tcp/17974
    if addr.starts_with('/') {
        let parts: Vec<&str> = addr.split('/').filter(|s| !s.is_empty()).collect();
        if parts.len() >= 4 {
            let ip = parts[1];
            let port_str = parts[3];
            let port: u16 = port_str.parse()?;
            let full = format!("{}:{}", ip, port);
            return Ok(full.parse()?);
        }
    }
    // 否则直接解析 host:port
    Ok(addr.parse()?)
}

impl NodeConfig {
    /// 加载配置：默认读取 `config/default.toml`，覆盖 `config/local.toml`
    #[allow(deprecated)]
    fn load() -> anyhow::Result<Self> {
        let mut cfg = Config::default();
        cfg.merge(File::with_name("config/default"))?;
        cfg.merge(File::with_name("config/local"))?;
        cfg.merge(Environment::with_prefix("NRCS"))?;
        let config: NodeConfig = cfg.try_deserialize()?;
        Ok(config)
    }

    fn p2p_listen_addr(&self) -> anyhow::Result<SocketAddr> {
        parse_socket_addr(&self.p2p.listen_addr)
    }

    fn p2p_external_addr(&self) -> anyhow::Result<Option<SocketAddr>> {
        match &self.p2p.external_addr {
            Some(addr) => Ok(Some(parse_socket_addr(addr)?)),
            None => Ok(None),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    let filter = EnvFilter::from_default_env()
        .add_directive("nrcs_node=debug".parse()?)
        .add_directive("p2p=debug".parse()?);
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting NRCS Node...");

    // 加载配置
    let cfg = NodeConfig::load().context("Failed to load configuration")?;
    info!(
        "Config loaded: p2p={}, api={}:{}",
        cfg.p2p.listen_addr, cfg.api.host, cfg.api.port
    );

    // 初始化数据库
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| cfg.database.url.clone());
    let db_type = DatabaseType::from_url(&database_url);
    info!("Connecting to database: {}", database_url.split('@').next_back().unwrap_or(&database_url));
    
    match db_type {
        DatabaseType::PostgreSQL => {
            return Err(anyhow::anyhow!("PostgreSQL is temporarily disabled. Please use SQLite."));
        }
        DatabaseType::SQLite => {
            let pool = SqlitePool::connect(&database_url)
                .await
                .context("Failed to connect to SQLite database")?;
            
            info!("SQLite database connected, running migrations...");
            
            let migration_sql = include_str!("../../../migrations/sqlite/0.sql");
            for statement in migration_sql.split(';') {
                let statement = statement.trim();
                if !statement.is_empty() && !statement.starts_with("/*") {
                    if let Err(e) = sqlx::query(statement).execute(&pool).await {
                        if !e.to_string().contains("already exists") {
                            warn!("Migration warning: {}", e);
                        }
                    }
                }
            }
            info!("SQLite migrations completed");
            
            // 创建数据库仓库
            let block_repo: Arc<dyn BlockRepository> = Arc::new(orm::SqliteBlockRepository::new(pool.clone()));
            let tx_repo: Arc<dyn TransactionRepository> = Arc::new(orm::SqliteTransactionRepository::new(pool.clone()));
            let account_repo: Arc<dyn AccountRepository> = Arc::new(orm::SqliteAccountRepository::new(pool.clone()));
            let asset_repo: Arc<dyn AssetRepository> = Arc::new(orm::SqliteAssetRepository::new(pool.clone()));
            let account_asset_repo: Arc<dyn AccountAssetRepository> = Arc::new(orm::SqliteAccountAssetRepository::new(pool.clone()));
            let asset_transfer_repo: Arc<dyn AssetTransferRepository> = Arc::new(orm::SqliteAssetTransferRepository::new(pool.clone()));
            let public_key_repo: Arc<dyn PublicKeyRepository> = Arc::new(orm::SqlitePublicKeyRepository::new(pool.clone()));
            let ledger_repo: Arc<dyn AccountLedgerRepository> = Arc::new(orm::SqliteAccountLedgerRepository::new(pool.clone()));
            let guaranteed_balance_repo: Arc<dyn AccountGuaranteedBalanceRepository> = Arc::new(
                SqliteAccountGuaranteedBalanceRepository::new(pool.clone())
            );

            // 新增：完整交易处理所需的Repository
            let alias_repo: Arc<dyn AliasRepository> = Arc::new(orm::SqliteAliasRepository::new(pool.clone()));
            let alias_offer_repo: Arc<dyn AliasOfferRepository> = Arc::new(orm::SqliteAliasOfferRepository::new(pool.clone()));
            let poll_repo: Arc<dyn PollRepository> = Arc::new(orm::SqlitePollRepository::new(pool.clone()));
            let vote_repo: Arc<dyn VoteRepository> = Arc::new(orm::SqliteVoteRepository::new(pool.clone()));
            // let account_property_repo: Arc<dyn AccountPropertyRepository> = ...  // 暂时注释
            let tagged_data_repo: Arc<dyn TaggedDataRepository> = Arc::new(orm::SqliteTaggedDataRepository::new(pool.clone()));
            let tagged_data_tag_repo: Arc<dyn TaggedDataTagRepository> = Arc::new(orm::SqliteTaggedDataTagRepository::new(pool.clone()));
            let tagged_data_extend_repo: Arc<dyn TaggedDataExtendRepository> = Arc::new(orm::SqliteTaggedDataExtendRepository::new(pool.clone()));
            let tagged_timestamp_repo: Arc<dyn TaggedTimestampRepository> = Arc::new(orm::SqliteTaggedTimestampRepository::new(pool.clone()));
            let contract_ref_repo: Arc<dyn ContractReferenceRepository> = Arc::new(orm::SqliteContractReferenceRepository::new(pool.clone()));
            let ask_order_repo: Arc<dyn AskOrderRepository> = Arc::new(orm::SqliteAskOrderRepository::new(pool.clone()));
            let bid_order_repo: Arc<dyn BidOrderRepository> = Arc::new(orm::SqliteBidOrderRepository::new(pool.clone()));
            let currency_repo: Arc<dyn CurrencyRepository> = Arc::new(orm::SqliteCurrencyRepository::new(pool.clone()));
            let account_currency_repo: Arc<dyn AccountCurrencyRepository> = Arc::new(orm::SqliteAccountCurrencyRepository::new(pool.clone()));
            let currency_transfer_repo: Arc<dyn CurrencyTransferRepository> = Arc::new(orm::SqliteCurrencyTransferRepository::new(pool.clone()));
            let asset_property_repo: Arc<dyn AssetPropertyRepository> = Arc::new(orm::SqliteAssetPropertyRepository::new(pool.clone()));
            let account_property_repo: Arc<dyn AccountPropertyRepository> = Arc::new(orm::SqliteAccountPropertyRepository::new(pool.clone()));
            let phasing_poll_repo: Arc<dyn PhasingPollRepository> = Arc::new(orm::SqlitePhasingPollRepository::new(pool.clone()));
            let phasing_vote_repo: Arc<dyn PhasingVoteRepository> = Arc::new(orm::SqlitePhasingVoteRepository::new(pool.clone()));
            let account_control_phasing_repo: Arc<dyn AccountControlPhasingRepository> = Arc::new(orm::SqliteAccountControlPhasingRepository::new(pool.clone()));
            let account_info_repo: Arc<dyn AccountInfoRepository> = Arc::new(orm::SqliteAccountInfoRepository::new(pool.clone()));
            // Exchange和Mint使用专用Repository（P1优化完成）
            let exchange_request_repo: Arc<dyn orm::Repository<orm::models::ExchangeRequestModel>> =
                Arc::new(orm::SqliteExchangeRequestRepository::new(pool.clone()));
            let currency_mint_repo: Arc<dyn orm::Repository<orm::models::CurrencyMintModel>> =
                Arc::new(orm::SqliteCurrencyMintRepository::new(pool.clone()));
            // Digital Goods
            let goods_repo: Arc<dyn GoodsRepository> = Arc::new(orm::SqliteGoodsRepository::new(pool.clone()));
            let purchase_repo: Arc<dyn PurchaseRepository> = Arc::new(orm::SqlitePurchaseRepository::new(pool.clone()));
            // Shuffling
            let shuffling_repo: Arc<dyn ShufflingRepository> = Arc::new(orm::SqliteShufflingRepository::new(pool.clone()));
            // Account Lease
            let account_lease_repo: Arc<dyn AccountLeaseRepository> = Arc::new(orm::SqliteAccountLeaseRepository::new(pool.clone()));

            // 确保创世区块存在
            orm::genesis::ensure_genesis(
                &*block_repo,
                &*account_repo,
                &*tx_repo,
                &*ledger_repo,
                &*guaranteed_balance_repo,
            ).await.context("Failed to create genesis block")?;
            info!("Genesis block ensured");

            start_node(cfg, pool, block_repo, tx_repo, asset_repo, account_asset_repo, asset_transfer_repo, account_repo, public_key_repo, ledger_repo, guaranteed_balance_repo,
                alias_repo, alias_offer_repo, poll_repo, vote_repo, account_property_repo, account_info_repo, phasing_poll_repo, phasing_vote_repo, account_control_phasing_repo, tagged_data_repo, tagged_data_tag_repo, tagged_data_extend_repo, tagged_timestamp_repo, contract_ref_repo, ask_order_repo, bid_order_repo, currency_repo, account_currency_repo, currency_transfer_repo, asset_property_repo,
                exchange_request_repo, currency_mint_repo,
                goods_repo, purchase_repo,
                shuffling_repo,
                account_lease_repo
            ).await
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn start_node(
    cfg: NodeConfig,
    pool: SqlitePool,
    block_repo: Arc<dyn BlockRepository>,
    tx_repo: Arc<dyn TransactionRepository>,
    asset_repo: Arc<dyn AssetRepository>,
    account_asset_repo: Arc<dyn AccountAssetRepository>,
    asset_transfer_repo: Arc<dyn AssetTransferRepository>,
    account_repo: Arc<dyn AccountRepository>,
    public_key_repo: Arc<dyn PublicKeyRepository>,
    ledger_repo: Arc<dyn AccountLedgerRepository>,
    guaranteed_balance_repo: Arc<dyn AccountGuaranteedBalanceRepository>,
    // 新增参数
    alias_repo: Arc<dyn AliasRepository>,
    alias_offer_repo: Arc<dyn AliasOfferRepository>,
    poll_repo: Arc<dyn PollRepository>,
    vote_repo: Arc<dyn VoteRepository>,
    account_property_repo: Arc<dyn AccountPropertyRepository>,
    account_info_repo: Arc<dyn AccountInfoRepository>,
    phasing_poll_repo: Arc<dyn PhasingPollRepository>,
    phasing_vote_repo: Arc<dyn PhasingVoteRepository>,
    account_control_phasing_repo: Arc<dyn AccountControlPhasingRepository>,
    tagged_data_repo: Arc<dyn TaggedDataRepository>,
    tagged_data_tag_repo: Arc<dyn TaggedDataTagRepository>,
    tagged_data_extend_repo: Arc<dyn TaggedDataExtendRepository>,
    tagged_timestamp_repo: Arc<dyn TaggedTimestampRepository>,
    contract_ref_repo: Arc<dyn ContractReferenceRepository>,
    ask_order_repo: Arc<dyn AskOrderRepository>,
    bid_order_repo: Arc<dyn BidOrderRepository>,
    currency_repo: Arc<dyn CurrencyRepository>,
    account_currency_repo: Arc<dyn AccountCurrencyRepository>,
    currency_transfer_repo: Arc<dyn CurrencyTransferRepository>,
    asset_property_repo: Arc<dyn AssetPropertyRepository>,
    // 新增：Exchange和Mint（P1优化完成）
    exchange_request_repo: Arc<dyn orm::Repository<orm::models::ExchangeRequestModel>>,
    currency_mint_repo: Arc<dyn orm::Repository<orm::models::CurrencyMintModel>>,
    // Digital Goods
    goods_repo: Arc<dyn GoodsRepository>,
    purchase_repo: Arc<dyn PurchaseRepository>,
    // Shuffling
    shuffling_repo: Arc<dyn ShufflingRepository>,
    // Account Lease
    account_lease_repo: Arc<dyn AccountLeaseRepository>,
) -> Result<()> {
    // 创建交易处理器（完整版本 - 支持所有交易类型）
    let tx_processor: Arc<dyn TransactionProcessor> = Arc::new(DatabaseTransactionProcessor::new(
        Arc::clone(&account_repo),
        Arc::clone(&account_asset_repo),
        Arc::clone(&asset_repo),
        Arc::clone(&asset_transfer_repo),
        Arc::clone(&tx_repo),
        Arc::clone(&guaranteed_balance_repo),
        Arc::clone(&ledger_repo),
        // 新增：完整交易处理所需的Repository
        Arc::clone(&alias_repo),
        Arc::clone(&alias_offer_repo),
        Arc::clone(&poll_repo),
        Arc::clone(&vote_repo),
        Arc::clone(&account_property_repo),
        Arc::clone(&account_info_repo),
        Arc::clone(&phasing_poll_repo),
        Arc::clone(&phasing_vote_repo),
        Arc::clone(&account_control_phasing_repo),
        Arc::clone(&tagged_data_repo),
        Arc::clone(&tagged_data_tag_repo),
        Arc::clone(&tagged_data_extend_repo),
        Arc::clone(&tagged_timestamp_repo),
        Arc::clone(&contract_ref_repo),
        Arc::clone(&ask_order_repo),
        Arc::clone(&bid_order_repo),
        Arc::clone(&currency_repo),
        Arc::clone(&account_currency_repo),
        Arc::clone(&currency_transfer_repo),
        Arc::clone(&asset_property_repo),
        // 新增：Exchange和Mint
        exchange_request_repo,
        currency_mint_repo,
        // Digital Goods
        goods_repo,
        purchase_repo,
        // Shuffling
        shuffling_repo,
        // Account Lease
        account_lease_repo,
    ));

    // 创建区块奖励应用器
    let block_reward_applicator = Arc::new(BlockRewardApplicator::new(
        Arc::clone(&account_repo),
        Arc::clone(&block_repo),
        Arc::clone(&public_key_repo),
    ));

    // 创建区块验证器（包含完整的两阶段提交逻辑）
    let block_verifier: Arc<dyn p2p::handlers::BlockVerifier> = Arc::new(
        BlockchainVerifier::new(
            Arc::clone(&block_repo),
            Arc::clone(&tx_repo),
            Arc::clone(&tx_processor),
            Arc::clone(&block_reward_applicator),
            pool.clone(),
        )
    );

    // 解析本机 P2P 地址
    let listen_addr: SocketAddr = cfg.p2p_listen_addr()?;
    let external_addr = cfg.p2p_external_addr()?;

    // 构建本节点 Peer 信息
    let mut my_peer = Peer::new(listen_addr, false);
    my_peer.announced_address = external_addr.map(|a| a.to_string());
    my_peer.version = Some("0.1.0".to_string());
    my_peer.application = Some("NRCS".to_string());
    my_peer.platform = Some("Rust".to_string());
    my_peer.services = 4; // API service bit (0b100)
    my_peer.api_port = Some(cfg.api.port);
    my_peer.state = PeerState::Disconnected;

    // 初始化 P2P 管理器（使用完整仓库支持）
    let peers = Arc::new(Peers::new(my_peer.clone()));
    let mut p2p_config = GlobalP2PConfig::default();
    p2p_config.listen_addr = listen_addr;
    let p2p_config = Arc::new(p2p_config);
    let handler = Arc::new(Handler::with_repositories(
        Arc::clone(&peers),
        Arc::clone(&block_verifier),
        Arc::clone(&block_repo),
        Arc::clone(&tx_repo),
        Arc::clone(&tx_processor),
        Arc::clone(&p2p_config),
    ));

    // 启动出站连接任务（连接 bootstrap 节点）
    if !cfg.p2p.bootstrap_nodes.is_empty() {
        let peers_clone = Arc::clone(&peers);
        let handler_clone = Arc::clone(&handler);
        let bootstrap_addrs: Vec<Result<SocketAddr, _>> = cfg
            .p2p
            .bootstrap_nodes
            .iter()
            .map(|addr| parse_socket_addr(addr))
            .collect();
        info!("[BOOTSTRAP] Loaded {} bootstrap nodes", bootstrap_addrs.len());

        tokio::spawn(async move {
            info!("[BOOTSTRAP_TASK] Starting in 2s...");
            tokio::time::sleep(Duration::from_secs(2)).await;
            info!("[BOOTSTRAP_TASK] Starting connections");

            for addr_res in bootstrap_addrs {
                match addr_res {
                    Ok(addr) => {
                        info!("[BOOTSTRAP] Connecting to {}...", addr);
                        if let Err(e) = websocket::WebsocketClient::connect(addr, peers_clone.clone(), handler_clone.clone()).await {
                            error!("[BOOTSTRAP] Failed to connect to {}: {}", addr, e);
                        } else {
                            info!("[BOOTSTRAP] connect() returned OK for {}", addr);
                        }
                    }
                    Err(e) => warn!("[BOOTSTRAP] Invalid address: {}", e),
                }
            }
            info!("[BOOTSTRAP_TASK] All connections attempted");
        });
    }

    // 启动 WebSocket P2P 服务器
    if cfg.websocket.enabled {
        let ws_config = WsServerConfig {
            listen_addr,
            max_connections: cfg.p2p.max_connections,
        };
        let ws_server = WebsocketServer::new(ws_config, Arc::clone(&peers), handler);
        tokio::spawn(async move {
            if let Err(e) = ws_server.serve().await {
                eprintln!("WebSocket server error: {}", e);
            }
        });
        info!("P2P WebSocket server started on {}", listen_addr);
    }

    // 启动区块链同步守护进程
    let sync_config = GlobalP2PConfig::default();
    let sync_daemon = BlockchainSyncDaemon::new(sync_config);
    sync_daemon.start(Arc::clone(&peers), Arc::clone(&block_verifier)).await;
    info!("Blockchain sync daemon started");

    // 启动 HTTP API 服务器
    // 创建账户存储
    let account_store: Arc<dyn AccountStore> = Arc::new(PgAccountStore::new(
        Arc::clone(&account_repo),
        Arc::clone(&public_key_repo),
    ));
    
    // 创建账户管理器
    let account_config = AccountConfig::default();
    let account_manager: Arc<dyn AccountManager> = Arc::new(DatabaseAccountManager::new(
        account_store,
        Arc::clone(&account_repo),
        Arc::clone(&account_asset_repo),
        Arc::clone(&public_key_repo),
        account_config,
    ));
    
    // 创建锻造服务
    let forging_service = Arc::new(forging::ForgingService::new(
        Arc::clone(&block_repo),
        Arc::clone(&tx_repo),
        Arc::clone(&account_repo),
        Arc::clone(&block_verifier),
        Arc::clone(&peers),
        Arc::clone(&p2p_config),
    ));

    // 启动出块循环
    let forging_for_loop = Arc::clone(&forging_service);
    tokio::spawn(async move {
        forging_for_loop.run().await;
    });
    info!("Forging service started");

    let api_state = ApiState {
        account_manager,
        tx_processor,
        block_repo,
        tx_repo,
        asset_repo,
        account_asset_repo,
        p2p_manager: None,
        forging_service: Some(forging_service),
    };
    
    let api_addr: SocketAddr = format!("{}:{}", cfg.api.host, cfg.api.port)
        .parse()
        .context("Invalid API server address")?;
    
    let api_router = http_api::routes::create_router(api_state);
    
    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(api_addr).await.unwrap();
        info!("HTTP API server starting on {}", api_addr);
        if let Err(e) = axum::serve(listener, api_router).await {
            error!("HTTP API server error: {}", e);
        }
    });
    info!("HTTP API server configured on {}:{}", cfg.api.host, cfg.api.port);

    info!("Node running successfully!");

    // 保持运行
    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}

/// 获取当前时间戳（秒）
#[allow(dead_code)]
fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}