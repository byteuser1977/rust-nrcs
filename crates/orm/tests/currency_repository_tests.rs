//! Currency Repository 集成测试
//!
//! 测试 CurrencyRepository 和 AccountCurrencyRepository 的全量操作。

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
    let tables = ["CURRENCY", "ACCOUNT_CURRENCY", "CURRENCY_TRANSFER", "ACCOUNT", "PUBLIC_KEY"];
    for table in &tables {
        let _ = sqlx::query(&format!("TRUNCATE TABLE \"{}\" CASCADE", table)).execute(&pool).await;
    }
    pool
}

fn make_currency(id: i64, account_id: i64, name: &str, code: &str, initial_supply: i64, max_supply: i64, height: i32) -> CurrencyModel {
    CurrencyModel {
        db_id: 0,
        id,
        account_id,
        name: name.to_string(),
        name_lower: name.to_lowercase(),
        code: code.to_string(),
        description: None,
        type: 1,
        initial_supply,
        reserve_supply: 0,
        max_supply,
        creation_height: height,
        issuance_height: 0,
        min_reserve_per_unit_nqt: 0,
        min_difficulty: 0,
        max_difficulty: 0,
        ruleset: 0,
        algorithm: 0,
        decimals: 4,
        height,
        latest: true,
    }
}

fn make_account_currency(account_id: i64, currency_id: i64, units: i64, height: i32) -> AccountCurrencyModel {
    AccountCurrencyModel {
        db_id: 0,
        account_id,
        currency_id,
        units,
        unconfirmed_units: units,
        height,
        latest: true,
    }
}

#[tokio::test]
async fn test_currency_insert_and_find_by_id() {
    let pool = setup().await;
    let repo = SqliteCurrencyRepository::new(pool.clone());
    let currency = make_currency(1001, 111, "TestCoin", "TST", 1_000_000, 10_000_000, 100);
    repo.insert(&currency).await.expect("insert failed");

    let found = repo.find_by_currency_id(1001).await.expect("find failed");
    assert!(found.is_some());
    let c = found.unwrap();
    assert_eq!(c.id, 1001);
    assert_eq!(c.name, "TestCoin");
    assert_eq!(c.code, "TST");
    assert_eq!(c.initial_supply, 1_000_000);
}

#[tokio::test]
async fn test_currency_find_by_code() {
    let pool = setup().await;
    let repo = SqliteCurrencyRepository::new(pool.clone());
    repo.insert(&make_currency(1001, 111, "TestCoin", "TST", 1000, 10000, 100)).await.expect("insert failed");

    let found = repo.find_by_code("TST").await.expect("find failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, 1001);
}

#[tokio::test]
async fn test_currency_find_by_owner() {
    let pool = setup().await;
    let repo = SqliteCurrencyRepository::new(pool.clone());
    repo.insert(&make_currency(1001, 111, "Coin1", "C1", 1000, 10000, 100)).await.expect("insert failed");
    repo.insert(&make_currency(1002, 111, "Coin2", "C2", 2000, 20000, 101)).await.expect("insert failed");
    repo.insert(&make_currency(1003, 222, "Coin3", "C3", 3000, 30000, 102)).await.expect("insert failed");

    let currencies = repo.find_by_owner(111).await.expect("find failed");
    assert_eq!(currencies.len(), 2);
}

#[tokio::test]
async fn test_currency_count() {
    let pool = setup().await;
    let repo = SqliteCurrencyRepository::new(pool.clone());
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_currency(1, 1, "C", "CC", 100, 1000, 0)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
}

#[tokio::test]
async fn test_account_currency_insert_and_find() {
    let pool = setup().await;
    let repo = SqliteAccountCurrencyRepository::new(pool.clone());
    let ac = make_account_currency(111, 1001, 500, 100);
    repo.insert(&ac).await.expect("insert failed");

    let found = repo.find_by_account_and_currency(111, 1001).await.expect("find failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().units, 500);
}

#[tokio::test]
async fn test_account_currency_find_by_account() {
    let pool = setup().await;
    let repo = SqliteAccountCurrencyRepository::new(pool.clone());
    repo.insert(&make_account_currency(111, 1001, 500, 100)).await.expect("insert failed");
    repo.insert(&make_account_currency(111, 1002, 300, 101)).await.expect("insert failed");
    repo.insert(&make_account_currency(222, 1001, 200, 102)).await.expect("insert failed");

    let currencies = repo.find_by_account(111).await.expect("find failed");
    assert_eq!(currencies.len(), 2);
}

#[tokio::test]
async fn test_account_currency_find_by_currency() {
    let pool = setup().await;
    let repo = SqliteAccountCurrencyRepository::new(pool.clone());
    repo.insert(&make_account_currency(111, 1001, 500, 100)).await.expect("insert failed");
    repo.insert(&make_account_currency(222, 1001, 200, 101)).await.expect("insert failed");

    let holders = repo.find_by_currency(1001).await.expect("find failed");
    assert_eq!(holders.len(), 2);
}

// ==================== PostgreSQL Tests ====================

#[tokio::test]
#[ignore]
async fn pg_test_currency_insert_and_find_by_id() {
    let pool = setup_pg().await;
    let repo = PgCurrencyRepository::new(pool.clone());
    let currency = make_currency(1001, 111, "TestCoin", "TST", 1_000_000, 10_000_000, 100);
    repo.insert(&currency).await.expect("insert failed");

    let found = repo.find_by_currency_id(1001).await.expect("find failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "TestCoin");
}

#[tokio::test]
#[ignore]
async fn pg_test_currency_count() {
    let pool = setup_pg().await;
    let repo = PgCurrencyRepository::new(pool.clone());
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_currency(1, 1, "C", "CC", 100, 1000, 0)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
}

#[tokio::test]
#[ignore]
async fn pg_test_account_currency_insert_and_find() {
    let pool = setup_pg().await;
    let repo = PgAccountCurrencyRepository::new(pool.clone());
    let ac = make_account_currency(111, 1001, 500, 100);
    repo.insert(&ac).await.expect("insert failed");

    let found = repo.find_by_account_and_currency(111, 1001).await.expect("find failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().units, 500);
}
