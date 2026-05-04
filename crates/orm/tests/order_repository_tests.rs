//! Order Repository 集成测试
//!
//! 测试 AskOrderRepository 和 BidOrderRepository 的全量操作。

use orm::repository::*;
use orm::models::*;
use sqlx::SqlitePool;
use sqlx::PgPool;

async fn setup() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool failed");
    let schema_sql = include_str!("../../../migrations/sqlite/0.sql");
    for statement in schema_sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("--") {
            let _ = sqlx::query(trimmed).execute(&pool).await;
        }
    }
    pool
}

async fn setup_pg() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://nrcs_user:password@localhost:5432/nrcs_db".to_string());
    let pool = PgPool::connect(&database_url).await.expect("pg pool failed");
    let tables = ["ask_order", "bid_order", "trade", "account", "public_key"];
    for table in &tables {
        let _ = sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table)).execute(&pool).await;
    }
    pool
}

fn make_ask_order(id: i64, account_id: i64, asset_id: i64, price: i64, quantity: i64, height: i32) -> AskOrderModel {
    AskOrderModel {
        db_id: 0,
        id,
        account_id,
        asset_id,
        price,
        transaction_index: 0,
        transaction_height: height,
        quantity,
        creation_height: height,
        height,
        latest: true,
    }
}

fn make_bid_order(id: i64, account_id: i64, asset_id: i64, price: i64, quantity: i64, height: i32) -> BidOrderModel {
    BidOrderModel {
        db_id: 0,
        id,
        account_id,
        asset_id,
        price,
        transaction_index: 0,
        transaction_height: height,
        quantity,
        creation_height: height,
        height,
        latest: true,
    }
}

#[tokio::test]
async fn test_ask_order_insert_and_find() {
    let pool = setup().await;
    let repo = SqliteAskOrderRepository::new(pool.clone());
    let order = make_ask_order(1001, 111, 5001, 100, 50, 100);
    repo.insert(&order).await.expect("insert failed");

    let found = repo.find_by_order_id(1001).await.expect("find failed");
    assert!(found.is_some());
    let o = found.unwrap();
    assert_eq!(o.id, 1001);
    assert_eq!(o.asset_id, 5001);
    assert_eq!(o.price, 100);
    assert_eq!(o.quantity, 50);
}

#[tokio::test]
async fn test_ask_order_find_by_asset() {
    let pool = setup().await;
    let repo = SqliteAskOrderRepository::new(pool.clone());
    repo.insert(&make_ask_order(1001, 111, 5001, 100, 50, 100)).await.expect("insert failed");
    repo.insert(&make_ask_order(1002, 222, 5001, 200, 30, 101)).await.expect("insert failed");
    repo.insert(&make_ask_order(1003, 111, 5002, 150, 40, 102)).await.expect("insert failed");

    let orders = repo.find_by_asset(5001, 100).await.expect("find failed");
    assert_eq!(orders.len(), 2);
}

#[tokio::test]
async fn test_ask_order_find_by_account() {
    let pool = setup().await;
    let repo = SqliteAskOrderRepository::new(pool.clone());
    repo.insert(&make_ask_order(1001, 111, 5001, 100, 50, 100)).await.expect("insert failed");
    repo.insert(&make_ask_order(1002, 111, 5002, 200, 30, 101)).await.expect("insert failed");
    repo.insert(&make_ask_order(1003, 222, 5001, 150, 40, 102)).await.expect("insert failed");

    let orders = repo.find_by_account(111).await.expect("find failed");
    assert_eq!(orders.len(), 2);
}

#[tokio::test]
async fn test_bid_order_insert_and_find() {
    let pool = setup().await;
    let repo = SqliteBidOrderRepository::new(pool.clone());
    let order = make_bid_order(2001, 111, 5001, 100, 50, 100);
    repo.insert(&order).await.expect("insert failed");

    let found = repo.find_by_order_id(2001).await.expect("find failed");
    assert!(found.is_some());
    let o = found.unwrap();
    assert_eq!(o.id, 2001);
    assert_eq!(o.asset_id, 5001);
    assert_eq!(o.price, 100);
}

#[tokio::test]
async fn test_bid_order_find_by_asset() {
    let pool = setup().await;
    let repo = SqliteBidOrderRepository::new(pool.clone());
    repo.insert(&make_bid_order(2001, 111, 5001, 100, 50, 100)).await.expect("insert failed");
    repo.insert(&make_bid_order(2002, 222, 5001, 200, 30, 101)).await.expect("insert failed");
    repo.insert(&make_bid_order(2003, 111, 5002, 150, 40, 102)).await.expect("insert failed");

    let orders = repo.find_by_asset(5001, 100).await.expect("find failed");
    assert_eq!(orders.len(), 2);
}

#[tokio::test]
async fn test_bid_order_find_by_account() {
    let pool = setup().await;
    let repo = SqliteBidOrderRepository::new(pool.clone());
    repo.insert(&make_bid_order(2001, 111, 5001, 100, 50, 100)).await.expect("insert failed");
    repo.insert(&make_bid_order(2002, 111, 5002, 200, 30, 101)).await.expect("insert failed");

    let orders = repo.find_by_account(111).await.expect("find failed");
    assert_eq!(orders.len(), 2);
}

#[tokio::test]
async fn test_ask_order_count() {
    let pool = setup().await;
    let repo = SqliteAskOrderRepository::new(pool.clone());
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_ask_order(1, 1, 1, 100, 50, 0)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
}

#[tokio::test]
async fn test_bid_order_count() {
    let pool = setup().await;
    let repo = SqliteBidOrderRepository::new(pool.clone());
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_bid_order(1, 1, 1, 100, 50, 0)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
}

// ==================== PostgreSQL Tests ====================

#[tokio::test]
#[ignore]
async fn pg_test_ask_order_insert_and_find() {
    let pool = setup_pg().await;
    let repo = PgAskOrderRepository::new(pool.clone());
    let order = make_ask_order(1001, 111, 5001, 100, 50, 100);
    repo.insert(&order).await.expect("insert failed");

    let found = repo.find_by_order_id(1001).await.expect("find failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().asset_id, 5001);
}

#[tokio::test]
#[ignore]
async fn pg_test_bid_order_insert_and_find() {
    let pool = setup_pg().await;
    let repo = PgBidOrderRepository::new(pool.clone());
    let order = make_bid_order(2001, 111, 5001, 100, 50, 100);
    repo.insert(&order).await.expect("insert failed");

    let found = repo.find_by_order_id(2001).await.expect("find failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().asset_id, 5001);
}

#[tokio::test]
#[ignore]
async fn pg_test_ask_order_count() {
    let pool = setup_pg().await;
    let repo = PgAskOrderRepository::new(pool.clone());
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_ask_order(1, 1, 1, 100, 50, 0)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
}

#[tokio::test]
#[ignore]
async fn pg_test_bid_order_count() {
    let pool = setup_pg().await;
    let repo = PgBidOrderRepository::new(pool.clone());
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_bid_order(1, 1, 1, 100, 50, 0)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
}
