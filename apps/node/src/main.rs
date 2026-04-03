//! NRCS Node - Main entry point
//!
//! 启动完整的区块链节点，包括：
//! - HTTP REST API 服务器
//! - P2P 网络
//! - 共识引擎
//! - 交易处理
//! - 账户管理

use anyhow::Result;
use clap::Parser;
use figment::Figment;
use std::sync::Arc;
use tracing::{info, error,Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use nrcs_node::config::NodeConfig;
use nrcs_node::services::NodeService;
use nrcs_node::p2p::P2PService;
use nrcs_node::chain::ChainService;
use nrcs_node::api::ApiService;

mod config;
mod services;
mod p2p;
mod chain;
mod api;

/// 命令行参数
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 配置目录 (默认: config/)
    #[arg(short, long, default_value = "config")]
    config_dir: String,

    /// 网络环境 (mainnet|testnet)
    #[arg(short, long, default_value = "testnet")]
    network: String,

    /// 数据目录 (默认: data/)
    #[arg(short, long, default_value = "data")]
    data_dir: String,

    /// 是否启用调试日志
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 解析命令行参数
    let cli = Cli::parse();

    // 初始化日志系统
    let filter = if cli.debug {
        EnvFilter::from_default_env().add_directive("nrcs_node=debug".parse()?)
    } else {
        EnvFilter::from_default_env()
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting NRCS Node...");
    info!("Network: {}", cli.network);
    info!("Data dir: {}", cli.data_dir);

    // 加载配置
    let config = NodeConfig::load(&cli)?;
    info!("Configuration loaded");

    // 初始化组件
    let (p2p_service, chain_service, api_service) = NodeService::init(config.clone()).await?;

    // 启动服务
    NodeService::start(p2p_service, chain_service, api_service).await?;

    Ok(())
}