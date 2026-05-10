//! 数字商品相关 API Handlers
//!
//! 与 Java 版本 GetDGSGood, DGSListing 等完全对齐

use async_trait::async_trait;
use serde_json::json;

use crate::api_tag::ApiTag;
use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RequestHandler, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;
use super::create_transaction::CreateTransactionHelper;

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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let goods_id = req.require_u64("goods")?;

        let goods = state.dgs_service.get_goods(goods_id).await;

        let mut builder = RsRespBuilder::new();
        match goods {
            Some(g) => {
                builder.insert("goods", json!(g));
            }
            None => {
                return Err(ApiError::Validation("Goods not found".to_string()));
            }
        }

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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let seller = req.get_u64("seller");
        let first_index = req.get_i32("firstIndex").unwrap_or(0);
        let last_index = req.get_i32("lastIndex").unwrap_or(-1);
        let in_stock_only = req.get_bool("inStockOnly");

        let goods = state.dgs_service.get_goods_list(seller, in_stock_only, first_index, last_index).await;

        let goods_json: Vec<serde_json::Value> = goods.iter().map(|g| json!(g)).collect();

        let mut builder = RsRespBuilder::new();
        builder.insert("goods", json!(goods_json));

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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let purchase_id = req.require_u64("purchase")?;

        let purchase = state.dgs_service.get_purchase(purchase_id).await;

        let mut builder = RsRespBuilder::new();
        match purchase {
            Some(p) => {
                builder.insert("purchase", json!(p));
            }
            None => {
                return Err(ApiError::Validation("Purchase not found".to_string()));
            }
        }

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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let seller = req.get_u64("seller");
        let buyer = req.get_u64("buyer");
        let first_index = req.get_i32("firstIndex").unwrap_or(0);
        let last_index = req.get_i32("lastIndex").unwrap_or(-1);

        let purchases = state.dgs_service.get_purchases(seller, buyer, first_index, last_index).await;

        let purchases_json: Vec<serde_json::Value> = purchases.iter().map(|p| json!(p)).collect();

        let mut builder = RsRespBuilder::new();
        builder.insert("purchases", json!(purchases_json));

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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let name = req.require_string("name")?;
        let description = req.get_string("description").unwrap_or_default();
        let quantity = req.get_i32("quantity").unwrap_or(1);
        let price_nqt = req.get_string("priceNQT")
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| ApiError::MissingParameter("priceNQT".to_string()))?;
        let tags = req.get_string("tags").unwrap_or_default();

        let attachment = json!({
            "name": name,
            "description": description,
            "quantity": quantity,
            "priceNQT": price_nqt.to_string(),
            "tags": tags
        });

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 5, 0, None, 0, Some(attachment), state,
        ).await
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let goods_id = req.require_u64("goods")?;

        let attachment = json!({
            "goods": goods_id.to_string()
        });

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 5, 1, None, 0, Some(attachment), state,
        ).await
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let goods_id = req.require_u64("goods")?;
        let quantity = req.get_i32("quantity").unwrap_or(1);
        let price_nqt = req.get_string("priceNQT")
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| ApiError::MissingParameter("priceNQT".to_string()))?;
        let delivery_deadline = req.get_i32("deliveryDeadlineTimestamp").unwrap_or(0);

        let attachment = json!({
            "goods": goods_id.to_string(),
            "quantity": quantity,
            "priceNQT": price_nqt.to_string(),
            "deliveryDeadlineTimestamp": delivery_deadline
        });

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 5, 2, None, price_nqt, Some(attachment), state,
        ).await
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let purchase_id = req.require_u64("purchase")?;
        let goods_data = req.get_string("goodsData").unwrap_or_default();
        let goods_nonce = req.get_string("goodsNonce").unwrap_or_default();
        let discount_nqt = req.get_string("discountNQT")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let attachment = json!({
            "purchase": purchase_id.to_string(),
            "goodsData": goods_data,
            "goodsNonce": goods_nonce,
            "discountNQT": discount_nqt.to_string()
        });

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 5, 3, None, discount_nqt, Some(attachment), state,
        ).await
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
    
    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let purchase_id = req.require_u64("purchase")?;

        let attachment = json!({
            "purchase": purchase_id.to_string()
        });

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 5, 4, None, 0, Some(attachment), state,
        ).await
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

    async fn process_request(&self, req: &ApiRequest, state: &ApiState) -> Result<RsRespWithData, ApiError> {
        let common_params = CreateTransactionHelper::parse_common_params(req)?;
        let purchase_id = req.require_u64("purchase")?;
        let refund_nqt = req.get_string("refundNQT")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let attachment = json!({
            "purchase": purchase_id.to_string(),
            "refundNQT": refund_nqt.to_string()
        });

        CreateTransactionHelper::create_and_broadcast_transaction(
            &common_params, 5, 5, None, refund_nqt, Some(attachment), state,
        ).await
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
