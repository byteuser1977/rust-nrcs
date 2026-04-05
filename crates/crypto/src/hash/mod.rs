//! Hash algorithms module

mod sha256;
mod sm3;

pub use sha256::Sha256;
pub use sm3::Sm3;

/// Re-export the hash function types
pub type Hash256 = [u8; 32];