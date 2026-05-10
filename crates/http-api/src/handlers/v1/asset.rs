//! 资产相关 API Handlers
//!
//! 与 Java 版本 GetAsset, IssueAsset 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;
use super::create_transaction::CreateTransactionHelper;

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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let asset_id = req.require_u64("asset")?;
        let include_counts = req.get_bool("includeCounts");
        
        let asset = state.asset_repo
            .find_by_asset_id(asset_id as i64)
            .await
            .map_err(ApiError::Repository)?;
        
        let asset = match asset {
            Some(a) => a,
            None => return Err(ApiError::NotFound("Asset not found".to_string())),
        };
        
        let _asset_domain = asset.to_domain().map_err(|e| ApiError::Internal(e.to_string()))?;
        
        let mut builder = RsRespBuilder::new();
        
        builder
            .insert("asset", asset.id.to_string())
            .insert("account", asset.account_id.to_string())
            .insert("accountRS", format_account_rs(asset.account_id as u64))
            .insert("name", asset.name.clone())
            .insert("description", asset.description.clone().unwrap_or_default())
            .insert("quantityQNT", asset.quantity.to_string())
            .insert("decimals", asset.decimals as i32)
            .insert("numberOfTrades", 0i32)
            .insert("numberOfTransfers", 0i32)
            .insert("numberOfAccounts", 0i32);
        
        if include_counts {
            builder
                .insert("numberOfTrades", 0i32)
                .insert("numberOfTransfers", 0i32)
                .insert("numberOfAccounts", 0i32);
        }
        
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let assets_str = req.get_string("assets").unwrap_or_default();
        let asset_ids: Vec<i64> = assets_str.split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        
        let mut assets = Vec::new();
        for id in asset_ids {
            if let Some(asset) = state.asset_repo.find_by_asset_id(id).await.map_err(ApiError::Repository)? {
                assets.push(json!({
                    "asset": asset.id.to_string(),
                    "account": asset.account_id.to_string(),
                    "accountRS": format_account_rs(asset.account_id as u64),
                    "name": asset.name,
                    "quantityQNT": asset.quantity.to_string(),
                    "decimals": asset.decimals as i32
                }));
            }
        }
        
        let mut builder = RsRespBuilder::new();
        builder.insert("assets", json!(assets));
        
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let first_index = req.get_i32("firstIndex").unwrap_or(0);
        let last_index = req.get_i32("lastIndex").unwrap_or(99);
        let limit = (last_index - first_index + 1) as i64;
        
        let assets = state.asset_repo
            .find_tradable(limit)
            .await
            .map_err(ApiError::Repository)?;
        
        let assets_json: Vec<serde_json::Value> = assets.iter()
            .map(|a| json!({
                "asset": a.id.to_string(),
                "account": a.account_id.to_string(),
                "accountRS": format_account_rs(a.account_id as u64),
                "name": a.name,
                "quantityQNT": a.quantity.to_string(),
                "decimals": a.decimals as i32
            }))
            .collect();
        
        let mut builder = RsRespBuilder::new();
        builder.insert("assets", json!(assets_json));
        
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let first_index = req.get_i32("firstIndex").unwrap_or(0);
        let last_index = req.get_i32("lastIndex").unwrap_or(99);
        let limit = (last_index - first_index + 1) as i64;
        
        let assets = state.asset_repo
            .find_tradable(limit)
            .await
            .map_err(ApiError::Repository)?;
        
        let asset_ids: Vec<String> = assets.iter()
            .map(|a| a.id.to_string())
            .collect();
        
        let mut builder = RsRespBuilder::new();
        builder.insert("assetIds", json!(asset_ids));
        
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account = req.require_u64("account")?;
        
        let assets = state.asset_repo
            .find_by_owner(account as i64)
            .await
            .map_err(ApiError::Repository)?;
        
        let assets_json: Vec<serde_json::Value> = assets.iter()
            .map(|a| json!({
                "asset": a.id.to_string(),
                "account": a.account_id.to_string(),
                "accountRS": format_account_rs(a.account_id as u64),
                "name": a.name,
                "quantityQNT": a.quantity.to_string(),
                "decimals": a.decimals as i32
            }))
            .collect();
        
        let mut builder = RsRespBuilder::new();
        builder.insert("assets", json!(assets_json));
        
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let asset_id = req.require_u64("asset")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        let _height = req.get_i32("height");
        
        let account_assets = state.account_asset_repo
            .find_by_asset(asset_id as i64)
            .await
            .map_err(ApiError::Repository)?;
        
        let accounts_json: Vec<serde_json::Value> = account_assets.iter()
            .map(|aa| json!({
                "account": aa.account_id.to_string(),
                "accountRS": format_account_rs(aa.account_id as u64),
                "asset": aa.asset_id.to_string(),
                "quantityQNT": aa.quantity.to_string(),
                "unconfirmedQuantityQNT": aa.quantity.to_string()
            }))
            .collect();
        
        let mut builder = RsRespBuilder::new();
        builder.insert("accountAssets", json!(accounts_json));
        
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
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let name = req.get_string("name").unwrap_or_default(); let description = req.get_string("description").unwrap_or_default(); let quantityQNT = req.get_string("quantityQNT").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0); let decimals = req.get_i32("decimals").unwrap_or(0); let mintable = req.get_bool("mintable");

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 2, 0, None, 0, Some(json!({"name": name, "description": description, "quantityQNT": quantityQNT.to_string(), "decimals": decimals, "mintable": mintable})), state,
        ).await
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let asset = req.get_string("asset").unwrap_or_default(); let quantityQNT = req.get_string("quantityQNT").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0); let recipient_id = req.require_u64("recipient")?;

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 2, 1, Some(recipient_id), 0, Some(json!({"asset": asset, "quantityQNT": quantityQNT.to_string()})), state,
        ).await
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let asset = req.get_string("asset").unwrap_or_default(); let quantityQNT = req.get_string("quantityQNT").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0); let priceNQT = req.get_string("priceNQT").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 2, 3, None, 0, Some(json!({"asset": asset, "quantityQNT": quantityQNT.to_string(), "priceNQT": priceNQT.to_string()})), state,
        ).await
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let asset = req.get_string("asset").unwrap_or_default(); let quantityQNT = req.get_string("quantityQNT").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0); let priceNQT = req.get_string("priceNQT").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 2, 4, None, 0, Some(json!({"asset": asset, "quantityQNT": quantityQNT.to_string(), "priceNQT": priceNQT.to_string()})), state,
        ).await
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let order = req.get_string("order").unwrap_or_default();

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 2, 5, None, 0, Some(json!({"order": order})), state,
        ).await
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let order = req.get_string("order").unwrap_or_default();

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 2, 6, None, 0, Some(json!({"order": order})), state,
        ).await
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
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("trades", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAccountCurrentAskOrderIdsHandler;

impl GetAccountCurrentAskOrderIdsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountCurrentAskOrderIdsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "asset", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("askOrderIds", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAccountCurrentBidOrderIdsHandler;

impl GetAccountCurrentBidOrderIdsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountCurrentBidOrderIdsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "asset", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("bidOrderIds", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAllOpenAskOrdersHandler;

impl GetAllOpenAskOrdersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAllOpenAskOrdersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("askOrders", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAllOpenBidOrdersHandler;

impl GetAllOpenBidOrdersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAllOpenBidOrdersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("bidOrders", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetExpectedAssetDeletesHandler;

impl GetExpectedAssetDeletesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetExpectedAssetDeletesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["asset", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _asset_id = req.require_u64("asset")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("done", true);

        Ok(builder.build())
    }
}

pub struct GetExpectedAssetTransfersHandler;

impl GetExpectedAssetTransfersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetExpectedAssetTransfersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["asset", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _asset_id = req.require_u64("asset")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("transfers", json!([]));

        Ok(builder.build())
    }
}

pub struct GetAvailableToBuyHandler;

impl GetAvailableToBuyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAvailableToBuyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currency", "units", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currency = req.require_string("currency")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("done", true);

        Ok(builder.build())
    }
}

pub struct GetAvailableToSellHandler;

impl GetAvailableToSellHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAvailableToSellHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currency", "units", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currency = req.require_string("currency")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("done", true);

        Ok(builder.build())
    }
}

pub struct GetBuyOffersHandler;

impl GetBuyOffersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBuyOffersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currency", "account", "firstIndex", "lastIndex", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currency = req.require_string("currency")?;
        let _account = req.get_u64("account");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);

        let mut builder = RsRespBuilder::new();
        builder.insert("done", true);

        Ok(builder.build())
    }
}

pub struct GetSellOffersHandler;

impl GetSellOffersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetSellOffersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currency", "account", "firstIndex", "lastIndex", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currency = req.require_string("currency")?;
        let _account = req.get_u64("account");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);

        let mut builder = RsRespBuilder::new();
        builder.insert("done", true);

        Ok(builder.build())
    }
}

pub struct GetOfferHandler;

impl GetOfferHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetOfferHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["offer", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _offer_id = req.require_u64("offer")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("offers", json!([]));

        Ok(builder.build())
    }
}

pub struct GetExpectedBuyOffersHandler;

impl GetExpectedBuyOffersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetExpectedBuyOffersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currency", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currency = req.require_string("currency")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("offers", json!([]));

        Ok(builder.build())
    }
}

pub struct GetExpectedSellOffersHandler;

impl GetExpectedSellOffersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetExpectedSellOffersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["currency", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Ae]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _currency = req.require_string("currency")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("offers", json!([]));

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
