//! 交易数据结构定义

use super::*;
use serde::{Serialize, Deserialize};

pub const TYPE_PAYMENT: u8 = 0;
pub const TYPE_MESSAGING: u8 = 1;
pub const TYPE_COLORED_COINS: u8 = 2;
pub const TYPE_DIGITAL_GOODS: u8 = 3;
pub const TYPE_ACCOUNT_CONTROL: u8 = 4;
pub const TYPE_MONETARY_SYSTEM: u8 = 5;
pub const TYPE_DATA: u8 = 6;
pub const TYPE_SHUFFLING: u8 = 7;
pub const TYPE_ALIASES: u8 = 8;
pub const TYPE_VOTING: u8 = 9;
pub const TYPE_ACCOUNT_PROPERTY: u8 = 10;
pub const TYPE_COIN_EXCHANGE: u8 = 11;
pub const TYPE_LIGHT_CONTRACT: u8 = 12;

// Payment subtypes
pub const SUBTYPE_PAYMENT_ORDINARY_PAYMENT: u8 = 0;

// Messaging subtypes
pub const SUBTYPE_MESSAGING_ARBITRARY_MESSAGE: u8 = 0;
pub const SUBTYPE_MESSAGING_ALIAS_ASSIGNMENT: u8 = 1;
pub const SUBTYPE_MESSAGING_POLL_CREATION: u8 = 2;
pub const SUBTYPE_MESSAGING_VOTE_CASTING: u8 = 3;
pub const SUBTYPE_MESSAGING_HUB_ANNOUNCEMENT: u8 = 4;
pub const SUBTYPE_MESSAGING_ACCOUNT_INFO: u8 = 5;
pub const SUBTYPE_MESSAGING_ALIAS_SELL: u8 = 6;
pub const SUBTYPE_MESSAGING_ALIAS_BUY: u8 = 7;
pub const SUBTYPE_MESSAGING_ALIAS_DELETE: u8 = 8;
pub const SUBTYPE_MESSAGING_PHASING_VOTE_CASTING: u8 = 9;
pub const SUBTYPE_MESSAGING_ACCOUNT_PROPERTY: u8 = 10;
pub const SUBTYPE_MESSAGING_ACCOUNT_PROPERTY_DELETE: u8 = 11;
pub const SUBTYPE_MESSAGING_ACCOUNT_LONG_VALUE_PROPERTY: u8 = 12;

// ColoredCoins subtypes
pub const SUBTYPE_COLORED_COINS_ASSET_ISSUANCE: u8 = 0;
pub const SUBTYPE_COLORED_COINS_ASSET_TRANSFER: u8 = 1;
pub const SUBTYPE_COLORED_COINS_ASK_ORDER_PLACEMENT: u8 = 2;
pub const SUBTYPE_COLORED_COINS_BID_ORDER_PLACEMENT: u8 = 3;
pub const SUBTYPE_COLORED_COINS_ASK_ORDER_CANCELLATION: u8 = 4;
pub const SUBTYPE_COLORED_COINS_BID_ORDER_CANCELLATION: u8 = 5;
pub const SUBTYPE_COLORED_COINS_DIVIDEND_PAYMENT: u8 = 6;
pub const SUBTYPE_COLORED_COINS_ASSET_DELETE: u8 = 7;
pub const SUBTYPE_COLORED_COINS_ASSET_INCREASE: u8 = 8;
pub const SUBTYPE_COLORED_COINS_SET_PHASING_CONTROL: u8 = 9;
pub const SUBTYPE_COLORED_COINS_PROPERTY_SET: u8 = 10;
pub const SUBTYPE_COLORED_COINS_PROPERTY_DELETE: u8 = 11;
pub const SUBTYPE_COLORED_COINS_LONG_VALUE_PROPERTY_SET: u8 = 12;

// DigitalGoods subtypes
pub const SUBTYPE_DIGITAL_GOODS_LISTING: u8 = 0;
pub const SUBTYPE_DIGITAL_GOODS_DELISTING: u8 = 1;
pub const SUBTYPE_DIGITAL_GOODS_PRICE_CHANGE: u8 = 2;
pub const SUBTYPE_DIGITAL_GOODS_QUANTITY_CHANGE: u8 = 3;
pub const SUBTYPE_DIGITAL_GOODS_PURCHASE: u8 = 4;
pub const SUBTYPE_DIGITAL_GOODS_DELIVERY: u8 = 5;
pub const SUBTYPE_DIGITAL_GOODS_FEEDBACK: u8 = 6;
pub const SUBTYPE_DIGITAL_GOODS_REFUND: u8 = 7;

// AccountControl subtypes
pub const SUBTYPE_ACCOUNT_CONTROL_EFFECTIVE_BALANCE_LEASING: u8 = 0;
pub const SUBTYPE_ACCOUNT_CONTROL_PHASING_ONLY: u8 = 1;

// MonetarySystem subtypes
pub const SUBTYPE_MONETARY_SYSTEM_CURRENCY_ISSUANCE: u8 = 0;
pub const SUBTYPE_MONETARY_SYSTEM_RESERVE_INCREASE: u8 = 1;
pub const SUBTYPE_MONETARY_SYSTEM_RESERVE_CLAIM: u8 = 2;
pub const SUBTYPE_MONETARY_SYSTEM_CURRENCY_TRANSFER: u8 = 3;
pub const SUBTYPE_MONETARY_SYSTEM_PUBLISH_EXCHANGE_OFFER: u8 = 4;
pub const SUBTYPE_MONETARY_SYSTEM_EXCHANGE_BUY: u8 = 5;
pub const SUBTYPE_MONETARY_SYSTEM_EXCHANGE_SELL: u8 = 6;
pub const SUBTYPE_MONETARY_SYSTEM_CURRENCY_MINTING: u8 = 7;
pub const SUBTYPE_MONETARY_SYSTEM_CURRENCY_DELETION: u8 = 8;

// Data subtypes
pub const SUBTYPE_DATA_TAGGED_DATA_UPLOAD: u8 = 0;
pub const SUBTYPE_DATA_TAGGED_DATA_EXTEND: u8 = 1;

// Shuffling subtypes
pub const SUBTYPE_SHUFFLING_CREATION: u8 = 0;
pub const SUBTYPE_SHUFFLING_REGISTRATION: u8 = 1;
pub const SUBTYPE_SHUFFLING_PROCESSING: u8 = 2;
pub const SUBTYPE_SHUFFLING_RECIPIENTS: u8 = 3;
pub const SUBTYPE_SHUFFLING_VERIFICATION: u8 = 4;
pub const SUBTYPE_SHUFFLING_CANCELLATION: u8 = 5;

// Aliases subtypes (same as Messaging)
pub const SUBTYPE_ALIASES_ALIAS_ASSIGNMENT: u8 = 0;
pub const SUBTYPE_ALIASES_ALIAS_SELL: u8 = 1;
pub const SUBTYPE_ALIASES_ALIAS_BUY: u8 = 2;
pub const SUBTYPE_ALIASES_ALIAS_DELETE: u8 = 3;

// Voting subtypes
pub const SUBTYPE_VOTING_POLL_CREATION: u8 = 0;
pub const SUBTYPE_VOTING_VOTE_CASTING: u8 = 1;
pub const SUBTYPE_VOTING_PHASING_VOTE_CASTING: u8 = 2;

// CoinExchange subtypes
pub const SUBTYPE_COIN_EXCHANGE_ORDER_ISSUE: u8 = 0;
pub const SUBTYPE_COIN_EXCHANGE_ORDER_CANCEL: u8 = 1;

// LightContract subtypes
pub const SUBTYPE_LIGHT_CONTRACT_REFERENCE_SET: u8 = 0;
pub const SUBTYPE_LIGHT_CONTRACT_REFERENCE_DELETE: u8 = 1;

pub const TRANSACTION_VERSION: u8 = 1;

/// 交易类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Default)]
pub enum TransactionType {
    #[default]
    Payment,
    Messaging,
    ColoredCoins,
    DigitalGoods,
    AccountControl,
    MonetarySystem,
    Data,
    Shuffling,
    Aliases,
    Voting,
    AccountProperty,
    CoinExchange,
    LightContract,
    Unknown,
}

impl<'de> Deserialize<'de> for TransactionType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        
        struct TransactionTypeVisitor;
        
        impl<'de> Visitor<'de> for TransactionTypeVisitor {
            type Value = TransactionType;
            
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a number or string representing transaction type")
            }
            
            fn visit_u64<E>(self, value: u64) -> std::result::Result<TransactionType, E>
            where
                E: de::Error,
            {
                Ok(TransactionType::from_type(value as u8))
            }
            
            fn visit_i64<E>(self, value: i64) -> std::result::Result<TransactionType, E>
            where
                E: de::Error,
            {
                Ok(TransactionType::from_type(value as u8))
            }
            
            fn visit_str<E>(self, value: &str) -> std::result::Result<TransactionType, E>
            where
                E: de::Error,
            {
                match value.parse::<u8>() {
                    Ok(type_byte) => Ok(TransactionType::from_type(type_byte)),
                    Err(_) => Err(de::Error::invalid_value(de::Unexpected::Str(value), &self)),
                }
            }
        }
        
        deserializer.deserialize_any(TransactionTypeVisitor)
    }
}

impl TransactionType {
    pub fn from_type(type_byte: u8) -> Self {
        match type_byte {
            TYPE_PAYMENT => TransactionType::Payment,
            TYPE_MESSAGING => TransactionType::Messaging,
            TYPE_COLORED_COINS => TransactionType::ColoredCoins,
            TYPE_DIGITAL_GOODS => TransactionType::DigitalGoods,
            TYPE_ACCOUNT_CONTROL => TransactionType::AccountControl,
            TYPE_MONETARY_SYSTEM => TransactionType::MonetarySystem,
            TYPE_DATA => TransactionType::Data,
            TYPE_SHUFFLING => TransactionType::Shuffling,
            TYPE_ALIASES => TransactionType::Aliases,
            TYPE_VOTING => TransactionType::Voting,
            TYPE_ACCOUNT_PROPERTY => TransactionType::AccountProperty,
            TYPE_COIN_EXCHANGE => TransactionType::CoinExchange,
            TYPE_LIGHT_CONTRACT => TransactionType::LightContract,
            _ => TransactionType::Unknown,
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            TransactionType::Payment => TYPE_PAYMENT,
            TransactionType::Messaging => TYPE_MESSAGING,
            TransactionType::ColoredCoins => TYPE_COLORED_COINS,
            TransactionType::DigitalGoods => TYPE_DIGITAL_GOODS,
            TransactionType::AccountControl => TYPE_ACCOUNT_CONTROL,
            TransactionType::MonetarySystem => TYPE_MONETARY_SYSTEM,
            TransactionType::Data => TYPE_DATA,
            TransactionType::Shuffling => TYPE_SHUFFLING,
            TransactionType::Aliases => TYPE_ALIASES,
            TransactionType::Voting => TYPE_VOTING,
            TransactionType::AccountProperty => TYPE_ACCOUNT_PROPERTY,
            TransactionType::CoinExchange => TYPE_COIN_EXCHANGE,
            TransactionType::LightContract => TYPE_LIGHT_CONTRACT,
            TransactionType::Unknown => 255,
        }
    }
}

impl From<TransactionType> for u8 {
    fn from(t: TransactionType) -> u8 {
        t.to_byte()
    }
}

/// 交易结构体
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(alias = "transaction", default)]
    pub id: u64,
    #[serde(default)]
    pub version: u8,
    #[serde(alias = "type", default)]
    pub type_id: TransactionType,
    #[serde(default)]
    pub subtype: u8,
    #[serde(default)]
    pub timestamp: Timestamp,
    #[serde(default)]
    pub deadline: u16,
    #[serde(alias = "senderPublicKey")]
    pub sender_public_key: Hash256,
    #[serde(alias = "sender", default)]
    pub sender_id: AccountId,
    #[serde(alias = "recipient", default, skip_serializing_if = "Option::is_none")]
    pub recipient_id: Option<AccountId>,
    #[serde(alias = "amountNQT", deserialize_with = "deserialize_amount_string", default)]
    pub amount: Amount,
    #[serde(alias = "feeNQT", deserialize_with = "deserialize_amount_string", default)]
    pub fee: Amount,
    #[serde(default)]
    pub height: Height,
    #[serde(alias = "block", default)]
    pub block_id: BlockId,
    #[serde(alias = "blockTimestamp", default)]
    pub block_timestamp: Timestamp,
    #[serde(alias = "transactionIndex", default)]
    pub transaction_index: u16,
    pub signature: Signature,
    #[serde(alias = "fullHash", default)]
    pub full_hash: Hash256,
    #[serde(alias = "referencedTransactionFullHash", default, skip_serializing_if = "Option::is_none")]
    pub referenced_transaction_full_hash: Option<Hash256>,
    #[serde(default)]
    pub attachment_bytes: Vec<u8>,
    #[serde(skip)]
    pub pruned_attachment_bytes: u32,
    #[serde(skip)]
    pub attachment_json: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(default)]
    pub phased: bool,
    #[serde(default)]
    pub has_message: bool,
    #[serde(default)]
    pub has_encrypted_message: bool,
    #[serde(default)]
    pub has_public_key_announcement: bool,
    #[serde(default)]
    pub has_prunable_message: bool,
    #[serde(default)]
    pub has_prunable_attachment: bool,
    #[serde(alias = "ecBlockHeight", default)]
    pub ec_block_height: Option<u32>,
    #[serde(alias = "ecBlockId", default)]
    pub ec_block_id: Option<u64>,
    #[serde(default)]
    pub has_encrypttoself_message: bool,
    #[serde(default)]
    pub has_prunable_encrypted_message: bool,
}

impl Default for Transaction {
    fn default() -> Self {
        Self {
            id: 0,
            version: TRANSACTION_VERSION,
            type_id: TransactionType::Payment,
            subtype: 0,
            timestamp: 0,
            deadline: 32767,
            sender_public_key: Hash256([0u8; 32]),
            sender_id: 0,
            recipient_id: None,
            amount: 0,
            fee: 0,
            height: 0,
            block_id: 0,
            block_timestamp: 0,
            transaction_index: 0,
            signature: Signature([0u8; 64]),
            full_hash: Hash256([0u8; 32]),
            referenced_transaction_full_hash: None,
            attachment_bytes: vec![],
            pruned_attachment_bytes: 0,
            attachment_json: None,
            phased: false,
            has_message: false,
            has_encrypted_message: false,
            has_public_key_announcement: false,
            has_prunable_message: false,
            has_prunable_attachment: false,
            ec_block_height: None,
            ec_block_id: None,
            has_encrypttoself_message: false,
            has_prunable_encrypted_message: false,
        }
    }
}

impl Transaction {
    pub fn new(
        type_id: TransactionType,
        sender_id: AccountId,
        recipient_id: Option<AccountId>,
        amount: Amount,
        fee: Amount,
        timestamp: Timestamp,
        deadline: u16,
    ) -> Self {
        Self {
            id: 0,
            version: TRANSACTION_VERSION,
            type_id,
            subtype: 0,
            timestamp,
            deadline,
            sender_public_key: Hash256([0u8; 32]),
            sender_id,
            recipient_id,
            amount,
            fee,
            height: 0,
            block_id: 0,
            block_timestamp: 0,
            transaction_index: 0,
            signature: Signature([0u8; 64]),
            full_hash: Hash256([0u8; 32]),
            referenced_transaction_full_hash: None,
            attachment_bytes: vec![],
            pruned_attachment_bytes: 0,
            attachment_json: None,
            phased: false,
            has_message: false,
            has_encrypted_message: false,
            has_public_key_announcement: false,
            has_prunable_message: false,
            has_prunable_attachment: false,
            ec_block_height: None,
            ec_block_id: None,
            has_encrypttoself_message: false,
            has_prunable_encrypted_message: false,
        }
    }

    pub fn from_json(json: &serde_json::Value) -> Result<Self> {
        let obj = json.as_object().ok_or_else(|| {
            BlockchainError::InvalidTransaction("transaction JSON must be an object".to_string())
        })?;

        let get_u64 = |key: &str| -> Result<u64> {
            match obj.get(key) {
                Some(serde_json::Value::Number(n)) => {
                    n.as_u64().ok_or_else(|| {
                        BlockchainError::InvalidTransaction(format!("invalid {} value", key))
                    })
                }
                Some(serde_json::Value::String(s)) => {
                    s.parse::<u64>().map_err(|_| {
                        BlockchainError::InvalidTransaction(format!("invalid {} string", key))
                    })
                }
                _ => Ok(0),
            }
        };

        let get_u32 = |key: &str| -> Result<u32> {
            Ok(get_u64(key)? as u32)
        };

        let get_i32 = |key: &str| -> Result<i32> {
            match obj.get(key) {
                Some(serde_json::Value::Number(n)) => {
                    n.as_i64()
                        .map(|v| v as i32)
                        .ok_or_else(|| {
                            BlockchainError::InvalidTransaction(format!("invalid {} value", key))
                        })
                }
                Some(serde_json::Value::String(s)) => {
                    s.parse::<i32>().map_err(|_| {
                        BlockchainError::InvalidTransaction(format!("invalid {} string", key))
                    })
                }
                _ => Ok(0),
            }
        };

        let get_u16 = |key: &str| -> Result<u16> {
            Ok(get_u64(key)? as u16)
        };

        let get_u8 = |key: &str| -> Result<u8> {
            Ok(get_u64(key)? as u8)
        };

        let get_hex_bytes = |key: &str| -> Result<Vec<u8>> {
            match obj.get(key) {
                Some(serde_json::Value::String(s)) => {
                    hex::decode(s).map_err(|_| {
                        BlockchainError::InvalidTransaction(format!("invalid hex for {}", key))
                    })
                }
                _ => Ok(vec![]),
            }
        };

        let get_hash256 = |key: &str| -> Result<Hash256> {
            let bytes = get_hex_bytes(key)?;
            if bytes.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Ok(Hash256(arr))
            } else {
                Ok(Hash256([0u8; 32]))
            }
        };

        let get_hash256_opt = |key: &str| -> Result<Option<Hash256>> {
            let bytes = get_hex_bytes(key)?;
            if bytes.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Ok(Some(Hash256(arr)))
            } else {
                Ok(None)
            }
        };

        let get_signature = |key: &str| -> Result<Signature> {
            let bytes = get_hex_bytes(key)?;
            if bytes.len() == 64 {
                let mut arr = [0u8; 64];
                arr.copy_from_slice(&bytes);
                Ok(Signature(arr))
            } else {
                Ok(Signature([0u8; 64]))
            }
        };

        let type_byte = get_u8("type")?;
        let subtype = get_u8("subtype")?;
        let version = obj.get("version")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .unwrap_or(1);

        let sender_public_key = get_hash256("senderPublicKey")?;
        let sender_id = Self::public_key_to_account_id(&sender_public_key.0);

        let recipient_id = match obj.get("recipient") {
            Some(serde_json::Value::String(s)) => {
                s.parse::<i64>().ok().map(|i| i as u64)
                    .or_else(|| s.parse::<u64>().ok())
            }
            Some(serde_json::Value::Number(n)) => {
                n.as_i64().map(|i| i as u64)
                    .or_else(|| n.as_u64())
            }
            _ => None,
        };

        let amount = get_u64("amountNQT")?;
        let fee = get_u64("feeNQT")?;
        let timestamp = get_i32("timestamp")? as Timestamp;
        let deadline = get_u16("deadline")?;

        let signature = get_signature("signature")?;
        let full_hash = get_hash256("fullHash")?;
        let referenced_transaction_full_hash = get_hash256_opt("referencedTransactionFullHash")?;

        let ec_block_height = obj.get("ecBlockHeight")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);
        let ec_block_id = obj.get("ecBlockId")
            .and_then(|v| {
                // Java 中 ecBlockId 是 long (i64) 类型，可能为负数
                // 优先从字符串解析，支持有符号和无符号两种形式
                v.as_str()
                    .and_then(|s| {
                        s.parse::<i64>().ok().map(|i| i as u64)
                            .or_else(|| s.parse::<u64>().ok())
                    })
                    .or_else(|| v.as_i64().map(|i| i as u64))
                    .or_else(|| v.as_u64())
            });

        let att_obj = obj.get("attachment")
            .and_then(|a| a.as_object())
            .cloned();

        // 使用二进制协议序列化 attachment_bytes（与 Java NRCS 一致）
        // 优先使用 JSON 中的 attachmentBytes 字段（hex 编码），否则从 attachment 对象生成
        let mut pruned_bytes: u32 = 0;
        let attachment_bytes = if let Some(hex_str) = obj.get("attachmentBytes")
            .and_then(|v| v.as_str())
        {
            hex::decode(hex_str).unwrap_or_default()
        } else {
            crate::attachment_serde::build_attachment_bytes_from_json(
                type_byte, subtype, version, att_obj.as_ref(), &mut pruned_bytes,
            )
        };

        // 检测各 appendix 标志（与 Java getFlags() 位图一致）
        let (has_message, has_encrypted_message, has_public_key_announcement,
             has_encrypttoself_message, phased, has_prunable_message,
             has_prunable_encrypted_message, has_tagged_data_prunable) =
            crate::attachment_serde::detect_appendix_flags(att_obj.as_ref(), type_byte, subtype);

        let mut tx = Self {
            id: 0,
            version,
            type_id: TransactionType::from_type(type_byte),
            subtype,
            timestamp,
            deadline,
            sender_public_key,
            sender_id,
            recipient_id,
            amount,
            fee,
            height: 0,
            block_id: 0,
            block_timestamp: 0,
            transaction_index: 0,
            signature,
            full_hash,
            referenced_transaction_full_hash,
            attachment_bytes,
            pruned_attachment_bytes: pruned_bytes,
            attachment_json: att_obj,
            phased,
            has_message,
            has_encrypted_message,
            has_public_key_announcement,
            has_prunable_message,
            has_prunable_attachment: has_tagged_data_prunable,
            ec_block_height,
            ec_block_id,
            has_encrypttoself_message,
            has_prunable_encrypted_message,
        };

        if tx.full_hash.0 == [0u8; 32] {
            if let Ok(calculated_hash) = tx.calculate_full_hash() {
                tx.full_hash = calculated_hash;
            }
        }

        tx.id = tx.calculate_id();

        Ok(tx)
    }

    pub fn public_key_to_account_id(public_key: &[u8; 32]) -> AccountId {
        crate::block::account_id_from_public_key(public_key)
    }

    pub fn calculate_id(&self) -> u64 {
        if !self.full_hash.0.iter().all(|&b| b == 0) {
            let mut id_bytes = [0u8; 8];
            id_bytes.copy_from_slice(&self.full_hash.0[..8]);
            return u64::from_le_bytes(id_bytes);
        }

        match self.calculate_full_hash() {
            Ok(full_hash) => {
                let mut id_bytes = [0u8; 8];
                id_bytes.copy_from_slice(&full_hash.0[..8]);

                u64::from_le_bytes(id_bytes)
            }
            Err(_) => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(&self.signature.0);
                let hash = hasher.finalize();

                let mut id_bytes = [0u8; 8];
                id_bytes.copy_from_slice(&hash[..8]);
                u64::from_le_bytes(id_bytes)
            }
        }
    }

    pub fn calculate_full_hash(&self) -> Result<Hash256> {
        use sha2::{Digest, Sha256};

        // 对应 Java: zeroSignature(getBytes()) 作为 data
        let data = self.serialize_for_signing();

        let mut hasher = Sha256::new();
        hasher.update(&self.signature.0);
        let signature_hash = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(&data);
        hasher.update(&signature_hash);
        let hash = hasher.finalize();
        
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hash);
        
        Ok(Hash256(arr))
    }

    pub fn compute_hash(&self) -> Result<Hash256> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(&self.serialize_for_signing());
        hasher.update(&self.signature.0);
        let hash = hasher.finalize();

        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hash);
        Ok(Hash256(arr))
    }

    /// 序列化交易基础字段（type ~ refTxHash，共 96 字节）
    ///
    /// 对应 Java Transaction.bytes() 的前 signatureOffset 个字节
    fn serialize_base_fields(&self) -> Vec<u8> {
        const GENESIS_CREATOR_ID: i64 = -80957052124787088i64;

        let mut buf = Vec::with_capacity(96);
        buf.extend_from_slice(&self.type_id.to_byte().to_le_bytes());
        buf.extend_from_slice(&((self.subtype & 0x0f) | (self.version << 4)).to_le_bytes());
        buf.extend_from_slice(&self.timestamp.to_le_bytes());
        buf.extend_from_slice(&(self.deadline as i16).to_le_bytes());
        buf.extend_from_slice(&self.sender_public_key.0);

        if let Some(recipient) = self.recipient_id {
            buf.extend_from_slice(&recipient.to_le_bytes());
        } else {
            buf.extend_from_slice(&GENESIS_CREATOR_ID.to_le_bytes());
        }

        buf.extend_from_slice(&self.amount.to_le_bytes());
        buf.extend_from_slice(&self.fee.to_le_bytes());
        if let Some(ref hash) = self.referenced_transaction_full_hash {
            buf.extend_from_slice(&hash.0);
        } else {
            buf.extend_from_slice(&[0u8; 32]);
        }

        buf
    }

    /// 序列化用于签名验证的数据（signature 部分为零填充）
    ///
    /// 对应 Java: zeroSignature(getBytes())
    /// 用于 Curve25519 verify() 的 message 参数
    pub fn serialize_for_signing(&self) -> Vec<u8> {
        let mut buf = self.serialize_base_fields();

        // signature 占位（64 字节零填充）
        // 对应 Java: zeroSignature() 将 signatureOffset ~ signatureOffset+63 置零
        buf.extend_from_slice(&[0u8; 64]);

        // version > 0 时的扩展字段
        // 对应 Java: bytes() 中 if (this.getVersion() > 0) 分支
        if self.version > 0 {
            let flags = self.get_flags();
            buf.extend_from_slice(&flags.to_le_bytes());
            buf.extend_from_slice(&self.ec_block_height.unwrap_or(0).to_le_bytes());
            buf.extend_from_slice(&self.ec_block_id.unwrap_or(0).to_le_bytes());
        }

        // attachment 数据
        // 对应 Java: appendages.putBytes(buffer)
        buf.extend_from_slice(&self.attachment_bytes);

        buf
    }

    /// 获取完整交易字节数据（包含实际签名）
    ///
    /// 对应 Java: Transaction.getBytes()
    /// 用于 payload hash 计算：SHA256(tx1.getBytes() + tx2.getBytes() + ...)
    pub fn get_bytes(&self) -> Vec<u8> {
        let mut buf = self.serialize_base_fields();

        // 实际签名数据（64 字节）
        buf.extend_from_slice(&self.signature.0);

        // version > 0 时的扩展字段
        if self.version > 0 {
            let flags = self.get_flags();
            buf.extend_from_slice(&flags.to_le_bytes());
            buf.extend_from_slice(&self.ec_block_height.unwrap_or(0).to_le_bytes());
            buf.extend_from_slice(&self.ec_block_id.unwrap_or(0).to_le_bytes());
        }

        // attachment 数据
        buf.extend_from_slice(&self.attachment_bytes);

        buf
    }

    pub fn serialize_for_full_hash(&self) -> Vec<u8> {
        // serialize_for_signing() 已包含完整数据（对应 Java zeroSignature(getBytes())）
        self.serialize_for_signing()
    }

    fn get_flags(&self) -> u32 {
        // 对应 Java Transaction.getFlags() 的位图定义（严格匹配 Java 源码）：
        // bit 0 (1): message != null
        // bit 1 (2): encryptedMessage != null
        // bit 2 (4): publicKeyAnnouncement != null
        // bit 3 (8): encryptToSelfMessage != null
        // bit 4 (16): phasing != null
        // bit 5 (32): prunablePlainMessage != null
        // bit 6 (64): prunableEncryptedMessage != null
        let mut flags: u32 = 0;
        if self.has_message { flags |= 1; }
        if self.has_encrypted_message { flags |= 2; }
        if self.has_public_key_announcement { flags |= 4; }
        if self.has_encrypttoself_message { flags |= 8; }
        if self.phased { flags |= 16; }
        if self.has_prunable_message { flags |= 32; }
        if self.has_prunable_encrypted_message { flags |= 64; }
        flags
    }

    /// 验证交易签名（使用 Curve25519 EC-KCDSA）
    ///
    /// 对应 Java: Crypto.verify(signature, message, publicKey, enforceCanonical)
    pub fn verify_signature(&self) -> bool {
        let pub_key = crypto::PublicKey::Curve25519(self.sender_public_key.0);
        let message = self.serialize_for_signing();
        crypto::verify(&pub_key, &message, &self.signature.0).is_ok()
    }

    pub fn validate_basic(&self) -> Result<()> {
        if self.version < 1 {
            return Err(BlockchainError::InvalidTransaction("invalid transaction version".to_string()));
        }

        if self.id == 0 {
            return Err(BlockchainError::InvalidTransaction("invalid transaction id 0".to_string()));
        }

        if self.deadline == 0 {
            return Err(BlockchainError::InvalidTransaction("deadline cannot be zero".to_string()));
        }

        if self.sender_id == 0 {
            return Err(BlockchainError::InvalidTransaction("sender_id cannot be zero".to_string()));
        }

        Ok(())
    }

    pub fn is_payment(&self) -> bool {
        matches!(self.type_id, TransactionType::Payment)
    }

    pub fn size(&self) -> usize {
        200 + self.attachment_bytes.len()
    }

    /// 获取附加数据列表
    /// 对应 Java: getAppendages()
    pub fn get_appendages(&self) -> Vec<Appendage> {
        let mut appendages = Vec::new();
        
        if self.has_message {
            appendages.push(Appendage::Message);
        }
        if self.has_encrypted_message {
            appendages.push(Appendage::EncryptedMessage);
        }
        if self.has_public_key_announcement {
            appendages.push(Appendage::PublicKeyAnnouncement);
        }
        if self.has_prunable_attachment {
            appendages.push(Appendage::PrunableAttachment);
        }
        if self.has_encrypttoself_message {
            appendages.push(Appendage::EncryptToSelfMessage);
        }
        if self.has_prunable_encrypted_message {
            appendages.push(Appendage::PrunableEncryptedMessage);
        }
        
        appendages
    }

    /// 获取引用交易完整哈希
    /// 对应 Java: getReferencedTransactionFullHash()
    pub fn get_referenced_transaction_full_hash(&self) -> Option<Hash256> {
        self.referenced_transaction_full_hash
    }

    /// 设置引用交易完整哈希
    pub fn set_referenced_transaction_full_hash(&mut self, hash: Option<Hash256>) {
        self.referenced_transaction_full_hash = hash;
    }

    /// 取消区块关联
    /// 对应 Java: unsetBlock()
    pub fn unset_block(&mut self) {
        self.block_id = 0;
        self.block_timestamp = 0;
        self.height = 0;
        self.transaction_index = 0;
    }

    /// 检查是否为未确认重复交易
    /// 对应 Java: isUnconfirmedDuplicate()
    pub fn is_unconfirmed_duplicate(&self, known_tx_ids: &[u64]) -> bool {
        known_tx_ids.contains(&self.id)
    }
}

/// 附加数据类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Appendage {
    Message,
    EncryptedMessage,
    PublicKeyAnnouncement,
    PrunableAttachment,
    EncryptToSelfMessage,
    PrunableEncryptedMessage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new(
            TransactionType::Payment,
            1234567890,
            Some(9876543210),
            1_000_000_000,
            100_000,
            1_704_000_000,
            32767,
        );

        assert_eq!(tx.type_id, TransactionType::Payment);
        assert_eq!(tx.amount, 1_000_000_000);
        assert_eq!(tx.sender_id, 1234567890);
        assert!(matches!(tx.recipient_id, Some(9876543210)));
    }

    #[test]
    fn test_transaction_type_conversion() {
        assert_eq!(TransactionType::from_type(TYPE_PAYMENT), TransactionType::Payment);
        assert_eq!(TransactionType::from_type(TYPE_MESSAGING), TransactionType::Messaging);
        assert_eq!(TransactionType::Payment.to_byte(), TYPE_PAYMENT);
    }
}

fn deserialize_amount_string<'de, D>(deserializer: D) -> std::result::Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    
    struct AmountVisitor;
    
    impl<'de> Visitor<'de> for AmountVisitor {
        type Value = u64;
        
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or number")
        }
        
        fn visit_str<E>(self, value: &str) -> std::result::Result<u64, E>
        where
            E: de::Error,
        {
            value.parse::<u64>().map_err(de::Error::custom)
        }
        
        fn visit_u64<E>(self, value: u64) -> std::result::Result<u64, E>
        where
            E: de::Error,
        {
            Ok(value)
        }
        
        fn visit_i64<E>(self, value: i64) -> std::result::Result<u64, E>
        where
            E: de::Error,
        {
            if value < 0 {
                Ok(0)
            } else {
                Ok(value as u64)
            }
        }
    }
    
    deserializer.deserialize_any(AmountVisitor)
}
