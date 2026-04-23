//! 签名算法模块
//!
//! 提供多种签名算法的实现，包括 Ed25519 和 Curve25519（NRCS 兼容）。

pub mod ed25519;
pub mod curve25519;
// pub mod sm2; // TODO: 修复 SM2 导入问题

pub use ed25519::Ed25519;
pub use curve25519::Curve25519;
// pub use sm2::Sm2;
