//! SM3 哈希算法测试
//!
//! 验证 SM3 国密哈希算法的正确性，使用标准测试向量。

use crypto::algorithms::{Sm3, HashAlgorithm};

#[test]
fn test_sm3_empty_string() {
    let algo = Sm3;
    let hash = algo.hash(b"");
    assert_eq!(hash.len(), 32);
    let expected = hex::decode("1ab21d8355cfa17f8e61194831e81a8f22bec8c728fefb747ed035eb5082aa2b").unwrap();
    assert_eq!(hash, expected.as_slice());
}

#[test]
fn test_sm3_abc() {
    let algo = Sm3;
    let hash = algo.hash(b"abc");
    let expected = hex::decode("66c7f0f462eeedd9d1f2d46bdc10e4e24167c4875cf2f7a2297da02b8f4ba8e0").unwrap();
    assert_eq!(hash, expected.as_slice());
}

#[test]
fn test_sm3_abcdefgh() {
    let algo = Sm3;
    let data = b"abcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcd";
    let hash = algo.hash(data);
    assert_eq!(hash.len(), 32);
}

#[test]
fn test_sm3_deterministic() {
    let algo = Sm3;
    let data = b"test data for determinism";
    let hash1 = algo.hash(data);
    let hash2 = algo.hash(data);
    assert_eq!(hash1, hash2);
}

#[test]
fn test_sm3_different_inputs() {
    let algo = Sm3;
    let hash1 = algo.hash(b"input1");
    let hash2 = algo.hash(b"input2");
    assert_ne!(hash1, hash2);
}

#[test]
fn test_sm3_large_input() {
    let algo = Sm3;
    let data = vec![0x42u8; 100000];
    let hash = algo.hash(&data);
    assert_eq!(hash.len(), 32);
}

#[test]
fn test_sm3_algorithm_name() {
    assert_eq!(Sm3.name(), "sm3");
}
