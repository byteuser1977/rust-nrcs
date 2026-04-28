//! 交易附件二进制序列化模块
//!
//! 对应 Java: AbstractAppendix.putBytes() / 各子类的 putMyBytes()
//! 以及 TransactionService.saveTransactions() 中的 attachment_bytes 构建
//!
//! 二进制协议规范（与 Java NRCS 完全一致）：
//! - 字节序: LITTLE_ENDIAN
//! - 每个附录格式: [version(1B, 仅version>0)] + [putMyBytes()自定义数据]
//! - attachmentBytes = Attachment二进制 + Message二进制 + EncryptedMessage二进制 + ...

use super::transaction::*;
use serde_json::Map;

/// 从 JSON attachment 对象构建完整的二进制 attachment_bytes
///
/// 完整结构：
/// [Attachment部分] [Message部分] [EncryptedMessage部分] [PublicKeyAnnouncement部分]
/// [EncryptToSelfMessage部分] [Phasing部分] [PrunablePlainMessage部分] [PrunableEncryptedMessage部分]
pub fn build_attachment_bytes_from_json(
    type_byte: u8,
    subtype: u8,
    version: u8,
    att_obj: Option<&Map<String, serde_json::Value>>,
) -> Vec<u8> {
    let mut result = Vec::new();

    // === 1. 序列化 Attachment 部分 ===
    let att_bytes = serialize_attachment(type_byte, subtype, version, att_obj);
    if !att_bytes.is_empty() {
        put_version_and_data(&mut result, version, |buf| {
            put_bytes(buf, &att_bytes);
        });
    }

    // === 2. 序列化各 Appendix 部分（从 attachment JSON 中检测） ===
    let att_map = match att_obj {
        Some(m) => m,
        None => return result,
    };

    // Message appendix（对应 Java: AppendixMessage）
    if let Some(msg_val) = att_map.get("message") {
        let message_str = msg_val.as_str().unwrap_or("");
        let message_bytes = message_str.as_bytes();
        let is_text = true;
        let len_with_flag = if is_text {
            (message_bytes.len() as i32) | (0x80000000u32 as i32)
        } else {
            message_bytes.len() as i32
        };
        put_version_and_data(&mut result, version, |buf| {
            put_i32(buf, len_with_flag);
            put_bytes(buf, message_bytes);
        });
    }

    // EncryptedMessage appendix（对应 Java: EncryptedMessage）
    if let Some(enc_msg) = att_map.get("encryptedMessage") {
        if let Some(enc_obj) = enc_msg.as_object() {
            let data_hex = enc_obj.get("data")
                .and_then(|v| v.as_str())
                .and_then(|s| hex::decode(s).ok())
                .unwrap_or_default();
            let nonce_hex = enc_obj.get("nonce")
                .and_then(|v| v.as_str())
                .and_then(|s| hex::decode(s).ok())
                .unwrap_or_default();
            let is_text = enc_obj.get("isText")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let len_with_flag = if is_text {
                (data_hex.len() as i32) | (0x80000000u32 as i32)
            } else {
                data_hex.len() as i32
            };
            put_version_and_data(&mut result, version, |buf| {
                put_i32(buf, len_with_flag);
                put_bytes(buf, &data_hex);
                put_bytes(buf, &nonce_hex);
            });
        }
    }

    // EncryptToSelfMessage appendix（对应 Java: EncryptToSelfMessage）
    if let Some(ets_msg) = att_map.get("encryptToSelfMessage") {
        if let Some(ets_obj) = ets_msg.as_object() {
            let data_hex = ets_obj.get("data")
                .and_then(|v| v.as_str())
                .and_then(|s| hex::decode(s).ok())
                .unwrap_or_default();
            let nonce_hex = ets_obj.get("nonce")
                .and_then(|v| v.as_str())
                .and_then(|s| hex::decode(s).ok())
                .unwrap_or_default();
            let is_text = ets_obj.get("isText")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let len_with_flag = if is_text {
                (data_hex.len() as i32) | (0x80000000u32 as i32)
            } else {
                data_hex.len() as i32
            };
            put_version_and_data(&mut result, version, |buf| {
                put_i32(buf, len_with_flag);
                put_bytes(buf, &data_hex);
                put_bytes(buf, &nonce_hex);
            });
        }
    }

    // PublicKeyAnnouncement appendix（对应 Java: PublicKeyAnnouncement）
    if let Some(pk_val) = att_map.get("recipientPublicKey") {
        if let Some(pk_str) = pk_val.as_str() {
            if let Ok(pk_bytes) = hex::decode(pk_str) {
                put_version_and_data(&mut result, version, |buf| {
                    put_bytes(buf, &pk_bytes);
                });
            }
        }
    }

    // Phasing appendix（对应 Java: AppendixPhasing）
    // 检测条件: 有 phasingFinishHeight 或 phased=true
    if att_map.get("phasingFinishHeight").is_some()
        || att_map.get("phased").and_then(|v| v.as_bool()).unwrap_or(false)
    {
        serialize_phasing_appendix(&mut result, version, att_map);
    }

    // PrunablePlainMessage appendix（对应 Java: PrunablePlainMessage）
    if let Some(ppm_val) = att_map.get("prunablePlainMessage") {
        if let Some(hash_str) = ppm_val.as_str().or_else(|| ppm_val.as_str()) {
            if let Ok(hash_bytes) = hex::decode(hash_str) {
                put_version_and_data(&mut result, version, |buf| {
                    put_bytes(buf, &hash_bytes);
                });
            }
        }
    }

    // PrunableEncryptedMessage appendix（对应 Java: PrunableEncryptedMessage）
    if let Some(pem_val) = att_map.get("prunableEncryptedMessage") {
        if let Some(hash_str) = pem_val.as_str().or_else(|| pem_val.as_str()) {
            if let Ok(hash_bytes) = hex::decode(hash_str) {
                put_version_and_data(&mut result, version, |buf| {
                    put_bytes(buf, &hash_bytes);
                });
            }
        }
    }

    result
}

/// 序列化 Phasing appendix
/// 对应 Java: AppendixPhasing.putMyBytes()
/// 格式: finishHeight(4B) + PhasingParams + linkedFullHashesCount(1B) + [hash(32B)]... + hashedSecretLen(1B) + hashedSecret(NB) + algorithm(1B)
///
/// PhasingParams.putMyBytes():
///   votingModel(1B) + quorum(8B) + minBalance(8B) + whitelistLen(1B) + [accountId(8B)]... + holdingId(8B) + minBalanceModel(1B)
fn serialize_phasing_appendix(buf: &mut Vec<u8>, version: u8, att_map: &Map<String, serde_json::Value>) {
    put_version_and_data(buf, version, |buf| {
        // finishHeight (4 bytes, i32 LE)
        let finish_height = att_map.get("phasingFinishHeight")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        put_i32(buf, finish_height);

        // === PhasingParams ===
        // votingModel (1 byte)
        let voting_model = att_map.get("phasingVotingModel")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;
        put_byte(buf, voting_model);

        // quorum (8 bytes, u64 LE - Java long)
        let quorum = att_map.get("phasingQuorum")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .or_else(|| att_map.get("phasingQuorum").and_then(|v| v.as_u64()))
            .unwrap_or(0);
        put_u64(buf, quorum);

        // minBalance (8 bytes, u64 LE - Java long)
        let min_balance = att_map.get("phasingMinBalance")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .or_else(|| att_map.get("phasingMinBalance").and_then(|v| v.as_u64()))
            .unwrap_or(0);
        put_u64(buf, min_balance);

        // whitelist (1 byte count + 8 bytes each account ID)
        if let Some(whitelist_arr) = att_map.get("phasingWhitelist").and_then(|v| v.as_array()) {
            put_byte(buf, whitelist_arr.len() as u8);
            for account_val in whitelist_arr {
                // accountId 可能超出 i64 范围（如 14411432778108101696）
                // Java 使用有符号 long 存储，但 JSON API 返回正数字符串
                // 字节表示上 i64 和 u64 是相同的（8 字节 LE）
                let account_id = account_val.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| account_val.as_u64())
                    .unwrap_or(0);
                put_u64(buf, account_id);
            }
        } else {
            put_byte(buf, 0);
        }

        // holdingId (8 bytes, 可能是大数值)
        let holding_id = att_map.get("phasingHolding")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .or_else(|| att_map.get("phasingHolding").and_then(|v| v.as_u64()))
            .unwrap_or(0);
        put_u64(buf, holding_id);

        // minBalanceModel (1 byte)
        let min_balance_model = att_map.get("phasingMinBalanceModel")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;
        put_byte(buf, min_balance_model);

        // linkedFullHashes (1 byte count + 32 bytes each hash)
        // 通常为空
        put_byte(buf, 0);

        // hashedSecret (1 byte length + data)
        // 通常为空
        put_byte(buf, 0);

        // algorithm (1 byte)
        // 通常为 0 (SHA-256)
        put_byte(buf, 0);
    });
}

/// 检测 attachment JSON 中是否有指定字段，返回各 appendix 标志
///
/// 返回值顺序：(has_message, has_encrypted_message, has_public_key_announcement,
///              has_encrypttoself_message, has_phasing, has_prunable_message, has_prunable_encrypted_message)
pub fn detect_appendix_flags(att_obj: Option<&Map<String, serde_json::Value>>) -> (bool, bool, bool, bool, bool, bool, bool) {
    let mut has_message = false;
    let mut has_encrypted_message = false;
    let mut has_public_key_announcement = false;
    let mut has_encrypttoself_message = false;
    let mut has_phasing = false;
    let mut has_prunable_message = false;
    let mut has_prunable_encrypted_message = false;

    let att_map = match att_obj {
        Some(m) => m,
        None => return (false, false, false, false, false, false, false),
    };

    has_message = att_map.get("message").is_some();
    has_encrypted_message = att_map.get("encryptedMessage").is_some();
    has_public_key_announcement = att_map.get("recipientPublicKey").is_some();
    has_encrypttoself_message = att_map.get("encryptToSelfMessage").is_some();
    has_phasing = att_map.get("phasing").is_some()
        || att_map.get("phased").is_some()
        || att_map.get("phasingFinishHeight").is_some();  // 有 phasingFinishHeight 即表示存在 Phasing
    has_prunable_message = att_map.get("prunablePlainMessage").is_some();
    has_prunable_encrypted_message = att_map.get("prunableEncryptedMessage").is_some();

    (has_message, has_encrypted_message, has_public_key_announcement, has_encrypttoself_message,
     has_phasing, has_prunable_message, has_prunable_encrypted_message)
}

// ============================================================
// 二进制写入工具函数
// ============================================================

fn put_byte(buf: &mut Vec<u8>, val: u8) { buf.push(val); }

fn put_u16(buf: &mut Vec<u8>, val: u16) { buf.extend_from_slice(&val.to_le_bytes()); }

fn put_i16(buf: &mut Vec<u8>, val: i16) { buf.extend_from_slice(&val.to_le_bytes()); }

fn put_u32(buf: &mut Vec<u8>, val: u32) { buf.extend_from_slice(&val.to_le_bytes()); }

fn put_i32(buf: &mut Vec<u8>, val: i32) { buf.extend_from_slice(&val.to_le_bytes()); }

fn put_u64(buf: &mut Vec<u8>, val: u64) { buf.extend_from_slice(&val.to_le_bytes()); }

fn put_i64(buf: &mut Vec<u8>, val: i64) { buf.extend_from_slice(&val.to_le_bytes()); }

fn put_bytes(buf: &mut Vec<u8>, data: &[u8]) { buf.extend_from_slice(data); }

fn put_string(buf: &mut Vec<u8>, s: &str) { buf.extend_from_slice(s.as_bytes()); }

fn put_version_and_data<F>(buf: &mut Vec<u8>, version: u8, put_fn: F)
where
    F: FnOnce(&mut Vec<u8>),
{
    if version > 0 {
        put_byte(buf, version);
    }
    put_fn(buf);
}

// ============================================================
// Attachment 部分序列化
// ============================================================

fn serialize_attachment(
    type_byte: u8,
    subtype: u8,
    _version: u8,
    att_obj: Option<&Map<String, serde_json::Value>>,
) -> Vec<u8> {
    let att_map = match att_obj {
        Some(m) => m,
        None => return vec![],
    };

    match type_byte {
        TYPE_PAYMENT => serialize_payment_attachment(subtype, att_map),
        TYPE_MESSAGING => serialize_messaging_attachment(subtype, att_map),
        TYPE_COLORED_COINS => serialize_colored_coins_attachment(subtype, att_map),
        TYPE_ACCOUNT_CONTROL => serialize_account_control_attachment(subtype, att_map),
        TYPE_ACCOUNT_PROPERTY => serialize_account_property_attachment(subtype, att_map),
        _ => vec![],
    }
}

fn serialize_payment_attachment(_subtype: u8, _att_map: &Map<String, serde_json::Value>) -> Vec<u8> {
    vec![]
}

fn serialize_messaging_attachment(subtype: u8, att_map: &Map<String, serde_json::Value>) -> Vec<u8> {
    let mut buf = Vec::new();

    match subtype {
        SUBTYPE_MESSAGING_ARBITRARY_MESSAGE => {}
        SUBTYPE_MESSAGING_ALIAS_ASSIGNMENT => {
            if let Some(alias) = att_map.get("alias").and_then(|v| v.as_str()) {
                put_byte(&mut buf, alias.as_bytes().len() as u8);
                put_string(&mut buf, alias);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(price) = att_map.get("priceNQT") {
                put_u64(&mut buf, price.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| price.as_u64()).unwrap_or(0));
            } else {
                put_u64(&mut buf, 0);
            }
        }
        SUBTYPE_MESSAGING_ACCOUNT_INFO => {
            if let Some(name) = att_map.get("name").and_then(|v| v.as_str()) {
                put_byte(&mut buf, name.as_bytes().len() as u8);
                put_string(&mut buf, name);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(desc) = att_map.get("description").and_then(|v| v.as_str()) {
                put_u16(&mut buf, desc.as_bytes().len() as u16);
                put_string(&mut buf, desc);
            } else {
                put_u16(&mut buf, 0);
            }
        }
        SUBTYPE_MESSAGING_ACCOUNT_PROPERTY => {
            if let Some(prop) = att_map.get("property").and_then(|v| v.as_str()) {
                put_byte(&mut buf, prop.as_bytes().len() as u8);
                put_string(&mut buf, prop);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(val) = att_map.get("value").and_then(|v| v.as_str()) {
                put_byte(&mut buf, val.as_bytes().len() as u8);
                put_string(&mut buf, val);
            } else {
                put_byte(&mut buf, 0);
            }
        }
        SUBTYPE_MESSAGING_ALIAS_SELL => {
            if let Some(alias) = att_map.get("alias").and_then(|v| v.as_str()) {
                put_byte(&mut buf, alias.as_bytes().len() as u8);
                put_string(&mut buf, alias);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(price) = att_map.get("priceNQT") {
                put_u64(&mut buf, price.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| price.as_u64()).unwrap_or(0));
            } else {
                put_u64(&mut buf, 0);
            }
        }
        SUBTYPE_MESSAGING_ALIAS_BUY | SUBTYPE_MESSAGING_ALIAS_DELETE => {
            if let Some(alias) = att_map.get("alias").and_then(|v| v.as_str()) {
                put_byte(&mut buf, alias.as_bytes().len() as u8);
                put_string(&mut buf, alias);
            } else {
                put_byte(&mut buf, 0);
            }
        }
        _ => {}
    }

    buf
}

fn serialize_colored_coins_attachment(subtype: u8, att_map: &Map<String, serde_json::Value>) -> Vec<u8> {
    let mut buf = Vec::new();

    match subtype {
        SUBTYPE_COLORED_COINS_ASSET_ISSUANCE => {
            if let Some(name) = att_map.get("name").and_then(|v| v.as_str()) {
                put_byte(&mut buf, name.as_bytes().len() as u8);
                put_string(&mut buf, name);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(desc) = att_map.get("description").and_then(|v| v.as_str()) {
                put_u16(&mut buf, desc.as_bytes().len() as u16);
                put_string(&mut buf, desc);
            } else {
                put_u16(&mut buf, 0);
            }
            if let Some(qty) = att_map.get("quantityQNT") {
                put_u64(&mut buf, qty.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| qty.as_u64()).unwrap_or(0));
            } else {
                put_u64(&mut buf, 0);
            }
            if let Some(decimals) = att_map.get("decimals") {
                put_byte(&mut buf, decimals.as_u64().unwrap_or(0) as u8);
            } else {
                put_byte(&mut buf, 0);
            }
        }
        SUBTYPE_COLORED_COINS_ASSET_TRANSFER => {
            if let Some(asset) = att_map.get("asset") {
                put_u64(&mut buf, asset.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| asset.as_u64()).unwrap_or(0));
            } else {
                put_u64(&mut buf, 0);
            }
            if let Some(qty) = att_map.get("quantityQNT") {
                put_u64(&mut buf, qty.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| qty.as_u64()).unwrap_or(0));
            } else {
                put_u64(&mut buf, 0);
            }
        }
        SUBTYPE_COLORED_COINS_ASK_ORDER_PLACEMENT |
        SUBTYPE_COLORED_COINS_BID_ORDER_PLACEMENT => {
            if let Some(asset) = att_map.get("asset") {
                put_u64(&mut buf, asset.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| asset.as_u64()).unwrap_or(0));
            } else {
                put_u64(&mut buf, 0);
            }
            if let Some(qty) = att_map.get("quantityQQT").or_else(|| att_map.get("quantityQNT")) {
                put_u64(&mut buf, qty.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| qty.as_u64()).unwrap_or(0));
            } else {
                put_u64(&mut buf, 0);
            }
            if let Some(price) = att_map.get("priceNQT") {
                put_u64(&mut buf, price.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| price.as_u64()).unwrap_or(0));
            } else {
                put_u64(&mut buf, 0);
            }
        }
        SUBTYPE_COLORED_COINS_ASK_ORDER_CANCELLATION |
        SUBTYPE_COLORED_COINS_BID_ORDER_CANCELLATION => {
            if let Some(order) = att_map.get("order") {
                put_u64(&mut buf, order.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| order.as_u64()).unwrap_or(0));
            } else {
                put_u64(&mut buf, 0);
            }
        }
        SUBTYPE_COLORED_COINS_DIVIDEND_PAYMENT => {
            if let Some(asset) = att_map.get("asset") {
                put_u64(&mut buf, asset.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| asset.as_u64()).unwrap_or(0));
            } else {
                put_u64(&mut buf, 0);
            }
            if let Some(height) = att_map.get("height") {
                put_u32(&mut buf, height.as_u64().unwrap_or(0) as u32);
            } else {
                put_u32(&mut buf, 0);
            }
            if let Some(amount) = att_map.get("amountNQTPerQNT") {
                put_u64(&mut buf, amount.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| amount.as_u64()).unwrap_or(0));
            } else {
                put_u64(&mut buf, 0);
            }
        }
        SUBTYPE_COLORED_COINS_ASSET_DELETE => {
            if let Some(asset) = att_map.get("asset") {
                put_u64(&mut buf, asset.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| asset.as_u64()).unwrap_or(0));
            } else {
                put_u64(&mut buf, 0);
            }
            if let Some(qty) = att_map.get("quantityQNT") {
                put_u64(&mut buf, qty.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| qty.as_u64()).unwrap_or(0));
            } else {
                put_u64(&mut buf, 0);
            }
        }
        SUBTYPE_COLORED_COINS_ASSET_INCREASE => {
            if let Some(asset) = att_map.get("asset") {
                put_u64(&mut buf, asset.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| asset.as_u64()).unwrap_or(0));
            } else {
                put_u64(&mut buf, 0);
            }
            if let Some(qty) = att_map.get("quantityDeltaQNT") {
                put_u64(&mut buf, qty.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| qty.as_u64()).unwrap_or(0));
            } else {
                put_u64(&mut buf, 0);
            }
        }
        _ => {}
    }

    buf
}

fn serialize_account_control_attachment(subtype: u8, att_map: &Map<String, serde_json::Value>) -> Vec<u8> {
    let mut buf = Vec::new();

    match subtype {
        SUBTYPE_ACCOUNT_CONTROL_EFFECTIVE_BALANCE_LEASING => {
            if let Some(period) = att_map.get("period") {
                put_u16(&mut buf, period.as_u64().unwrap_or(0) as u16);
            } else {
                put_u16(&mut buf, 0);
            }
        }
        _ => {}
    }

    buf
}

fn serialize_account_property_attachment(_subtype: u8, att_map: &Map<String, serde_json::Value>) -> Vec<u8> {
    let mut buf = Vec::new();

    if let Some(prop) = att_map.get("property").and_then(|v| v.as_str()) {
        put_byte(&mut buf, prop.as_bytes().len() as u8);
        put_string(&mut buf, prop);
    } else {
        put_byte(&mut buf, 0);
    }
    if let Some(val) = att_map.get("value").and_then(|v| v.as_str()) {
        put_byte(&mut buf, val.as_bytes().len() as u8);
        put_string(&mut buf, val);
    } else {
        put_byte(&mut buf, 0);
    }

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_serialize_empty_payment() {
        let bytes = build_attachment_bytes_from_json(TYPE_PAYMENT, SUBTYPE_PAYMENT_ORDINARY_PAYMENT, 1, None);
        assert!(bytes.is_empty(), "Ordinary payment should have empty attachment bytes");
    }

    #[test]
    fn test_binary_serialize_asset_issuance() {
        let json_str = r#"{"name":"TEST","description":"Test Asset","quantityQNT":"100000","decimals":2}"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        let bytes = build_attachment_bytes_from_json(
            TYPE_COLORED_COINS, SUBTYPE_COLORED_COINS_ASSET_ISSUANCE, 1, Some(att_map)
        );

        assert_eq!(bytes.len(), 27, "Expected 27 bytes for asset issuance");
        assert_eq!(bytes[0], 1, "First byte should be version=1");
        assert_eq!(bytes[1], 4, "Name length should be 4");
        assert_eq!(&bytes[2..6], b"TEST", "Name should be TEST");
    }

    #[test]
    fn test_binary_serialize_asset_transfer() {
        let json_str = r#"{"asset":"12345678901234567","quantityQNT":"1000"}"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        let bytes = build_attachment_bytes_from_json(
            TYPE_COLORED_COINS, SUBTYPE_COLORED_COINS_ASSET_TRANSFER, 1, Some(att_map)
        );

        assert_eq!(bytes.len(), 17, "Expected 17 bytes for asset transfer");

        let asset_id = u64::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]);
        assert_eq!(asset_id, 12345678901234567);

        let quantity = u64::from_le_bytes([bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15], bytes[16]]);
        assert_eq!(quantity, 1000);
    }

    #[test]
    fn test_binary_serialize_payment_with_message() {
        let json_str = r#"{"message":"Hello World"}"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        let bytes = build_attachment_bytes_from_json(
            TYPE_PAYMENT, SUBTYPE_PAYMENT_ORDINARY_PAYMENT, 1, Some(att_map)
        );

        assert_eq!(bytes.len(), 16, "Expected 16 bytes for payment with message");

        let msg_len_field = i32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
        assert!(msg_len_field < 0, "Message length flag should be negative (isText=true)");
        let actual_len = (msg_len_field as u32) & 0x7FFFFFFF;
        assert_eq!(actual_len, 11, "Message length should be 11");
        assert_eq!(&bytes[5..16], b"Hello World");
    }

    #[test]
    fn test_detect_flags() {
        let json_str = r#"{"message":"hello","recipientPublicKey":"abc123"}"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        let (has_msg, has_enc, has_pk, has_ets, has_ph, has_pm, has_pem) =
            detect_appendix_flags(Some(att_map));

        assert!(has_msg, "Should detect message");
        assert!(!has_enc, "Should not detect encrypted message");
        assert!(has_pk, "Should detect public key announcement");
        assert!(!has_ets, "Should not detect encrypt-to-self");
        assert!(!has_ph, "Should not detect phasing");
        assert!(!has_pm, "Should not detect prunable message");
        assert!(!has_pem, "Should not detect prunable encrypted message");
    }

    /// 测试真实交易数据：type=2(subtype=0) 资产发行 + Phasing 分阶段投票
    ///
    /// 这笔交易来自 Java NRCS 区块高度 244，transactionIndex=0
    /// 关键特征：
    /// - type=2, subtype=0: ColoredCoins.AssetIssuance
    /// - version=1: 有 flags/ecBlock 字段
    /// - phased=true: 有 Phasing appendix（phasingFinishHeight=10322, whitelist 含2个账户）
    /// - ecBlockId="3488276486778630462": 字符串格式
    /// - 期望 fullHash: f0dfef7be95acd764476bca5913dcfb3a3e72f6eed6c26a638194558be760bbc
    /// - 期望 id: 8560598425554378736
    #[test]
    fn test_real_transaction_asset_issuance_with_phasing() {
        let json_str = r#"{
            "quantityQNT": "1",
            "phasingQuorum": "2",
            "version.Phasing": 1,
            "description": "办公桌椅",
            "phasingWhitelist": [
                "14411432778108101696",
                "899669674531378169"
            ],
            "version.AssetIssuance": 1,
            "phasingFinishHeight": 10322,
            "phasingHolding": "0",
            "decimals": 0,
            "name": "OFFICE",
            "phasingMinBalance": "0",
            "phasingMinBalanceModel": 0,
            "phasingVotingModel": 0
        }"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        // === 1. 检测 flags ===
        let (has_msg, has_enc, has_pk, has_ets, has_ph, has_pm, has_pem) =
            detect_appendix_flags(Some(att_map));

        assert!(!has_msg, "No message in this transaction");
        assert!(!has_enc, "No encrypted message");
        assert!(!has_pk, "No public key announcement");
        assert!(!has_ets, "No encrypt-to-self message");
        assert!(has_ph, "Should detect phasing (phased=true)");
        assert!(!has_pm, "No prunable plain message");
        assert!(!has_pem, "No prunable encrypted message");

        // === 2. 序列化 attachment_bytes ===
        // type=2(ColoredCoins), subtype=0(AssetIssuance), version=1
        let bytes = build_attachment_bytes_from_json(
            TYPE_COLORED_COINS,
            SUBTYPE_COLORED_COINS_ASSET_ISSUANCE,
            1,
            Some(att_map),
        );

        // 预期结构：
        // [Attachment部分] [Phasing部分]
        //
        // Attachment (AssetIssuance):
        //   version(1B) + nameLen(1B=6) + "OFFICE"(6B) + descLen(2B=12) + "办公桌椅"(12B,UTF-8) + quantity(8B=1) + decimals(1B=0)
        //   = 1 + 1 + 6 + 2 + 12 + 8 + 1 = 31 bytes
        //
        // Phasing:
        //   version(1B) + finishHeight(4B) + votingModel(1B) + quorum(8B) +
        //   minBalance(8B) + whitelistLen(1B=2) + accountId1(8B) + accountId2(8B) +
        //   holdingId(8B) + minBalanceModel(1B) + linkedHashesLen(1B=0) +
        //   hashedSecretLen(1B=0) + algorithm(1B)
        //   = 1 + 4 + 1 + 8 + 8 + 1 + 8 + 8 + 8 + 1 + 1 + 1 + 1 = 51 bytes
        //
        // Total = 31 + 51 = 82 bytes

        assert_eq!(bytes.len(), 82, "Expected 82 bytes for asset issuance with phasing, got {}", bytes.len());

        // 验证 Attachment 部分 (前31字节)
        assert_eq!(bytes[0], 1, "Attachment version should be 1");           // version
        assert_eq!(bytes[1], 6, "Name length should be 6 (OFFICE)");       // nameLen
        assert_eq!(&bytes[2..8], b"OFFICE", "Name should be OFFICE");     // name
        let desc_len = u16::from_le_bytes([bytes[8], bytes[9]]);
        assert_eq!(desc_len, 12, "Description length should be 12 (UTF-8 bytes for 4 Chinese chars)");
        assert_eq!(&bytes[10..22], "办公桌椅".as_bytes(), "Description mismatch");

        // 验证 Phasing 部分 (从第31字节开始)
        let phasing_start = 31;
        assert_eq!(bytes[phasing_start], 1, "Phasing version should be 1");

        let finish_height = i32::from_le_bytes([
            bytes[phasing_start + 1],
            bytes[phasing_start + 2],
            bytes[phasing_start + 3],
            bytes[phasing_start + 4],
        ]);
        assert_eq!(finish_height, 10322, "phasingFinishHeight should be 10322");

        assert_eq!(bytes[phasing_start + 5], 0, "votingModel should be 0");

        let quorum = i64::from_le_bytes([
            bytes[phasing_start + 6], bytes[phasing_start + 7],
            bytes[phasing_start + 8], bytes[phasing_start + 9],
            bytes[phasing_start + 10], bytes[phasing_start + 11],
            bytes[phasing_start + 12], bytes[phasing_start + 13],
        ]);
        assert_eq!(quorum, 2, "phasingQuorum should be 2");

        // whitelist count at phasing_start+21 (after minBalance)
        assert_eq!(bytes[phasing_start + 22], 2, "Whitelist should have 2 accounts");

        // first account: 14411432778108101696 (超出i64范围, Java中存储为负数)
        // Java long 有符号范围: -9223372036854775808 ~ 9223372036854775807
        // 14411432778108101696 as i64 = 14411432778108101696 - 2^64 = -4035311295601449920
        let acct1_bytes = &bytes[phasing_start + 23..phasing_start + 31];
        let acct1_u64 = u64::from_le_bytes([
            acct1_bytes[0], acct1_bytes[1], acct1_bytes[2], acct1_bytes[3],
            acct1_bytes[4], acct1_bytes[5], acct1_bytes[6], acct1_bytes[7],
        ]);
        assert_eq!(acct1_u64, 14411432778108101696, "First whitelist account mismatch");

        // second account: 899669674531378169 (在 i64 范围内)
        let acct2 = i64::from_le_bytes([
            bytes[phasing_start + 31], bytes[phasing_start + 32],
            bytes[phasing_start + 33], bytes[phasing_start + 34],
            bytes[phasing_start + 35], bytes[phasing_start + 36],
            bytes[phasing_start + 37], bytes[phasing_start + 38],
        ]);
        assert_eq!(acct2, 899669674531378169, "Second whitelist account mismatch");
    }

    /// 测试 Phasing appendix 单独序列化
    #[test]
    fn test_phasing_appendix_serialization() {
        let json_str = r#"{
            "phased": true,
            "phasingFinishHeight": 50000,
            "phasingVotingModel": 0,
            "phasingQuorum": "100",
            "phasingMinBalance": "50",
            "phasingHolding": "12345",
            "phasingMinBalanceModel": 1,
            "phasingWhitelist": ["111", "222"]
        }"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        let mut buf = Vec::new();
        serialize_phasing_appendix(&mut buf, 1, att_map);

        // version(1B) + finishHeight(4B) + votingModel(1B) + quorum(8B) +
        // minBalance(8B) + whitelistLen(1B=2) + acct1(8B) + acct2(8B) +
        // holdingId(8B) + minBalanceModel(1B) + linkedHashesLen(1B=0) +
        // hashedSecretLen(1B=0) + algorithm(1B=0)
        // = 1 + 4 + 1 + 8 + 8 + 1 + 8 + 8 + 8 + 1 + 1 + 1 + 1 = 51 bytes
        assert_eq!(buf.len(), 51, "Expected 51 bytes for phasing appendix");

        assert_eq!(buf[0], 1, "Version byte");
        let fh = i32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]]);
        assert_eq!(fh, 50000);
        assert_eq!(buf[5], 0, "Voting model");
    }
}
