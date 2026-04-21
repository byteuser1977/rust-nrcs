//! API配置
//!
//! 对应 Java: nrcs-default.properties 中的API配置

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub cors_allowed_origins: Vec<String>,
    pub debug: bool,
    pub api_prefix: Option<String>,
    pub admin_password: Option<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            database_url: "postgres://nrcs:password@localhost:5432/nrcs_db".to_string(),
            cors_allowed_origins: vec!["*".to_string()],
            debug: false,
            api_prefix: None,
            admin_password: None,
        }
    }
}

impl ApiConfig {
    pub fn from_toml(value: &toml::Value) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut config = Self::default();
        
        if let Some(host) = value.get("host").and_then(|v| v.as_str()) {
            config.host = host.to_string();
        }
        if let Some(port) = value.get("port").and_then(|v| v.as_integer()) {
            config.port = port as u16;
        }
        if let Some(database_url) = value.get("database_url").and_then(|v| v.as_str()) {
            config.database_url = database_url.to_string();
        }
        if let Some(cors) = value.get("cors_allowed_origins").and_then(|v| v.as_array()) {
            config.cors_allowed_origins = cors
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }
        if let Some(debug) = value.get("debug").and_then(|v| v.as_bool()) {
            config.debug = debug;
        }
        if let Some(api_prefix) = value.get("api_prefix").and_then(|v| v.as_str()) {
            config.api_prefix = Some(api_prefix.to_string());
        }
        if let Some(admin_password) = value.get("admin_password").and_then(|v| v.as_str()) {
            config.admin_password = Some(admin_password.to_string());
        }
        
        Ok(config)
    }
    
    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_api_config() {
        let config = ApiConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert!(!config.debug);
    }
}
