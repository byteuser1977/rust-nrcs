//! 加密算法模块
//!
//! 提供多种加密算法的实现，包括 SM4-GCM 和 AES-256-GCM。

pub mod sm4;
pub mod sm4_gcm;
pub mod aes_gcm;

pub use sm4_gcm::Sm4Gcm;
pub use aes_gcm::AesGcm;
