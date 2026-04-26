//! Transaction-related database models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use blockchain_types::{AccountId, Amount, BlockId, Result, TransactionType};
use blockchain_types::prelude::*;

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
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
    pub block_timestamp: i32,
    pub transaction_index: i16,
    pub signature: Vec<u8>,
    pub timestamp: i32,
    pub r#type: i16,
    pub subtype: i16,
    pub sender_id: i64,
    pub referenced_transaction_full_hash: Option<Vec<u8>>,
    pub attachment_bytes: Option<Vec<u8>>,
    pub version: i16,
    pub phased: bool,
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
        let signature = if self.signature.len() == 64 {
            let mut arr = [0u8; 64];
            arr.copy_from_slice(&self.signature);
            Signature(arr)
        } else {
            Signature([0u8; 64])
        };

        let full_hash = if self.full_hash.len() == 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&self.full_hash);
            Hash256(arr)
        } else {
            Hash256([0u8; 32])
        };

        let referenced_transaction_full_hash = self.referenced_transaction_full_hash.as_ref().and_then(|v| {
            if v.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(v);
                Some(Hash256(arr))
            } else {
                None
            }
        });

        let tx = Transaction {
            id: self.id as u64,
            version: self.version as u8,
            type_id: TransactionType::from_type(self.r#type as u8),
            subtype: self.subtype as u8,
            timestamp: self.timestamp as Timestamp,
            deadline: self.deadline as u16,
            sender_public_key: Hash256([0u8; 32]),
            sender_id: self.sender_id as AccountId,
            recipient_id: self.recipient_id.map(|id| id as AccountId),
            amount: self.amount as Amount,
            fee: self.fee as Amount,
            height: self.height as Height,
            block_id: self.block_id as BlockId,
            block_timestamp: self.block_timestamp as Timestamp,
            transaction_index: self.transaction_index as u16,
            signature,
            full_hash,
            referenced_transaction_full_hash,
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
            id: tx.id as i64,
            deadline: tx.deadline as i16,
            recipient_id: tx.recipient_id.map(|id| id as i64),
            amount: tx.amount as i64,
            fee: tx.fee as i64,
            full_hash: tx.full_hash.0.to_vec(),
            height: tx.height as i32,
            block_id: tx.block_id as i64,
            block_timestamp: tx.block_timestamp as i32,
            transaction_index: tx.transaction_index as i16,
            signature: tx.signature.0.to_vec(),
            timestamp: tx.timestamp as i32,
            r#type: tx.type_id.to_byte() as i16,
            subtype: tx.subtype as i16,
            sender_id: tx.sender_id as i64,
            referenced_transaction_full_hash: tx.referenced_transaction_full_hash.as_ref().map(|h| h.0.to_vec()),
            attachment_bytes: if tx.attachment_bytes.is_empty() { None } else { Some(tx.attachment_bytes.clone()) },
            version: tx.version as i16,
            phased: tx.phased,
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
