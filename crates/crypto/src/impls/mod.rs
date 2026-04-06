//! 具体算法实现
//!
//! 为各种加密算法实现对应的 trait，包括：
//! - 哈希：SHA-256, SM3
//! - 签名：Ed25519

pub mod hash;
pub mod signature;

// 重新导出所有算法 struct，方便使用
pub use hash::{Sha256, Sm3};
pub use signature::Ed25519;
