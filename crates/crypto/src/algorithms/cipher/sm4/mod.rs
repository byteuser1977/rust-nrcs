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
//!
//! ### CBC 模式
//! ```
//! use crypto::algorithms::cipher::sm4::{encrypt_cbc, decrypt_cbc, Sm4Key};
//!
//! let key = Sm4Key::random();
//! let iv = [0u8; 16];
//! let plaintext = b"Hello, SM4!";
//!
//! // CBC 加密（返回 iv || ciphertext）
//! let ciphertext = encrypt_cbc(plaintext, &key, &iv);
//!
//! // CBC 解密
//! let decrypted = decrypt_cbc(&ciphertext, &key).unwrap();
//! assert_eq!(plaintext, decrypted.as_slice());
//! ```
//!
//! ### ECB 模式
//! ```
//! use crypto::algorithms::cipher::sm4::{encrypt_ecb, decrypt_ecb, Sm4Key};
//!
//! let key = Sm4Key::random();
//! let plaintext = [0x42u8; 32]; // 必须是 16 字节的倍数
//!
//! let ciphertext = encrypt_ecb(&plaintext, &key).unwrap();
//! let decrypted = decrypt_ecb(&ciphertext, &key).unwrap();
//! assert_eq!(plaintext.as_slice(), decrypted.as_slice());
//! ```

mod cipher;

pub use cipher::*;
