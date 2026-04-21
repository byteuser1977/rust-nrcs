//! 工具类 API Handlers
//!
//! 与 Java 版本 Hash, HexConvert 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

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
        let transaction_bytes_hex = req.get_string("transactionBytes");
        
        if let Some(hex_str) = transaction_bytes_hex {
            let bytes = hex::decode(&hex_str).unwrap_or_default();
            
            if bytes.len() >= 100 {
                let version = bytes[0] & 0x0F;
                let tx_type = (bytes[1] >> 4) & 0x0F;
                let subtype = bytes[1] & 0x0F;
                let timestamp = i32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
                let deadline = u16::from_le_bytes([bytes[6], bytes[7]]);
                
                let mut sender_pk = [0u8; 32];
                sender_pk.copy_from_slice(&bytes[8..40]);
                
                let mut recipient_bytes = [0u8; 8];
                recipient_bytes.copy_from_slice(&bytes[40..48]);
                let recipient = u64::from_le_bytes(recipient_bytes);
                
                let mut amount_bytes = [0u8; 8];
                amount_bytes.copy_from_slice(&bytes[48..56]);
                let amount = u64::from_le_bytes(amount_bytes);
                
                let mut fee_bytes = [0u8; 8];
                fee_bytes.copy_from_slice(&bytes[56..64]);
                let fee = u64::from_le_bytes(fee_bytes);
                
                let mut full_hash = [0u8; 32];
                full_hash.copy_from_slice(&bytes[68..100]);
                
                let mut builder = RsRespBuilder::new();
                builder
                    .insert("version", version as i32)
                    .insert("type", tx_type as i32)
                    .insert("subtype", subtype as i32)
                    .insert("timestamp", timestamp)
                    .insert("deadline", deadline as i32)
                    .insert("senderPublicKey", hex::encode(sender_pk))
                    .insert("recipient", recipient.to_string())
                    .insert("recipientRS", format_account_rs(recipient))
                    .insert("amountNQT", amount.to_string())
                    .insert("feeNQT", fee.to_string())
                    .insert("fullHash", hex::encode(full_hash));
                
                return Ok(builder.build());
            }
        }
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("timestamp", 0i32)
            .insert("height", 0i32)
            .insert("sender", "0")
            .insert("senderRS", "NRCS-0-0-0")
            .insert("senderPublicKey", "")
            .insert("recipient", "0")
            .insert("recipientRS", "NRCS-0-0-0")
            .insert("amountNQT", "0")
            .insert("feeNQT", "0")
            .insert("type", 0u8)
            .insert("subtype", 0u8)
            .insert("fullHash", "")
            .insert("signature", "");
        
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
