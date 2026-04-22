//! 密码过滤模块
//!
//! 对应 Java: PasswordFilteringContentTransformer, PasswordFinder
//!
//! 检测请求中的敏感参数，防止敏感数据被代理到其他节点

use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PasswordFilterError {
    #[error("Sensitive parameter detected: {0}")]
    SensitiveParameterDetected(String),
    
    #[error("Invalid UTF-8 in request body")]
    InvalidUtf8,
}

pub type PasswordFilterResult<T> = std::result::Result<T, PasswordFilterError>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SensitiveParameter {
    SecretPhrase,
    AdminPassword,
    SharedKey,
}

impl SensitiveParameter {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SecretPhrase => "secretPhrase",
            Self::AdminPassword => "adminPassword",
            Self::SharedKey => "sharedKey",
        }
    }
    
    pub fn all() -> &'static [SensitiveParameter] {
        &[
            SensitiveParameter::SecretPhrase,
            SensitiveParameter::AdminPassword,
            SensitiveParameter::SharedKey,
        ]
    }
}

pub struct PasswordFilter {
    sensitive_patterns: Vec<String>,
}

impl Default for PasswordFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordFilter {
    pub fn new() -> Self {
        Self {
            sensitive_patterns: SensitiveParameter::all()
                .iter()
                .map(|p| format!("{}=", p.as_str()))
                .collect(),
        }
    }
    
    pub fn check_query_params(&self, query: &str) -> PasswordFilterResult<HashSet<SensitiveParameter>> {
        let mut detected = HashSet::new();
        
        for param in SensitiveParameter::all() {
            let pattern = format!("{}=", param.as_str());
            if query.contains(&pattern) {
                detected.insert(param.clone());
            }
        }
        
        if !detected.is_empty() {
            let params: Vec<&str> = detected.iter().map(|p| p.as_str()).collect();
            return Err(PasswordFilterError::SensitiveParameterDetected(
                params.join(", ")
            ));
        }
        
        Ok(detected)
    }
    
    pub fn check_body(&self, body: &[u8]) -> PasswordFilterResult<HashSet<SensitiveParameter>> {
        let body_str = std::str::from_utf8(body)
            .map_err(|_| PasswordFilterError::InvalidUtf8)?;
        
        let mut detected = HashSet::new();
        
        for param in SensitiveParameter::all() {
            let pattern = format!("{}=", param.as_str());
            if body_str.contains(&pattern) {
                detected.insert(param.clone());
            }
        }
        
        if !detected.is_empty() {
            let params: Vec<&str> = detected.iter().map(|p| p.as_str()).collect();
            return Err(PasswordFilterError::SensitiveParameterDetected(
                params.join(", ")
            ));
        }
        
        Ok(detected)
    }
    
    pub fn check_json_body(&self, body: &[u8]) -> PasswordFilterResult<HashSet<SensitiveParameter>> {
        let body_str = std::str::from_utf8(body)
            .map_err(|_| PasswordFilterError::InvalidUtf8)?;
        
        let mut detected = HashSet::new();
        
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(body_str) {
            if let Some(obj) = json.as_object() {
                for param in SensitiveParameter::all() {
                    if obj.contains_key(param.as_str()) {
                        detected.insert(param.clone());
                    }
                }
            }
        } else {
            return self.check_body(body);
        }
        
        if !detected.is_empty() {
            let params: Vec<&str> = detected.iter().map(|p| p.as_str()).collect();
            return Err(PasswordFilterError::SensitiveParameterDetected(
                params.join(", ")
            ));
        }
        
        Ok(detected)
    }
    
    pub fn is_safe_to_proxy(&self, query: Option<&str>, body: Option<&[u8]>) -> bool {
        if let Some(q) = query {
            if self.check_query_params(q).is_err() {
                return false;
            }
        }
        
        if let Some(b) = body {
            if self.check_json_body(b).is_err() {
                return false;
            }
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_query_params_safe() {
        let filter = PasswordFilter::new();
        let query = "requestType=getAccount&account=12345";
        assert!(filter.check_query_params(query).is_ok());
    }

    #[test]
    fn test_check_query_params_secret_phrase() {
        let filter = PasswordFilter::new();
        let query = "requestType=sendMoney&secretPhrase=mypassword";
        let result = filter.check_query_params(query);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_query_params_admin_password() {
        let filter = PasswordFilter::new();
        let query = "requestType=shutdown&adminPassword=admin123";
        let result = filter.check_query_params(query);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_body_safe() {
        let filter = PasswordFilter::new();
        let body = b"amount=1000&recipient=12345";
        assert!(filter.check_body(body).is_ok());
    }

    #[test]
    fn test_check_body_secret_phrase() {
        let filter = PasswordFilter::new();
        let body = b"secretPhrase=mypassword&amount=1000";
        let result = filter.check_body(body);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_json_body_safe() {
        let filter = PasswordFilter::new();
        let body = br#"{"requestType":"getAccount","account":"12345"}"#;
        assert!(filter.check_json_body(body).is_ok());
    }

    #[test]
    fn test_check_json_body_secret_phrase() {
        let filter = PasswordFilter::new();
        let body = br#"{"requestType":"sendMoney","secretPhrase":"mypassword"}"#;
        let result = filter.check_json_body(body);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_safe_to_proxy() {
        let filter = PasswordFilter::new();
        
        assert!(filter.is_safe_to_proxy(Some("requestType=getAccount"), None));
        
        assert!(!filter.is_safe_to_proxy(Some("secretPhrase=test"), None));
        
        assert!(filter.is_safe_to_proxy(None, Some(b"amount=100")));
        
        assert!(!filter.is_safe_to_proxy(None, Some(b"secretPhrase=test")));
    }
}
