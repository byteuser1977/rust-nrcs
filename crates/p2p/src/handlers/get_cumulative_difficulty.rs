//! GetCumulativeDifficulty Handler
//!
//! 对应 Java: GetCumulativeDifficulty.java
//!
//! 返回本地区块链的累计难度和高度

use crate::protocol::PeerRequest;
use serde_json;
use tracing::debug;
use orm::BlockRepository;
use std::sync::Arc;

pub struct GetCumulativeDifficultyHandler {
    block_repo: Option<Arc<dyn BlockRepository>>,
}

impl GetCumulativeDifficultyHandler {
    pub fn new() -> Self {
        Self { block_repo: None }
    }

    pub fn with_block_repo(block_repo: Arc<dyn BlockRepository>) -> Self {
        Self { block_repo: Some(block_repo) }
    }

    pub async fn handle(&self, _request: PeerRequest) -> serde_json::Value {
        debug!("Handling GetCumulativeDifficulty request");

        if let Some(ref block_repo) = self.block_repo {
            match block_repo.find_latest().await {
                Ok(Some(last_block)) => {
                    let cumulative_difficulty = if last_block.cumulative_difficulty.is_empty() {
                        "0".to_string()
                    } else {
                        let bytes = &last_block.cumulative_difficulty;
                        if let Some(biguint) = num_bigint::BigUint::parse_bytes(bytes, 10) {
                            biguint.to_string()
                        } else {
                            "0".to_string()
                        }
                    };

                    serde_json::json!({
                        "cumulativeDifficulty": cumulative_difficulty,
                        "blockchainHeight": last_block.height
                    })
                }
                Ok(None) => {
                    debug!("No blocks in database");
                    serde_json::json!({
                        "cumulativeDifficulty": "0",
                        "blockchainHeight": 0
                    })
                }
                Err(e) => {
                    debug!("Failed to get last block: {}", e);
                    serde_json::json!({
                        "error": "DATABASE_ERROR"
                    })
                }
            }
        } else {
            debug!("GetCumulativeDifficulty: block repository not configured");
            serde_json::json!({
                "cumulativeDifficulty": "0",
                "blockchainHeight": 0
            })
        }
    }
}

impl Default for GetCumulativeDifficultyHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_creation() {
        let handler = GetCumulativeDifficultyHandler::new();
        assert!(handler.block_repo.is_none());
    }
}
