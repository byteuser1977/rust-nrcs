use crypto::*;
use sha2::{Sha256, Sha512};
use blake3::Hasher as Blake3Hasher;

#[test]
fn test_sha256_hash() {
    let data = b"Hello, NRCS!";
    let hash = sha256_hash(data);

    assert_eq!(hash.len(), 32);
    // Known hash
    let expected = Sha256::digest(data);
    assert_eq!(hash, expected.as_slice());
}

#[test]
fn test_sha256_hash_empty() {
    let data = b"";
    let hash = sha256_hash(data);

    let expected = Sha256::digest(data);
    assert_eq!(hash, expected.as_slice());
}

#[test]
fn test_sha256_hash_large() {
    let data = vec![0xAA; 10000];
    let hash = sha256_hash(&data);

    assert_eq!(hash.len(), 32);
    let expected = Sha256::digest(&data);
    assert_eq!(hash, expected.as_slice());
}

#[test]
fn test_sha512_hash() {
    let data = b"Hello, NRCS!";
    let hash = sha512_hash(data);

    assert_eq!(hash.len(), 64);
    let expected = Sha512::digest(data);
    assert_eq!(hash, expected.as_slice());
}

#[test]
fn test_blake3_hash() {
    let data = b"Hello, NRCS!";
    let hash = blake3_hash(data);

    assert_eq!(hash.len(), 32);
    let expected = Blake3Hasher::new().update(data).finalize();
    assert_eq!(hash, expected.as_bytes());
}

#[test]
fn test_double_sha256() {
    let data = b"test data";
    let hash = double_sha256(data);

    assert_eq!(hash.len(), 32);

    // Manual double SHA256
    let first = Sha256::digest(data);
    let second = Sha256::digest(&first);
    assert_eq!(hash, second.as_slice());
}

#[test]
fn test_hash_consistency() {
    let data = b"consistent hash test";

    let sha256_1 = sha256_hash(data);
    let sha256_2 = sha256_hash(data);
    assert_eq!(sha256_1, sha256_2);

    let sha512_1 = sha512_hash(data);
    let sha512_2 = sha512_hash(data);
    assert_eq!(sha512_1, sha512_2);

    let blake3_1 = blake3_hash(data);
    let blake3_2 = blake3_hash(data);
    assert_eq!(blake3_1, blake3_2);
}

#[test]
fn test_hash_different_data() {
    let data1 = b"data1";
    let data2 = b"data2";

    let hash1_sha256 = sha256_hash(data1);
    let hash2_sha256 = sha256_hash(data2);
    assert_ne!(hash1_sha256, hash2_sha256);

    let hash1_blake3 = blake3_hash(data1);
    let hash2_blake3 = blake3_hash(data2);
    assert_ne!(hash1_blake3, hash2_blake3);
}

#[test]
fn test_ripemd160_hash() {
    // If ripemd160 is implemented
    #[cfg(any(feature = "ripemd160", feature = "all-hashes"))]
    {
        use crypto::ripemd160_hash;
        let data = b"Hello, NRCS!";
        let hash = ripemd160_hash(data);

        assert_eq!(hash.len(), 20); // RIPEMD-160 produces 20 bytes
    }
}
