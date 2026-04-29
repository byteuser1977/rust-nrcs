//! BundlerRate 处理器
//!
//! 对应 NRCS Java: BundlerRateServlet
//!
//! 功能:
//! - 接收并处理 bundler rate 信息
//! - 返回当前节点的 bundler rate
//! - 支持协议版本 2

use crate::protocol::PeerRequest;
use serde_json::{self, json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// BundlerRate 处理器
///
/// 对应 NRCS Java: BundlerRateServlet
pub struct BundlerRateHandler {
    /// 当前 rates 数据（动态更新）
    current_rates: Arc<RwLock<Value>>,
}

impl Default for BundlerRateHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl BundlerRateHandler {
    /// 创建新的 BundlerRate 处理器
    pub fn new() -> Self {
        Self {
            current_rates: Arc::new(RwLock::new(Value::Null)),
        }
    }

    /// 使用初始 rates 创建处理器
    pub fn with_initial_rates(initial_rates: Value) -> Self {
        Self {
            current_rates: Arc::new(RwLock::new(initial_rates)),
        }
    }

    /// 处理 BundlerRate 请求
    ///
    /// 对应 NRCS Java: BundlerRateServlet.processRequest()
    ///
    /// 请求格式 (protocol=2):
    /// ```json
    /// {
    ///   "requestType": "bundlerRate",
    ///   "protocol": 2,
    ///   "rates": { ... }
    /// }
    /// ```
    ///
    /// 响应格式:
    /// ```json
    /// {
    ///   "rates": { ... },
    ///   "requestProcessingTime": 0
    /// }
    /// ```
    pub async fn handle(&self, request: PeerRequest) -> Value {
        debug!("Handling BundlerRate request");

        let protocol: i32 = request.get("protocol").unwrap_or(1);

        // 检查是否包含传入的 rates（其他节点广播的）
        if let Some(incoming_rates) = request.get::<Value>("rates") {
            if !incoming_rates.is_null() {
                info!("[BundlerRate] Received rates from peer");
                self.update_rates(&incoming_rates).await;
            }
        }

        // 构建响应
        let response = self.build_response(protocol).await;

        debug!("[BundlerRate] Response sent");
        response
    }

    /// 更新 rates 数据
    ///
    /// 对应 NRCS Java: 接收其他节点广播的 bundler rates
    pub async fn update_rates(&self, new_rates: &Value) {
        let mut rates = self.current_rates.write().await;

        // 合并 incoming rates 到现有数据
        // 实际实现中可能需要更复杂的合并逻辑
        if rates.is_null() {
            *rates = new_rates.clone();
        } else if let Some(obj) = new_rates.as_object() {
            if let Some(existing) = rates.as_object_mut() {
                for (key, value) in obj {
                    existing.insert(key.clone(), value.clone());
                }
            }
        }

        debug!("[BundlerRate] Rates updated");
    }

    /// 获取当前的 rates 数据
    pub async fn get_current_rates(&self) -> Value {
        self.current_rates.read().await.clone()
    }

    /// 设置 rates 数据（由外部调用，如从区块链状态获取）
    pub async fn set_rates(&self, rates: Value) {
        *self.current_rates.write().await = rates;
    }

    /// 构建 BundlerRate 响应
    async fn build_response(&self, _protocol: i32) -> Value {
        let rates = self.current_rates.read().await;

        // 如果没有数据，返回空对象
        if rates.is_null() {
            return json!({
                "rates": json!({}),
                "requestProcessingTime": 0
            });
        }

        json!({
            "rates": (*rates).clone(),
            "requestProcessingTime": 0
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bundler_rate_handler_creation() {
        let handler = BundlerRateHandler::new();
        let rates = handler.get_current_rates().await;
        assert!(rates.is_null());
    }

    #[tokio::test]
    async fn test_bundler_rate_with_initial_rates() {
        let initial = json!({
            "bundlers": [
                {"account": "123", "rate": 100}
            ]
        });
        let handler = BundlerRateHandler::with_initial_rates(initial.clone());

        let rates = handler.get_current_rates().await;
        assert_eq!(rates, initial);
    }

    #[tokio::test]
    async fn test_bundler_rate_handle_request() {
        let handler = BundlerRateHandler::new();

        // 设置一些 rates
        handler.set_rates(json!({"minFeePerByte": 100})).await;

        let request = PeerRequest::new(crate::protocol::RequestType::BundlerRate, 2);
        let response = handler.handle(request).await;

        assert!(response.get("rates").is_some());
        assert!(response.get("requestProcessingTime").is_some());
    }

    #[tokio::test]
    async fn test_bundler_rate_update_and_merge() {
        let handler = BundlerRateHandler::new();

        // 初始设置
        handler.set_rates(json!({"field1": "value1"})).await;

        // 更新新字段
        handler.update_rates(&json!({"field2": "value2"})).await;

        let rates = handler.get_current_rates().await;
        assert_eq!(rates.get("field1").unwrap(), "value1");
        assert_eq!(rates.get("field2").unwrap(), "value2");
    }

    #[tokio::test]
    async fn test_bundler_rate_handle_incoming_rates() {
        let handler = BundlerRateHandler::new();

        let mut request = PeerRequest::new(crate::protocol::RequestType::BundlerRate, 2);
        request.set("rates", json!({"external": true}));

        let _response = handler.handle(request).await;

        // 验证 incoming rates 已被接收
        let rates = handler.get_current_rates().await;
        assert_eq!(rates.get("external").unwrap(), true);
    }
}
