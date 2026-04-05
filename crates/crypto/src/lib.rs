//! # ZeroClaw Crypto
//! Pluggable cryptography with support for both standard and Chinese national algorithms.
//!
//! ## Features
//! - Hash: SHA-256 (default) or SM3
//! - Signature: Ed25519 (default) or SM2
//! - Symmetric encryption: AES-CBC/GCM (default) or SM4-CBC/GCM
//!
//! Configuration via `config/default.toml`:
//! ```toml
//! [crypto]
//! hash = "sm3"
//! signature = "sm2"
//! cipher = "sm4"
//! cipher_mode = "gcm"
//! ```
//!
//! ## Usage
//! ```rust
//! use zeroclaw_crypto::{sha256, sign, verify, KeyPair};
//!
//! let data = b"hello world";
//! let hash = sha256(data);
//!
//! let kp = KeyPair::generate()?;
//! let signature = sign(kp.secret_key(), data);
//! verify(kp.public_key(), data, &signature)?;
//! ```

#![allow(unused_imports)]
#![allow(clippy::module_inception)]

mod traits;
mod hash;
mod signature;
mod cipher;
mod sm2;
mod sm4;
mod config;
mod keys;

pub use crate::{
    hash::Hash256,
    signature::{PublicKey, SecretKey, Signature},
    cipher::{CipherText, EncryptedMessage},
    traits::{
        HashAlgorithm, SignerAlgorithm, CipherAlgorithm,
        CryptoResult, CryptoError,
    },
    config::CryptoConfig,
    keys::KeyPair,
};

// Re-export commonly used types from underlying crates for convenience
pub use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SIGNATURE_LENGTH};

/// Global crypto configuration (initialized at startup)
use once_cell::sync::Lazy;
pub static CRYPTO_CONFIG: Lazy<CryptoConfig> = Lazy::new(|| {
    config::load_config().expect("Failed to load crypto configuration")
});

// ============================================================================
// Public API - Backward compatible functions
// ============================================================================

/// Hash data using the configured hash algorithm (SHA-256 or SM3)
///
/// Default: SHA-256
pub fn sha256(data: &[u8]) -> Hash256 {
    CRYPTO_CONFIG.hash.hash(data)
}

/// Hash data using SM3 (only available when configured as hash algorithm)
///
/// Note: If SM3 is not configured, this will still produce a hash but may be
/// inconsistent with expectations. Prefer `sha256()` which respects config.
pub fn sm3(data: &[u8]) -> Hash256 {
    CRYPTO_CONFIG.hash.hash(data)
}

/// Sign a message with the configured signature algorithm (Ed25519 or SM2)
///
/// Default: Ed25519
pub fn sign(secret_key: &SecretKey, message: &[u8]) -> Signature {
    CRYPTO_CONFIG.signer.sign(secret_key, message)
}

/// Verify a signature with the configured algorithm
///
/// Returns `Ok(())` if signature is valid, `Err(CryptoError)` otherwise.
pub fn verify(public_key: &PublicKey, message: &[u8], signature: &Signature) -> CryptoResult<()> {
    CRYPTO_CONFIG.signer.verify(public_key, message, signature)
}

/// Generate a new key pair using the configured signature algorithm
pub fn generate_keypair() -> CryptoResult<KeyPair> {
    CRYPTO_CONFIG.signer.generate_keypair()
}

// ============================================================================
// Symmetric encryption API (optional)
// ============================================================================

/// Encrypt data using the configured cipher (AES or SM4) and mode (CBC or GCM)
///
/// Returns encrypted bytes. For GCM mode, includes authentication tag.
/// Default: AES-GCM (if cipher feature is enabled)
pub fn encrypt(key: &[u8], iv: &[u8], plaintext: &[u8]) -> CryptoResult<CipherText> {
    match &CRYPTO_CONFIG.cipher {
        Some(cipher) => cipher.encrypt(key, iv, plaintext),
        None => Err(CryptoError::CipherNotConfigured),
    }
}

/// Decrypt data using the configured cipher
///
/// For GCM mode, also verifies authentication tag.
pub fn decrypt(key: &[u8], iv: &[u8], ciphertext: &CipherText) -> CryptoResult<Vec<u8>> {
    match &CRYPTO_CONFIG.cipher {
        Some(cipher) => cipher.decrypt(key, iv, ciphertext),
        None => Err(CryptoError::CipherNotConfigured),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_configurable() {
        let data = b"test data";
        let hash = sha256(data);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_keypair_generation() -> CryptoResult<()> {
        let kp = generate_keypair()?;
        assert!(kp.public_key().as_bytes().len() > 0);
        assert!(kp.secret_key().as_bytes().len() > 0);
        Ok(())
    }

    #[test]
    fn test_sign_verify_roundtrip() -> CryptoResult<()> {
        let kp = generate_keypair()?;
        let message = b"hello world";
        let signature = sign(kp.secret_key(), message);
        verify(kp.public_key(), message, &signature)?;
        Ok(())
    }
}