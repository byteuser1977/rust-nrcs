//! 哈希算法实现
//!
//! 为 SHA-256 和 SM3 提供 `HashAlgorithm` trait 实现。

use super::algorithms::HashAlgorithm;
use crate::Hash256;

/// SHA-256 算法
#[derive(Debug, Clone, Copy)]
pub struct Sha256;

impl HashAlgorithm for Sha256 {
    fn hash(&self, data: &[u8]) -> Hash256 {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        result.into()
    }
}

impl Sha256 {
    pub fn name() -> &'static str {
        "sha256"
    }
}

/// SM3 国密算法
#[derive(Debug, Clone, Copy)]
pub struct Sm3;

impl HashAlgorithm for Sm3 {
    fn hash(&self, data: &[u8]) -> Hash256 {
        use sm3::Sm3Hash;
        let mut hasher = Sm3Hash::new();
        hasher.update(data);
        hasher.finalize().into()
    }
}

impl Sm3 {
    pub fn name() -> &'static str {
        "sm3"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hash() {
        let algo = Sha256;
        let data = b"hello world";
        let hash = algo.hash(data);
        assert_eq!(hash.len(), 32);
        // Verify against known value
        let mut expected = [0u8; 32];
        expected.copy_from_slice(&[
            0xb9, 0x4d, 0x27, 0xb9, 0x93, 0x4d, 0x3e, 0x71,
            0x38, 0x71, 0x7d, 0x90, 0xc7, 0x5c, 0xbb, 0x21,
            0x93, 0x44, 0x44, 0x9a, 0xe3, 0x2a, 0x66, 0x6b,
            0xf3, 0xaf, 0x03, 0x9a, 0x9e, 0x87, 0x91, 0x7c,
        ]);
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sm3_hash() {
        let algo = Sm3;
        let data = b"abc";
        let hash = algo.hash(data);
        let mut expected = [0u8; 32];
        expected.copy_from_slice(&[
            0x66, 0xc7, 0xf0, 0xf4, 0x62, 0xee, 0xed, 0xd9,
            0xd1, 0xf2, 0xd4, 0x6b, 0xdc, 0x10, 0xe4, 0xe2,
            0x41, 0x67, 0xc4, 0x87, 0x13, 0x8c, 0x77, 0x52,
            0x90, 0xc5, 0x32, 0x2f, 0x89, 0xa1, 0x14, 0xcd,
        ]);
        assert_eq!(hash, expected);
    }
}
