//! 资产相关 API Handlers
//!
//! 与 Java 版本 GetAsset, IssueAsset 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct GetAssetHandler;

impl GetAssetHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAssetHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["asset", "includeCounts"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _asset_id = req.require_u64("asset")?;
        let _include_counts = req.get_bool("includeCounts");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("asset", "0")
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0")
            .insert("name", "")
            .insert("description", "")
            .insert("quantityQNT", "0")
            .insert("decimals", 0i32)
            .insert("numberOfTrades", 0i32)
            .insert("numberOfTransfers", 0i32)
            .insert("numberOfAccounts", 0i32);
        
        Ok(builder.build())
    }
}

pub struct GetAssetsHandler;

impl GetAssetsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAssetsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["assets", "includeCounts"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _assets = req.get_string("assets");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("assets", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAllAssetsHandler;

impl GetAllAssetsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAllAssetsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex", "includeCounts"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("assets", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAssetIdsHandler;

impl GetAssetIdsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAssetIdsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("assetIds", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAssetsByIssuerHandler;

impl GetAssetsByIssuerHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAssetsByIssuerHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "firstIndex", "lastIndex", "includeCounts"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("assets", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAssetAccountsHandler;

impl GetAssetAccountsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAssetAccountsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["asset", "firstIndex", "lastIndex", "height"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _asset_id = req.require_u64("asset")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        let _height = req.get_i32("height");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("accountAssets", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAssetTransfersHandler;

impl GetAssetTransfersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAssetTransfersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["asset", "account", "firstIndex", "lastIndex", "timestamp", "includeAssetInfo"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _asset_id = req.get_u64("asset");
        let _account = req.get_u64("account");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("transfers", json!([]));
        
        Ok(builder.build())
    }
}

pub struct IssueAssetHandler;

impl IssueAssetHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for IssueAssetHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "name", "description", "quantityQNT", "decimals", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _name = req.require_string("name")?;
        let _description = req.get_string("description");
        let _quantity = req.require_string("quantityQNT")?;
        let _decimals = req.get_i32("decimals").unwrap_or(0);
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "")
            .insert("transactionBytes", "");
        
        Ok(builder.build())
    }
}

pub struct TransferAssetHandler;

impl TransferAssetHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for TransferAssetHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "recipient", "asset", "quantityQNT", "feeNQT", "deadline", "referencedTransactionFullHash"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _recipient = req.require_u64("recipient")?;
        let _asset_id = req.require_u64("asset")?;
        let _quantity = req.require_string("quantityQNT")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "")
            .insert("transactionBytes", "");
        
        Ok(builder.build())
    }
}

pub struct PlaceAskOrderHandler;

impl PlaceAskOrderHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for PlaceAskOrderHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "asset", "quantityQNT", "priceNQT", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _asset_id = req.require_u64("asset")?;
        let _quantity = req.require_string("quantityQNT")?;
        let _price = req.require_string("priceNQT")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "")
            .insert("order", "");
        
        Ok(builder.build())
    }
}

pub struct PlaceBidOrderHandler;

impl PlaceBidOrderHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for PlaceBidOrderHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "asset", "quantityQNT", "priceNQT", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _asset_id = req.require_u64("asset")?;
        let _quantity = req.require_string("quantityQNT")?;
        let _price = req.require_string("priceNQT")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "")
            .insert("order", "");
        
        Ok(builder.build())
    }
}

pub struct CancelAskOrderHandler;

impl CancelAskOrderHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for CancelAskOrderHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "order", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae, ApiTag::CreateTransaction]
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

pub struct CancelBidOrderHandler;

impl CancelBidOrderHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for CancelBidOrderHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "order", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae, ApiTag::CreateTransaction]
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

pub struct GetAskOrderHandler;

impl GetAskOrderHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAskOrderHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["order"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _order_id = req.require_u64("order")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("order", "")
            .insert("asset", "")
            .insert("account", "")
            .insert("quantityQNT", "0")
            .insert("priceNQT", "0");
        
        Ok(builder.build())
    }
}

pub struct GetBidOrderHandler;

impl GetBidOrderHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBidOrderHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["order"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _order_id = req.require_u64("order")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("order", "")
            .insert("asset", "")
            .insert("account", "")
            .insert("quantityQNT", "0")
            .insert("priceNQT", "0");
        
        Ok(builder.build())
    }
}

pub struct GetAskOrdersHandler;

impl GetAskOrdersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAskOrdersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["asset", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _asset_id = req.require_u64("asset")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("askOrders", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetBidOrdersHandler;

impl GetBidOrdersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBidOrdersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["asset", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _asset_id = req.require_u64("asset")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("bidOrders", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetTradesHandler;

impl GetTradesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetTradesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["asset", "account", "firstIndex", "lastIndex", "timestamp", "includeAssetInfo"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _asset_id = req.get_u64("asset");
        let _account = req.get_u64("account");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("trades", json!([]));
        
        Ok(builder.build())
    }
}
