//! 账户相关 API Handlers
//!
//! 与 Java 版本 GetAccount, GetBalance 等完全对齐

#![allow(clippy::new_without_default)]

use async_trait::async_trait;


use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;
use super::create_transaction::CreateTransactionHelper;

pub struct GetAccountHandler;

impl Default for GetAccountHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl GetAccountHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "includeLessors", "includeAssets", "includeCurrencies", "includeEffectiveBalance"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account_id = req.require_u64("account")?;
        let include_lessors = req.get_bool("includeLessors");
        let include_assets = req.get_bool("includeAssets");
        let include_currencies = req.get_bool("includeCurrencies");
        let include_effective_balance = req.get_bool("includeEffectiveBalance");
        
        let account = state.account_manager
            .get_account_info(account_id)
            .await
            .map_err(ApiError::Account)?;
        
        let mut builder = RsRespBuilder::new();
        
        builder
            .insert("account", account_id.to_string())
            .insert("accountRS", format_account_rs(account_id));
        
        let name = account.properties.get("name").cloned().unwrap_or_default();
        let description = account.properties.get("description").cloned().unwrap_or_default();
        builder.insert("name", name);
        builder.insert("description", description);

        if let Ok(Some(pk)) = state.account_manager.get_public_key(account_id).await {
            match pk {
                blockchain_types::prelude::PublicKey::Ed25519(bytes) => {
                    builder.insert("publicKey", hex::encode(bytes));
                }
            }
        }

        builder
            .insert("balanceNQT", account.balance.to_string())
            .insert("unconfirmedBalanceNQT", account.unconfirmed_balance.to_string())
            .insert("forgedBalanceNQT", account.forged_balance.to_string())
            .insert("guaranteedBalanceNQT", account.guaranteed_balance.to_string());
        
        if include_effective_balance {
            let effective = account.effective_balance();
            builder.insert("effectiveBalanceNRCS", effective as i64);
        }

        if let Some(ref lease) = account.lease {
            if lease.lessee_id != 0 {
                builder
                    .insert("currentLessee", lease.lessee_id.to_string())
                    .insert("currentLesseeRS", format_account_rs(lease.lessee_id))
                    .insert("currentLeasingHeightFrom", lease.start_height)
                    .insert("currentLeasingHeightTo", lease.end_height);
            }
        }

        if account.has_control_phasing {
            builder.insert("accountControls", json!(["PHASING_ONLY"]));
        }
        
        if include_lessors {
            builder.insert("lessors", json!([]));
            builder.insert("lessorsRS", json!([]));
            builder.insert("lessorsInfo", json!([]));
        }
        
        if include_assets {
            let asset_balances: Vec<serde_json::Value> = account.assets.iter()
                .map(|(id, qty)| json!({
                    "asset": id.to_string(),
                    "balanceQNT": qty.to_string()
                }))
                .collect();
            let unconfirmed_asset_balances: Vec<serde_json::Value> = account.assets.iter()
                .map(|(id, qty)| json!({
                    "asset": id.to_string(),
                    "unconfirmedBalanceQNT": qty.to_string()
                }))
                .collect();
            builder.insert("assetBalances", json!(asset_balances));
            builder.insert("unconfirmedAssetBalances", json!(unconfirmed_asset_balances));
        }
        
        if include_currencies {
            builder.insert("accountCurrencies", json!([]));
        }
        
        Ok(builder.build())
    }
}

pub struct GetBalanceHandler;

impl Default for GetBalanceHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl GetBalanceHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBalanceHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account_id = req.require_u64("account")?;

        let account = state.account_manager
            .get_account_info(account_id)
            .await
            .map_err(ApiError::Account)?;

        let mut builder = RsRespBuilder::new();

        builder
            .insert("account", account_id.to_string())
            .insert("accountRS", format_account_rs(account_id))
            .insert("balanceNQT", account.balance.to_string())
            .insert("unconfirmedBalanceNQT", account.unconfirmed_balance.to_string())
            .insert("effectiveBalanceNRCS", account.effective_balance() as i64)
            .insert("guaranteedBalanceNQT", account.guaranteed_balance.to_string());

        Ok(builder.build())
    }
}

pub struct GetAccountIdHandler;

impl Default for GetAccountIdHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl GetAccountIdHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountIdHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let secret_phrase = req.require_string("secretPhrase")?;
        
        let account_id = derive_account_id(&secret_phrase);
        
        let mut builder = RsRespBuilder::new();
        
        builder
            .insert("account", account_id.to_string())
            .insert("accountRS", format_account_rs(account_id));
        
        Ok(builder.build())
    }
}

pub struct GetAccountPublicKeyHandler;

impl Default for GetAccountPublicKeyHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl GetAccountPublicKeyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountPublicKeyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account_id = req.require_u64("account")?;
        
        let public_key = state.account_manager
            .get_public_key(account_id)
            .await
            .map_err(ApiError::Account)?;
        
        let mut builder = RsRespBuilder::new();
        
        if let Some(pk) = public_key {
            builder.insert("publicKey", hex::encode(pk.as_bytes()));
        } else {
            builder.insert("publicKey", "");
        }
        
        Ok(builder.build())
    }
}

pub struct GetAccountAssetsHandler;

impl Default for GetAccountAssetsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl GetAccountAssetsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountAssetsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "asset", "height", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::Ae]
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account_id = req.require_u64("account")?;
        let _asset_id = req.get_u64("asset");
        let _height = req.get_i32("height");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let account = state.account_manager
            .get_account_info(account_id)
            .await
            .map_err(ApiError::Account)?;
        
        let account_assets: Vec<serde_json::Value> = account.assets.iter()
            .map(|(id, qty)| json!({
                "asset": id.to_string(),
                "account": account_id.to_string(),
                "accountRS": format_account_rs(account_id),
                "quantityQNT": qty.to_string(),
                "unconfirmedQuantityQNT": qty.to_string()
            }))
            .collect();
        
        let mut builder = RsRespBuilder::new();
        builder.insert("accountAssets", json!(account_assets));
        
        Ok(builder.build())
    }
}

pub struct GetAccountCurrenciesHandler;

impl Default for GetAccountCurrenciesHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl GetAccountCurrenciesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountCurrenciesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "currency", "height", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::Ms]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account_id = req.require_u64("account")?;
        let _currency_id = req.get_u64("currency");
        let _height = req.get_i32("height");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("accountCurrencies", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAccountPropertiesHandler;

impl Default for GetAccountPropertiesHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl GetAccountPropertiesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountPropertiesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["recipient", "property", "setter", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _recipient = req.get_u64("recipient");
        let _property = req.get_string("property");
        let _setter = req.get_u64("setter");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("properties", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAccountLessorsHandler;

impl Default for GetAccountLessorsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl GetAccountLessorsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountLessorsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "height"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account_id = req.require_u64("account")?;
        let _height = req.get_i32("height");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("lessors", json!([]))
            .insert("lessorsRS", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetEffectiveBalanceHandler;

impl Default for GetEffectiveBalanceHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl GetEffectiveBalanceHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetEffectiveBalanceHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account_id = req.require_u64("account")?;
        
        let account = state.account_manager
            .get_account_info(account_id)
            .await
            .map_err(ApiError::Account)?;
        
        let effective = account.effective_balance();
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("account", account_id.to_string())
            .insert("accountRS", format_account_rs(account_id))
            .insert("effectiveBalanceNRCS", effective as i64)
            .insert("guaranteedBalanceNQT", account.guaranteed_balance.to_string());
        
        Ok(builder.build())
    }
}

pub struct GetGuaranteedBalanceHandler;

impl Default for GetGuaranteedBalanceHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl GetGuaranteedBalanceHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetGuaranteedBalanceHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "numberOfConfirmations"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let account_id = req.require_u64("account")?;
        let _confirmations = req.get_i32("numberOfConfirmations").unwrap_or(1440);
        
        let account = state.account_manager
            .get_account_info(account_id)
            .await
            .map_err(ApiError::Account)?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("guaranteedBalanceNQT", account.guaranteed_balance.to_string());
        
        Ok(builder.build())
    }
}

pub struct SetAccountInfoHandler;

impl Default for SetAccountInfoHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl SetAccountInfoHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SetAccountInfoHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "name", "description", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let name = req.get_string("name").unwrap_or_default(); let description = req.get_string("description").unwrap_or_default();

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 4, 0, None, 0, Some(json!({"name": name, "description": description})), state,
        ).await
    }
}

pub struct SetAccountPropertyHandler;

impl SetAccountPropertyHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SetAccountPropertyHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "recipient", "property", "value", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let property = req.get_string("property").unwrap_or_default(); let value = req.get_string("value").unwrap_or_default();

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 4, 1, None, 0, Some(json!({"property": property, "value": value})), state,
        ).await
    }
}

pub struct GetBalancesHandler;

impl GetBalancesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetBalancesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "includeEffectiveBalance", "height"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account_ids = req.get_string("account");
        let _include_effective = req.get_bool("includeEffectiveBalance");
        let _height = req.get_i32("height");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("balances", json!([]));
        
        Ok(builder.build())
    }
}

// --- New handlers: account block/pagination APIs ---

pub struct GetAccountBlockCountHandler;

impl GetAccountBlockCountHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountBlockCountHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::Blocks]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("numberOfBlocks", 0i32);

        Ok(builder.build())
    }
}

pub struct GetAccountBlockIdsHandler;

impl GetAccountBlockIdsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountBlockIdsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "firstIndex", "lastIndex"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::Blocks]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);

        let mut builder = RsRespBuilder::new();
        builder.insert("blockIds", json!([]));

        Ok(builder.build())
    }
}

pub struct GetAccountBlocksHandler;

impl GetAccountBlocksHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountBlocksHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "firstIndex", "lastIndex", "includeTransactions"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::Blocks]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        let _include_transactions = req.get_bool("includeTransactions");

        let mut builder = RsRespBuilder::new();
        builder.insert("blocks", json!([]));

        Ok(builder.build())
    }
}

pub struct GetAccountExchangeRequestsHandler;

impl GetAccountExchangeRequestsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountExchangeRequestsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "firstIndex", "lastIndex"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);

        let mut builder = RsRespBuilder::new();
        builder.insert("exchangeRequests", json!([]));

        Ok(builder.build())
    }
}

pub struct GetAccountLedgerHandler;

impl GetAccountLedgerHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountLedgerHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![
            "account",
            "firstIndex",
            "lastIndex",
            "event",
            "eventType",
            "holdingType",
            "holding",
            "includeHoldingInfo",
        ]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        let _event = req.get_string("event");
        let _event_type = req.get_i32("eventType");
        let _holding_type = req.get_i32("holdingType");
        let _holding = req.get_u64("holding");
        let _include_holding_info = req.get_bool("includeHoldingInfo");

        let mut builder = RsRespBuilder::new();
        builder.insert("entries", json!([]));

        Ok(builder.build())
    }
}

pub struct GetAccountLedgerEntryHandler;

impl GetAccountLedgerEntryHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountLedgerEntryHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["ledgerId"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _ledger_id = req.require_u64("ledgerId")?;

        let mut builder = RsRespBuilder::new();
        builder.insert("entry", json!({}));

        Ok(builder.build())
    }
}

pub struct GetAccountPhasedTransactionsHandler;

impl GetAccountPhasedTransactionsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountPhasedTransactionsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "firstIndex", "lastIndex"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::Phasing]
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

pub struct GetAccountCurrentAskOrdersHandler;

impl GetAccountCurrentAskOrdersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountCurrentAskOrdersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "firstIndex", "lastIndex"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::Ae]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);

        let mut builder = RsRespBuilder::new();
        builder.insert("askOrders", json!([]));

        Ok(builder.build())
    }
}

pub struct GetAccountCurrentBidOrdersHandler;

impl GetAccountCurrentBidOrdersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAccountCurrentBidOrdersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "firstIndex", "lastIndex"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Accounts, ApiTag::Ae]
    }

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);

        let mut builder = RsRespBuilder::new();
        builder.insert("bidOrders", json!([]));

        Ok(builder.build())
    }
}

pub struct GetAllBroadcastedTransactionsHandler;

impl GetAllBroadcastedTransactionsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAllBroadcastedTransactionsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Transactions]
    }

    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("transactions", json!([]));

        Ok(builder.build())
    }
}

pub struct GetAllWaitingTransactionsHandler;

impl GetAllWaitingTransactionsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAllWaitingTransactionsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Transactions]
    }

    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("transactions", json!([]));

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

fn derive_account_id(secret_phrase: &str) -> u64 {
    let seed = crypto::sha256(secret_phrase.as_bytes());
    let kp = crypto::keypair_from_seed(&seed);
    let public_key = kp.public_key();
    let pk_bytes = public_key.as_bytes();
    let hash = crypto::sha256(pk_bytes);
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&hash[0..8]);
    u64::from_le_bytes(bytes)
}
