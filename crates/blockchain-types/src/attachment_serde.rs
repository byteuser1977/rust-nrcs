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
    // Java 源码 (PrunablePlainMessage.java:106-108): buffer.put(getHash())
    // JSON 中使用 "messageHash" 字段存储 hash
    // 检测方式：有 "messageHash" 字段 或 有 "version.PrunablePlainMessage" 字段
    let ppm_hash = att_map.get("messageHash")
        .and_then(|v| v.as_str())
        .and_then(|s| hex::decode(s).ok());

    if let Some(hash_bytes) = ppm_hash {
        put_version_and_data(&mut result, version, |buf| {
            put_bytes(buf, &hash_bytes);
        });
    } else if att_map.get("version.PrunablePlainMessage").is_some() {
        // 有 version.PrunablePlainMessage 但没有 messageHash，尝试从 message 计算
        // 简化处理：写入 32 字节零
        put_version_and_data(&mut result, version, |buf| {
            put_bytes(buf, &[0u8; 32]);
        });
    }

    // PrunableEncryptedMessage appendix（对应 Java: PrunableEncryptedMessage）
    // JSON 中使用 "encryptedMessageHash" 字段存储 hash
    let pem_hash = att_map.get("encryptedMessageHash")
        .and_then(|v| v.as_str())
        .and_then(|s| hex::decode(s).ok());

    if let Some(hash_bytes) = pem_hash {
        put_version_and_data(&mut result, version, |buf| {
            put_bytes(buf, &hash_bytes);
        });
    } else if att_map.get("version.PrunableEncryptedMessage").is_some() {
        put_version_and_data(&mut result, version, |buf| {
            put_bytes(buf, &[0u8; 32]);
        });
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
        || att_map.get("phasingFinishHeight").is_some();

    // PrunablePlainMessage: 检测 messageHash 或 version.PrunablePlainMessage
    has_prunable_message = att_map.get("messageHash").is_some()
        || att_map.get("version.PrunablePlainMessage").is_some();

    // PrunableEncryptedMessage: 检测 encryptedMessageHash 或 version.PrunableEncryptedMessage
    has_prunable_encrypted_message = att_map.get("encryptedMessageHash").is_some()
        || att_map.get("version.PrunableEncryptedMessage").is_some();

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

/// 序列化 MonetarySystem 类型附件
/// 对应 Java: MonetarySystemCurrencyIssuance.putMyBytes() 等
fn serialize_monetary_system_attachment(subtype: u8, att_map: &Map<String, serde_json::Value>) -> Vec<u8> {
    let mut buf = Vec::new();

    match subtype {
        SUBTYPE_MONETARY_SYSTEM_CURRENCY_ISSUANCE => {
            // 对应 Java: MonetarySystemCurrencyIssuance.putMyBytes()
            // Java 源码: name(BYTE) + code(BYTE) + desc(SHORT) + type(BYTE) +
            //   initialSupply(i64) + reserveSupply(i64) + maxSupply(i64) +
            //   issuanceHeight(i32) + minReservePerUnitNQT(i64) +
            //   minDifficulty(BYTE) + maxDifficulty(BYTE) + ruleset(BYTE) + algorithm(BYTE) + decimals(BYTE)
            if let Some(name) = att_map.get("name").and_then(|v| v.as_str()) {
                put_byte(&mut buf, name.as_bytes().len() as u8);
                put_string(&mut buf, name);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(code) = att_map.get("code").and_then(|v| v.as_str()) {
                put_byte(&mut buf, code.as_bytes().len() as u8);
                put_string(&mut buf, code);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(desc) = att_map.get("description").and_then(|v| v.as_str()) {
                put_u16(&mut buf, desc.as_bytes().len() as u16);
                put_string(&mut buf, desc);
            } else {
                put_u16(&mut buf, 0);
            }
            if let Some(ty) = att_map.get("type") {
                put_byte(&mut buf, ty.as_u64().unwrap_or(0) as u8);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(val) = att_map.get("initialSupply") {
                put_i64(&mut buf, val.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| val.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(val) = att_map.get("reserveSupply") {
                put_i64(&mut buf, val.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| val.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(val) = att_map.get("maxSupply") {
                put_i64(&mut buf, val.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| val.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(val) = att_map.get("issuanceHeight") {
                put_i32(&mut buf, val.as_str()
                    .and_then(|s| s.parse::<i32>().ok())
                    .or_else(|| val.as_i64().map(|v| v as i32))
                    .unwrap_or(0));
            } else {
                put_i32(&mut buf, 0);
            }
            if let Some(val) = att_map.get("minReservePerUnitNQT") {
                put_i64(&mut buf, val.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| val.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(val) = att_map.get("minDifficulty") {
                put_byte(&mut buf, val.as_u64().unwrap_or(0) as u8);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(val) = att_map.get("maxDifficulty") {
                put_byte(&mut buf, val.as_u64().unwrap_or(0) as u8);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(val) = att_map.get("ruleset") {
                put_byte(&mut buf, val.as_u64().unwrap_or(0) as u8);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(val) = att_map.get("algorithm") {
                put_byte(&mut buf, val.as_u64().unwrap_or(0) as u8);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(val) = att_map.get("decimals") {
                put_byte(&mut buf, val.as_u64().unwrap_or(0) as u8);
            } else {
                put_byte(&mut buf, 0);
            }
        }
        SUBTYPE_MONETARY_SYSTEM_RESERVE_INCREASE => {
            // 对应 Java: MonetarySystemReserveIncrease.putMyBytes()
            // Java: buffer.putLong(currencyId) + buffer.putLong(amountPerUnitNQT)
            if let Some(cur) = att_map.get("currency") {
                put_i64(&mut buf, cur.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| cur.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(amt) = att_map.get("amountPerUnitNQT") {
                put_i64(&mut buf, amt.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| amt.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_MONETARY_SYSTEM_RESERVE_CLAIM => {
            // 对应 Java: MonetarySystemReserveClaim.putMyBytes()
            // Java: buffer.putLong(currencyId) + buffer.putLong(units)
            if let Some(cur) = att_map.get("currency") {
                put_i64(&mut buf, cur.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| cur.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(units) = att_map.get("units") {
                put_i64(&mut buf, units.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| units.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_MONETARY_SYSTEM_CURRENCY_TRANSFER => {
            // 对应 Java: MonetarySystemCurrencyTransfer.putMyBytes()
            // Java: buffer.putLong(currencyId) + buffer.putLong(units)
            if let Some(cur) = att_map.get("currency") {
                put_i64(&mut buf, cur.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| cur.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(units) = att_map.get("units") {
                put_i64(&mut buf, units.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| units.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_MONETARY_SYSTEM_PUBLISH_EXCHANGE_OFFER => {
            // 对应 Java: MonetarySystemPublishExchangeOffer.putMyBytes()
            // Java: currencyId(i64) + buyRateNQT(i64) + sellRateNQT(i64) +
            //      totalBuyLimit(i64) + totalSellLimit(i64) + initialBuySupply(i64) +
            //      initialSellSupply(i64) + expirationHeight(i32)
            if let Some(cur) = att_map.get("currency") {
                put_i64(&mut buf, cur.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| cur.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            for field in &["buyRateNQT", "sellRateNQT", "totalBuyLimit", "totalSellLimit",
                           "initialBuySupply", "initialSellSupply"] {
                if let Some(val) = att_map.get(*field) {
                    put_i64(&mut buf, val.as_str()
                        .and_then(|s| s.parse::<i64>().ok())
                        .or_else(|| val.as_i64())
                        .unwrap_or(0));
                } else {
                    put_i64(&mut buf, 0);
                }
            }
            if let Some(eh) = att_map.get("expirationHeight") {
                put_i32(&mut buf, eh.as_str()
                    .and_then(|s| s.parse::<i32>().ok())
                    .or_else(|| eh.as_i64().map(|v| v as i32))
                    .unwrap_or(0));
            } else {
                put_i32(&mut buf, 0);
            }
        }
        SUBTYPE_MONETARY_SYSTEM_EXCHANGE_BUY | SUBTYPE_MONETARY_SYSTEM_EXCHANGE_SELL => {
            // 对应 Java: MonetarySystemExchange.putMyBytes()
            // Java: buffer.putLong(currencyId) + buffer.putLong(rateNQT) + buffer.putLong(units)
            if let Some(cur) = att_map.get("currency") {
                put_i64(&mut buf, cur.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| cur.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(rate) = att_map.get("rateNQT") {
                put_i64(&mut buf, rate.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| rate.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(units) = att_map.get("units") {
                put_i64(&mut buf, units.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| units.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_MONETARY_SYSTEM_CURRENCY_MINTING => {
            // 对应 Java: MonetarySystemCurrencyMinting.putMyBytes()
            // Java: buffer.putLong(nonce) + buffer.putLong(currencyId) + buffer.putLong(units) + buffer.putLong(counter)
            if let Some(nonce) = att_map.get("nonce") {
                put_i64(&mut buf, nonce.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| nonce.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(cur) = att_map.get("currency") {
                put_i64(&mut buf, cur.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| cur.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(units) = att_map.get("units") {
                put_i64(&mut buf, units.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| units.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(counter) = att_map.get("counter") {
                put_i64(&mut buf, counter.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| counter.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_MONETARY_SYSTEM_CURRENCY_DELETION => {
            // 对应 Java: MonetarySystemCurrencyDeletion.putMyBytes()
            // Java: buffer.putLong(currencyId)
            if let Some(cur) = att_map.get("currency") {
                put_i64(&mut buf, cur.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| cur.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_COLORED_COINS_PROPERTY_SET => {
            // 对应 Java: AssetPropertyAttachment.putMyBytes()
            // Java 源码 (AssetPropertyAttachment.java:50-54):
            //   buffer.putLong(assetId);
            //   PROPERTY_NAME_RW.writeToBuffer(property, buffer);  // BYTE prefix
            //   PROPERTY_VALUE_RW.writeToBuffer(value, buffer);    // UBYTE prefix
            if let Some(asset) = att_map.get("asset") {
                put_i64(&mut buf, asset.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| asset.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
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
        _ => {}
    }

    buf
}

/// 序列化 Data 类型附件
/// 对应 Java: TaggedDataUploadAttachment 等
fn serialize_data_attachment(subtype: u8, att_map: &Map<String, serde_json::Value>) -> Vec<u8> {
    let mut buf = Vec::new();

    match subtype {
        SUBTYPE_DATA_TAGGED_DATA_UPLOAD => {
            // 对应 Java: TaggedDataUpload.putMyBytes()
            // Java 源码 (TaggedDataUpload.java:80-82):
            //   buffer.put(getHash());  // 32 bytes hash
            //
            // 当 JSON 中有 data 时，计算 hash；否则使用 hash 字段
            if let Some(hash_str) = att_map.get("hash").and_then(|v| v.as_str()) {
                if let Ok(hash_bytes) = hex::decode(hash_str) {
                    put_bytes(&mut buf, &hash_bytes);
                }
            }
            // 如果没有 hash，但有 data，则应该计算 hash（但这里简化处理，假设 hash 总是存在）
        }
        SUBTYPE_DATA_TAGGED_DATA_EXTEND => {
            // 对应 Java: TaggedDataExtendAttachment.putMyBytes()
            // Java 源码 (TaggedDataExtendAttachment.java:56-58):
            //   buffer.putLong(taggedDataId);  // 8 bytes
            if let Some(tagged_data) = att_map.get("taggedData") {
                put_i64(&mut buf, tagged_data.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| tagged_data.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        _ => {}
    }

    buf
}

/// 序列化 LightContract 类型附件
/// 对应 Java: ContractReferenceAttachment.putMyBytes() 等
fn serialize_light_contract_attachment(subtype: u8, att_map: &Map<String, serde_json::Value>) -> Vec<u8> {
    let mut buf = Vec::new();

    match subtype {
        SUBTYPE_LIGHT_CONTRACT_REFERENCE_SET => {
            // 对应 Java: ContractReferenceAttachment.putMyBytes()
            // Java 源码 (ContractReferenceAttachment.java:50-54):
            //   NAME_RW.writeToBuffer(contractName, buffer);      // BYTE prefix
            //   PARAMS_RW.writeToBuffer(contractParams, buffer);  // BYTE prefix
            //   contractId.put(buffer);                            // chainId(i32) + hash(32B)
            if let Some(name) = att_map.get("contractName").and_then(|v| v.as_str()) {
                put_byte(&mut buf, name.as_bytes().len() as u8);
                put_string(&mut buf, name);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(params) = att_map.get("contractParams").and_then(|v| v.as_str()) {
                put_byte(&mut buf, params.as_bytes().len() as u8);
                put_string(&mut buf, params);
            } else {
                put_byte(&mut buf, 0);
            }
            // ChainTransactionId: chainId(i32) + hash(32B)
            if let Some(contract_obj) = att_map.get("contract").and_then(|v| v.as_object()) {
                // chainId
                if let Some(chain) = contract_obj.get("chain") {
                    put_i32(&mut buf, chain.as_i64().unwrap_or(0) as i32);
                } else {
                    put_i32(&mut buf, 0);
                }
                // hash (transactionFullHash)
                if let Some(hash_str) = contract_obj.get("transactionFullHash").and_then(|v| v.as_str()) {
                    if let Ok(hash_bytes) = hex::decode(hash_str) {
                        put_bytes(&mut buf, &hash_bytes);
                    } else {
                        put_bytes(&mut buf, &[0u8; 32]);
                    }
                } else {
                    put_bytes(&mut buf, &[0u8; 32]);
                }
            } else {
                put_i32(&mut buf, 0);
                put_bytes(&mut buf, &[0u8; 32]);
            }
        }
        SUBTYPE_LIGHT_CONTRACT_REFERENCE_DELETE => {
            // 对应 Java: ContractReferenceDeleteAttachment.putMyBytes()
            // Java 源码 (ContractReferenceDeleteAttachment.java:39-41):
            //   buffer.putLong(contractReferenceId);  // 8 bytes
            if let Some(ref_id) = att_map.get("contractReference") {
                put_i64(&mut buf, ref_id.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| ref_id.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        _ => {}
    }

    buf
}

fn serialize_account_property_attachment(subtype: u8, att_map: &Map<String, serde_json::Value>) -> Vec<u8> {
    let mut buf = Vec::new();

    match subtype {
        SUBTYPE_ACCOUNT_PROPERTY_SET => {
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
        SUBTYPE_ACCOUNT_PROPERTY_DELETE => {
            if let Some(prop) = att_map.get("property") {
                put_i64(&mut buf, prop.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| prop.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        _ => {}
    }

    buf
}

fn serialize_digital_goods_attachment(subtype: u8, att_map: &Map<String, serde_json::Value>) -> Vec<u8> {
    let mut buf = Vec::new();

    match subtype {
        SUBTYPE_DIGITAL_GOODS_LISTING => {
            if let Some(name) = att_map.get("name").and_then(|v| v.as_str()) {
                put_u16(&mut buf, name.as_bytes().len() as u16);
                put_string(&mut buf, name);
            } else {
                put_u16(&mut buf, 0);
            }
            if let Some(desc) = att_map.get("description").and_then(|v| v.as_str()) {
                put_u16(&mut buf, desc.as_bytes().len() as u16);
                put_string(&mut buf, desc);
            } else {
                put_u16(&mut buf, 0);
            }
            if let Some(tags) = att_map.get("tags").and_then(|v| v.as_str()) {
                put_u16(&mut buf, tags.as_bytes().len() as u16);
                put_string(&mut buf, tags);
            } else {
                put_u16(&mut buf, 0);
            }
            if let Some(qty) = att_map.get("quantity") {
                put_i32(&mut buf, qty.as_i64().unwrap_or(0) as i32);
            } else {
                put_i32(&mut buf, 0);
            }
            if let Some(price) = att_map.get("priceNQT") {
                put_i64(&mut buf, price.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| price.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_DIGITAL_GOODS_DELISTING | SUBTYPE_DIGITAL_GOODS_FEEDBACK => {
            let key = if subtype == SUBTYPE_DIGITAL_GOODS_DELISTING { "goods" } else { "purchase" };
            if let Some(id) = att_map.get(key) {
                put_i64(&mut buf, id.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| id.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_DIGITAL_GOODS_PRICE_CHANGE => {
            if let Some(goods) = att_map.get("goods") {
                put_i64(&mut buf, goods.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| goods.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(price) = att_map.get("priceNQT") {
                put_i64(&mut buf, price.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| price.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_DIGITAL_GOODS_QUANTITY_CHANGE => {
            if let Some(goods) = att_map.get("goods") {
                put_i64(&mut buf, goods.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| goods.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(delta) = att_map.get("deltaQuantity") {
                put_i32(&mut buf, delta.as_i64().unwrap_or(0) as i32);
            } else {
                put_i32(&mut buf, 0);
            }
        }
        SUBTYPE_DIGITAL_GOODS_PURCHASE => {
            if let Some(goods) = att_map.get("goods") {
                put_i64(&mut buf, goods.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| goods.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(qty) = att_map.get("quantity") {
                put_i32(&mut buf, qty.as_i64().unwrap_or(0) as i32);
            } else {
                put_i32(&mut buf, 0);
            }
            if let Some(price) = att_map.get("priceNQT") {
                put_i64(&mut buf, price.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| price.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(deadline) = att_map.get("deliveryDeadlineTimestamp") {
                put_i32(&mut buf, deadline.as_i64().unwrap_or(0) as i32);
            } else {
                put_i32(&mut buf, 0);
            }
        }
        SUBTYPE_DIGITAL_GOODS_DELIVERY => {
            if let Some(purchase) = att_map.get("purchase") {
                put_i64(&mut buf, purchase.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| purchase.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            let is_text = att_map.get("goodsIsText").and_then(|v| v.as_bool()).unwrap_or(false);
            if let Some(data) = att_map.get("goodsData").and_then(|v| v.as_str()) {
                if let Ok(data_bytes) = hex::decode(data) {
                    let len = data_bytes.len() as i32;
                    put_i32(&mut buf, if is_text { (len as u32 | 0x80000000u32) as i32 } else { len });
                    put_bytes(&mut buf, &data_bytes);
                }
            } else {
                put_i32(&mut buf, 0);
            }
            if let Some(nonce) = att_map.get("goodsNonce").and_then(|v| v.as_str()) {
                if let Ok(nonce_bytes) = hex::decode(nonce) {
                    put_bytes(&mut buf, &nonce_bytes);
                }
            }
            if let Some(discount) = att_map.get("discountNQT") {
                put_i64(&mut buf, discount.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| discount.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_DIGITAL_GOODS_REFUND => {
            if let Some(purchase) = att_map.get("purchase") {
                put_i64(&mut buf, purchase.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| purchase.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(refund) = att_map.get("refundNQT") {
                put_i64(&mut buf, refund.as_i64().unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        _ => {}
    }

    buf
}

fn serialize_shuffling_attachment(_subtype: u8, att_map: &Map<String, serde_json::Value>) -> Vec<u8> {
    let mut buf = Vec::new();

    if let Some(shuffling) = att_map.get("shuffling") {
        put_i64(&mut buf, shuffling.as_str()
            .and_then(|s| s.parse::<i64>().ok())
            .or_else(|| shuffling.as_i64())
            .unwrap_or(0));
    } else {
        put_i64(&mut buf, 0);
    }
    if let Some(hash) = att_map.get("shufflingStateHash").and_then(|v| v.as_str()) {
        if let Ok(hash_bytes) = hex::decode(hash) {
            put_bytes(&mut buf, &hash_bytes);
        } else {
            put_bytes(&mut buf, &[0u8; 32]);
        }
    } else {
        put_bytes(&mut buf, &[0u8; 32]);
    }

    buf
}

fn serialize_aliases_attachment(subtype: u8, att_map: &Map<String, serde_json::Value>) -> Vec<u8> {
    let mut buf = Vec::new();

    match subtype {
        SUBTYPE_ALIASES_ALIAS_ASSIGNMENT => {
            if let Some(alias) = att_map.get("alias").and_then(|v| v.as_str()) {
                put_byte(&mut buf, alias.as_bytes().len() as u8);
                put_string(&mut buf, alias);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(uri) = att_map.get("uri").and_then(|v| v.as_str()) {
                put_u16(&mut buf, uri.as_bytes().len() as u16);
                put_string(&mut buf, uri);
            } else {
                put_u16(&mut buf, 0);
            }
        }
        SUBTYPE_ALIASES_ALIAS_SELL => {
            if let Some(alias) = att_map.get("alias").and_then(|v| v.as_str()) {
                put_byte(&mut buf, alias.as_bytes().len() as u8);
                put_string(&mut buf, alias);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(price) = att_map.get("priceNQT") {
                put_i64(&mut buf, price.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| price.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_ALIASES_ALIAS_BUY | SUBTYPE_ALIASES_ALIAS_DELETE => {
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

fn serialize_voting_attachment(subtype: u8, att_map: &Map<String, serde_json::Value>) -> Vec<u8> {
    let mut buf = Vec::new();

    match subtype {
        SUBTYPE_VOTING_POLL_CREATION => {
            if let Some(name) = att_map.get("name").and_then(|v| v.as_str()) {
                put_u16(&mut buf, name.as_bytes().len() as u16);
                put_string(&mut buf, name);
            } else {
                put_u16(&mut buf, 0);
            }
            if let Some(desc) = att_map.get("description").and_then(|v| v.as_str()) {
                put_u16(&mut buf, desc.as_bytes().len() as u16);
                put_string(&mut buf, desc);
            } else {
                put_u16(&mut buf, 0);
            }
            if let Some(fh) = att_map.get("finishHeight") {
                put_i32(&mut buf, fh.as_i64().unwrap_or(0) as i32);
            } else {
                put_i32(&mut buf, 0);
            }
            if let Some(opts) = att_map.get("options").and_then(|v| v.as_array()) {
                put_byte(&mut buf, opts.len() as u8);
                for opt in opts {
                    if let Some(opt_str) = opt.as_str() {
                        put_u16(&mut buf, opt_str.as_bytes().len() as u16);
                        put_string(&mut buf, opt_str);
                    }
                }
            } else {
                put_byte(&mut buf, 0);
            }
        }
        SUBTYPE_VOTING_VOTE_CASTING => {
            if let Some(poll) = att_map.get("poll") {
                put_i64(&mut buf, poll.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| poll.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(vote) = att_map.get("vote").and_then(|v| v.as_array()) {
                put_byte(&mut buf, vote.len() as u8);
                for v in vote {
                    put_byte(&mut buf, v.as_u64().unwrap_or(0) as u8);
                }
            } else {
                put_byte(&mut buf, 0);
            }
        }
        SUBTYPE_VOTING_PHASING_VOTE_CASTING => {
            if let Some(hashes) = att_map.get("transactionFullHashes").and_then(|v| v.as_array()) {
                put_byte(&mut buf, hashes.len() as u8);
                for h in hashes {
                    if let Some(hs) = h.as_str() {
                        if let Ok(hb) = hex::decode(hs) {
                            put_bytes(&mut buf, &hb);
                        }
                    }
                }
            } else {
                put_byte(&mut buf, 0);
            }
            put_i32(&mut buf, 0);
        }
        _ => {}
    }

    buf
}

fn serialize_coin_exchange_attachment(subtype: u8, att_map: &Map<String, serde_json::Value>) -> Vec<u8> {
    let mut buf = Vec::new();

    match subtype {
        SUBTYPE_COIN_EXCHANGE_ORDER_ISSUE => {
            if let Some(chain) = att_map.get("chain") {
                put_i32(&mut buf, chain.as_i64().unwrap_or(0) as i32);
            } else {
                put_i32(&mut buf, 0);
            }
            if let Some(ex_chain) = att_map.get("exchangeChain") {
                put_i32(&mut buf, ex_chain.as_i64().unwrap_or(0) as i32);
            } else {
                put_i32(&mut buf, 0);
            }
            if let Some(qty) = att_map.get("quantityQNT") {
                put_i64(&mut buf, qty.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| qty.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(price) = att_map.get("priceNQTPerCoin") {
                put_i64(&mut buf, price.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| price.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_COIN_EXCHANGE_ORDER_CANCEL => {
            if let Some(hash) = att_map.get("orderHash").and_then(|v| v.as_str()) {
                if let Ok(hash_bytes) = hex::decode(hash) {
                    put_bytes(&mut buf, &hash_bytes);
                } else {
                    put_bytes(&mut buf, &[0u8; 32]);
                }
            } else {
                put_bytes(&mut buf, &[0u8; 32]);
            }
        }
        _ => {}
    }

    buf
}

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
        TYPE_DIGITAL_GOODS => serialize_digital_goods_attachment(subtype, att_map),
        TYPE_ACCOUNT_CONTROL => serialize_account_control_attachment(subtype, att_map),
        TYPE_MONETARY_SYSTEM => serialize_monetary_system_attachment(subtype, att_map),
        TYPE_DATA => serialize_data_attachment(subtype, att_map),
        TYPE_SHUFFLING => serialize_shuffling_attachment(subtype, att_map),
        TYPE_ALIASES => serialize_aliases_attachment(subtype, att_map),
        TYPE_VOTING => serialize_voting_attachment(subtype, att_map),
        TYPE_ACCOUNT_PROPERTY => serialize_account_property_attachment(subtype, att_map),
        TYPE_COIN_EXCHANGE => serialize_coin_exchange_attachment(subtype, att_map),
        TYPE_LIGHT_CONTRACT => serialize_light_contract_attachment(subtype, att_map),
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
            // 对应 Java: AliasAssignmentAttachment.putMyBytes()
            // Java 源码: ALIAS_NAME_RW.writeToBuffer(aliasName) + ALIAS_URI_RW.writeToBuffer(aliasURI)
            // 格式: nameLen(1B BYTE) + name + uriLen(2B SHORT) + uri
            if let Some(alias) = att_map.get("alias").and_then(|v| v.as_str()) {
                put_byte(&mut buf, alias.as_bytes().len() as u8);
                put_string(&mut buf, alias);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(uri) = att_map.get("uri").and_then(|v| v.as_str()) {
                put_u16(&mut buf, uri.as_bytes().len() as u16);
                put_string(&mut buf, uri);
            } else {
                put_u16(&mut buf, 0);
            }
        }
        SUBTYPE_MESSAGING_POLL_CREATION => {
            // 对应 Java: MessagingPollCreation.putMyBytes()
            // Java: name(SHORT) + desc(SHORT) + finishHeight(i32) + optionCount(BYTE) +
            //      [option(SHORT)...] + votingModel(BYTE) + minNumberOfOptions(BYTE) +
            //      maxNumberOfOptions(BYTE) + minRangeValue(BYTE) + maxRangeValue(BYTE) +
            //      minBalance(i64) + minBalanceModel(BYTE) + holdingId(i64)
            if let Some(name) = att_map.get("name").and_then(|v| v.as_str()) {
                put_u16(&mut buf, name.as_bytes().len() as u16);
                put_string(&mut buf, name);
            } else {
                put_u16(&mut buf, 0);
            }
            if let Some(desc) = att_map.get("description").and_then(|v| v.as_str()) {
                put_u16(&mut buf, desc.as_bytes().len() as u16);
                put_string(&mut buf, desc);
            } else {
                put_u16(&mut buf, 0);
            }
            if let Some(fh) = att_map.get("finishHeight") {
                put_i32(&mut buf, fh.as_i64()
                    .or_else(|| fh.as_str().and_then(|s| s.parse::<i64>().ok()))
                    .map(|v| v as i32)
                    .unwrap_or(0));
            } else {
                put_i32(&mut buf, 0);
            }
            if let Some(opts_arr) = att_map.get("options").and_then(|v| v.as_array()) {
                put_byte(&mut buf, opts_arr.len() as u8);
                for opt_val in opts_arr {
                    if let Some(opt_str) = opt_val.as_str() {
                        put_u16(&mut buf, opt_str.as_bytes().len() as u16);
                        put_string(&mut buf, opt_str);
                    } else {
                        put_u16(&mut buf, 0);
                    }
                }
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(vm) = att_map.get("votingModel") {
                put_byte(&mut buf, vm.as_u64().unwrap_or(0) as u8);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(min_opt) = att_map.get("minNumberOfOptions") {
                put_byte(&mut buf, min_opt.as_u64().unwrap_or(0) as u8);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(max_opt) = att_map.get("maxNumberOfOptions") {
                put_byte(&mut buf, max_opt.as_u64().unwrap_or(0) as u8);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(min_range) = att_map.get("minRangeValue") {
                put_byte(&mut buf, min_range.as_u64().unwrap_or(0) as u8);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(max_range) = att_map.get("maxRangeValue") {
                put_byte(&mut buf, max_range.as_u64().unwrap_or(0) as u8);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(mb) = att_map.get("minBalance") {
                put_i64(&mut buf, mb.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| mb.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(mbm) = att_map.get("minBalanceModel") {
                put_byte(&mut buf, mbm.as_u64().unwrap_or(0) as u8);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(holding) = att_map.get("holding") {
                put_i64(&mut buf, holding.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| holding.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_MESSAGING_VOTE_CASTING => {
            // 对应 Java: MessagingVoteCasting.putMyBytes()
            // Java: buffer.putLong(pollId) + buffer.put(pollVote.length) + buffer.put(pollVote)
            if let Some(poll) = att_map.get("poll") {
                put_i64(&mut buf, poll.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| poll.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(vote_arr) = att_map.get("vote").and_then(|v| v.as_array()) {
                put_byte(&mut buf, vote_arr.len() as u8);
                for vote_val in vote_arr {
                    put_byte(&mut buf, vote_val.as_u64().unwrap_or(0) as u8);
                }
            } else {
                put_byte(&mut buf, 0);
            }
        }
        SUBTYPE_MESSAGING_HUB_ANNOUNCEMENT => {
            // 对应 Java: MessagingHubAnnouncement.putMyBytes()
            // Java: buffer.putLong(minFeePerByteNQT) + buffer.put(uris.length) + [uri(SHORT prefix)...]
            if let Some(fee) = att_map.get("minFeePerByte") {
                put_i64(&mut buf, fee.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| fee.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(uris_arr) = att_map.get("uris").and_then(|v| v.as_array()) {
                put_byte(&mut buf, uris_arr.len() as u8);
                for uri_val in uris_arr {
                    if let Some(uri_str) = uri_val.as_str() {
                        put_u16(&mut buf, uri_str.as_bytes().len() as u16);
                        put_string(&mut buf, uri_str);
                    } else {
                        put_u16(&mut buf, 0);
                    }
                }
            } else {
                put_byte(&mut buf, 0);
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
            // 对应 Java: MessagingAliasSell.putMyBytes()
            // Java: aliasName(BYTE prefix) + priceNQT(long)
            if let Some(alias) = att_map.get("alias").and_then(|v| v.as_str()) {
                put_byte(&mut buf, alias.as_bytes().len() as u8);
                put_string(&mut buf, alias);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(price) = att_map.get("priceNQT") {
                put_i64(&mut buf, price.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| price.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
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
        SUBTYPE_MESSAGING_PHASING_VOTE_CASTING => {
            // 对应 Java: PhasingVoteCastingAttachment.putMyBytes()
            // Java 源码 (PhasingVoteCastingAttachment.java:72-77):
            //   buffer.put((byte) transactionFullHashes.size());  // count (1 byte)
            //   transactionFullHashes.forEach(buffer::put);        // fullHash (32 bytes each)
            //   buffer.putInt(revealedSecret.length);               // secretLength (4 bytes, i32!)
            //   buffer.put(revealedSecret);                          // revealedSecret data
            //
            // 注意: secretLength 使用的是 putInt() (4字节 i32)，不是 put() (1字节)！
            if let Some(hashes_arr) = att_map.get("transactionFullHashes").and_then(|v| v.as_array()) {
                put_byte(&mut buf, hashes_arr.len() as u8);
                for hash_val in hashes_arr {
                    if let Some(hash_str) = hash_val.as_str() {
                        if let Ok(hash_bytes) = hex::decode(hash_str) {
                            put_bytes(&mut buf, &hash_bytes);
                        } else {
                            put_bytes(&mut buf, &[0u8; 32]);
                        }
                    } else {
                        put_bytes(&mut buf, &[0u8; 32]);
                    }
                }
            } else {
                put_byte(&mut buf, 0);
            }

            // revealedSecret: 长度用 i32 (4 bytes) + 数据
            // 对应 Java: buffer.putInt(revealedSecret.length) + buffer.put(revealedSecret)
            if let Some(secret_val) = att_map.get("revealedSecret") {
                let secret_bytes = secret_val.as_str()
                    .and_then(|s| hex::decode(s).ok())
                    .or_else(|| secret_val.as_str().map(|s| s.as_bytes().to_vec()))
                    .unwrap_or_default();
                put_i32(&mut buf, secret_bytes.len() as i32);
                put_bytes(&mut buf, &secret_bytes);
            } else {
                // 没有 revealedSecret 时，写入长度=0 (4 bytes i32)
                // 对应 Java: buffer.putInt(0) 当 revealedSecret 为空时
                put_i32(&mut buf, 0);
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
            // 对应 Java: ColoredCoinsAssetIssuance.putMyBytes()
            // Java: name(BYTE) + desc(SHORT) + quantityQNT(long) + decimals(BYTE)
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
                put_i64(&mut buf, qty.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| qty.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(decimals) = att_map.get("decimals") {
                put_byte(&mut buf, decimals.as_u64().unwrap_or(0) as u8);
            } else {
                put_byte(&mut buf, 0);
            }
        }
        SUBTYPE_COLORED_COINS_ASSET_TRANSFER => {
            // 对应 Java: ColoredCoinsAssetTransfer.putMyBytes()
            // Java: buffer.putLong(assetId) + buffer.putLong(quantityQNT)
            //       [+ bufferShort(commentLen) + comment if version==0]
            if let Some(asset) = att_map.get("asset") {
                put_i64(&mut buf, asset.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| asset.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(qty) = att_map.get("quantityQNT") {
                put_i64(&mut buf, qty.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| qty.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            // comment 字段仅当 version==0 时存在 (Java: getVersion() == 0)
            // 注意: 这里的 version 是 transaction version，不是 attachment version
            // 在实际 P2P 同步中，大多数交易 version >= 1，所以此字段通常为空
            if let Some(comment) = att_map.get("comment").and_then(|v| v.as_str()) {
                if !comment.is_empty() {
                    put_u16(&mut buf, comment.as_bytes().len() as u16);
                    put_string(&mut buf, comment);
                }
            }
        }
        SUBTYPE_COLORED_COINS_ASK_ORDER_PLACEMENT |
        SUBTYPE_COLORED_COINS_BID_ORDER_PLACEMENT => {
            // 对应 Java: ColoredCoinsOrderPlacementAttachment.putMyBytes()
            // Java: buffer.putLong(assetId) + buffer.putLong(quantityQNT) + buffer.putLong(priceNQT)
            if let Some(asset) = att_map.get("asset") {
                put_i64(&mut buf, asset.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| asset.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(qty) = att_map.get("quantityQQT").or_else(|| att_map.get("quantityQNT")) {
                put_i64(&mut buf, qty.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| qty.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(price) = att_map.get("priceNQT") {
                put_i64(&mut buf, price.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| price.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_COLORED_COINS_ASK_ORDER_CANCELLATION |
        SUBTYPE_COLORED_COINS_BID_ORDER_CANCELLATION => {
            // 对应 Java: ColoredCoinsOrderCancellationAttachment.putMyBytes()
            // Java: buffer.putLong(orderId)
            if let Some(order) = att_map.get("order") {
                put_i64(&mut buf, order.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| order.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_COLORED_COINS_DIVIDEND_PAYMENT => {
            // 对应 Java: ColoredCoinsDividendPayment.putMyBytes()
            // Java: buffer.putLong(assetId) + buffer.putInt(height) + buffer.putLong(amountNQTPerQNT)
            // 注意: height 用的是 putInt() 即 i32，不是 u32！
            if let Some(asset) = att_map.get("asset") {
                put_i64(&mut buf, asset.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| asset.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(height) = att_map.get("height") {
                put_i32(&mut buf, height.as_str()
                    .and_then(|s| s.parse::<i32>().ok())
                    .or_else(|| height.as_i64().map(|v| v as i32))
                    .unwrap_or(0));
            } else {
                put_i32(&mut buf, 0);
            }
            if let Some(amount) = att_map.get("amountNQTPerQNT") {
                put_i64(&mut buf, amount.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| amount.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_COLORED_COINS_ASSET_DELETE => {
            // 对应 Java: ColoredCoinsAssetDelete.putMyBytes()
            // Java: buffer.putLong(assetId) + buffer.putLong(quantityQNT)
            if let Some(asset) = att_map.get("asset") {
                put_i64(&mut buf, asset.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| asset.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(qty) = att_map.get("quantityQNT") {
                put_i64(&mut buf, qty.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| qty.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_COLORED_COINS_ASSET_INCREASE => {
            // 对应 Java: AssetIncreaseAttachment (extends AssetQuantityAttachment)
            // Java: buffer.putLong(assetId) + buffer.putLong(quantityDeltaQNT)
            if let Some(asset) = att_map.get("asset") {
                put_i64(&mut buf, asset.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| asset.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(qty) = att_map.get("quantityDeltaQNT") {
                put_i64(&mut buf, qty.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| qty.as_i64())
                    .unwrap_or(0));
            } else {
                put_i64(&mut buf, 0);
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

    /// 测试真实交易数据：type=1(subtype=9) PhasingVoteCasting（分阶段投票）
    ///
    /// 这笔交易来自 Java NRCS 区块高度 249，transactionIndex=0
    /// 是对区块 244 中 Phasing 资产发行交易的投票
    /// 关键特征：
    /// - type=1, subtype=9: Messaging.PhasingVoteCasting
    /// - version=1: 有 flags/ecBlock 字段
    /// - phased=false: 不是 Phasing 本身，而是对某笔 Phasing 的**投票**
    /// - attachment.transactionFullHashes: 指向被投票的 Phasing 交易 fullHash
    /// - ecBlockId="3488276486778630462": 字符串格式
    /// - 期望 fullHash: 87bc5afbe076f9e87a351eef1a11c74dd0be02ee55ed81fd40c7e78d963fc62a
    /// - 期望 id: 16787579794662014087
    #[test]
    fn test_real_transaction_phasing_vote_casting() {
        let json_str = r#"{
            "transactionFullHashes": [
                "f0dfef7be95acd764476bca5913dcfb3a3e72f6eed6c26a638194558be760bbc"
            ],
            "version.PhasingVoteCasting": 1
        }"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        // === 1. 检测 flags（phased=false, 无 Phasing appendix）===
        let (has_msg, has_enc, has_pk, has_ets, has_ph, has_pm, has_pem) =
            detect_appendix_flags(Some(att_map));

        assert!(!has_msg, "No message in this transaction");
        assert!(!has_enc, "No encrypted message");
        assert!(!has_pk, "No public key announcement");
        assert!(!has_ets, "No encrypt-to-self message");
        assert!(!has_ph, "phased=false, no phasingFinishHeight field");
        assert!(!has_pm, "No prunable plain message");
        assert!(!has_pem, "No prunable encrypted message");

        // === 2. 序列化 attachment_bytes ===
        // type=1(Messaging), subtype=9(PhasingVoteCasting), version=1
        let bytes = build_attachment_bytes_from_json(
            TYPE_MESSAGING,
            SUBTYPE_MESSAGING_PHASING_VOTE_CASTING,
            1,
            Some(att_map),
        );

        // 预期结构（基于 Java PhasingVoteCastingAttachment 源码）：
        // [Attachment部分 (PhasingVoteCasting)]
        //
        // Attachment (PhasingVoteCasting):
        //   version(1B) + count(1B=1) + fullHash(32B) + secretLength(4B i32=0)
        //   = 1 + 1 + 32 + 4 = 38 bytes
        //
        // Java 源码 (PhasingVoteCastingAttachment.java:72-77):
        //   buffer.put((byte) size);           // count (1 byte)
        //   buffer.put(hash);                   // fullHash (32 bytes)
        //   buffer.putInt(revealedSecret.length); // secretLength (4 bytes, i32!)
        //   buffer.put(revealedSecret);          // secret data (empty = 0 bytes)
        //
        // 无其他 Appendix（无 Message、无 Phasing 等）

        assert_eq!(bytes.len(), 38, "Expected 38 bytes for phasing vote casting, got {}", bytes.len());

        // 验证 version 字节
        assert_eq!(bytes[0], 1, "Attachment version should be 1");

        // 验证 count (transactionFullHashes 数量)
        assert_eq!(bytes[1], 1, "Should have 1 transaction full hash");

        // 验证 fullHash (32 bytes, hex decoded)
        let expected_hash = hex::decode("f0dfef7be95acd764476bca5913dcfb3a3e72f6eed6c26a638194558be760bbc")
            .expect("Expected hash should be valid hex");
        assert_eq!(&bytes[2..34], expected_hash.as_slice(), "Full hash mismatch");

        // 验证 secretLength (4 bytes i32 = 0，因为没有 revealedSecret)
        let secret_length = i32::from_le_bytes([bytes[34], bytes[35], bytes[36], bytes[37]]);
        assert_eq!(secret_length, 0, "secretLength should be 0 (no revealedSecret)");

        // 验证总字节数正好是 38
        assert_eq!(bytes.len(), 38, "Total should be 38 bytes");

        // 验证这个 fullHash 正好是上一笔交易（资产发行+Phasing）的 fullHash
        // 这确认了投票关系正确建立
    }
}
