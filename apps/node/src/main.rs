//! NRCS Node - Main entry point
//!
//! 启动完整的区块链节点，包括：
//! - HTTP REST API 服务器
//! - P2P 网络
//! - 共识引擎
//! - 交易处理
//! - 账户管理

use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use p2p::{
    peer::{Peer, PeerState, Peers},
    websocket::{self, WebsocketServer, WebsocketConfig as WsServerConfig},
    Handler,
};

use blockchain_types::prelude::*;

use config::{Config, File, Environment};
use async_trait::async_trait;
use sqlx::PgPool;

use http_api::state::ApiState;
use account::AccountManager;
use tx_engine::TransactionProcessor;
use orm::{BlockRepository, TransactionRepository, AssetRepository, AccountAssetRepository, RepositoryResult, BlockModel, TransactionModel, AssetModel, AccountAssetModel};

/// 简单的 DummyBlockVerifier
struct DummyBlockVerifier;

#[async_trait]
impl p2p::handlers::BlockVerifier for DummyBlockVerifier {
    async fn verify_and_process(&self, _block: Block) -> Result<()> {
        Ok(())
    }
}

/// 模拟账户管理器
struct MockAccountManager;

#[async_trait]
impl AccountManager for MockAccountManager {
    async fn create_account(&self, _initial_balance: Option<Amount>) -> account::AccountResult<(crypto::KeyPair, AccountId, String)> {
        let kp = crypto::generate_keypair();
        let account_id = 1;
        let address = "test_address".to_string();
        Ok((kp, account_id, address))
    }
    
    async fn register_account(&self, _account_id: AccountId, _public_key: Vec<u8>) -> account::AccountResult<()> {
        Ok(())
    }
    
    async fn get_balance(&self, _account_id: AccountId) -> account::AccountResult<Amount> {
        Ok(1000)
    }
    
    async fn get_account_info(&self, account_id: AccountId) -> account::AccountResult<Account> {
        Ok(Account {
            id: account_id,
            address: Some("test_address".to_string()),
            balance: 1000,
            unconfirmed_balance: 1000,
            reserved_balance: 0,
            guaranteed_balance: 0,
            assets: Default::default(),
            properties: Default::default(),
            lease: None,
            created_at: 0,
            last_updated: 0,
            current_height: 0,
        })
    }
    
    async fn transfer(&self, _from: AccountId, _to: AccountId, _amount: Amount) -> account::AccountResult<()> {
        Ok(())
    }
    
    async fn credit(&self, _account_id: AccountId, _amount: Amount) -> account::AccountResult<()> {
        Ok(())
    }
    
    async fn debit(&self, _account_id: AccountId, _amount: Amount) -> account::AccountResult<()> {
        Ok(())
    }
    
    async fn get_and_increment_nonce(&self, _sender_id: AccountId) -> account::AccountResult<u64> {
        Ok(0)
    }
    
    async fn current_nonce(&self, _account_id: AccountId) -> account::AccountResult<u64> {
        Ok(0)
    }
    
    async fn mint_asset(&self, _asset_id: AssetId, _to: AccountId, _amount: Amount) -> account::AccountResult<()> {
        Ok(())
    }
    
    async fn burn_asset(&self, _asset_id: AssetId, _from: AccountId, _amount: Amount) -> account::AccountResult<()> {
        Ok(())
    }
    
    async fn get_public_key(&self, _account_id: AccountId) -> account::AccountResult<Option<blockchain_types::PublicKey>> {
        Ok(None)
    }
}

/// 模拟交易处理器
struct MockTxProcessor;

#[async_trait]
impl TransactionProcessor for MockTxProcessor {
    async fn validate(&self, _tx: &Transaction) -> tx_engine::ProcessorResult<()> {
        Ok(())
    }
    
    async fn apply(&self, _tx: &Transaction) -> tx_engine::ProcessorResult<()> {
        Ok(())
    }
    
    async fn execute(&self, _tx: &Transaction) -> tx_engine::ProcessorResult<tx_engine::TxReceiptInfo> {
        Ok(tx_engine::TxReceiptInfo {
            transaction_id: 0,
            status: tx_engine::TxStatus::Success,
            block_height: None,
            gas_used: 0,
            logs: vec![],
            contract_address: None,
            executed_at: 0,
        })
    }
}

/// 模拟区块仓库
struct MockBlockRepository;

#[async_trait]
impl orm::Repository<BlockModel> for MockBlockRepository {
    async fn insert(&self, _item: &BlockModel) -> RepositoryResult<()> {
        Ok(())
    }
    
    async fn find_by_id(&self, _db_id: i64) -> RepositoryResult<Option<BlockModel>> {
        Ok(None)
    }
    
    async fn update(&self, _item: &BlockModel) -> RepositoryResult<()> {
        Ok(())
    }
    
    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Ok(())
    }
    
    async fn find_all(&self, _limit: Option<i64>, _offset: Option<i64>) -> RepositoryResult<Vec<BlockModel>> {
        Ok(vec![])
    }
    
    async fn count(&self) -> RepositoryResult<i64> {
        Ok(0)
    }
}

#[async_trait]
impl BlockRepository for MockBlockRepository {
    async fn find_by_height(&self, _height: i32) -> RepositoryResult<Option<BlockModel>> {
        Ok(None)
    }
    
    async fn find_by_id_column(&self, _id: i64) -> RepositoryResult<Option<BlockModel>> {
        Ok(None)
    }
    
    async fn find_by_hash(&self, _hash: &[u8]) -> RepositoryResult<Option<BlockModel>> {
        Ok(None)
    }
    
    async fn find_latest(&self) -> RepositoryResult<Option<BlockModel>> {
        Ok(None)
    }
    
    async fn find_range(&self, _start_height: i32, _end_height: i32) -> RepositoryResult<Vec<BlockModel>> {
        Ok(vec![])
    }
    
    async fn find_by_generator(&self, _generator_id: i64) -> RepositoryResult<Vec<BlockModel>> {
        Ok(vec![])
    }
}

/// 模拟交易仓库
struct MockTransactionRepository;

#[async_trait]
impl orm::Repository<TransactionModel> for MockTransactionRepository {
    async fn insert(&self, _item: &TransactionModel) -> RepositoryResult<()> {
        Ok(())
    }
    
    async fn find_by_id(&self, _db_id: i64) -> RepositoryResult<Option<TransactionModel>> {
        Ok(None)
    }
    
    async fn update(&self, _item: &TransactionModel) -> RepositoryResult<()> {
        Ok(())
    }
    
    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Ok(())
    }
    
    async fn find_all(&self, _limit: Option<i64>, _offset: Option<i64>) -> RepositoryResult<Vec<TransactionModel>> {
        Ok(vec![])
    }
    
    async fn count(&self) -> RepositoryResult<i64> {
        Ok(0)
    }
}

#[async_trait]
impl TransactionRepository for MockTransactionRepository {
    async fn find_by_txid(&self, _id: i64) -> RepositoryResult<Option<TransactionModel>> {
        Ok(None)
    }
    
    async fn find_by_full_hash(&self, _full_hash: &[u8]) -> RepositoryResult<Option<TransactionModel>> {
        Ok(None)
    }
    
    async fn find_by_sender(&self, _sender_id: i64, _limit: i64) -> RepositoryResult<Vec<TransactionModel>> {
        Ok(vec![])
    }
    
    async fn find_by_recipient(&self, _recipient_id: i64, _limit: i64) -> RepositoryResult<Vec<TransactionModel>> {
        Ok(vec![])
    }
    
    async fn find_by_block(&self, _block_id: i64) -> RepositoryResult<Vec<TransactionModel>> {
        Ok(vec![])
    }
    
    async fn find_by_height(&self, _height: i32) -> RepositoryResult<Vec<TransactionModel>> {
        Ok(vec![])
    }
    
    async fn find_unconfirmed(&self, _limit: i64) -> RepositoryResult<Vec<TransactionModel>> {
        Ok(vec![])
    }
}

/// 模拟资产仓库
struct MockAssetRepository;

#[async_trait]
impl orm::Repository<AssetModel> for MockAssetRepository {
    async fn insert(&self, _item: &AssetModel) -> RepositoryResult<()> {
        Ok(())
    }
    
    async fn find_by_id(&self, _db_id: i64) -> RepositoryResult<Option<AssetModel>> {
        Ok(None)
    }
    
    async fn update(&self, _item: &AssetModel) -> RepositoryResult<()> {
        Ok(())
    }
    
    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Ok(())
    }
    
    async fn find_all(&self, _limit: Option<i64>, _offset: Option<i64>) -> RepositoryResult<Vec<AssetModel>> {
        Ok(vec![])
    }
    
    async fn count(&self) -> RepositoryResult<i64> {
        Ok(0)
    }
}

#[async_trait]
impl AssetRepository for MockAssetRepository {
    async fn find_by_asset_id(&self, _id: i64) -> RepositoryResult<Option<AssetModel>> {
        Ok(None)
    }
    
    async fn find_by_owner(&self, _owner_id: i64) -> RepositoryResult<Vec<AssetModel>> {
        Ok(vec![])
    }
    
    async fn find_by_height(&self, _height: i32) -> RepositoryResult<Vec<AssetModel>> {
        Ok(vec![])
    }
    
    async fn find_tradable(&self, _limit: i64) -> RepositoryResult<Vec<AssetModel>> {
        Ok(vec![])
    }
}

/// 模拟账户资产仓库
struct MockAccountAssetRepository;

#[async_trait]
impl orm::Repository<AccountAssetModel> for MockAccountAssetRepository {
    async fn insert(&self, _item: &AccountAssetModel) -> RepositoryResult<()> {
        Ok(())
    }
    
    async fn find_by_id(&self, _db_id: i64) -> RepositoryResult<Option<AccountAssetModel>> {
        Ok(None)
    }
    
    async fn update(&self, _item: &AccountAssetModel) -> RepositoryResult<()> {
        Ok(())
    }
    
    async fn delete(&self, _db_id: i64) -> RepositoryResult<()> {
        Ok(())
    }
    
    async fn find_all(&self, _limit: Option<i64>, _offset: Option<i64>) -> RepositoryResult<Vec<AccountAssetModel>> {
        Ok(vec![])
    }
    
    async fn count(&self) -> RepositoryResult<i64> {
        Ok(0)
    }
}

#[async_trait]
impl AccountAssetRepository for MockAccountAssetRepository {
    async fn find_by_account(&self, _account_id: i64) -> RepositoryResult<Vec<AccountAssetModel>> {
        Ok(vec![])
    }
    
    async fn find_by_asset(&self, _asset_id: i64) -> RepositoryResult<Vec<AccountAssetModel>> {
        Ok(vec![])
    }
    
    async fn find_by_account_and_asset(&self, _account_id: i64, _asset_id: i64) -> RepositoryResult<Option<AccountAssetModel>> {
        Ok(None)
    }
    
    async fn update_quantity(&self, _account_id: i64, _asset_id: i64, _quantity: i64, _height: i32) -> RepositoryResult<()> {
        Ok(())
    }
    
    async fn increase_quantity(&self, _account_id: i64, _asset_id: i64, _delta: i64) -> RepositoryResult<()> {
        Ok(())
    }
    
    async fn decrease_quantity(&self, _account_id: i64, _asset_id: i64, _delta: i64) -> RepositoryResult<()> {
        Ok(())
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

#[derive(Debug, Clone, serde::Deserialize)]
struct DatabaseConfig {
    url: String,
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
    info!("Connecting to database: {}", database_url.split('@').last().unwrap_or("hidden"));
    
    let pool = PgPool::connect(&database_url)
        .await
        .context("Failed to connect to database")?;
    
    // 暂不创建创世区块，等待从 Java-NRCS 同步
    // orm::ensure_genesis(&pool).await.context("Failed to create genesis block")?;
    info!("Database connected, genesis creation skipped - awaiting sync from Java-NRCS");

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

    // 初始化 P2P 管理器
    let peers = Arc::new(Peers::new(my_peer.clone()));
    let block_verifier: Arc<dyn p2p::handlers::BlockVerifier> = Arc::new(DummyBlockVerifier);
    let handler = Arc::new(Handler::new(Arc::clone(&peers), Arc::clone(&block_verifier)));

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

    // 启动 HTTP API 服务器
    let api_state = ApiState {
        account_manager: Arc::new(MockAccountManager),
        tx_processor: Arc::new(MockTxProcessor),
        block_repo: Arc::new(MockBlockRepository),
        tx_repo: Arc::new(MockTransactionRepository),
        asset_repo: Arc::new(MockAssetRepository),
        account_asset_repo: Arc::new(MockAccountAssetRepository),
        p2p_manager: None,
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