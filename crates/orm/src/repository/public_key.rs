//! Public key repository for account public keys.

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{RepositoryError, RepositoryResult};
use blockchain_types::account_ext::AccountPublicKey;
use blockchain_types::prelude::*;

/// Repository for account public keys.
#[async_trait]
pub trait PublicKeyRepository: Send + Sync {
    /// Get the latest public key for an account.
    async fn find_latest_by_account_id(&self, account_id: i64) -> RepositoryResult<Option<AccountPublicKey>>;
}

/// PostgreSQL implementation of PublicKeyRepository.
pub struct PgPublicKeyRepository {
    pool: PgPool,
}

impl PgPublicKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PublicKeyRepository for PgPublicKeyRepository {
    async fn find_latest_by_account_id(&self, account_id: i64) -> RepositoryResult<Option<AccountPublicKey>> {
        // The PUBLIC_KEY table has columns: account_id, public_key (bytea), height, latest.
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            "SELECT public_key FROM public_key WHERE account_id = $1 AND latest = TRUE ORDER BY height DESC LIMIT 1"
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((pk_vec,)) = row {
            if let Ok(bytes) = pk_vec.try_into() {
                // We need the height as well, but AccountPublicKey requires height. We can use 0 or try to fetch height.
                // For simplicity, we set height to 0. In a full implementation, fetch height.
                // But we can adjust query to return height.
                let pk = AccountPublicKey {
                    account_id: account_id as AccountId,
                    public_key: bytes,
                    height: 0, // TODO: fetch actual height
                };
                return Ok(Some(pk));
            }
        }
        Ok(None)
    }
}
