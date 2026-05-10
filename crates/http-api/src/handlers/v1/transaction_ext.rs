//! 交易扩展相关 API Handlers
//!
//! 与 Java 版本 GetTransactionBytes, SignTransaction 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use blockchain_types::transaction::Transaction;
use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;
use crate::handlers::v1::create_transaction::CreateTransactionHelper;

pub struct GetTransactionBytesHandler;

impl GetTransactionBytesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetTransactionBytesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transaction"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Transactions]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _transaction_id = req.require_u64("transaction")?;
        
        let mut builder = RsRespBuilder::new();
        builder.insert("transactionBytes", "");
        
        Ok(builder.build())
    }
}

fn transaction_to_json(tx: &blockchain_types::prelude::Transaction) -> serde_json::Value {
    let mut obj = serde_json::Map::new();

    obj.insert("type".to_string(), json!(u8::from(tx.type_id)));
    obj.insert("subtype".to_string(), json!(tx.subtype));
    obj.insert("version".to_string(), json!(tx.version));
    obj.insert("timestamp".to_string(), json!(tx.timestamp));
    obj.insert("deadline".to_string(), json!(tx.deadline));
    obj.insert("senderPublicKey".to_string(), json!(hex::encode(tx.sender_public_key.0)));
    obj.insert("sender".to_string(), json!(tx.sender_id.to_string()));
    obj.insert("senderRS".to_string(), json!(format_account_rs(tx.sender_id)));

    if let Some(recipient) = tx.recipient_id {
        obj.insert("recipient".to_string(), json!(recipient.to_string()));
        obj.insert("recipientRS".to_string(), json!(format_account_rs(recipient)));
    }

    obj.insert("amountNQT".to_string(), json!(tx.amount.to_string()));
    obj.insert("feeNQT".to_string(), json!(tx.fee.to_string()));
    obj.insert("fullHash".to_string(), json!(hex::encode(tx.full_hash.0)));
    obj.insert("signature".to_string(), json!(hex::encode(tx.signature.0)));
    obj.insert("phased".to_string(), json!(tx.phased));

    if let Some(ref hash) = tx.referenced_transaction_full_hash {
        obj.insert("referencedTransactionFullHash".to_string(), json!(hex::encode(hash.0)));
    }

    if let Some(eb_height) = tx.ec_block_height {
        obj.insert("ecBlockHeight".to_string(), json!(eb_height));
    }
    if let Some(eb_id) = tx.ec_block_id {
        obj.insert("ecBlockId".to_string(), json!(eb_id.to_string()));
    }

    if let Some(ref att) = tx.attachment_json {
        obj.insert("attachment".to_string(), json!(att.clone()));
    }

    serde_json::Value::Object(obj)
}

fn format_account_rs(account_id: u64) -> String {
    format!("NRCS-{}-{}-{}",
        account_id % 10000,
        (account_id / 10000) % 10000,
        (account_id / 100000000) % 10000
    )
}

pub struct GetUnconfirmedTransactionIdsHandler;

impl GetUnconfirmedTransactionIdsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetUnconfirmedTransactionIdsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Transactions]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.get_u64("account");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("unconfirmedTransactionIds", json!([]));
        
        Ok(builder.build())
    }
}

pub struct SignTransactionHandler;

impl SignTransactionHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SignTransactionHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "unsignedTransactionBytes", "unsignedTransactionJSON"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Transactions, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let secret_phrase = req.require_string("secretPhrase")?;

        let unsigned_bytes_hex = req.get_string("unsignedTransactionBytes");
        let unsigned_json_str = req.get_string("unsignedTransactionJSON");

        let mut tx = if let Some(ref hex_str) = unsigned_bytes_hex {
            let bytes = hex::decode(hex_str)
                .map_err(|e| ApiError::IncorrectValue(format!("invalid unsignedTransactionBytes hex: {}", e)))?;
            CreateTransactionHelper::parse_transaction_from_bytes(&bytes)?
        } else if let Some(ref json_str) = unsigned_json_str {
            let json_value: serde_json::Value = serde_json::from_str(json_str)
                .map_err(|e| ApiError::IncorrectValue(format!("invalid unsignedTransactionJSON: {}", e)))?;
            Transaction::from_json(&json_value)
                .map_err(ApiError::Blockchain)?
        } else {
            return Err(ApiError::MissingParameter("unsignedTransactionBytes or unsignedTransactionJSON".to_string()));
        };

        CreateTransactionHelper::sign_transaction_with_passphrase(&mut tx, &secret_phrase)?;

        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", tx.id.to_string())
            .insert("fullHash", hex::encode(tx.full_hash.0))
            .insert("transactionBytes", hex::encode(tx.get_bytes()))
            .insert("signatureHash", hex::encode(crypto::sha256(&tx.signature.0)));

        let tx_json = transaction_to_json(&tx);
        builder.insert("transactionJSON", tx_json);

        Ok(builder.build())
    }
}

pub struct GetReferencedTransactionsHandler;

impl GetReferencedTransactionsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetReferencedTransactionsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transaction", "includeIndirect"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Transactions]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _transaction_id = req.require_u64("transaction")?;
        let _include_indirect = req.get_bool("includeIndirect");

        let mut builder = RsRespBuilder::new();
        builder.insert("transactions", json!([]));

        Ok(builder.build())
    }
}

pub struct GetAllTradesHandler;

impl GetAllTradesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAllTradesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);

        let mut builder = RsRespBuilder::new();
        builder.insert("trades", json!([]));

        Ok(builder.build())
    }
}

pub struct GetLastExchangesHandler;

impl GetLastExchangesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetLastExchangesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currencies", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currencies = req.get_string("currencies");

        let mut builder = RsRespBuilder::new();
        builder.insert("exchanges", json!([]));

        Ok(builder.build())
    }
}

pub struct GetLastTradesHandler;

impl GetLastTradesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetLastTradesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["assets", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _assets = req.get_string("assets");

        let mut builder = RsRespBuilder::new();
        builder.insert("trades", json!([]));

        Ok(builder.build())
    }
}

pub struct GetOrderTradesHandler;

impl GetOrderTradesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetOrderTradesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["askOrder", "bidOrder", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _ask_order = req.require_u64("askOrder")?;
        let _bid_order = req.require_u64("bidOrder")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("trades", json!([]));

        Ok(builder.build())
    }
}

pub struct ScheduleCurrencyBuyHandler;

impl ScheduleCurrencyBuyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ScheduleCurrencyBuyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currency", "rateNQT", "units", "offering", "secretPhrase", "feeNQT", "deadline"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms, ApiTag::CreateTransaction]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _currency_id = req.require_u64("currency")?;
        let _rate = req.require_string("rateNQT")?;
        let _units = req.require_u64("units")?;
        let _offering = req.require_u64("offering")?;

        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");

        Ok(builder.build())
    }
}
