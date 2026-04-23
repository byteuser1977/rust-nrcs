//! Network API DTOs
//!
//! 网络 API 的数据传输对象

use serde::{Deserialize, Serialize};

/// 获取节点请求
#[derive(Debug, Deserialize)]
pub struct GetPeerRequest {
    pub peer: String,
}

/// 获取节点列表响应
#[derive(Debug, Serialize)]
pub struct GetPeersResponse {
    pub peers: Vec<PeerInfo>,
    #[serde(rename = "requestProcessingTime")]
    pub request_processing_time: u32,
}

/// 节点信息
#[derive(Debug, Serialize)]
pub struct PeerInfo {
    pub address: String,
    pub port: u16,
    pub state: u8,
    #[serde(rename = "announcedAddress")]
    pub announced_address: String,
    #[serde(rename = "shareAddress")]
    pub share_address: bool,
    #[serde(rename = "downloadedVolume")]
    pub downloaded_volume: u64,
    #[serde(rename = "uploadedVolume")]
    pub uploaded_volume: u64,
    pub application: String,
    pub version: String,
    pub platform: String,
    pub blacklisted: bool,
}
