use crate::{peer::Peers, protocol::PeerRequest};
use serde_json;
use std::sync::Arc;
use tracing::{debug, warn, info};
use tx_engine::TransactionProcessor;
use blockchain_types::prelude::*;

pub struct ProcessTransactionsHandler {
    peers: Arc<Peers>,
    tx_processor: Option<Arc<dyn TransactionProcessor>>,
}

impl ProcessTransactionsHandler {
    pub fn new(peers: Arc<Peers>) -> Self {
        Self { peers, tx_processor: None }
    }

    pub fn with_tx_processor(peers: Arc<Peers>, tx_processor: Arc<dyn TransactionProcessor>) -> Self {
        Self { peers, tx_processor: Some(tx_processor) }
    }

    pub async fn handle(&self, request: PeerRequest, _peers: Arc<Peers>) -> serde_json::Value {
        debug!("Handling ProcessTransactions request");

        let transactions: Vec<serde_json::Value> = match request.get("transactions") {
            Some(txs) => txs,
            None => {
                warn!("ProcessTransactions missing 'transactions' field");
                return serde_json::json!({ "error": "MISSING_TRANSACTIONS" });
            }
        };

        debug!("Received {} transactions", transactions.len());

        if let Some(ref tx_processor) = self.tx_processor {
            let mut valid_count = 0;
            let mut invalid_count = 0;

            for tx_json in &transactions {
                match self.parse_transaction(tx_json) {
                    Ok(tx) => {
                        match tx_processor.validate(&tx).await {
                            Ok(()) => {
                                debug!("Transaction {} validated successfully", tx.sender_id);
                                valid_count += 1;
                            }
                            Err(e) => {
                                warn!("Transaction validation failed: {}", e);
                                invalid_count += 1;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse transaction: {}", e);
                        invalid_count += 1;
                    }
                }
            }

            info!("Processed {} transactions: {} valid, {} invalid", 
                  transactions.len(), valid_count, invalid_count);

            serde_json::json!({})
        } else {
            warn!("ProcessTransactions: transaction processor not configured");
            serde_json::json!({})
        }
    }

    fn parse_transaction(&self, tx_json: &serde_json::Value) -> std::result::Result<Transaction, String> {
        let obj = tx_json.as_object().ok_or("transaction is not an object")?;
        
        let type_byte: u8 = obj.get("type")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;
        
        let _subtype: u8 = obj.get("subtype")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;

        let type_id = TransactionType::from(type_byte);

        let timestamp: u32 = obj.get("timestamp")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let deadline: u16 = obj.get("deadline")
            .and_then(|v| v.as_u64())
            .unwrap_or(1440) as u16;

        let sender_id: u64 = obj.get("sender")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .or_else(|| obj.get("senderId").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()))
            .unwrap_or(0);

        let recipient_id: Option<u64> = obj.get("recipient")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .or_else(|| obj.get("recipientId").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()));

        let amount: u64 = obj.get("amountNQT")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .or_else(|| obj.get("amount").and_then(|v| v.as_u64()))
            .unwrap_or(0);

        let fee: u64 = obj.get("feeNQT")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .or_else(|| obj.get("fee").and_then(|v| v.as_u64()))
            .unwrap_or(0);

        let signature_bytes = obj.get("signature")
            .and_then(|v| v.as_str())
            .and_then(|s| hex::decode(s).ok())
            .unwrap_or_default();

        let attachment_bytes = obj.get("attachmentBytes")
            .and_then(|v| v.as_str())
            .and_then(|s| hex::decode(s).ok())
            .unwrap_or_default();

        let full_hash_bytes = obj.get("fullHash")
            .and_then(|v| v.as_str())
            .and_then(|s| hex::decode(s).ok())
            .unwrap_or_default();

        let mut tx = Transaction::new(
            type_id,
            sender_id,
            recipient_id,
            amount,
            fee,
            timestamp,
            deadline,
        );

        if signature_bytes.len() == 64 {
            let mut sig_arr = [0u8; 64];
            sig_arr.copy_from_slice(&signature_bytes);
            tx.signature = Signature(sig_arr);
        }

        if full_hash_bytes.len() == 32 {
            let mut hash_arr = [0u8; 32];
            hash_arr.copy_from_slice(&full_hash_bytes);
            tx.full_hash = Hash256(hash_arr);
        }

        tx.attachment_bytes = attachment_bytes;

        Ok(tx)
    }
}
