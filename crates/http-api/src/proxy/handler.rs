//! 代理请求处理器
//!
//! 对应 Java: APIProxyServlet
//!
//! 处理代理请求的转发和响应

use std::sync::Arc;
use reqwest::Client;
use serde_json::Value;
use thiserror::Error;
use tracing::{debug, error, info};

use super::password_filter::PasswordFilter;
use super::api_proxy::{ApiProxy, ApiProxyError, ProxyPeer};

#[derive(Debug, Error)]
pub enum ProxyHandlerError {
    #[error("Password filter error: {0}")]
    PasswordFilter(String),
    
    #[error("Proxy error: {0}")]
    Proxy(#[from] ApiProxyError),
    
    #[error("HTTP request failed: {0}")]
    HttpFailed(String),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

pub type ProxyHandlerResult<T> = std::result::Result<T, ProxyHandlerError>;

#[derive(Debug, Clone)]
pub struct ProxyResponse {
    pub status: u16,
    pub body: String,
    pub from_peer: String,
}

pub struct ProxyRequestHandler {
    proxy: Arc<ApiProxy>,
    password_filter: PasswordFilter,
    http_client: Client,
}

impl ProxyRequestHandler {
    pub fn new(proxy: Arc<ApiProxy>) -> Self {
        Self {
            proxy,
            password_filter: PasswordFilter::new(),
            http_client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
    
    pub fn check_sensitive_params(&self, query: Option<&str>, body: Option<&[u8]>) -> ProxyHandlerResult<()> {
        if let Some(q) = query {
            self.password_filter.check_query_params(q)
                .map_err(|e| ProxyHandlerError::PasswordFilter(e.to_string()))?;
        }
        
        if let Some(b) = body {
            self.password_filter.check_json_body(b)
                .map_err(|e| ProxyHandlerError::PasswordFilter(e.to_string()))?;
        }
        
        Ok(())
    }
    
    pub async fn forward_request(
        &self,
        request_type: &str,
        query: Option<&str>,
        body: Option<&[u8]>,
        require_blockchain: bool,
        require_full_client: bool,
        is_downloading: bool,
    ) -> ProxyHandlerResult<ProxyResponse> {
        if !self.proxy.is_activated(is_downloading) {
            return Err(ProxyHandlerError::Proxy(ApiProxyError::NotForwardable(
                request_type.to_string()
            )));
        }
        
        if !self.proxy.is_forwardable(request_type, require_blockchain, require_full_client) {
            return Err(ProxyHandlerError::Proxy(ApiProxyError::NotForwardable(
                request_type.to_string()
            )));
        }
        
        self.check_sensitive_params(query, body)?;
        
        let peer = self.proxy.get_serving_peer(request_type).await?;
        let url = self.build_proxy_url(&peer, query);
        
        debug!("Forwarding request {} to {}", request_type, url);
        
        let response = self.send_request(&url, body).await?;
        
        if response.status < 200 || response.status >= 300 {
            self.proxy.blacklist_host(&peer.host).await;
            error!("Proxy request failed with status {}, blacklisting peer {}", response.status, peer.host);
        }
        
        Ok(response)
    }
    
    fn build_proxy_url(&self, peer: &ProxyPeer, query: Option<&str>) -> String {
        let base_url = peer.api_url();
        
        match query {
            Some(q) => format!("{}?{}", base_url, q),
            None => base_url,
        }
    }
    
    async fn send_request(&self, url: &str, body: Option<&[u8]>) -> ProxyHandlerResult<ProxyResponse> {
        let response = if let Some(body_data) = body {
            self.http_client
                .post(url)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(body_data.to_vec())
                .send()
                .await
        } else {
            self.http_client
                .get(url)
                .send()
                .await
        };
        
        match response {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let body = resp.text().await.unwrap_or_default();
                let from_peer = url.split('/').nth(2).unwrap_or("unknown").to_string();
                
                Ok(ProxyResponse {
                    status,
                    body,
                    from_peer,
                })
            }
            Err(e) => {
                error!("HTTP request failed: {}", e);
                Err(ProxyHandlerError::HttpFailed(e.to_string()))
            }
        }
    }
    
    #[allow(clippy::too_many_arguments)]
    pub async fn try_local_first_then_proxy(
        &self,
        request_type: &str,
        query: Option<&str>,
        body: Option<&[u8]>,
        require_blockchain: bool,
        require_full_client: bool,
        is_downloading: bool,
        local_handler: impl FnOnce() -> ProxyHandlerResult<Value>,
    ) -> ProxyHandlerResult<Value> {
        if !self.proxy.is_activated(is_downloading) {
            return local_handler();
        }
        
        if !self.proxy.is_forwardable(request_type, require_blockchain, require_full_client) {
            return local_handler();
        }
        
        match local_handler() {
            Ok(result) => Ok(result),
            Err(_) => {
                info!("Local handler failed, trying proxy for {}", request_type);
                
                let response = self.forward_request(
                    request_type,
                    query,
                    body,
                    require_blockchain,
                    require_full_client,
                    is_downloading,
                ).await?;
                
                let json: Value = serde_json::from_str(&response.body)
                    .map_err(|e| ProxyHandlerError::InvalidResponse(e.to_string()))?;
                
                Ok(json)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_sensitive_params_safe() {
        let proxy = Arc::new(ApiProxy::new(Default::default()));
        let handler = ProxyRequestHandler::new(proxy);
        
        let result = handler.check_sensitive_params(Some("requestType=getAccount"), None);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_sensitive_params_unsafe() {
        let proxy = Arc::new(ApiProxy::new(Default::default()));
        let handler = ProxyRequestHandler::new(proxy);
        
        let result = handler.check_sensitive_params(Some("secretPhrase=test"), None);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_forward_request_not_activated() {
        let proxy = Arc::new(ApiProxy::new(Default::default()));
        let handler = ProxyRequestHandler::new(proxy);
        
        let result = handler.forward_request(
            "getAccount",
            Some("requestType=getAccount"),
            None,
            true,
            false,
            false,
        ).await;
        
        assert!(result.is_err());
    }
}
