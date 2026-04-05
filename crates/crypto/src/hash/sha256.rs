use super::super::traits::{HashAlgorithm, Hash256};

/// SHA-256 hash algorithm implementation
#[derive(Debug, Clone, Copy)]
pub struct Sha256;

impl HashAlgorithm for Sha256 {
    fn hash(&self, data: &[u8]) -> Hash256 {
        use sha2::{Digest, Sha256 as Sha256Hasher};
        let mut hasher = Sha256Hasher::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_sha256_empty() {
        let hasher = Sha256;
        let hash = hasher.hash(b"");
        // SHA-256 of empty string
        let expected = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sha256_hello() {
        let hasher = Sha256;
        let hash = hasher.hash(b"hello world");
        // SHA-256 of "hello world"
        let expected = hex!("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sha256_known_vector() {
        let hasher = Sha256;
        // FIPS PUB 180-4 test vector
        let data = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq".as_bytes();
        let hash = hasher.hash(data);
        let expected = hex!("248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");
        assert_eq!(hash, expected);
    }
}