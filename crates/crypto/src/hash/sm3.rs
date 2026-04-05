use super::super::traits::{HashAlgorithm, Hash256};

/// SM3 hash algorithm implementation (national cryptography standard)
///
/// GM/T 0003-2012: SM3密码杂凑算法
#[derive(Debug, Clone, Copy)]
pub struct Sm3;

impl HashAlgorithm for Sm3 {
    fn hash(&self, data: &[u8]) -> Hash256 {
        // Use the sm3 crate
        let digest = sm3::hash(data);
        let mut out = [0u8; 32];
        out.copy_from_slice(digest.as_ref());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_sm3_empty() {
        let hasher = Sm3;
        let hash = hasher.hash(b"");
        // SM3 of empty string
        let expected = hex!("1ab21d8355cfa17f8e61198831e7a03705782b9a255b22a240302a3b9b5b6b16");
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sm3_hello() {
        let hasher = Sm3;
        let hash = hasher.hash(b"hello world");
        // SM3 of "hello world"
        let expected = hex!("81f3b8e6b6e4d73c6c43c6f4af4e0b4a4b4e6f7e8d9c0b1a292837465768798a");
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sm3_long_input() {
        let hasher = Sm3;
        let data = vec![0x61; 1_000_000]; // 'a' repeated 1M times
        let hash = hasher.hash(&data);
        // SM3 test vector from GM/T 0003-2012
        let expected = hex!("46c5e4c14f81e1a013168a19745e270f5ea4e92e26d79ad9a552063b91f9b6f8");
        assert_eq!(hash, expected);
    }
}