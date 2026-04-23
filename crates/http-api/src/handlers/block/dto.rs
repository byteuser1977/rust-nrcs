//! Block API DTOs
//!
//! 区块 API 的数据传输对象

use serde::{Deserialize, Serialize};

/// 获取区块请求
#[derive(Debug, Deserialize)]
pub struct GetBlockRequest {
    pub height: Option<u32>,
    pub block: Option<String>,
}

/// 获取区块列表请求
#[derive(Debug, Deserialize)]
pub struct GetBlocksRequest {
    #[serde(rename = "firstIndex")]
    pub first_index: Option<u32>,
    #[serde(rename = "lastIndex")]
    pub last_index: Option<u32>,
}

/// 获取区块链状态响应
#[derive(Debug, Serialize)]
pub struct GetBlockchainStatusResponse {
    pub application: String,
    pub version: String,
    pub time: u64,
    #[serde(rename = "lastBlock")]
    pub last_block: String,
    #[serde(rename = "lastBlockchainFeeder")]
    pub last_blockchain_feeder: Option<String>,
    #[serde(rename = "lastBlockchainFeederHeight")]
    pub last_blockchain_feeder_height: Option<u32>,
    #[serde(rename = "isScanning")]
    pub is_scanning: bool,
    #[serde(rename = "availablePeers")]
    pub available_peers: bool,
    #[serde(rename = "numberOfBlocks")]
    pub number_of_blocks: u32,
    #[serde(rename = "isTestnet")]
    pub is_testnet: bool,
    #[serde(rename = "requestProcessingTime")]
    pub request_processing_time: u32,
}
