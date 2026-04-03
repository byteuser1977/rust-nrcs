//! SM2 签名算法实现
//!
//! 本模块实现 SM2 数字签名（GM/T 0003-2012）。
//!
//! 签名格式：
//! - 输出 64 字节（r || s）
//! - r 和 s 各 32 字节
//!
//! 与 Ed25519 签名格式类似，便于替换
//!
//! SM2 签名步骤：
//! 1. 计算消息摘要：e = H(M)（使用 SM3）
//! 2. 生成随机数 k ∈ [1, n-1]
//! 3. 计算椭圆曲线点 R = k * G，取 x 坐标作为 r = x_R mod n
//! 4. 计算 s = (1 + d)^{-1} * (e + r * d) mod n，其中 d 为私钥
//! 5. 签名 = (r, s)

use super::*;
use sm2::{sign, verify as sm2_verify};
use sm3::sm3;

/// 对消息签名
///
/// # 参数
/// - `secret_key`: 32 字节 SM2 私钥
/// - `message`: 待签名的消息
///
/// # 返回
/// 64 字节签名（r || s）
pub fn sign_with_secret(secret_key: &SecretKey, message: &[u8]) -> Signature {
    let sig = sign(secret_key, message);
    sig.to_bytes().into()
}

/// 验证签名
///
/// # 参数
/// - `public_key`: 64 字节 SM2 公钥
/// - `message`: 原始消息
/// - `signature`: 64 字节签名
///
/// # 返回
/// `Ok(())` 如果验证通过，`Err` 否则
pub fn verify_with_public(
    public_key: &Sm2PublicKey,
    message: &[u8],
    signature: &[u8; 64],
) -> Result<(), ()> {
    let sig = sm2::Signature::from_bytes(signature)
        .map_err(|_| ())?;
    sm2_verify(public_key, message, &sig)
        .map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sm2_sign_verify_compat() {
        let kp = KeyPair::generate();
        let msg = b"compatibility test";
        let sig_bytes = kp.sign(msg);

        let pubkey_bytes = kp.public_key();
        let pubkey = Sm2PublicKey::from_bytes(&pubkey_bytes.try_into().unwrap()).unwrap();

        assert!(verify_with_public(&pubkey, msg, &sig_bytes.try_into().unwrap()).is_ok());
    }

    #[test]
    fn test_signature_format() {
        let kp = KeyPair::generate();
        let msg = b"format test";
        let sig = kp.sign(msg);

        // 签名应为 64 字节
        assert_eq!(sig.len(), 64);

        // 手动分割 r 和 s
        let r = &sig[..32];
        let s = &sig[32..];

        // r 和 s 不应全为零（除非特殊情况）
        // 对于随机消息，r 和 s 应大部分非零
        let r_nonzero = r.iter().filter(|&&b| b != 0).count();
        let s_nonzero = s.iter().filter(|&&b| b != 0).count();
        assert!(r_nonzero > 0);
        assert!(s_nonzero > 0);
    }

    #[test]
    fn test_signature_reproducibility() {
        // SM2 签名使用随机数 k，所以相同消息签名应不同
        let kp = KeyPair::generate();
        let msg = b"reproducibility test";

        let sig1 = kp.sign(msg);
        let sig2 = kp.sign(msg);

        // 签名应不同（因为 k 随机）
        assert_ne!(sig1, sig2);

        // 但都应能被验证
        assert!(verify(&kp.public_key(), msg, &sig1).is_ok());
        assert!(verify(&kp.public_key(), msg, &sig2).is_ok());
    }
}
