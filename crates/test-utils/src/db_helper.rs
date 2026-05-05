//! 数据库测试辅助
//!
//! 参照 Java NRCS 的 AbstractBlockchainTest，提供 SQLite 内存数据库和 PostgreSQL 测试数据库创建和 Schema 初始化。
//! SQLite 使用内存数据库，PostgreSQL 连接真实数据库实例。

use sqlx::SqlitePool;
use sqlx::PgPool;
use orm::repository::*;
use orm::models::*;

const DEFAULT_PG_URL: &str = "postgres://nrcs_user:password@localhost:5432/nrcs_db";

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

pub struct PgTestRepositories {
    pub pool: PgPool,
    pub block_repo: PgBlockRepository,
    pub account_repo: PgAccountRepository,
    pub transaction_repo: PgTransactionRepository,
    pub account_ledger_repo: PgAccountLedgerRepository,
    pub account_guaranteed_balance_repo: PgAccountGuaranteedBalanceRepository,
    pub account_asset_repo: PgAccountAssetRepository,
    pub asset_repo: PgAssetRepository,
    pub alias_repo: PgAliasRepository,
    pub account_info_repo: PgAccountInfoRepository,
    pub account_lease_repo: PgAccountLeaseRepository,
    pub account_property_repo: PgAccountPropertyRepository,
    pub account_control_phasing_repo: PgAccountControlPhasingRepository,
    pub public_key_repo: PgPublicKeyRepository,
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

pub async fn setup_test_pg_db() -> PgTestRepositories {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_PG_URL.to_string());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    cleanup_pg_tables(&pool).await;

    let schema_sql = include_str!("../../../migrations/postgres/0.sql");
    for statement in schema_sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("--") && !trimmed.starts_with("/*") {
            if let Err(e) = sqlx::query(trimmed).execute(&pool).await {
                if !e.to_string().contains("already exists") {
                    eprintln!("PG schema warning: {} - {}", trimmed.chars().take(80).collect::<String>(), e);
                }
            }
        }
    }

    PgTestRepositories {
        block_repo: PgBlockRepository::new(pool.clone()),
        account_repo: PgAccountRepository::new(pool.clone()),
        transaction_repo: PgTransactionRepository::new(pool.clone()),
        account_ledger_repo: PgAccountLedgerRepository::new(pool.clone()),
        account_guaranteed_balance_repo: PgAccountGuaranteedBalanceRepository::new(pool.clone()),
        account_asset_repo: PgAccountAssetRepository::new(pool.clone()),
        asset_repo: PgAssetRepository::new(pool.clone()),
        alias_repo: PgAliasRepository::new(pool.clone()),
        account_info_repo: PgAccountInfoRepository::new(pool.clone()),
        account_lease_repo: PgAccountLeaseRepository::new(pool.clone()),
        account_property_repo: PgAccountPropertyRepository::new(pool.clone()),
        account_control_phasing_repo: PgAccountControlPhasingRepository::new(pool.clone()),
        public_key_repo: PgPublicKeyRepository::new(pool.clone()),
        pool,
    }
}

pub async fn setup_core_pg_db() -> (PgPool, PgBlockRepository, PgAccountRepository, PgTransactionRepository) {
    let repos = setup_test_pg_db().await;
    let pool = repos.pool.clone();
    (pool, repos.block_repo, repos.account_repo, repos.transaction_repo)
}

async fn cleanup_pg_tables(pool: &PgPool) {
    let tables = [
        "BLOCK", "TRANSACTION", "ACCOUNT", "PUBLIC_KEY",
        "ACCOUNT_ASSET", "ACCOUNT_LEDGER", "ACCOUNT_GUARANTEED_BALANCE",
        "ALIAS", "ALIAS_OFFER", "ASSET", "ASSET_TRANSFER",
        "ACCOUNT_INFO", "ACCOUNT_LEASE", "ACCOUNT_PROPERTY",
        "ACCOUNT_CONTROL_PHASING", "ACCOUNT_CURRENCY",
        "ASK_ORDER", "BID_ORDER", "TRADE",
        "POLL", "VOTE", "POLL_RESULT",
        "TAGGED_DATA", "TAGGED_DATA_TAG", "TAGGED_DATA_EXTEND", "TAGGED_TIMESTAMP",
        "PURCHASE", "PURCHASE_FEEDBACK", "GOODS",
        "CURRENCY", "CURRENCY_FOUNDER", "CURRENCY_MINT", "CURRENCY_TRANSFER",
        "SHUFFLING", "SHUFFLING_DATA", "SHUFFLING_PARTICIPANT",
        "PHASING_POLL", "PHASING_VOTE", "PHASING_POLL_RESULT",
        "PHASING_POLL_VOTER", "PHASING_POLL_LINKED_TRANSACTION", "PHASING_POLL_HASHED_SECRET",
        "ACCOUNT_CONTROL_PHASING", "CONTRACT_REFERENCE",
        "ASSET_PROPERTY", "ASSET_HISTORY", "ASSET_DELETE", "ASSET_DIVIDEND",
        "EXCHANGE_REQUEST", "HUB",
        "COIN_ORDER_FXT", "COIN_TRADE_FXT",
        "PRUNABLE_MESSAGE", "REFERENCED_TRANSACTION",
    ];

    for table in &tables {
        let sql = format!("TRUNCATE TABLE \"{}\" CASCADE", table);
        let _ = sqlx::query(&sql).execute(pool).await;
    }
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

pub async fn insert_test_account_pg(
    account_repo: &PgAccountRepository,
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

pub async fn insert_test_block_pg(
    block_repo: &PgBlockRepository,
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
