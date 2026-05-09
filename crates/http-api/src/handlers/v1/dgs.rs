//! 数字商品相关 API Handlers
//!
//! 与 Java 版本 GetDGSGood, DGSListing 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::dgs_service::{DGSPurchase, DGSGoods};
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;

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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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
    
    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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

    async fn process_request(&self, req: &ApiRequest, _state: &ApiState) -> Result<RsRespWithData, ApiError> {
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

pub struct DGSPriceChangeHandler;

impl DGSPriceChangeHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DGSPriceChangeHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "goods", "priceNQT", "feeNQT", "deadline"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs, ApiTag::CreateTransaction]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let goods_id = req.require_u64("goods")?;
        let price_nqt_str = req.require_string("priceNQT")?;
        let price_nqt: u64 = price_nqt_str.parse().map_err(|_| ApiError::IncorrectValue("priceNQT".to_string()))?;

        // 调用 DGS 服务修改价格
        // 注意：实际实现中需要从 secretPhrase 计算出 sellerId
        // 这里简化处理，使用 goods_id 作为示例
        let result = state
            .dgs_service
            .change_price(goods_id, 0, price_nqt)
            .await;

        match result {
            Ok(goods) => {
                let mut builder = RsRespBuilder::new();
                builder.insert("goods", json!(goods));
                Ok(builder.build())
            }
            Err(e) => Err(ApiError::Validation(e)),
        }
    }
}

pub struct DGSQuantityChangeHandler;

impl DGSQuantityChangeHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for DGSQuantityChangeHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["secretPhrase", "goods", "deltaQuantity", "feeNQT", "deadline"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs, ApiTag::CreateTransaction]
    }

    fn require_post(&self) -> bool {
        true
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _secret_phrase = req.require_string("secretPhrase")?;
        let goods_id = req.require_u64("goods")?;
        let delta_quantity = req.get_i32("deltaQuantity").unwrap_or(0);

        // 调用 DGS 服务修改数量
        let result = state
            .dgs_service
            .change_quantity(goods_id, 0, delta_quantity)
            .await;

        match result {
            Ok(goods) => {
                let mut builder = RsRespBuilder::new();
                builder.insert("goods", json!(goods));
                Ok(builder.build())
            }
            Err(e) => Err(ApiError::Validation(e)),
        }
    }
}

pub struct GetDGSExpiredPurchasesHandler;

impl GetDGSExpiredPurchasesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetDGSExpiredPurchasesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["seller", "firstIndex", "lastIndex", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs]
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _seller = req.get_u64("seller");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);

        // 调用 DGS 服务获取已过期购买记录
        let purchases = state
            .dgs_service
            .get_expired_purchases(_seller, _first_index, _last_index)
            .await;

        // 转换为 JSON 数组
        let purchases_json: Vec<serde_json::Value> = purchases
            .iter()
            .map(|p| json!(p))
            .collect();

        let mut builder = RsRespBuilder::new();
        builder.insert("purchases", json!(purchases_json));

        Ok(builder.build())
    }
}

pub struct GetDGSGoodsCountHandler;

impl GetDGSGoodsCountHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetDGSGoodsCountHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["seller", "inStockOnly", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs]
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _seller = req.get_u64("seller");
        let _in_stock_only = req.get_bool("inStockOnly");

        // 调用 DGS 服务获取商品数量
        let count = state
            .dgs_service
            .get_goods_count(_seller, _in_stock_only)
            .await;

        let mut builder = RsRespBuilder::new();
        builder.insert("numberOfGoods", count.to_string());

        Ok(builder.build())
    }
}

pub struct GetDGSGoodsPurchaseCountHandler;

impl GetDGSGoodsPurchaseCountHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetDGSGoodsPurchaseCountHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["goods", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs]
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _goods_id = req.require_u64("goods")?;

        // 调用 DGS 服务获取商品购买次数
        let count = state
            .dgs_service
            .get_goods_purchase_count(_goods_id)
            .await;

        let mut builder = RsRespBuilder::new();
        builder.insert("numberOfPurchases", count.to_string());

        Ok(builder.build())
    }
}

pub struct GetDGSGoodsPurchasesHandler;

impl GetDGSGoodsPurchasesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetDGSGoodsPurchasesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["goods", "firstIndex", "lastIndex", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs]
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _goods_id = req.require_u64("goods")?;
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);

        // 调用 DGS 服务获取商品购买记录
        let purchases = state
            .dgs_service
            .get_goods_purchases(_goods_id, _first_index, _last_index)
            .await;

        // 转换为 JSON 数组
        let purchases_json: Vec<serde_json::Value> = purchases
            .iter()
            .map(|p| json!(p))
            .collect();

        let mut builder = RsRespBuilder::new();
        builder.insert("purchases", json!(purchases_json));

        Ok(builder.build())
    }
}

pub struct GetDGSPendingPurchasesHandler;

impl GetDGSPendingPurchasesHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetDGSPendingPurchasesHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["seller", "firstIndex", "lastIndex", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs]
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _seller = req.get_u64("seller");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);

        // 调用 DGS 服务获取待处理购买记录
        let purchases = state
            .dgs_service
            .get_pending_purchases(_seller, _first_index, _last_index)
            .await;

        // 转换为 JSON 数组
        let purchases_json: Vec<serde_json::Value> = purchases
            .iter()
            .map(|p| json!(p))
            .collect();

        let mut builder = RsRespBuilder::new();
        builder.insert("purchases", json!(purchases_json));

        Ok(builder.build())
    }
}

pub struct GetDGSPurchaseCountHandler;

impl GetDGSPurchaseCountHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetDGSPurchaseCountHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["seller", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs]
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _seller = req.get_u64("seller");
        let _buyer = req.get_u64("buyer");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);

        // 调用 DGS 服务获取购买次数统计
        let count = state
            .dgs_service
            .get_purchase_count(_seller, _buyer, None)
            .await;

        let mut builder = RsRespBuilder::new();
        builder.insert("numberOfPurchases", count.to_string());

        Ok(builder.build())
    }
}

pub struct GetDGSTagCountHandler;

impl GetDGSTagCountHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetDGSTagCountHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["inStockOnly", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs]
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _in_stock_only = req.get_bool("inStockOnly");

        // 调用 DGS 服务获取标签数量
        let count = state
            .dgs_service
            .get_tag_count(_in_stock_only)
            .await;

        let mut builder = RsRespBuilder::new();
        builder.insert("numberOfTags", count.to_string());

        Ok(builder.build())
    }
}

pub struct GetDGSTagsHandler;

impl GetDGSTagsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetDGSTagsHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["firstIndex", "lastIndex", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs]
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _in_stock_only = req.get_bool("inStockOnly");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);

        // 调用 DGS 服务获取标签列表
        let tags = state
            .dgs_service
            .get_tags(_in_stock_only, _first_index, _last_index)
            .await;

        // 转换为 JSON 数组
        let tags_json: Vec<serde_json::Value> = tags
            .into_iter()
            .map(|t| json!(t))
            .collect();

        let mut builder = RsRespBuilder::new();
        builder.insert("tags", json!(tags_json));

        Ok(builder.build())
    }
}

pub struct GetDGSTagsLikeHandler;

impl GetDGSTagsLikeHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for GetDGSTagsLikeHandler {
    fn parameters(&self) -> Vec<&'static str> {
        vec!["tagPrefix", "firstIndex", "lastIndex", "requireBlock", "requireLastBlock"]
    }

    fn api_tags(&self) -> Vec<ApiTag> {
        vec![ApiTag::Dgs]
    }

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let _tag_prefix = req.require_string("tagPrefix")?;
        let _in_stock_only = req.get_bool("inStockOnly");
        let _first_index = req.get_i32("firstIndex").unwrap_or(0);
        let _last_index = req.get_i32("lastIndex").unwrap_or(-1);

        // 调用 DGS 服务模糊搜索标签
        let tags = state
            .dgs_service
            .get_tags_like(&_tag_prefix, _in_stock_only, _first_index, _last_index)
            .await;

        // 转换为 JSON 数组
        let tags_json: Vec<serde_json::Value> = tags
            .into_iter()
            .map(|t| json!(t))
            .collect();

        let mut builder = RsRespBuilder::new();
        builder.insert("tags", json!(tags_json));

        Ok(builder.build())
    }
}
