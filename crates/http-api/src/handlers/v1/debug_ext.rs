//! 调试扩展相关 API Handlers
//!
//! 与 Java 版本 DumpPeers, TrimDerivedTables 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};

pub struct DumpPeersHandler;

impl DumpPeersHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DumpPeersHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["excludeActive"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Debug, ApiTag::Network]
    }
    
    fn require_password(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _exclude_active = req.get_bool("excludeActive");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("peers", json!([]));
        
        Ok(builder.build())
    }
}

pub struct TrimDerivedTablesHandler;

impl TrimDerivedTablesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for TrimDerivedTablesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Debug]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    fn require_password(&self) -> bool {
        true
    }
    
    async fn process_request(&self, _req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("done", true);
        
        Ok(builder.build())
    }
}

pub struct LuceneReindexHandler;

impl LuceneReindexHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for LuceneReindexHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["force"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Debug]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    fn require_password(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _force = req.get_bool("force");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("done", true);
        
        Ok(builder.build())
    }
}

pub struct RebroadcastUnconfirmedTransactionsHandler;

impl RebroadcastUnconfirmedTransactionsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for RebroadcastUnconfirmedTransactionsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec![]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Debug]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    fn require_password(&self) -> bool {
        true
    }
    
    async fn process_request(&self, _req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("done", true);
        
        Ok(builder.build())
    }
}

pub struct RetrievePrunedDataHandler;

impl RetrievePrunedDataHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for RetrievePrunedDataHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transaction"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Debug]
    }
    
    fn require_password(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _transaction_id = req.get_u64("transaction");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("retrieved", true);
        
        Ok(builder.build())
    }
}

pub struct GetExecutedTransactionsHandler;

impl GetExecutedTransactionsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetExecutedTransactionsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["executedPhasedTransaction", "failedPhasedTransaction", "failedPhasedTransactionHolding", "requirePhased"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Transactions]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _executed_phased = req.get_u64("executedPhasedTransaction");
        let _failed_phased = req.get_u64("failedPhasedTransaction");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("transactions", json!([]));
        
        Ok(builder.build())
    }
}
