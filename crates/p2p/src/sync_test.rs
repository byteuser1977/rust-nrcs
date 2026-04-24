//! 区块同步协议测试
//!
//! 测试与 Java NRCS 的区块同步协议兼容性

use std::sync::Arc;
use crate::protocol::{PeerRequest, RequestType};
use crate::handlers::{GetMilestoneBlockIdsHandler, GetNextBlockIdsHandler, GetNextBlocksHandler};
use crate::peer::{Peer, Peers};

fn create_test_peers() -> Arc<Peers> {
    let my_peer = Peer::new("127.0.0.1:17984".parse().unwrap(), false);
    Arc::new(Peers::new(my_peer))
}

#[tokio::test]
async fn test_get_milestone_block_ids_handler_creation() {
    let _handler = GetMilestoneBlockIdsHandler::new();
}

#[tokio::test]
async fn test_get_milestone_block_ids_without_repo() {
    let handler = GetMilestoneBlockIdsHandler::new();
    let request = PeerRequest::new(RequestType::GetMilestoneBlockIds, 1);
    
    let response = handler.handle(request).await;
    assert!(response.is_object());
}

#[tokio::test]
async fn test_get_next_block_ids_handler_creation() {
    let _handler = GetNextBlockIdsHandler::new();
}

#[tokio::test]
async fn test_get_next_block_ids_without_repo() {
    let handler = GetNextBlockIdsHandler::new();
    let mut request = PeerRequest::new(RequestType::GetNextBlockIds, 1);
    request.set("blockId", "123456789");
    request.set("limit", 100);
    
    let response = handler.handle(request).await;
    assert!(response.is_object());
}

#[tokio::test]
async fn test_get_next_blocks_handler_creation() {
    let peers = create_test_peers();
    let _handler = GetNextBlocksHandler::new(peers);
}

#[tokio::test]
async fn test_get_next_blocks_without_repo() {
    let peers = create_test_peers();
    let handler = GetNextBlocksHandler::new(Arc::clone(&peers));
    let mut request = PeerRequest::new(RequestType::GetNextBlocks, 1);
    request.set("blockId", "123456789");
    request.set("limit", 36);
    
    let response = handler.handle(request, peers).await;
    assert!(response.is_object());
}

#[test]
fn test_sync_request_protocol_format() {
    let mut request = PeerRequest::new(RequestType::GetMilestoneBlockIds, 1);
    request.set("lastBlockId", "90399831811024079");
    
    let json = serde_json::to_string(&request).unwrap();
    
    assert!(json.contains("\"protocol\":1"));
    assert!(json.contains("\"requestType\":\"getMilestoneBlockIds\""));
    assert!(json.contains("\"lastBlockId\":\"90399831811024079\""));
}

#[test]
fn test_sync_constants_match_java() {
    use blockchain_types::sync::{MAX_ROLLBACK, SEGMENT_SIZE, MAX_BLOCK_IDS, MAX_MILESTONE_IDS};
    
    assert_eq!(MAX_ROLLBACK, 720);
    assert_eq!(SEGMENT_SIZE, 36);
    assert_eq!(MAX_BLOCK_IDS, 1440);
    assert_eq!(MAX_MILESTONE_IDS, 10);
}

#[test]
fn test_block_syncer_creation() {
    use blockchain_types::sync::BlockSyncer;
    
    let syncer = BlockSyncer::new();
    assert!(!syncer.is_syncing());
}

#[test]
fn test_block_syncer_start_stop() {
    use blockchain_types::sync::BlockSyncer;
    
    let mut syncer = BlockSyncer::new();
    syncer.start_sync();
    assert!(syncer.is_syncing());
    
    syncer.stop_sync();
    assert!(!syncer.is_syncing());
}

#[test]
fn test_block_downloader_creation() {
    use blockchain_types::sync::BlockDownloader;
    
    let _downloader = BlockDownloader::new();
}

#[test]
fn test_block_importer_creation() {
    use blockchain_types::sync::BlockImporter;
    
    let _importer = BlockImporter::new();
}

#[test]
fn test_peer_info_better_chain() {
    use blockchain_types::sync::PeerInfo;
    use num_bigint::BigUint;
    
    let peer = PeerInfo::new("peer1".to_string(), "127.0.0.1:8080".to_string());
    let local_diff = BigUint::from(1000u64);
    
    assert!(!peer.has_better_chain(&local_diff));
    
    let mut better_peer = peer.clone();
    better_peer.cumulative_difficulty = BigUint::from(2000u64);
    assert!(better_peer.has_better_chain(&local_diff));
}

#[test]
fn test_find_common_milestone_block() {
    use blockchain_types::sync::BlockSyncer;
    
    let syncer = BlockSyncer::new();
    let local_ids = vec![1, 2, 3, 4, 5];
    let peer_ids = vec![6, 5, 4, 3];
    
    let result = syncer.find_common_milestone_block(&local_ids, &peer_ids);
    assert_eq!(result.unwrap(), 5);
}

#[test]
fn test_get_block_ids_after_common() {
    use blockchain_types::sync::BlockSyncer;
    
    let syncer = BlockSyncer::new();
    let peer_ids = vec![1, 2, 3, 4, 5, 6, 7];
    
    let result = syncer.get_block_ids_after_common(3, &peer_ids);
    assert_eq!(result, vec![4, 5, 6, 7]);
}

#[test]
fn test_validate_download_range() {
    use blockchain_types::sync::BlockSyncer;
    
    let syncer = BlockSyncer::new();
    
    assert!(syncer.validate_download_range(100, 200).is_ok());
    assert!(syncer.validate_download_range(100, 900).is_err());
}
