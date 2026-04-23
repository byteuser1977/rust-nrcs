//! 加密算法模块
//!
//! 提供多种加密算法的实现，包括 SM4-GCM。

// pub mod sm4; // TODO: 修复 SM4 导入问题
pub mod sm4_gcm;

pub use sm4_gcm::Sm4Gcm;
