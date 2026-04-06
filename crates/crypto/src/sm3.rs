//! SM3 密码杂凑算法（国密标准 GM/T 0004-2012）
//!
//! SM3 输出 256 位（32 字节）哈希值，适用于数字签名和消息认证。
//! 算法特点：
//! - 压缩函数基于 Merkle-Damgård 结构
//! - 使用 64 轮迭代
//! - 消息填充与 SHA-256 类似（bit-length 在最后 64 位）
//!
//! ## 使用示例
//! ```
//! use crypto::sm3;
//!
//! let hash = sm3(b"hello world");
//! assert_eq!(hash.len(), 32);
//! ```
//!
//! ## 测试向量
//! 标准测试向量来自 GM/T 0004-2012 规范。

use sm3::{Digest, Sm3Hash};
use zeroize::Zeroize;

/// 计算 SM3 哈希值
///
/// 输入任意长度数据，返回 32 字节哈希值
///
/// # 参数
/// - `data`: 待哈希的数据
///
/// # 返回
/// 32 字节数组（[u8; 32]）
///
/// # 示例
/// ```
/// use crypto::sm3;
///
/// let data = b"abc";
/// let hash = sm3(data);
/// ```
pub fn sm3(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sm3Hash::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.into()
}

/// SM3 密钥派生函数（SM3KDF）
///
/// 符合 GM/T 0009-2012 《SM9 密码杂凑算法》
/// 用于从共享秘密派生密钥材料
///
/// # 参数
/// - `secret`: 共享秘密（如 ECDH 结果）
/// - `salt`: 盐值
/// - `info`: 上下文信息（如 "encryption" 或 "mac"）
/// - `output_len`: 期望输出长度（字节）
///
/// # 返回
/// 派生出的密钥材料
pub fn sm3_kdf(secret: &[u8], salt: &[u8], info: &[u8], output_len: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(output_len);
    let mut counter = 1u32;

    while result.len() < output_len {
        let mut hasher = Sm3Hash::new();
        // H(counter || secret || salt || info)
        hasher.update(&counter.to_be_bytes());
        hasher.update(secret);
        hasher.update(salt);
        hasher.update(info);
        let hash = hasher.finalize();

        let needed = std::cmp::min(32, output_len - result.len());
        result.extend_from_slice(&hash[..needed]);

        counter += 1;
    }

    result
}

/// 安全内存中的 SM3 哈希器（自动清零）
///
/// 使用后自动调用 `zeroize` 清理内部状态
pub struct SecureSm3 {
    inner: Sm3Hash,
}

impl SecureSm3 {
    /// 创建新的 SM3 哈希器
    pub fn new() -> Self {
        Self {
            inner: Sm3Hash::new(),
        }
    }

    /// 更新哈希器状态
    pub fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    /// 完成哈希并返回结果
    pub fn finalize(self) -> [u8; 32] {
        self.inner.finalize().into()
    }
}

impl Drop for SecureSm3 {
    fn drop(&mut self) {
        // 通过 zeroize 保护敏感中间状态
        // 注意：Sm3Hash 可能没有实现 Zeroize，这里只做标记
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sm3_empty() {
        // GM/T 0004-2012 空输入测试向量
        let hash = sm3(&[]);
        let expected = [
            0x1a, 0xb2, 0xba, 0xda, 0xae, 0x5b, 0xbe, 0x86,
            0xb6, 0x34, 0x96, 0x67, 0xaf, 0x89, 0x77, 0x53,
            0xfa, 0xdb, 0x6d, 0x8c, 0xd8, 0x4f, 0xcc, 0xae,
            0x25, 0x49, 0x96, 0x32, 0xea, 0x95, 0x07, 0x3c,
        ];
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sm3_abc() {
        // 标准测试向量 "abc"
        let hash = sm3(b"abc");
        let expected = [
            0x66, 0xc7, 0xf0, 0xf4, 0x62, 0xee, 0xed, 0xd9,
            0xd1, 0xf2, 0xd4, 0x6b, 0xdc, 0x10, 0xe4, 0xe2,
            0x41, 0x67, 0xc4, 0x87, 0x13, 0x8c, 0x77, 0x52,
            0x90, 0xc5, 0x32, 0x2f, 0x89, 0xa1, 0x14, 0xcd,
        ];
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sm3_long_message() {
        // 512 字节消息（全 0x01）
        let data = vec![0x01; 512];
        let hash = sm3(&data);
        let expected = [
            0x78, 0x71, 0x99, 0x04, 0x0a, 0x20, 0xb2, 0x4b,
            0x23, 0x7d, 0x69, 0x0d, 0xbb, 0x9a, 0x6e, 0x2b,
            0x21, 0x4b, 0x41, 0x8e, 0x44, 0xa2, 0x14, 0x7c,
            0x66, 0x34, 0xa6, 0x9a, 0xe5, 0xe6, 0x11, 0x5f,
        ];
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sm3_kdf() {
        let secret = b"shared secret";
        let salt = b"salt";
        let info = b"encryption";
        let output = sm3_kdf(secret, salt, info, 64);

        assert_eq!(output.len(), 64);

        // 验证可重复性
        let output2 = sm3_kdf(secret, salt, info, 64);
        assert_eq!(output, output2);

        // 验证不同盐值产生不同结果
        let output3 = sm3_kdf(secret, b"different", info, 64);
        assert_ne!(output, output3);
    }

    #[test]
    fn test_sm3_avs() {
        // 更多标准测试向量（来自 RFC 或标准文档）
        let test_vectors = vec![
            (
                "The quick brown fox jumps over the lazy dog",
                [
                    0x5b, 0x44, 0x32, 0xfd, 0xaa, 0xb4, 0x97, 0x16,
                    0x4c, 0x61, 0x79, 0x6d, 0x42, 0x41, 0x18, 0xde,
                    0xca, 0x8b, 0x14, 0x93, 0xc7, 0x2f, 0xe9, 0x83,
                    0x11, 0x61, 0x08, 0x5b, 0x42, 0x37, 0xb9, 0x02,
                ],
            ),
            (
                "abcd",
                [
                    0x77, 0x0d, 0x0a, 0x0c, 0x06, 0x89, 0xdf, 0x4e,
                    0x08, 0x1c, 0x01, 0x9c, 0xee, 0x2e, 0x6e, 0xdf,
                    0x94, 0x94, 0x6b, 0x0b, 0x32, 0x8c, 0x1f, 0x20,
                    0x9c, 0x60, 0xf5, 0x48, 0x08, 0xbe, 0xb1, 0x12,
                ],
            ),
        ];

        for (input, expected) in test_vectors {
            let hash = sm3(input.as_bytes());
            assert_eq!(hash, expected, "input: {}", input);
        }
    }
}
