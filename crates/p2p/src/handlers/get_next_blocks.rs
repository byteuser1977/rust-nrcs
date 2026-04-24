use crate::{peer::Peers, protocol::PeerRequest};
use serde_json;
use std::sync::Arc;
use tracing::{debug, warn};
use orm::BlockRepository;

pub struct GetNextBlocksHandler {
    peers: Arc<Peers>,
    block_repo: Option<Arc<dyn BlockRepository>>,
}

impl GetNextBlocksHandler {
    pub fn new(peers: Arc<Peers>) -> Self {
        Self { peers, block_repo: None }
    }

    pub fn with_block_repo(peers: Arc<Peers>, block_repo: Arc<dyn BlockRepository>) -> Self {
        Self { peers, block_repo: Some(block_repo) }
    }

    pub async fn handle(&self, request: PeerRequest, _peers: Arc<Peers>) -> serde_json::Value {
        debug!("Handling GetNextBlocks request");

        let block_ids_opt: Option<Vec<String>> = request.get("blockIds");
        let block_id_opt: Option<String> = request.get("blockId");
        let limit: i64 = request.get("limit").unwrap_or(36);

        let mut response = serde_json::Map::new();

        if let Some(ref block_repo) = self.block_repo {
            if let Some(ids) = block_ids_opt {
                debug!("Requested {} specific blocks", ids.len());
                let mut blocks = Vec::new();
                
                for id_str in ids.iter().take(limit as usize) {
                    if let Ok(id) = id_str.parse::<i64>() {
                        match block_repo.find_by_id_column(id).await {
                            Ok(Some(model)) => {
                                let block_json = serde_json::json!({
                                    "block": {
                                        "version": model.version,
                                        "timestamp": model.timestamp,
                                        "previousBlock": model.previous_block_id,
                                        "totalAmountNQT": model.total_amount,
                                        "totalFeeNQT": model.total_fee,
                                        "payloadLength": model.payload_length,
                                        "payloadHash": hex::encode(&model.payload_hash),
                                        "generatorPublicKey": hex::encode(&model.generation_signature),
                                        "generationSignature": hex::encode(&model.generation_signature),
                                        "blockSignature": hex::encode(&model.block_signature),
                                        "previousBlockHash": model.previous_block_hash.as_ref().map(|h| hex::encode(h)).unwrap_or_default(),
                                        "baseTarget": model.base_target,
                                        "height": model.height,
                                        "nonce": 0,
                                    }
                                });
                                blocks.push(block_json);
                            }
                            Ok(None) => {
                                debug!("Block {} not found", id);
                            }
                            Err(e) => {
                                warn!("Error fetching block {}: {}", id, e);
                            }
                        }
                    }
                }
                response.insert("nextBlocks".to_string(), serde_json::Value::Array(blocks));
            } else if let Some(ref block_id) = block_id_opt {
                debug!("Requesting blocks after {} (limit: {})", block_id, limit);
                
                match block_id.parse::<i64>() {
                    Ok(start_id) => {
                        match block_repo.find_by_id_column(start_id).await {
                            Ok(Some(start_block)) => {
                                let start_height = start_block.height;
                                let end_height = start_height + limit as i32;
                                
                                match block_repo.find_range(start_height + 1, end_height).await {
                                    Ok(models) => {
                                        let blocks: Vec<serde_json::Value> = models.into_iter().map(|model| {
                                            serde_json::json!({
                                                "block": {
                                                    "version": model.version,
                                                    "timestamp": model.timestamp,
                                                    "previousBlock": model.previous_block_id,
                                                    "totalAmountNQT": model.total_amount,
                                                    "totalFeeNQT": model.total_fee,
                                                    "payloadLength": model.payload_length,
                                                    "payloadHash": hex::encode(&model.payload_hash),
                                                    "generatorPublicKey": hex::encode(&model.generation_signature),
                                                    "generationSignature": hex::encode(&model.generation_signature),
                                                    "blockSignature": hex::encode(&model.block_signature),
                                                    "previousBlockHash": model.previous_block_hash.as_ref().map(|h| hex::encode(h)).unwrap_or_default(),
                                                    "baseTarget": model.base_target,
                                                    "height": model.height,
                                                    "nonce": 0,
                                                }
                                            })
                                        }).collect();
                                        response.insert("nextBlocks".to_string(), serde_json::Value::Array(blocks));
                                    }
                                    Err(e) => {
                                        warn!("Error fetching block range: {}", e);
                                        response.insert("nextBlocks".to_string(), serde_json::Value::Array(vec![]));
                                    }
                                }
                            }
                            Ok(None) => {
                                warn!("Start block {} not found", block_id);
                                response.insert("nextBlocks".to_string(), serde_json::Value::Array(vec![]));
                            }
                            Err(e) => {
                                warn!("Error fetching start block: {}", e);
                                response.insert("nextBlocks".to_string(), serde_json::Value::Array(vec![]));
                            }
                        }
                    }
                    Err(_) => {
                        warn!("Invalid blockId format: {}", block_id);
                        return serde_json::json!({ "error": "INVALID_BLOCK_ID" });
                    }
                }
            } else {
                warn!("GetNextBlocks missing both blockId and blockIds");
                return serde_json::json!({ "error": "MISSING_BLOCK_ID" });
            }
        } else {
            warn!("GetNextBlocks: block repository not configured");
            response.insert("nextBlocks".to_string(), serde_json::Value::Array(vec![]));
        }

        serde_json::Value::Object(response)
    }
}
