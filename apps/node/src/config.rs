//! Node configuration

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub api: ApiConfig,
    pub database: DatabaseConfig,
    pub p2p: P2PConfig,
    pub chain: ChainConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub enabled: bool,
    pub listen_addr: SocketAddr,
    pub cors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    pub enabled: bool,
    pub listen_addr: SocketAddr,
    pub seed_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: String,
    pub initial_height: u32,
    pub admin_public_key: Option<String>,
}

impl NodeConfig {
    pub fn load(cli: &crate::Cli) -> Result<Self, anyhow::Error> {
        // 简化的配置加载：从文件或使用默认值
        let config = Self {
            api: ApiConfig {
                enabled: true,
                listen_addr: "127.0.0.1:8080".parse()?,
                cors: true,
            },
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@localhost/nrcs".to_string()),
                max_connections: 10,
            },
            p2p: P2PConfig {
                enabled: true,
                listen_addr: "/ip4/0.0.0.0/tcp/4001".parse()?,
                seed_nodes: vec!["/ip4/127.0.0.1/tcp/4001/p2p/QmSeed".to_string()],
            },
            chain: ChainConfig {
                chain_id: "nrcs-testnet-v1".to_string(),
                initial_height: 0,
                admin_public_key: None,
            },
        };
        Ok(config)
    }
}