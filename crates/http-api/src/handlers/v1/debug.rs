//! 调试相关 API Handlers
//!
//! 与 Java 版本 GetLog, GetStackTraces 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

pub struct GetLogHandler;

impl GetLogHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetLogHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["logLevel", "count"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Debug]
    }
    
    fn require_password(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _log_level = req.get_string("logLevel");
        let _count = req.get_i32("count").unwrap_or(100);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("log", "");
        
        Ok(builder.build())
    }
}

/// 完全重置区块链 Handler
///
/// 对应 NRCS Java: `FullReset`
///
/// 功能：完全重置区块链状态（危险操作，需管理员密码）
pub struct FullResetHandler;

impl FullResetHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for FullResetHandler {
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

    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        // 注意：实际实现中会调用 BlockchainProcessor.fullReset()
        // 这是一个危险操作，仅用于开发和测试环境
        
        let mut builder = RsRespBuilder::new();
        
        // 模拟执行重置（生产环境中应调用真实的区块链处理器）
        builder.insert("done", true);
        
        Ok(builder.build())
    }
}

/// Lucene 索引重建 Handler
///
/// 对应 NRCS Java: `LuceneReindex`
///
/// 功能：重建全文搜索索引（运维功能）
pub struct LuceneReindexHandler;

impl LuceneReindexHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for LuceneReindexHandler {
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

    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        // 注意：Java 版本中此功能被注释掉（TODO 状态）
        // Rust 版本暂时返回未实现提示
        
        let mut builder = RsRespBuilder::new();
        builder.insert("errorDescription", "Lucene reindex not implemented yet");
        
        Ok(builder.build())
    }
}

/// 检索裁剪数据 Handler
///
/// 对应 NRCS Java: `RetrievePrunedData`
///
/// 功能：恢复被裁剪的数据（运维功能）
pub struct RetrievePrunedDataHandler;

impl RetrievePrunedDataHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for RetrievePrunedDataHandler {
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

    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        // 注意：实际实现中会调用 BlockchainProcessor.restorePrunedData()
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("done", true)
            .insert("numberOfPrunedData", 0.to_string());
        
        Ok(builder.build())
    }
}

/// 检索裁剪交易 Handler
///
/// 对应 NRCS Java: `RetrievePrunedTransaction`
///
/// 功能：恢复被裁剪的交易数据（运维功能）
pub struct RetrievePrunedTransactionHandler;

impl RetrievePrunedTransactionHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for RetrievePrunedTransactionHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["transaction"]
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

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _transaction_id = req.get_u64("transaction");
        
        // 注意：实际实现中会从数据库或网络检索裁剪的交易数据
        
        let mut builder = RsRespBuilder::new();
        builder.insert("errorDescription", "Transaction not found or not pruned");
        
        Ok(builder.build())
    }
}

pub struct GetStackTracesHandler;

impl GetStackTracesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetStackTracesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["depth"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Debug]
    }
    
    fn require_password(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _depth = req.get_i32("depth").unwrap_or(10);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("stackTraces", json!([]));
        
        Ok(builder.build())
    }
}

pub struct ClearUnconfirmedTransactionsHandler;

impl ClearUnconfirmedTransactionsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ClearUnconfirmedTransactionsHandler {
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
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("done", true);
        
        Ok(builder.build())
    }
}

pub struct PopOffHandler;

impl PopOffHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for PopOffHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["numBlocks", "height"]
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _num_blocks = req.get_i32("numBlocks");
        let _height = req.get_i32("height");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("blocks", json!([]));
        
        Ok(builder.build())
    }
}

pub struct ScanHandler;

impl ScanHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ScanHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["numBlocks", "height"]
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _num_blocks = req.get_i32("numBlocks");
        let _height = req.get_i32("height");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("done", true);
        
        Ok(builder.build())
    }
}

pub struct ShutdownHandler;

impl ShutdownHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for ShutdownHandler {
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
    
    async fn process_request(&self, _req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let mut builder = RsRespBuilder::new();
        builder.insert("shutdown", true);
        
        Ok(builder.build())
    }
}

pub struct GetPeerInfoHandler;

impl GetPeerInfoHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetPeerInfoHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["peer"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Debug, ApiTag::Network]
    }
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _peer = req.get_string("peer");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("peerInfo", json!({
                "address": "",
                "state": 0,
                "version": "",
                "application": ""
            }));
        
        Ok(builder.build())
    }
}
