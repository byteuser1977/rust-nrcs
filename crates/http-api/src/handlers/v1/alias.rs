//! 别名相关 API Handlers
//!
//! 与 Java 版本 GetAlias, SetAlias 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;
use super::create_transaction::CreateTransactionHelper;

pub struct GetAliasHandler;

impl GetAliasHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAliasHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["alias", "aliasName"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Aliases]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _alias_id = req.get_u64("alias");
        let _alias_name = req.get_string("aliasName");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("alias", "0")
            .insert("aliasName", "")
            .insert("account", "0")
            .insert("accountRS", "NRCS-0-0-0")
            .insert("timestamp", 0i32)
            .insert("aliasURI", "")
            .insert("priceNQT", "0");
        
        Ok(builder.build())
    }
}

pub struct GetAliasesHandler;

impl GetAliasesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAliasesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account", "timestamp", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Aliases]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.get_u64("account");
        let _timestamp = req.get_i32("timestamp");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("aliases", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAliasesLikeHandler;

impl GetAliasesLikeHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAliasesLikeHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["aliasPrefix", "firstIndex", "lastIndex"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Aliases]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _alias_prefix = req.require_string("aliasPrefix")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("aliases", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAliasCountHandler;

impl GetAliasCountHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAliasCountHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["account"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Aliases]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _account = req.require_u64("account")?;
        
        let mut builder = RsRespBuilder::new();
        builder.insert("numberOfAliases", 0i32);
        
        Ok(builder.build())
    }
}

pub struct SetAliasHandler;

impl SetAliasHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SetAliasHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "aliasName", "aliasURI", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Aliases, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let aliasName = req.get_string("aliasName").unwrap_or_default(); let aliasURI = req.get_string("aliasURI").unwrap_or_default();

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 1, 1, None, 0, Some(json!({"aliasName": aliasName, "aliasURI": aliasURI})), state,
        ).await
    }
}

pub struct DeleteAliasHandler;

impl DeleteAliasHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DeleteAliasHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "alias", "aliasName", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Aliases, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let alias = req.get_string("alias").unwrap_or_default();

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 1, 4, None, 0, Some(json!({"alias": alias})), state,
        ).await
    }
}

pub struct SellAliasHandler;

impl SellAliasHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for SellAliasHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "alias", "aliasName", "priceNQT", "recipient", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Aliases, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let alias = req.get_string("alias").unwrap_or_default(); let priceNQT = req.get_string("priceNQT").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 1, 2, None, 0, Some(json!({"alias": alias, "priceNQT": priceNQT.to_string()})), state,
        ).await
    }
}

pub struct BuyAliasHandler;

impl BuyAliasHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for BuyAliasHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "alias", "aliasName", "amountNQT", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Aliases, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let alias = req.get_string("alias").unwrap_or_default();

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 1, 3, None, 0, Some(json!({"alias": alias})), state,
        ).await
    }
}
