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
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use std::cmp::min;

const SEGMENT_SIZE: usize = 36;
const MAX_BLOCKS_BATCH: usize = 720;
const MAX_BLOCKS_LIMIT: usize = 1440;
const SYNC_INTERVAL_SECS: u64 = 1;
const SYNC_EMPTY_RETRY: u32 = 3;

#[derive(Debug, Clone, Default)]
pub struct SyncState {
    pub is_syncing: bool,
    pub start_time: Option<Instant>,
    pub start_height: u32,
    pub current_height: u32,
    pub target_height: u32,
    pub blocks_downloaded: usize,
    pub blocks_processed: usize,
    pub errors_count: usize,
    pub active_peers: usize,
    pub last_sync_time: Option<Instant>,
}

impl SyncState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn progress_percent(&self) -> f64 {
        if self.target_height <= self.start_height {
            return 100.0;
        }
        let total = self.target_height - self.start_height;
        let done = self.current_height.saturating_sub(self.start_height);
        (done as f64 / total as f64 * 100.0).min(100.0)
    }

    pub fn elapsed_secs(&self) -> u64 {
        self.start_time
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0)
    }

    pub fn blocks_per_sec(&self) -> f64 {
        let elapsed = self.elapsed_secs();
        if elapsed == 0 {
            return 0.0;
        }
        self.blocks_processed as f64 / elapsed as f64
    }

    pub fn eta_secs(&self) -> u64 {
        let rate = self.blocks_per_sec();
        if rate == 0.0 {
            return 0;
        }
        let remaining = self.target_height.saturating_sub(self.current_height);
        (remaining as f64 / rate) as u64
    }
}

pub struct BlockchainSyncDaemon {
    config: P2PConfig,
    running: Arc<RwLock<bool>>,
    is_downloading: Arc<RwLock<bool>>,
    sync_state: Arc<RwLock<SyncState>>,
}

impl BlockchainSyncDaemon {
    pub fn new(config: P2PConfig) -> Self {
        Self {
            config,
            running: Arc::new(RwLock::new(false)),
            is_downloading: Arc::new(RwLock::new(false)),
            sync_state: Arc::new(RwLock::new(SyncState::new())),
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
        let sync_state = Arc::clone(&self.sync_state);

        tokio::spawn(async move {
            info!("Blockchain sync daemon started");
            Self::sync_loop(peers, config, running, is_downloading, sync_state, block_verifier).await;
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

    pub async fn get_sync_state(&self) -> SyncState {
        self.sync_state.read().await.clone()
    }

    async fn update_sync_progress(
        sync_state: &Arc<RwLock<SyncState>>,
        current_height: u32,
        blocks_processed: usize,
    ) {
        let mut state = sync_state.write().await;
        state.current_height = current_height;
        state.blocks_processed = blocks_processed;
        
        if state.blocks_processed % 100 == 0 && state.blocks_processed > 0 {
            let progress = state.progress_percent();
            let rate = state.blocks_per_sec();
            let eta = state.eta_secs();
            debug!(
                "Sync progress: {:.1}% | Height: {} | Blocks: {} | Rate: {:.1} blocks/s | ETA: {}s",
                progress, current_height, blocks_processed, rate, eta
            );
        }
    }

    async fn log_sync_summary(sync_state: &Arc<RwLock<SyncState>>) {
        let state = sync_state.read().await;
        if state.blocks_processed > 0 {
            let elapsed = state.elapsed_secs();
            let rate = state.blocks_per_sec();
            debug!(
                "Sync completed: {} blocks in {}s ({:.1} blocks/s)",
                state.blocks_processed, elapsed, rate
            );
        }
    }

    async fn sync_loop(
        peers: Arc<crate::peer::Peers>,
        _config: P2PConfig,
        running: Arc<RwLock<bool>>,
        is_downloading: Arc<RwLock<bool>>,
        sync_state: Arc<RwLock<SyncState>>,
        block_verifier: Arc<dyn BlockVerifier>,
    ) {
        loop {
            if !*running.read().await {
                break;
            }

            {
                let mut state = sync_state.write().await;
                state.is_syncing = true;
                state.start_time = Some(Instant::now());
                state.start_height = block_verifier.get_height().await.unwrap_or(0);
                state.current_height = state.start_height;
                state.blocks_downloaded = 0;
                state.blocks_processed = 0;
                state.errors_count = 0;
            }

            let mut consecutive_empty = 0u32;
            let mut total_processed = 0usize;
            
            loop {
                if !*running.read().await {
                    break;
                }

                match Self::download_peer(&peers, &is_downloading, &sync_state, &block_verifier).await {
                    Ok(downloaded) => {
                        if downloaded == 0 {
                            consecutive_empty += 1;
                            if consecutive_empty >= SYNC_EMPTY_RETRY {
                                debug!("No more blocks to download after {} attempts", SYNC_EMPTY_RETRY);
                                break;
                            }
                        } else {
                            consecutive_empty = 0;
                            total_processed += downloaded;
                            
                            let current_height = block_verifier.get_height().await.unwrap_or(0);
                            info!("Downloaded {} blocks (total: {}), continuing sync... | Height: {}", downloaded, total_processed, current_height);
                            Self::update_sync_progress(&sync_state, current_height, total_processed).await;
                        }
                    }
                    Err(e) => {
                        debug!("Sync error: {}", e);
                        consecutive_empty += 1;
                        {
                            let mut state = sync_state.write().await;
                            state.errors_count += 1;
                        }
                        if consecutive_empty >= SYNC_EMPTY_RETRY {
                            break;
                        }
                    }
                }

                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            {
                let mut state = sync_state.write().await;
                state.is_syncing = false;
                state.last_sync_time = Some(Instant::now());
            }

            Self::log_sync_summary(&sync_state).await;

            *is_downloading.write().await = false;

            tokio::time::sleep(Duration::from_secs(SYNC_INTERVAL_SECS)).await;
        }
    }

    async fn download_peer(
        peers: &Arc<crate::peer::Peers>,
        is_downloading: &Arc<RwLock<bool>>,
        sync_state: &Arc<RwLock<SyncState>>,
        block_verifier: &Arc<dyn BlockVerifier>,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let connected_peers = peers.get_known_peers().await;
        let connected_peers: Vec<Peer> = connected_peers.into_iter()
            .filter(|p| p.state == PeerState::Connected)
            .collect();

        if connected_peers.is_empty() {
            debug!("No connected peers for sync");
            return Ok(0);
        }

        let peer_addrs: Vec<std::net::SocketAddr> = connected_peers.iter()
            .map(|p| p.address)
            .collect();

        {
            let mut state = sync_state.write().await;
            state.active_peers = peer_addrs.len();
        }

        let feeder_peer = match connected_peers.first() {
            Some(p) => p.clone(),
            None => {
                debug!("No suitable peer found for sync");
                return Ok(0);
            }
        };

        let feeder_addr = feeder_peer.address;
        let unknown_label = "unknown".to_string();
        let peer_label = feeder_peer.announced_address.as_ref().unwrap_or(&unknown_label);

        let cumulative_difficulty = Self::get_cumulative_difficulty(feeder_addr).await?;
        if cumulative_difficulty.is_none() {
            debug!("Failed to get cumulative difficulty from peer");
            return Ok(0);
        }

        let peer_cumulative_difficulty = cumulative_difficulty.unwrap();

        let local_cumulative_difficulty = block_verifier.get_cumulative_difficulty().await
            .unwrap_or_else(|_| "0".to_string());
        
        debug!("Comparing difficulties: peer={} (len={}) vs local={} (len={})", 
               peer_cumulative_difficulty, peer_cumulative_difficulty.len(),
               local_cumulative_difficulty, local_cumulative_difficulty.len());
        
        // 使用字符串长度和字典序比较 BigInteger 值
        fn compare_bigint_strings(a: &str, b: &str) -> std::cmp::Ordering {
            let a_len = a.len();
            let b_len = b.len();
            if a_len != b_len {
                a_len.cmp(&b_len)
            } else {
                a.cmp(b)
            }
        }

        let cmp_result = compare_bigint_strings(&peer_cumulative_difficulty, &local_cumulative_difficulty);
        debug!("Compare result: {:?}", cmp_result);
        
        if cmp_result != std::cmp::Ordering::Greater {
            debug!("Peer cumulative difficulty ({}) not higher than local ({})", 
                   peer_cumulative_difficulty, local_cumulative_difficulty);
            return Ok(0);
        }

        if peer_cumulative_difficulty == local_cumulative_difficulty {
            return Ok(0);
        }

        debug!("Peer {} has higher cumulative difficulty: {} > {}",
              peer_label, peer_cumulative_difficulty, local_cumulative_difficulty);

        let last_local_block_id = block_verifier.get_last_block_id().await
            .ok()
            .flatten()
            .unwrap_or(0);
        
        let common_block_id = if last_local_block_id != 0 {
            debug!("Last local block ID: {}, continuing sync from there", last_local_block_id);
            last_local_block_id
        } else {
            debug!("No local blocks, starting from genesis block");
            GENESIS_BLOCK_ID
        };

        let common_block_height = block_verifier.get_block_height(common_block_id).await
            .ok()
            .flatten()
            .unwrap_or(0);

        let current_height = block_verifier.get_height().await.unwrap_or(0);
        
        if current_height > 0 && current_height - common_block_height >= blockchain_types::constants::MAX_ROLLBACK {
            warn!("Common block is too far behind ({} blocks >= {}), skipping sync from peer {}", 
                  current_height - common_block_height, blockchain_types::constants::MAX_ROLLBACK, peer_label);
            return Ok(0);
        }

        debug!("Starting sync from block ID: {} (height: {})", common_block_id, common_block_height);

        let chain_block_ids = Self::get_block_ids_after_common(feeder_addr, common_block_id, block_verifier).await?;
        if chain_block_ids.len() < 2 {
            debug!("Not enough blocks after common block");
            return Ok(0);
        }

        let blocks_to_download = chain_block_ids.len() - 1;
        debug!("Blocks to download: {} from {} peer(s)", blocks_to_download, peer_addrs.len());

        {
            let mut state = sync_state.write().await;
            state.target_height = common_block_height + blocks_to_download as u32;
            state.blocks_downloaded = blocks_to_download;
        }

        if !*is_downloading.read().await && blocks_to_download > 10 {
            info!("Blockchain download in progress");
            *is_downloading.write().await = true;
        }

        let downloaded = Self::download_blocks_multi_peer(
            &peer_addrs,
            &chain_block_ids,
            common_block_height,
            block_verifier,
        ).await?;

        Ok(downloaded)
    }

    async fn get_cumulative_difficulty(
        peer_addr: std::net::SocketAddr,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let request = PeerRequest::new(RequestType::GetCumulativeDifficulty, 1);

        match WebsocketClient::send_request(peer_addr, request).await {
            Ok(response) => {
                if let Some(cumulative_diff) = response.get("cumulativeDifficulty").and_then(|v| v.as_str()) {
                    Ok(Some(cumulative_diff.to_string()))
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

    #[allow(dead_code)]
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
                request.set("lastBlockId", "0");
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
                                    debug!("Checking milestone block ID: {}", block_id);
                                    
                                    if Self::has_block(block_id, block_verifier).await? {
                                        debug!("Found common milestone block: {}", block_id);
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

        debug!("Requesting block IDs after {} from peer", start_block_id);

        loop {
            let mut request = PeerRequest::new(RequestType::GetNextBlockIds, 1);
            request.set("blockId", match_id.to_string());
            request.set("limit", MAX_BLOCKS_LIMIT as i32);

            match WebsocketClient::send_request(peer_addr, request).await {
                Ok(response) => {
                    if let Some(next_block_ids) = response.get("nextBlockIds").and_then(|v| v.as_array()) {
                        debug!("Received {} block IDs from peer (match_id={})", next_block_ids.len(), match_id);

                        if next_block_ids.is_empty() {
                            break;
                        }

                        if next_block_ids.len() > MAX_BLOCKS_LIMIT {
                            warn!("Peer {} sends too many nextBlockIds ({})", peer_addr, next_block_ids.len());
                            return Ok(Vec::new());
                        }

                        let mut matching = true;
                        for next_block_id in next_block_ids.iter() {
                            if let Some(id_str) = next_block_id.as_str() {
                                if let Ok(block_id) = Self::parse_block_id(id_str) {
                                    if matching {
                                        if Self::has_block(block_id, block_verifier).await? {
                                            match_id = block_id;
                                        } else {
                                            block_list.push(match_id);
                                            block_list.push(block_id);
                                            matching = false;
                                        }
                                    } else {
                                        block_list.push(block_id);
                                    }
                                    if block_list.len() >= MAX_BLOCKS_BATCH {
                                        return Ok(block_list);
                                    }
                                }
                            }
                        }

                        if !matching {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                Err(e) => {
                    debug!("Failed to get next block ids: {}", e);
                    break;
                }
            }
        }

        if block_list.is_empty() {
            block_list.push(match_id);
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

    async fn download_blocks_multi_peer(
        peer_addrs: &[std::net::SocketAddr],
        chain_block_ids: &[u64],
        common_block_height: u32,
        block_verifier: &Arc<dyn BlockVerifier>,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        if chain_block_ids.len() < 2 {
            return Ok(0);
        }

        let stop = chain_block_ids.len() - 1;
        let mut get_list: Vec<(usize, usize)> = Vec::new();

        for start in (0..stop).step_by(SEGMENT_SIZE) {
            let seg_stop = min(start + SEGMENT_SIZE, stop);
            get_list.push((start, seg_stop));
        }

        let peer_count = peer_addrs.len().max(1);
        debug!("Downloading {} blocks using {} peer(s), {} segments",
              chain_block_ids.len() - 1, peer_count, get_list.len());

        let mut download_futures = Vec::new();
        
        for (segment_idx, &(start_idx, stop_idx)) in get_list.iter().enumerate() {
            let peer_addr = peer_addrs[segment_idx % peer_count];
            let common_block_id = chain_block_ids.first().copied().unwrap_or(0);
            
            // Reference: Java GetNextBlocks.call() line 82:
            //   for (int i = start + 1; i <= stop; i++) {
            //       idList.add(Long.toUnsignedString(blockIds.get(i)));
            //   }
            //   request.put("blockId", Long.toUnsignedString(blockIds.get(start)));
            //
            // Java requests blockIds[start+1..stop] with blockId=blockIds[start]
            // So we need to request chain_block_ids[start_idx+1..stop_idx]
            // with blockId=chain_block_ids[start_idx]
            let prev_block_id = chain_block_ids.get(start_idx).copied().unwrap_or(common_block_id);
            let id_start_idx = start_idx + 1;

            let id_list: Vec<serde_json::Value> = (id_start_idx..=stop_idx)
                .filter_map(|i| chain_block_ids.get(i).copied())
                .map(|id| serde_json::Value::String(id.to_string()))
                .collect();

            let mut request = PeerRequest::new(RequestType::GetNextBlocks, 1);
            request.set("blockIds", &id_list);
            request.set("blockId", prev_block_id.to_string());

            download_futures.push(async move {
                let mut last_error = None;
                for retry in 0..Self::MAX_RETRIES {
                    match WebsocketClient::send_request(peer_addr, request.clone()).await {
                        Ok(response) => {
                            if let Some(next_blocks) = response.get("nextBlocks").and_then(|v| v.as_array()) {
                                if !next_blocks.is_empty() {
                                    return (start_idx, stop_idx, Ok(response));
                                }
                            }
                            last_error = Some("Empty response".to_string());
                            if retry < Self::MAX_RETRIES - 1 {
                                tokio::time::sleep(tokio::time::Duration::from_millis(Self::RETRY_DELAY_MS)).await;
                            }
                        }
                        Err(e) => {
                            last_error = Some(e.to_string());
                            if retry < Self::MAX_RETRIES - 1 {
                                tokio::time::sleep(tokio::time::Duration::from_millis(Self::RETRY_DELAY_MS)).await;
                            }
                        }
                    }
                }
                (start_idx, stop_idx, Err(last_error.unwrap_or_else(|| "Unknown error".to_string())))
            });
        }

        let results = futures::future::join_all(download_futures).await;

        let mut processed = 0;
        let mut all_blocks: Vec<(usize, serde_json::Value)> = Vec::new();
        let mut failed_segments = Vec::new();

        for (start_idx, _stop_idx, result) in results {
            match result {
                Ok(response) => {
                    if let Some(next_blocks) = response.get("nextBlocks").and_then(|v| v.as_array()) {
                        debug!("Received {} blocks for segment starting at {}", 
                              next_blocks.len(), start_idx);

                        for (block_idx, block_data) in next_blocks.iter().enumerate() {
                            all_blocks.push((start_idx + block_idx, block_data.clone()));
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to get blocks for segment {}: {}", start_idx, e);
                    failed_segments.push(start_idx);
                }
            }
        }

        if !failed_segments.is_empty() {
            warn!("Failed to download {} segments: {:?}", failed_segments.len(), failed_segments);
        }

        all_blocks.sort_by_key(|(idx, _)| *idx);

        for (block_idx, block_data) in all_blocks {
            let block_height = common_block_height + block_idx as u32 + 1;
            if let Err(e) = Self::process_downloaded_block(&block_data, block_height, block_verifier).await {
                error!("Failed to process downloaded block at height {}: {}", block_height, e);
            } else {
                processed += 1;
            }
        }

        if processed > 0 {
            debug!("Downloaded and processed {} blocks total", processed);
        }

        Ok(processed)
    }

    #[allow(dead_code)]
    async fn download_blocks(
        peer_addr: std::net::SocketAddr,
        chain_block_ids: &[u64],
        common_block_height: u32,
        block_verifier: &Arc<dyn BlockVerifier>,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        Self::download_blocks_multi_peer(&[peer_addr], chain_block_ids, common_block_height, block_verifier).await
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
                error!("Failed to deserialize block at height {}: {}", block_height, e);
                debug!("Block JSON keys: {:?}", block_json.as_object().map(|m| m.keys().collect::<Vec<_>>()));
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())) as Box<dyn std::error::Error + Send + Sync>);
            }
        };

        // Reference: Java Block.setPrevious(Block block):
        //   if (block != null) {
        //       this.setHeight(block.getHeight() + 1);
        //   } else {
        //       this.setHeight(0);
        //   }
        //
        // Java calculates height from the PREVIOUS BLOCK's height + 1,
        // using previousBlockId to look up the previous block.
        let calculated_height = if let Some(prev_id) = block.previous_block_id.filter(|&id| id != 0) {
            match block_verifier.get_block_height(prev_id).await {
                Ok(Some(prev_height)) => prev_height + 1,
                Ok(None) => {
                    warn!("Previous block {} not found in DB, falling back to index height {}", prev_id, block_height);
                    block_height
                }
                Err(e) => {
                    warn!("Error querying previous block {} height: {}, falling back to index height {}", prev_id, e, block_height);
                    block_height
                }
            }
        } else {
            0
        };

        if calculated_height != block_height {
            warn!("Height mismatch: calculated={} (prev_block+1), expected={}. Using calculated height.",
                  calculated_height, block_height);
        }

        block.height = calculated_height;

        if block.generator_id.is_none() {
            if let Some(pub_key) = &block.generator_public_key {
                block.generator_id = Some(blockchain_types::block::account_id_from_public_key(pub_key));
            }
        }

        let mut transactions = Vec::new();
        for (tx_idx, tx_json) in transactions_json.iter().enumerate() {
            match Transaction::from_json(tx_json) {
                Ok(mut tx) => {
                    // 对应 Java: transaction.setBlock(this) → this.setBlockTimestamp(block.getTimestamp())
                    // block_timestamp 必须设置为所属区块的 timestamp
                    tx.block_timestamp = block.timestamp;
                    debug!("Transaction[{}] in block {}: id={}, type={:?}, sender={}, recipient={:?}, amount={}, fee={}, block_ts={}",
                          tx_idx, block_height, tx.id, tx.type_id, tx.sender_id, tx.recipient_id, tx.amount, tx.fee, tx.block_timestamp);
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

        debug!("Processing downloaded block: height={}, id={}, version={}, timestamp={}, generator={}, base_target={}, txs={}",
               block.height, block.get_id(), block.version, block.timestamp, block.get_generator_id(), block.base_target, block.transactions.len());

        match block_verifier.verify_and_process(block).await {
            Ok(_) => {
                debug!("Block verified and processed successfully at height {}", block_height);
                Ok(())
            }
            Err(e) => {
                warn!("Block verification/processing failed at height {}: {}", block_height, e);
                Err(Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error + Send + Sync>)
            }
        }
    }

    fn parse_block_id(id_str: &str) -> Result<u64, std::num::ParseIntError> {
        id_str.parse::<u64>()
    }
}