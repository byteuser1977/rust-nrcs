//! P2P 服务核心实现
//!
//! 基于 libp2p 构建的区块链网络服务,提供:
//! - 节点发现与连接管理
//! - 区块/交易广播
//! - 同步协议

use super::*;
use blockchain_types::*;
use libp2p::{
    identity, mdns, noise, swarm::SwarmEvent,
    tcp, yamux, Multiaddr, PeerId, Swarm,
    gossipsub, kad::{self, store::MemoryStore},
    identify, ping,
};
use std::convert::TryFrom;
use std::error::Error;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

/// P2P 服务错误类型
#[derive(Debug, thiserror::Error)]
pub enum P2PError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("libp2p error: {0}")]
    Libp2p(#[from] libp2p::swarm::SwarmError),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("channel closed")]
    ChannelClosed,

    #[error("timeout")]
    Timeout,
}

pub type P2PResult<T> = Result<T, P2PError>;

/// P2P 服务主结构
pub struct P2PService {
    /// 本地节点 ID
    local_peer_id: PeerId,
    /// libp2p Swarm
    swarm: Swarm<Behaviour>,
    /// 节点状态
    state: RwLock<P2PState>,
    /// 区块广播发送端
    block_tx: mpsc::Sender<Block>,
    /// 区块广播接收端
    block_rx: RwLock<Option<mpsc::Receiver<Block>>>,
    /// 交易广播发送端
    tx_tx: mpsc::Sender<Transaction>,
    /// 交易广播接收端
    tx_rx: RwLock<Option<mpsc::Receiver<Transaction>>>,
}

/// P2P 状态
struct P2PState {
    /// 已连接的节点列表
    peers: Vec<PeerInfo>,
    /// 已知区块高度（本地最高）
    current_height: Height,
    /// 内存池中的交易
    tx_pool: Vec<Transaction>,
}

impl P2PService {
    /// 创建并启动 P2P 服务
    pub async fn new(
        bind_addr: Multiaddr,
        bootstrap_nodes: Vec<Multiaddr>,
    ) -> P2PResult<(Self, mpsc::Receiver<Block>, mpsc::Receiver<Transaction>)> {
        // 1. 生成本机密钥对
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        info!(peer_id = %local_peer_id, "Generating new peer ID");

        // 2. 构建传输层 (TCP + Yamux + Noise)
        let transport = tcp::TokioTcpConfig::new()
            .nodelay(true)
            .upgrade(libp2p::core::upgrade::Version::V1Lazy)
            .authenticate(noise::Config::new(&local_key))
            .multiplex(yamux::Config::default())
            .boxed();

        // 3. 创建 gossip 协议 (用于交易/区块传播)
        let message_id_fn = |message: &P2PMessage| -> libp2p:: gossipsub::MessageId {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            message.hash(&mut hasher);
            libp2p::gossipsub::MessageId::from(hasher.finish().to_be_bytes())
        };

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::AllowEphemeral)
            .message_id_fn(message_id_fn)
            .build()
            .map_err(|e| P2PError::Serialization(e.to_string()))?;

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key),
            gossipsub_config,
        )?;

        // 4. Kademlia DHT (节点发现)
        let kademlia_store = MemoryStore::new(local_peer_id);
        let kademlia = kad::Behaviour::new(local_peer_id, kademlia_store);

        // 5. 节点识别 (Identify)
        let identify = identify::Behaviour::new(identify::Config::new(
            "/nrcs/0.1.0".to_string(),
            local_key.public(),
        ));

        // 6. Ping
        let ping = ping::Behaviour::new(ping::Config::new());

        // 7. 自定义协议行为
        let custom_behaviour = Behaviour {
            gossipsub,
            kademlia,
            identify,
            ping,
        };

        // 8. 创建 Swarm
        let mut swarm = Swarm::new(transport, custom_behaviour, local_peer_id);
        Swarm::listen_on(&mut swarm, bind_addr)?;

        info!("P2P service listening on {}", bind_addr);

        // 9. 连接引导节点
        for addr in bootstrap_nodes {
            if let Ok(peer_id) = resolve_peer_id_from_address(&addr) {
                Swarm::dial_address(&mut swarm, addr)?;
                debug!(peer = %peer_id, "Dialing bootstrap node");
            }
        }

        // 10. 通道
        let (block_tx, block_rx) = mpsc::channel(100);
        let (tx_tx, tx_rx) = mpsc::channel(1000);

        let state = RwLock::new(P2PState {
            peers: vec![],
            current_height: 0,
            tx_pool: vec![],
        });

        let service = Self {
            local_peer_id,
            swarm,
            state,
            block_tx,
            block_rx: RwLock::new(Some(block_rx)),
            tx_tx,
            tx_rx: RwLock::new(Some(tx_rx)),
        };

        Ok((service, block_rx, tx_rx))
    }

    /// 启动 P2P 事件循环
    pub async fn start(mut self) -> P2PResult<()> {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
                    self.handle_event(event).await?;
                }
                block = self.block_rx.read().await.as_mut().unwrap().recv() => {
                    match block {
                        Some(block) => self.broadcast_block(block).await,
                        None => break,
                    }
                }
                tx = self.tx_rx.read().await.as_mut().unwrap().recv() => {
                    match tx {
                        Some(tx) => self.broadcast_transaction(tx).await,
                        None => break,
                    }
                }
            }
        }
        Ok(())
    }

    /// 处理 P2P 事件
    async fn handle_event(&mut self, event: SwarmEvent<Behaviour>) -> P2PResult<()> {
        match event {
            SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event {
                peer_id,
                peer_info,
            })) => {
                debug!(peer = %peer_id, "Identified peer");
                // 添加到地址簿
                for addr in &peer_info.addresses {
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(gossipsub::Event::Message {
                message_id: _,
                source,
                message,
            })) => {
                match message {
                    P2PMessage::TransactionBroadcast(txb) => {
                        debug!(from = %source, tx_count = txb.transactions.len(), "Received tx broadcast");
                        // 处理交易广播 (转发到本地 tx_pool)
                        for tx in txb.transactions {
                            self.handle_incoming_transaction(tx).await;
                        }
                    }
                    P2PMessage::BlockAnnounce(announce) => {
                        debug!(from = %source, height = announce.height, "Received block announce");
                        // 请求完整区块
                        self.request_block(announce.block_id, announce.height).await;
                    }
                    _ => {}
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Ping(ping::Event::PingResponse {
                peer,
                rtt,
            })) => {
                debug!(peer = %peer, rtt = ?rtt, "Ping response");
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                info!(address = %address, "New listen address");
            }
            SwarmEvent::ConnectionClosed { peer_id, ... } => {
                warn!(peer = %peer_id, "Connection closed");
            }
            _ => {}
        }
        Ok(())
    }

    /// 广播区块
    async fn broadcast_block(&mut self, block: Block) {
        // 1. 创建区块公告 (仅头部)
        let announce = BlockAnnounce {
            block_id: block.id,
            height: block.height,
            hash: block.compute_hash().unwrap_or_default(),
            generator_id: block.generator_id,
            timestamp: block.timestamp,
            tx_count: block.transactions.len() as u32,
            total_amount: block.total_amount,
            total_fee: block.total_fee,
        };

        let msg = P2PMessage::BlockAnnounce(announce);
        if let Ok(data) = bincode::serialize(&msg) {
            let _ = self.publish_message(data).await;
        }
    }

    /// 广播交易
    async fn broadcast_transaction(&mut self, tx: Transaction) {
        let msg = P2PMessage::transaction_broadcast(vec![tx.clone()]);
        if let Ok(data) = bincode::serialize(&msg) {
            let _ = self.publish_message(data).await;
        }
    }

    /// 发布消息到 GossipSub
    async fn publish_message(&mut self, data: Vec<u8>) -> P2PResult<()> {
        // 简化：实际需要选择 topic
        // self.swarm.behaviour_mut().gossipsub.publish(topic_id, data)?;
        Ok(())
    }

    /// 处理接收到的交易
    async fn handle_incoming_transaction(&self, tx: Transaction) {
        // TODO: 验证交易签名、余额等
        // 如果有效，加入内存池并继续广播
        debug!(tx_id = ?tx.full_hash, "Incoming transaction");
    }

    /// 请求完整区块
    async fn request_block(&mut self, block_id: BlockId, height: Height) {
        // TODO: 向特定 peer 发送 BlockRequest
    }

    /// 获取本地节点 ID
    pub fn local_peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }

    /// 获取连接节点数
    pub fn connected_peers(&self) -> usize {
        self.state.read().await.peers.len()
    }
}

/// 解析地址中的 PeerId (简化版)
fn resolve_peer_id_from_address(_addr: &Multiaddr) -> P2PResult<PeerId> {
    // 实际实现需要 libp2p 的 multitool 或 DHT 查询
    // 这里简化：返回错误，引导节点需要先已知 peer ID
    Err(P2PError::Serialization("peer id resolution not implemented".to_string()))
}

/// 自定义行为组合
#[derive(Debug)]
pub struct Behaviour {
    /// GossipSub 用于交易/区块传播
    gossipsub: gossipsub::Behaviour,
    /// Kademlia DHT 用于节点发现
    kademlia: kad::Behaviour<MemoryStore>,
    /// 节点识别
    identify: identify::Behaviour,
    /// Ping
    ping: ping::Behaviour,
}

impl libp2p::swarm::NetworkBehaviour for Behaviour {
    type Connection = libp2p::swarm::Connection;

    type PollEvent = BehaviourEvent;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context,
        _params: &libp2p::swarm::poll::PollParameters,
    ) -> std::task::Poll<libp2p::swarm::NetworkBehaviourAction<
        Self::Connection,
        BehaviourEvent,
    >> {
        // 简化：轮询各个子行为
        // 实际实现需要手动实现 poll 方法
        std::task::Poll::Pending
    }
}

/// 行为事件枚举
#[derive(Debug)]
pub enum BehaviourEvent {
    Gossipsub(gossipsub::Event),
    Kademlia(kad::Event),
    Identify(identify::Event),
    Ping(ping::Event),
}

impl From<gossipsub::Event> for BehaviourEvent {
    fn from(evt: gossipsub::Event) -> Self {
        Self::Gossipsub(evt)
    }
}

impl From<kad::Event> for BehaviourEvent {
    fn from(evt: kad::Event) -> Self {
        Self::Kademlia(evt)
    }
}

impl From<identify::Event> for BehaviourEvent {
    fn from(evt: identify::Event) -> Self {
        Self::Identify(evt)
    }
}

impl From<ping::Event> for BehaviourEvent {
    fn from(evt: ping::Event) -> Self {
        Self::Ping(evt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p2p_service_creation() {
        // TODO: 实现测试
    }
}
