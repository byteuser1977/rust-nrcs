//! Blockchain Sync Daemon (区块链同步守护进程)
//!
//! 对应 NRCS Java: BlockchainProcessor 内部类 DownloadThread
//!
//! 职责:
//! - 检测本地区块链高度与 peer 累计难度
//! - 找到与 peer 的共同里程碑区块
//! - 下载缺失的区块并验证

use crate::config::P2PConfig;
use crate::peer::{Peer, PeerState};
use crate::protocol::{PeerRequest, RequestType};
use crate::websocket::WebsocketClient;
use crate::handlers::BlockVerifier;
use blockchain_types::prelude::Block;
use blockchain_types::prelude::Transaction;
use blockchain_types::constants::GENESIS_BLOCK_ID;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use std::cmp::min;

const SEGMENT_SIZE: usize = 36;
const MAX_BLOCKS_BATCH: usize = 720;

pub struct BlockchainSyncDaemon {
    config: P2PConfig,
    running: Arc<RwLock<bool>>,
    is_downloading: Arc<RwLock<bool>>,
}

impl BlockchainSyncDaemon {
    pub fn new(config: P2PConfig) -> Self {
        Self {
            config,
            running: Arc::new(RwLock::new(false)),
            is_downloading: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(
        &self,
        peers: Arc<crate::peer::Peers>,
        block_verifier: Arc<dyn BlockVerifier>,
    ) {
        let mut running = self.running.write().await;
        if *running {
            warn!("Blockchain sync daemon is already running");
            return;
        }
        *running = true;
        drop(running);

        let config = self.config.clone();
        let running = Arc::clone(&self.running);
        let is_downloading = Arc::clone(&self.is_downloading);

        tokio::spawn(async move {
            info!("Blockchain sync daemon started");
            Self::sync_loop(peers, config, running, is_downloading, block_verifier).await;
            info!("Blockchain sync daemon stopped");
        });
    }

    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Blockchain sync daemon stopping...");
    }

    pub async fn is_downloading(&self) -> bool {
        *self.is_downloading.read().await
    }

    async fn sync_loop(
        peers: Arc<crate::peer::Peers>,
        config: P2PConfig,
        running: Arc<RwLock<bool>>,
        is_downloading: Arc<RwLock<bool>>,
        block_verifier: Arc<dyn BlockVerifier>,
    ) {
        loop {
            if !*running.read().await {
                break;
            }

            tokio::time::sleep(Duration::from_secs(30)).await;

            if let Err(e) = Self::download_peer(&peers, &is_downloading, &block_verifier).await {
                if *running.read().await {
                    debug!("Sync loop error: {}", e);
                }
            }
        }
    }

    async fn download_peer(
        peers: &Arc<crate::peer::Peers>,
        is_downloading: &Arc<RwLock<bool>>,
        block_verifier: &Arc<dyn BlockVerifier>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connected_peers = peers.get_known_peers().await;
        let connected_peers: Vec<Peer> = connected_peers.into_iter()
            .filter(|p| p.state == PeerState::Connected)
            .collect();

        if connected_peers.is_empty() {
            debug!("No connected peers for sync");
            return Ok(());
        }

        let peer = match connected_peers.first() {
            Some(p) => p.clone(),
            None => {
                debug!("No suitable peer found for sync");
                return Ok(());
            }
        };

        let peer_addr = peer.address;
        let unknown_label = "unknown".to_string();
        let peer_label = peer.announced_address.as_ref().unwrap_or(&unknown_label);
        info!("Selected peer for sync: {}", peer_label);

        let cumulative_difficulty = Self::get_cumulative_difficulty(peer_addr).await?;
        if cumulative_difficulty.is_none() {
            debug!("Failed to get cumulative difficulty from peer");
            return Ok(());
        }

        let peer_cumulative_difficulty = cumulative_difficulty.unwrap();
        info!("Peer cumulative difficulty: {}", peer_cumulative_difficulty);

        let common_block_id = if block_verifier.has_block(GENESIS_BLOCK_ID).await.unwrap_or(false) {
            Self::get_common_milestone_block_id(peer_addr, block_verifier).await?
        } else {
            info!("No local blocks, starting from genesis block");
            GENESIS_BLOCK_ID
        };
        
        if common_block_id == 0 {
            debug!("Could not find common milestone block, using genesis block");
        }

        info!("Common milestone block id: {}", common_block_id);

        let common_block_height = block_verifier.get_block_height(common_block_id).await
            .ok()
            .flatten()
            .unwrap_or(0);
        info!("Common block height: {}", common_block_height);

        let chain_block_ids = Self::get_block_ids_after_common(peer_addr, common_block_id, block_verifier).await?;
        if chain_block_ids.len() < 2 {
            debug!("Not enough blocks after common block");
            return Ok(());
        }

        info!("Blocks to download: {}", chain_block_ids.len() - 1);

        if !*is_downloading.read().await && chain_block_ids.len() > 10 {
            info!("Blockchain download in progress");
            *is_downloading.write().await = true;
        }

        Self::download_blocks(peer_addr, &chain_block_ids, common_block_height, block_verifier).await?;

        *is_downloading.write().await = false;

        Ok(())
    }

    async fn get_cumulative_difficulty(
        peer_addr: std::net::SocketAddr,
    ) -> Result<Option<u128>, Box<dyn std::error::Error + Send + Sync>> {
        let request = PeerRequest::new(RequestType::GetCumulativeDifficulty, 1);

        match WebsocketClient::send_request(peer_addr, request).await {
            Ok(response) => {
                if let Some(cumulative_diff) = response.get("cumulativeDifficulty").and_then(|v| v.as_str()) {
                    match cumulative_diff.parse::<u128>() {
                        Ok(diff) => Ok(Some(diff)),
                        Err(_) => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                debug!("Failed to get cumulative difficulty: {}", e);
                Ok(None)
            }
        }
    }

    async fn get_common_milestone_block_id(
        peer_addr: std::net::SocketAddr,
        block_verifier: &Arc<dyn BlockVerifier>,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let mut last_milestone_block_id: Option<String> = None;

        loop {
            let mut request = PeerRequest::new(RequestType::GetMilestoneBlockIds, 1);

            if let Some(ref last_id) = last_milestone_block_id {
                request.set("lastMilestoneBlockId", last_id);
            } else {
                request.set("lastBlockId", &"0");
            }

            match WebsocketClient::send_request(peer_addr, request).await {
                Ok(response) => {
                    if let Some(milestone_ids) = response.get("milestoneBlockIds").and_then(|v| v.as_array()) {
                        if milestone_ids.is_empty() {
                            return Ok(GENESIS_BLOCK_ID);
                        }

                        if milestone_ids.len() > 20 {
                            warn!("Peer {} sends too many milestoneBlockIds", peer_addr);
                            return Ok(0);
                        }

                        for milestone_id in milestone_ids {
                            if let Some(id_str) = milestone_id.as_str() {
                                if let Ok(block_id) = Self::parse_block_id(id_str) {
                                    info!("Checking milestone block ID: {}", block_id);
                                    
                                    if Self::has_block(block_id, block_verifier).await? {
                                        info!("Found common milestone block: {}", block_id);
                                        return Ok(block_id);
                                    }
                                    
                                    last_milestone_block_id = Some(id_str.to_string());
                                }
                            }
                        }

                        if last_milestone_block_id.is_none() {
                            return Ok(GENESIS_BLOCK_ID);
                        }
                    } else {
                        return Ok(GENESIS_BLOCK_ID);
                    }
                }
                Err(e) => {
                    debug!("Failed to get milestone block ids: {}", e);
                    return Ok(GENESIS_BLOCK_ID);
                }
            }
        }
    }

    async fn get_block_ids_after_common(
        peer_addr: std::net::SocketAddr,
        start_block_id: u64,
        block_verifier: &Arc<dyn BlockVerifier>,
    ) -> Result<Vec<u64>, Box<dyn std::error::Error + Send + Sync>> {
        let mut block_list = Vec::new();
        let mut match_id = start_block_id;
        let limit = 1440;

        let mut request = PeerRequest::new(RequestType::GetNextBlockIds, 1);
        request.set("blockId", &start_block_id.to_string());
        request.set("limit", &(limit as i32));

        info!("Requesting block IDs after {} from peer", start_block_id);

        match WebsocketClient::send_request(peer_addr, request).await {
            Ok(response) => {
                info!("Received response: {:?}", response);
                
                if let Some(next_block_ids) = response.get("nextBlockIds").and_then(|v| v.as_array()) {
                    info!("Received {} block IDs from peer", next_block_ids.len());
                    
                    if next_block_ids.is_empty() {
                        block_list.push(match_id);
                        return Ok(block_list);
                    }

                    if next_block_ids.len() > limit {
                        warn!("Peer {} sends too many nextBlockIds", peer_addr);
                        return Ok(Vec::new());
                    }

                    let mut matching = true;
                    for (index, next_block_id) in next_block_ids.iter().enumerate() {
                        if let Some(id_str) = next_block_id.as_str() {
                            if let Ok(block_id) = Self::parse_block_id(id_str) {
                                info!("Block ID[{}]: {}", index, block_id);
                                
                                if matching {
                                    if Self::has_block(block_id, block_verifier).await? {
                                        match_id = block_id;
                                        info!("Block {} already exists locally, continuing", block_id);
                                    } else {
                                        block_list.push(match_id);
                                        block_list.push(block_id);
                                        matching = false;
                                        info!("Block {} not found locally, adding to download list", block_id);
                                    }
                                } else {
                                    block_list.push(block_id);
                                }
                                if block_list.len() >= MAX_BLOCKS_BATCH {
                                    break;
                                }
                            }
                        }
                    }

                    if block_list.is_empty() {
                        block_list.push(match_id);
                    }
                }
            }
            Err(e) => {
                debug!("Failed to get next block ids: {}", e);
                block_list.push(match_id);
            }
        }

        Ok(block_list)
    }

    async fn has_block(
        block_id: u64,
        block_verifier: &Arc<dyn BlockVerifier>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        match block_verifier.has_block(block_id).await {
            Ok(exists) => Ok(exists),
            Err(e) => {
                debug!("Failed to check if block {} exists: {}", block_id, e);
                Ok(false)
            }
        }
    }

    /// 最大重试次数
    const MAX_RETRIES: usize = 3;
    
    /// 重试延迟（毫秒）
    const RETRY_DELAY_MS: u64 = 1000;

    async fn download_blocks(
        peer_addr: std::net::SocketAddr,
        chain_block_ids: &[u64],
        common_block_height: u32,
        block_verifier: &Arc<dyn BlockVerifier>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if chain_block_ids.len() < 2 {
            return Ok(());
        }

        info!("Downloading {} blocks from peer", chain_block_ids.len() - 1);

        let stop = chain_block_ids.len() - 1;
        let mut get_list: Vec<(usize, usize)> = Vec::new();

        for start in (0..stop).step_by(SEGMENT_SIZE) {
            let seg_stop = min(start + SEGMENT_SIZE, stop);
            get_list.push((start, seg_stop));
        }

        // 并发下载所有段（带重试机制）
        // Java NRCS 限制 blockIds 数组长度不能超过 36
        let mut download_futures = Vec::new();
        
        for &(start_idx, stop_idx) in &get_list {
            let block_id = chain_block_ids.get(start_idx).copied().unwrap_or(0);

            // blockIds 不能超过 36 个，所以最多请求 36 个区块
            // start_idx 是公共区块，stop_idx 是最后一个需要下载的区块
            // 我们需要下载 (stop_idx - start_idx) 个区块
            // blockIds 应该包含从 start_idx 开始的区块 ID，最多 36 个
            let id_list: Vec<serde_json::Value> = (start_idx..stop_idx)
                .filter_map(|i| chain_block_ids.get(i).copied())
                .map(|id| serde_json::Value::String(id.to_string()))
                .collect();

            let mut request = PeerRequest::new(RequestType::GetNextBlocks, 1);
            request.set("blockIds", &id_list);
            request.set("blockId", &block_id.to_string());

            let peer_addr_clone = peer_addr;
            download_futures.push(async move {
                // 重试机制
                let mut last_error = None;
                for retry in 0..Self::MAX_RETRIES {
                    match WebsocketClient::send_request(peer_addr_clone, request.clone()).await {
                        Ok(response) => {
                            // 调试：打印响应内容
                            debug!("Response for segment {}: {:?}", start_idx, response);
                            
                            // 验证响应
                            if let Some(next_blocks) = response.get("nextBlocks").and_then(|v| v.as_array()) {
                                debug!("nextBlocks array length: {}", next_blocks.len());
                                if !next_blocks.is_empty() {
                                    return (start_idx, stop_idx, Ok(response));
                                }
                            }
                            // 空响应，重试
                            last_error = Some("Empty response".to_string());
                            warn!("Empty nextBlocks in response for segment {}, keys: {:?}", start_idx, response.as_object().map(|m| m.keys().collect::<Vec<_>>()));
                            if retry < Self::MAX_RETRIES - 1 {
                                tokio::time::sleep(tokio::time::Duration::from_millis(Self::RETRY_DELAY_MS)).await;
                            }
                        }
                        Err(e) => {
                            last_error = Some(e.to_string());
                            if retry < Self::MAX_RETRIES - 1 {
                                warn!("Retry {}/{} for segment {}: {}", retry + 1, Self::MAX_RETRIES, start_idx, e);
                                tokio::time::sleep(tokio::time::Duration::from_millis(Self::RETRY_DELAY_MS)).await;
                            }
                        }
                    }
                }
                (start_idx, stop_idx, Err(last_error.unwrap_or_else(|| "Unknown error".to_string())))
            });
        }

        // 并发执行所有下载请求
        let results = futures::future::join_all(download_futures).await;

        // 按顺序处理结果
        let mut processed = 0;
        let mut all_blocks: Vec<(usize, serde_json::Value)> = Vec::new();
        let mut failed_segments = Vec::new();

        for (start_idx, _stop_idx, result) in results {
            match result {
                Ok(response) => {
                    if let Some(next_blocks) = response.get("nextBlocks").and_then(|v| v.as_array()) {
                        if next_blocks.len() > SEGMENT_SIZE {
                            warn!("Peer {} sends {} nextBlocks (expected <= {}), but continuing...", 
                                  peer_addr, next_blocks.len(), SEGMENT_SIZE);
                        }

                        info!("Received {} blocks from peer for segment starting at {}", 
                              next_blocks.len(), start_idx);

                        for (block_idx, block_data) in next_blocks.iter().enumerate() {
                            all_blocks.push((start_idx + block_idx, block_data.clone()));
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to get next blocks for segment {} after {} retries: {}", 
                          start_idx, Self::MAX_RETRIES, e);
                    failed_segments.push(start_idx);
                }
            }
        }

        // 报告失败的段
        if !failed_segments.is_empty() {
            warn!("Failed to download {} segments: {:?}", failed_segments.len(), failed_segments);
        }

        // 按区块索引排序
        all_blocks.sort_by_key(|(idx, _)| *idx);

        // 处理所有下载的区块
        for (block_idx, block_data) in all_blocks {
            let block_height = common_block_height + block_idx as u32 + 1;
            if let Err(e) = Self::process_downloaded_block(&block_data, block_height, block_verifier).await {
                warn!("Failed to process downloaded block at height {}: {}", block_height, e);
            } else {
                processed += 1;
            }
        }

        if processed > 0 {
            info!("Downloaded and processed {} blocks total", processed);
        }

        Ok(())
    }

    async fn process_downloaded_block(
        block_data: &serde_json::Value,
        block_height: u32,
        block_verifier: &Arc<dyn BlockVerifier>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let block_json = match block_data.get("block") {
            Some(b) => b.clone(),
            None => block_data.clone(),
        };

        info!("Raw block JSON at height {}: {}", block_height, serde_json::to_string(&block_json).unwrap_or_default());

        let transactions_json = block_json.get("transactions")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut block_json_no_tx = block_json.clone();
        if let Some(obj) = block_json_no_tx.as_object_mut() {
            obj.remove("transactions");
        }

        let mut block: Block = match serde_json::from_value(block_json_no_tx) {
            Ok(b) => b,
            Err(e) => {
                warn!("Failed to deserialize block at height {}: {}", block_height, e);
                debug!("Block JSON keys: {:?}", block_json.as_object().map(|m| m.keys().collect::<Vec<_>>()));
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())) as Box<dyn std::error::Error + Send + Sync>);
            }
        };

        info!("Deserialized block at height {}: version={}, timestamp={}, prev_block_id={:?}, total_amount={}, total_fee={}, payload_length={}, base_target={}, generator_id={:?}", 
              block_height, block.version, block.timestamp, block.previous_block_id, 
              block.total_amount, block.total_fee, block.payload_length, block.base_target, block.generator_id);

        block.height = block_height;

        if block.generator_id.is_none() && block.generator_public_key.is_some() {
            block.generator_id = Some(blockchain_types::block::account_id_from_public_key(
                block.generator_public_key.as_ref().unwrap()
            ));
        }

        let mut transactions = Vec::new();
        for (tx_idx, tx_json) in transactions_json.iter().enumerate() {
            match Transaction::from_json(tx_json) {
                Ok(tx) => {
                    info!("Transaction[{}] in block {}: id={}, type={:?}, sender={}, recipient={:?}, amount={}, fee={}", 
                          tx_idx, block_height, tx.id, tx.type_id, tx.sender_id, tx.recipient_id, tx.amount, tx.fee);
                    transactions.push(tx);
                }
                Err(e) => {
                    warn!("Failed to deserialize transaction[{}] in block {}: {}", tx_idx, block_height, e);
                    debug!("Transaction JSON: {:?}", tx_json);
                }
            }
        }
        block.transactions = transactions;

        // Calculate block ID AFTER transactions are added
        if block.id.is_none() {
            block.id = Some(block.calculate_id().unwrap_or(0));
        }

        info!("Processing downloaded block: height={}, id={}, version={}, timestamp={}, generator={}, base_target={}, txs={}", 
               block.height, block.get_id(), block.version, block.timestamp, block.get_generator_id(), block.base_target, block.transactions.len());

        match block_verifier.verify_and_process(block).await {
            Ok(_) => {
                info!("Block verified and processed successfully at height {}", block_height);
                Ok(())
            }
            Err(e) => {
                warn!("Block verification/processing failed at height {}: {}", block_height, e);
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn std::error::Error + Send + Sync>)
            }
        }
    }

    fn parse_block_id(id_str: &str) -> Result<u64, std::num::ParseIntError> {
        id_str.parse::<u64>()
    }
}