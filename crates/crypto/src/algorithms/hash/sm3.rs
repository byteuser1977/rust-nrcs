//! SM3 国密哈希算法实现

use crate::algorithms::HashAlgorithm;
use crate::Hash256;

/// SM3 国密算法
#[derive(Debug, Clone, Copy)]
pub struct Sm3;

impl HashAlgorithm for Sm3 {
    fn hash(&self, data: &[u8]) -> Hash256 {
        use sm3::{Digest, Sm3 as Sm3Impl};
        let mut hasher = Sm3Impl::new();
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
    
    #[test]
    fn test_sm3_known_vector() {
        let algo = Sm3;
        let data = b"abc";
        let hash = algo.hash(data);
        let expected = hex::decode("66c7f0f462eeedd9d1f2d46bdc10e4e24167c4875cf2f7a2297da02b8f4ba8e0").unwrap();
        assert_eq!(hash.as_slice(), expected.as_slice());
    }
}
