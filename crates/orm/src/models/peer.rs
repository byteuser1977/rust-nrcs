//! Peer 数据库模型（对应 PEER 表）
//!
//! 对应 Java NRCS: PeerDb.java
//! 数据库表定义（3 字段）:
//! - ADDRESS varchar(220) PRIMARY KEY
//! - LAST_UPDATED integer
//! - SERVICES bigint

use serde::{Deserialize, Serialize};

/// Peer 数据库模型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerModel {
    /// 节点地址（主键）
    pub address: String,
    /// 最后更新时间（Unix 时间戳，秒）
    pub last_updated: Option<i64>,
    /// 服务标志位
    pub services: Option<i64>,
}

impl PeerModel {
    /// 创建新的 PeerModel
    pub fn new(address: String) -> Self {
        Self {
            address,
            last_updated: None,
            services: Some(0),
        }
    }

    /// 带完整参数的构造函数
    pub fn with_details(address: String, last_updated: i64, services: i64) -> Self {
        Self {
            address,
            last_updated: Some(last_updated),
            services: Some(services),
        }
    }
}

impl Default for PeerModel {
    fn default() -> Self {
        Self {
            address: String::new(),
            last_updated: None,
            services: Some(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_model_new() {
        let model = PeerModel::new("127.0.0.1:16974".to_string());
        assert_eq!(model.address, "127.0.0.1:16974");
        assert!(model.last_updated.is_none());
        assert_eq!(model.services, Some(0));
    }

    #[test]
    fn test_peer_model_with_details() {
        let model = PeerModel::with_details("127.0.0.1:17974".to_string(), 1701144000, 1);
        assert_eq!(model.address, "127.0.0.1:17974");
        assert_eq!(model.last_updated, Some(1701144000));
        assert_eq!(model.services, Some(1));
    }
}
