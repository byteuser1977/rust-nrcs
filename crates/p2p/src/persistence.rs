//! 节点持久化（数据库存储）
//!
//! 对应 NRCS Java: Peers.updateSavedPeers() / Peers.loadSavedPeers()
//!
//! 功能:
//! - 保存节点到 SQLite 数据库
//! - 从数据库加载已知节点
//! - 定期更新节点状态
//! - 支持离线后恢复连接
//!
//! 集成 ORM 模块实现实际的数据库操作

use crate::config::P2PConfig;
use crate::peer::{Peer, Peers};
use orm::models::misc::PeerModel as DbPeerModel;
use orm::repository::PeerRepository;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, info};

/// 节点持久化错误
#[derive(Debug)]
pub enum PersistenceError {
    /// 数据库错误
    Database(String),
    /// 序列化错误
    Serialization(String),
    /// 验证错误
    Validation(String),
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(e) => write!(f, "Database error: {}", e),
            Self::Serialization(e) => write!(f, "Serialization error: {}", e),
            Self::Validation(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for PersistenceError {}

/// 节点持久化管理器（带数据库支持）
///
/// 对应 NRCS Java: Peers.savePeers / Peers.loadPeers
pub struct PeerPersistence {
    /// 是否启用持久化
    enabled: bool,
    /// 配置引用（保留用于未来扩展）
    #[allow(dead_code)]
    config: Arc<P2PConfig>,
    /// 数据库仓库（可选，如果未启用则为 None）
    repository: Option<Arc<dyn PeerRepository + Send + Sync>>,
}

impl PeerPersistence {
    /// 创建新的持久化管理器（无数据库）
    pub fn new(config: Arc<P2PConfig>) -> Self {
        Self {
            enabled: config.use_peers_db && config.save_peers,
            config,
            repository: None,
        }
    }

    /// 创建新的持久化管理器（带数据库支持）
    pub fn with_repository(
        config: Arc<P2PConfig>,
        repository: Arc<dyn PeerRepository + Send + Sync>,
    ) -> Self {
        Self {
            enabled: config.use_peers_db && config.save_peers,
            config,
            repository: Some(repository),
        }
    }

    /// 检查是否启用持久化
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 将 Peer 转换为数据库模型
    ///
    /// 只保存 PEER 表需要的 3 个字段：ADDRESS, LAST_UPDATED, SERVICES
    pub fn peer_to_db_model(peer: &Peer) -> Result<DbPeerModel, PersistenceError> {
        Ok(DbPeerModel {
            address: peer.address.to_string(),
            last_updated: if peer.last_updated > 0 { Some(peer.last_updated as i32) } else { None },
            services: Some(peer.services as i64),
        })
    }

    /// 从数据库模型转换为 Peer
    pub fn db_model_to_peer(model: &DbPeerModel) -> Result<Peer, PersistenceError> {
        let addr: SocketAddr = model.address.parse()
            .map_err(|e| PersistenceError::Validation(format!("Invalid address: {}", e)))?;

        let mut peer = Peer::new(addr, false);

        // 恢复服务标志
        if let Some(services) = model.services {
            peer.services = services as u64;
        }

        // 恢复最后更新时间（i32 -> i64）
        if let Some(last_updated) = model.last_updated {
            peer.last_updated = last_updated as i64;
        }

        Ok(peer)
    }

    /// 保存单个节点到数据库
    ///
    /// 对应 Java: updateSavedPeers()
    pub async fn save_peer(&self, peer: &Peer) -> Result<(), PersistenceError> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(ref repo) = self.repository {
            let db_model = Self::peer_to_db_model(peer)?;
            repo.upsert(&db_model).await
                .map_err(|e| PersistenceError::Database(e.to_string()))?;
            
            debug!("[Persistence] Saved peer: {}", peer.address);
        } else {
            debug!("[Persistence] No repository configured, skipping save");
        }

        Ok(())
    }

    /// 批量保存节点
    pub async fn save_peers(&self, peers: &[Peer]) -> Result<(), PersistenceError> {
        if !self.enabled || peers.is_empty() || self.repository.is_none() {
            return Ok(());
        }

        let repo = self.repository.as_ref().unwrap();
        let mut saved = 0usize;

        for peer in peers {
            let db_model = Self::peer_to_db_model(peer)?;
            repo.upsert(&db_model).await
                .map_err(|e| PersistenceError::Database(e.to_string()))?;
            saved += 1;
        }

        info!("[Persistence] Batch saved {} peers", saved);
        Ok(())
    }

    /// 从数据库加载所有已知节点并注册到 Peers
    ///
    /// 对应 Java: loadSavedPeers()
    pub async fn load_all_peers(&self, peers: &Arc<Peers>) -> Result<usize, PersistenceError> {
        if !self.enabled {
            return Ok(0);
        }

        if let Some(ref repo) = self.repository {
            let db_models = repo.find_all(None).await
                .map_err(|e| PersistenceError::Database(e.to_string()))?;
            
            let mut loaded = 0usize;
            
            for model in db_models {
                match Self::db_model_to_peer(&model) {
                    Ok(peer) => {
                        // 注册到内存中的 Peers 管理
                        if !peers.contains_peer(&peer.address).await {
                            peers.register_peer(peer).await;
                            loaded += 1;
                        }
                    }
                    Err(e) => {
                        debug!("[Persistence] Failed to restore peer {}: {}", model.address, e);
                    }
                }
            }

            info!("[Persistence] Loaded {} peers from database", loaded);
            Ok(loaded)
        } else {
            debug!("[Persistence] No repository configured, skipping load");
            Ok(0)
        }
    }

    /// 删除指定节点
    pub async fn delete_peer(&self, address: &str) -> Result<(), PersistenceError> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(ref repo) = self.repository {
            repo.delete_by_address(address).await
                .map_err(|e| PersistenceError::Database(e.to_string()))?;
            
            debug!("[Persistence] Deleted peer: {}", address);
        }

        Ok(())
    }

    /// 清理过期节点记录
    ///
    /// 定期调用以清理长时间未更新的节点
    pub async fn cleanup_old_peers(&self, threshold: i64) -> Result<u64, PersistenceError> {
        if !self.enabled || self.repository.is_none() {
            return Ok(0);
        }

        let repo = self.repository.as_ref().unwrap();
        let deleted = repo.cleanup_old_peers(threshold).await
            .map_err(|e| PersistenceError::Database(e.to_string()))?;

        if deleted > 0 {
            info!("[Persistence] Cleaned up {} old peers", deleted);
        }

        Ok(deleted)
    }

    /// 获取数据库中存储的节点总数
    pub async fn get_peer_count(&self) -> Result<i64, PersistenceError> {
        if !self.enabled || self.repository.is_none() {
            return Ok(0);
        }

        let repo = self.repository.as_ref().unwrap();
        repo.count().await
            .map_err(|e| PersistenceError::Database(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_to_db_model_conversion() {
        let addr: SocketAddr = "127.0.0.1:16974".parse().unwrap();
        let mut peer = Peer::new(addr, false);
        peer.services = 1; // API 服务
        peer.last_updated = 1701144000;

        let db_model = PeerPersistence::peer_to_db_model(&peer).unwrap();
        assert_eq!(db_model.address, "127.0.0.1:16974");
        assert_eq!(db_model.last_updated, Some(1701144000));
        assert_eq!(db_model.services, Some(1));
    }

    #[test]
    fn test_db_model_to_peer_conversion() {
        let db_model = DbPeerModel {
            address: "127.0.0.1:17974".to_string(),
            last_updated: Some(1701144000),
            services: Some(1),
        };

        let peer = PeerPersistence::db_model_to_peer(&db_model).unwrap();
        assert_eq!(peer.address.to_string(), "127.0.0.1:17974");
        assert_eq!(peer.last_updated, 1701144000);
        assert_eq!(peer.services, 1);
    }

    #[test]
    fn test_persistence_disabled_when_config_off() {
        let mut config = P2PConfig::default();
        config.use_peers_db = false;

        let persistence = PeerPersistence::new(Arc::new(config));
        assert!(!persistence.is_enabled());
    }
}
