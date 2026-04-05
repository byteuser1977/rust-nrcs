use crate::{peer::Peers, protocol::PeerRequest};
use serde_json;
use std::sync::Arc;
use tracing::{debug, warn};

/// GetNextBlocks 处理器
/// 请求字段：blockId (string), blockIds (Vec<String>), limit (i64)
/// 响应：{"nextBlocks": [区块 JSON 数组]}
pub struct GetNextBlocksHandler {
    peers: Arc<Peers>,
}

impl GetNextBlocksHandler {
    pub fn new(peers: Arc<Peers>) -> Self {
        Self { peers }
    }

    pub async fn handle(&self, request: PeerRequest, _peers: Arc<Peers>) -> serde_json::Value {
        debug!("Handling GetNextBlocks request");

        // 解析请求参数
        let block_ids_opt: Option<Vec<String>> = request.get("blockIds");
        let block_id_opt: Option<String> = request.get("blockId");
        let limit: i64 = request.get("limit").unwrap_or(36); // 默认 36

        // TODO: 实际的区块读取逻辑需要对接区块链数据库
        // 这里先返回占位数据
        warn!("GetNextBlocks not fully implemented - returning placeholder");

        // 模拟响应结构（与 Java 兼容）
        let mut response = serde_json::Map::new();

        if let Some(ids) = block_ids_opt {
            // 如果指定了 blockIds（最多36个），则准确返回这些 ID 的区块
            debug!("Requested {} specific blocks", ids.len());
            // TODO: 从数据库查询这些区块
            let mut blocks = Vec::new();
            for id in ids {
                // TODO: 获取区块 { "block": ... }
                let block_placeholder = serde_json::json!({
                    "block": {
                        "timestamp": 0,
                        "version": 0,
                        "baseTarget": 0,
                        "generationSignature": "",
                        "previousBlockHash": "",
                        "forgedBy": "",
                        "payloadHash": "",
                        "payloadLength": 0,
                        "nonce": 0
                    },
                    "version": 1
                });
                blocks.push(block_placeholder);
            }
            response.insert("nextBlocks".to_string(), serde_json::Value::Array(blocks));
        } else if let Some(ref block_id) = block_id_opt {
            // 从 blockId 开始返回后续 limit 个区块
            debug!("Requesting blocks after {} (limit: {})", block_id, limit);
            // TODO: 从数据库查询后续区块
            let blocks = Vec::new(); // 占位
            response.insert("nextBlocks".to_string(), serde_json::Value::Array(blocks));
        } else {
            warn!("GetNextBlocks missing both blockId and blockIds");
            return serde_json::json!({ "error": "MISSING_BLOCK_ID" });
        }

        serde_json::Value::Object(response)
    }
}