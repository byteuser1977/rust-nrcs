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

/// 获取 attachment 的版本号
/// 从 JSON 中读取 `version.{AttachmentName}` 字段
/// 如果没有找到，返回默认版本 1
fn get_attachment_version(type_byte: u8, subtype: u8, att_map: &Map<String, serde_json::Value>) -> u8 {
    let version_key = match type_byte {
        TYPE_PAYMENT => match subtype {
            SUBTYPE_PAYMENT_ORDINARY_PAYMENT => "version.OrdinaryPayment",
            _ => "version.Payment",
        },
        TYPE_MESSAGING => match subtype {
            SUBTYPE_MESSAGING_ARBITRARY_MESSAGE => "version.ArbitraryMessage",
            SUBTYPE_MESSAGING_ALIAS_ASSIGNMENT => "version.AliasAssignment",
            SUBTYPE_MESSAGING_POLL_CREATION => "version.PollCreation",
            SUBTYPE_MESSAGING_VOTE_CASTING => "version.VoteCasting",
            SUBTYPE_MESSAGING_HUB_ANNOUNCEMENT => "version.HubAnnouncement",
            SUBTYPE_MESSAGING_ACCOUNT_INFO => "version.AccountInfo",
            SUBTYPE_MESSAGING_ALIAS_SELL => "version.AliasSell",
            SUBTYPE_MESSAGING_ALIAS_BUY => "version.AliasBuy",
            SUBTYPE_MESSAGING_ALIAS_DELETE => "version.AliasDelete",
            SUBTYPE_MESSAGING_PHASING_VOTE_CASTING => "version.PhasingVoteCasting",
            SUBTYPE_MESSAGING_ACCOUNT_PROPERTY => "version.AccountProperty",
            _ => "version.Messaging",
        },
        TYPE_COLORED_COINS => match subtype {
            SUBTYPE_COLORED_COINS_ASSET_ISSUANCE => "version.AssetIssuance",
            SUBTYPE_COLORED_COINS_ASSET_TRANSFER => "version.AssetTransfer",
            SUBTYPE_COLORED_COINS_ASK_ORDER_PLACEMENT => "version.AskOrderPlacement",
            SUBTYPE_COLORED_COINS_BID_ORDER_PLACEMENT => "version.BidOrderPlacement",
            SUBTYPE_COLORED_COINS_ASK_ORDER_CANCELLATION => "version.AskOrderCancellation",
            SUBTYPE_COLORED_COINS_BID_ORDER_CANCELLATION => "version.BidOrderCancellation",
            SUBTYPE_COLORED_COINS_DIVIDEND_PAYMENT => "version.DividendPayment",
            SUBTYPE_COLORED_COINS_ASSET_DELETE => "version.AssetDelete",
            SUBTYPE_COLORED_COINS_ASSET_INCREASE => "version.AssetIncrease",
            SUBTYPE_COLORED_COINS_PROPERTY_SET => "version.AssetProperty",
            _ => "version.ColoredCoins",
        },
        TYPE_DIGITAL_GOODS => match subtype {
            SUBTYPE_DIGITAL_GOODS_LISTING => "version.DigitalGoodsListing",
            SUBTYPE_DIGITAL_GOODS_DELISTING => "version.DigitalGoodsDelisting",
            SUBTYPE_DIGITAL_GOODS_PRICE_CHANGE => "version.DigitalGoodsPriceChange",
            SUBTYPE_DIGITAL_GOODS_QUANTITY_CHANGE => "version.DigitalGoodsQuantityChange",
            SUBTYPE_DIGITAL_GOODS_PURCHASE => "version.DigitalGoodsPurchase",
            SUBTYPE_DIGITAL_GOODS_DELIVERY => "version.DigitalGoodsDelivery",
            SUBTYPE_DIGITAL_GOODS_FEEDBACK => "version.DigitalGoodsFeedback",
            SUBTYPE_DIGITAL_GOODS_REFUND => "version.DigitalGoodsRefund",
            _ => "version.DigitalGoods",
        },
        TYPE_ACCOUNT_CONTROL => match subtype {
            SUBTYPE_ACCOUNT_CONTROL_EFFECTIVE_BALANCE_LEASING => "version.EffectiveBalanceLeasing",
            SUBTYPE_ACCOUNT_CONTROL_PHASING_ONLY => "version.PhaserOnly",
            _ => "version.AccountControl",
        },
        TYPE_MONETARY_SYSTEM => match subtype {
            SUBTYPE_MONETARY_SYSTEM_CURRENCY_ISSUANCE => "version.CurrencyIssuance",
            SUBTYPE_MONETARY_SYSTEM_RESERVE_INCREASE => "version.ReserveIncrease",
            SUBTYPE_MONETARY_SYSTEM_RESERVE_CLAIM => "version.ReserveClaim",
            SUBTYPE_MONETARY_SYSTEM_CURRENCY_TRANSFER => "version.CurrencyTransfer",
            SUBTYPE_MONETARY_SYSTEM_PUBLISH_EXCHANGE_OFFER => "version.PublishExchangeOffer",
            SUBTYPE_MONETARY_SYSTEM_EXCHANGE_BUY => "version.ExchangeBuy",
            SUBTYPE_MONETARY_SYSTEM_EXCHANGE_SELL => "version.ExchangeSell",
            SUBTYPE_MONETARY_SYSTEM_CURRENCY_MINTING => "version.CurrencyMinting",
            SUBTYPE_MONETARY_SYSTEM_CURRENCY_DELETION => "version.CurrencyDeletion",
            _ => "version.MonetarySystem",
        },
        TYPE_DATA => match subtype {
            SUBTYPE_DATA_TAGGED_DATA_UPLOAD => "version.TaggedDataUpload",
            SUBTYPE_DATA_TAGGED_DATA_EXTEND => "version.TaggedDataExtend",
            _ => "version.Data",
        },
        TYPE_SHUFFLING => "version.Shuffling",
        TYPE_ALIASES => match subtype {
            SUBTYPE_ALIASES_ALIAS_ASSIGNMENT => "version.AliasAssignment",
            SUBTYPE_ALIASES_ALIAS_SELL => "version.AliasSell",
            SUBTYPE_ALIASES_ALIAS_BUY => "version.AliasBuy",
            SUBTYPE_ALIASES_ALIAS_DELETE => "version.AliasDelete",
            _ => "version.Aliases",
        },
        TYPE_VOTING => match subtype {
            SUBTYPE_VOTING_POLL_CREATION => "version.PollCreation",
            SUBTYPE_VOTING_VOTE_CASTING => "version.VoteCasting",
            SUBTYPE_VOTING_PHASING_VOTE_CASTING => "version.PhasingVoteCasting",
            _ => "version.Voting",
        },
        TYPE_ACCOUNT_PROPERTY => match subtype {
            SUBTYPE_ACCOUNT_PROPERTY_SET => "version.AccountPropertySet",
            _ => "version.AccountProperty",
        },
        TYPE_COIN_EXCHANGE => match subtype {
            SUBTYPE_COIN_EXCHANGE_ORDER_ISSUE => "version.CoinExchangeOrderIssue",
            SUBTYPE_COIN_EXCHANGE_ORDER_CANCEL => "version.CoinExchangeOrderCancel",
            _ => "version.CoinExchange",
        },
        TYPE_LIGHT_CONTRACT => match subtype {
            SUBTYPE_LIGHT_CONTRACT_REFERENCE_SET => "version.ContractReferenceSet",
            SUBTYPE_LIGHT_CONTRACT_REFERENCE_DELETE => "version.ContractReferenceDelete",
            _ => "version.LightContract",
        },
        _ => "version.Unknown",
    };

    att_map.get(version_key)
        .and_then(|v| v.as_u64())
        .map(|v| v as u8)
        .unwrap_or(1)
}

/// 获取 appendix 的版本号
fn get_appendix_version(version_key: &str, att_map: &Map<String, serde_json::Value>) -> u8 {
    att_map.get(version_key)
        .and_then(|v| v.as_u64())
        .map(|v| v as u8)
        .unwrap_or(1)
}

/// 从 JSON attachment 对象构建完整的二进制 attachment_bytes
///
/// 完整结构：
/// [Attachment部分] [Message部分] [EncryptedMessage部分] [PublicKeyAnnouncement部分]
/// [EncryptToSelfMessage部分] [Phasing部分] [PrunablePlainMessage部分] [PrunableEncryptedMessage部分]
pub fn build_attachment_bytes_from_json(
    type_byte: u8,
    subtype: u8,
    _version: u8,
    att_obj: Option<&Map<String, serde_json::Value>>,
) -> Vec<u8> {
    let mut result = Vec::new();

    let att_map = match att_obj {
        Some(m) => m,
        None => return result,
    };

    // === 1. 序列化 Attachment 部分 ===
    // 从 JSON 中读取 attachment 的版本号
    let att_version = get_attachment_version(type_byte, subtype, att_map);
    let att_bytes = serialize_attachment(type_byte, subtype, att_version, att_obj);
    if !att_bytes.is_empty() {
        put_version_and_data(&mut result, att_version, |buf| {
            put_bytes(buf, &att_bytes);
        });
    }

    // === 2. 序列化各 Appendix 部分（从 attachment JSON 中检测） ===

    // Message appendix（对应 Java: AppendixMessage）
    if let Some(msg_val) = att_map.get("message") {
        let msg_version = get_appendix_version("version.Message", att_map);
        let message_str = msg_val.as_str().unwrap_or("");
        let message_bytes = message_str.as_bytes();
        let is_text = true;
        let len_with_flag = if is_text {
            (message_bytes.len() as i32) | (0x80000000u32 as i32)
        } else {
            message_bytes.len() as i32
        };
        put_version_and_data(&mut result, msg_version, |buf| {
            put_i32(buf, len_with_flag);
            put_bytes(buf, message_bytes);
        });
    }

    // EncryptedMessage appendix（对应 Java: EncryptedMessage）
    if let Some(enc_msg) = att_map.get("encryptedMessage") {
        if let Some(enc_obj) = enc_msg.as_object() {
            let enc_version = get_appendix_version("version.EncryptedMessage", att_map);
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
            put_version_and_data(&mut result, enc_version, |buf| {
                put_i32(buf, len_with_flag);
                put_bytes(buf, &data_hex);
                put_bytes(buf, &nonce_hex);
            });
        }
    }

    // PublicKeyAnnouncement appendix（对应 Java: PublicKeyAnnouncement）
    // 注意：Java 中顺序是 message -> encryptedMessage -> publicKeyAnnouncement -> encryptToSelfMessage
    if let Some(pk_val) = att_map.get("recipientPublicKey") {
        if let Some(pk_str) = pk_val.as_str() {
            if let Ok(pk_bytes) = hex::decode(pk_str) {
                let pk_version = get_appendix_version("version.PublicKeyAnnouncement", att_map);
                put_version_and_data(&mut result, pk_version, |buf| {
                    put_bytes(buf, &pk_bytes);
                });
            }
        }
    }

    // EncryptToSelfMessage appendix（对应 Java: EncryptToSelfMessage）
    if let Some(ets_msg) = att_map.get("encryptToSelfMessage") {
        if let Some(ets_obj) = ets_msg.as_object() {
            let ets_version = get_appendix_version("version.EncryptToSelfMessage", att_map);
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
            put_version_and_data(&mut result, ets_version, |buf| {
                put_i32(buf, len_with_flag);
                put_bytes(buf, &data_hex);
                put_bytes(buf, &nonce_hex);
            });
        }
    }

    // Phasing appendix（对应 Java: AppendixPhasing）
    // 检测条件: 有 phasingFinishHeight 或 phased=true
    if att_map.get("phasingFinishHeight").is_some()
        || att_map.get("phased").and_then(|v| v.as_bool()).unwrap_or(false)
    {
        let phasing_version = get_appendix_version("version.Phasing", att_map);
        serialize_phasing_appendix(&mut result, phasing_version, att_map);
    }

    // PrunablePlainMessage appendix（对应 Java: PrunablePlainMessage）
    // Java 源码 (PrunablePlainMessage.java:106-108): buffer.put(getHash())
    // JSON 中使用 "messageHash" 字段存储 hash
    // 检测方式：有 "messageHash" 字段 或 有 "version.PrunablePlainMessage" 字段
    let ppm_hash = att_map.get("messageHash")
        .and_then(|v| v.as_str())
        .and_then(|s| hex::decode(s).ok());

    if let Some(hash_bytes) = ppm_hash {
        let ppm_version = get_appendix_version("version.PrunablePlainMessage", att_map);
        put_version_and_data(&mut result, ppm_version, |buf| {
            put_bytes(buf, &hash_bytes);
        });
    } else if att_map.get("version.PrunablePlainMessage").is_some() {
        let ppm_version = get_appendix_version("version.PrunablePlainMessage", att_map);
        put_version_and_data(&mut result, ppm_version, |buf| {
            put_bytes(buf, &[0u8; 32]);
        });
    }

    // PrunableEncryptedMessage appendix（对应 Java: PrunableEncryptedMessage）
    // JSON 中使用 "encryptedMessageHash" 字段存储 hash
    let pem_hash = att_map.get("encryptedMessageHash")
        .and_then(|v| v.as_str())
        .and_then(|s| hex::decode(s).ok());

    if let Some(hash_bytes) = pem_hash {
        let pem_version = get_appendix_version("version.PrunableEncryptedMessage", att_map);
        put_version_and_data(&mut result, pem_version, |buf| {
            put_bytes(buf, &hash_bytes);
        });
    } else if att_map.get("version.PrunableEncryptedMessage").is_some() {
        let pem_version = get_appendix_version("version.PrunableEncryptedMessage", att_map);
        put_version_and_data(&mut result, pem_version, |buf| {
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
///   + [如果 votingModel==TRANSACTION: linkedTransactionsIds]
///   + [如果 votingModel==HASH: hashedSecret + algorithm]
///   + [如果 votingModel==COMPOSITE: compositeVoting]
///   + [如果 votingModel==PROPERTY: senderPropertyVoting + recipientPropertyVoting]
fn serialize_phasing_appendix(buf: &mut Vec<u8>, version: u8, att_map: &Map<String, serde_json::Value>) {
    put_version_and_data(buf, version, |buf| {
        // finishHeight (4 bytes, i32 LE)
        let finish_height = if let Some(val) = att_map.get("phasingFinishHeight") {
            parse_i32(val)
        } else {
            0
        };
        put_i32(buf, finish_height);

        // === PhasingParams ===
        // votingModel (1 byte)
        let voting_model = att_map.get("phasingVotingModel")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i8;

        put_byte(buf, voting_model as u8);

        // quorum (8 bytes, i64 LE - Java long)
        let quorum = parse_u64_as_i64(att_map.get("phasingQuorum").unwrap_or(&serde_json::Value::Null));
        put_i64(buf, quorum);

        // minBalance (8 bytes, i64 LE - Java long)
        let min_balance = parse_u64_as_i64(att_map.get("phasingMinBalance").unwrap_or(&serde_json::Value::Null));
        put_i64(buf, min_balance);

        // whitelist (1 byte count + 8 bytes each account ID)
        if let Some(whitelist_arr) = att_map.get("phasingWhitelist").and_then(|v| v.as_array()) {
            put_byte(buf, whitelist_arr.len() as u8);
            for account_val in whitelist_arr {
                let account_id = account_val.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| account_val.as_u64())
                    .unwrap_or(0);
                put_u64(buf, account_id);
            }
        } else {
            put_byte(buf, 0);
        }

        // holdingId (8 bytes, i64 LE)
        let holding_id = parse_u64_as_i64(att_map.get("phasingHolding").unwrap_or(&serde_json::Value::Null));
        put_i64(buf, holding_id);

        // minBalanceModel (1 byte)
        let min_balance_model = att_map.get("phasingMinBalanceModel")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as u8;
        put_byte(buf, min_balance_model);

        // VotingModel::TRANSACTION = 4
        // 对应 Java: PhasingParams.putMyBytes() 第 224-228 行
        if voting_model == 4 {
            if let Some(linked_txs) = att_map.get("phasingLinkedTransactions").and_then(|v| v.as_array()) {
                put_byte(buf, linked_txs.len() as u8);
                for tx_obj in linked_txs {
                    if let Some(tx) = tx_obj.as_object() {
                        // ChainTransactionId: chain(4B) + fullHash(32B)
                        if let Some(chain) = tx.get("chain") {
                            put_i32(buf, chain.as_i64().unwrap_or(0) as i32);
                        } else {
                            put_i32(buf, 0);
                        }
                        if let Some(full_hash) = tx.get("fullHash").and_then(|h| h.as_str()) {
                            if let Ok(hash_bytes) = hex::decode(full_hash) {
                                put_bytes(buf, &hash_bytes);
                            } else {
                                put_bytes(buf, &[0u8; 32]);
                            }
                        } else {
                            put_bytes(buf, &[0u8; 32]);
                        }
                    }
                }
            } else {
                put_byte(buf, 0);
            }
        }

        // VotingModel::HASH = 5
        // 对应 Java: PhasingParams.putMyBytes() 第 229-231 行
        if voting_model == 5 {
            if let Some(hashed_secret) = att_map.get("phasingHashedSecret").and_then(|h| h.as_str()) {
                if let Ok(secret_bytes) = hex::decode(hashed_secret) {
                    put_byte(buf, secret_bytes.len() as u8);
                    put_bytes(buf, &secret_bytes);
                } else {
                    put_byte(buf, 0);
                }
            } else {
                put_byte(buf, 0);
            }
            if let Some(algorithm) = att_map.get("phasingHashedSecretAlgorithm") {
                put_byte(buf, algorithm.as_i64().unwrap_or(0) as u8);
            } else {
                put_byte(buf, 0);
            }
        }

        // VotingModel::COMPOSITE = 6 和 PROPERTY = 7 暂时简化处理
        // 实际使用时需要根据具体需求完善

        // === AppendixPhasing 额外字段 ===
        // linkedFullHashes (1 byte count + 32 bytes each hash)
        // 对应 Java: AppendixPhasing.putMyBytes() 第 136-139 行
        if let Some(linked_hashes) = att_map.get("phasingLinkedFullHashes").and_then(|v| v.as_array()) {
            put_byte(buf, linked_hashes.len() as u8);
            for hash_val in linked_hashes {
                if let Some(hash_str) = hash_val.as_str() {
                    if let Ok(hash_bytes) = hex::decode(hash_str) {
                        put_bytes(buf, &hash_bytes);
                    } else {
                        put_bytes(buf, &[0u8; 32]);
                    }
                }
            }
        } else {
            put_byte(buf, 0);
        }

        // hashedSecret (1 byte length + data) - AppendixPhasing 级别
        // 对应 Java: AppendixPhasing.putMyBytes() 第 140-142 行
        if let Some(hashed_secret) = att_map.get("phasingHashedSecret").and_then(|h| h.as_str()) {
            if let Ok(secret_bytes) = hex::decode(hashed_secret) {
                put_byte(buf, secret_bytes.len() as u8);
                put_bytes(buf, &secret_bytes);
            } else {
                put_byte(buf, 0);
            }
        } else {
            put_byte(buf, 0);
        }

        // algorithm (1 byte)
        if let Some(algorithm) = att_map.get("phasingHashedSecretAlgorithm") {
            put_byte(buf, algorithm.as_i64().unwrap_or(0) as u8);
        } else {
            put_byte(buf, 0);
        }
    });
}

/// 检测 attachment JSON 中是否有指定字段，返回各 appendix 标志
///
/// 返回值顺序：(has_message, has_encrypted_message, has_public_key_announcement,
///              has_encrypttoself_message, has_phasing, has_prunable_message, has_prunable_encrypted_message)
pub fn detect_appendix_flags(
    att_obj: Option<&Map<String, serde_json::Value>>,
    type_id: u8,
    subtype: u8,
) -> (bool, bool, bool, bool, bool, bool, bool, bool) {
    let mut has_message = false;
    let mut has_encrypted_message = false;
    let mut has_public_key_announcement = false;
    let mut has_encrypttoself_message = false;
    let mut has_phasing = false;
    let mut has_prunable_message = false;
    let mut has_prunable_encrypted_message = false;
    let mut has_tagged_data_upload = false;

    let att_map = match att_obj {
        Some(m) => m,
        None => return (false, false, false, false, false, false, false, false),
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

    // 对应 Java IPrunable 接口的其他实现：
    // - TaggedDataUpload (type=6, subtype=0): 检测 "version.TaggedDataUpload"
    // - TaggedDataExtendAttachment (type=6, subtype=1): 检测 "version.TaggedDataExtend"
    // - ShufflingProcessing (type=7): 检测 "version.Shuffling"
    //
    // Java 源码参考:
    // - AbstractAppendix.hasAppendix(): return attachmentData.get("version." + appendixName) != null
    // - Transaction.newTransactionBuilder(JSONObject): 调用各附录的 parse() 方法
    // - getPrunableAttachmentJSON(): 遍历 appendages，筛选 instanceof IPrunable 的附录
    match (type_id, subtype) {
        (TYPE_DATA, SUBTYPE_DATA_TAGGED_DATA_UPLOAD) => {
            has_tagged_data_upload = att_map.get("version.TaggedDataUpload").is_some()
                || att_map.get("hash").is_some()
                || att_map.get("data").is_some();
        }
        (TYPE_DATA, SUBTYPE_DATA_TAGGED_DATA_EXTEND) => {
            has_tagged_data_upload = att_map.get("version.TaggedDataExtend").is_some()
                || att_map.get("taggedData").is_some();
        }
        (TYPE_SHUFFLING, _) => {
            has_tagged_data_upload = att_map.get("version.Shuffling").is_some();
        }
        _ => {}
    }

    (
        has_message,
        has_encrypted_message,
        has_public_key_announcement,
        has_encrypttoself_message,
        has_phasing,
        has_prunable_message,
        has_prunable_encrypted_message,
        has_tagged_data_upload,
    )
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

/// 从 JSON Value 中解析可能超出 i64 范围的 u64 值
/// Java long 是有符号的，但 JSON API 返回无符号字符串
/// 字节表示上 i64 和 u64 是相同的（8 字节 LE）
fn parse_u64_as_i64(val: &serde_json::Value) -> i64 {
    val.as_str()
        .and_then(|s| s.parse::<u64>().ok().map(|u| u as i64))
        .or_else(|| val.as_u64().map(|u| u as i64))
        .or_else(|| val.as_i64())
        .unwrap_or(0)
}

fn parse_i32(val: &serde_json::Value) -> i32 {
    val.as_str()
        .and_then(|s| s.parse::<i32>().ok())
        .or_else(|| val.as_i64().map(|i| i as i32))
        .unwrap_or(0)
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
                put_i64(&mut buf, parse_u64_as_i64(val));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(val) = att_map.get("reserveSupply") {
                put_i64(&mut buf, parse_u64_as_i64(val));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(val) = att_map.get("maxSupply") {
                put_i64(&mut buf, parse_u64_as_i64(val));
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
                put_i64(&mut buf, parse_u64_as_i64(val));
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
                put_i64(&mut buf, parse_u64_as_i64(cur));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(amt) = att_map.get("amountPerUnitNQT") {
                put_i64(&mut buf, parse_u64_as_i64(amt));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_MONETARY_SYSTEM_RESERVE_CLAIM => {
            // 对应 Java: MonetarySystemReserveClaim.putMyBytes()
            // Java: buffer.putLong(currencyId) + buffer.putLong(units)
            if let Some(cur) = att_map.get("currency") {
                put_i64(&mut buf, parse_u64_as_i64(cur));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(units) = att_map.get("units") {
                put_i64(&mut buf, parse_u64_as_i64(units));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_MONETARY_SYSTEM_CURRENCY_TRANSFER => {
            // 对应 Java: MonetarySystemCurrencyTransfer.putMyBytes()
            // Java: buffer.putLong(currencyId) + buffer.putLong(units)
            if let Some(cur) = att_map.get("currency") {
                put_i64(&mut buf, parse_u64_as_i64(cur));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(units) = att_map.get("units") {
                put_i64(&mut buf, parse_u64_as_i64(units));
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
                put_i64(&mut buf, parse_u64_as_i64(cur));
            } else {
                put_i64(&mut buf, 0);
            }
            for field in &["buyRateNQT", "sellRateNQT", "totalBuyLimit", "totalSellLimit",
                           "initialBuySupply", "initialSellSupply"] {
                if let Some(val) = att_map.get(*field) {
                    put_i64(&mut buf, parse_u64_as_i64(val));
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
                put_i64(&mut buf, parse_u64_as_i64(cur));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(rate) = att_map.get("rateNQT") {
                put_i64(&mut buf, parse_u64_as_i64(rate));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(units) = att_map.get("units") {
                put_i64(&mut buf, parse_u64_as_i64(units));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_MONETARY_SYSTEM_CURRENCY_MINTING => {
            // 对应 Java: MonetarySystemCurrencyMinting.putMyBytes()
            // Java: buffer.putLong(nonce) + buffer.putLong(currencyId) + buffer.putLong(units) + buffer.putLong(counter)
            if let Some(nonce) = att_map.get("nonce") {
                put_i64(&mut buf, parse_u64_as_i64(nonce));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(cur) = att_map.get("currency") {
                put_i64(&mut buf, parse_u64_as_i64(cur));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(units) = att_map.get("units") {
                put_i64(&mut buf, parse_u64_as_i64(units));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(counter) = att_map.get("counter") {
                put_i64(&mut buf, parse_u64_as_i64(counter));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_MONETARY_SYSTEM_CURRENCY_DELETION => {
            // 对应 Java: MonetarySystemCurrencyDeletion.putMyBytes()
            // Java: buffer.putLong(currencyId)
            if let Some(cur) = att_map.get("currency") {
                put_i64(&mut buf, parse_u64_as_i64(cur));
            } else {
                put_i64(&mut buf, 0);
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
            // JSON 字段名: "taggedData" (不是 "taggedDataId")
            if let Some(tagged_data) = att_map.get("taggedData") {
                put_i64(&mut buf, parse_u64_as_i64(tagged_data));
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
            
            // 支持多种字段名: contractName 或 name
            let name = att_map.get("contractName")
                .and_then(|v| v.as_str())
                .or_else(|| att_map.get("name").and_then(|v| v.as_str()))
                .unwrap_or("");
            
            if !name.is_empty() {
                put_byte(&mut buf, name.as_bytes().len() as u8);
                put_string(&mut buf, name);
            } else {
                put_byte(&mut buf, 0);
            }
            
            // 支持多种字段名: contractParams 或 params
            let params = att_map.get("contractParams")
                .and_then(|v| v.as_str())
                .or_else(|| att_map.get("params").and_then(|v| v.as_str()))
                .unwrap_or("");
            
            if !params.is_empty() {
                put_byte(&mut buf, params.as_bytes().len() as u8);
                put_string(&mut buf, params);
            } else {
                put_byte(&mut buf, 0);
            }
            
            // ChainTransactionId: chainId(i32) + hash(32B or 24B)
            // 支持多种格式:
            // 1. contract.transactionFullHash (嵌套对象)
            // 2. transactionFullHash (顶层字段)
            // 3. hash (顶层字段)
            // 4. referencedTransactionFullHash (顶层字段)
            let hash_bytes = if let Some(contract_obj) = att_map.get("contract").and_then(|v| v.as_object()) {
                // 格式1: 嵌套对象
                if let Some(hash_str) = contract_obj.get("transactionFullHash")
                    .or_else(|| contract_obj.get("hash"))
                    .and_then(|v| v.as_str())
                {
                    hex::decode(hash_str).unwrap_or_default()
                } else {
                    Vec::new()
                }
            } else if let Some(hash_str) = att_map.get("transactionFullHash")
                .or_else(|| att_map.get("hash"))
                .or_else(|| att_map.get("referencedTransactionFullHash"))
                .and_then(|v| v.as_str())
            {
                // 格式2/3/4: 顶层字段
                hex::decode(hash_str).unwrap_or_default()
            } else {
                Vec::new()
            };
            
            // chainId - 支持多种字段位置
            let chain_id = if let Some(contract_obj) = att_map.get("contract").and_then(|v| v.as_object()) {
                contract_obj.get("chain")
                    .or_else(|| contract_obj.get("chainId"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32
            } else {
                att_map.get("chain")
                    .or_else(|| att_map.get("chainId"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32
            };
            
            put_i32(&mut buf, chain_id);
            
            // 写入 hash（Java NRCS 存储时截断到 24 字节）
            // 注意：实际观察到的数据表明 hash 可能是 24 字节或 32 字节
            // 为了与数据库兼容，统一截断到 24 字节
            if !hash_bytes.is_empty() {
                let truncated_hash = if hash_bytes.len() > 24 {
                    &hash_bytes[..24]
                } else {
                    &hash_bytes
                };
                put_bytes(&mut buf, truncated_hash);
            } else {
                // 默认写入 24 字节的零（与数据库格式一致）
                put_bytes(&mut buf, &[0u8; 24]);
            }
        }
        SUBTYPE_LIGHT_CONTRACT_REFERENCE_DELETE => {
            // 对应 Java: ContractReferenceDeleteAttachment.putMyBytes()
            // Java 源码 (ContractReferenceDeleteAttachment.java:39-41):
            //   buffer.putLong(contractReferenceId);  // 8 bytes
            // JSON 字段名: "contractReference" (不是 "contractReferenceId")
            if let Some(ref_id) = att_map.get("contractReference") {
                put_i64(&mut buf, parse_u64_as_i64(ref_id));
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
                put_i64(&mut buf, parse_u64_as_i64(prop));
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
                put_i64(&mut buf, parse_u64_as_i64(price));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_DIGITAL_GOODS_DELISTING | SUBTYPE_DIGITAL_GOODS_FEEDBACK => {
            let key = if subtype == SUBTYPE_DIGITAL_GOODS_DELISTING { "goods" } else { "purchase" };
            if let Some(id) = att_map.get(key) {
                put_i64(&mut buf, parse_u64_as_i64(id));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_DIGITAL_GOODS_PRICE_CHANGE => {
            if let Some(goods) = att_map.get("goods") {
                put_i64(&mut buf, parse_u64_as_i64(goods));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(price) = att_map.get("priceNQT") {
                put_i64(&mut buf, parse_u64_as_i64(price));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_DIGITAL_GOODS_QUANTITY_CHANGE => {
            if let Some(goods) = att_map.get("goods") {
                put_i64(&mut buf, parse_u64_as_i64(goods));
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
                put_i64(&mut buf, parse_u64_as_i64(goods));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(qty) = att_map.get("quantity") {
                put_i32(&mut buf, qty.as_i64().unwrap_or(0) as i32);
            } else {
                put_i32(&mut buf, 0);
            }
            if let Some(price) = att_map.get("priceNQT") {
                put_i64(&mut buf, parse_u64_as_i64(price));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(deadline) = att_map.get("deliveryDeadlineTimestamp") {
                put_i32(&mut buf, parse_i32(deadline));
            } else {
                put_i32(&mut buf, 0);
            }
        }
        SUBTYPE_DIGITAL_GOODS_DELIVERY => {
            if let Some(purchase) = att_map.get("purchase") {
                put_i64(&mut buf, parse_u64_as_i64(purchase));
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
                put_i64(&mut buf, parse_u64_as_i64(discount));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_DIGITAL_GOODS_REFUND => {
            if let Some(purchase) = att_map.get("purchase") {
                put_i64(&mut buf, parse_u64_as_i64(purchase));
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
        put_i64(&mut buf, parse_u64_as_i64(shuffling));
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
                put_i64(&mut buf, parse_u64_as_i64(price));
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
                put_i64(&mut buf, parse_u64_as_i64(poll));
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
                put_i64(&mut buf, parse_u64_as_i64(qty));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(price) = att_map.get("priceNQTPerCoin") {
                put_i64(&mut buf, parse_u64_as_i64(price));
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
                put_i64(&mut buf, parse_u64_as_i64(mb));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(mbm) = att_map.get("minBalanceModel") {
                put_byte(&mut buf, mbm.as_u64().unwrap_or(0) as u8);
            } else {
                put_byte(&mut buf, 0);
            }
            if let Some(holding) = att_map.get("holding") {
                put_i64(&mut buf, parse_u64_as_i64(holding));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_MESSAGING_VOTE_CASTING => {
            // 对应 Java: MessagingVoteCasting.putMyBytes()
            // Java: buffer.putLong(pollId) + buffer.put(pollVote.length) + buffer.put(pollVote)
            if let Some(poll) = att_map.get("poll") {
                put_i64(&mut buf, parse_u64_as_i64(poll));
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
                put_i64(&mut buf, parse_u64_as_i64(fee));
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
                put_i64(&mut buf, parse_u64_as_i64(price));
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
        SUBTYPE_MESSAGING_ACCOUNT_PROPERTY_DELETE => {
            // 对应 Java: MessagingAccountPropertyDelete.putMyBytes()
            // Java: buffer.putLong(propertyId)
            if let Some(prop) = att_map.get("property") {
                put_i64(&mut buf, parse_u64_as_i64(prop));
            } else {
                put_i64(&mut buf, 0);
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
                put_i64(&mut buf, parse_u64_as_i64(qty));
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
                put_i64(&mut buf, parse_u64_as_i64(asset));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(qty) = att_map.get("quantityQNT") {
                put_i64(&mut buf, parse_u64_as_i64(qty));
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
                put_i64(&mut buf, parse_u64_as_i64(asset));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(qty) = att_map.get("quantityQQT").or_else(|| att_map.get("quantityQNT")) {
                put_i64(&mut buf, parse_u64_as_i64(qty));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(price) = att_map.get("priceNQT") {
                put_i64(&mut buf, parse_u64_as_i64(price));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_COLORED_COINS_ASK_ORDER_CANCELLATION |
        SUBTYPE_COLORED_COINS_BID_ORDER_CANCELLATION => {
            // 对应 Java: ColoredCoinsOrderCancellationAttachment.putMyBytes()
            // Java: buffer.putLong(orderId)
            if let Some(order) = att_map.get("order") {
                put_i64(&mut buf, parse_u64_as_i64(order));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_COLORED_COINS_DIVIDEND_PAYMENT => {
            // 对应 Java: ColoredCoinsDividendPayment.putMyBytes()
            // Java: buffer.putLong(assetId) + buffer.putInt(height) + buffer.putLong(amountNQTPerQNT)
            // 注意: height 用的是 putInt() 即 i32，不是 u32！
            if let Some(asset) = att_map.get("asset") {
                put_i64(&mut buf, parse_u64_as_i64(asset));
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
                put_i64(&mut buf, parse_u64_as_i64(amount));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_COLORED_COINS_ASSET_DELETE => {
            // 对应 Java: ColoredCoinsAssetDelete.putMyBytes()
            // Java: buffer.putLong(assetId) + buffer.putLong(quantityQNT)
            if let Some(asset) = att_map.get("asset") {
                put_i64(&mut buf, parse_u64_as_i64(asset));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(qty) = att_map.get("quantityQNT") {
                put_i64(&mut buf, parse_u64_as_i64(qty));
            } else {
                put_i64(&mut buf, 0);
            }
        }
        SUBTYPE_COLORED_COINS_ASSET_INCREASE => {
            // 对应 Java: AssetIncreaseAttachment (extends AssetQuantityAttachment)
            // Java: buffer.putLong(assetId) + buffer.putLong(quantityDeltaQNT)
            if let Some(asset) = att_map.get("asset") {
                put_i64(&mut buf, parse_u64_as_i64(asset));
            } else {
                put_i64(&mut buf, 0);
            }
            if let Some(qty) = att_map.get("quantityDeltaQNT") {
                put_i64(&mut buf, parse_u64_as_i64(qty));
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
                put_i64(&mut buf, parse_u64_as_i64(asset));
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
        SUBTYPE_ACCOUNT_CONTROL_PHASING_ONLY => {
            serialize_phasing_only_attachment(&mut buf, att_map);
        }
        _ => {}
    }

    buf
}

/// 序列化 SetPhasingOnly 附件
/// 对应 Java: SetPhasingOnly.putMyBytes()
/// 格式: PhasingParams + maxFees(8B) + minDuration(2B) + maxDuration(2B)
/// 
/// JSON 格式:
/// {
///   "phasingControlParams": {
///     "phasingVotingModel": ...,
///     "phasingQuorum": ...,
///     ...
///   },
///   "controlMaxFees": ...,
///   "controlMinDuration": ...,
///   "controlMaxDuration": ...
/// }
fn serialize_phasing_only_attachment(buf: &mut Vec<u8>, att_map: &Map<String, serde_json::Value>) {
    // 获取 phasingControlParams 嵌套对象
    let phasing_params_map = att_map.get("phasingControlParams")
        .and_then(|v| v.as_object())
        .map(|m| m as &Map<String, serde_json::Value>);
    
    // 序列化 PhasingParams
    if let Some(params_map) = phasing_params_map {
        serialize_phasing_params(buf, params_map);
    } else {
        // 如果没有 phasingControlParams，尝试直接从 att_map 读取（兼容旧格式）
        serialize_phasing_params(buf, att_map);
    }
    
    // maxFees (8 bytes, i64)
    if let Some(max_fees) = att_map.get("controlMaxFees") {
        put_i64(buf, parse_u64_as_i64(max_fees));
    } else {
        put_i64(buf, 0);
    }
    
    // minDuration (2 bytes, i16)
    if let Some(min_duration) = att_map.get("controlMinDuration") {
        put_i16(buf, min_duration.as_i64().unwrap_or(0) as i16);
    } else {
        put_i16(buf, 0);
    }
    
    // maxDuration (2 bytes, i16)
    if let Some(max_duration) = att_map.get("controlMaxDuration") {
        put_i16(buf, max_duration.as_i64().unwrap_or(0) as i16);
    } else {
        put_i16(buf, 0);
    }
}

/// 序列化 PhasingParams
/// 对应 Java: PhasingParams.putMyBytes()
/// 格式: votingModel(1B) + quorum(8B) + minBalance(8B) + whitelistLen(1B) + [accountId(8B)]... 
///       + holdingId(8B) + minBalanceModel(1B) + [额外字段根据votingModel]
fn serialize_phasing_params(buf: &mut Vec<u8>, att_map: &Map<String, serde_json::Value>) {
    let voting_model = att_map.get("phasingVotingModel")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i8;
    
    put_byte(buf, voting_model as u8);
    
    if let Some(quorum) = att_map.get("phasingQuorum") {
        put_i64(buf, parse_u64_as_i64(quorum));
    } else {
        put_i64(buf, 0);
    }
    
    if let Some(min_balance) = att_map.get("phasingMinBalance") {
        put_i64(buf, parse_u64_as_i64(min_balance));
    } else {
        put_i64(buf, 0);
    }
    
    if let Some(whitelist_arr) = att_map.get("phasingWhitelist").and_then(|v| v.as_array()) {
        put_byte(buf, whitelist_arr.len() as u8);
        for account_val in whitelist_arr {
            let account_id = account_val.as_str()
                .and_then(|s| s.parse::<u64>().ok())
                .or_else(|| account_val.as_u64())
                .unwrap_or(0);
            put_u64(buf, account_id);
        }
    } else {
        put_byte(buf, 0);
    }
    
    if let Some(holding) = att_map.get("phasingHolding") {
        put_i64(buf, parse_u64_as_i64(holding));
    } else {
        put_i64(buf, 0);
    }
    
    if let Some(min_balance_model) = att_map.get("phasingMinBalanceModel") {
        put_byte(buf, min_balance_model.as_i64().unwrap_or(0) as u8);
    } else {
        put_byte(buf, 0);
    }
    
    // VotingModel::TRANSACTION = 4
    if voting_model == 4 {
        if let Some(linked_txs) = att_map.get("phasingLinkedTransactions").and_then(|v| v.as_array()) {
            put_byte(buf, linked_txs.len() as u8);
            for tx_obj in linked_txs {
                if let Some(tx) = tx_obj.as_object() {
                    if let Some(chain) = tx.get("chain") {
                        put_i32(buf, chain.as_i64().unwrap_or(0) as i32);
                    } else {
                        put_i32(buf, 0);
                    }
                    if let Some(full_hash) = tx.get("fullHash").and_then(|h| h.as_str()) {
                        if let Ok(hash_bytes) = hex::decode(full_hash) {
                            put_bytes(buf, &hash_bytes);
                        } else {
                            put_bytes(buf, &[0u8; 32]);
                        }
                    } else {
                        put_bytes(buf, &[0u8; 32]);
                    }
                }
            }
        } else {
            put_byte(buf, 0);
        }
    }
    
    // VotingModel::HASH = 5
    if voting_model == 5 {
        if let Some(hashed_secret) = att_map.get("phasingHashedSecret").and_then(|h| h.as_str()) {
            if let Ok(secret_bytes) = hex::decode(hashed_secret) {
                put_byte(buf, secret_bytes.len() as u8);
                put_bytes(buf, &secret_bytes);
            } else {
                put_byte(buf, 0);
            }
        } else {
            put_byte(buf, 0);
        }
        if let Some(algorithm) = att_map.get("phasingHashedSecretAlgorithm") {
            put_byte(buf, algorithm.as_i64().unwrap_or(0) as u8);
        } else {
            put_byte(buf, 0);
        }
    }
    
    // VotingModel::COMPOSITE = 6 和 PROPERTY = 7 暂时简化处理
    // 实际使用时需要根据具体需求完善
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

        let (has_msg, has_enc, has_pk, has_ets, has_ph, has_pm, has_pem, _has_tdp) =
            detect_appendix_flags(Some(att_map), 0, 0);

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
        let (has_msg, has_enc, has_pk, has_ets, has_ph, has_pm, has_pem, _has_tdp) =
            detect_appendix_flags(Some(att_map), TYPE_COLORED_COINS, SUBTYPE_COLORED_COINS_ASSET_ISSUANCE);

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
        let (has_msg, has_enc, has_pk, has_ets, has_ph, has_pm, has_pem, _has_tdp) =
            detect_appendix_flags(Some(att_map), TYPE_MESSAGING, SUBTYPE_MESSAGING_PHASING_VOTE_CASTING);

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

    /// 测试真实交易数据：type=2(subtype=2) AskOrderPlacement（卖单挂单）
    ///
    /// 这笔交易来自 Java NRCS 区块高度 239，transactionIndex=0
    /// 关键特征：
    /// - type=2, subtype=2: ColoredCoins.AskOrderPlacement
    /// - version.AskOrderPlacement: 1
    /// - asset: "16132763665229324019"
    /// - quantityQNT: "1"
    /// - priceNQT: "1000000000000"
    /// - 期望 ATTACHMENT_BYTES: 01f39e52171417e3df01000000000000000010a5d4e8000000 (25 bytes)
    #[test]
    fn test_real_transaction_ask_order_placement() {
        let json_str = r#"{
            "version.AskOrderPlacement": 1,
            "quantityQNT": "1",
            "priceNQT": "1000000000000",
            "asset": "16132763665229324019"
        }"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        // === 1. 序列化 attachment_bytes ===
        let bytes = build_attachment_bytes_from_json(
            TYPE_COLORED_COINS,
            SUBTYPE_COLORED_COINS_ASK_ORDER_PLACEMENT,
            1,
            Some(att_map),
        );

        // 预期结构：
        // version(1B) + assetId(8B) + quantityQNT(8B) + priceNQT(8B) = 25 bytes
        assert_eq!(bytes.len(), 25, "Expected 25 bytes for ask order placement, got {}", bytes.len());

        // 验证 version
        assert_eq!(bytes[0], 1, "Version should be 1");

        // 验证 assetId (小端序)
        let asset_id = u64::from_le_bytes([
            bytes[1], bytes[2], bytes[3], bytes[4],
            bytes[5], bytes[6], bytes[7], bytes[8],
        ]);
        assert_eq!(asset_id, 16132763665229324019, "Asset ID mismatch");

        // 验证 quantityQNT (小端序)
        let quantity = u64::from_le_bytes([
            bytes[9], bytes[10], bytes[11], bytes[12],
            bytes[13], bytes[14], bytes[15], bytes[16],
        ]);
        assert_eq!(quantity, 1, "Quantity should be 1");

        // 验证 priceNQT (小端序)
        let price = u64::from_le_bytes([
            bytes[17], bytes[18], bytes[19], bytes[20],
            bytes[21], bytes[22], bytes[23], bytes[24],
        ]);
        assert_eq!(price, 1000000000000, "Price should be 1000000000000");

        // 验证完整的 hex 字符串
        let expected_hex = "01f39e52171417e3df01000000000000000010a5d4e8000000";
        assert_eq!(hex::encode(&bytes), expected_hex, "Attachment bytes mismatch");
    }

    /// 测试真实交易数据：type=2(subtype=6) DividendPayment（股息支付）
    ///
    /// 这笔交易来自 Java NRCS 区块高度 245
    /// 关键特征：
    /// - type=2, subtype=6: ColoredCoins.DividendPayment
    /// - version.DividendPayment: 1
    /// - asset: "16132763665229324019"
    /// - height: 243
    /// - amountNQTPerQNT: "100000000"
    /// - 数据库 ATTACHMENT_BYTES: 01f39e52171417e3dff300000000e1f50500000000 (21 bytes)
    #[test]
    fn test_real_transaction_dividend_payment() {
        let json_str = r#"{
            "version.DividendPayment": 1,
            "amountNQTPerQNT": "100000000",
            "asset": "16132763665229324019",
            "height": 243
        }"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        let bytes = build_attachment_bytes_from_json(
            TYPE_COLORED_COINS,
            SUBTYPE_COLORED_COINS_DIVIDEND_PAYMENT,
            1,
            Some(att_map),
        );

        // 预期结构：
        // version(1B) + assetId(8B) + height(4B) + amountNQTPerQNT(8B) = 21 bytes
        assert_eq!(bytes.len(), 21, "Expected 21 bytes for dividend payment, got {}", bytes.len());

        // 验证 version
        assert_eq!(bytes[0], 1, "Version should be 1");

        // 验证 assetId (小端序)
        let asset_id = u64::from_le_bytes([
            bytes[1], bytes[2], bytes[3], bytes[4],
            bytes[5], bytes[6], bytes[7], bytes[8],
        ]);
        assert_eq!(asset_id, 16132763665229324019, "Asset ID mismatch");

        // 验证 height (小端序 i32)
        let height = i32::from_le_bytes([
            bytes[9], bytes[10], bytes[11], bytes[12],
        ]);
        assert_eq!(height, 243, "Height should be 243");

        // 验证 amountNQTPerQNT (小端序)
        let amount = u64::from_le_bytes([
            bytes[13], bytes[14], bytes[15], bytes[16],
            bytes[17], bytes[18], bytes[19], bytes[20],
        ]);
        assert_eq!(amount, 100000000, "Amount should be 100000000");

        // 验证完整的 hex 字符串
        let expected_hex = "01f39e52171417e3dff300000000e1f50500000000";
        assert_eq!(hex::encode(&bytes), expected_hex, "Attachment bytes mismatch");
    }

    /// 测试真实交易数据：type=1(subtype=0) ArbitraryMessage + PrunableEncryptedMessage
    ///
    /// DB_ID=23, 数据库 ATTACHMENT_BYTES: 0120a26df7b6c94b475f3c20ce9bdb1ef8409f3079a0a5dc1b3cf246f485309c87 (33 bytes)
    /// 注意：version.ArbitraryMessage=0 意味着 ArbitraryMessage 没有数据（version=0 不写入）
    #[test]
    fn test_real_transaction_arbitrary_message_with_prunable_encrypted() {
        let json_str = r#"{
            "version.ArbitraryMessage": 0,
            "version.PrunableEncryptedMessage": 1,
            "encryptedMessageHash": "20a26df7b6c94b475f3c20ce9bdb1ef8409f3079a0a5dc1b3cf246f485309c87"
        }"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        let bytes = build_attachment_bytes_from_json(
            TYPE_MESSAGING,
            SUBTYPE_MESSAGING_ARBITRARY_MESSAGE,
            1,
            Some(att_map),
        );

        // 预期结构：只有 PrunableEncryptedMessage
        // version(1B) + hash(32B) = 33 bytes
        assert_eq!(bytes.len(), 33, "Expected 33 bytes, got {}", bytes.len());

        // 验证 version
        assert_eq!(bytes[0], 1, "Version should be 1");

        // 验证 hash
        let expected_hash = "20a26df7b6c94b475f3c20ce9bdb1ef8409f3079a0a5dc1b3cf246f485309c87";
        assert_eq!(&hex::encode(&bytes[1..33]), expected_hash, "Hash mismatch");
    }

    /// 测试真实交易数据：type=6(subtype=1) TaggedDataExtend
    ///
    /// DB_ID=43, 数据库 ATTACHMENT_BYTES: 0115930917a7e0eabd (9 bytes)
    #[test]
    fn test_real_transaction_tagged_data_extend() {
        let json_str = r#"{
            "version.TaggedDataExtend": 1,
            "taggedData": "13684997425969337109"
        }"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        let bytes = build_attachment_bytes_from_json(
            TYPE_DATA,
            SUBTYPE_DATA_TAGGED_DATA_EXTEND,
            1,
            Some(att_map),
        );

        // 预期结构：version(1B) + taggedDataId(8B) = 9 bytes
        assert_eq!(bytes.len(), 9, "Expected 9 bytes, got {}", bytes.len());

        // 验证 version
        assert_eq!(bytes[0], 1, "Version should be 1");

        // 验证 taggedDataId (小端序)
        let tagged_data_id = u64::from_le_bytes([
            bytes[1], bytes[2], bytes[3], bytes[4],
            bytes[5], bytes[6], bytes[7], bytes[8],
        ]);
        assert_eq!(tagged_data_id, 13684997425969337109, "TaggedDataId mismatch");

        // 验证完整的 hex 字符串
        let expected_hex = "0115930917a7e0eabd";
        assert_eq!(hex::encode(&bytes), expected_hex, "Attachment bytes mismatch");
    }

    /// 测试真实交易数据：type=0(subtype=0) OrdinaryPayment + PrunableEncryptedMessage
    ///
    /// DB_ID=55, 数据库 ATTACHMENT_BYTES: 0179f3221c559eaabf7d96163188bc256a05fa0907e29b1a3a6e487e706b6e9d93 (33 bytes)
    /// 注意：version.OrdinaryPayment=0 意味着 OrdinaryPayment 没有数据（version=0 不写入）
    #[test]
    fn test_real_transaction_ordinary_payment_with_prunable_encrypted() {
        let json_str = r#"{
            "version.OrdinaryPayment": 0,
            "version.PrunableEncryptedMessage": 1,
            "encryptedMessageHash": "79f3221c559eaabf7d96163188bc256a05fa0907e29b1a3a6e487e706b6e9d93"
        }"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        let bytes = build_attachment_bytes_from_json(
            TYPE_PAYMENT,
            SUBTYPE_PAYMENT_ORDINARY_PAYMENT,
            1,
            Some(att_map),
        );

        // 预期结构：只有 PrunableEncryptedMessage
        // version(1B) + hash(32B) = 33 bytes
        assert_eq!(bytes.len(), 33, "Expected 33 bytes, got {}", bytes.len());

        // 验证 version
        assert_eq!(bytes[0], 1, "Version should be 1");

        // 验证 hash
        let expected_hash = "79f3221c559eaabf7d96163188bc256a05fa0907e29b1a3a6e487e706b6e9d93";
        assert_eq!(&hex::encode(&bytes[1..33]), expected_hash, "Hash mismatch");
    }

    /// 测试真实交易数据：type=2(subtype=10) AssetPropertySet + PublicKeyAnnouncement
    ///
    /// DB_ID=58, 数据库 ATTACHMENT_BYTES: 01f39e52171417e3df026e6f06313233343536012d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c (57 bytes)
    #[test]
    fn test_real_transaction_asset_property_set_with_public_key() {
        let json_str = r#"{
            "version.AssetProperty": 1,
            "asset": "16132763665229324019",
            "property": "no",
            "value": "123456",
            "version.PublicKeyAnnouncement": 1,
            "recipientPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c"
        }"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        let bytes = build_attachment_bytes_from_json(
            TYPE_COLORED_COINS,
            SUBTYPE_COLORED_COINS_PROPERTY_SET,
            1,
            Some(att_map),
        );

        // 预期结构：
        // AssetPropertySet: version(1B) + assetId(8B) + property(var) + value(var)
        // PublicKeyAnnouncement: version(1B) + publicKey(32B)
        // = 1 + 8 + (1+2) + (1+6) + 1 + 32 = 52 bytes
        // 但数据库是 57 bytes，让我重新计算...
        // 实际上 property 和 value 使用的是 SHORT 前缀 (2 bytes)，不是 1 byte
        // 所以：1 + 8 + (2+2) + (2+6) + 1 + 32 = 54 bytes? 还是不对
        
        // 让我先验证实际长度
        println!("Generated bytes: {}", hex::encode(&bytes));
        println!("Length: {}", bytes.len());
        
        // 预期 hex
        let expected_hex = "01f39e52171417e3df026e6f06313233343536012d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c";
        assert_eq!(hex::encode(&bytes), expected_hex, "Attachment bytes mismatch");
    }

    /// 测试真实交易数据：type=12(subtype=1) ContractReferenceDelete
    ///
    /// DB_ID=65, 数据库 ATTACHMENT_BYTES: 01e5c6099a6a80f8e0 (9 bytes)
    #[test]
    fn test_real_transaction_contract_reference_delete() {
        let json_str = r#"{
            "version.ContractReferenceDelete": 1,
            "contractReference": "16210848054059321061"
        }"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        let bytes = build_attachment_bytes_from_json(
            TYPE_LIGHT_CONTRACT,
            SUBTYPE_LIGHT_CONTRACT_REFERENCE_DELETE,
            1,
            Some(att_map),
        );

        // 预期结构：version(1B) + contractReference(8B) = 9 bytes
        assert_eq!(bytes.len(), 9, "Expected 9 bytes, got {}", bytes.len());

        // 验证 version
        assert_eq!(bytes[0], 1, "Version should be 1");

        // 验证 contractReference (小端序)
        let contract_ref = u64::from_le_bytes([
            bytes[1], bytes[2], bytes[3], bytes[4],
            bytes[5], bytes[6], bytes[7], bytes[8],
        ]);
        assert_eq!(contract_ref, 16210848054059321061, "ContractReference mismatch");

        // 验证完整的 hex 字符串
        let expected_hex = "01e5c6099a6a80f8e0";
        assert_eq!(hex::encode(&bytes), expected_hex, "Attachment bytes mismatch");
    }

    /// 完整端到端测试：DB_ID=23 (type=1:0 ArbitraryMessage + PrunableEncryptedMessage)
    ///
    /// 验证完整交易解析流程，包括 flags 计算和 attachment_bytes 生成
    #[test]
    fn test_e2e_db_id_23_full_hash() {
        use crate::transaction::Transaction;
        
        let json_str = r#"{
            "type": 1,
            "subtype": 0,
            "timestamp": 16075,
            "deadline": 1440,
            "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
            "recipient": "14411432778108101696",
            "amountNQT": "0",
            "feeNQT": "100000000",
            "signature": "e2e54690c709d2af13302113ba7d4d57250f255d9fd40303c46e18accb474b0f39becfa2b4555046cfc41f4dbd19e818ae63f1d7b53c8e57cfdc185a96a93c75",
            "fullHash": "a546e8db59204007db3b0cb08312f001155a17140039b8aa51b8317d44248a82",
            "attachment": {
                "version.ArbitraryMessage": 0,
                "version.PrunableEncryptedMessage": 1,
                "encryptedMessageHash": "20a26df7b6c94b475f3c20ce9bdb1ef8409f3079a0a5dc1b3cf246f485309c87"
            },
            "ecBlockHeight": 0,
            "ecBlockId": "3488276486778630462",
            "version": 1,
            "blockTimestamp": 16083
        }"#;
        
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let tx = Transaction::from_json(&val).expect("Failed to parse transaction");
        
        // 调试输出各字段值
        println!("DB_ID=23 Debug:");
        println!("  has_message: {}", tx.has_message);
        println!("  has_encrypted_message: {}", tx.has_encrypted_message);
        println!("  has_public_key_announcement: {}", tx.has_public_key_announcement);
        println!("  has_encrypttoself_message: {}", tx.has_encrypttoself_message);
        println!("  phased: {}", tx.phased);
        println!("  has_prunable_attachment: {}", tx.has_prunable_attachment);
        println!("  has_prunable_encrypted_message: {}", tx.has_prunable_encrypted_message);
        
        // 验证 attachment_bytes
        assert_eq!(hex::encode(&tx.attachment_bytes), 
                   "0120a26df7b6c94b475f3c20ce9bdb1ef8409f3079a0a5dc1b3cf246f485309c87",
                   "Attachment bytes mismatch for DB_ID=23");
        
        // 验证 fullHash - 这是最关键的测试
        let expected_full_hash = "a546e8db59204007db3b0cb08312f001155a17140039b8aa51b8317d44248a82";
        println!("  calculated full_hash: {}", hex::encode(&tx.full_hash.0));
        println!("  expected full_hash: {}", expected_full_hash);
        assert_eq!(hex::encode(&tx.full_hash.0), expected_full_hash, 
                   "FullHash mismatch for DB_ID=23! This is the critical bug.");
    }

    /// 完整端到端测试：DB_ID=55 (type=0:0 OrdinaryPayment + PrunableEncryptedMessage)
    #[test]
    fn test_e2e_db_id_55_full_hash() {
        use crate::transaction::Transaction;
        
        let json_str = r#"{
            "type": 0,
            "subtype": 0,
            "timestamp": 126006,
            "deadline": 1440,
            "senderPublicKey": "ccb796af901297bfaf80113cd1f3e4e7e6adc45419c8339f5bac38295d65963e",
            "recipient": "996325769485053218",
            "amountNQT": "10000000000",
            "feeNQT": "100000000",
            "signature": "74f4f89bfe5d42d9f5368cb1167b3787190f903ce3b6935fea124c01c7f0b30458f04f0b71a4a3402fa803baeb52cd1a75b7fc72e0c42e31c1408b4fc838e4a5",
            "fullHash": "ca5d746307ed0cf1945e49e626a9d9ff180084b54bc267b703080af320e1ef57",
            "attachment": {
                "version.OrdinaryPayment": 0,
                "version.PrunableEncryptedMessage": 1,
                "encryptedMessageHash": "79f3221c559eaabf7d96163188bc256a05fa0907e29b1a3a6e487e706b6e9d93"
            },
            "ecBlockHeight": 0,
            "ecBlockId": "3488276486778630462",
            "version": 1,
            "blockTimestamp": 126070
        }"#;
        
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let tx = Transaction::from_json(&val).expect("Failed to parse transaction");
        
        // 验证 fullHash
        let expected_full_hash = "ca5d746307ed0cf1945e49e626a9d9ff180084b54bc267b703080af320e1ef57";
        println!("\nDB_ID=55 Debug:");
        println!("  has_prunable_attachment: {}", tx.has_prunable_attachment);
        println!("  has_prunable_encrypted_message: {}", tx.has_prunable_encrypted_message);
        println!("  attachment_bytes: {}", hex::encode(&tx.attachment_bytes));
        println!("  calculated full_hash: {}", hex::encode(&tx.full_hash.0));
        println!("  expected full_hash: {}", expected_full_hash);
        
        assert_eq!(hex::encode(&tx.full_hash.0), expected_full_hash, 
                   "FullHash mismatch for DB_ID=55!");
    }

    /// 模拟 P2P 同步场景：没有 fullHash 字段（Java getJSONObject 不返回 fullHash）
    /// 验证 Rust 能正确计算 fullHash
    #[test]
    fn test_e2e_p2p_no_fullhash_field() {
        use crate::transaction::Transaction;
        
        // 模拟 P2P GetNextBlocks 返回的交易格式（无 fullHash 字段）
        let json_str = r#"{
            "type": 1,
            "subtype": 0,
            "timestamp": 16075,
            "deadline": 1440,
            "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
            "recipient": "14411432778108101696",
            "amountNQT": "0",
            "feeNQT": "100000000",
            "signature": "e2e54690c709d2af13302113ba7d4d57250f255d9fd40303c46e18accb474b0f39becfa2b4555046cfc41f4dbd19e818ae63f1d7b53c8e57cfdc185a96a93c75",
            "attachment": {
                "version.ArbitraryMessage": 0,
                "version.PrunableEncryptedMessage": 1,
                "encryptedMessageHash": "20a26df7b6c94b475f3c20ce9bdb1ef8409f3079a0a5dc1b3cf246f485309c87"
            },
            "ecBlockHeight": 0,
            "ecBlockId": "3488276486778630462",
            "version": 1
        }"#;
        
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let tx = Transaction::from_json(&val).expect("Failed to parse transaction");
        
        // 详细诊断序列化输出
        let signing_bytes = tx.serialize_for_signing();
        let full_hash_bytes = tx.serialize_for_full_hash();
        
        println!("\n=== serialize_for_signing ({} bytes) ===", signing_bytes.len());
        println!("  type: {} (0x{:02x})", signing_bytes[0], signing_bytes[0]);
        println!("  version|subtype: {} (0x{:02x})", signing_bytes[1], signing_bytes[1]);
        println!("  timestamp: {} (LE i32)", i32::from_le_bytes([signing_bytes[2], signing_bytes[3], signing_bytes[4], signing_bytes[5]]));
        println!("  deadline: {} (LE i16)", i16::from_le_bytes([signing_bytes[6], signing_bytes[7]]));
        println!("  senderPublicKey: {}", hex::encode(&signing_bytes[8..40]));
        println!("  recipient: {} (LE u64)", u64::from_le_bytes([
            signing_bytes[40], signing_bytes[41], signing_bytes[42], signing_bytes[43],
            signing_bytes[44], signing_bytes[45], signing_bytes[46], signing_bytes[47]
        ]));
        println!("  amount: {}", i64::from_le_bytes([
            signing_bytes[48], signing_bytes[49], signing_bytes[50], signing_bytes[51],
            signing_bytes[52], signing_bytes[53], signing_bytes[54], signing_bytes[55]
        ]));
        println!("  fee: {}", i64::from_le_bytes([
            signing_bytes[56], signing_bytes[57], signing_bytes[58], signing_bytes[59],
            signing_bytes[60], signing_bytes[61], signing_bytes[62], signing_bytes[63]
        ]));
        println!("  refTxHash: {}", hex::encode(&signing_bytes[64..96]));
        println!("  zeroSignature pad: {} bytes @ offset 96", if signing_bytes.len() > 96 { signing_bytes.len() - 96 } else { 0 });

        let sig_end = 96 + 64;
        if signing_bytes.len() > sig_end {
            println!("  flags: {} (LE u32) @ offset {}", u32::from_le_bytes([
                signing_bytes[sig_end], signing_bytes[sig_end+1],
                signing_bytes[sig_end+2], signing_bytes[sig_end+3]
            ]), sig_end);
            println!("  ecBlockHeight: {} @ offset {}", u32::from_le_bytes([
                signing_bytes[sig_end+4], signing_bytes[sig_end+5],
                signing_bytes[sig_end+6], signing_bytes[sig_end+7]
            ]), sig_end + 4);
            println!("  ecBlockId: {} @ offset {}", u64::from_le_bytes([
                signing_bytes[sig_end+8], signing_bytes[sig_end+9],
                signing_bytes[sig_end+10], signing_bytes[sig_end+11],
                signing_bytes[sig_end+12], signing_bytes[sig_end+13],
                signing_bytes[sig_end+14], signing_bytes[sig_end+15]
            ]), sig_end + 8);
            let att_offset = sig_end + 16;
            println!("  attachmentBytes: {} bytes @ offset {}", signing_bytes.len() - att_offset, att_offset);
        }

        println!("\n=== Summary ===");
        println!("  serialize_for_signing == serialize_for_full_hash: {}", signing_bytes.len() == full_hash_bytes.len());
        println!("  total size: {} bytes", full_hash_bytes.len());
        println!("  calculated full_hash: {}", hex::encode(&tx.full_hash.0));
        println!("  expected full_hash: a546e8db59204007db3b0cb08312f001155a17140039b8aa51b8317d44248a82");
        
        // 预期值
        let expected_full_hash = "a546e8db59204007db3b0cb08312f001155a17140039b8aa51b8317d44248a82";
        assert_eq!(hex::encode(&tx.full_hash.0), expected_full_hash, 
                   "FullHash mismatch in P2P no-fullHash scenario!");
    }

    #[test]
    fn test_tagged_data_upload_attachment_bytes() {
        let json_str = r#"{
            "version.TaggedDataUpload": 1,
            "hash": "66d745082ab2b69563689c33cca9f2aea0867fcab96450f7343984cea3411ca8"
        }"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        let bytes = build_attachment_bytes_from_json(
            TYPE_DATA, SUBTYPE_DATA_TAGGED_DATA_UPLOAD, 1, Some(att_map)
        );

        let expected_hex = "0166d745082ab2b69563689c33cca9f2aea0867fcab96450f7343984cea3411ca8";
        let expected = hex::decode(expected_hex).unwrap();

        if bytes != expected {
            println!("Generated (hex): {}", hex::encode(&bytes));
            println!("Expected (hex):  {}", expected_hex);
            println!("Generated length: {}, Expected length: {}", bytes.len(), expected.len());
            for i in 0..std::cmp::max(bytes.len(), expected.len()) {
                let b = bytes.get(i).copied();
                let e = expected.get(i).copied();
                if b != e {
                    println!("  Byte[{}]: generated={:?}, expected={:?}", i, b, e);
                }
            }
        }

        assert_eq!(bytes, expected, "TaggedDataUpload attachment_bytes mismatch");
    }

    #[test]
    fn test_full_transaction_tagged_data_upload() {
        let json_str = r#"{
            "amountNQT":"0",
            "attachment":{
                "version.TaggedDataUpload":1,
                "hash":"66d745082ab2b69563689c33cca9f2aea0867fcab96450f7343984cea3411ca8"
            },
            "block":"9844141426725231932",
            "blockTimestamp":29593,
            "chainId":0,
            "deadline":15,
            "ecBlockHeight":0,
            "ecBlockId":"3488276486778630462",
            "feeNQT":"280000000",
            "fullHash":"3094de8870cc7d6711b849a075b0851457258fd224e338f2a0e1e7feb937ea30",
            "height":514,
            "phased":false,
            "sender":"996325769485053218",
            "senderPublicKey":"2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
            "signature":"c72f0ff5daeff1751e8fe6d00cf64b1148c63d134822c7d42945c97e2f3400030bcf7702826d53d93243f0901f6948275a9de1c7afa67dc04a3bc1fb605d53de",
            "subtype":0,
            "timestamp":29563,
            "transaction":"7457341341700101168",
            "transactionIndex":0,
            "type":6,
            "version":1
        }"#;

        let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let tx = crate::transaction::Transaction::from_json(&json).unwrap();

        let expected_attachment_hex = "0166d745082ab2b69563689c33cca9f2aea0867fcab96450f7343984cea3411ca8";
        let expected_attachment = hex::decode(expected_attachment_hex).unwrap();

        if tx.attachment_bytes != expected_attachment {
            println!("Generated attachment_bytes (hex): {}", hex::encode(&tx.attachment_bytes));
            println!("Expected attachment_bytes (hex):  {}", expected_attachment_hex);
            println!("Generated length: {}, Expected length: {}", tx.attachment_bytes.len(), expected_attachment.len());
        }

        assert_eq!(tx.attachment_bytes, expected_attachment, "Full transaction TaggedDataUpload attachment_bytes mismatch");
        assert_eq!(tx.id, 7457341341700101168u64, "Transaction ID mismatch");
        assert_eq!(tx.type_id, crate::transaction::TransactionType::Data, "Transaction type should be Data");
        assert_eq!(tx.subtype, 0, "Subtype should be 0 (TaggedDataUpload)");

        assert!(tx.attachment_json.is_some(), "attachment_json should be preserved");
        let att_json = tx.attachment_json.as_ref().unwrap();
        assert_eq!(att_json.get("hash").and_then(|v| v.as_str()), Some("66d745082ab2b69563689c33cca9f2aea0867fcab96450f7343984cea3411ca8"), "hash field in attachment_json mismatch");
    }

    #[test]
    fn test_tagged_data_extend_attachment_bytes() {
        let json_str = r#"{
            "version.TaggedDataExtend": 1,
            "taggedData": "13684997425969337109"
        }"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        let bytes = build_attachment_bytes_from_json(
            TYPE_DATA, SUBTYPE_DATA_TAGGED_DATA_EXTEND, 1, Some(att_map)
        );

        let expected_hex = "0115930917a7e0eabd";
        let expected = hex::decode(expected_hex).unwrap();

        if bytes != expected {
            println!("Generated (hex): {}", hex::encode(&bytes));
            println!("Expected (hex):  {}", expected_hex);
        }

        assert_eq!(hex::encode(&bytes), expected_hex, "TaggedDataExtend attachment_bytes mismatch");
    }

    #[test]
    fn test_payment_with_prunable_encrypted_message() {
        let json_str = r#"{
            "version.OrdinaryPayment": 0,
            "version.PrunableEncryptedMessage": 1,
            "encryptedMessageHash": "8cdca43d00adad301c3cca02274cc38e95564871c33d35dc1c5be04240c4f43c"
        }"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        let bytes = build_attachment_bytes_from_json(
            TYPE_PAYMENT, SUBTYPE_PAYMENT_ORDINARY_PAYMENT, 1, Some(att_map)
        );

        let expected_hex = "018cdca43d00adad301c3cca02274cc38e95564871c33d35dc1c5be04240c4f43c";
        let expected = hex::decode(expected_hex).unwrap();

        if bytes != expected {
            println!("Generated (hex): {}", hex::encode(&bytes));
            println!("Expected (hex):  {}", expected_hex);
        }

        assert_eq!(hex::encode(&bytes), expected_hex, "Payment with PrunableEncryptedMessage attachment_bytes mismatch");
    }

    #[test]
    fn test_messaging_with_prunable_plain_message() {
        let json_str = r#"{
            "version.ArbitraryMessage": 0,
            "version.PrunablePlainMessage": 1,
            "messageHash": "986433f798041860352547dd044a281bc66e1769875696ef91debcccd3781885"
        }"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        let bytes = build_attachment_bytes_from_json(
            TYPE_MESSAGING, SUBTYPE_MESSAGING_ARBITRARY_MESSAGE, 1, Some(att_map)
        );

        let expected_hex = "01986433f798041860352547dd044a281bc66e1769875696ef91debcccd3781885";
        let expected = hex::decode(expected_hex).unwrap();

        if bytes != expected {
            println!("Generated (hex): {}", hex::encode(&bytes));
            println!("Expected (hex):  {}", expected_hex);
        }

        assert_eq!(hex::encode(&bytes), expected_hex, "Messaging with PrunablePlainMessage attachment_bytes mismatch");
    }

    #[test]
    fn test_contract_reference_delete_attachment_bytes() {
        let json_str = r#"{
            "contractReference": "16210848054059321061",
            "version.ContractReferenceDelete": 1
        }"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        let bytes = build_attachment_bytes_from_json(
            TYPE_LIGHT_CONTRACT, SUBTYPE_LIGHT_CONTRACT_REFERENCE_DELETE, 1, Some(att_map)
        );

        let expected_hex = "01e5c6099a6a80f8e0";
        let expected = hex::decode(expected_hex).unwrap();

        if bytes != expected {
            println!("Generated (hex): {}", hex::encode(&bytes));
            println!("Expected (hex):  {}", expected_hex);
        }

        assert_eq!(hex::encode(&bytes), expected_hex, "ContractReferenceDelete attachment_bytes mismatch");
    }

    #[test]
    fn test_currency_issuance_attachment_bytes() {
        let json_str = r#"{
            "initialSupply": "1000000",
            "code": "NUSD",
            "minDifficulty": 1,
            "ruleset": 0,
            "description": "\u4e0eUSD\u4ef7\u683c\u951a\u5b9a\u7684\u79ef\u5206",
            "minReservePerUnitNQT": "0",
            "issuanceHeight": 0,
            "type": 51,
            "reserveSupply": "0",
            "version.CurrencyIssuance": 1,
            "maxDifficulty": 10,
            "decimals": 2,
            "name": "NUSD",
            "maxSupply": "10000000000",
            "algorithm": 2
        }"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        let bytes = build_attachment_bytes_from_json(
            TYPE_MONETARY_SYSTEM, SUBTYPE_MONETARY_SYSTEM_CURRENCY_ISSUANCE, 1, Some(att_map)
        );

        let expected_hex = "01044e555344044e5553441b00e4b88e555344e4bbb7e6a0bce9949ae5ae9ae79a84e7a7afe588863340420f0000000000000000000000000000e40b5402000000000000000000000000000000010a000202";
        let expected = hex::decode(expected_hex).unwrap();

        if bytes != expected {
            println!("Generated (hex): {}", hex::encode(&bytes));
            println!("Expected (hex):  {}", expected_hex);
            println!("Generated length: {}, Expected length: {}", bytes.len(), expected.len());
        }

        assert_eq!(hex::encode(&bytes), expected_hex, "CurrencyIssuance attachment_bytes mismatch");
    }

    #[test]
    fn test_colored_coins_asset_property_with_public_key() {
        let json_str = r#"{
            "version.PublicKeyAnnouncement": 1,
            "recipientPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
            "property": "no",
            "asset": "16132763665229324019",
            "version.AssetProperty": 1,
            "value": "123456"
        }"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        let bytes = build_attachment_bytes_from_json(
            TYPE_COLORED_COINS, SUBTYPE_COLORED_COINS_PROPERTY_SET, 1, Some(att_map)
        );

        let expected_hex = "01f39e52171417e3df026e6f06313233343536012d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c";
        let expected = hex::decode(expected_hex).unwrap();

        if bytes != expected {
            println!("Generated (hex): {}", hex::encode(&bytes));
            println!("Expected (hex):  {}", expected_hex);
            println!("Generated length: {}, Expected length: {}", bytes.len(), expected.len());
        }

        assert_eq!(hex::encode(&bytes), expected_hex, "AssetProperty with PublicKeyAnnouncement attachment_bytes mismatch");
    }

    #[test]
    fn test_e2e_transaction2_data_tagged_data_upload() {
        let json_str = r#"{
            "amountNQT": "0",
            "attachment": {
                "version.TaggedDataUpload": 1,
                "hash": "66d745082ab2b69563689c33cca9f2aea0867fcab96450f7343984cea3411ca8"
            },
            "block": "-8602602646984319684",
            "blockTimestamp": 29593,
            "deadline": 15,
            "ecBlockHeight": 0,
            "ecBlockId": "3488276486778630462",
            "feeNQT": "280000000",
            "fullHash": "3094de8870cc7d6711b849a075b0851457258fd224e338f2a0e1e7feb937ea30",
            "height": 514,
            "phased": false,
            "sender": "996325769485053218",
            "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
            "signature": "c72f0ff5daeff1751e8fe6d00cf64b1148c63d134822c7d42945c97e2f3400030bcf7702826d53d93243f0901f6948275a9d",
            "subtype": 0,
            "timestamp": 29563,
            "transaction": "7457341341700101168",
            "transactionIndex": 0,
            "type": 6,
            "version": 1
        }"#;

        let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let tx = crate::transaction::Transaction::from_json(&json).unwrap();

        println!("Parsed tx.id: {}", tx.id);
        println!("Expected id:  {}", 7457341341700101168u64);
        println!("Parsed full_hash: {}", hex::encode(&tx.full_hash.0));
        println!("Expected full_hash: 3094de8870cc7d6711b849a075b0851457258fd224e338f2a0e1e7feb937ea30");

        assert_eq!(tx.type_id, crate::transaction::TransactionType::Data);
        assert_eq!(tx.subtype, 0);

        let expected_att_hex = "0166d745082ab2b69563689c33cca9f2aea0867fcab96450f7343984cea3411ca8";
        assert_eq!(hex::encode(&tx.attachment_bytes), expected_att_hex, "attachment_bytes mismatch");

        assert!(tx.attachment_json.is_some(), "attachment_json should be preserved");
        let att = tx.attachment_json.as_ref().unwrap();
        assert_eq!(att.get("hash").and_then(|v| v.as_str()), Some("66d745082ab2b69563689c33cca9f2aea0867fcab96450f7343984cea3411ca8"));
    }

    #[test]
    fn test_e2e_asset_property_set_with_pk_announcement() {
        let json_str = r#"{
            "amountNQT": "0",
            "attachment": {
                "version.PublicKeyAnnouncement": 1,
                "recipientPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
                "property": "no",
                "asset": "16132763665229324019",
                "version.AssetProperty": 1,
                "value": "123456"
            },
            "block": "4807844805401117927",
            "blockTimestamp": 133506,
            "deadline": 15,
            "ecBlockHeight": 1647,
            "ecBlockId": "-3101292290248805829",
            "feeNQT": "100000000",
            "fullHash": "a82003fc9d935c59e9a62bacb8381f92398ec722a0f02550544d087c18080c89",
            "height": 2369,
            "phased": false,
            "recipient": "996325769485053218",
            "sender": "996325769485053218",
            "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
            "signature": "81c12bdb6a45b05252dc068a34d6212897848055da72d899be6819b9635fdc0f76c37ff5f19093a9d52c7cf24fcc5682fb25",
            "subtype": 10,
            "timestamp": 133354,
            "transaction": "6439183873980178600",
            "transactionIndex": 0,
            "type": 2,
            "version": 1
        }"#;

        let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let tx = crate::transaction::Transaction::from_json(&json).unwrap();

        println!("AssetProperty - Parsed tx.id: {}, expected: {}", tx.id, 6439183873980178600u64);
        println!("full_hash (from JSON): {}", hex::encode(&tx.full_hash.0));
        match tx.calculate_full_hash() {
            Ok(calculated) => println!("full_hash (calculated): {}", hex::encode(&calculated.0)),
            Err(e) => println!("calculate_full_hash error: {:?}", e),
        }
        println!("full_hash first 8 bytes: {}", hex::encode(&tx.full_hash.0[..8]));
        println!("calculate_id() result: {}", tx.calculate_id());
        println!("attachment_bytes: {}", hex::encode(&tx.attachment_bytes));
        println!("recipient_id: {:?}", tx.recipient_id);

        assert_eq!(tx.id, 6439183873980178600u64);
        assert_eq!(tx.type_id, crate::transaction::TransactionType::ColoredCoins);
        assert_eq!(tx.subtype, 10);
        assert_eq!(tx.recipient_id, Some(996325769485053218));

        let expected_att_hex = "01f39e52171417e3df026e6f06313233343536012d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c";
        assert_eq!(hex::encode(&tx.attachment_bytes), expected_att_hex, "attachment_bytes mismatch");

        assert!(tx.attachment_json.is_some());
        let att = tx.attachment_json.as_ref().unwrap();
        assert_eq!(att.get("property").and_then(|v| v.as_str()), Some("no"));
        assert_eq!(att.get("value").and_then(|v| v.as_str()), Some("123456"));
        assert_eq!(att.get("recipientPublicKey").and_then(|v| v.as_str()), Some("2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c"));
    }

    #[test]
    fn test_e2e_contract_reference_delete() {
        let json_str = r#"{
            "amountNQT": "0",
            "attachment": {
                "contractReference": "16210848054059321061",
                "version.ContractReferenceDelete": 1
            },
            "block": "-4050848184299982100",
            "blockTimestamp": 171769,
            "deadline": 15,
            "ecBlockHeight": 2338,
            "ecBlockId": "7386031426484071329",
            "feeNQT": "300000000",
            "fullHash": "0b9f62178466df745e2b673016991327e5cc961ebd803491e22477c505669c50",
            "height": 3057,
            "phased": false,
            "sender": "996325769485053218",
            "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
            "signature": "24ab6c187d676d8539cdfc1f1ae87ca050f0949b682add0680c18e275efc46069f937d40f486687c7a2282ea3e227e84e149",
            "subtype": 1,
            "timestamp": 171767,
            "transaction": "8421562545720172299",
            "transactionIndex": 0,
            "type": 12,
            "version": 1
        }"#;

        let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let tx = crate::transaction::Transaction::from_json(&json).unwrap();

        assert_eq!(tx.id, 8421562545720172299u64);
        assert_eq!(tx.type_id, crate::transaction::TransactionType::LightContract);
        assert_eq!(tx.subtype, 1);

        let expected_att_hex = "01e5c6099a6a80f8e0";
        assert_eq!(hex::encode(&tx.attachment_bytes), expected_att_hex, "attachment_bytes mismatch");

        assert!(tx.attachment_json.is_some());
        let att = tx.attachment_json.as_ref().unwrap();
        assert_eq!(att.get("contractReference").and_then(|v| v.as_str()), Some("16210848054059321061"));
    }

    /// 测试 TaggedDataUpload 的 has_prunable_attachment 检测
    ///
    /// 对应 transaction.data 中的记录：
    /// - DB_ID=53: type=6(Data), subtype=0(TaggedDataUpload), HAS_PRUNABLE_ATTACHMENT=TRUE
    /// - attachment_bytes = 01a1474a67570fbabbf... (33 bytes = version + hash)
    #[test]
    fn test_tagged_data_upload_prunable_detection() {
        // 模拟 TaggedDataUpload 的 JSON（有 hash 字段）
        let json_str = r#"{
            "version.TaggedDataUpload": 1,
            "hash": "a1474a67570fbabbf6794cb86400c119e0b23f4e69f464daea9e0fa93c5ef80c",
            "data": "test data"
        }"#;
        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let att_map = val.as_object().unwrap();

        // type=6 (Data), subtype=0 (TaggedDataUpload)
        let (_has_msg, _has_enc, _has_pk, _has_ets, _has_ph,
             _has_pm, _has_pem, has_tdp) =
            detect_appendix_flags(Some(att_map), TYPE_DATA, SUBTYPE_DATA_TAGGED_DATA_UPLOAD);

        assert!(has_tdp, "Should detect TaggedDataUpload as prunable attachment");

        // 测试没有 version.TaggedDataUpload 的情况
        let json_str2 = r#"{"message": "hello"}"#;
        let val2: serde_json::Value = serde_json::from_str(json_str2).unwrap();
        let att_map2 = val2.as_object().unwrap();

        let (_, _, _, _, _, _, _, has_tdp2) =
            detect_appendix_flags(Some(att_map2), TYPE_DATA, SUBTYPE_DATA_TAGGED_DATA_UPLOAD);

        assert!(!has_tdp2, "Should not detect prunable when no TaggedDataUpload fields");
    }

    /// 测试完整的 transaction.data DB_ID=53 记录
    ///
    /// 验证 type=6, subtype=0 的 TaggedDataUpload 交易能正确解析
    #[test]
    fn test_e2e_tagged_data_upload_transaction() {
        let json_str = r#"{
            "amountNQT": "0",
            "attachment": {
                "version.TaggedDataUpload": 1,
                "hash": "a1474a67570fbabbf6794cb86400c119e0b23f4e69f464daea9e0fa93c5ef80c"
            },
            "block": "-6444971140270725924",
            "blockTimestamp": 125757,
            "deadline": 15,
            "ecBlockHeight": 1506,
            "ecBlockId": "-7988487755785480949",
            "feeNQT": "260000000",
            "fullHash": "9193ff27cfae35775252126725132334af1397d58d784ee5064d636f244b425d",
            "height": 2227,
            "phased": false,
            "sender": "996325769485053218",
            "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
            "signature": "2f3c84e074623ee706a0999a03e44c8729987e0d0fcf2f5daad86d3d0495e206d516227a2fb12aa88650e19a4ac642f17017",
            "subtype": 0,
            "timestamp": 125712,
            "transaction": "8589964069031613329",
            "transactionIndex": 0,
            "type": 6,
            "version": 1
        }"#;

        let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let tx = crate::transaction::Transaction::from_json(&json).unwrap();

        assert_eq!(tx.id, 8589964069031613329u64);
        assert_eq!(tx.type_id, crate::transaction::TransactionType::Data);
        assert_eq!(tx.subtype, 0);

        // 验证 HAS_PRUNABLE_ATTACHMENT = TRUE
        assert!(tx.has_prunable_attachment,
            "TaggedDataUpload should have has_prunable_attachment=true");

        // 验证 HAS_PRUNABLE_MESSAGE = FALSE（这是正确的，因为不是 PrunablePlainMessage）
        assert!(!tx.has_prunable_message,
            "TaggedDataUpload should not have has_prunable_message=true");

        // 验证 attachment_bytes 结构：version(1B) + hash(32B) = 33 bytes
        let expected_att_hex = "01a1474a67570fbabbf6794cb86400c119e0b23f4e69f464daea9e0fa93c5ef80c";
        assert_eq!(hex::encode(&tx.attachment_bytes), expected_att_hex,
            "attachment_bytes mismatch for TaggedDataUpload");
    }
}
