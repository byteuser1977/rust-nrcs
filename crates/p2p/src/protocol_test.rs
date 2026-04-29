//! P2P 协议兼容性测试
//!
//! 验证 Rust NRCS P2P 协议与 Java NRCS 的兼容性

use crate::protocol::{PeerRequest, RequestType};

#[test]
fn test_protocol_parameter_required() {
    let request = PeerRequest::new(RequestType::GetCumulativeDifficulty, 1);
    assert_eq!(request.protocol, 1);
    
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"protocol\":1"));
    assert!(json.contains("\"requestType\":\"getCumulativeDifficulty\""));
}

#[test]
fn test_get_cumulative_difficulty_request() {
    let request = PeerRequest::new(RequestType::GetCumulativeDifficulty, 1);
    let json = serde_json::to_string(&request).unwrap();
    
    let expected = r#"{"requestType":"getCumulativeDifficulty","protocol":1}"#;
    assert_eq!(json, expected);
}

#[test]
fn test_get_milestone_block_ids_request() {
    let mut request = PeerRequest::new(RequestType::GetMilestoneBlockIds, 1);
    request.set("lastBlockId", "123456789");
    
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"lastBlockId\":\"123456789\""));
    assert!(json.contains("\"protocol\":1"));
}

#[test]
fn test_get_next_block_ids_request() {
    let mut request = PeerRequest::new(RequestType::GetNextBlockIds, 1);
    request.set("blockId", "123456789");
    request.set("limit", 1440);
    
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"blockId\":\"123456789\""));
    assert!(json.contains("\"limit\":1440"));
    assert!(json.contains("\"protocol\":1"));
}

#[test]
fn test_get_next_blocks_request() {
    let mut request = PeerRequest::new(RequestType::GetNextBlocks, 1);
    request.set("blockId", "123456789");
    request.set("limit", 36);
    
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"blockId\":\"123456789\""));
    assert!(json.contains("\"limit\":36"));
    assert!(json.contains("\"protocol\":1"));
}

#[test]
fn test_process_transactions_request() {
    let mut request = PeerRequest::new(RequestType::ProcessTransactions, 1);
    let transactions = vec![
        serde_json::json!({"type": 0, "senderId": "123"}),
        serde_json::json!({"type": 0, "senderId": "456"}),
    ];
    request.set("transactions", transactions);
    
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"transactions\""));
    assert!(json.contains("\"protocol\":1"));
}

#[test]
fn test_request_type_from_str() {
    assert_eq!(RequestType::from("getCumulativeDifficulty"), RequestType::GetCumulativeDifficulty);
    assert_eq!(RequestType::from("getMilestoneBlockIds"), RequestType::GetMilestoneBlockIds);
    assert_eq!(RequestType::from("getNextBlockIds"), RequestType::GetNextBlockIds);
    assert_eq!(RequestType::from("getNextBlocks"), RequestType::GetNextBlocks);
    assert_eq!(RequestType::from("getInfo"), RequestType::GetInfo);
    assert_eq!(RequestType::from("getPeers"), RequestType::GetPeers);
    assert_eq!(RequestType::from("processBlock"), RequestType::ProcessBlock);
    assert_eq!(RequestType::from("processTransactions"), RequestType::ProcessTransactions);
}

#[test]
fn test_protocol_version_2() {
    let request = PeerRequest::new(RequestType::GetInfo, 2);
    assert_eq!(request.protocol, 2);
    
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"protocol\":2"));
}

#[test]
fn test_deserialize_java_request() {
    let json = r#"{"requestType":"getCumulativeDifficulty","protocol":1}"#;
    let request: PeerRequest = serde_json::from_str(json).unwrap();
    
    assert_eq!(request.request_type, RequestType::GetCumulativeDifficulty);
    assert_eq!(request.protocol, 1);
}

#[test]
fn test_deserialize_java_request_with_extra_fields() {
    let json = r#"{"requestType":"getMilestoneBlockIds","protocol":1,"lastBlockId":"12345","lastMilestoneBlockId":"67890"}"#;
    let request: PeerRequest = serde_json::from_str(json).unwrap();
    
    assert_eq!(request.request_type, RequestType::GetMilestoneBlockIds);
    assert_eq!(request.protocol, 1);
    
    let last_block_id: String = request.get("lastBlockId").unwrap();
    assert_eq!(last_block_id, "12345");
    
    let last_milestone: String = request.get("lastMilestoneBlockId").unwrap();
    assert_eq!(last_milestone, "67890");
}

#[test]
fn test_java_nrcs_compatible_request_format() {
    let mut request = PeerRequest::new(RequestType::GetMilestoneBlockIds, 1);
    request.set("lastBlockId", "90399831811024079");
    
    let json = serde_json::to_string(&request).unwrap();
    
    assert!(json.starts_with("{"));
    assert!(json.ends_with("}"));
    assert!(json.contains("\"requestType\":\"getMilestoneBlockIds\""));
    assert!(json.contains("\"protocol\":1"));
    assert!(json.contains("\"lastBlockId\":\"90399831811024079\""));
}
