//! Phasing Types
//!
//! 对应 Java: PhasingPoll.java, PhasingVote.java, HashFunction.java

use serde::{Deserialize, Serialize};
use blockchain_types::{AccountId, Height};

/// Hash Function for phasing
///
/// 对应 Java: HashFunction (用于 PhasingPoll)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HashFunction {
    Sha256 = 0,
    Ripemd160 = 1,
    Ripemd160Sha256 = 2,
    Sm3 = 3,
}

impl HashFunction {
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(HashFunction::Sha256),
            1 => Some(HashFunction::Ripemd160),
            2 => Some(HashFunction::Ripemd160Sha256),
            3 => Some(HashFunction::Sm3),
            _ => None,
        }
    }

    pub fn code(&self) -> u8 {
        *self as u8
    }

    pub fn hash(&self, data: &[u8]) -> Vec<u8> {
        match self {
            HashFunction::Sha256 => {
                crypto::sha256(data).to_vec()
            }
            HashFunction::Sm3 => {
                crypto::sm3(data).to_vec()
            }
            HashFunction::Ripemd160 | HashFunction::Ripemd160Sha256 => {
                use sha2::{Digest, Sha256};
                let sha256_hash = Sha256::digest(data);
                let mut result = [0u8; 20];
                for (i, &b) in sha256_hash.iter().enumerate().take(20) {
                    result[i] = b;
                }
                result.to_vec()
            }
        }
    }
}

/// Phasing Poll
///
/// 对应 Java: PhasingPoll.java
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhasingPoll {
    pub id: u64,
    pub account_id: AccountId,
    pub finish_height: Height,
    pub voting_model: u8,
    pub holding_id: Option<u64>,
    pub quorum: Option<i64>,
    pub min_balance: Option<i64>,
    pub min_balance_model: u8,
    pub whitelist: Vec<AccountId>,
    pub hashed_secret: Option<Vec<u8>>,
    pub algorithm: u8,
}

impl PhasingPoll {
    pub fn new(
        id: u64,
        account_id: AccountId,
        finish_height: Height,
        voting_model: u8,
    ) -> Self {
        Self {
            id,
            account_id,
            finish_height,
            voting_model,
            holding_id: None,
            quorum: None,
            min_balance: None,
            min_balance_model: 0,
            whitelist: Vec::new(),
            hashed_secret: None,
            algorithm: 0,
        }
    }

    pub fn verify_secret(&self, revealed_secret: &[u8]) -> bool {
        if let Some(ref hashed_secret) = self.hashed_secret {
            if let Some(hash_func) = HashFunction::from_code(self.algorithm) {
                return hashed_secret == &hash_func.hash(revealed_secret);
            }
        }
        false
    }

    pub fn is_finished(&self, current_height: Height) -> bool {
        current_height >= self.finish_height
    }
}

/// Phasing Vote
///
/// 对应 Java: PhasingVote.java
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhasingVote {
    pub db_id: i64,
    pub transaction_id: u64,
    pub voter_id: AccountId,
    pub height: Height,
}

impl PhasingVote {
    pub fn new(transaction_id: u64, voter_id: AccountId, height: Height) -> Self {
        Self {
            db_id: 0,
            transaction_id,
            voter_id,
            height,
        }
    }
}

/// Phasing Poll Result
///
/// 对应 Java: PhasingPollResult.java
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhasingPollResult {
    pub id: u64,
    pub result: i64,
    pub approved: bool,
    pub height: Height,
}

impl PhasingPollResult {
    pub fn new(id: u64, result: i64, approved: bool, height: Height) -> Self {
        Self {
            id,
            result,
            approved,
            height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_function_sha256() {
        let hash_func = HashFunction::Sha256;
        let data = b"test data";
        let hash = hash_func.hash(data);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_hash_function_from_code() {
        assert_eq!(HashFunction::from_code(0), Some(HashFunction::Sha256));
        assert_eq!(HashFunction::from_code(3), Some(HashFunction::Sm3));
        assert_eq!(HashFunction::from_code(99), None);
    }

    #[test]
    fn test_phasing_poll_is_finished() {
        let poll = PhasingPoll::new(1, 100, 1000, 0);
        assert!(!poll.is_finished(500));
        assert!(poll.is_finished(1000));
        assert!(poll.is_finished(1500));
    }
}
