//! DGS（Decentralized Goods Store）服务模块
//!
//! 对应 NRCS Java: `com.bytechain.nrcs.service.digitalGoods.DigitalGoodsStore`
//!
//! 功能：
//! - 商品管理（上架、下架、价格修改、数量修改）
//! - 购买记录管理
//! - 标签查询

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// DGS 商品信息
///
/// 对应 NRCS Java: `DigitalGoodsStoreGoods`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DGSGoods {
    /// 商品 ID (db_id)
    #[serde(rename = "goods")]
    pub id: u64,
    /// 卖家账户 ID
    #[serde(rename = "seller")]
    pub seller_id: u64,
    /// 商品名称
    pub name: String,
    /// 商品描述
    pub description: String,
    /// 数量
    pub quantity: i32,
    /// 价格 (NQT)
    #[serde(rename = "priceNQT")]
    pub price_nqt: u64,
    /// 标签
    pub tags: Option<String>,
    /// 是否已删除/下架
    pub delisted: bool,
    /// 时间戳
    pub timestamp: i32,
}

/// DGS 购买信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DGSPurchase {
    /// 购买 ID
    #[serde(rename = "purchase")]
    pub id: u64,
    /// 商品 ID
    #[serde(rename = "goods")]
    pub goods_id: u64,
    /// 买家 ID
    #[serde(rename = "buyer")]
    pub buyer_id: u64,
    /// 卖家 ID
    #[serde(rename = "seller")]
    pub seller_id: u64,
    /// 数量
    pub quantity: i32,
    /// 价格 (NQT)
    #[serde(rename = "priceNQT")]
    pub price_nqt: u64,
    /// 到期时间戳
    pub deadline: i32,
    /// 购买时间戳
    pub timestamp: i32,
    /// 是否已送达
    pub delivered: bool,
    /// 是否已退款
    pub refunded: bool,
    /// 是否已过期
    pub expired: bool,
    /// 是否待处理（未送达且未过期）
    pub pending: bool,
}

/// DGS 服务 Trait
///
/// 定义 DGS 操作的 API 接口
#[async_trait::async_trait]
pub trait DGSServiceApi: Send + Sync {
    /// 修改商品价格
    ///
    /// 对应 NRCS Java: `DGSPriceChange.processRequest()`
    async fn change_price(
        &self,
        goods_id: u64,
        seller_id: u64,
        new_price_nqt: u64,
    ) -> Result<DGSGoods, String>;

    async fn change_quantity(
        &self,
        goods_id: u64,
        seller_id: u64,
        delta_quantity: i32,
    ) -> Result<DGSGoods, String>;

    async fn get_goods(&self, goods_id: u64) -> Option<DGSGoods>;

    async fn get_goods_list(
        &self,
        seller_id: Option<u64>,
        in_stock_only: bool,
        first_index: i32,
        last_index: i32,
    ) -> Vec<DGSGoods>;

    async fn get_purchase(&self, purchase_id: u64) -> Option<DGSPurchase>;

    async fn get_purchases(
        &self,
        seller_id: Option<u64>,
        buyer_id: Option<u64>,
        first_index: i32,
        last_index: i32,
    ) -> Vec<DGSPurchase>;

    async fn get_goods_count(&self, seller_id: Option<u64>, in_stock_only: bool) -> i64;

    /// 获取商品的购买次数统计
    ///
    /// 对应 NRCS Java: `GetDGSGoodsPurchaseCount.processRequest()`
    async fn get_goods_purchase_count(&self, goods_id: u64) -> i64;

    /// 获取商品的所有购买记录
    ///
    /// 对应 NRCS Java: `GetDGSGoodsPurchases.processRequest()`
    async fn get_goods_purchases(
        &self,
        goods_id: u64,
        first_index: i32,
        last_index: i32,
    ) -> Vec<DGSPurchase>;

    /// 获取已过期的购买记录
    ///
    /// 对应 NRCS Java: `GetDGSExpiredPurchases.processRequest()`
    async fn get_expired_purchases(
        &self,
        seller_id: Option<u64>,
        first_index: i32,
        last_index: i32,
    ) -> Vec<DGSPurchase>;

    /// 获取待处理的购买记录
    ///
    /// 对应 NRCS Java: `GetDGSPendingPurchases.processRequest()`
    async fn get_pending_purchases(
        &self,
        seller_id: Option<u64>,
        first_index: i32,
        last_index: i32,
    ) -> Vec<DGSPurchase>;

    /// 获取总购买次数
    ///
    /// 对应 NRCS Java: `GetDGSPurchaseCount.processRequest()`
    async fn get_purchase_count(
        &self,
        seller_id: Option<u64>,
        buyer_id: Option<u64>,
        completed: Option<bool>,
    ) -> i64;

    /// 获取标签数量
    ///
    /// 对应 NRCS Java: `GetDGSTagCount.processRequest()`
    async fn get_tag_count(&self, in_stock_only: bool) -> i64;

    /// 获取所有标签列表
    ///
    /// 对应 NRCS Java: `GetDGSTags.processRequest()`
    async fn get_tags(&self, in_stock_only: bool, first_index: i32, last_index: i32) -> Vec<String>;

    /// 模糊搜索标签
    ///
    /// 对应 NRCS Java: `GetDGSTagsLike.processRequest()`
    async fn get_tags_like(
        &self,
        tag_prefix: &str,
        in_stock_only: bool,
        first_index: i32,
        last_index: i32,
    ) -> Vec<String>;
}

/// 内存实现的 DGS 服务
///
/// 用于开发和测试环境，生产环境应使用持久化实现
#[derive(Clone)]
pub struct MemoryDGSService {
    goods: Arc<RwLock<HashMap<u64, DGSGoods>>>,
    purchases: Arc<RwLock<HashMap<u64, DGSPurchase>>>,
}

impl Default for MemoryDGSService {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryDGSService {
    /// 创建新的内存 DGS 服务
    pub fn new() -> Self {
        Self {
            goods: Arc::new(RwLock::new(HashMap::new())),
            purchases: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 分页处理辅助函数
    fn paginate<T: Clone>(items: &[T], first_index: i32, last_index: i32) -> Vec<T> {
        let start = if first_index < 0 { 0 } else { first_index as usize };
        let end = if last_index < 0 {
            items.len()
        } else {
            (last_index as usize + 1).min(items.len())
        };

        if start >= items.len() || start > end {
            return vec![];
        }

        items[start..end].to_vec()
    }
}

#[async_trait::async_trait]
impl DGSServiceApi for MemoryDGSService {
    async fn change_price(
        &self,
        goods_id: u64,
        seller_id: u64,
        new_price_nqt: u64,
    ) -> Result<DGSGoods, String> {
        let mut goods_map = self.goods.write().await;
        
        match goods_map.get_mut(&goods_id) {
            Some(goods) => {
                if goods.delisted {
                    return Err("Goods is delisted".to_string());
                }
                if goods.seller_id != seller_id {
                    return Err("Not the owner of this goods".to_string());
                }
                goods.price_nqt = new_price_nqt;
                Ok(goods.clone())
            }
            None => Err(format!("Goods {} not found", goods_id)),
        }
    }

    async fn change_quantity(
        &self,
        goods_id: u64,
        seller_id: u64,
        delta_quantity: i32,
    ) -> Result<DGSGoods, String> {
        let mut goods_map = self.goods.write().await;
        
        match goods_map.get_mut(&goods_id) {
            Some(goods) => {
                if goods.delisted {
                    return Err("Goods is delisted".to_string());
                }
                if goods.seller_id != seller_id {
                    return Err("Not the owner of this goods".to_string());
                }
                goods.quantity += delta_quantity;
                Ok(goods.clone())
            }
            None => Err(format!("Goods {} not found", goods_id)),
        }
    }

    async fn get_goods(&self, goods_id: u64) -> Option<DGSGoods> {
        let goods_map = self.goods.read().await;
        goods_map.get(&goods_id).cloned()
    }

    async fn get_goods_list(
        &self,
        seller_id: Option<u64>,
        in_stock_only: bool,
        first_index: i32,
        last_index: i32,
    ) -> Vec<DGSGoods> {
        let goods_map = self.goods.read().await;
        let mut matching: Vec<DGSGoods> = goods_map
            .values()
            .filter(|g| !g.delisted)
            .filter(|g| {
                if let Some(seller) = seller_id {
                    g.seller_id == seller
                } else {
                    true
                }
            })
            .filter(|g| {
                if in_stock_only {
                    g.quantity > 0
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        matching.sort_by_key(|g| g.id);
        Self::paginate(&matching, first_index, last_index)
    }

    async fn get_purchase(&self, purchase_id: u64) -> Option<DGSPurchase> {
        let purchases_map = self.purchases.read().await;
        purchases_map.get(&purchase_id).cloned()
    }

    async fn get_purchases(
        &self,
        seller_id: Option<u64>,
        buyer_id: Option<u64>,
        first_index: i32,
        last_index: i32,
    ) -> Vec<DGSPurchase> {
        let purchases_map = self.purchases.read().await;
        let mut matching: Vec<DGSPurchase> = purchases_map
            .values()
            .filter(|p| {
                if let Some(seller) = seller_id {
                    p.seller_id == seller
                } else {
                    true
                }
            })
            .filter(|p| {
                if let Some(buyer) = buyer_id {
                    p.buyer_id == buyer
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        matching.sort_by_key(|p| p.id);
        Self::paginate(&matching, first_index, last_index)
    }

    async fn get_goods_count(&self, seller_id: Option<u64>, in_stock_only: bool) -> i64 {
        let goods_map = self.goods.read().await;
        
        goods_map.values()
            .filter(|g| !g.delisted)
            .filter(|g| {
                if let Some(seller) = seller_id {
                    g.seller_id == seller
                } else {
                    true
                }
            })
            .filter(|g| {
                if in_stock_only {
                    g.quantity > 0
                } else {
                    true
                }
            })
            .count() as i64
    }

    async fn get_goods_purchase_count(&self, goods_id: u64) -> i64 {
        let purchases_map = self.purchases.read().await;
        
        purchases_map.values()
            .filter(|p| p.goods_id == goods_id)
            .count() as i64
    }

    async fn get_goods_purchases(
        &self,
        goods_id: u64,
        first_index: i32,
        last_index: i32,
    ) -> Vec<DGSPurchase> {
        let purchases_map = self.purchases.read().await;
        
        let matching: Vec<DGSPurchase> = purchases_map
            .values()
            .filter(|p| p.goods_id == goods_id)
            .cloned()
            .collect();

        Self::paginate(&matching, first_index, last_index)
    }

    async fn get_expired_purchases(
        &self,
        seller_id: Option<u64>,
        first_index: i32,
        last_index: i32,
    ) -> Vec<DGSPurchase> {
        let purchases_map = self.purchases.read().await;
        let now = current_epoch_time();
        
        let matching: Vec<DGSPurchase> = purchases_map
            .values()
            .filter(|p| {
                // 已过期：deadline < 当前时间 且 未送达 且 未退款
                p.deadline < now && !p.delivered && !p.refunded && !p.expired
            })
            .filter(|p| {
                if let Some(seller) = seller_id {
                    p.seller_id == seller
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        Self::paginate(&matching, first_index, last_index)
    }

    async fn get_pending_purchases(
        &self,
        seller_id: Option<u64>,
        first_index: i32,
        last_index: i32,
    ) -> Vec<DGSPurchase> {
        let purchases_map = self.purchases.read().await;
        let now = current_epoch_time();
        
        let matching: Vec<DGSPurchase> = purchases_map
            .values()
            .filter(|p| {
                // 待处理：未送达 且 未退款 且 未过期 且 deadline > 当前时间
                !p.delivered && !p.refunded && !p.expired && p.deadline > now
            })
            .filter(|p| {
                if let Some(seller) = seller_id {
                    p.seller_id == seller
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        Self::paginate(&matching, first_index, last_index)
    }

    async fn get_purchase_count(
        &self,
        seller_id: Option<u64>,
        _buyer_id: Option<u64>,
        completed: Option<bool>,
    ) -> i64 {
        let purchases_map = self.purchases.read().await;
        
        purchases_map.values()
            .filter(|p| {
                if let Some(seller) = seller_id {
                    p.seller_id == seller
                } else {
                    true
                }
            })
            .filter(|_p| true)
            .filter(|p| {
                if let Some(completed_flag) = completed {
                    p.delivered == completed_flag
                } else {
                    true
                }
            })
            .count() as i64
    }

    async fn get_tag_count(&self, in_stock_only: bool) -> i64 {
        let goods_map = self.goods.read().await;
        
        let tags: std::collections::HashSet<String> = goods_map
            .values()
            .filter(|g| !g.delisted)
            .filter(|g| {
                if in_stock_only {
                    g.quantity > 0
                } else {
                    true
                }
            })
            .filter_map(|g| g.tags.as_ref().cloned())
            .flat_map(|tags_str| {
                tags_str.split(',').map(str::trim).map(String::from).collect::<Vec<_>>()
            })
            .collect();

        tags.len() as i64
    }

    async fn get_tags(&self, in_stock_only: bool, first_index: i32, last_index: i32) -> Vec<String> {
        let goods_map = self.goods.read().await;
        
        let tags: std::collections::HashSet<String> = goods_map
            .values()
            .filter(|g| !g.delisted)
            .filter(|g| {
                if in_stock_only {
                    g.quantity > 0
                } else {
                    true
                }
            })
            .filter_map(|g| g.tags.as_ref().cloned())
            .flat_map(|tags_str| {
                tags_str.split(',').map(str::trim).map(String::from).collect::<Vec<_>>()
            })
            .collect();

        let mut tags_list: Vec<String> = tags.into_iter().collect();
        tags_list.sort();
        
        Self::paginate(&tags_list, first_index, last_index)
    }

    async fn get_tags_like(
        &self,
        tag_prefix: &str,
        in_stock_only: bool,
        first_index: i32,
        last_index: i32,
    ) -> Vec<String> {
        let all_tags = self.get_tags(in_stock_only, 0, -1).await;
        
        let matched: Vec<String> = all_tags
            .into_iter()
            .filter(|tag| tag.to_lowercase().starts_with(&tag_prefix.to_lowercase()))
            .collect();

        Self::paginate(&matched, first_index, last_index)
    }
}

/// Get current epoch time (seconds since genesis)
fn current_epoch_time() -> i32 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let genesis_timestamp: i64 = 1514764800; // 2018-01-01 00:00:00 UTC
    ((now.as_secs() as i64) - genesis_timestamp) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dgs_service_creation() {
        let service = MemoryDGSService::new();
        let count = service.get_goods_count(None, false).await;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_change_price_and_quantity() {
        let service = MemoryDGSService::new();

        // 先添加一个测试商品
        let test_goods = DGSGoods {
            id: 1,
            seller_id: 100,
            name: "Test Product".to_string(),
            description: "Test Description".to_string(),
            quantity: 10,
            price_nqt: 1000000,
            tags: Some("electronics,gadgets".to_string()),
            delisted: false,
            timestamp: 1000,
        };

        service.goods.write().await.insert(1, test_goods);

        // 测试修改价格
        let updated = service.change_price(1, 100, 2000000).await.unwrap();
        assert_eq!(updated.price_nqt, 2000000);

        // 测试修改数量
        let updated = service.change_quantity(1, 100, -3).await.unwrap();
        assert_eq!(updated.quantity, 7);
    }

    #[tokio::test]
    async fn test_get_purchase_stats() {
        let service = MemoryDGSService::new();

        // 添加测试购买记录
        let purchase = DGSPurchase {
            id: 1,
            goods_id: 10,
            buyer_id: 200,
            seller_id: 100,
            quantity: 2,
            price_nqt: 500000,
            deadline: current_epoch_time() + 3600, // 1小时后到期
            timestamp: current_epoch_time() - 600,   // 10分钟前购买
            delivered: false,
            refunded: false,
            expired: false,
            pending: true,
        };
        service.purchases.write().await.insert(1, purchase);

        // 测试查询
        let count = service.get_goods_purchase_count(10).await;
        assert_eq!(count, 1);

        let pending = service.get_pending_purchases(None, 0, -1).await;
        assert_eq!(pending.len(), 1);
    }

    #[tokio::test]
    async fn test_get_tags() {
        let service = MemoryDGSService::new();

        // 添加测试商品
        let goods1 = DGSGoods {
            id: 1,
            seller_id: 100,
            name: "Product 1".to_string(),
            description: "".to_string(),
            quantity: 5,
            price_nqt: 1000,
            tags: Some("electronics,phones".to_string()),
            delisted: false,
            timestamp: 1000,
        };
        service.goods.write().await.insert(1, goods1);

        let goods2 = DGSGoods {
            id: 2,
            seller_id: 101,
            name: "Product 2".to_string(),
            description: "".to_string(),
            quantity: 3,
            price_nqt: 2000,
            tags: Some("computers,laptops".to_string()),
            delisted: false,
            timestamp: 1000,
        };
        service.goods.write().await.insert(2, goods2);

        // 测试标签查询
        let tags = service.get_tags(false, 0, -1).await;
        assert_eq!(tags.len(), 4); // electronics, phones, computers, laptops

        let count = service.get_tag_count(false).await;
        assert_eq!(count, 4);

        let like_results = service.get_tags_like("elec", false, 0, -1).await;
        assert_eq!(like_results.len(), 1);
        assert_eq!(like_results[0], "electronics");
    }
}
