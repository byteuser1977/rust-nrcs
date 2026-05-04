//! Account Repository 集成测试
//!
//! 测试 AccountRepository 的全量 CRUD 操作和扩展方法。

use orm::repository::*;
use orm::models::*;
use sqlx::SqlitePool;
use sqlx::PgPool;

async fn setup() -> (SqlitePool, SqliteAccountRepository) {
    let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool failed");
    let schema_sql = include_str!("../../../migrations/sqlite/0.sql");
    for statement in schema_sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("--") {
            let _ = sqlx::query(trimmed).execute(&pool).await;
        }
    }
    let repo = SqliteAccountRepository::new(pool.clone());
    (pool, repo)
}

async fn setup_pg() -> (PgPool, PgAccountRepository) {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://nrcs_user:password@localhost:5432/nrcs_db".to_string());
    let pool = PgPool::connect(&database_url).await.expect("pg pool failed");
    cleanup_all_pg(&pool).await;
    let repo = PgAccountRepository::new(pool.clone());
    (pool, repo)
}

async fn cleanup_all_pg(pool: &PgPool) {
    let tables = [
        "account_guaranteed_balance", "account_ledger", "account_asset",
        "account_currency", "account_info", "account_lease",
        "account_property", "account_control_phasing",
        "alias", "alias_offer", "asset_transfer", "asset_property",
        "asset_history", "asset_delete", "asset_dividend",
        "ask_order", "bid_order", "trade",
        "purchase", "purchase_feedback", "goods",
        "currency_transfer", "currency_founder", "currency_mint",
        "shuffling_participant", "shuffling_data",
        "vote", "poll_result", "poll",
        "phasing_vote", "phasing_poll_result", "phasing_poll_voter",
        "phasing_poll_linked_transaction", "phasing_poll_hashed_secret", "phasing_poll",
        "tagged_data", "tagged_data_tag", "tagged_data_extend", "tagged_timestamp",
        "prunable_message", "referenced_transaction",
        "exchange_request", "hub", "contract_reference",
        "coin_order_fxt", "coin_trade_fxt",
        "public_key", "transaction", "block",
        "account",
    ];
    for table in &tables {
        let _ = sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table)).execute(pool).await;
    }
}

fn make_account(id: i64, balance: i64, height: i32) -> AccountModel {
    AccountModel {
        db_id: 0,
        id,
        balance,
        unconfirmed_balance: balance,
        forged_balance: 0,
        active_lessee_id: None,
        has_control_phasing: false,
        height,
        latest: true,
    }
}

#[tokio::test]
async fn test_insert_and_find_by_account_id() {
    let (_pool, repo) = setup().await;
    let account = make_account(12345, 1000 * 100_000_000, 0);
    repo.insert(&account).await.expect("insert failed");

    let found = repo.find_by_account_id(12345).await.expect("find failed");
    assert!(found.is_some());
    let a = found.unwrap();
    assert_eq!(a.id, 12345);
    assert_eq!(a.balance, 1000 * 100_000_000);
}

#[tokio::test]
async fn test_find_by_account_id_not_found() {
    let (_pool, repo) = setup().await;
    let found = repo.find_by_account_id(99999).await.expect("find failed");
    assert!(found.is_none());
}

#[tokio::test]
async fn test_count() {
    let (_pool, repo) = setup().await;
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_account(1, 100, 0)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
}

#[tokio::test]
async fn test_update_balance() {
    let (_pool, repo) = setup().await;
    repo.insert(&make_account(12345, 1000, 0)).await.expect("insert failed");
    repo.update_balance(12345, 2000, 1500, 1).await.expect("update_balance failed");

    let found = repo.find_by_account_id(12345).await.expect("find failed").unwrap();
    assert_eq!(found.balance, 2000);
    assert_eq!(found.unconfirmed_balance, 1500);
}

#[tokio::test]
async fn test_find_all() {
    let (_pool, repo) = setup().await;
    repo.insert(&make_account(1, 100, 0)).await.expect("insert failed");
    repo.insert(&make_account(2, 200, 0)).await.expect("insert failed");

    let all = repo.find_all(None, None).await.expect("find_all failed");
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn test_multiple_accounts() {
    let (_pool, repo) = setup().await;
    for i in 1..=10 {
        repo.insert(&make_account(i, i * 100, 0)).await.expect("insert failed");
    }
    assert_eq!(repo.count().await.expect("count failed"), 10);

    let acc5 = repo.find_by_account_id(5).await.expect("find failed").unwrap();
    assert_eq!(acc5.balance, 500);
}

// ==================== PostgreSQL Tests ====================

#[tokio::test]
#[ignore]
async fn pg_test_insert_and_find_by_account_id() {
    let (_pool, repo) = setup_pg().await;
    let account = make_account(12345, 1000 * 100_000_000, 0);
    repo.insert(&account).await.expect("insert failed");

    let found = repo.find_by_account_id(12345).await.expect("find failed");
    assert!(found.is_some());
    let a = found.unwrap();
    assert_eq!(a.id, 12345);
    assert_eq!(a.balance, 1000 * 100_000_000);
}

#[tokio::test]
#[ignore]
async fn pg_test_count() {
    let (_pool, repo) = setup_pg().await;
    assert_eq!(repo.count().await.expect("count failed"), 0);
    repo.insert(&make_account(1, 100, 0)).await.expect("insert failed");
    assert_eq!(repo.count().await.expect("count failed"), 1);
}

#[tokio::test]
#[ignore]
async fn pg_test_update_balance() {
    let (_pool, repo) = setup_pg().await;
    repo.insert(&make_account(12345, 1000, 0)).await.expect("insert failed");
    repo.update_balance(12345, 2000, 1500, 1).await.expect("update_balance failed");

    let found = repo.find_by_account_id(12345).await.expect("find failed").unwrap();
    assert_eq!(found.balance, 2000);
    assert_eq!(found.unconfirmed_balance, 1500);
}

#[tokio::test]
#[ignore]
async fn pg_test_find_all() {
    let (_pool, repo) = setup_pg().await;
    repo.insert(&make_account(1, 100, 0)).await.expect("insert failed");
    repo.insert(&make_account(2, 200, 0)).await.expect("insert failed");

    let all = repo.find_all(None, None).await.expect("find_all failed");
    assert_eq!(all.len(), 2);
}
