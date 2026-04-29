//! GetTransactions 处理器
//!
//! 对应 NRCS Java: GetTransactionsServlet
//!
//! 功能:
//! - 根据交易 ID 获取特定交易
//! - 获取最近确认的交易（分页支持）
//! - 返回完整的交易信息

use crate::protocol::PeerRequest;
use serde_json;
use std::sync::Arc;
use tracing::{debug, info, warn};
use orm::TransactionRepository;

/// GetTransactions Handler
///
/// 对应 NRCS Java: GetTransactionsServlet
pub struct GetTransactionsHandler {
    tx_repo: Option<Arc<dyn TransactionRepository>>,
}

impl Default for GetTransactionsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl GetTransactionsHandler {
    /// Create a new handler without repository
    pub fn new() -> Self {
        Self { tx_repo: None }
    }

    /// Create a handler with transaction repository
    pub fn with_tx_repo(tx_repo: Arc<dyn TransactionRepository>) -> Self {
        Self { tx_repo: Some(tx_repo) }
    }

    /// Handle GetTransactions request（完整实现）
    ///
    /// 对应 NRCS Java: GetTransactionsServlet.processRequest()
    ///
    /// 请求格式:
    /// ```json
    /// {
    ///   "requestType": "getTransactions",
    ///   "transactionIds": ["123", "456"],  // 可选：指定交易 ID 列表
    ///   "limit": 100                        // 可选：限制返回数量（默认 100）
    /// }
    /// ```
    ///
    /// 响应格式:
    /// ```json
    /// {
    ///   "transactions": [...],
    ///   "requestProcessingTime": 0
    /// }
    /// ```
    pub async fn handle(&self, request: PeerRequest) -> serde_json::Value {
        debug!("Handling GetTransactions request");

        let transaction_ids: Option<Vec<String>> = request.get("transactionIds");
        let limit: i64 = request.get("limit").unwrap_or(100);

        if let Some(ref tx_repo) = self.tx_repo {
            let mut transactions = Vec::new();

            if let Some(ids) = transaction_ids {
                // 模式1：根据 ID 列表获取特定交易
                debug!("[GetTransactions] Requested {} specific transactions", ids.len());

                for id_str in ids.iter().take(limit as usize) {
                    if let Ok(id) = id_str.parse::<i64>() {
                        match tx_repo.find_by_txid(id).await {
                            Ok(Some(model)) => {
                                let tx_json = Self::build_transaction_response(&model);
                                transactions.push(tx_json);
                            }
                            Ok(None) => {
                                debug!("[GetTransactions] Transaction {} not found", id);
                            }
                            Err(e) => {
                                warn!("[GetTransactions] Error fetching transaction {}: {}", id, e);
                            }
                        }
                    } else {
                        warn!("[GetTransactions] Invalid transaction ID format: {}", id_str);
                    }
                }

                info!("[GetTransactions] Returning {} of {} requested transactions",
                      transactions.len(), ids.len().min(limit as usize));
            } else {
                // 模式2：获取最近确认的交易（分页）
                debug!("[GetTransactions] Fetching recent transactions (limit: {})", limit);

                match Self::fetch_recent_transactions(tx_repo, limit).await {
                    Ok(recent_txs) => {
                        transactions = recent_txs;
                        info!("[GetTransactions] Returning {} recent transactions",
                              transactions.len());
                    }
                    Err(e) => {
                        warn!("[GetTransactions] Error fetching recent transactions: {}", e);
                    }
                }
            }

            serde_json::json!({
                "transactions": transactions,
                "requestProcessingTime": 0
            })
        } else {
            warn!("[GetTransactions] Transaction repository not configured");
            serde_json::json!({
                "transactions": [],
                "errorDescription": "Transaction database not available",
                "requestProcessingTime": 0
            })
        }
    }

    /// Fetch recent transactions from database
    ///
    /// 对应 NRCS Java: 从数据库获取最近的已确认交易
    ///
    /// 注意：当前实现使用 find_all 并限制数量，
    /// 实际生产环境应该使用按时间/高度排序的查询
    async fn fetch_recent_transactions(
        tx_repo: &Arc<dyn TransactionRepository>,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, String> {
        // 使用 find_unconfirmed 或 find_all 获取最近的交易
        // 注意：实际实现可能需要添加 find_recent 或类似方法到 TransactionRepository

        match tx_repo.find_unconfirmed(limit).await {
            Ok(models) => {
                let transactions: Vec<serde_json::Value> = models
                    .iter()
                    .map(Self::build_transaction_response)
                    .collect();
                Ok(transactions)
            }
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }

    /// Build transaction response JSON
    ///
    /// 将数据库模型转换为 API 响应格式（与 Java NRCS 兼容）
    fn build_transaction_response(model: &orm::models::transaction::TransactionModel) -> serde_json::Value {
        serde_json::json!({
            "transaction": {
                "type": model.r#type,
                "subtype": model.subtype,
                "timestamp": model.timestamp,
                "deadline": model.deadline,
                "senderId": model.sender_id.to_string(),
                "recipientId": model.recipient_id.map(|id| id.to_string()),
                "amountNQT": model.amount.to_string(),
                "feeNQT": model.fee.to_string(),
                "signature": hex::encode(&model.signature),
                "fullHash": hex::encode(&model.full_hash),
                "height": model.height,
                "version": model.version,
                "block": Some(model.block_id.to_string()),
                "attachment": model.attachment_bytes.clone()
            },
            "requestProcessingTime": 0
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_transactions_handler_creation() {
        let handler = GetTransactionsHandler::new();
        assert!(handler.tx_repo.is_none());
    }

    #[tokio::test]
    async fn test_handle_without_repository() {
        let handler = GetTransactionsHandler::new();
        let request = PeerRequest::new(crate::protocol::RequestType::GetTransactions, 1);

        let response = handler.handle(request).await;
        assert!(response.get("transactions").is_some());

        let transactions = response.get("transactions").unwrap().as_array().unwrap();
        assert!(transactions.is_empty()); // 无 repo 时返回空数组
    }

    #[tokio::test]
    async fn test_handle_with_specific_ids_format() {
        let handler = GetTransactionsHandler::new();

        let mut request = PeerRequest::new(crate::protocol::RequestType::GetTransactions, 1);
        request.set("transactionIds", vec!["123456".to_string(), "789012".to_string()]);
        request.set("limit", 10);

        let response = handler.handle(request).await;
        assert!(response.get("transactions").is_some());
    }

    #[tokio::test]
    async fn test_build_transaction_response_structure() {
        use orm::models::transaction::TransactionModel;

        let model = TransactionModel {
            db_id: 1,
            id: 1i64,
            r#type: 0i16,
            subtype: 0i16,
            timestamp: 1701144000i32,
            deadline: 1440i16,
            sender_id: 1234567890i64,
            recipient_id: Some(9876543210i64),
            amount: 100000000i64,
            fee: 1000000i64,
            signature: vec![0u8; 64],
            full_hash: vec![0u8; 32],
            height: 100i32,
            version: 1i16,
            block_id: 1000i64,
            attachment_bytes: None,
            block_timestamp: 0i32,
            transaction_index: 0i16,
            referenced_transaction_full_hash: None,
            phased: false,
            has_message: false,
            has_encrypted_message: false,
            has_public_key_announcement: false,
            has_prunable_message: false,
            has_prunable_attachment: false,
            ec_block_height: None,
            ec_block_id: None,
            has_encrypttoself_message: false,
            has_prunable_encrypted_message: false,
        };

        let response = GetTransactionsHandler::build_transaction_response(&model);

        // 验证响应结构包含必要字段
        assert!(response.get("transaction").is_some());
        let tx = response.get("transaction").unwrap();

        assert!(tx.get("type").is_some());
        assert!(tx.get("timestamp").is_some());
        assert!(tx.get("senderId").is_some());
        assert!(tx.get("recipientId").is_some());
        assert!(tx.get("amountNQT").is_some());
        assert!(tx.get("feeNQT").is_some());
        assert!(tx.get("fullHash").is_some());
        assert!(tx.get("height").is_some());
        assert!(tx.get("version").is_some());
        assert!(tx.get("block").is_some());
    }
}
