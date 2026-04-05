//! Symmetric cipher algorithms: AES and SM4 with CBC/GCM modes

mod aes;
mod sm4;

pub use aes::{AesCbc, AesGcm};
pub use sm4::{Sm4Cbc, Sm4Gcm};

/// Encrypted ciphertext container
///
/// For GCM modes, includes authentication tag
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CipherText {
    /// The actual ciphertext bytes
    pub data: Vec<u8>,
    /// Authentication tag (for AEAD modes like GCM)
    pub tag: Option<[u8; 16]>,
}

impl CipherText {
    /// Create a new ciphertext without authentication tag (CBC mode)
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, tag: None }
    }

    /// Create a new ciphertext with authentication tag (GCM mode)
    pub fn with_tag(data: Vec<u8>, tag: [u8; 16]) -> Self {
        Self { data, tag: Some(tag) }
    }
}