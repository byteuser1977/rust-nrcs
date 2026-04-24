use crate::protocol::PeerRequest;
use serde_json;
use std::sync::Arc;
use tracing::{debug, warn, info};
use orm::TransactionRepository;

pub struct GetTransactionsHandler {
    tx_repo: Option<Arc<dyn TransactionRepository>>,
}

impl GetTransactionsHandler {
    pub fn new() -> Self {
        Self { tx_repo: None }
    }

    pub fn with_tx_repo(tx_repo: Arc<dyn TransactionRepository>) -> Self {
        Self { tx_repo: Some(tx_repo) }
    }

    pub async fn handle(&self, request: PeerRequest) -> serde_json::Value {
        debug!("Handling GetTransactions request");

        let transaction_ids: Option<Vec<String>> = request.get("transactionIds");
        let limit: i64 = request.get("limit").unwrap_or(100);

        if let Some(ref tx_repo) = self.tx_repo {
            let mut transactions = Vec::new();

            if let Some(ids) = transaction_ids {
                debug!("Requested {} specific transactions", ids.len());
                
                for id_str in ids.iter().take(limit as usize) {
                    if let Ok(id) = id_str.parse::<i64>() {
                        match tx_repo.find_by_txid(id).await {
                            Ok(Some(model)) => {
                                let tx_json = serde_json::json!({
                                    "transaction": {
                                        "type": model.r#type,
                                        "subtype": model.subtype,
                                        "timestamp": model.timestamp,
                                        "deadline": model.deadline,
                                        "senderId": model.sender_id.to_string(),
                                        "recipientId": model.recipient_id.map(|id| id.to_string()),
                                        "amountNQT": model.amount.to_string(),
                                        "feeNQT": model.fee.to_string(),
                                        "fullHash": hex::encode(&model.full_hash),
                                        "signature": hex::encode(&model.signature),
                                        "height": model.height,
                                        "version": model.version,
                                    }
                                });
                                transactions.push(tx_json);
                            }
                            Ok(None) => {
                                debug!("Transaction {} not found", id);
                            }
                            Err(e) => {
                                warn!("Error fetching transaction {}: {}", id, e);
                            }
                        }
                    }
                }
            } else {
                debug!("Fetching recent transactions (limit: {})", limit);
                
                match tx_repo.count().await {
                    Ok(total) => {
                        debug!("Total transactions in database: {}", total);
                        // TODO: 实现获取最近交易的逻辑
                        // 当前 TransactionRepository 没有获取最近交易的方法
                        // 需要添加 find_recent 或类似方法
                    }
                    Err(e) => {
                        warn!("Error counting transactions: {}", e);
                    }
                }
            }

            info!("Returning {} transactions", transactions.len());
            serde_json::json!({ "transactions": transactions })
        } else {
            warn!("GetTransactions: transaction repository not configured");
            serde_json::json!({ "transactions": [] })
        }
    }
}
