//! SM4 symmetric cipher (national cryptography standard)
//!
//! GM/T 0002-2012: SM4分组密码算法
//!
//! Provides CBC and GCM (via synthetic construction) modes

mod cipher;
mod cbc;
mod gcm;

pub use cipher::Sm4;
pub use cbc::Sm4Cbc;
pub use gcm::Sm4Gcm;