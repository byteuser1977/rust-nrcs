//! Order-related database models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use blockchain_types::{AccountId, Amount, AssetId, Height, OrderId, Result};
use blockchain_types::prelude::*;

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct AskOrderModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub asset_id: i64,
    pub price: i64,
    pub transaction_index: i16,
    pub transaction_height: i32,
    pub quantity: i64,
    pub creation_height: i32,
    pub height: i32,
    pub latest: bool,
}

impl AskOrderModel {
    pub fn to_domain(&self) -> Result<AskOrder> {
        Ok(AskOrder {
            id: self.id as OrderId,
            account_id: self.account_id as AccountId,
            asset_id: self.asset_id as AssetId,
            price: self.price as Amount,
            quantity: self.quantity as Amount,
            creation_height: self.creation_height as Height,
            transaction_height: self.transaction_height as Height,
            transaction_index: self.transaction_index as u16,
            height: self.height as Height,
        })
    }

    pub fn from_domain(order: &AskOrder) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: order.id as i64,
            account_id: order.account_id as i64,
            asset_id: order.asset_id as i64,
            price: order.price as i64,
            transaction_index: order.transaction_index as i16,
            transaction_height: order.transaction_height as i32,
            quantity: order.quantity as i64,
            creation_height: order.creation_height as i32,
            height: order.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct BidOrderModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub asset_id: i64,
    pub price: i64,
    pub transaction_index: i16,
    pub transaction_height: i32,
    pub quantity: i64,
    pub creation_height: i32,
    pub height: i32,
    pub latest: bool,
}

impl BidOrderModel {
    pub fn to_domain(&self) -> Result<BidOrder> {
        Ok(BidOrder {
            id: self.id as OrderId,
            account_id: self.account_id as AccountId,
            asset_id: self.asset_id as AssetId,
            price: self.price as Amount,
            quantity: self.quantity as Amount,
            creation_height: self.creation_height as Height,
            transaction_height: self.transaction_height as Height,
            transaction_index: self.transaction_index as u16,
            height: self.height as Height,
        })
    }

    pub fn from_domain(order: &BidOrder) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: order.id as i64,
            account_id: order.account_id as i64,
            asset_id: order.asset_id as i64,
            price: order.price as i64,
            transaction_index: order.transaction_index as i16,
            transaction_height: order.transaction_height as i32,
            quantity: order.quantity as i64,
            creation_height: order.creation_height as i32,
            height: order.height as i32,
            latest: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct TradeModel {
    pub db_id: i64,
    pub asset_id: i64,
    pub block_id: i64,
    pub ask_order_id: i64,
    pub bid_order_id: i64,
    pub ask_order_height: i32,
    pub bid_order_height: i32,
    pub seller_id: i64,
    pub buyer_id: i64,
    pub is_buy: bool,
    pub quantity: i64,
    pub price: i64,
    pub timestamp: i32,
    pub height: i32,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct CoinOrderFxtModel {
    pub db_id: i64,
    pub id: i64,
    pub account_id: i64,
    pub chain_id: i32,
    pub exchange_id: i32,
    pub full_hash: Vec<u8>,
    pub amount: i64,
    pub quantity: i64,
    pub bid_price: i64,
    pub ask_price: i64,
    pub creation_height: i32,
    pub height: i32,
    pub transaction_height: i32,
    pub transaction_index: i16,
    pub latest: bool,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct CoinTradeFxtModel {
    pub db_id: i64,
    pub chain_id: i32,
    pub exchange_id: i32,
    pub account_id: i64,
    pub block_id: i64,
    pub height: i32,
    pub timestamp: i32,
    pub exchange_quantity: i64,
    pub exchange_price: i64,
    pub order_id: i64,
    pub order_full_hash: Vec<u8>,
    pub match_id: i64,
    pub match_full_hash: Vec<u8>,
}
