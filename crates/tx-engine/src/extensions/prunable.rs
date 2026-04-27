//! Prunable Transaction Module
//!
//! 对应 Java: PrunableTransaction.java
//!
//! 处理可修剪的交易数据（消息、附件等）

use blockchain_types::prelude::TransactionType;

/// 可修剪交易
#[derive(Debug, Clone)]
pub struct PrunableTransaction {
    pub id: u64,
    pub transaction_type: TransactionType,
    pub prunable_attachment: bool,
    pub prunable_plain_message: bool,
    pub prunable_encrypted_message: bool,
}

impl PrunableTransaction {
    pub fn new(
        id: u64,
        transaction_type: TransactionType,
        prunable_attachment: bool,
        prunable_plain_message: bool,
        prunable_encrypted_message: bool,
    ) -> Self {
        Self {
            id,
            transaction_type,
            prunable_attachment,
            prunable_plain_message,
            prunable_encrypted_message,
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn get_transaction_type(&self) -> TransactionType {
        self.transaction_type
    }

    pub fn has_prunable_attachment(&self) -> bool {
        self.prunable_attachment
    }

    pub fn has_prunable_plain_message(&self) -> bool {
        self.prunable_plain_message
    }

    pub fn has_prunable_encrypted_message(&self) -> bool {
        self.prunable_encrypted_message
    }

    pub fn is_pruned(&self) -> bool {
        self.prunable_attachment || self.prunable_plain_message || self.prunable_encrypted_message
    }

    pub fn from_transaction(tx: &blockchain_types::prelude::Transaction) -> Self {
        Self {
            id: tx.id,
            transaction_type: tx.type_id,
            prunable_attachment: tx.has_prunable_attachment,
            prunable_plain_message: tx.has_message,
            prunable_encrypted_message: tx.has_prunable_encrypted_message,
        }
    }
}

/// 可修剪数据存储
#[derive(Debug, Clone)]
pub struct PrunableData {
    pub transaction_id: u64,
    pub data_type: PrunableDataType,
    pub data: Vec<u8>,
    pub nonce: Option<Vec<u8>>,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrunableDataType {
    PlainMessage,
    EncryptedMessage,
    Attachment,
    EncryptToSelfMessage,
}

impl PrunableData {
    pub fn new(
        transaction_id: u64,
        data_type: PrunableDataType,
        data: Vec<u8>,
        nonce: Option<Vec<u8>>,
        height: i32,
    ) -> Self {
        Self {
            transaction_id,
            data_type,
            data,
            nonce,
            height,
        }
    }

    pub fn is_expired(&self, current_height: i32, prune_height: i32) -> bool {
        current_height - self.height > prune_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prunable_transaction_creation() {
        let pt = PrunableTransaction::new(
            12345,
            TransactionType::Messaging,
            true,
            false,
            true,
        );

        assert_eq!(pt.get_id(), 12345);
        assert!(pt.has_prunable_attachment());
        assert!(!pt.has_prunable_plain_message());
        assert!(pt.has_prunable_encrypted_message());
        assert!(pt.is_pruned());
    }
}
