//! Reed-Solomon 编码测试
//!
//! 验证 Account ID 到 RS 地址的编码正确性。

use crypto::reed_solomon;

#[test]
fn test_encode_known_account_id() {
    let account_id: u64 = 1234567890;
    let rs_address = reed_solomon::encode(account_id);
    assert!(!rs_address.is_empty(), "RS address should not be empty");
    assert!(rs_address.contains('-'), "RS address should contain dashes");
}

#[test]
fn test_encode_zero() {
    let rs_address = reed_solomon::encode(0);
    assert!(!rs_address.is_empty());
}

#[test]
fn test_encode_max_u64() {
    let rs_address = reed_solomon::encode(u64::MAX);
    assert!(!rs_address.is_empty());
}

#[test]
fn test_encode_deterministic() {
    let id: u64 = 9876543210;
    let addr1 = reed_solomon::encode(id);
    let addr2 = reed_solomon::encode(id);
    assert_eq!(addr1, addr2, "Same ID should produce same RS address");
}

#[test]
fn test_encode_different_ids() {
    let addr1 = reed_solomon::encode(1);
    let addr2 = reed_solomon::encode(2);
    assert_ne!(addr1, addr2, "Different IDs should produce different RS addresses");
}

#[test]
fn test_encode_format() {
    let id: u64 = 17329363171487254902;
    let rs = reed_solomon::encode(id);
    let parts: Vec<&str> = rs.split('-').collect();
    assert!(parts.len() >= 4, "RS address should have at least 4 dash-separated groups, got: {}", rs);
}

#[test]
fn test_encode_small_values() {
    for id in [1u64, 10, 100, 1000, 10000].iter() {
        let rs = reed_solomon::encode(*id);
        assert!(!rs.is_empty(), "ID {} should produce non-empty RS address", id);
    }
}

#[test]
fn test_encode_alphabet() {
    let rs = reed_solomon::encode(12345);
    for c in rs.chars() {
        if c != '-' {
            assert!(
                "23456789ABCDEFGHJKLMNPQRSTUVWXYZ".contains(c),
                "Character '{}' should be in RS alphabet",
                c
            );
        }
    }
}
