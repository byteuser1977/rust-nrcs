//! Hallmark 身份标识解析与验证
//!
//! 对应 NRCS Java: Peer.analyzeHallmark(String hallmark)
//!
//! Hallmark 格式:
//! - Base64 编码的数据结构
//! - 包含: public_key(32B) + weight(4B) + date(4B) + signature(64B)
//! - 用于节点身份认证和权重计算
//!
//! 签名验证:
//! - 使用 Ed25519 签名算法
//! - 签名内容: public_key + weight + date
//! - 验证公钥: 从 hallmark 中提取

use tracing::{debug, warn};

/// Hallmark 信息结构
#[derive(Debug, Clone, Default)]
pub struct HallmarkInfo {
    /// 公钥（32字节，用于 Ed25519 签名验证）
    pub public_key: Option<[u8; 32]>,
    /// 权重值（影响广播优先级）
    pub weight: u32,
    /// 创建日期（Unix 时间戳）
    pub date: u32,
    /// 是否有效（签名验证通过）
    pub is_valid: bool,
    /// 原始字符串
    pub raw: String,
    /// 签名（64字节，Ed25519）
    signature: Option<[u8; 64]>,
}

/// Hallmark 解析器
pub struct HallmarkParser;

impl HallmarkParser {
    /// 解析并验证 Hallmark 字符串（完整实现）
    ///
    /// 对应 Java: Peer.analyzeHallmark(String hallmark)
    ///
    /// # Arguments
    /// * `hallmark` - Base64 编码的 hallmark 字符串
    ///
    /// # Returns
    /// * `HallmarkInfo` - 解析后的信息（包含验证结果）
    ///
    /// # Example
    /// ```rust
    /// use p2p::hallmark::{HallmarkParser, HallmarkInfo};
    ///
    /// let info = HallmarkParser::parse("base64_encoded_data");
    /// if info.is_valid {
    ///     println!("Valid hallmark with weight {}", info.weight);
    /// }
    /// ```
    pub fn parse(hallmark: &str) -> HallmarkInfo {
        if hallmark.is_empty() || hallmark.trim().is_empty() {
            return HallmarkInfo::default();
        }

        // 1. Base64 解码
        let decoded = match base64::decode(hallmark) {
            Ok(data) => data,
            Err(e) => {
                warn!("[Hallmark] Failed to decode base64: {}", e);
                return HallmarkInfo {
                    raw: hallmark.to_string(),
                    ..Default::default()
                };
            }
        };

        // 2. 检查最小长度（public_key 32B + weight 4B + date 4B + signature 64B = 104B）
        const MIN_HALLMARK_SIZE: usize = 104;
        if decoded.len() < MIN_HALLMARK_SIZE {
            warn!("[Hallmark] Decoded data too short: {} bytes (minimum {})",
                  decoded.len(), MIN_HALLMARK_SIZE);
            return HallmarkInfo {
                raw: hallmark.to_string(),
                ..Default::default()
            };
        }

        // 3. 解析字段
        let mut info = HallmarkInfo {
            raw: hallmark.to_string(),
            ..Default::default()
        };

        // 提取公钥（前 32 字节）
        let mut key = [0u8; 32];
        key.copy_from_slice(&decoded[0..32]);
        info.public_key = Some(key);

        // 提取权重（接下来 4 字节，小端序）
        info.weight = u32::from_le_bytes([
            decoded[32], decoded[33], decoded[34], decoded[35],
        ]);

        // 提取日期（接下来 4 字节，小端序）
        info.date = u32::from_le_bytes([
            decoded[36], decoded[37], decoded[38], decoded[39],
        ]);

        // 提取签名（最后 64 字节）
        let mut sig = [0u8; 64];
        sig.copy_from_slice(&decoded[40..104]);
        info.signature = Some(sig);

        // 4. 执行完整的签名验证
        info.is_valid = Self::verify_signature(&info, &decoded[0..40]);

        debug!("[Hallmark] Parsed: weight={}, date={}, valid={}",
               info.weight, info.date, info.is_valid);

        info
    }

    /// 验证 Ed25519 签名（完整实现）
    ///
    /// 对应 Java: Crypto.verify(signature, message, publicKey, true)
    ///
    /// # Arguments
    /// * `info` - 已解析的 Hallmark 信息
    /// * `message` - 待验证的消息（public_key + weight + date）
    ///
    /// # Returns
    /// * `bool` - 签名是否有效
    fn verify_signature(info: &HallmarkInfo, message: &[u8]) -> bool {
        use ed25519_dalek::{Signature, Verifier, VerifyingKey};

        // 获取公钥和签名
        let public_key = match &info.public_key {
            Some(key) => key,
            None => {
                warn!("[Hallmark] Cannot verify: no public key");
                return false;
            }
        };

        let signature = match &info.signature {
            Some(sig) => sig,
            None => {
                warn!("[Hallmark] Cannot verify: no signature");
                return false;
            }
        };

        // 构造 VerifyingKey 对象 (ed25519-dalek 2.x)
        let vk = match VerifyingKey::from_bytes(public_key) {
            Ok(vk) => vk,
            Err(e) => {
                warn!("[Hallmark] Invalid public key: {}", e);
                return false;
            }
        };

        // 构造 Signature 对象 (ed25519-dalek 2.x: from_bytes 直接返回 Signature)
        let sig = Signature::from_bytes(signature);

        // 执行 Ed25519 验证
        match vk.verify(message, &sig) {
            Ok(()) => {
                debug!("[Hallmark] Signature verification passed");
                true
            }
            Err(e) => {
                warn!("[Hallmark] Signature verification failed: {}", e);
                false
            }
        }
    }

    /// 验证 Hallmark 是否在有效期内
    ///
    /// Hallmark 通常有有效期限制（如 1 年 = 31536000 秒）
    pub fn is_within_validity_period(info: &HallmarkInfo, max_age_secs: u64) -> bool {
        if !info.is_valid || info.date == 0 {
            return false;
        }

        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now < (info.date as u64) + max_age_secs
    }

    /// 计算节点权重（用于广播选择）
    ///
    /// 对应 Java: Peer.getWeight()
    ///
    /// 权重计算规则:
    /// - 无 hallmark 或无效: 返回基础权重（基于流量）
    /// - 有效 hallmark: 返回 hallmark 中声明的权重
    pub fn calculate_peer_weight(
        info: Option<&HallmarkInfo>,
        downloaded_volume: u64,
        uploaded_volume: u64,
    ) -> u64 {
        match info {
            Some(hm) if hm.is_valid => {
                // 如果有有效的 hallmark，使用其声明权重
                hm.weight as u64
            }
            _ => {
                // 否则基于流量计算权重
                // 公式: downloaded/MB + uploaded/MB
                let download_weight = downloaded_volume / (1024 * 1024);
                let upload_weight = uploaded_volume / (1024 * 1024);
                download_weight.saturating_add(upload_weight).max(1)
            }
        }
    }

    /// 创建新的 Hallmark（用于生成自己的身份标识）
    ///
    /// 对应 Java: Peer.buildHallmark(publicKey, secretKey, weight, date)
    ///
    /// # Arguments
    /// * `secret_key` - Ed25519 私钥（用于签名）
    /// * `weight` - 节点权重
    /// * `date` - 创建时间戳
    ///
    /// # Returns
    /// * `String` - Base64 编码的 hallmark 字符串
    #[allow(dead_code)]
    pub fn create_hallmark(
        _secret_key: &[u8; 32],
        _weight: u32,
        _date: u32,
    ) -> String {
        // TODO: 完整实现需要 Ed25519 签名逻辑
        // 当前返回空字符串作为占位符
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_hallmark() {
        let info = HallmarkParser::parse("");
        assert!(!info.is_valid);
        assert_eq!(info.weight, 0);
    }

    #[test]
    fn test_parse_invalid_base64() {
        let info = HallmarkParser::parse("not-valid-base64!!!");
        assert!(!info.is_valid);
    }

    #[test]
    fn test_default_hallmark_info() {
        let info = HallmarkInfo::default();
        assert!(!info.is_valid);
        assert_eq!(info.weight, 0);
        assert!(info.public_key.is_none());
    }

    #[test]
    fn test_parse_too_short_data() {
        // 创建一个不足 104 字节的 base64 数据
        let short_data = vec![0u8; 40]; // 只有 40 字节
        let encoded = base64::encode(&short_data);

        let info = HallmarkParser::parse(&encoded);
        assert!(!info.is_valid); // 应该因为太短而无效
    }

    #[test]
    fn test_is_within_validity_period() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        let valid_info = HallmarkInfo {
            date: now, // 当前时间
            is_valid: true,
            ..Default::default()
        };

        let expired_info = HallmarkInfo {
            date: now - 40000000u32, // 约 1.3 年前
            is_valid: true,
            ..Default::default()
        };

        // 测试有效期 1 年（31536000 秒）
        assert!(HallmarkParser::is_within_validity_period(&valid_info, 31536000));
        assert!(!HallmarkParser::is_within_validity_period(&expired_info, 31536000));
    }

    #[test]
    fn test_calculate_peer_weight_with_hallmark() {
        let hm = HallmarkInfo {
            weight: 100,
            is_valid: true,
            ..Default::default()
        };

        let weight = HallmarkParser::calculate_peer_weight(Some(&hm), 1024 * 1024, 2048 * 2048);
        assert_eq!(weight, 100); // 应该使用 hallmark 的权重
    }

    #[test]
    fn test_calculate_peer_weight_without_hallmark() {
        let weight = HallmarkParser::calculate_peer_weight(None, 5 * 1024 * 1024, 10 * 1024 * 1024);
        // 5 MB download + 10 MB upload = 15
        assert_eq!(weight, 15);
    }

    #[test]
    fn test_calculate_peer_weight_minimum_one() {
        let weight = HallmarkParser::calculate_peer_weight(None, 0, 0);
        assert_eq!(weight, 1); // 最小权重为 1
    }
}
