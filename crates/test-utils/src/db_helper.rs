//! 内存数据库辅助
//!
//! 参照 Java NRCS 的 AbstractBlockchainTest，提供 SQLite 内存数据库创建和 Schema 初始化。
//! 使用项目中的 migration SQL 文件初始化全量 Schema。

use sqlx::SqlitePool;
use orm::repository::*;
use orm::models::*;

pub struct TestRepositories {
    pub pool: SqlitePool,
    pub block_repo: SqliteBlockRepository,
    pub account_repo: SqliteAccountRepository,
    pub transaction_repo: SqliteTransactionRepository,
    pub account_ledger_repo: SqliteAccountLedgerRepository,
    pub account_guaranteed_balance_repo: SqliteAccountGuaranteedBalanceRepository,
    pub account_asset_repo: SqliteAccountAssetRepository,
    pub asset_repo: SqliteAssetRepository,
    pub alias_repo: SqliteAliasRepository,
    pub account_info_repo: SqliteAccountInfoRepository,
    pub account_lease_repo: SqliteAccountLeaseRepository,
    pub account_property_repo: SqliteAccountPropertyRepository,
    pub account_control_phasing_repo: SqliteAccountControlPhasingRepository,
    pub public_key_repo: SqlitePublicKeyRepository,
}

pub async fn setup_test_db() -> TestRepositories {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create SQLite memory pool");

    let schema_sql = include_str!("../../../migrations/sqlite/0.sql");
    for statement in schema_sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("--") {
            if let Err(e) = sqlx::query(trimmed).execute(&pool).await {
                if !e.to_string().contains("already exists") {
                    panic!("Failed to execute schema statement: {}\nError: {}", trimmed.chars().take(100).collect::<String>(), e);
                }
            }
        }
    }

    TestRepositories {
        block_repo: SqliteBlockRepository::new(pool.clone()),
        account_repo: SqliteAccountRepository::new(pool.clone()),
        transaction_repo: SqliteTransactionRepository::new(pool.clone()),
        account_ledger_repo: SqliteAccountLedgerRepository::new(pool.clone()),
        account_guaranteed_balance_repo: SqliteAccountGuaranteedBalanceRepository::new(pool.clone()),
        account_asset_repo: SqliteAccountAssetRepository::new(pool.clone()),
        asset_repo: SqliteAssetRepository::new(pool.clone()),
        alias_repo: SqliteAliasRepository::new(pool.clone()),
        account_info_repo: SqliteAccountInfoRepository::new(pool.clone()),
        account_lease_repo: SqliteAccountLeaseRepository::new(pool.clone()),
        account_property_repo: SqliteAccountPropertyRepository::new(pool.clone()),
        account_control_phasing_repo: SqliteAccountControlPhasingRepository::new(pool.clone()),
        public_key_repo: SqlitePublicKeyRepository::new(pool.clone()),
        pool,
    }
}

pub async fn setup_core_db() -> (SqlitePool, SqliteBlockRepository, SqliteAccountRepository, SqliteTransactionRepository) {
    let repos = setup_test_db().await;
    let pool = repos.pool.clone();
    (pool, repos.block_repo, repos.account_repo, repos.transaction_repo)
}

pub async fn insert_test_account(
    account_repo: &SqliteAccountRepository,
    account_id: i64,
    balance: i64,
    height: i32,
) -> RepositoryResult<()> {
    let model = AccountModel {
        db_id: 0,
        id: account_id,
        balance,
        unconfirmed_balance: balance,
        forged_balance: 0,
        active_lessee_id: None,
        has_control_phasing: false,
        height,
        latest: true,
    };
    account_repo.insert(&model).await
}

pub async fn insert_test_block(
    block_repo: &SqliteBlockRepository,
    id: i64,
    height: i32,
    timestamp: i32,
    generator_id: i64,
) -> RepositoryResult<()> {
    let model = BlockModel {
        db_id: 0,
        id,
        version: 3,
        timestamp,
        previous_block_id: None,
        total_amount: 0,
        total_fee: 0,
        payload_length: 0,
        previous_block_hash: None,
        cumulative_difficulty: vec![0u8; 1],
        base_target: blockchain_types::constants::INITIAL_BASE_TARGET as i64,
        next_block_id: None,
        height,
        generation_signature: vec![0u8; 64],
        block_signature: vec![0u8; 64],
        payload_hash: vec![0u8; 32],
        generator_id,
    };
    block_repo.insert(&model).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_test_db() {
        let repos = setup_test_db().await;
        let count = repos.block_repo.count().await.expect("count failed");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_insert_test_account() {
        let repos = setup_test_db().await;
        insert_test_account(&repos.account_repo, 12345, 1000 * 100_000_000, 0)
            .await
            .expect("insert failed");
        let found = repos.account_repo.find_by_account_id(12345).await.expect("find failed");
        assert!(found.is_some());
        let acc = found.unwrap();
        assert_eq!(acc.balance, 1000 * 100_000_000);
    }

    #[tokio::test]
    async fn test_insert_test_block() {
        let repos = setup_test_db().await;
        insert_test_block(&repos.block_repo, 100, 0, 0, 12345)
            .await
            .expect("insert failed");
        let found = repos.block_repo.find_by_height(0).await.expect("find failed");
        assert!(found.is_some());
        let block = found.unwrap();
        assert_eq!(block.id, 100);
        assert_eq!(block.height, 0);
    }
}
