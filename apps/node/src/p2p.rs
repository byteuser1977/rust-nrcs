//! P2P Network Service (simplified implementation)
//!
//! 负责节点发现、区块和交易广播、同步

use std::sync::Arc;

use libp2p::{
    gossipsub, identity, mdns, noise, swarm::SwarmEvent,
    tcp, yamux, Multiaddr, PeerId, Swarm,
};
use tokio::sync::Mutex;

use blockchain_types::*;
use tx_engine::TransactionProcessor;
use chain::ChainService;

pub struct P2PService {
    local_peer_id: PeerId,
    swarm: Mutex<Swarm<'_, gossipsub::Behaviour>>,
    tx_processor: Arc<dyn TransactionProcessor>,
    chain_service: Arc<ChainService>,
}

impl P2PService {
    pub fn new(
        listen_addr: Multiaddr,
        seed_nodes: Vec<String>,
        tx_processor: Arc<dyn TransactionProcessor>,
        chain_service: Arc<ChainService>,
    ) -> Self {
        // 生成密钥对
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from_public_key(&identity::PublicKey::from(&local_key));

        // 创建 gossipsub 行为
        let message_id_fn = |message: &gossipsub::Message| {
            gossipsub::MessageAuthenticity::Signed(message.serial);
        };
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(std::time::Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .build()
            .unwrap();
        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key),
            gossipsub_config,
        ).unwrap();

        let behaviour = gossipsub;
        let swarm = Swarm::new(tcp::tokio::Transport::new(tcp::Config::default()), behaviour, local_peer_id);
        let swarm = Mutex::new(swarm);

        Self {
            local_peer_id,
            swarm,
            tx_processor,
            chain_service,
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let mut swarm = self.swarm.lock().await;

        // 监听地址
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        info!("P2P service started, peer_id: {}", self.local_peer_id);

        loop {
            match swarm.select_next_some().await {
                SwarmEvent::Behaviour(gossipsub::Event::Message { message, .. }) => {
                    // 处理收到的区块或交易
                    self.handle_gossip_message(&message).await;
                }
                SwarmEvent::Behaviour(gossipsub::Event::Subscribed { .. }) => {}
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("P2P listening on: {}", address);
                }
                _ => {}
            }
        }
    }

    async fn handle_gossip_message(&self, message: &gossipsub::Message) {
        // 根据 topic 区分区块或交易
        let topic = &message.topic;
        let data = &message.data;

        if topic.as_str().contains("blocks") {
            // 处理区块广播
            if let Ok(block) = bincode::deserialize::<Block>(data) {
                self.chain_service.handle_received_block(block).await.ok();
            }
        } else if topic.as_str().contains("transactions") {
            // 处理交易广播
            if let Ok(tx) = bincode::deserialize::<Transaction>(data) {
                // 验证并添加到 mempool
                self.tx_processor.validate(&tx).await.ok();
            }
        }
    }

    pub async fn broadcast_block(&self, block: Block) {
        // 序列化并广播
        let data = bincode::serialize(&block).unwrap();
        // 发布到 gossipsub topic
        // ...
    }

    pub async fn broadcast_transaction(&self, tx: Transaction) {
        let data = bincode::serialize(&tx).unwrap();
        // ...
    }

    pub fn peer_count(&self) -> usize {
        // TODO: 从 swarm 获取连接的 peer 数量
        0
    }
}