//! Database connection management and configuration
//!
//! Provides unified database pool creation for SQLite and PostgreSQL
//! based on configuration file settings.
//!
//! Also provides transaction management through `TransactionManager`,
//! allowing upper-layer modules to manage transactions without
//! directly depending on sqlx.

use sqlx::{AnyPool, Pool, any::AnyConnectOptions};
use tracing::{info, warn};

pub type DbPool = sqlx::AnyPool;
pub type DbTransaction<'a> = sqlx::Transaction<'a, sqlx::Any>;

#[async_trait::async_trait]
pub trait TransactionManager: Send + Sync {
    async fn begin(&self) -> anyhow::Result<DbTransaction<'_>>;
}

pub struct PoolTransactionManager {
    pool: DbPool,
}

impl PoolTransactionManager {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &DbPool {
        &self.pool
    }
}

#[async_trait::async_trait]
impl TransactionManager for PoolTransactionManager {
    async fn begin(&self) -> anyhow::Result<DbTransaction<'_>> {
        self.pool.begin().await
            .map_err(|e| anyhow::anyhow!("Failed to begin transaction: {}", e))
    }
}

/// Database type enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseType {
    SQLite,
    PostgreSQL,
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::SQLite => write!(f, "sqlite"),
            DatabaseType::PostgreSQL => write!(f, "postgresql"),
        }
    }
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub db_type: DatabaseType,
    pub max_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite://./nrcs.db?mode=rwc".to_string(),
            db_type: DatabaseType::SQLite,
            max_connections: 10,
        }
    }
}

impl DatabaseConfig {
    /// Parse database URL to determine type
    pub fn from_url(url: &str) -> Self {
        let db_type = if url.starts_with("sqlite:") || url.starts_with("sqlite://") {
            DatabaseType::SQLite
        } else if url.starts_with("postgres:") || url.starts_with("postgresql:") {
            DatabaseType::PostgreSQL
        } else {
            warn!("Unknown database protocol in URL: {}, defaulting to SQLite", url);
            DatabaseType::SQLite
        };

        Self {
            url: url.to_string(),
            db_type,
            max_connections: 10,
        }
    }

    /// Create with custom max connections
    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }
}

/// Create database pool based on configuration
///
/// # Example
///
/// ```rust,no_run
/// use orm::connection::{DatabaseConfig, create_pool};
///
/// # async fn example() -> anyhow::Result<()> {
/// let config = DatabaseConfig::from_url("sqlite://./test.db?mode=rwc");
/// let pool = create_pool(&config).await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_pool(config: &DatabaseConfig) -> anyhow::Result<AnyPool> {
    info!("Creating {} database pool: {}", config.db_type, config.url);

    let options: AnyConnectOptions = config.url.parse()
        .map_err(|e| anyhow::anyhow!("Invalid database URL {}: {}", config.url, e))?;

    let pool = Pool::<sqlx::Any>::connect_with(options)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

    info!(
        "Database connection established successfully | type={} | url={} | max_connections={}",
        config.db_type,
        mask_url(&config.url),
        config.max_connections
    );
    Ok(pool)
}

/// Get database type from URL string (convenience function)
pub fn detect_database_type(url: &str) -> DatabaseType {
    if url.contains("sqlite") {
        DatabaseType::SQLite
    } else if url.contains("postgres") || url.contains("postgresql") {
        DatabaseType::PostgreSQL
    } else {
        DatabaseType::SQLite
    }
}

/// Mask sensitive information (username and password) in database URL
///
/// # Examples
///
/// ```
/// assert_eq!(
///     mask_url("postgres://user:pass@localhost:5432/db"),
///     "postgres://****:****@localhost:5432/db"
/// );
/// ```
pub fn mask_url(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(scheme_end) = url.find("://") {
            let scheme = &url[..scheme_end + 3];
            let host_part = &url[at_pos..];
            format!("{}****:****{}", scheme, host_part)
        } else {
            url.to_string()
        }
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_sqlite_from_url() {
        assert_eq!(
            detect_database_type("sqlite://./test.db?mode=rwc"),
            DatabaseType::SQLite
        );
    }

    #[test]
    fn test_detect_postgres_from_url() {
        assert_eq!(
            detect_database_type("postgres://user:pass@localhost:5432/db"),
            DatabaseType::PostgreSQL
        );
    }

    #[test]
    fn test_config_from_url() {
        let config = DatabaseConfig::from_url("sqlite:///tmp/nrcs.db");
        assert_eq!(config.db_type, DatabaseType::SQLite);
        assert_eq!(config.url, "sqlite:///tmp/nrcs.db");
    }

    #[test]
    fn test_mask_url_with_credentials() {
        assert_eq!(
            mask_url("postgres://user:pass@localhost:5432/db"),
            "postgres://****:****@localhost:5432/db"
        );
    }

    #[test]
    fn test_mask_url_without_credentials() {
        assert_eq!(
            mask_url("sqlite://./nrcs.db?mode=rwc"),
            "sqlite://./nrcs.db?mode=rwc"
        );
    }

    #[test]
    fn test_mask_url_complex_credentials() {
        assert_eq!(
            mask_url("postgresql://admin:secret123@db.example.com:5432/production"),
            "postgresql://****:****@db.example.com:5432/production"
        );
    }
}
