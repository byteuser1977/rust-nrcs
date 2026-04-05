//! Key types: PublicKey, SecretKey, Signature, KeyPair

use super::traits::{CryptoError, CryptoResult};
use serde::{Serialize, Deserialize};

/// Public key - wraps either Ed25519 or SM2 public key
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicKey {
    Ed25519(ed25519_dalek::PublicKey),
    Sm2(sm2::PublicKey),
}

impl PublicKey {
    /// Get the raw bytes of the public key
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Ed25519(pk) => pk.as_bytes(),
            Self::Sm2(pk) => pk.as_ref(),
        }
    }

    /// Get the byte length of the public key
    pub fn len(&self) -> usize {
        match self {
            Self::Ed25519(_) => 32,
            Self::Sm2(pk) => pk.as_ref().len(),
        }
    }

    /// Check if the public key is empty
    pub fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }

    /// Deserialize from bytes (auto-detects format based on length)
    ///
    /// Ed25519: exactly 32 bytes
    /// SM2: 33 or 65 bytes (compressed/uncompressed)
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        match bytes.len() {
            32 => {
                let pk = ed25519_dalek::PublicKey::from_bytes(bytes)
                    .map_err(|_| CryptoError::InvalidData("Invalid Ed25519 public key".into()))?;
                Ok(Self::Ed25519(pk))
            }
            33 | 65 => {
                let pk = sm2::PublicKey::from_bytes(bytes)
                    .map_err(|_| CryptoError::InvalidData("Invalid SM2 public key".into()))?;
                Ok(Self::Sm2(pk))
            }
            _ => Err(CryptoError::InvalidData(format!("Invalid public key length: {}", bytes.len()))),
        }
    }
}

/// Secret (private) key - wraps either Ed25519 or SM2 secret key
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretKey {
    Ed25519(ed25519_dalek::SecretKey),
    Sm2(sm2::SecretKey),
}

impl SecretKey {
    /// Get the raw bytes of the secret key
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Ed25519(sk) => sk.as_bytes(),
            Self::Sm2(sk) => sk.as_ref(),
        }
    }

    /// Get the byte length of the secret key
    pub fn len(&self) -> usize {
        match self {
            Self::Ed25519(_) => 64,
            Self::Sm2(_) => 32,
        }
    }

    /// Check if the secret key is empty
    pub fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }

    /// Deserialize from bytes (auto-detects format)
    ///
    /// Ed25519: exactly 64 bytes
    /// SM2: exactly 32 bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        match bytes.len() {
            64 => {
                let sk = ed25519_dalek::SecretKey::from_bytes(bytes)
                    .map_err(|_| CryptoError::InvalidData("Invalid Ed25519 secret key".into()))?;
                Ok(Self::Ed25519(sk))
            }
            32 => {
                let sk = sm2::SecretKey::from_bytes(bytes)
                    .map_err(|_| CryptoError::InvalidData("Invalid SM2 secret key".into()))?;
                Ok(Self::Sm2(sk))
            }
            _ => Err(CryptoError::InvalidData(format!("Invalid secret key length: {}", bytes.len()))),
        }
    }

    /// Get the corresponding public key
    pub fn to_public(&self) -> PublicKey {
        match self {
            Self::Ed25519(sk) => PublicKey::Ed25519(sk.public_key()),
            Self::Sm2(sk) => PublicKey::Sm2(sm2::PublicKey::from(sk)),
        }
    }
}

/// Signature - wraps either Ed25519 or SM2 signature
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signature {
    Ed25519(ed25519_dalek::Signature),
    Sm2(sm2::Signature),
}

impl Signature {
    /// Get the raw bytes of the signature
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Ed25519(sig) => sig.as_bytes(),
            Self::Sm2(sig) => sig.as_ref(),
        }
    }

    /// Get the byte length of the signature
    pub fn len(&self) -> usize {
        match self {
            Self::Ed25519(_) => 64,
            Self::Sm2(_) => 64,
        }
    }

    /// Deserialize from bytes
    ///
    /// Ed25519 signature: 64 bytes
    /// SM2 signature: 64 bytes (r and s concatenated)
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        match bytes.len() {
            64 => {
                // Try Ed25519 first (more common in default config)
                if let Ok(sig) = ed25519_dalek::Signature::from_bytes(bytes) {
                    return Ok(Self::Ed25519(sig));
                }
                // Fall back to SM2
                let sig = sm2::Signature::from_bytes(bytes)
                    .map_err(|_| CryptoError::InvalidData("Invalid signature".into()))?;
                Ok(Self::Sm2(sig))
            }
            _ => Err(CryptoError::InvalidData(format!("Invalid signature length: {}", bytes.len()))),
        }
    }
}

/// Key pair containing both public and secret key
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyPair {
    Ed25519(ed25519_dalek::Keypair),
    Sm2(sm2::KeyPair),
}

impl KeyPair {
    /// Generate a new random key pair using the default algorithm
    pub fn generate() -> CryptoResult<Self> {
        // Default to Ed25519 for now - this should be configured
        // The actual configuration will be handled by CRYPTO_CONFIG
        // This is just a fallback
        let kp = ed25519_dalek::Keypair::generate(&mut rand::thread_rng());
        Ok(Self::Ed25519(kp))
    }

    /// Get the public key
    pub fn public_key(&self) -> PublicKey {
        match self {
            Self::Ed25519(kp) => PublicKey::Ed25519(kp.public),
            Self::Sm2(kp) => PublicKey::Sm2(kp.public().clone()),
        }
    }

    /// Get the secret key
    pub fn secret_key(&self) -> SecretKey {
        match self {
            Self::Ed25519(kp) => SecretKey::Ed25519(kp.secret),
            Self::Sm2(kp) => SecretKey::Sm2(kp.private().clone()),
        }
    }

    /// Construct from public and secret keys (they must match)
    pub fn from_parts(public: PublicKey, secret: SecretKey) -> CryptoResult<Self> {
        match (public, secret) {
            (PublicKey::Ed25519(pk), SecretKey::Ed25519(sk)) => {
                // Verify they match
                if pk != sk.public_key() {
                    return Err(CryptoError::InvalidData("Mismatched Ed25519 key pair".into()));
                }
                Ok(Self::Ed25519(ed25519_dalek::Keypair { public: pk, secret: sk }))
            }
            (PublicKey::Sm2(pk), SecretKey::Sm2(sk)) => {
                let kp = sm2::KeyPair::from(sk);
                if kp.public() != &pk {
                    return Err(CryptoError::InvalidData("Mismatched SM2 key pair".into()));
                }
                Ok(Self::Sm2(kp))
            }
            _ => Err(CryptoError::InvalidData("Mixed key types".into())),
        }
    }
}

// Helper module for secure random number generation
mod rand {
    use super::*;

    pub fn thread_rng() -> impl rand::RngCore + rand::CryptoRng {
        // Using rand's thread_rng which is cryptographically secure
        // In production, consider using rand::rngs::OsRng for better security
        rand::thread_rng()
    }
}