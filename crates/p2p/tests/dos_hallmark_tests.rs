//! P2P DoS 过滤器和 Hallmark 外部测试
//!
//! 测试 DosFilterConfig、HallmarkParser 等功能

use p2p::dos_filter::DosFilterConfig;
use p2p::hallmark::{HallmarkParser, HallmarkInfo};

#[test]
fn test_dos_filter_config_default() {
    let config = DosFilterConfig::default();
    assert_eq!(config.max_requests_per_sec, 30);
    assert_eq!(config.delay_ms, 1000);
    assert_eq!(config.max_request_ms, 300000);
    assert_eq!(config.throttle_ms, 1000);
    assert_eq!(config.max_throttle_ms, 30000);
    assert_eq!(config.max_wait_ms, 50000);
    assert_eq!(config.max_idle_tracker_ms, 30000);
    assert!(!config.track_sessions);
    assert!(config.whitelist.is_empty());
    assert!(config.blacklist.is_empty());
}

#[test]
fn test_dos_filter_config_custom() {
    let config = DosFilterConfig {
        max_requests_per_sec: 100,
        delay_ms: 500,
        max_request_ms: 60000,
        throttle_ms: 2000,
        max_throttle_ms: 60000,
        max_wait_ms: 10000,
        max_idle_tracker_ms: 60000,
        track_sessions: true,
        remote_port: 8080,
        insert_port: 8080,
        whitelist: vec!["127.0.0.1".to_string()],
        blacklist: vec!["10.0.0.1".to_string()],
    };

    assert_eq!(config.max_requests_per_sec, 100);
    assert_eq!(config.whitelist.len(), 1);
    assert_eq!(config.blacklist.len(), 1);
}

#[test]
fn test_hallmark_parse_empty() {
    let info = HallmarkParser::parse("");
    assert!(!info.is_valid);
    assert_eq!(info.weight, 0);
    assert!(info.public_key.is_none());
}

#[test]
fn test_hallmark_parse_whitespace() {
    let info = HallmarkParser::parse("   ");
    assert!(!info.is_valid);
}

#[test]
fn test_hallmark_parse_invalid_base64() {
    let info = HallmarkParser::parse("not-valid-base64!!!");
    assert!(!info.is_valid);
}

#[test]
fn test_hallmark_default_info() {
    let info = HallmarkInfo::default();
    assert!(!info.is_valid);
    assert_eq!(info.weight, 0);
    assert!(info.public_key.is_none());
    assert!(info.raw.is_empty());
}

#[test]
fn test_hallmark_parse_too_short_data() {
    let short_data = vec![0u8; 40];
    let encoded = base64::encode(&short_data);
    let info = HallmarkParser::parse(&encoded);
    assert!(!info.is_valid);
}

#[test]
fn test_hallmark_parse_exactly_104_bytes() {
    let data = vec![0u8; 104];
    let encoded = base64::encode(&data);
    let info = HallmarkParser::parse(&encoded);
    assert!(!info.is_valid, "All zeros should not produce valid signature");
    assert!(info.public_key.is_some(), "Should extract public key");
    assert_eq!(info.weight, 0);
    assert_eq!(info.date, 0);
}

#[test]
fn test_hallmark_is_within_validity_period_valid() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let mut info = HallmarkInfo::default();
    info.date = now;
    info.is_valid = true;

    assert!(HallmarkParser::is_within_validity_period(&info, 31536000));
}

#[test]
fn test_hallmark_is_within_validity_period_expired() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let mut info = HallmarkInfo::default();
    info.date = now - 40000000u32;
    info.is_valid = true;

    assert!(!HallmarkParser::is_within_validity_period(&info, 31536000));
}

#[test]
fn test_hallmark_is_within_validity_period_invalid() {
    let mut info = HallmarkInfo::default();
    info.date = 1000000;
    info.is_valid = false;

    assert!(!HallmarkParser::is_within_validity_period(&info, 31536000));
}

#[test]
fn test_hallmark_calculate_weight_with_valid_hallmark() {
    let mut hm = HallmarkInfo::default();
    hm.weight = 100;
    hm.is_valid = true;

    let weight = HallmarkParser::calculate_peer_weight(Some(&hm), 5 * 1024 * 1024, 10 * 1024 * 1024);
    assert_eq!(weight, 100);
}

#[test]
fn test_hallmark_calculate_weight_with_invalid_hallmark() {
    let mut hm = HallmarkInfo::default();
    hm.weight = 100;
    hm.is_valid = false;

    let weight = HallmarkParser::calculate_peer_weight(Some(&hm), 5 * 1024 * 1024, 10 * 1024 * 1024);
    assert_eq!(weight, 15, "Invalid hallmark should use traffic-based weight");
}

#[test]
fn test_hallmark_calculate_weight_without_hallmark() {
    let weight = HallmarkParser::calculate_peer_weight(None, 5 * 1024 * 1024, 10 * 1024 * 1024);
    assert_eq!(weight, 15);
}

#[test]
fn test_hallmark_calculate_weight_minimum() {
    let weight = HallmarkParser::calculate_peer_weight(None, 0, 0);
    assert_eq!(weight, 1);
}

#[test]
fn test_hallmark_calculate_weight_zero_weight_hallmark() {
    let mut hm = HallmarkInfo::default();
    hm.weight = 0;
    hm.is_valid = true;

    let weight = HallmarkParser::calculate_peer_weight(Some(&hm), 0, 0);
    assert_eq!(weight, 0, "Valid hallmark with weight=0 should return 0");
}

#[test]
fn test_hallmark_parse_with_weight_and_date() {
    let mut data = vec![0u8; 104];
    data[32..36].copy_from_slice(&100u32.to_le_bytes());
    data[36..40].copy_from_slice(&1609459200u32.to_le_bytes());

    let encoded = base64::encode(&data);
    let info = HallmarkParser::parse(&encoded);

    assert_eq!(info.weight, 100);
    assert_eq!(info.date, 1609459200);
    assert!(!info.is_valid);
}

#[test]
fn test_hallmark_parse_large_weight() {
    let mut data = vec![0u8; 104];
    data[32..36].copy_from_slice(&u32::MAX.to_le_bytes());

    let encoded = base64::encode(&data);
    let info = HallmarkParser::parse(&encoded);

    assert_eq!(info.weight, u32::MAX);
}

#[test]
fn test_hallmark_raw_preserved() {
    let input = base64::encode(&vec![0u8; 104]);
    let info = HallmarkParser::parse(&input);
    assert_eq!(info.raw, input);
}
