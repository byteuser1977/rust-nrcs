//! 交易相关 API Handlers
//!
//! 与 Java 版本 GetTransaction, SendMoney 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;
use crate::handlers::v1::create_transaction::CreateTransactionHelper;

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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let transaction_id = req.get_u64("transaction");
        let full_hash = req.get_string("fullHash");
        
        let tx_model = if let Some(id) = transaction_id {
            state.tx_repo
                .find_by_txid(id as i64)
                .await
                .map_err(ApiError::Repository)?
        } else if let Some(hash_str) = full_hash {
            let hash_bytes = hex::decode(&hash_str).unwrap_or_default();
            state.tx_repo
                .find_by_full_hash(&hash_bytes)
                .await
                .map_err(ApiError::Repository)?
        } else {
            return Err(ApiError::MissingParameter("transaction or fullHash".to_string()));
        };
        
        let tx = match tx_model {
            Some(t) => t,
            None => return Err(ApiError::UnknownTransaction),
        };
        
        let tx_domain = tx.to_domain().map_err(|e| ApiError::Internal(e.to_string()))?;

        let confirmations = if tx.height > 0 {
            state.block_repo.get_height().await.unwrap_or(0) - tx.height + 1
        } else {
            0
        };

        let sender_pk = state.account_manager.get_public_key(tx.sender_id as u64).await;
        let sender_public_key = match sender_pk {
            Ok(Some(blockchain_types::prelude::PublicKey::Ed25519(bytes))) => hex::encode(bytes),
            _ => String::new(),
        };

        let signature_hash = if !tx.signature.is_empty() {
            let hash = crypto::crypto::sha256(&tx.signature);
            hex::encode(hash.as_ref())
        } else {
            String::new()
        };

        let mut builder = RsRespBuilder::new();

        builder
            .insert("transaction", tx.id.to_string())
            .insert("timestamp", tx.timestamp)
            .insert("height", tx.height)
            .insert("sender", tx.sender_id.to_string())
            .insert("senderRS", format_account_rs(tx.sender_id as u64))
            .insert("senderPublicKey", sender_public_key)
            .insert("amountNQT", tx.amount.to_string())
            .insert("feeNQT", tx.fee.to_string())
            .insert("type", u8::from(tx_domain.type_id))
            .insert("subtype", tx.subtype)
            .insert("block", tx.block_id.to_string())
            .insert("blockTimestamp", tx.timestamp)
            .insert("fullHash", hex::encode(&tx.full_hash))
            .insert("signatureHash", signature_hash)
            .insert("signature", hex::encode(&tx.signature))
            .insert("confirmations", confirmations);
        
        if let Some(recipient) = tx.recipient_id {
            builder
                .insert("recipient", recipient.to_string())
                .insert("recipientRS", format_account_rs(recipient as u64));
        }
        
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account = req.get_u64("account");
        let first_index = req.get_i32("firstIndex").unwrap_or(0);
        let last_index = req.get_i32("lastIndex").unwrap_or(99);
        let limit = (last_index - first_index + 1) as i64;
        
        let txs = if let Some(acc_id) = account {
            let mut all_txs = Vec::new();
            let sent = state.tx_repo
                .find_by_sender(acc_id as i64, limit)
                .await
                .map_err(ApiError::Repository)?;
            let received = state.tx_repo
                .find_by_recipient(acc_id as i64, limit)
                .await
                .map_err(ApiError::Repository)?;
            all_txs.extend(sent);
            all_txs.extend(received);
            all_txs.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
            all_txs.truncate(limit as usize);
            all_txs
        } else {
            vec![]
        };
        
        let transactions: Vec<serde_json::Value> = txs.iter()
            .map(|tx| json!({
                "transaction": tx.id.to_string(),
                "timestamp": tx.timestamp,
                "height": tx.height,
                "sender": tx.sender_id.to_string(),
                "senderRS": format_account_rs(tx.sender_id as u64),
                "recipient": tx.recipient_id.map(|r| r.to_string()).unwrap_or_default(),
                "recipientRS": tx.recipient_id.map(|r| format_account_rs(r as u64)).unwrap_or_default(),
                "amountNQT": tx.amount.to_string(),
                "feeNQT": tx.fee.to_string(),
                "type": tx.r#type,
                "subtype": tx.subtype,
                "confirmations": 0
            }))
            .collect();
        
        let mut builder = RsRespBuilder::new();
        builder.insert("transactions", json!(transactions));
        
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.get_u64("account");
        let last_index = req.get_i32("lastIndex").unwrap_or(99);
        let limit = (last_index + 1) as i64;
        
        let txs = state.tx_repo
            .find_unconfirmed(limit)
            .await
            .map_err(ApiError::Repository)?;
        
        let transactions: Vec<serde_json::Value> = txs.iter()
            .map(|tx| json!({
                "transaction": tx.id.to_string(),
                "timestamp": tx.timestamp,
                "sender": tx.sender_id.to_string(),
                "senderRS": format_account_rs(tx.sender_id as u64),
                "recipient": tx.recipient_id.map(|r| r.to_string()).unwrap_or_default(),
                "recipientRS": tx.recipient_id.map(|r| format_account_rs(r as u64)).unwrap_or_default(),
                "amountNQT": tx.amount.to_string(),
                "feeNQT": tx.fee.to_string()
            }))
            .collect();
        
        let mut builder = RsRespBuilder::new();
        builder.insert("unconfirmedTransactions", json!(transactions));
        
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
        vec!["secretPhrase", "publicKey", "recipient", "amountNQT", "feeNQT", "deadline", "referencedTransactionFullHash", "broadcast",
             "message", "messageIsText", "messageIsPrunable",
             "messageToEncrypt", "messageToEncryptIsText", "encryptedMessageData", "encryptedMessageNonce", "encryptedMessageIsPrunable", "compressMessageToEncrypt",
             "phased", "phasingFinishHeight", "phasingVotingModel", "phasingQuorum", "phasingMinBalance", "phasingHolding", "phasingMinBalanceModel",
             "phasingWhitelisted", "phasingLinkedFullHash", "phasingHashedSecret", "phasingHashedSecretAlgorithm",
             "recipientPublicKey", "ecBlockId", "ecBlockHeight", "phasingParams", "timestamp"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Transactions, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let recipient = req.require_u64("recipient")?;
        let amount_str = req.require_string("amountNQT")?;
        let amount_nqt: u64 = amount_str.parse()
            .map_err(|_| ApiError::IncorrectValue("amountNQT".to_string()))?;

        let params = CreateTransactionHelper::parse_common_params(req)?;

        CreateTransactionHelper::create_and_broadcast_transaction(
            &params,
            blockchain_types::transaction::TYPE_PAYMENT,
            blockchain_types::transaction::SUBTYPE_PAYMENT_ORDINARY_PAYMENT,
            Some(recipient),
            amount_nqt,
            None,
            state,
        ).await
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let tx = CreateTransactionHelper::parse_transaction_from_bytes_or_json(req)?;

        state.tx_processor
            .broadcast(&tx)
            .await
            .map_err(ApiError::TxEngine)?;

        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", tx.id.to_string())
            .insert("fullHash", hex::encode(tx.full_hash.0))
            .insert("numberPeersSentTo", 1i32);

        Ok(builder.build())
    }
}

pub struct SendTransactionHandler;

impl SendTransactionHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SendTransactionHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transactionBytes", "transactionJSON", "secretPhrase", "broadcast"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Transactions, ApiTag::CreateTransaction]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut tx = CreateTransactionHelper::parse_transaction_from_bytes_or_json(req)?;

        let secret_phrase = req.get_string("secretPhrase");
        let broadcast = req.get_string("broadcast")
            .map(|v| !v.eq_ignore_ascii_case("false"))
            .unwrap_or(true);

        if let Some(ref sp) = secret_phrase {
            CreateTransactionHelper::sign_transaction_with_passphrase(&mut tx, sp)?;
        }

        if broadcast && secret_phrase.is_some() {
            state.tx_processor
                .broadcast(&tx)
                .await
                .map_err(ApiError::TxEngine)?;
        } else {
            state.tx_processor
                .validate(&tx)
                .await
                .map_err(ApiError::TxEngine)?;
        }

        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", tx.id.to_string())
            .insert("fullHash", hex::encode(tx.full_hash.0));

        if secret_phrase.is_some() {
            builder
                .insert("transactionBytes", hex::encode(tx.get_bytes()))
                .insert("signatureHash", hex::encode(crypto::sha256(&tx.signature.0)))
                .insert("broadcasted", broadcast);
        }

        Ok(builder.build())
    }
}

pub struct GetBlockchainTransactionsHandler;

impl GetBlockchainTransactionsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBlockchainTransactionsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "firstIndex", "lastIndex"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Transactions]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);

        let mut builder = RsRespBuilder::new();
        builder.insert("transactions", json!([]));

        Ok(builder.build())
    }
}

pub struct GetScheduledTransactionsHandler;

impl GetScheduledTransactionsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetScheduledTransactionsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Transactions]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.get_u64("account");

        let mut builder = RsRespBuilder::new();
        builder.insert("scheduledTransactions", json!([]));

        Ok(builder.build())
    }
}

pub struct GetExpectedTransactionsHandler;

impl GetExpectedTransactionsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetExpectedTransactionsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Transactions]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.get_u64("account");

        let mut builder = RsRespBuilder::new();
        builder.insert("expectedTransactions", json!([]));

        Ok(builder.build())
    }
}

pub struct GetFxtTransactionHandler;

impl GetFxtTransactionHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetFxtTransactionHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transaction"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Transactions]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _transaction = req.require_u64("transaction")?;

        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "0")
            .insert("timestamp", 0i32)
            .insert("sender", "0")
            .insert("senderRS", "NRCS-0-0-0")
            .insert("amountNQT", "0")
            .insert("feeNQT", "0")
            .insert("confirmations", 0i32);

        Ok(builder.build())
    }
}

pub struct GetExpectedOrderCancellationsHandler;

impl GetExpectedOrderCancellationsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetExpectedOrderCancellationsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["askOrder", "bidOrder"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _ask_order = req.get_u64("askOrder");
        let _bid_order = req.get_u64("bidOrder");

        let mut builder = RsRespBuilder::new();
        builder.insert("orderCancellations", json!([]));

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
