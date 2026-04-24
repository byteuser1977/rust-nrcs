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

use crate::{peer::Peers, protocol::PeerRequest};
use std::sync::Arc;
use tracing::warn;
use anyhow::Result;
use async_trait::async_trait;
use blockchain_types::prelude::*;
use orm::{BlockRepository, TransactionRepository};
use tx_engine::TransactionProcessor;

#[async_trait]
pub trait BlockVerifier: Send + Sync {
    async fn verify_and_process(&self, block: Block) -> Result<()>;
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
}

impl Handler {
    pub fn new(peers: Arc<Peers>, block_verifier: Arc<dyn BlockVerifier>) -> Self {
        Self {
            get_info: Arc::new(GetInfoHandler::new(Arc::clone(&peers))),
            get_peers: Arc::new(GetPeersHandler::new(Arc::clone(&peers))),
            add_peers: Arc::new(AddPeersHandler::new(Arc::clone(&peers))),
            get_cumulative_difficulty: Arc::new(GetCumulativeDifficultyHandler {}),
            get_milestone_block_ids: Arc::new(GetMilestoneBlockIdsHandler {}),
            get_next_block_ids: Arc::new(GetNextBlockIdsHandler {}),
            get_next_blocks: Arc::new(GetNextBlocksHandler::new(Arc::clone(&peers))),
            get_transactions: Arc::new(GetTransactionsHandler::new()),
            get_unconfirmed_transactions: Arc::new(GetTransactionsHandler::new()),
            process_block: Arc::new(ProcessBlockHandler::new(Arc::clone(&peers), block_verifier)),
            process_transactions: Arc::new(ProcessTransactionsHandler::new(Arc::clone(&peers))),
            bundler_rate: Arc::new(BundlerRateHandler {}),
        }
    }

    pub fn with_repositories(
        peers: Arc<Peers>,
        block_verifier: Arc<dyn BlockVerifier>,
        block_repo: Arc<dyn BlockRepository>,
        tx_repo: Arc<dyn TransactionRepository>,
        tx_processor: Arc<dyn TransactionProcessor>,
    ) -> Self {
        Self {
            get_info: Arc::new(GetInfoHandler::new(Arc::clone(&peers))),
            get_peers: Arc::new(GetPeersHandler::new(Arc::clone(&peers))),
            add_peers: Arc::new(AddPeersHandler::new(Arc::clone(&peers))),
            get_cumulative_difficulty: Arc::new(GetCumulativeDifficultyHandler {}),
            get_milestone_block_ids: Arc::new(GetMilestoneBlockIdsHandler {}),
            get_next_block_ids: Arc::new(GetNextBlockIdsHandler {}),
            get_next_blocks: Arc::new(GetNextBlocksHandler::with_block_repo(Arc::clone(&peers), block_repo)),
            get_transactions: Arc::new(GetTransactionsHandler::with_tx_repo(tx_repo)),
            get_unconfirmed_transactions: Arc::new(GetTransactionsHandler::new()),
            process_block: Arc::new(ProcessBlockHandler::new(Arc::clone(&peers), block_verifier)),
            process_transactions: Arc::new(ProcessTransactionsHandler::with_tx_processor(Arc::clone(&peers), tx_processor)),
            bundler_rate: Arc::new(BundlerRateHandler {}),
        }
    }

    pub async fn handle(&self, request: PeerRequest, peers: Arc<Peers>) -> serde_json::Value {
        match request.request_type {
            crate::protocol::RequestType::GetInfo => {
                self.get_info.handle(request, Arc::clone(&peers)).await
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
