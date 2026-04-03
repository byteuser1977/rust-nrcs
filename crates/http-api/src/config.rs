//! API 服务器配置

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// 监听地址
    pub host: String,
    /// 监听端口
    pub port: u16,
    /// 数据库连接字符串
    pub database_url: String,
    /// CORS 允许的源
    pub cors_allowed_origins: Vec<String>,
    /// 是否启用调试日志
    pub debug: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            database_url: "postgres://user:password@localhost/nrcs_db".to_string(),
            cors_allowed_origins: vec!["*".to_string()],
            debug: false,
        }
    }
}

impl ApiConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> anyhow::Result<Self> {
        let config: Self = figment::Figment::new()
            .merge(figment::providers::Env::prefixed("NRCS_"))
            .merge(figment::providers::Toml::file("config/api.toml"))
            .merge(figment::providers::Json::file("config/api.json"))
            .extract()?;
        Ok(config)
    }

    /// 获取绑定的 SocketAddr
    pub fn addr(&self) -> SocketAddr {
        SocketAddr::from((self.host.parse().unwrap_or([127, 0, 0, 1].into()), self.port))
    }
}