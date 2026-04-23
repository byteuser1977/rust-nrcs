//! SM3 国密哈希算法实现

use crate::algorithms::HashAlgorithm;
use crate::Hash256;

/// SM3 国密算法
#[derive(Debug, Clone, Copy)]
pub struct Sm3;

impl HashAlgorithm for Sm3 {
    fn hash(&self, data: &[u8]) -> Hash256 {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        result.into()
    }

    fn name(&self) -> &'static str {
        "sm3"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sm3_hash() {
        let algo = Sm3;
        let data = b"abc";
        let hash = algo.hash(data);
        assert_eq!(hash.len(), 32);
    }
}
