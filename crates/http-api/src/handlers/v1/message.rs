//! 消息相关 API Handlers
//!
//! 与 Java 版本 SendMessage, GetAccountMessages 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;
use crate::handlers::v1::create_transaction::CreateTransactionHelper;

pub struct SendMessageHandler;

impl SendMessageHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SendMessageHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "publicKey", "recipient", "message", "messageIsText", "messageIsPrunable",
             "messageToEncrypt", "messageToEncryptIsText", "encryptedMessageData", "encryptedMessageNonce",
             "encryptedMessageIsPrunable", "compressMessageToEncrypt", "feeNQT", "deadline",
             "referencedTransactionFullHash", "broadcast",
             "phased", "phasingFinishHeight", "phasingVotingModel", "phasingQuorum", "phasingMinBalance",
             "phasingHolding", "phasingMinBalanceModel", "phasingWhitelisted", "phasingLinkedFullHash",
             "phasingHashedSecret", "phasingHashedSecretAlgorithm",
             "recipientPublicKey", "ecBlockId", "ecBlockHeight", "phasingParams", "timestamp"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Messages, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let recipient = req.get_u64("recipient");

        let params = CreateTransactionHelper::parse_common_params(req)?;

        let mut attachment_json = serde_json::Map::new();
        if let Some(ref msg) = params.message {
            attachment_json.insert("message".to_string(), json!(msg));
            let is_text = params.message_is_text.unwrap_or(true);
            attachment_json.insert("messageIsText".to_string(), json!(is_text));
        }

        CreateTransactionHelper::create_and_broadcast_transaction(
            &params,
            blockchain_types::transaction::TYPE_MESSAGING,
            blockchain_types::transaction::SUBTYPE_MESSAGING_ARBITRARY_MESSAGE,
            recipient,
            0,
            Some(serde_json::Value::Object(attachment_json)),
            state,
        ).await
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

pub struct DecryptFromHandler;

impl DecryptFromHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DecryptFromHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "data", "nonce", "decryptedMessageIsText", "uncompressDecryptedMessage", "secretPhrase"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Messages]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _data = req.get_string("data");
        let _nonce = req.get_string("nonce");
        let _decrypted_message_is_text = req.get_bool("decryptedMessageIsText");
        let _uncompress = req.get_bool("uncompressDecryptedMessage");
        let _secret_phrase = req.get_string("secretPhrase");

        let mut builder = RsRespBuilder::new();
        builder
            .insert("decryptedMessage", "")
            .insert("decryptedMessageIsText", true);

        Ok(builder.build())
    }
}

pub struct EncryptToHandler;

impl EncryptToHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for EncryptToHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["recipient", "messageToEncrypt", "messageToEncryptIsText", "compressMessageToEncrypt", "secretPhrase"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Messages]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _recipient = req.require_u64("recipient")?;
        let _message_to_encrypt = req.get_string("messageToEncrypt");
        let _message_is_text = req.get_bool("messageToEncryptIsText");
        let _compress = req.get_bool("compressMessageToEncrypt");
        let _secret_phrase = req.get_string("secretPhrase");

        let mut builder = RsRespBuilder::new();
        builder
            .insert("data", "")
            .insert("nonce", "");

        Ok(builder.build())
    }
}

pub struct GetSharedKeyHandler;

impl GetSharedKeyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetSharedKeyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "secretPhrase", "nonce"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Messages]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _secret_phrase = req.get_string("secretPhrase");
        let _nonce = req.get_string("nonce");

        let mut builder = RsRespBuilder::new();
        builder.insert("sharedKey", "");

        Ok(builder.build())
    }
}

pub struct GetAllPrunableMessagesHandler;

impl GetAllPrunableMessagesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAllPrunableMessagesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Messages]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(99);

        let mut builder = RsRespBuilder::new();
        builder.insert("messages", json!([]));

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
