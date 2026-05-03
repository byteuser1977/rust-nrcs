//! Account Repository 集成测试
//!
//! 测试 AccountRepository 的全量 CRUD 操作和扩展方法。

use orm::repository::*;
use orm::models::*;
use sqlx::SqlitePool;

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
