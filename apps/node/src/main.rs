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
use account::{AccountManager, AccountConfig, DatabaseAccountManager, AccountStore, PgAccountStore};
use tx_engine::TransactionProcessor;
use orm::{BlockRepository, TransactionRepository, AssetRepository, AccountAssetRepository, AccountRepository, PublicKeyRepository, RepositoryResult, BlockModel, TransactionModel, AssetModel, AccountAssetModel};

/// 简单的区块验证器（占位实现）
/// TODO: 集成真实的 BlockchainVerifier（需要重构 processor.rs 的依赖关系）
struct SimpleBlockVerifier;

#[async_trait]
impl p2p::handlers::BlockVerifier for SimpleBlockVerifier {
    async fn verify_and_process(&self, _block: Block) -> Result<()> {
        // TODO: 实现真实的区块验证和处理
        // 当前仅接受所有区块（用于测试）
        Ok(())
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
    let block_verifier: Arc<dyn p2p::handlers::BlockVerifier> = Arc::new(SimpleBlockVerifier);
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
    // 创建真实的数据库仓库
    let block_repo: Arc<dyn BlockRepository> = Arc::new(orm::PgBlockRepository::new(pool.clone()));
    let tx_repo: Arc<dyn TransactionRepository> = Arc::new(orm::PgTransactionRepository::new(pool.clone()));
    let asset_repo: Arc<dyn AssetRepository> = Arc::new(orm::PgAssetRepository::new(pool.clone()));
    let account_asset_repo: Arc<dyn AccountAssetRepository> = Arc::new(orm::PgAccountAssetRepository::new(pool.clone()));
    let account_repo: Arc<dyn AccountRepository> = Arc::new(orm::PgAccountRepository::new(pool.clone()));
    let public_key_repo: Arc<dyn PublicKeyRepository> = Arc::new(orm::PgPublicKeyRepository::new(pool.clone()));
    
    // 创建账户存储
    let account_store: Arc<dyn AccountStore> = Arc::new(PgAccountStore::new(account_repo.clone()));
    
    // 创建账户管理器
    let account_config = AccountConfig::default();
    let account_manager: Arc<dyn AccountManager> = Arc::new(DatabaseAccountManager::new(
        account_store,
        account_repo,
        Arc::clone(&account_asset_repo),
        public_key_repo,
        account_config,
    ));
    
    // 创建交易处理器（使用模拟实现，待后续集成真实实现）
    let tx_processor: Arc<dyn TransactionProcessor> = Arc::new(MockTxProcessor);
    
    let api_state = ApiState {
        account_manager,
        tx_processor,
        block_repo,
        tx_repo,
        asset_repo,
        account_asset_repo,
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