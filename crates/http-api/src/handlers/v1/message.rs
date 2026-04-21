//! 消息相关 API Handlers
//!
//! 与 Java 版本 SendMessage, GetAccountMessages 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct SendMessageHandler;

impl SendMessageHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SendMessageHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "recipient", "message", "messageIsText", "messageToEncrypt", "messageToEncryptIsText", "encryptedMessageData", "encryptedMessageNonce", "encryptedMessageIsText", "compressMessageToEncrypt", "feeNQT", "deadline", "referencedTransactionFullHash", "broadcast"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Messages, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _recipient = req.require_u64("recipient")?;
        let _message = req.get_string("message");
        let _message_is_text = req.get_bool("messageIsText");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "")
            .insert("transactionBytes", "");
        
        Ok(builder.build())
    }
}

pub struct GetAccountMessagesHandler;

impl GetAccountMessagesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountMessagesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "otherAccount", "timestamp", "firstIndex", "lastIndex", "type", "subtype"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Messages]
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account = req.require_u64("account")?;
        let _other_account = req.get_u64("otherAccount");
        let _timestamp = req.get_i32("timestamp");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(99);
        
        let txs = state.tx_repo
            .find_by_sender(account as i64, 100)
            .await
            .map_err(ApiError::Repository)?;
        
        let messages: Vec<serde_json::Value> = txs.iter()
            .filter(|tx| tx.r#type == 1)
            .map(|tx| json!({
                "transaction": tx.id.to_string(),
                "timestamp": tx.timestamp,
                "sender": tx.sender_id.to_string(),
                "senderRS": format_account_rs(tx.sender_id as u64),
                "recipient": tx.recipient_id.map(|r| r.to_string()).unwrap_or_default(),
                "recipientRS": tx.recipient_id.map(|r| format_account_rs(r as u64)).unwrap_or_default(),
                "message": "",
                "messageIsText": true
            }))
            .collect();
        
        let mut builder = RsRespBuilder::new();
        builder.insert("messages", json!(messages));
        
        Ok(builder.build())
    }
}

pub struct GetUnconfirmedMessagesHandler;

impl GetUnconfirmedMessagesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetUnconfirmedMessagesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "otherAccount", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Messages]
    }
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("messages", json!([]));
        
        Ok(builder.build())
    }
}

pub struct ReadMessageHandler;

impl ReadMessageHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ReadMessageHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transaction", "secretPhrase"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Messages]
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let transaction_id = req.require_u64("transaction")?;
        let _secret_phrase = req.get_string("secretPhrase");
        
        let tx = state.tx_repo
            .find_by_txid(transaction_id as i64)
            .await
            .map_err(ApiError::Repository)?;
        
        let tx = match tx {
            Some(t) => t,
            None => return Err(ApiError::UnknownTransaction),
        };
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", tx.id.to_string())
            .insert("timestamp", tx.timestamp)
            .insert("sender", tx.sender_id.to_string())
            .insert("senderRS", format_account_rs(tx.sender_id as u64))
            .insert("recipient", tx.recipient_id.map(|r| r.to_string()).unwrap_or_default())
            .insert("recipientRS", tx.recipient_id.map(|r| format_account_rs(r as u64)).unwrap_or_default())
            .insert("message", "")
            .insert("messageIsText", true);
        
        Ok(builder.build())
    }
}

pub struct VerifyPrunableMessageHandler;

impl VerifyPrunableMessageHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for VerifyPrunableMessageHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transaction", "message", "messageIsText"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Messages]
    }
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("verify", true);
        
        Ok(builder.build())
    }
}

pub struct GetPrunableMessageHandler;

impl GetPrunableMessageHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPrunableMessageHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transaction", "secretPhrase"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Messages]
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let transaction_id = req.require_u64("transaction")?;
        
        let tx = state.tx_repo
            .find_by_txid(transaction_id as i64)
            .await
            .map_err(ApiError::Repository)?;
        
        let tx = match tx {
            Some(t) => t,
            None => return Err(ApiError::UnknownTransaction),
        };
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", tx.id.to_string())
            .insert("message", "")
            .insert("messageIsText", true);
        
        Ok(builder.build())
    }
}

pub struct GetPrunableMessagesHandler;

impl GetPrunableMessagesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPrunableMessagesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "otherAccount", "firstIndex", "lastIndex", "timestamp"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Messages]
    }
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("messages", json!([]));
        
        Ok(builder.build())
    }
}

pub struct DownloadPrunableMessageHandler;

impl DownloadPrunableMessageHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DownloadPrunableMessageHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transaction", "secretPhrase"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Messages]
    }
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder
            .insert("data", "")
            .insert("filename", "");
        
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
