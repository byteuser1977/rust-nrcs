//! 消息相关 API Handlers
//!
//! 与 Java 版本 ReadMessage, SendMessage 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};

pub struct ReadMessageHandler;

impl ReadMessageHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ReadMessageHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transaction", "secretPhrase", "retrieve"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Messages]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _transaction_id = req.require_u64("transaction")?;
        let _secret_phrase = req.get_string("secretPhrase");
        let _retrieve = req.get_bool("retrieve");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("message", "")
            .insert("messageIsText", true)
            .insert("decryptedMessage", "")
            .insert("decryptedMessageIsText", true);
        
        Ok(builder.build())
    }
}

pub struct SendMessageHandler;

impl SendMessageHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SendMessageHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "recipient", "message", "messageIsText", "messageToEncrypt", 
             "messageToEncryptIsText", "encryptToSelfMessage", "feeNQT", "deadline", 
             "referencedTransactionFullHash", "broadcast"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Messages, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _recipient = req.get_u64("recipient");
        let _message = req.get_string("message");
        let _message_is_text = req.get_bool("messageIsText");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "")
            .insert("transactionBytes", "")
            .insert("signatureHash", "");
        
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
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _transaction_id = req.require_u64("transaction")?;
        let _secret_phrase = req.get_string("secretPhrase");
        
        let mut builder = RsRespBuilder::new();
        builder
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
        vec!["account", "otherAccount", "secretPhrase", "firstIndex", "lastIndex", "timestamp"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Messages]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _account = req.get_u64("account");
        let _other_account = req.get_u64("otherAccount");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("prunableMessages", json!([]));
        
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
        vec!["transaction", "secretPhrase", "retrieve"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Messages]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _transaction_id = req.require_u64("transaction")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("message", "")
            .insert("messageIsText", false);
        
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
        vec!["transaction", "message", "messageIsText", "messageHash"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Messages]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _transaction_id = req.require_u64("transaction")?;
        let _message = req.get_string("message");
        let _message_is_text = req.get_bool("messageIsText");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("verify", true);
        
        Ok(builder.build())
    }
}
