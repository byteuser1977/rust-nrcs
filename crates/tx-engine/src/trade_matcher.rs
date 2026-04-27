//! Trade Matcher Module
//!
//! 对应 Java: Order.matchOrders()
//!
//! 负责:
//! - 匹配 Ask 和 Bid 订单
//! - 创建 Trade 记录
//! - 更新账户资产余额

use orm::models::{AskOrderModel, BidOrderModel, TradeModel, AccountAssetModel};
use orm::repository::{AskOrderRepository, BidOrderRepository, TradeRepository, AccountAssetRepository, Repository};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TradeMatcherError {
    #[error("database error: {0}")]
    Database(#[from] orm::repository::RepositoryError),
    
    #[error("order not found: {0}")]
    OrderNotFound(i64),
    
    #[error("insufficient balance")]
    InsufficientBalance,
    
    #[error("invalid order state")]
    InvalidOrderState,
}

pub type TradeMatcherResult<T> = std::result::Result<T, TradeMatcherError>;

#[derive(Debug, Clone)]
pub struct MatchedTrade {
    pub ask_order_id: i64,
    pub bid_order_id: i64,
    pub quantity: i64,
    pub price: i64,
    pub seller_id: i64,
    pub buyer_id: i64,
    pub asset_id: i64,
}

#[async_trait]
pub trait TradeMatcher: Send + Sync {
    async fn match_orders(
        &self,
        asset_id: i64,
        height: i32,
        timestamp: i32,
        block_id: i64,
    ) -> TradeMatcherResult<Vec<MatchedTrade>>;
    
    async fn match_new_order(
        &self,
        order_id: i64,
        is_ask: bool,
        height: i32,
        timestamp: i32,
        block_id: i64,
    ) -> TradeMatcherResult<Vec<MatchedTrade>>;
}

pub struct DefaultTradeMatcher<A, B, T, AA>
where
    A: AskOrderRepository,
    B: BidOrderRepository,
    T: TradeRepository,
    AA: AccountAssetRepository,
{
    ask_repo: A,
    bid_repo: B,
    trade_repo: T,
    account_asset_repo: AA,
}

impl<A, B, T, AA> DefaultTradeMatcher<A, B, T, AA>
where
    A: AskOrderRepository,
    B: BidOrderRepository,
    T: TradeRepository,
    AA: AccountAssetRepository,
{
    pub fn new(
        ask_repo: A,
        bid_repo: B,
        trade_repo: T,
        account_asset_repo: AA,
    ) -> Self {
        Self {
            ask_repo,
            bid_repo,
            trade_repo,
            account_asset_repo,
        }
    }
    
    fn create_trade_model(
        &self,
        matched: &MatchedTrade,
        height: i32,
        timestamp: i32,
        block_id: i64,
    ) -> TradeModel {
        TradeModel {
            db_id: 0,
            asset_id: matched.asset_id,
            block_id,
            ask_order_id: matched.ask_order_id,
            bid_order_id: matched.bid_order_id,
            ask_order_height: height,
            bid_order_height: height,
            seller_id: matched.seller_id,
            buyer_id: matched.buyer_id,
            is_buy: false,
            quantity: matched.quantity,
            price: matched.price,
            timestamp,
            height,
        }
    }
}

#[async_trait]
impl<A, B, T, AA> TradeMatcher for DefaultTradeMatcher<A, B, T, AA>
where
    A: AskOrderRepository,
    B: BidOrderRepository,
    T: TradeRepository,
    AA: AccountAssetRepository,
{
    async fn match_orders(
        &self,
        asset_id: i64,
        height: i32,
        timestamp: i32,
        block_id: i64,
    ) -> TradeMatcherResult<Vec<MatchedTrade>> {
        let ask_orders = self.ask_repo.find_by_asset(asset_id, 100).await?;
        let bid_orders = self.bid_repo.find_by_asset(asset_id, 100).await?;
        
        let mut matched_trades = Vec::new();
        
        for ask in ask_orders {
            if ask.quantity <= 0 {
                continue;
            }
            
            for bid in &bid_orders {
                if bid.quantity <= 0 {
                    continue;
                }
                
                if ask.price <= bid.price {
                    let trade_quantity = std::cmp::min(ask.quantity, bid.quantity);
                    
                    let matched = MatchedTrade {
                        ask_order_id: ask.id,
                        bid_order_id: bid.id,
                        quantity: trade_quantity,
                        price: ask.price,
                        seller_id: ask.account_id,
                        buyer_id: bid.account_id,
                        asset_id,
                    };
                    
                    matched_trades.push(matched);
                }
            }
        }
        
        for matched in &matched_trades {
            let trade = self.create_trade_model(matched, height, timestamp, block_id);
            self.trade_repo.insert(&trade).await?;
        }
        
        Ok(matched_trades)
    }
    
    async fn match_new_order(
        &self,
        order_id: i64,
        is_ask: bool,
        height: i32,
        timestamp: i32,
        block_id: i64,
    ) -> TradeMatcherResult<Vec<MatchedTrade>> {
        let mut matched_trades = Vec::new();
        
        if is_ask {
            let ask = self.ask_repo.find_by_order_id(order_id).await?
                .ok_or(TradeMatcherError::OrderNotFound(order_id))?;
            
            let bid_orders = self.bid_repo.find_by_asset(ask.asset_id, 100).await?;
            
            for bid in bid_orders {
                if bid.quantity <= 0 {
                    continue;
                }
                
                if ask.price <= bid.price && ask.quantity > 0 {
                    let trade_quantity = std::cmp::min(ask.quantity, bid.quantity);
                    
                    let matched = MatchedTrade {
                        ask_order_id: ask.id,
                        bid_order_id: bid.id,
                        quantity: trade_quantity,
                        price: ask.price,
                        seller_id: ask.account_id,
                        buyer_id: bid.account_id,
                        asset_id: ask.asset_id,
                    };
                    
                    matched_trades.push(matched);
                }
            }
        } else {
            let bid = self.bid_repo.find_by_order_id(order_id).await?
                .ok_or(TradeMatcherError::OrderNotFound(order_id))?;
            
            let ask_orders = self.ask_repo.find_by_asset(bid.asset_id, 100).await?;
            
            for ask in ask_orders {
                if ask.quantity <= 0 {
                    continue;
                }
                
                if ask.price <= bid.price && bid.quantity > 0 {
                    let trade_quantity = std::cmp::min(ask.quantity, bid.quantity);
                    
                    let matched = MatchedTrade {
                        ask_order_id: ask.id,
                        bid_order_id: bid.id,
                        quantity: trade_quantity,
                        price: ask.price,
                        seller_id: ask.account_id,
                        buyer_id: bid.account_id,
                        asset_id: bid.asset_id,
                    };
                    
                    matched_trades.push(matched);
                }
            }
        }
        
        for matched in &matched_trades {
            let trade = self.create_trade_model(matched, height, timestamp, block_id);
            self.trade_repo.insert(&trade).await?;
        }
        
        Ok(matched_trades)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_matched_trade_creation() {
        let trade = MatchedTrade {
            ask_order_id: 1,
            bid_order_id: 2,
            quantity: 100,
            price: 1000,
            seller_id: 100,
            buyer_id: 200,
            asset_id: 12345,
        };
        
        assert_eq!(trade.ask_order_id, 1);
        assert_eq!(trade.bid_order_id, 2);
        assert_eq!(trade.quantity, 100);
    }
}
