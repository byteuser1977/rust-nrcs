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
use blockchain_types::constants::GENESIS_BLOCK_ID;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use std::cmp::min;

const SEGMENT_SIZE: usize = 10;
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
            Self::get_common_milestone_block_id(peer_addr).await?
        } else {
            info!("No local blocks, starting from genesis block");
            GENESIS_BLOCK_ID
        };
        
        if common_block_id == 0 {
            debug!("Could not find common milestone block");
            return Ok(());
        }

        info!("Common milestone block id: {}", common_block_id);

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

        Self::download_blocks(peer_addr, &chain_block_ids, block_verifier).await?;

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
                            return Ok(1);
                        }

                        if milestone_ids.len() > 20 {
                            warn!("Peer {} sends too many milestoneBlockIds", peer_addr);
                            return Ok(0);
                        }

                        for milestone_id in milestone_ids {
                            if let Some(id_str) = milestone_id.as_str() {
                                let block_id = Self::parse_block_id(id_str);
                                if let Ok(block_id) = block_id {
                                    last_milestone_block_id = Some(id_str.to_string());
                                    if block_id == 1 {
                                        return Ok(block_id);
                                    }
                                }
                            }
                        }

                        if last_milestone_block_id.is_some() {
                            let last_id_str = last_milestone_block_id.as_ref().unwrap();
                            if let Ok(block_id) = Self::parse_block_id(last_id_str) {
                                return Ok(block_id);
                            }
                        }
                    }
                    return Ok(0);
                }
                Err(e) => {
                    debug!("Failed to get milestone block ids: {}", e);
                    return Ok(0);
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

    async fn download_blocks(
        peer_addr: std::net::SocketAddr,
        chain_block_ids: &[u64],
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

        let mut processed = 0;

        for &(start_idx, stop_idx) in &get_list {
            let block_id = chain_block_ids.get(start_idx).copied().unwrap_or(0);

            let id_list: Vec<serde_json::Value> = (start_idx..=stop_idx)
                .filter_map(|i| chain_block_ids.get(i).copied())
                .map(|id| serde_json::Value::String(id.to_string()))
                .collect();

            let mut request = PeerRequest::new(RequestType::GetNextBlocks, 1);
            request.set("blockIds", &id_list);
            request.set("blockId", &block_id.to_string());

            match WebsocketClient::send_request(peer_addr, request).await {
                Ok(response) => {
                    if let Some(next_blocks) = response.get("nextBlocks").and_then(|v| v.as_array()) {
                        if next_blocks.len() > SEGMENT_SIZE {
                            warn!("Peer {} sends {} nextBlocks (expected <= {}), but continuing...", 
                                  peer_addr, next_blocks.len(), SEGMENT_SIZE);
                        }

                        processed += next_blocks.len();
                        info!("Received {} blocks from peer, processing...", next_blocks.len());

                        let base_height = start_idx as u32 + 1;
                        for (block_idx, block_data) in next_blocks.iter().enumerate() {
                            if let Err(e) = Self::process_downloaded_block(&block_data, base_height + block_idx as u32, block_verifier).await {
                                warn!("Failed to process downloaded block: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to get next blocks: {}", e);
                }
            }
        }

        if processed > 0 {
            info!("Downloaded {} blocks total", processed);
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
            None => {
                block_data.clone()
            }
        };

        debug!("Raw block JSON: {}", serde_json::to_string(&block_json).unwrap_or_default());

        let mut normalized_json = Self::normalize_block_json(block_json);

        if let Some(obj) = normalized_json.as_object_mut() {
            if !obj.contains_key("height") {
                obj.insert("height".to_string(), serde_json::Value::Number(block_height.into()));
            }
            if !obj.contains_key("nonce") {
                obj.insert("nonce".to_string(), serde_json::Value::Number(0.into()));
            }
            if !obj.contains_key("cumulative_difficulty") {
                obj.insert("cumulative_difficulty".to_string(), serde_json::Value::Array(vec![]));
            }
            if !obj.contains_key("base_target") {
                obj.insert("base_target".to_string(), serde_json::Value::Number(1_000_000.into()));
            }
            if !obj.contains_key("total_amount") && obj.contains_key("total_amount_n_q_t") {
                let v = obj.get("total_amount_n_q_t").cloned().unwrap_or(serde_json::Value::Number(0.into()));
                obj.insert("total_amount".to_string(), v);
            }
            if !obj.contains_key("total_fee") && obj.contains_key("total_fee_n_q_t") {
                let v = obj.get("total_fee_n_q_t").cloned().unwrap_or(serde_json::Value::Number(0.into()));
                obj.insert("total_fee".to_string(), v);
            }
        }

        debug!("Normalized block JSON keys: {:?}", normalized_json.as_object().map(|m| {
            m.iter().map(|(k, v)| {
                let len = match v {
                    serde_json::Value::Array(arr) => arr.len(),
                    _ => 0
                };
                format!("{}:{}", k, len)
            }).collect::<Vec<_>>()
        }));

        let block: Block = match serde_json::from_value(normalized_json) {
            Ok(b) => b,
            Err(e) => {
                debug!("Failed to deserialize block: {}", e);
                return Ok(());
            }
        };

        debug!("Processing downloaded block: height={}", block.height);

        match block_verifier.verify_and_process(block).await {
            Ok(_) => {
                info!("Block verified and processed successfully");
                Ok(())
            }
            Err(e) => {
                warn!("Block verification/processing failed: {}", e);
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn std::error::Error + Send + Sync>)
            }
        }
    }

    fn parse_block_id(id_str: &str) -> Result<u64, std::num::ParseIntError> {
        id_str.parse::<u64>()
    }

    fn camel_to_snake(s: &str) -> String {
        let mut result = String::new();
        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap_or(c));
        }
        result
    }

    fn convert_json_key(value: serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let converted: serde_json::Map<String, serde_json::Value> = map
                    .into_iter()
                    .map(|(k, v)| {
                        let new_key = Self::camel_to_snake(&k);
                        (new_key, Self::convert_json_key(v))
                    })
                    .collect();
                serde_json::Value::Object(converted)
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.into_iter().map(Self::convert_json_key).collect())
            }
            other => other,
        }
    }

    fn normalize_block_json(json: serde_json::Value) -> serde_json::Value {
        let hex_fields = [
            "generation_signature",
            "block_signature",
            "previous_block_hash",
            "payload_hash",
            "generator_public_key",
        ];

        let account_id_fields = ["generator_id"];

        fn convert_transaction(tx_json: serde_json::Value) -> serde_json::Value {
            match tx_json {
                serde_json::Value::Object(mut map) => {
                    let converted: serde_json::Map<String, serde_json::Value> = map
                        .into_iter()
                        .map(|(k, v)| {
                            let new_key = BlockchainSyncDaemon::camel_to_snake(&k);
                            let converted_v = match new_key.as_str() {
                                "sender_public_key" | "full_hash" | "referenced_transaction_full_hash" => {
                                    match &v {
                                        serde_json::Value::String(s) => {
                                            match hex::decode(s) {
                                                Ok(bytes) => {
                                                    serde_json::Value::Array(
                                                        bytes.iter().map(|b| serde_json::Number::from(*b)).map(serde_json::Value::Number).collect()
                                                    )
                                                }
                                                Err(_) => v,
                                            }
                                        }
                                        _ => v,
                                    }
                                }
                                "signature" => {
                                    match &v {
                                        serde_json::Value::String(s) => {
                                            match hex::decode(s) {
                                                Ok(bytes) => {
                                                    serde_json::Value::Array(
                                                        bytes.iter().map(|b| serde_json::Number::from(*b)).map(serde_json::Value::Number).collect()
                                                    )
                                                }
                                                Err(_) => v,
                                            }
                                        }
                                        _ => v,
                                    }
                                }
                                _ => v
                            };
                            (new_key, converted_v)
                        })
                        .collect();
                    serde_json::Value::Object(converted)
                }
                other => other
            }
        }

        fn convert_value(value: serde_json::Value, hex_fields: &[&str], account_id_fields: &[&str]) -> serde_json::Value {
            match value {
                serde_json::Value::Object(map) => {
                    let converted: serde_json::Map<String, serde_json::Value> = map
                        .into_iter()
                        .map(|(k, v)| {
                            let mut new_key = BlockchainSyncDaemon::camel_to_snake(&k);
                            let converted_v = if new_key == "transactions" {
                                match v {
                                    serde_json::Value::Array(txs) => {
                                        serde_json::Value::Array(
                                            txs.into_iter().map(convert_transaction).collect()
                                        )
                                    }
                                    other => other
                                }
                            } else if hex_fields.contains(&new_key.as_str()) {
                                match &v {
                                    serde_json::Value::String(s) => {
                                        match hex::decode(s) {
                                            Ok(bytes) => {
                                                serde_json::Value::Array(
                                                    bytes.iter().map(|b| serde_json::Number::from(*b)).map(serde_json::Value::Number).collect()
                                                )
                                            }
                                            Err(_) => v,
                                        }
                                    }
                                    _ => v,
                                }
                            } else if account_id_fields.contains(&new_key.as_str()) {
                                match &v {
                                    serde_json::Value::String(s) => {
                                        debug!("Converting account_id field: {} = {}", new_key, s);
                                        match hex::decode(s) {
                                            Ok(bytes) if bytes.len() == 32 => {
                                                let mut buf = [0u8; 8];
                                                buf.copy_from_slice(&bytes[..8]);
                                                let account_id = u64::from_le_bytes(buf);
                                                debug!("Converted {} to account_id: {}", new_key, account_id);
                                                serde_json::Value::Number(serde_json::Number::from(account_id))
                                            }
                                            Ok(bytes) => {
                                                debug!("Invalid length for {}: {}", new_key, bytes.len());
                                                v
                                            }
                                            Err(e) => {
                                                debug!("Hex decode error for {}: {}", new_key, e);
                                                v
                                            }
                                        }
                                    }
                                    _ => {
                                        debug!("Non-string value for {}: {:?}", new_key, v);
                                        v
                                    }
                                }
                            } else if new_key == "previous_block" {
                                match &v {
                                    serde_json::Value::String(s) => {
                                        match s.parse::<u64>() {
                                            Ok(num) => {
                                                debug!("Converted {} to u64: {}", new_key, num);
                                                serde_json::Value::Number(serde_json::Number::from(num))
                                            }
                                            Err(e) => {
                                                debug!("Failed to parse {} as u64: {}", new_key, e);
                                                v
                                            }
                                        }
                                    }
                                    _ => v
                                }
                            } else {
                                convert_value(v, hex_fields, account_id_fields)
                            };
                            (new_key, converted_v)
                        })
                        .collect();
                    serde_json::Value::Object(converted)
                }
                serde_json::Value::Array(arr) => {
                    serde_json::Value::Array(arr.into_iter().map(|v| convert_value(v, hex_fields, account_id_fields)).collect())
                }
                other => other,
            }
        }

        convert_value(json, &hex_fields, &account_id_fields)
    }
}