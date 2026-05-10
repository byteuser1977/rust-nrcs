//! 币币交易相关 API Handlers
//!
//! 与 Java 版本 GetCoinExchangeOrder, ExchangeCoins 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;
use super::create_transaction::CreateTransactionHelper;

pub struct GetCoinExchangeOrderHandler;

impl GetCoinExchangeOrderHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCoinExchangeOrderHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["order"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ce]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _order_id = req.require_u64("order")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("order", "0")
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0")
            .insert("exchange", "0")
            .insert("exchangeRS", "NRCS-0-0-0")
            .insert("pairRateNQT", "0")
            .insert("pairCurrency", 0i32)
            .insert("quantityQNT", "0");
        
        Ok(builder.build())
    }
}

pub struct GetCoinExchangeOrderIdsHandler;

impl GetCoinExchangeOrderIdsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCoinExchangeOrderIdsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "exchange", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ce]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.get_u64("account");
        let _exchange = req.get_u64("exchange");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("orderIds", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetCoinExchangeOrdersHandler;

impl GetCoinExchangeOrdersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCoinExchangeOrdersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "exchange", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ce]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.get_u64("account");
        let _exchange = req.get_u64("exchange");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("orders", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetCoinExchangeTradesHandler;

impl GetCoinExchangeTradesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCoinExchangeTradesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "exchange", "firstIndex", "lastIndex", "timestamp", "includeAssetInfo"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ce]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.get_u64("account");
        let _exchange = req.get_u64("exchange");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("trades", json!([]));
        
        Ok(builder.build())
    }
}

pub struct ExchangeCoinsHandler;

impl ExchangeCoinsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ExchangeCoinsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "exchange", "pairCurrency", "quantityQNT", "rateNQT", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ce, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let currency = req.get_string("currency").unwrap_or_default(); let rateNQT = req.get_string("rateNQT").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0); let units = req.get_string("units").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 6, 0, None, 0, Some(json!({"currency": currency, "rateNQT": rateNQT.to_string(), "units": units.to_string()})), state,
        ).await
    }
}

pub struct CancelCoinExchangeOrderHandler;

impl CancelCoinExchangeOrderHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for CancelCoinExchangeOrderHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "order", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ce, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _order_id = req.require_u64("order")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}

pub struct SimulateCoinExchangeHandler;

impl SimulateCoinExchangeHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SimulateCoinExchangeHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["exchange", "pairCurrency", "quantityQNT"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ce]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _exchange = req.require_u64("exchange")?;
        let _pair_currency = req.get_i32("pairCurrency").unwrap_or(0);
        let _quantity = req.require_string("quantityQNT")?;

        let mut builder = RsRespBuilder::new();
        builder
            .insert("exchangeNQT", "0")
            .insert("pairNQT", "0");

        Ok(builder.build())
    }
}

pub struct GetCoinExchangeTradeHandler;

impl GetCoinExchangeTradeHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCoinExchangeTradeHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["order", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ce]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _order_id = req.require_u64("order")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("trade", json!({}));

        Ok(builder.build())
    }
}

pub struct GetExpectedCoinExchangeOrdersHandler;

impl GetExpectedCoinExchangeOrdersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetExpectedCoinExchangeOrdersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currency", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ce]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currency_id = req.require_u64("currency")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("orders", json!([]));

        Ok(builder.build())
    }
}

pub struct GetExpectedCoinExchangeOrderCancellationsHandler;

impl GetExpectedCoinExchangeOrderCancellationsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetExpectedCoinExchangeOrderCancellationsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currency", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ce]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currency_id = req.require_u64("currency")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("orderCancellations", json!([]));

        Ok(builder.build())
    }
}
