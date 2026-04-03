//! SM4 分组密码算法（国密标准 GM/T 0002-2012）
//!
//! SM4 是一个 128 位分组、128 位密钥的对称密码算法。
//! 算法特点：
//! - 分组长度：128 位（16 字节）
//! - 密钥长度：128 位（16 字节）
//! - 轮数：32 轮
//! - 结构：Feistel 网络
//!
//! ## 使用示例
//! ```
//! use crypto::sm4::{self, Sm4Key, encrypt_cbc, decrypt_cbc};
//!
//! let key = Sm4Key::random();
//! let iv = [0u8; 16];
//! let plaintext = b"Hello, SM4!";
//!
//! // CBC 加密
//! let ciphertext = encrypt_cbc(plaintext, &key, &iv);
//!
//! // CBC 解密
//! let decrypted = decrypt_cbc(&ciphertext, &key).unwrap();
//! assert_eq!(plaintext, decrypted.as_slice());
//! ```
//!
//! ## GCM 模式（推荐）
//! ```
//! use crypto::sm4::{encrypt_gcm, decrypt_gcm, Sm4Key};
//!
//! let key = Sm4Key::random();
//! let nonce = [0u8; 12]; // GCM nonce 96 位
//! let plaintext = b"Secret data";
//! let aad = b"additional data";
//!
//! // GCM 加密（返回密文 + 认证标签）
//! let (ciphertext, tag) = encrypt_gcm(plaintext, &key, &nonce, aad);
//!
//! // GCM 解密（同时验证标签）
//! let decrypted = decrypt_gcm(&ciphertext, &key, &nonce, &tag, aad).unwrap();
//! assert_eq!(plaintext, decrypted.as_slice());
//! ```

mod cipher;
pub use cipher::*;
