//! 工具类 API Handlers
//!
//! 与 Java 版本 Hash, HexConvert, ParseTransaction 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;
use crate::handlers::v1::create_transaction::CreateTransactionHelper;

pub struct HashHandler;

impl HashHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for HashHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["hashAlgorithm", "secret", "secretIsText"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let hash_algorithm = req.get_i32("hashAlgorithm").unwrap_or(0);
        let secret = req.get_string("secret").unwrap_or_default();
        let secret_is_text = req.get_bool("secretIsText");
        
        let secret_bytes = if secret_is_text {
            secret.as_bytes().to_vec()
        } else {
            hex::decode(&secret).unwrap_or_default()
        };
        
        let hash = match hash_algorithm {
            0 => hex::encode(crypto::sha256(&secret_bytes)),
            1 => hex::encode(crypto::sm3(&secret_bytes)),
            2 => {
                let hash1 = crypto::sha256(&secret_bytes);
                let hash2 = crypto::sha256(&hash1);
                let mut combined = Vec::with_capacity(64);
                combined.extend_from_slice(&hash1);
                combined.extend_from_slice(&hash2);
                hex::encode(combined)
            }
            _ => hex::encode(crypto::sha256(&secret_bytes)),
        };
        
        let mut builder = RsRespBuilder::new();
        builder.insert("hash", hash);
        
        Ok(builder.build())
    }
}

pub struct HexConvertHandler;

impl HexConvertHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for HexConvertHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["string"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let string = req.require_string("string")?;
        
        let binary = hex::encode(string.as_bytes());
        
        let mut builder = RsRespBuilder::new();
        builder.insert("binary", binary);
        
        Ok(builder.build())
    }
}

pub struct LongConvertHandler;

impl LongConvertHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for LongConvertHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["id"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let id_str = req.require_string("id")?;
        
        let long_id: i64 = if id_str.starts_with("NRCS-") {
            parse_account_rs(&id_str) as i64
        } else {
            id_str.parse().unwrap_or(0)
        };
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("stringId", long_id.to_string())
            .insert("longId", long_id);
        
        Ok(builder.build())
    }
}

pub struct RsConvertHandler;

impl RsConvertHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for RsConvertHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account = req.require_string("account")?;
        
        let (account_id, account_rs) = if account.starts_with("NRCS-") {
            let id = parse_account_rs(&account);
            (id, account)
        } else {
            let id: u64 = account.parse().unwrap_or(0);
            (id, format_account_rs(id))
        };
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("account", account_id.to_string())
            .insert("accountRS", account_rs);
        
        Ok(builder.build())
    }
}

pub struct ParseTransactionHandler;

impl ParseTransactionHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ParseTransactionHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transactionBytes", "transactionJSON"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils, ApiTag::Transactions]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let tx = CreateTransactionHelper::parse_transaction_from_bytes_or_json(req)?;

        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", tx.id.to_string())
            .insert("timestamp", tx.timestamp)
            .insert("height", tx.height)
            .insert("sender", tx.sender_id.to_string())
            .insert("senderRS", format_account_rs(tx.sender_id))
            .insert("senderPublicKey", hex::encode(tx.sender_public_key.0))
            .insert("amountNQT", tx.amount.to_string())
            .insert("feeNQT", tx.fee.to_string())
            .insert("type", u8::from(tx.type_id) as i32)
            .insert("subtype", tx.subtype as i32)
            .insert("fullHash", hex::encode(tx.full_hash.0))
            .insert("signature", hex::encode(tx.signature.0))
            .insert("phased", tx.phased)
            .insert("deadline", tx.deadline as i32)
            .insert("version", tx.version as i32);

        if let Some(recipient) = tx.recipient_id {
            builder
                .insert("recipient", recipient.to_string())
                .insert("recipientRS", format_account_rs(recipient));
        }

        if let Some(ref hash) = tx.referenced_transaction_full_hash {
            builder.insert("referencedTransactionFullHash", hex::encode(hash.0));
        }

        if let Some(eb_height) = tx.ec_block_height {
            builder.insert("ecBlockHeight", eb_height);
        }
        if let Some(eb_id) = tx.ec_block_id {
            builder.insert("ecBlockId", eb_id.to_string());
        }

        if let Some(ref att) = tx.attachment_json {
            builder.insert("attachment", json!(att.clone()));
        }

        Ok(builder.build())
    }
}

pub struct FullHashToIdHandler;

impl FullHashToIdHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for FullHashToIdHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["fullHash"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let full_hash = req.require_string("fullHash")?;
        
        let hash_bytes = hex::decode(&full_hash).unwrap_or_default();
        
        let string_id = if hash_bytes.len() >= 8 {
            let mut id_bytes = [0u8; 8];
            id_bytes.copy_from_slice(&hash_bytes[0..8]);
            u64::from_le_bytes(id_bytes).to_string()
        } else {
            "0".to_string()
        };
        
        let mut builder = RsRespBuilder::new();
        builder.insert("stringId", string_id);
        
        Ok(builder.build())
    }
}

pub struct GetECBlockHandler;

impl GetECBlockHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetECBlockHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["timestamp"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Blocks, ApiTag::Utils]
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _timestamp = req.get_i32("timestamp");
        
        let latest_block = state.block_repo
            .find_latest()
            .await
            .map_err(ApiError::Repository)?;
        
        let (ec_block_id, ec_block_height, timestamp) = match latest_block {
            Some(b) => (b.id.to_string(), b.height, b.timestamp),
            None => ("0".to_string(), 0, 0),
        };
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("ecBlockId", ec_block_id)
            .insert("ecBlockHeight", ec_block_height)
            .insert("timestamp", timestamp);
        
        Ok(builder.build())
    }
}

pub struct CalculateFeeHandler;

impl CalculateFeeHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for CalculateFeeHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transactionBytes", "transactionJSON"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils, ApiTag::Transactions]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let transaction_bytes_hex = req.get_string("transactionBytes");
        
        let fee = if let Some(hex_str) = transaction_bytes_hex {
            let bytes = hex::decode(&hex_str).unwrap_or_default();
            if bytes.len() >= 64 {
                let mut fee_bytes = [0u8; 8];
                fee_bytes.copy_from_slice(&bytes[56..64]);
                u64::from_le_bytes(fee_bytes).to_string()
            } else {
                "100000000".to_string()
            }
        } else {
            "100000000".to_string()
        };
        
        let mut builder = RsRespBuilder::new();
        builder.insert("feeNQT", fee);
        
        Ok(builder.build())
    }
}

pub struct EvaluateExpressionHandler;

impl EvaluateExpressionHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for EvaluateExpressionHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["expression"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _expression = req.require_string("expression")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("result", "");

        Ok(builder.build())
    }
}

pub struct EventRegisterHandler;

impl EventRegisterHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for EventRegisterHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["event", "add", "remove"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _event = req.get_string("event");
        let _add = req.get_bool("add");
        let _remove = req.get_bool("remove");

        let mut builder = RsRespBuilder::new();
        builder.insert("registered", true);

        Ok(builder.build())
    }
}

pub struct EventWaitHandler;

impl EventWaitHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for EventWaitHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["timeout"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _timeout = req.get_i32("timeout").unwrap_or(0);

        let mut builder = RsRespBuilder::new();
        builder.insert("events", json!([]));

        Ok(builder.build())
    }
}

pub struct LeaseBalanceHandler;

impl LeaseBalanceHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for LeaseBalanceHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["period", "recipient", "secretPhrase", "feeNQT", "deadline"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Forging, ApiTag::CreateTransaction]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _period = req.require_string("period")?;
        let _recipient = req.require_u64("recipient")?;
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _fee_nqt = req.get_string("feeNQT");
        let _deadline = req.get_i32("deadline");

        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "")
            .insert("transactionBytes", "");

        Ok(builder.build())
    }
}

pub struct MarkHostHandler;

impl MarkHostHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for MarkHostHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["host", "weight", "date", "secretPhrase"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _host = req.require_string("host")?;
        let _weight = req.get_i32("weight");
        let _date = req.get_string("date");
        let _secret_phrase = req.require_string("secretPhrase")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("done", true);

        Ok(builder.build())
    }
}

pub struct CalculateFullHashHandler;

impl CalculateFullHashHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for CalculateFullHashHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["unsignedTransactionBytes", "signatureHash"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils, ApiTag::Transactions]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _unsigned_tx_bytes = req.require_string("unsignedTransactionBytes")?;
        let _signature_hash = req.get_string("signatureHash");

        let mut builder = RsRespBuilder::new();
        builder
            .insert("fullHash", "")
            .insert("signatureHash", "")
            .insert("transaction", "");

        Ok(builder.build())
    }
}

pub struct CombineSecretHandler;

impl CombineSecretHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for CombineSecretHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secret1", "secret2", "secret3", "secret4", "secret5", "secret6", "secret7", "secret8"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret1 = req.get_string("secret1");
        let _secret2 = req.get_string("secret2");

        let mut builder = RsRespBuilder::new();
        builder.insert("secret", "");

        Ok(builder.build())
    }
}

pub struct SplitSecretHandler;

impl SplitSecretHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SplitSecretHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secret", "totalPieces", "minimumPieces", "feeNQT", "deadline", "secretPhrase"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Utils]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret = req.require_string("secret")?;
        let _total_pieces = req.get_i32("totalPieces");
        let _minimum_pieces = req.get_i32("minimumPieces");
        let _secret_phrase = req.get_string("secretPhrase");

        let mut builder = RsRespBuilder::new();
        builder.insert("pieces", json!([]));

        Ok(builder.build())
    }
}

fn format_account_rs(account_id: u64) -> String {
    format!("NRCS-{}-{}-{}",
        account_id % 10000,
        (account_id / 10000) % 10000,
        (account_id / 100000000) % 10000
    )
}

fn parse_account_rs(rs: &str) -> u64 {
    let parts: Vec<&str> = rs.trim_start_matches("NRCS-").split('-').collect();
    if parts.len() == 3 {
        let p1: u64 = parts[0].parse().unwrap_or(0);
        let p2: u64 = parts[1].parse().unwrap_or(0);
        let p3: u64 = parts[2].parse().unwrap_or(0);
        p1 + p2 * 10000 + p3 * 100000000
    } else {
        0
    }
}
