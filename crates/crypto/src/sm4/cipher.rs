use super::traits::{CryptoResult, CryptoError};
use sm4::Sm4 as Sm4Core;

/// SM4 block cipher core (128-bit block, 128-bit key)
///
/// This is the raw block cipher - use Sm4Cbc or Sm4Gcm for authenticated/encryption modes
#[derive(Debug, Clone, Copy)]
pub struct Sm4 {
    key: [u8; 16],
    // Could store expanded round keys for performance
}

impl Sm4 {
    /// Create a new SM4 instance with the given 16-byte key
    pub fn new(key: &[u8]) -> CryptoResult<Self> {
        if key.len() != 16 {
            return Err(super::traits::CryptoError::InvalidKeyLength { expected: 16, got: key.len() });
        }
        let mut key_array = [0u8; 16];
        key_array.copy_from_slice(key);
        Ok(Self { key: key_array })
    }

    /// Encrypt a single 16-byte block
    pub fn encrypt_block(&self, block: &[u8; 16]) -> [u8; 16] {
        let sm4 = Sm4Core::new(&self.key);
        let mut out = [0u8; 16];
        out.copy_from_slice(&sm4.encrypt(block));
        out
    }

    /// Decrypt a single 16-byte block
    pub fn decrypt_block(&self, block: &[u8; 16]) -> [u8; 16] {
        let sm4 = Sm4Core::new(&self.key);
        let mut out = [0u8; 16];
        out.copy_from_slice(&sm4.decrypt(block));
        out
    }
}