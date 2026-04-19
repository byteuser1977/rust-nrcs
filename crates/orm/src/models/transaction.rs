//! Transaction-related database models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use blockchain_types::{AccountId, Amount, BlockId, BlockchainError, Result, TransactionType};
use blockchain_types::prelude::*;

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
pub struct TransactionModel {
    pub db_id: i64,
    pub id: i64,
    pub deadline: i16,
    pub recipient_id: Option<i64>,
    pub amount: i64,
    pub fee: i64,
    pub full_hash: Vec<u8>,
    pub height: i32,
    pub block_id: i64,
    pub signature: Vec<u8>,
    pub timestamp: i32,
    pub r#type: i16,
    pub subtype: i16,
    pub sender_id: i64,
    pub block_timestamp: i32,
    pub referenced_transaction_full_hash: Option<Vec<u8>>,
    pub transaction_index: i16,
    pub phased: bool,
    pub attachment_bytes: Option<Vec<u8>>,
    pub version: i16,
    pub has_message: bool,
    pub has_encrypted_message: bool,
    pub has_public_key_announcement: bool,
    pub has_prunable_message: bool,
    pub has_prunable_attachment: bool,
    pub ec_block_height: Option<i32>,
    pub ec_block_id: Option<i64>,
    pub has_encrypttoself_message: bool,
    pub has_prunable_encrypted_message: bool,
}

impl TransactionModel {
    pub fn to_domain(&self) -> Result<Transaction> {
        let sender_id = self.sender_id as AccountId;
        let tx = Transaction {
            version: self.version as u8,
            type_id: TransactionType::from(self.r#type as u8),
            subtype: self.subtype as u8,
            timestamp: self.timestamp as Timestamp,
            deadline: self.deadline as u16,
            sender_id,
            recipient_id: self.recipient_id.map(|id| id as AccountId),
            amount: self.amount as Amount,
            fee: self.fee as Amount,
            height: self.height as Height,
            block_id: self.block_id as BlockId,
            signature: self.signature.as_slice().try_into().map(Signature).unwrap_or(Signature([0u8; 64])),
            full_hash: self.full_hash.as_slice().try_into().map(Hash256).map_err(|_| {
                BlockchainError::InvalidHash("full_hash length mismatch".to_string())
            })?,
            attachment_bytes: self.attachment_bytes.clone().unwrap_or_default(),
            phased: self.phased,
            has_message: self.has_message,
            has_encrypted_message: self.has_encrypted_message,
            has_public_key_announcement: self.has_public_key_announcement,
            has_prunable_attachment: self.has_prunable_attachment,
            ec_block_height: self.ec_block_height.map(|h| h as u32),
            ec_block_id: self.ec_block_id.map(|id| id as u64),
            has_encrypttoself_message: self.has_encrypttoself_message,
            has_prunable_encrypted_message: self.has_prunable_encrypted_message,
        };
        Ok(tx)
    }

    pub fn from_domain(tx: &Transaction) -> Result<Self> {
        Ok(Self {
            db_id: 0,
            id: 0,
            deadline: tx.deadline as i16,
            recipient_id: tx.recipient_id.map(|id| id as i64),
            amount: tx.amount as i64,
            fee: tx.fee as i64,
            full_hash: tx.full_hash.0.to_vec(),
            height: tx.height as i32,
            block_id: tx.block_id as i64,
            signature: tx.signature.0.to_vec(),
            timestamp: tx.timestamp as i32,
            r#type: u8::from(tx.type_id) as i16,
            subtype: tx.subtype as i16,
            sender_id: tx.sender_id as i64,
            block_timestamp: 0,
            referenced_transaction_full_hash: None,
            transaction_index: 0,
            phased: tx.phased,
            attachment_bytes: Some(tx.attachment_bytes.clone()),
            version: tx.version as i16,
            has_message: tx.has_message,
            has_encrypted_message: tx.has_encrypted_message,
            has_public_key_announcement: tx.has_public_key_announcement,
            has_prunable_message: tx.has_prunable_attachment,
            has_prunable_attachment: tx.has_prunable_attachment,
            ec_block_height: tx.ec_block_height.map(|h| h as i32),
            ec_block_id: tx.ec_block_id.map(|id| id as i64),
            has_encrypttoself_message: tx.has_encrypttoself_message,
            has_prunable_encrypted_message: tx.has_prunable_encrypted_message,
        })
    }
}
