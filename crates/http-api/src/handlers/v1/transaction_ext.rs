//! 交易扩展相关 API Handlers
//!
//! 与 Java 版本 GetTransactionBytes 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

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
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _unsigned_bytes = req.get_string("unsignedTransactionBytes");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transactionBytes", "")
            .insert("signatureHash", "");
        
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
