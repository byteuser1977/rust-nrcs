//! Asset Repository 集成测试
//!
//! 测试 AssetRepository 和 AccountAssetRepository 的全量操作。

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
    let tables = ["asset", "ACCOUNT_ASSET", "ASSET_TRANSFER", "ACCOUNT", "PUBLIC_KEY"];
    for table in &tables {
        let _ = sqlx::query(&format!("TRUNCATE TABLE \"{}\" CASCADE", table)).execute(&pool).await;
    }
    pool
}

fn make_asset(id: i64, account_id: i64, name: &str, quantity: i64, decimals: i16, height: i32) -> AssetModel {
    AssetModel {
        db_id: 0,
        id,
        account_id,
        name: name.to_string(),
        description: None,
        quantity,
        decimals,
        has_control_phasing: false,
        initial_quantity: quantity,
        height,
        latest: true,
    }
}

fn make_account_asset(account_id: i64, asset_id: i64, quantity: i64, height: i32) -> AccountAssetModel {
    AccountAssetModel {
        db_id: 0,
        account_id,
        asset_id,
        quantity,
        unconfirmed_quantity: quantity,
        height,
        latest: true,
    }
}

#[tokio::test]
async fn test_asset_insert_and_find_by_id() {
    let pool = setup().await;
    let repo = SqliteAssetRepository::new(pool.clone());
    let asset = make_asset(1001, 111, "TestAsset", 1_000_000, 4, 100);
    repo.insert(&asset).await.expect("insert failed");

    let found = repo.find_by_asset_id(1001).await.expect("find failed");
    assert!(found.is_some());
    let a = found.unwrap();
    assert_eq!(a.id, 1001);
    assert_eq!(a.name, "TestAsset");
    assert_eq!(a.quantity, 1_000_000);
    assert_eq!(a.decimals, 4);
}

#[tokio::test]
async fn test_asset_find_by_owner() {
    let pool = setup().await;
    let repo = SqliteAssetRepository::new(pool.clone());
    repo.insert(&make_asset(1001, 111, "Asset1", 1000, 4, 100)).await.expect("insert failed");
    repo.insert(&make_asset(1002, 111, "Asset2", 2000, 2, 101)).await.expect("insert failed");
    repo.insert(&make_asset(1003, 222, "Asset3", 3000, 8, 102)).await.expect("insert failed");

    let assets = repo.find_by_owner(111).await.expect("find failed");
    assert_eq!(assets.len(), 2);
}

#[tokio::test]
async fn test_asset_count() {
    let pool = setup().await;
    let repo = SqliteAssetRepository::new(pool.clone());
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_asset(1, 1, "A", 100, 4, 0)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
}

#[tokio::test]
async fn test_account_asset_insert_and_find() {
    let pool = setup().await;
    let repo = SqliteAccountAssetRepository::new(pool.clone());
    let aa = make_account_asset(111, 1001, 500, 100);
    repo.insert(&aa).await.expect("insert failed");

    let found = repo.find_by_account_and_asset(111, 1001).await.expect("find failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().quantity, 500);
}

#[tokio::test]
async fn test_account_asset_find_by_account() {
    let pool = setup().await;
    let repo = SqliteAccountAssetRepository::new(pool.clone());
    repo.insert(&make_account_asset(111, 1001, 500, 100)).await.expect("insert failed");
    repo.insert(&make_account_asset(111, 1002, 300, 101)).await.expect("insert failed");
    repo.insert(&make_account_asset(222, 1001, 200, 102)).await.expect("insert failed");

    let assets = repo.find_by_account(111).await.expect("find failed");
    assert_eq!(assets.len(), 2);
}

#[tokio::test]
async fn test_account_asset_find_by_asset() {
    let pool = setup().await;
    let repo = SqliteAccountAssetRepository::new(pool.clone());
    repo.insert(&make_account_asset(111, 1001, 500, 100)).await.expect("insert failed");
    repo.insert(&make_account_asset(222, 1001, 200, 101)).await.expect("insert failed");

    let holders = repo.find_by_asset(1001).await.expect("find failed");
    assert_eq!(holders.len(), 2);
}

// ==================== PostgreSQL Tests ====================

#[tokio::test]
#[ignore]
async fn pg_test_asset_insert_and_find_by_id() {
    let pool = setup_pg().await;
    let repo = PgAssetRepository::new(pool.clone());
    let asset = make_asset(1001, 111, "TestAsset", 1_000_000, 4, 100);
    repo.insert(&asset).await.expect("insert failed");

    let found = repo.find_by_asset_id(1001).await.expect("find failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "TestAsset");
}

#[tokio::test]
#[ignore]
async fn pg_test_asset_count() {
    let pool = setup_pg().await;
    let repo = PgAssetRepository::new(pool.clone());
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_asset(1, 1, "A", 100, 4, 0)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
}

#[tokio::test]
#[ignore]
async fn pg_test_account_asset_insert_and_find() {
    let pool = setup_pg().await;
    let repo = PgAccountAssetRepository::new(pool.clone());
    let aa = make_account_asset(111, 1001, 500, 100);
    repo.insert(&aa).await.expect("insert failed");

    let found = repo.find_by_account_and_asset(111, 1001).await.expect("find failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().quantity, 500);
}
