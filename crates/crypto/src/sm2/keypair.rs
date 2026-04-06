//! SM2 密钥对管理（实现细节）
//!
//! 本模块提供 SM2 密钥对的生成和序列化功能。
//! 实际 API 由 `mod.rs` 导出，对外暴露统一的 `KeyPair` 结构。
//!
//! 内部使用 `sm2` crate 的 `SecretKey` 和 `PublicKey`。

use super::*;
use rand::rngs::OsRng;

/// 生成 SM2 密钥对（使用操作系统 CSPRNG）
pub fn generate_keypair() -> (SecretKey, Sm2PublicKey) {
    keygen()
}

/// 从 32 字节私钥派生公钥
pub fn derive_public_key(secret: &SecretKey) -> Sm2PublicKey {
    secret.random_public_key()
}

/// 验证私钥字节是否为有效的 SM2 私钥
pub fn is_valid_secret(bytes: &[u8; 32]) -> bool {
    SecretKey::from_bytes(bytes).is_ok()
}

/// 将公钥转换为压缩格式（可选，目前使用未压缩 64 字节）
///
/// SM2P256v1 压缩格式为 33 字节（0x02/0x03 + x）
/// 当前实现使用未压缩格式以简化验证
pub fn compress_public_key(pubkey: &Sm2PublicKey) -> [u8; 33] {
    // 使用未压缩格式，暂时不实现压缩
    let bytes = pubkey.to_bytes();
    let mut compressed = [0u8; 33];
    compressed[0] = 0x04; // 未压缩标识
    compressed[1..].copy_from_slice(&bytes);
    compressed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_consistency() {
        let (secret, public) = generate_keypair();
        let derived = derive_public_key(&secret);
        assert_eq!(public.to_bytes(), derived.to_bytes());
    }

    #[test]
    fn test_key_from_bytes() {
        let bytes = [0x12; 32];
        let secret = SecretKey::from_bytes(&bytes).expect("valid bytes");
        assert_eq!(secret.as_bytes(), &bytes);
    }

    #[test]
    fn test_invalid_secret() {
        // 全零私钥可能无效（在曲线外）
        let zero = [0u8; 32];
        assert!(!is_valid_secret(&zero));
    }
}
