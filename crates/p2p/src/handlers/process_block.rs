use crate::{peer::Peers, protocol::PeerRequest};
use serde_json;
use std::sync::Arc;
use tracing::{debug, info, warn};
use super::BlockVerifier;
use blockchain_types::prelude::*;
use anyhow::anyhow; // for error handling

/// ProcessBlock 处理器
/// 请求：完整的区块 JSON（同 Block.getJSONObject() 结构）
/// 处理：验证区块，然后异步提交到区块链处理器
/// 响应：空 JSON {}
pub struct ProcessBlockHandler {
    peers: Arc<Peers>,
    verifier: Arc<dyn BlockVerifier>,
}

impl ProcessBlockHandler {
    pub fn new(peers: Arc<Peers>, verifier: Arc<dyn BlockVerifier>) -> Self {
        Self { peers, verifier }
    }

    pub async fn handle(&self, request: PeerRequest, peers: Arc<Peers>) -> serde_json::Value {
        debug!("Handling ProcessBlock request");

        let block_json = match request.get::<serde_json::Value>("block") {
            Some(block) => block,
            None => {
                warn!("ProcessBlock request missing 'block' field");
                return serde_json::json!({ "error": "MISSING_BLOCK" });
            }
        };

        // Deserialize the block JSON into a Block struct
        let block: Block = match serde_json::from_value(block_json.clone()) {
            Ok(b) => b,
            Err(e) => {
                warn!("Failed to deserialize block: {}", e);
                return serde_json::json!({ "error": "INVALID_BLOCK_JSON", "details": e.to_string() });
            }
        };

        debug!("Received block: height={}, generator={}", block.height, block.generator_id);

        // Verify and process the block using the injected verifier
        match self.verifier.verify_and_process(block).await {
            Ok(_) => {
                info!("Block verified and processed successfully");
                serde_json::json!({})
            }
            Err(e) => {
                warn!("Block verification/processing failed: {}", e);
                serde_json::json!({ "error": "BLOCK_REJECTED", "reason": e.to_string() })
            }
        }
    }
}