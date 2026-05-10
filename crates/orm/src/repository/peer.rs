//! Peer Repository trait 和 SQLite 实现
//!
//! 对应 Java NRCS: PeerDb.java
//! 提供 PEER 表的 CRUD 操作

use async_trait::async_trait;
use crate::models::misc::PeerModel;
use crate::repository::{RepositoryError, RepositoryResult};
use sqlx::SqlitePool;

/// Peer 数据库操作 trait
#[async_trait]
pub trait PeerRepository: Send + Sync {
    /// 插入或更新节点（UPSERT）
    async fn upsert(&self, peer: &PeerModel) -> RepositoryResult<()>;

    /// 根据地址查找节点
    async fn find_by_address(&self, address: &str) -> RepositoryResult<Option<PeerModel>>;

    /// 获取所有节点
    async fn find_all(&self, limit: Option<i64>) -> RepositoryResult<Vec<PeerModel>>;

    /// 删除指定节点
    async fn delete_by_address(&self, address: &str) -> RepositoryResult<()>;

    /// 获取节点总数
    async fn count(&self) -> RepositoryResult<i64>;

    /// 清理过期黑名单记录（删除 last_updated < threshold 的记录）
    async fn cleanup_old_peers(&self, threshold: i64) -> RepositoryResult<u64>;
}

/// SQLite PeerRepository 实现
pub struct SqlitePeerRepository {
    pool: SqlitePool,
}

impl SqlitePeerRepository {
    /// 创建新的实例
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PeerRepository for SqlitePeerRepository {
    /// 插入或更新节点（对应 Java: PeerDb.savePeers()）
    async fn upsert(&self, peer: &PeerModel) -> RepositoryResult<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO peer (address, last_updated, services) VALUES (?, ?, ?)"
        )
        .bind(&peer.address)
        .bind(peer.last_updated)
        .bind(peer.services)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        Ok(())
    }

    /// 根据地址查找节点（对应 Java: PeerDb.loadPeers() 中的单条查询）
    async fn find_by_address(&self, address: &str) -> RepositoryResult<Option<PeerModel>> {
        let result = sqlx::query_as::<_, PeerModel>(
            "SELECT * FROM peer WHERE address = ?"
        )
        .bind(address)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        Ok(result)
    }

    /// 获取所有节点（对应 Java: PeerDb.loadPeers()）
    async fn find_all(&self, limit: Option<i64>) -> RepositoryResult<Vec<PeerModel>> {
        let limit = limit.unwrap_or(1000);

        let peers = sqlx::query_as::<_, PeerModel>(
            "SELECT * FROM peer ORDER BY last_updated DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        Ok(peers)
    }

    /// 删除指定节点
    async fn delete_by_address(&self, address: &str) -> RepositoryResult<()> {
        sqlx::query("DELETE FROM peer WHERE address = ?")
        .bind(address)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        Ok(())
    }

    /// 获取节点总数
    async fn count(&self) -> RepositoryResult<i64> {
        let (count,) = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM peer"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        Ok(count)
    }

    /// 清理过期节点（对应 Java: 清理旧节点逻辑）
    async fn cleanup_old_peers(&self, threshold: i64) -> RepositoryResult<u64> {
        let result = sqlx::query(
            "DELETE FROM peer WHERE last_updated < ? OR last_updated IS NULL"
        )
        .bind(threshold)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::DbError)?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_model_creation() {
        let model = PeerModel {
            address: "127.0.0.1:17974".to_string(),
            last_updated: Some(1701144000),
            services: Some(1),
        };
        assert_eq!(model.address, "127.0.0.1:17974");
        assert_eq!(model.services, Some(1));
    }
}
