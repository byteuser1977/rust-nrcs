//! Cryptographic utilities for account management (wrapper for crypto crate)
//!
//! - Keypair generation (delegates to crypto crate)
//! - Account ID derivation (delegates to crypto crate)
//! - Address encoding (Base58 with checksum)

use crypto::*;
use base58;

use blockchain_types::*;

/// Generate a new Ed25519 keypair (delegates to crypto crate)
pub fn generate_keypair() -> KeyPair {
    crypto::generate_keypair()
}

/// Derive account ID from public key (delegates to crypto crate)
/// Account ID = first 8 bytes of SHA-256(public_key) as u64 (big-endian)
pub fn derive_account_id(public_key: &VerifyingKey) -> AccountId {
    let public_key_bytes = public_key.as_bytes();
    let hash = crypto::sha256(public_key_bytes);
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&hash[0..8]);
    u64::from_be_bytes(bytes)
}

/// Derive account address (Base58Check encoding)
/// Format: [version (1 byte)] + [account_id (8 bytes)] + [checksum (4 bytes)]
pub fn derive_address(account_id: AccountId) -> String {
    let version = 0x00u8;
    let mut data = Vec::with_capacity(13);
    data.push(version);
    data.extend_from_slice(&account_id.to_be_bytes());

    let checksum = double_sha256(&data);
    data.extend_from_slice(&checksum[0..4]);

    base58::encode(&data)
}

/// Verify that an address matches an account ID
pub fn verify_address(account_id: AccountId, address: &str) -> bool {
    let expected = derive_address(account_id);
    expected == address
}

/// Double SHA-256
fn double_sha256(data: &[u8]) -> [u8; 32] {
    let first = crypto::sha256(data);
    crypto::sha256(&first)
}

/// Address generator trait (wrapper for crypto crate)
pub trait AddressGenerator {
    fn generate_account(&self) -> (KeyPair, AccountId, String);
    fn verify_address(&self, account_id: AccountId, address: &str) -> bool;
}

impl AddressGenerator for () {
    fn generate_account(&self) -> (KeyPair, AccountId, String) {
        let kp = generate_keypair();
        let public_key = kp.verifying_key();
        let account_id = derive_account_id(&public_key);
        let address = derive_address(account_id);
        (kp, account_id, address)
    }

    fn verify_address(&self, account_id: AccountId, address: &str) -> bool {
        verify_address(account_id, address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let kp = generate_keypair();
        assert_eq!(kp.verifying_key().as_bytes().len(), 32);
    }

    #[test]
    fn test_account_id_derivation() {
        let kp = generate_keypair();
        let account_id = derive_account_id(&kp.verifying_key());
        assert_ne!(account_id, 0);
    }

    #[test]
    fn test_address_derivation() {
        let kp = generate_keypair();
        let account_id = derive_account_id(&kp.verifying_key());
        let address = derive_address(account_id);
        assert!(address.len() > 20);
        assert!(verify_address(account_id, &address));
    }
}
