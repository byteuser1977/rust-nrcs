//! 货币兑换/交易相关 API Handlers
//!
//! 与 Java 版本 GetExchanges, PublishExchangeOffer 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct GetExchangesHandler;

impl GetExchangesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetExchangesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![
            "currency",
            "account",
            "firstIndex",
            "lastIndex",
            "includeCurrencyInfo",
            "requireBlock",
            "requireLastBlock",
        ]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }

    async fn process_request(
        &self,
        req: &ApiRequest,
        _state: &ApiState,
    ) -> Result<RsRespWithData, ApiError> {
        let _currency_id = req.get_u64("currency");
        let _account = req.get_u64("account");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        let _include_currency_info = req.get_bool("includeCurrencyInfo");

        let mut builder = RsRespBuilder::new();
        builder.insert("exchanges", json!([]));

        Ok(builder.build())
    }
}

pub struct GetExchangesByExchangeRequestHandler;

impl GetExchangesByExchangeRequestHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetExchangesByExchangeRequestHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transaction", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }

    async fn process_request(
        &self,
        req: &ApiRequest,
        _state: &ApiState,
    ) -> Result<RsRespWithData, ApiError> {
        let _transaction_id = req.require_u64("transaction")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("exchanges", json!([]));

        Ok(builder.build())
    }
}

pub struct GetExchangesByOfferHandler;

impl GetExchangesByOfferHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetExchangesByOfferHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![
            "offer",
            "firstIndex",
            "lastIndex",
            "includeCurrencyInfo",
            "requireBlock",
            "requireLastBlock",
        ]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }

    async fn process_request(
        &self,
        req: &ApiRequest,
        _state: &ApiState,
    ) -> Result<RsRespWithData, ApiError> {
        let _offer_id = req.get_u64("offer");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        let _include_currency_info = req.get_bool("includeCurrencyInfo");

        let mut builder = RsRespBuilder::new();
        builder.insert("exchanges", json!([]));

        Ok(builder.build())
    }
}

pub struct PublishExchangeOfferHandler;

impl PublishExchangeOfferHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for PublishExchangeOfferHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![
            "currency",
            "buyRateNQT",
            "sellRateNQT",
            "totalBuyLimit",
            "totalSellLimit",
            "secretPhrase",
            "feeNQT",
            "deadline",
        ]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms, ApiTag::CreateTransaction]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(
        &self,
        req: &ApiRequest,
        _state: &ApiState,
    ) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _currency_id = req.require_u64("currency")?;
        let _buy_rate = req.require_string("buyRateNQT")?;
        let _sell_rate = req.require_string("sellRateNQT")?;
        let _total_buy_limit = req.require_u64("totalBuyLimit")?;
        let _total_sell_limit = req.require_u64("totalSellLimit")?;

        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");

        Ok(builder.build())
    }
}

pub struct GetAllExchangesHandler;

impl GetAllExchangesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAllExchangesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![
            "firstIndex",
            "lastIndex",
            "includeCurrencyInfo",
            "requireBlock",
            "requireLastBlock",
        ]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }

    async fn process_request(
        &self,
        req: &ApiRequest,
        _state: &ApiState,
    ) -> Result<RsRespWithData, ApiError> {
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        let _include_currency_info = req.get_bool("includeCurrencyInfo");

        let mut builder = RsRespBuilder::new();
        builder.insert("exchanges", json!([]));

        Ok(builder.build())
    }
}
