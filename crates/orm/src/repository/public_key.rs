use async_trait::async_trait;

use crate::RepositoryResult;
use blockchain_types::account_ext::AccountPublicKey;

#[async_trait]
pub trait PublicKeyRepository: Send + Sync {
    async fn find_latest_by_account_id(&self, account_id: i64) -> RepositoryResult<Option<AccountPublicKey>>;

    async fn insert(&self, pk: &AccountPublicKey) -> RepositoryResult<()>;
}
