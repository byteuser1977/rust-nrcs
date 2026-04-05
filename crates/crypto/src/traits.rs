//! Core cryptographic traits that enable algorithm swapping

use super::{Hash256, PublicKey, SecretKey, Signature};

/// Result type for all crypto operations
pub type CryptoResult<T> = Result<T, CryptoError>;

/// Errors that can occur during cryptographic operations
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Invalid key length: expected {expected}, got {got}")]
    InvalidKeyLength { expected: usize, got: usize },

    #[error("Invalid signature length: expected {expected}, got {got}")]
    InvalidSignatureLength { expected: usize, got: usize },

    #[error("Invalid IV length: expected {expected}, got {got}")]
    InvalidIvLength { expected: usize, got: usize },

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Verification failed")]
    VerificationFailed,

    #[error("Cipher not configured")]
    CipherNotConfigured,

    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Trait for hash algorithms that produce a 256-bit digest
pub trait HashAlgorithm: Send + Sync {
    /// Compute hash of `data`
    fn hash(&self, data: &[u8]) -> Hash256;
}

/// Trait for asymmetric signature algorithms
pub trait SignerAlgorithm: Send + Sync {
    /// Sign a message with the given secret key
    fn sign(&self, key: &SecretKey, message: &[u8]) -> Signature;

    /// Verify a signature against a public key
    fn verify(&self, key: &PublicKey, message: &[u8], signature: &Signature) -> CryptoResult<()>;

    /// Generate a new key pair
    fn generate_keypair(&self) -> CryptoResult<KeyPair>;
}

/// Trait for symmetric ciphers
pub trait CipherAlgorithm: Send + Sync {
    /// Encrypt `plaintext` with `key` and `iv`
    ///
    /// Returns ciphertext (and authentication tag for AEAD modes)
    fn encrypt(&self, key: &[u8], iv: &[u8], plaintext: &[u8]) -> CryptoResult<CipherText>;

    /// Decrypt `ciphertext` with `key` and `iv`
    ///
    /// For AEAD modes, must verify authentication tag
    fn decrypt(&self, key: &[u8], iv: &[u8], ciphertext: &CipherText) -> CryptoResult<Vec<u8>>;
}

// ============================================================================
// Common types re-exported for convenience
// ============================================================================

/// 256-bit hash output (32 bytes)
pub type Hash256 = [u8; 32];

/// Box<dyn SignerAlgorithm> for static dispatch
pub type DynSigner = Box<dyn SignerAlgorithm>;

/// Box<dyn HashAlgorithm> for static dispatch
pub type DynHasher = Box<dyn HashAlgorithm>;

/// Box<dyn CipherAlgorithm> for static dispatch
pub type DynCipher = Box<dyn CipherAlgorithm>;