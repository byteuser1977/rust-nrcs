//! 具体算法实现
//!
//! 为各种加密算法实现对应的 trait，包括：
//! - 哈希：SHA-256, SM3
//! - 签名：Ed25519, SM2
//! - 对称加密（CBC）：AES-CBC, SM4-CBC
//! - 认证加密（GCM）：AES-GCM, SM4-GCM

pub mod hash;
pub mod signature;
pub mod cipher;

// 重新导出所有算法 struct，方便使用
pub use hash::{Sha256, Sm3};
pub use signature::{Ed25519, Sm2};
pub use cipher::{AesCbc, Sm4Cbc, AesGcm, Sm4Gcm};
