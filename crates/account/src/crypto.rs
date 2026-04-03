//! Cryptographic utilities for account management
//!
//! - Keypair generation (Ed25519)
//! - Account ID derivation from public key (SHA-256)
//! - Address encoding (Base58 with checksum)

use ed25519_dalek::{Keypair, SecretKey, PublicKey};
use rand::{rngs::StdRng, SeedableRng};
use sha2::{Digest, Sha256};
use std::convert::TryInto;

use blockchain_types::*;

/// Generate a new Ed25519 keypair
pub fn generate_keypair() -> Keypair {
    let mut rng = StdRng::from_entropy();
    Keypair::generate(&mut rng)
}

/// Derive account ID from public key
/// Account ID = first 8 bytes of SHA-256(public_key) as u64 (big-endian)
pub fn derive_account_id(public_key: &PublicKey) -> AccountId {
    let hash = Sha256::digest(public_key.as_bytes());
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&hash[0..8]);
    u64::from_be_bytes(bytes)
}

/// Derive account address (Base58Check encoding)
/// Format: [version (1 byte)] + [account_id (8 bytes)] + [checksum (4 bytes)]
pub fn derive_address(account_id: AccountId) -> String {
    let version = 0x00u8; // 版本字节
    let mut data = Vec::with_capacity(13);
    data.push(version);
    data.extend_from_slice(&account_id.to_be_bytes());

    // 计算 checksum (double SHA-256)
    let checksum = double_sha256(&data);
    data.extend_from_slice(&checksum[0..4]);

    // Base58 编码
    base58::encode(&data)
}

/// Double SHA-256
fn double_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let first = hasher.finalize();
    let mut hasher = Sha256::new();
    hasher.update(first);
    hasher.finalize().into()
}

/// Address generator trait
pub trait AddressGenerator {
    /// Generate a new account (keypair + address)
    fn generate_account(&self) -> (Keypair, AccountId, String);

    /// Verify that an address matches an account ID
    fn verify_address(&self, account_id: AccountId, address: &str) -> bool;
}

impl AddressGenerator for () {
    fn generate_account(&self) -> (Keypair, AccountId, String) {
        let kp = generate_keypair();
        let account_id = derive_account_id(&kp.public);
        let address = derive_address(account_id);
        (kp, account_id, address)
    }

    fn verify_address(&self, account_id: AccountId, address: &str) -> bool {
        let expected = derive_address(account_id);
        expected == address
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let kp = generate_keypair();
        assert_eq!(kp.public.as_bytes().len(), 32);
        assert_eq!(kp.secret.as_bytes().len(), 64);
    }

    #[test]
    fn test_account_id_derivation() {
        let kp = generate_keypair();
        let account_id = derive_account_id(&kp.public);
        assert_ne!(account_id, 0);
    }

    #[test]
    fn test_address_derivation() {
        let kp = generate_keypair();
        let account_id = derive_account_id(&kp.public);
        let address = derive_address(account_id);
        assert!(address.len() > 20); // Base58 地址有一定长度
        assert!(verify_address(account_id, &address));
    }
}