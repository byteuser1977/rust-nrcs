mod get_info;
mod get_peers;
mod add_peers;
mod get_next_blocks;
mod process_block;
mod process_transactions;
mod get_transactions;
mod get_cumulative_difficulty;
mod get_milestone_block_ids;
mod get_next_block_ids;
mod bundler_rate;
mod unknown;

pub use get_info::GetInfoHandler;
pub use get_peers::GetPeersHandler;
pub use add_peers::AddPeersHandler;
pub use get_next_blocks::GetNextBlocksHandler;
pub use process_block::ProcessBlockHandler;
pub use process_transactions::ProcessTransactionsHandler;
pub use get_transactions::GetTransactionsHandler;
pub use get_cumulative_difficulty::GetCumulativeDifficultyHandler;
pub use get_milestone_block_ids::GetMilestoneBlockIdsHandler;
pub use get_next_block_ids::GetNextBlockIdsHandler;
pub use bundler_rate::BundlerRateHandler;
pub use unknown::UnknownHandler;

use crate::{peer::Peers, protocol::PeerRequest, config::P2PConfig, peer::{PeerState, current_timestamp}};
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::sync::RwLock;
use tracing::{debug, warn};
use anyhow::Result;
use async_trait::async_trait;
use blockchain_types::prelude::*;
use orm::{BlockRepository, TransactionRepository};
use tx_engine::TransactionProcessor;

#[async_trait]
pub trait BlockVerifier: Send + Sync {
    async fn verify_and_process(&self, block: Block) -> Result<()>;
    async fn has_block(&self, block_id: u64) -> Result<bool>;
    
    async fn get_last_block_id(&self) -> Result<Option<u64>>;
    
    async fn get_last_block_cumulative_difficulty(&self) -> Result<Vec<u8>>;
    
    async fn can_connect_block(&self, previous_block_id: u64) -> Result<bool>;
    
    async fn process_fork_block(&self, block: Block) -> Result<()>;
    
    async fn get_block_height(&self, block_id: u64) -> Result<Option<u32>>;
    
    async fn get_height(&self) -> Result<u32>;

    async fn get_cumulative_difficulty(&self) -> Result<String>;

    async fn pop_off_to(&self, height: u32) -> Result<Vec<Block>>;
}

pub struct Handler {
    pub get_info: Arc<GetInfoHandler>,
    pub get_peers: Arc<GetPeersHandler>,
    pub add_peers: Arc<AddPeersHandler>,
    pub get_cumulative_difficulty: Arc<GetCumulativeDifficultyHandler>,
    pub get_milestone_block_ids: Arc<GetMilestoneBlockIdsHandler>,
    pub get_next_block_ids: Arc<GetNextBlockIdsHandler>,
    pub get_next_blocks: Arc<GetNextBlocksHandler>,
    pub get_transactions: Arc<GetTransactionsHandler>,
    pub get_unconfirmed_transactions: Arc<GetTransactionsHandler>,
    pub process_block: Arc<ProcessBlockHandler>,
    pub process_transactions: Arc<ProcessTransactionsHandler>,
    pub bundler_rate: Arc<BundlerRateHandler>,
    p2p_config: Arc<P2PConfig>,
    is_downloading: Arc<RwLock<bool>>,
}

impl Handler {
    pub fn new(peers: Arc<Peers>, block_verifier: Arc<dyn BlockVerifier>, p2p_config: Arc<P2PConfig>) -> Self {
        Self {
            get_info: Arc::new(GetInfoHandler::new(Arc::clone(&peers))),
            get_peers: Arc::new(GetPeersHandler::new(Arc::clone(&peers))),
            add_peers: Arc::new(AddPeersHandler::new(Arc::clone(&peers))),
            get_cumulative_difficulty: Arc::new(GetCumulativeDifficultyHandler::new()),
            get_milestone_block_ids: Arc::new(GetMilestoneBlockIdsHandler::new()),
            get_next_block_ids: Arc::new(GetNextBlockIdsHandler::new()),
            get_next_blocks: Arc::new(GetNextBlocksHandler::new(Arc::clone(&peers))),
            get_transactions: Arc::new(GetTransactionsHandler::new()),
            get_unconfirmed_transactions: Arc::new(GetTransactionsHandler::new()),
            process_block: Arc::new(ProcessBlockHandler::new(Arc::clone(&peers), block_verifier, Arc::clone(&p2p_config))),
            process_transactions: Arc::new(ProcessTransactionsHandler::new(Arc::clone(&peers))),
            bundler_rate: Arc::new(BundlerRateHandler::new()),
            p2p_config,
            is_downloading: Arc::new(RwLock::new(false)),
        }
    }

    pub fn with_repositories(
        peers: Arc<Peers>,
        block_verifier: Arc<dyn BlockVerifier>,
        block_repo: Arc<dyn BlockRepository>,
        tx_repo: Arc<dyn TransactionRepository>,
        tx_processor: Arc<dyn TransactionProcessor>,
        p2p_config: Arc<P2PConfig>,
    ) -> Self {
        Self {
            get_info: Arc::new(GetInfoHandler::new(Arc::clone(&peers))),
            get_peers: Arc::new(GetPeersHandler::new(Arc::clone(&peers))),
            add_peers: Arc::new(AddPeersHandler::new(Arc::clone(&peers))),
            get_cumulative_difficulty: Arc::new(GetCumulativeDifficultyHandler::with_block_repo(Arc::clone(&block_repo))),
            get_milestone_block_ids: Arc::new(GetMilestoneBlockIdsHandler::with_block_repo(Arc::clone(&block_repo))),
            get_next_block_ids: Arc::new(GetNextBlockIdsHandler::with_block_repo(Arc::clone(&block_repo))),
            get_next_blocks: Arc::new(GetNextBlocksHandler::with_block_repo(Arc::clone(&peers), block_repo)),
            get_transactions: Arc::new(GetTransactionsHandler::with_tx_repo(tx_repo)),
            get_unconfirmed_transactions: Arc::new(GetTransactionsHandler::new()),
            process_block: Arc::new(ProcessBlockHandler::new(Arc::clone(&peers), block_verifier, Arc::clone(&p2p_config))),
            process_transactions: Arc::new(ProcessTransactionsHandler::with_tx_processor(Arc::clone(&peers), tx_processor)),
            bundler_rate: Arc::new(BundlerRateHandler::new()),
            p2p_config,
            is_downloading: Arc::new(RwLock::new(false)),
        }
    }

    /// Set downloading state (called by BlockchainSyncDaemon when sync starts/stops)
    pub fn set_downloading(&self, downloading: bool) {
        // We can't block on async write here, so spawn
        let flag = Arc::clone(&self.is_downloading);
        tokio::spawn(async move {
            *flag.write().await = downloading;
        });
    }

    /// Check if currently downloading
    pub async fn is_downloading(&self) -> bool {
        *self.is_downloading.read().await
    }

    /// Whether a request type should be rejected while blockchain is downloading
    ///
    /// 对应 Java: PeerRequestHandler.rejectWhileDownloading()
    fn reject_while_downloading(&self, request_type: &crate::protocol::RequestType) -> bool {
        use crate::protocol::RequestType;
        !matches!(request_type,
            RequestType::GetInfo
            | RequestType::GetPeers
            | RequestType::AddPeers
            | RequestType::BundlerRate
        )
    }

    /// Full request processing with security checks (P2P dispatch entry point)
    ///
    /// 对应 Java: PeerServlet.process(IPeer peer, Reader reader)
    ///
    /// Performs all security validations before delegating to the specific handler:
    /// 1. Blacklist check — rejects blacklisted peers
    /// 2. Protocol version validation — rejects protocol > 2
    /// 3. getInfo sequence enforcement — first request MUST be getInfo
    /// 4. Inbound connection limit — rejects if too many inbound connections
    /// 5. Downloading check — rejects handlers that require up-to-date blockchain
    pub async fn process_request(
        &self,
        request: PeerRequest,
        peers: Arc<Peers>,
        peer_addr: SocketAddr,
    ) -> serde_json::Value {
        // 1. Blacklist check
        // 对应 Java: if (peer.isBlacklisted()) { return error(BLACKLISTED); }
        if peers.is_blacklisted_addr(&peer_addr).await {
            debug!("[P2P Dispatch] Rejected blacklisted peer: {}", peer_addr);
            return serde_json::json!({"error": "BLACKLISTED"});
        }

        // 2. Register or get the peer
        // 对应 Java: Peers.addPeer(peer)
        let peer_ref = peers.find_or_create_peer(peer_addr, true).await;
        let mut peer = peer_ref.lock().await;

        // 3. Protocol version validation
        // 对应 Java: if (request.getInt("protocol") > 2) { return error(UNSUPPORTED_PROTOCOL); }
        if request.protocol > 2 {
            debug!("[P2P Dispatch] Unsupported protocol {} from {}", request.protocol, peer_addr);
            return serde_json::json!({"error": "UNSUPPORTED_PROTOCOL"});
        }

        // 4. getInfo sequence enforcement
        // 对应 Java: if (peer.getVersion() == null && !"getInfo".equals(requestType))
        let is_get_info = matches!(request.request_type, crate::protocol::RequestType::GetInfo);
        if !is_get_info && peer.version.is_none() {
            debug!("[P2P Dispatch] Sequence error: peer {} sent {:?} before getInfo",
                   peer_addr, request.request_type);
            return serde_json::json!({"error": "SEQUENCE_ERROR"});
        }

        // 5. Inbound connection limit
        // 对应 Java: if (hasTooManyInboundPeers()) { return error(MAX_INBOUND_CONNECTIONS); }
        let inbound_count = peers.inbound_connection_count().await;
        if inbound_count >= self.p2p_config.max_inbound_connections {
            debug!("[P2P Dispatch] Max inbound connections reached ({}), rejecting {}",
                   inbound_count, peer_addr);
            return serde_json::json!({"error": "MAX_INBOUND_CONNECTIONS"});
        }

        // 6. Downloading check
        // 对应 Java: if (handler.rejectWhileDownloading() && isDownloading) { return error(DOWNLOADING); }
        if *self.is_downloading.read().await && self.reject_while_downloading(&request.request_type) {
            debug!("[P2P Dispatch] Rejecting {:?} from {} while downloading",
                   request.request_type, peer_addr);
            return serde_json::json!({"error": "DOWNLOADING"});
        }

        // Update peer activity tracking
        // 对应 Java: peer.setState(PeerState.CONNECTED) and timestamp updates
        let now = current_timestamp();
        let was_inbound = peer.is_inbound;
        peer.last_inbound_request = now;
        peer.last_updated = now;
        if peer.state == PeerState::NonConnected {
            peer.state = PeerState::Connected;
        }
        if !was_inbound {
            peer.is_inbound = true;
            peer.fire_event(crate::peer::PeerEvent::AddInbound);
        }
        drop(peer);

        // 7. Delegate to the specific handler
        // 对应 Java: peerRequestHandler.processRequest(request, peer)
        self.handle(request, peers, peer_addr).await
    }

    pub async fn handle(&self, request: PeerRequest, peers: Arc<Peers>, peer_addr: SocketAddr) -> serde_json::Value {
        match request.request_type {
            crate::protocol::RequestType::GetInfo => {
                self.get_info.handle(request, Arc::clone(&peers), peer_addr).await
            }
            crate::protocol::RequestType::GetPeers => {
                self.get_peers.handle(request, Arc::clone(&peers)).await
            }
            crate::protocol::RequestType::AddPeers => {
                self.add_peers.handle(request, Arc::clone(&peers)).await
            }
            crate::protocol::RequestType::GetCumulativeDifficulty => {
                self.get_cumulative_difficulty.handle(request).await
            }
            crate::protocol::RequestType::GetMilestoneBlockIds => {
                self.get_milestone_block_ids.handle(request).await
            }
            crate::protocol::RequestType::GetNextBlockIds => {
                self.get_next_block_ids.handle(request).await
            }
            crate::protocol::RequestType::GetNextBlocks => {
                self.get_next_blocks.handle(request, Arc::clone(&peers)).await
            }
            crate::protocol::RequestType::GetTransactions => {
                self.get_transactions.handle(request).await
            }
            crate::protocol::RequestType::GetUnconfirmedTransactions => {
                self.get_unconfirmed_transactions.handle(request).await
            }
            crate::protocol::RequestType::ProcessBlock => {
                self.process_block.handle(request, Arc::clone(&peers)).await
            }
            crate::protocol::RequestType::ProcessTransactions => {
                self.process_transactions.handle(request, Arc::clone(&peers)).await
            }
            crate::protocol::RequestType::BundlerRate => {
                self.bundler_rate.handle(request).await
            }
            crate::protocol::RequestType::Unknown(ref s) => {
                warn!("Unknown request type: {}", s);
                serde_json::json!({ "error": "UNSUPPORTED_REQUEST_TYPE" })
            }
        }
    }
}
