//! 参数解析器
//!
//! 对照 Java NRCS ParameterParser 实现，提供统一的 HTTP API 参数解析与验证。
//! 所有参数解析遵循 "取值 → 空值检查 → 类型转换 → 范围校验 → 返回/抛异常" 流程。
//!
//! 参考: /mnt/d/workspace/git/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/http/ParameterParser.java

use blockchain_types::constants::*;
use blockchain_types::transaction::Transaction;
use crate::error::ApiError;
use crate::request_handler::ApiRequest;

pub struct ParameterParser;

impl ParameterParser {
    // ==================== 基础数值解析 ====================

    pub fn get_byte(req: &ApiRequest, name: &str, min: i8, max: i8, is_mandatory: bool) -> Result<i8, ApiError> {
        Self::get_byte_with_default(req, name, min, max, 0, is_mandatory)
    }

    pub fn get_byte_with_default(
        req: &ApiRequest, name: &str, min: i8, max: i8, default: i8, is_mandatory: bool,
    ) -> Result<i8, ApiError> {
        match req.get_string(name) {
            Some(s) => {
                let v = s.parse::<i8>().map_err(|_| {
                    ApiError::IncorrectParameter(name.to_string())
                })?;
                if v < min || v > max {
                    return Err(ApiError::IncorrectParameterWithDetails {
                        param: name.to_string(),
                        details: format!("value {} not in range [{}, {}]", v, min, max),
                    });
                }
                Ok(v)
            }
            None => {
                if is_mandatory {
                    Err(ApiError::MissingParameter(name.to_string()))
                } else {
                    Ok(default)
                }
            }
        }
    }

    pub fn get_int(req: &ApiRequest, name: &str, min: i32, max: i32, is_mandatory: bool) -> Result<i32, ApiError> {
        Self::get_int_with_default(req, name, min, max, 0, is_mandatory)
    }

    pub fn get_int_with_default(
        req: &ApiRequest, name: &str, min: i32, max: i32, default: i32, is_mandatory: bool,
    ) -> Result<i32, ApiError> {
        match req.get_string(name) {
            Some(s) => {
                let v = s.parse::<i32>().map_err(|_| {
                    ApiError::IncorrectParameter(name.to_string())
                })?;
                if v < min || v > max {
                    return Err(ApiError::IncorrectParameterWithDetails {
                        param: name.to_string(),
                        details: format!("value {} not in range [{}, {}]", v, min, max),
                    });
                }
                Ok(v)
            }
            None => {
                if is_mandatory {
                    Err(ApiError::MissingParameter(name.to_string()))
                } else {
                    Ok(default)
                }
            }
        }
    }

    pub fn get_long(req: &ApiRequest, name: &str, min: i64, max: i64, is_mandatory: bool) -> Result<i64, ApiError> {
        Self::get_long_with_default(req, name, min, max, 0, is_mandatory)
    }

    pub fn get_long_with_default(
        req: &ApiRequest, name: &str, min: i64, max: i64, default: i64, is_mandatory: bool,
    ) -> Result<i64, ApiError> {
        match req.get_string(name) {
            Some(s) => {
                let v = s.parse::<i64>().map_err(|_| {
                    ApiError::IncorrectParameter(name.to_string())
                })?;
                if v < min || v > max {
                    return Err(ApiError::IncorrectParameterWithDetails {
                        param: name.to_string(),
                        details: format!("value {} not in range [{}, {}]", v, min, max),
                    });
                }
                Ok(v)
            }
            None => {
                if is_mandatory {
                    Err(ApiError::MissingParameter(name.to_string()))
                } else {
                    Ok(default)
                }
            }
        }
    }

    pub fn get_unsigned_long(req: &ApiRequest, name: &str, is_mandatory: bool) -> Result<u64, ApiError> {
        match req.get_string(name) {
            Some(s) => {
                let v = s.parse::<u64>().map_err(|_| {
                    ApiError::IncorrectParameter(name.to_string())
                })?;
                if v == 0 {
                    return Err(ApiError::IncorrectParameter(name.to_string()));
                }
                Ok(v)
            }
            None => {
                if is_mandatory {
                    Err(ApiError::MissingParameter(name.to_string()))
                } else {
                    Ok(0)
                }
            }
        }
    }

    pub fn get_unsigned_longs(req: &ApiRequest, name: &str) -> Result<Vec<u64>, ApiError> {
        let mut result = Vec::new();
        if let Some(s) = req.get_string(name) {
            for part in s.split(',') {
                let trimmed = part.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let v = trimmed.parse::<u64>().map_err(|_| {
                    ApiError::IncorrectParameter(name.to_string())
                })?;
                if v == 0 {
                    return Err(ApiError::IncorrectParameter(name.to_string()));
                }
                result.push(v);
            }
        }
        Ok(result)
    }

    // ==================== 字节/十六进制解析 ====================

    pub fn get_bytes(req: &ApiRequest, name: &str, is_mandatory: bool) -> Result<Option<Vec<u8>>, ApiError> {
        match req.get_string(name) {
            Some(s) => {
                let bytes = hex::decode(&s).map_err(|_| {
                    ApiError::IncorrectParameter(name.to_string())
                })?;
                Ok(Some(bytes))
            }
            None => {
                if is_mandatory {
                    Err(ApiError::MissingParameter(name.to_string()))
                } else {
                    Ok(None)
                }
            }
        }
    }

    // ==================== 账户相关解析 ====================

    pub fn get_account_id(req: &ApiRequest, is_mandatory: bool) -> Result<u64, ApiError> {
        Self::get_account_id_by_name(req, "account", is_mandatory)
    }

    pub fn get_account_id_by_name(req: &ApiRequest, name: &str, is_mandatory: bool) -> Result<u64, ApiError> {
        match req.get_string(name) {
            Some(s) => {
                let account_id = parse_account_id(&s)?;
                if account_id == 0 {
                    return Err(ApiError::IncorrectParameter(name.to_string()));
                }
                Ok(account_id)
            }
            None => {
                if is_mandatory {
                    Err(ApiError::MissingParameter(name.to_string()))
                } else {
                    Ok(0)
                }
            }
        }
    }

    pub fn get_account_ids(req: &ApiRequest, is_mandatory: bool) -> Result<Vec<u64>, ApiError> {
        let mut result = Vec::new();
        if let Some(s) = req.get_string("accounts") {
            for part in s.split(',') {
                let trimmed = part.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let id = parse_account_id(trimmed)?;
                if id == 0 {
                    return Err(ApiError::IncorrectParameter("accounts".to_string()));
                }
                result.push(id);
            }
        } else if is_mandatory {
            return Err(ApiError::MissingParameter("accounts".to_string()));
        }
        Ok(result)
    }

    pub fn get_sender_id(req: &ApiRequest) -> Result<u64, ApiError> {
        let public_key = Self::get_public_key(req)?;
        Ok(Transaction::public_key_to_account_id(&public_key))
    }

    // ==================== 密钥/密码解析 ====================

    pub fn get_secret_phrase(req: &ApiRequest, is_mandatory: bool) -> Result<Option<String>, ApiError> {
        match req.get_string("secretPhrase") {
            Some(s) if !s.is_empty() => Ok(Some(s)),
            _ => {
                if is_mandatory {
                    Err(ApiError::MissingParameter("secretPhrase".to_string()))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub fn get_public_key(req: &ApiRequest) -> Result<[u8; 32], ApiError> {
        Self::get_public_key_with_prefix(req, None)
    }

    pub fn get_public_key_with_prefix(req: &ApiRequest, prefix: Option<&str>) -> Result<[u8; 32], ApiError> {
        if let Some(sp) = req.get_string("secretPhrase") {
            if !sp.is_empty() {
                let pk = crypto::derive_public_key(&sp)
                    .map_err(|e| ApiError::Internal(format!("failed to derive public key: {}", e)))?;
                let mut arr = [0u8; 32];
                if pk.len() == 32 {
                    arr.copy_from_slice(&pk);
                }
                return Ok(arr);
            }
        }

        let key_name = prefix.map(|p| format!("{}PublicKey", p)).unwrap_or_else(|| "publicKey".to_string());
        match req.get_string(&key_name) {
            Some(s) => {
                let bytes = hex::decode(&s).map_err(|_| {
                    ApiError::IncorrectParameter(key_name.to_string())
                })?;
                if bytes.len() != 32 {
                    return Err(ApiError::IncorrectParameterWithDetails {
                        param: key_name,
                        details: format!("invalid public key length: {}", bytes.len()),
                    });
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Ok(arr)
            }
            None => Err(ApiError::MissingParameter("secretPhrase or publicKey".to_string())),
        }
    }

    pub fn get_public_keys(req: &ApiRequest, name: &str) -> Result<Vec<[u8; 32]>, ApiError> {
        let mut result = Vec::new();
        if let Some(s) = req.get_string(name) {
            for part in s.split(&[',', '\n'][..]) {
                let trimmed = part.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let bytes = hex::decode(trimmed).map_err(|_| {
                    ApiError::IncorrectParameter(name.to_string())
                })?;
                if bytes.len() != 32 {
                    return Err(ApiError::IncorrectParameterWithDetails {
                        param: name.to_string(),
                        details: "invalid public key length".to_string(),
                    });
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                result.push(arr);
            }
        }
        Ok(result)
    }

    // ==================== 金额/费用解析 ====================

    pub fn get_amount_nqt(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_long(req, "amountNQT", 1, MAX_BALANCE_NQT as i64, true)
            .map(|v| v as u64)
    }

    pub fn get_fee_nqt(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_long(req, "feeNQT", 0, MAX_BALANCE_NQT as i64, true)
            .map(|v| v as u64)
    }

    pub fn get_price_nqt(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_long(req, "priceNQT", 1, MAX_BALANCE_NQT as i64, true)
            .map(|v| v as u64)
    }

    pub fn get_quantity_qnt(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_long(req, "quantityQNT", 1, MAX_ASSET_QUANTITY_QNT as i64, true)
            .map(|v| v as u64)
    }

    pub fn get_amount_nqt_per_qnt(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_long(req, "amountNQTPerQNT", 1, MAX_BALANCE_NQT as i64, true)
            .map(|v| v as u64)
    }

    pub fn get_price_nqt_per_share(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_long(req, "priceNQTPerShare", 1, MAX_BALANCE_NQT as i64, true)
            .map(|v| v as u64)
    }

    pub fn get_price_nqt_per_coin(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_long(req, "priceNQTPerCoin", 1, MAX_BALANCE_NQT as i64, true)
            .map(|v| v as u64)
    }

    pub fn get_rate_nqt_per_unit(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_long(req, "rateNQTPerUnit", 1, MAX_BALANCE_NQT as i64, true)
            .map(|v| v as u64)
    }

    // ==================== 区块链状态参数 ====================

    pub fn get_timestamp(req: &ApiRequest) -> Result<i32, ApiError> {
        Self::get_int(req, "timestamp", 0, i32::MAX, false)
    }

    pub fn get_first_index(req: &ApiRequest) -> i32 {
        req.get_string("firstIndex")
            .and_then(|s| s.parse::<i32>().ok())
            .map(|v| if v < 0 { 0 } else { v })
            .unwrap_or(0)
    }

    pub fn get_last_index(req: &ApiRequest) -> i32 {
        req.get_string("lastIndex")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(i32::MAX)
    }

    pub fn get_number_of_confirmations(req: &ApiRequest) -> Result<i32, ApiError> {
        Self::get_int(req, "numberOfConfirmations", 0, i32::MAX, false)
    }

    pub fn get_height(req: &ApiRequest) -> Result<i32, ApiError> {
        Self::get_height_with_mandatory(req, false)
    }

    pub fn get_height_with_mandatory(req: &ApiRequest, is_mandatory: bool) -> Result<i32, ApiError> {
        Self::get_int_with_default(req, "height", 0, i32::MAX, -1, is_mandatory)
    }

    // ==================== 交易解析 ====================

    pub fn parse_transaction(
        transaction_json: Option<&str>,
        transaction_bytes: Option<&str>,
        prunable_attachment_json: Option<&str>,
    ) -> Result<Transaction, ApiError> {
        match (transaction_json, transaction_bytes) {
            (Some(json_str), None) => {
                let json_value: serde_json::Value = serde_json::from_str(json_str)
                    .map_err(|_| ApiError::IncorrectParameter("transactionJSON".to_string()))?;
                Transaction::from_json(&json_value).map_err(ApiError::Blockchain)
            }
            (None, Some(hex_str)) => {
                let bytes = hex::decode(hex_str)
                    .map_err(|_| ApiError::IncorrectParameter("transactionBytes".to_string()))?;
                let mut tx = crate::handlers::v1::create_transaction::CreateTransactionHelper
                    ::parse_transaction_from_bytes(&bytes)?;
                if let Some(prunable_str) = prunable_attachment_json {
                    let prunable: serde_json::Value = serde_json::from_str(prunable_str)
                        .map_err(|_| ApiError::IncorrectParameter("prunableAttachmentJSON".to_string()))?;
                    if let Some(obj) = prunable.as_object() {
                        tx.attachment_json = Some(obj.clone());
                    }
                }
                Ok(tx)
            }
            (Some(_), Some(_)) => {
                Err(ApiError::EitherParameter {
                    params: "transactionJSON, transactionBytes".to_string(),
                })
            }
            (None, None) => {
                Err(ApiError::MissingParameters {
                    params: "transactionJSON, transactionBytes".to_string(),
                })
            }
        }
    }

    // ==================== 消息解析 ====================

    pub fn get_plain_message(req: &ApiRequest, prunable: bool) -> Result<Option<serde_json::Value>, ApiError> {
        let message = req.get_string("message");
        let message_is_text = req.get_string("messageIsText")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(true);

        match message {
            Some(msg) if !msg.is_empty() => {
                let mut obj = serde_json::Map::new();
                obj.insert("message".to_string(), serde_json::Value::String(msg));
                obj.insert("messageIsText".to_string(), serde_json::Value::Bool(message_is_text));
                if prunable {
                    obj.insert("version.PrunablePlainMessage".to_string(), serde_json::Value::Number(1.into()));
                } else {
                    obj.insert("version.Message".to_string(), serde_json::Value::Number(1.into()));
                }
                Ok(Some(serde_json::Value::Object(obj)))
            }
            _ => Ok(None),
        }
    }

    pub fn get_encrypted_message(req: &ApiRequest) -> Result<Option<serde_json::Value>, ApiError> {
        let data = req.get_string("encryptedMessageData");
        let nonce = req.get_string("encryptedMessageNonce");

        match (data, nonce) {
            (Some(d), Some(n)) => {
                let mut obj = serde_json::Map::new();
                obj.insert("encryptedMessageData".to_string(), serde_json::Value::String(d));
                obj.insert("encryptedMessageNonce".to_string(), serde_json::Value::String(n));
                obj.insert("version.EncryptedMessage".to_string(), serde_json::Value::Number(1.into()));
                Ok(Some(serde_json::Value::Object(obj)))
            }
            _ => Ok(None),
        }
    }

    pub fn get_encrypt_to_self_message(req: &ApiRequest) -> Result<Option<serde_json::Value>, ApiError> {
        let data = req.get_string("encryptToSelfMessageData");
        let nonce = req.get_string("encryptToSelfMessageNonce");

        match (data, nonce) {
            (Some(d), Some(n)) => {
                let mut obj = serde_json::Map::new();
                obj.insert("encryptToSelfMessageData".to_string(), serde_json::Value::String(d));
                obj.insert("encryptToSelfMessageNonce".to_string(), serde_json::Value::String(n));
                obj.insert("version.EncryptToSelfMessage".to_string(), serde_json::Value::Number(1.into()));
                Ok(Some(serde_json::Value::Object(obj)))
            }
            _ => Ok(None),
        }
    }

    // ==================== DGS 数量解析 ====================

    pub fn get_goods_quantity(req: &ApiRequest) -> Result<u32, ApiError> {
        Self::get_int(req, "quantity", 0, MAX_DGS_LISTING_QUANTITY as i32, true)
            .map(|v| v as u32)
    }

    // ==================== 搜索查询 ====================

    pub fn get_search_query(req: &ApiRequest) -> Result<String, ApiError> {
        let query = req.get_string("query");
        let tag = req.get_string("tag");

        match (query, tag) {
            (Some(q), _) => Ok(q),
            (None, Some(t)) => Ok(t),
            (None, None) => Err(ApiError::MissingParameters {
                params: "query, tag".to_string(),
            }),
        }
    }

    pub fn get_account_property(req: &ApiRequest, is_mandatory: bool) -> Result<Option<String>, ApiError> {
        match req.get_string("property") {
            Some(s) => Ok(Some(s)),
            None => {
                if is_mandatory {
                    Err(ApiError::MissingParameter("property".to_string()))
                } else {
                    Ok(None)
                }
            }
        }
    }

    // ==================== 通用工具方法 ====================

    pub fn get_parameter(req: &ApiRequest, name: &str) -> Result<String, ApiError> {
        req.get_string(name)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ApiError::MissingParameter(name.to_string()))
    }

    pub fn get_json(req: &ApiRequest, name: &str) -> Result<Option<serde_json::Value>, ApiError> {
        match req.get_string(name) {
            Some(s) => {
                let v: serde_json::Value = serde_json::from_str(&s)
                    .map_err(|_| ApiError::IncorrectParameter(name.to_string()))?;
                Ok(Some(v))
            }
            None => Ok(None),
        }
    }

    pub fn get_json_array(req: &ApiRequest, name: &str) -> Result<Option<serde_json::Value>, ApiError> {
        match req.get_string(name) {
            Some(s) => {
                let v: serde_json::Value = serde_json::from_str(&s)
                    .map_err(|_| ApiError::IncorrectParameter(name.to_string()))?;
                if v.is_array() {
                    Ok(Some(v))
                } else {
                    Err(ApiError::IncorrectParameter(name.to_string()))
                }
            }
            None => Ok(None),
        }
    }

    pub fn empty_to_null(s: Option<String>) -> Option<String> {
        s.filter(|v| !v.trim().is_empty())
    }

    // ==================== 业务实体 ID 解析 ====================

    pub fn get_alias_id(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_unsigned_long(req, "alias", true)
    }

    pub fn get_alias_name(req: &ApiRequest) -> Result<String, ApiError> {
        Self::get_parameter(req, "aliasName")
    }

    pub fn get_asset_id(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_unsigned_long(req, "asset", true)
    }

    pub fn get_currency_id(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_unsigned_long(req, "currency", true)
    }

    pub fn get_poll_id(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_unsigned_long(req, "poll", true)
    }

    pub fn get_goods_id(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_unsigned_long(req, "goods", true)
    }

    pub fn get_purchase_id(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_unsigned_long(req, "purchase", true)
    }

    pub fn get_offer_id(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_unsigned_long(req, "offer", true)
    }

    pub fn get_shuffling_id(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_unsigned_long(req, "shuffling", true)
    }

    pub fn get_tagged_data_id(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_unsigned_long(req, "taggedData", true)
    }

    pub fn get_transaction_id(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_unsigned_long(req, "transaction", true)
    }

    // ==================== 持有类型解析 ====================

    pub fn get_holding_type(req: &ApiRequest) -> Result<u8, ApiError> {
        let val = Self::get_byte(req, "holdingType", 0, 2, false)?;
        Ok(val as u8)
    }

    pub fn get_holding_id(req: &ApiRequest, holding_type: u8) -> Result<u64, ApiError> {
        if holding_type == 0 {
            Ok(0)
        } else {
            Self::get_unsigned_long(req, "holding", true)
        }
    }

    // ==================== Phasing 参数解析 ====================

    pub fn parse_phasing_params(req: &ApiRequest, prefix: Option<&str>) -> Result<Option<serde_json::Value>, ApiError> {
        let p = prefix.unwrap_or("");
        let phased_key = if p.is_empty() { "phased".to_string() } else { format!("{}Phased", p) };

        let phased = req.get_string(&phased_key)
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(false);

        if !phased {
            return Ok(None);
        }

        let finish_height_key = if p.is_empty() { "phasingFinishHeight".to_string() } else { format!("{}PhasingFinishHeight", p) };
        let voting_model_key = if p.is_empty() { "phasingVotingModel".to_string() } else { format!("{}PhasingVotingModel", p) };
        let quorum_key = if p.is_empty() { "phasingQuorum".to_string() } else { format!("{}PhasingQuorum", p) };
        let min_balance_key = if p.is_empty() { "phasingMinBalance".to_string() } else { format!("{}PhasingMinBalance", p) };
        let min_balance_model_key = if p.is_empty() { "phasingMinBalanceModel".to_string() } else { format!("{}PhasingMinBalanceModel", p) };
        let holding_id_key = if p.is_empty() { "phasingHolding".to_string() } else { format!("{}PhasingHolding", p) };
        let hashed_secret_key = if p.is_empty() { "phasingHashedSecret".to_string() } else { format!("{}PhasingHashedSecret", p) };
        let hashed_secret_alg_key = if p.is_empty() { "phasingHashedSecretAlgorithm".to_string() } else { format!("{}PhasingHashedSecretAlgorithm", p) };

        let mut phasing = serde_json::Map::new();
        phasing.insert("phased".to_string(), serde_json::Value::Bool(true));

        if let Some(v) = req.get_string(&finish_height_key) {
            if let Ok(n) = v.parse::<i64>() {
                phasing.insert("finishHeight".to_string(), serde_json::Value::Number(n.into()));
            }
        }

        if let Some(v) = req.get_string(&voting_model_key) {
            if let Ok(n) = v.parse::<i64>() {
                phasing.insert("votingModel".to_string(), serde_json::Value::Number(n.into()));
            }
        }

        if let Some(v) = req.get_string(&quorum_key) {
            if let Ok(n) = v.parse::<i64>() {
                phasing.insert("quorum".to_string(), serde_json::Value::Number(n.into()));
            }
        }

        if let Some(v) = req.get_string(&min_balance_key) {
            if let Ok(n) = v.parse::<i64>() {
                phasing.insert("minBalance".to_string(), serde_json::Value::Number(n.into()));
            }
        }

        if let Some(v) = req.get_string(&min_balance_model_key) {
            if let Ok(n) = v.parse::<i64>() {
                phasing.insert("minBalanceModel".to_string(), serde_json::Value::Number(n.into()));
            }
        }

        if let Some(v) = req.get_string(&holding_id_key) {
            if let Ok(n) = v.parse::<u64>() {
                phasing.insert("holding".to_string(), serde_json::Value::Number(n.into()));
            }
        }

        if let Some(v) = req.get_string(&hashed_secret_key) {
            phasing.insert("hashedSecret".to_string(), serde_json::Value::String(v));
        }

        if let Some(v) = req.get_string(&hashed_secret_alg_key) {
            if let Ok(n) = v.parse::<i64>() {
                phasing.insert("hashedSecretAlgorithm".to_string(), serde_json::Value::Number(n.into()));
            }
        }

        let linked_key = if p.is_empty() { "phasingLinkedFullHash".to_string() } else { format!("{}PhasingLinkedFullHash", p) };
        if let Some(v) = Self::get_json_array(req, &linked_key).ok().flatten() {
            phasing.insert("linkedFullHashes".to_string(), v);
        }

        let whitelist_key = if p.is_empty() { "phasingWhitelisted".to_string() } else { format!("{}PhasingWhitelisted", p) };
        if let Some(v) = Self::get_json_array(req, &whitelist_key).ok().flatten() {
            phasing.insert("whitelist".to_string(), v);
        }

        Ok(Some(serde_json::Value::Object(phasing)))
    }

    // ==================== Bundler 过滤器解析 ====================

    pub fn get_bundling_filters(req: &ApiRequest) -> Result<Option<serde_json::Value>, ApiError> {
        Self::get_json_array(req, "bundlingFilters")
    }

    // ==================== EC 区块参数 ====================

    pub fn get_ec_block_id(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_unsigned_long(req, "ecBlockId", false)
    }

    pub fn get_ec_block_height(req: &ApiRequest) -> Result<i32, ApiError> {
        Self::get_int(req, "ecBlockHeight", 0, i32::MAX, false)
    }

    // ==================== Voucher 解析 ====================

    pub fn parse_voucher(req: &ApiRequest) -> Result<Option<serde_json::Value>, ApiError> {
        Self::get_json(req, "voucher")
    }

    // ==================== BigInteger 解析 ====================

    pub fn get_big_integer(req: &ApiRequest, name: &str, is_mandatory: bool) -> Result<Option<u64>, ApiError> {
        match req.get_string(name) {
            Some(s) => {
                let v = s.parse::<u64>().map_err(|_| {
                    ApiError::IncorrectParameter(name.to_string())
                })?;
                Ok(Some(v))
            }
            None => {
                if is_mandatory {
                    Err(ApiError::MissingParameter(name.to_string()))
                } else {
                    Ok(None)
                }
            }
        }
    }

    // ==================== Chain 参数 ====================

    pub fn get_chain_id(req: &ApiRequest) -> Result<u64, ApiError> {
        Self::get_unsigned_long(req, "chain", false)
    }
}

fn parse_account_id(account: &str) -> Result<u64, ApiError> {
    let trimmed = account.trim();
    if trimmed.is_empty() {
        return Ok(0);
    }

    let upper = trimmed.to_uppercase();

    if let Some(encoded) = upper.strip_prefix("NRCS-") {
        rs_decode(encoded)
    } else if let Some(dash_pos) = upper.find('-') {
        if dash_pos == 0 {
            trimmed[1..].parse::<u64>().map_err(|_| {
                ApiError::IncorrectParameter("account".to_string())
            })
        } else {
            let encoded = &upper[dash_pos + 1..];
            rs_decode(encoded)
        }
    } else {
        trimmed.parse::<u64>().map_err(|_| {
            ApiError::IncorrectParameter("account".to_string())
        })
    }
}

fn rs_decode(encoded: &str) -> Result<u64, ApiError> {
    let clean: String = encoded.chars().filter(|c| c.is_alphanumeric()).collect();
    if clean.is_empty() {
        return Err(ApiError::IncorrectParameter("account".to_string()));
    }
    match base58_decode(&clean) {
        Some(id) => Ok(id),
        None => Err(ApiError::IncorrectParameter("account".to_string())),
    }
}

fn base58_decode(s: &str) -> Option<u64> {
    const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut result: u64 = 0;
    for c in s.chars() {
        let idx = ALPHABET.iter().position(|&b| b as char == c)?;
        result = result.checked_mul(36)?.checked_add(idx as u64)?;
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_req(params: &[(&str, &str)]) -> ApiRequest {
        let mut map = HashMap::new();
        for (k, v) in params {
            map.insert(k.to_string(), v.to_string());
        }
        ApiRequest::new(map)
    }

    #[test]
    fn test_get_int_mandatory_present() {
        let req = make_req(&[("height", "100")]);
        let result = ParameterParser::get_int(&req, "height", 0, 10000, true);
        assert_eq!(result.unwrap(), 100);
    }

    #[test]
    fn test_get_int_mandatory_missing() {
        let req = make_req(&[]);
        let result = ParameterParser::get_int(&req, "height", 0, 10000, true);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code(), 3);
    }

    #[test]
    fn test_get_int_out_of_range() {
        let req = make_req(&[("height", "-1")]);
        let result = ParameterParser::get_int(&req, "height", 0, 10000, true);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code(), 4);
    }

    #[test]
    fn test_get_int_optional_missing() {
        let req = make_req(&[]);
        let result = ParameterParser::get_int(&req, "height", 0, 10000, false);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_get_unsigned_long_valid() {
        let req = make_req(&[("account", "123456789")]);
        let result = ParameterParser::get_unsigned_long(&req, "account", true);
        assert_eq!(result.unwrap(), 123456789);
    }

    #[test]
    fn test_get_unsigned_long_zero_rejected() {
        let req = make_req(&[("account", "0")]);
        let result = ParameterParser::get_unsigned_long(&req, "account", true);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code(), 4);
    }

    #[test]
    fn test_get_account_id_numeric() {
        let req = make_req(&[("account", "123456789")]);
        let result = ParameterParser::get_account_id(&req, true);
        assert_eq!(result.unwrap(), 123456789);
    }

    #[test]
    fn test_get_account_id_nrcs_prefix() {
        let req = make_req(&[("account", "NRCS-1234-5678-9ABC")]);
        let result = ParameterParser::get_account_id(&req, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_account_id_missing_mandatory() {
        let req = make_req(&[]);
        let result = ParameterParser::get_account_id(&req, true);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code(), 3);
    }

    #[test]
    fn test_get_fee_nqt_valid() {
        let req = make_req(&[("feeNQT", "100000000")]);
        let result = ParameterParser::get_fee_nqt(&req);
        assert_eq!(result.unwrap(), 100000000);
    }

    #[test]
    fn test_get_fee_nqt_zero_allowed() {
        let req = make_req(&[("feeNQT", "0")]);
        let result = ParameterParser::get_fee_nqt(&req);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_get_amount_nqt_minimum_one() {
        let req = make_req(&[("amountNQT", "0")]);
        let result = ParameterParser::get_amount_nqt(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_first_index_default() {
        let req = make_req(&[]);
        assert_eq!(ParameterParser::get_first_index(&req), 0);
    }

    #[test]
    fn test_get_last_index_default() {
        let req = make_req(&[]);
        assert_eq!(ParameterParser::get_last_index(&req), i32::MAX);
    }

    #[test]
    fn test_get_first_index_negative_becomes_zero() {
        let req = make_req(&[("firstIndex", "-5")]);
        assert_eq!(ParameterParser::get_first_index(&req), 0);
    }

    #[test]
    fn test_parse_account_id_empty() {
        assert_eq!(parse_account_id("").unwrap(), 0);
        assert_eq!(parse_account_id("  ").unwrap(), 0);
    }

    #[test]
    fn test_parse_account_id_numeric() {
        assert_eq!(parse_account_id("123456789").unwrap(), 123456789);
    }

    #[test]
    fn test_get_bytes_valid_hex() {
        let req = make_req(&[("data", "0102ff")]);
        let result = ParameterParser::get_bytes(&req, "data", true).unwrap();
        assert_eq!(result, Some(vec![1, 2, 255]));
    }

    #[test]
    fn test_get_bytes_invalid_hex() {
        let req = make_req(&[("data", "zzzz")]);
        let result = ParameterParser::get_bytes(&req, "data", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_codes_alignment() {
        assert_eq!(ApiError::MissingParameter("test".to_string()).error_code(), 3);
        assert_eq!(ApiError::IncorrectParameter("test".to_string()).error_code(), 4);
        assert_eq!(ApiError::UnknownObject("alias".to_string()).error_code(), 5);
        assert_eq!(ApiError::EitherParameter { params: "a, b".to_string() }.error_code(), 6);
        assert_eq!(ApiError::NotYetAvailable("test".to_string()).error_code(), 7);
        assert_eq!(ApiError::FeatureNotAvailable("test".to_string()).error_code(), 8);
        assert_eq!(ApiError::NotEnabled("test".to_string()).error_code(), 9);
        assert_eq!(ApiError::PrunedTransactionDataNotAvailable.error_code(), 15);
    }
}
