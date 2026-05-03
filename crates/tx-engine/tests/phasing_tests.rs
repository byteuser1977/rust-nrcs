//! Phasing 外部测试
//!
//! 测试 PhasingPoll、PhasingVote、PhasingPollResult、HashFunction 等阶段化交易功能

use tx_engine::phasing::{
    PhasingPoll, PhasingVote, PhasingPollResult, HashFunction,
};

#[test]
fn test_hash_function_from_code() {
    assert_eq!(HashFunction::from_code(0), Some(HashFunction::Sha256));
    assert_eq!(HashFunction::from_code(1), Some(HashFunction::Ripemd160));
    assert_eq!(HashFunction::from_code(2), Some(HashFunction::Ripemd160Sha256));
    assert_eq!(HashFunction::from_code(3), Some(HashFunction::Sm3));
    assert_eq!(HashFunction::from_code(99), None);
}

#[test]
fn test_hash_function_code() {
    assert_eq!(HashFunction::Sha256.code(), 0);
    assert_eq!(HashFunction::Ripemd160.code(), 1);
    assert_eq!(HashFunction::Ripemd160Sha256.code(), 2);
    assert_eq!(HashFunction::Sm3.code(), 3);
}

#[test]
fn test_hash_function_code_roundtrip() {
    for hf in [
        HashFunction::Sha256,
        HashFunction::Ripemd160,
        HashFunction::Ripemd160Sha256,
        HashFunction::Sm3,
    ] {
        assert_eq!(HashFunction::from_code(hf.code()), Some(hf));
    }
}

#[test]
fn test_hash_function_sha256() {
    let hash_func = HashFunction::Sha256;
    let data = b"test data";
    let hash = hash_func.hash(data);
    assert_eq!(hash.len(), 32);
}

#[test]
fn test_hash_function_sm3() {
    let hash_func = HashFunction::Sm3;
    let data = b"test data";
    let hash = hash_func.hash(data);
    assert!(!hash.is_empty());
}

#[test]
fn test_hash_function_sha256_deterministic() {
    let hash_func = HashFunction::Sha256;
    let data = b"deterministic test";
    let hash1 = hash_func.hash(data);
    let hash2 = hash_func.hash(data);
    assert_eq!(hash1, hash2);
}

#[test]
fn test_phasing_poll_new() {
    let poll = PhasingPoll::new(1, 100, 1000, 0);
    assert_eq!(poll.id, 1);
    assert_eq!(poll.account_id, 100);
    assert_eq!(poll.finish_height, 1000);
    assert_eq!(poll.voting_model, 0);
    assert!(poll.holding_id.is_none());
    assert!(poll.quorum.is_none());
    assert!(poll.min_balance.is_none());
    assert!(poll.whitelist.is_empty());
    assert!(poll.hashed_secret.is_none());
}

#[test]
fn test_phasing_poll_is_finished() {
    let poll = PhasingPoll::new(1, 100, 1000, 0);
    assert!(!poll.is_finished(500), "Before finish height");
    assert!(poll.is_finished(1000), "At finish height");
    assert!(poll.is_finished(1500), "After finish height");
}

#[test]
fn test_phasing_poll_verify_secret() {
    let secret = b"my secret";
    let hash = HashFunction::Sha256.hash(secret);

    let mut poll = PhasingPoll::new(1, 100, 1000, 0);
    poll.hashed_secret = Some(hash);
    poll.algorithm = HashFunction::Sha256.code();

    assert!(poll.verify_secret(secret), "Correct secret should verify");
    assert!(!poll.verify_secret(b"wrong secret"), "Wrong secret should not verify");
}

#[test]
fn test_phasing_poll_verify_secret_no_hashed_secret() {
    let poll = PhasingPoll::new(1, 100, 1000, 0);
    assert!(!poll.verify_secret(b"any secret"), "No hashed_secret should return false");
}

#[test]
fn test_phasing_poll_verify_secret_invalid_algorithm() {
    let secret = b"my secret";
    let hash = HashFunction::Sha256.hash(secret);

    let mut poll = PhasingPoll::new(1, 100, 1000, 0);
    poll.hashed_secret = Some(hash);
    poll.algorithm = 99;

    assert!(!poll.verify_secret(secret), "Invalid algorithm should return false");
}

#[test]
fn test_phasing_vote_new() {
    let vote = PhasingVote::new(12345, 100, 500);
    assert_eq!(vote.transaction_id, 12345);
    assert_eq!(vote.voter_id, 100);
    assert_eq!(vote.height, 500);
    assert_eq!(vote.db_id, 0);
}

#[test]
fn test_phasing_poll_result_new() {
    let result = PhasingPollResult::new(1, 100, true, 500);
    assert_eq!(result.id, 1);
    assert_eq!(result.result, 100);
    assert!(result.approved);
    assert_eq!(result.height, 500);
}

#[test]
fn test_phasing_poll_result_not_approved() {
    let result = PhasingPollResult::new(2, 0, false, 600);
    assert!(!result.approved);
    assert_eq!(result.result, 0);
}

#[test]
fn test_phasing_poll_with_whitelist() {
    let mut poll = PhasingPoll::new(1, 100, 1000, 1);
    poll.whitelist = vec![100, 200, 300];
    assert_eq!(poll.whitelist.len(), 3);
}

#[test]
fn test_phasing_poll_with_quorum() {
    let mut poll = PhasingPoll::new(1, 100, 1000, 1);
    poll.quorum = Some(3);
    assert_eq!(poll.quorum, Some(3));
}

#[test]
fn test_phasing_poll_with_min_balance() {
    let mut poll = PhasingPoll::new(1, 100, 1000, 2);
    poll.min_balance = Some(10000);
    poll.holding_id = Some(555);
    assert_eq!(poll.min_balance, Some(10000));
    assert_eq!(poll.holding_id, Some(555));
}
