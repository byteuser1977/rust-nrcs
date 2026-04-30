//! 交易验证模块
//!
//! 对应 Java: TransactionProcessor 中的交易验证逻辑
//!
//! 负责:
//! - 基础交易验证
//! - 签名验证
//! - 余额验证
//! - 时间戳验证
//! - 交易类型特定验证

use blockchain_types::*;
use blockchain_types::prelude::Transaction;
use sha2::{Sha256, Digest};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TxValidationError {
    #[error("invalid transaction version: {0}")]
    InvalidVersion(u8),
    
    #[error("invalid transaction type: {0}")]
    InvalidType(u8),
    
    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(String),
    
    #[error("invalid amount: {0}")]
    InvalidAmount(u64),
    
    #[error("invalid fee: {0}")]
    InvalidFee(u64),
    
    #[error("insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: u64, need: u64 },
    
    #[error("invalid signature")]
    InvalidSignature,
    
    #[error("missing recipient")]
    MissingRecipient,
    
    #[error("transaction expired")]
    Expired,
    
    #[error("duplicate transaction")]
    Duplicate,
    
    #[error("invalid attachment: {0}")]
    InvalidAttachment(String),
    
    #[error("attachment too large: {0} > {1}")]
    AttachmentTooLarge(usize, usize),
    
    #[error("invalid referenced transaction")]
    InvalidReferencedTransaction,
}

pub type TxValidationResult<T> = std::result::Result<T, TxValidationError>;

pub struct TransactionValidator {
    max_amount: u64,
    min_fee: u64,
    max_attachment_size: usize,
    max_timedrift: u32,
}

impl TransactionValidator {
    pub fn new() -> Self {
        Self {
            max_amount: MAX_BALANCE_NQT,
            min_fee: 0,
            max_attachment_size: MAX_PRUNABLE_MESSAGE_LENGTH,
            max_timedrift: MAX_TIMEDRIFT,
        }
    }
    
    pub fn validate_basic(&self, tx: &Transaction) -> TxValidationResult<()> {
        if tx.version != TRANSACTION_VERSION {
            return Err(TxValidationError::InvalidVersion(tx.version));
        }
        
        if tx.amount > self.max_amount {
            return Err(TxValidationError::InvalidAmount(tx.amount));
        }
        
        if tx.fee < self.min_fee {
            return Err(TxValidationError::InvalidFee(tx.fee));
        }
        
        if tx.attachment_bytes.len() > self.max_attachment_size {
            return Err(TxValidationError::AttachmentTooLarge(
                tx.attachment_bytes.len(),
                self.max_attachment_size
            ));
        }
        
        Ok(())
    }
    
    pub fn validate_timestamp(&self, tx: &Transaction, current_time: u32) -> TxValidationResult<()> {
        if tx.timestamp > current_time + self.max_timedrift {
            return Err(TxValidationError::InvalidTimestamp(
                format!("timestamp {} too far in future", tx.timestamp)
            ));
        }
        
        Ok(())
    }
    
    pub fn validate_expiration(&self, tx: &Transaction, current_height: u32) -> TxValidationResult<()> {
        let deadline = tx.deadline as u32;
        if deadline > 0 && current_height > deadline {
            return Err(TxValidationError::Expired);
        }
        
        Ok(())
    }
    
    pub fn validate_balance(
        &self,
        tx: &Transaction,
        available_balance: u64,
        unconfirmed_balance: u64,
    ) -> TxValidationResult<()> {
        let required = tx.amount + tx.fee;
        
        if required > available_balance {
            return Err(TxValidationError::InsufficientBalance {
                have: available_balance,
                need: required,
            });
        }
        
        if required > unconfirmed_balance {
            return Err(TxValidationError::InsufficientBalance {
                have: unconfirmed_balance,
                need: required,
            });
        }
        
        Ok(())
    }
    
    pub fn validate_signature(
        &self,
        tx: &Transaction,
        public_key: &[u8],
    ) -> TxValidationResult<()> {
        let tx_data = self.serialize_for_signing(tx);
        
        let mut hasher = Sha256::new();
        hasher.update(&tx_data);
        hasher.update(public_key);
        let expected_sig = hasher.finalize();
        
        let sig_valid = tx.signature.0[..32] == expected_sig[..];
        
        if !sig_valid {
            return Err(TxValidationError::InvalidSignature);
        }
        
        Ok(())
    }
    
    pub fn validate_recipient(&self, tx: &Transaction) -> TxValidationResult<()> {
        match tx.type_id {
            TransactionType::Payment |
            TransactionType::ColoredCoins if tx.recipient_id.is_none() => {
                return Err(TxValidationError::MissingRecipient);
            }
            _ => {}
        }
        
        Ok(())
    }
    
    pub fn validate_full(
        &self,
        tx: &Transaction,
        current_time: u32,
        current_height: u32,
        available_balance: u64,
        unconfirmed_balance: u64,
        public_key: Option<&[u8]>,
    ) -> TxValidationResult<()> {
        self.validate_basic(tx)?;
        self.validate_timestamp(tx, current_time)?;
        self.validate_expiration(tx, current_height)?;
        self.validate_recipient(tx)?;
        self.validate_balance(tx, available_balance, unconfirmed_balance)?;
        
        if let Some(pk) = public_key {
            self.validate_signature(tx, pk)?;
        }
        
        Ok(())
    }
    
    fn serialize_for_signing(&self, tx: &Transaction) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&[tx.version]);
        buf.extend_from_slice(&[u8::from(tx.type_id)]);
        buf.extend_from_slice(&tx.timestamp.to_be_bytes());
        buf.extend_from_slice(&tx.deadline.to_be_bytes());
        buf.extend_from_slice(&tx.sender_id.to_be_bytes());
        if let Some(recipient) = tx.recipient_id {
            buf.extend_from_slice(&recipient.to_be_bytes());
        }
        buf.extend_from_slice(&tx.amount.to_be_bytes());
        buf.extend_from_slice(&tx.fee.to_be_bytes());
        buf.extend_from_slice(&tx.attachment_bytes);
        buf
    }
}

impl Default for TransactionValidator {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PaymentValidator {
    max_amount: u64,
}

impl PaymentValidator {
    pub fn new() -> Self {
        Self {
            max_amount: MAX_BALANCE_NQT,
        }
    }
    
    pub fn validate(&self, tx: &Transaction) -> TxValidationResult<()> {
        if tx.amount > self.max_amount {
            return Err(TxValidationError::InvalidAmount(tx.amount));
        }
        
        if tx.recipient_id.is_none() {
            return Err(TxValidationError::MissingRecipient);
        }
        
        Ok(())
    }
}

impl Default for PaymentValidator {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AssetTransferValidator {
    #[allow(dead_code)]
    max_quantity: u64,
}

impl AssetTransferValidator {
    pub fn new() -> Self {
        Self {
            max_quantity: MAX_ASSET_QUANTITY_QNT,
        }
    }
    
    pub fn validate(&self, tx: &Transaction) -> TxValidationResult<()> {
        if tx.recipient_id.is_none() {
            return Err(TxValidationError::MissingRecipient);
        }
        
        if tx.attachment_bytes.is_empty() {
            return Err(TxValidationError::InvalidAttachment(
                "asset transfer requires asset info in attachment".to_string()
            ));
        }
        
        Ok(())
    }
}

impl Default for AssetTransferValidator {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LeaseValidator;

impl LeaseValidator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn validate(&self, tx: &Transaction) -> TxValidationResult<()> {
        if tx.recipient_id.is_none() {
            return Err(TxValidationError::MissingRecipient);
        }
        
        Ok(())
    }
}

impl Default for LeaseValidator {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ContractValidator {
    max_code_size: usize,
}

impl ContractValidator {
    pub fn new() -> Self {
        Self {
            max_code_size: MAX_PRUNABLE_MESSAGE_LENGTH,
        }
    }
    
    pub fn validate_deployment(&self, tx: &Transaction) -> TxValidationResult<()> {
        if tx.attachment_bytes.is_empty() {
            return Err(TxValidationError::InvalidAttachment(
                "contract deployment requires code in attachment".to_string()
            ));
        }
        
        if tx.attachment_bytes.len() > self.max_code_size {
            return Err(TxValidationError::AttachmentTooLarge(
                tx.attachment_bytes.len(),
                self.max_code_size
            ));
        }
        
        Ok(())
    }
    
    pub fn validate_invocation(&self, _tx: &Transaction) -> TxValidationResult<()> {
        Ok(())
    }
}

impl Default for ContractValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_transaction() -> Transaction {
        Transaction {
            id: 0,
            version: TRANSACTION_VERSION,
            type_id: TransactionType::Payment,
            subtype: 0,
            timestamp: 1000,
            deadline: 2000,
            sender_public_key: Hash256([0u8; 32]),
            sender_id: 123,
            recipient_id: Some(456),
            amount: 1000,
            fee: 10,
            height: 0,
            block_id: 0,
            block_timestamp: 0,
            transaction_index: 0,
            signature: Signature([0u8; 64]),
            full_hash: Hash256([0u8; 32]),
            referenced_transaction_full_hash: None,
            attachment_bytes: vec![],
            attachment_json: None,
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
        }
    }

    #[test]
    fn test_validator_creation() {
        let validator = TransactionValidator::new();
        assert!(validator.max_amount > 0);
    }

    #[test]
    fn test_validate_basic() {
        let validator = TransactionValidator::new();
        let tx = create_test_transaction();
        
        assert!(validator.validate_basic(&tx).is_ok());
    }

    #[test]
    fn test_validate_timestamp() {
        let validator = TransactionValidator::new();
        let tx = create_test_transaction();
        
        assert!(validator.validate_timestamp(&tx, 1000).is_ok());
        assert!(validator.validate_timestamp(&tx, 500).is_err());
    }

    #[test]
    fn test_validate_balance() {
        let validator = TransactionValidator::new();
        let tx = create_test_transaction();
        
        assert!(validator.validate_balance(&tx, 2000, 2000).is_ok());
        assert!(validator.validate_balance(&tx, 500, 2000).is_err());
    }

    #[test]
    fn test_validate_recipient() {
        let validator = TransactionValidator::new();
        let tx = create_test_transaction();
        
        assert!(validator.validate_recipient(&tx).is_ok());
        
        let mut tx_no_recipient = tx.clone();
        tx_no_recipient.recipient_id = None;
        assert!(validator.validate_recipient(&tx_no_recipient).is_err());
    }

    #[test]
    fn test_payment_validator() {
        let validator = PaymentValidator::new();
        let tx = create_test_transaction();
        
        assert!(validator.validate(&tx).is_ok());
    }
}
