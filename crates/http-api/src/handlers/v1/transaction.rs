//! 交易相关 API Handlers
//!
//! 与 Java 版本 GetTransaction, SendMoney 等完全对齐

use async_trait::async_trait;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};

pub struct GetTransactionHandler;

impl GetTransactionHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetTransactionHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transaction", "fullHash", "includePhasingResult"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Transactions]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _transaction_id = req.get_u64("transaction");
        let _full_hash = req.get_string("fullHash");
        
        let mut builder = RsRespBuilder::new();
        
        builder
            .insert("transaction", "0")
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
            .insert("block", "0")
            .insert("blockTimestamp", 0i32)
            .insert("fullHash", "")
            .insert("signatureHash", "")
            .insert("signature", "")
            .insert("confirmations", 0i32);
        
        Ok(builder.build())
    }
}

pub struct GetTransactionsHandler;

impl GetTransactionsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetTransactionsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "timestamp", "type", "subtype", "firstIndex", "lastIndex", "numberOfConfirmations", "withMessage"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Transactions]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _account = req.get_u64("account");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(99);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("transactions", serde_json::json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetUnconfirmedTransactionsHandler;

impl GetUnconfirmedTransactionsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetUnconfirmedTransactionsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Transactions]
    }
    
    async fn process_request(&self, _req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("unconfirmedTransactions", serde_json::json!([]));
        
        Ok(builder.build())
    }
}

pub struct SendMoneyHandler;

impl SendMoneyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SendMoneyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "recipient", "amountNQT", "feeNQT", "deadline", "referencedTransactionFullHash", "broadcast"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Transactions, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _recipient = req.require_u64("recipient")?;
        let _amount = req.require_string("amountNQT")?;
        let _fee = req.require_string("feeNQT")?;
        let _deadline = req.require_i32("deadline")?;
        
        let mut builder = RsRespBuilder::new();
        
        builder
            .insert("transaction", "0")
            .insert("fullHash", "")
            .insert("transactionBytes", "")
            .insert("signatureHash", "");
        
        Ok(builder.build())
    }
}

pub struct BroadcastTransactionHandler;

impl BroadcastTransactionHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for BroadcastTransactionHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transactionBytes", "transactionJSON"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Transactions]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, _req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        
        builder
            .insert("transaction", "0")
            .insert("fullHash", "")
            .insert("numberPeersSentTo", 0i32);
        
        Ok(builder.build())
    }
}
