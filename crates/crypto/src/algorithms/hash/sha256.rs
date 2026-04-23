//! SHA-256 哈希算法实现

use crate::algorithms::HashAlgorithm;
use crate::Hash256;

/// SHA-256 算法
#[derive(Debug, Clone, Copy)]
pub struct Sha256;

impl HashAlgorithm for Sha256 {
    fn hash(&self, data: &[u8]) -> Hash256 {
        use sha2::{Digest, Sha256 as Sha256Impl};
        let mut hasher = Sha256Impl::new();
        hasher.update(data);
        let result = hasher.finalize();
        result.into()
    }

    fn name(&self) -> &'static str {
        "sha256"
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
        let expected = [
            0xb9, 0x4d, 0x27, 0xb9, 0x93, 0x4d, 0x3e, 0x08,
            0xa5, 0x2e, 0x52, 0xd7, 0xda, 0x7d, 0xab, 0xfa,
            0xc4, 0x84, 0xef, 0xe3, 0x7a, 0x53, 0x80, 0xee,
            0x90, 0x88, 0xf7, 0xac, 0xe2, 0xef, 0xcd, 0xe9,
        ];
        assert_eq!(hash, expected);
    }
}
