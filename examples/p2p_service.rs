//! P2P Service Example
//!
//! 展示如何使用 P2P 模块

use p2p::{P2PConfig, P2PManager, BlacklistManager};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("=== NRCS P2P Service Example ===\n");

    // 1. 创建配置
    let mut config = P2PConfig::default();
    config.listen_addr = "0.0.0.0:16974".parse()?;
    config.my_address = Some("my-node.example.com:16974".to_string());
    config.share_my_address = true;
    config.use_websockets = true;
    config.max_connections = 20;
    config.max_inbound_connections = 100;
    config.max_outbound_connections = 20;

    println!("Configuration:");
    println!("  Listen: {}", config.listen_addr);
    println!("  Max connections: {}", config.max_connections);
    println!("  WebSocket: {}", config.use_websockets);
    println!();

    // 2. 创建 P2P 管理器
    let mut manager = P2PManager::new(config.clone());
    
    // 3. 初始化
    manager.init().await?;
    println!("P2P manager initialized\n");

    // 4. 创建黑名单管理器
    let blacklist = BlacklistManager::new();
    
    // 添加一些已知的黑名单节点
    let bad_addr: SocketAddr = "192.168.1.100:16974".parse()?;
    blacklist.add_to_blacklist(
        bad_addr,
        "Malicious behavior".to_string(),
        Some(3600), // 1 hour
        false,
    ).await;
    
    println!("Blacklist count: {}", blacklist.blacklist_count().await);
    println!("Is {} blacklisted? {}", bad_addr, blacklist.is_blacklisted(&bad_addr).await);
    println!();

    // 5. 启动 P2P 服务
    println!("Starting P2P service...");
    manager.start().await?;
    println!("P2P service started\n");

    // 6. 添加一些种子节点
    let seed_peers = vec![
        "seed1.nrcs.network:16974",
        "seed2.nrcs.network:16974",
        "seed3.nrcs.network:16974",
    ];

    for peer_addr in seed_peers {
        if let Ok(addr) = peer_addr.parse::<SocketAddr>() {
            let peer = p2p::peer::Peer::new(addr, false);
            manager.add_peer(peer).await;
            println!("Added seed peer: {}", peer_addr);
        }
    }
    println!();

    // 7. 运行一段时间
    println!("P2P service is running...");
    println!("Known peers: {}", manager.known_peers_count().await);
    println!("Connected peers: {}", manager.connected_peers_count().await);
    println!();

    // 8. 等待信号
    println!("Press Ctrl+C to stop...");
    tokio::signal::ctrl_c().await?;

    // 9. 停止服务
    println!("\nStopping P2P service...");
    manager.stop().await;
    println!("P2P service stopped");

    Ok(())
}
