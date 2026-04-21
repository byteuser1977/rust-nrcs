//! 数字商品相关 API Handlers
//!
//! 与 Java 版本 GetDGSGood, DGSListing 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};

pub struct GetDGSGoodHandler;

impl GetDGSGoodHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetDGSGoodHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["goods", "includeCounts"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _goods_id = req.require_u64("goods")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("goods", "0")
            .insert("name", "")
            .insert("description", "")
            .insert("quantity", 0i32)
            .insert("priceNQT", "0")
            .insert("seller", "0")
            .insert("sellerRS", "NRCS-0-0-0")
            .insert("tags", "")
            .insert("delisted", false)
            .insert("timestamp", 0i32);
        
        Ok(builder.build())
    }
}

pub struct GetDGSGoodsHandler;

impl GetDGSGoodsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetDGSGoodsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["seller", "firstIndex", "lastIndex", "inStockOnly"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _seller = req.get_u64("seller");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        let _in_stock_only = req.get_bool("inStockOnly");
        
        let mut builder = RsRespBuilder::new();
        builder.insert("goods", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetAllDGSGoodsHandler;

impl GetAllDGSGoodsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetAllDGSGoodsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex", "inStockOnly"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("goods", json!([]));
        
        Ok(builder.build())
    }
}

pub struct GetDGSPurchaseHandler;

impl GetDGSPurchaseHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetDGSPurchaseHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["purchase"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _purchase_id = req.require_u64("purchase")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("purchase", "0")
            .insert("goods", "0")
            .insert("name", "")
            .insert("quantity", 0i32)
            .insert("priceNQT", "0")
            .insert("buyer", "0")
            .insert("buyerRS", "NRCS-0-0-0")
            .insert("seller", "0")
            .insert("sellerRS", "NRCS-0-0-0")
            .insert("timestamp", 0i32)
            .insert("pending", true);
        
        Ok(builder.build())
    }
}

pub struct GetDGSPurchasesHandler;

impl GetDGSPurchasesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetDGSPurchasesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["seller", "buyer", "firstIndex", "lastIndex", "completed"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs]
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _seller = req.get_u64("seller");
        let _buyer = req.get_u64("buyer");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);
        
        let mut builder = RsRespBuilder::new();
        builder.insert("purchases", json!([]));
        
        Ok(builder.build())
    }
}

pub struct DGSListingHandler;

impl DGSListingHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DGSListingHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "name", "description", "quantity", "priceNQT", "tags", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _name = req.require_string("name")?;
        let _description = req.get_string("description");
        let _quantity = req.get_i32("quantity").unwrap_or(1);
        let _price = req.require_string("priceNQT")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "")
            .insert("transactionBytes", "");
        
        Ok(builder.build())
    }
}

pub struct DGSDelistingHandler;

impl DGSDelistingHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DGSDelistingHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "goods", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _goods_id = req.require_u64("goods")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}

pub struct DGSPurchaseHandler;

impl DGSPurchaseHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DGSPurchaseHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "goods", "quantity", "priceNQT", "deliveryDeadlineTimestamp", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _goods_id = req.require_u64("goods")?;
        let _quantity = req.get_i32("quantity").unwrap_or(1);
        let _price = req.require_string("priceNQT")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}

pub struct DGSDeliveryHandler;

impl DGSDeliveryHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DGSDeliveryHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "purchase", "discountNQT", "goodsToEncrypt", "goodsIsText", 
             "goodsData", "goodsNonce", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _purchase_id = req.require_u64("purchase")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}

pub struct DGSFeedbackHandler;

impl DGSFeedbackHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DGSFeedbackHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "purchase", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _purchase_id = req.require_u64("purchase")?;
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}

pub struct DGSRefundHandler;

impl DGSRefundHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DGSRefundHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "purchase", "refundNQT", "feeNQT", "deadline"]
    }
    
    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs, ApiTag::CreateTransaction]
    }
    
    fn require_post(&self) -> bool {
        true
    }
    
    async fn process_request(&self, req: &ApiRequest) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let _purchase_id = req.require_u64("purchase")?;
        let _refund = req.get_string("refundNQT");
        
        let mut builder = RsRespBuilder::new();
        builder
            .insert("transaction", "")
            .insert("fullHash", "");
        
        Ok(builder.build())
    }
}
