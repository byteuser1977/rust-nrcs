//! 交易附件模块
//!
//! 对应 Java: Attachment 及其子类、AbstractAppendix 及其子类
//!
//! 负责:
//! - 每种交易类型的数据结构定义
//! - 数据库保存（TransactionDbModel）
//!
//! 二进制序列化逻辑已移至 blockchain-types::attachment_serde 模块

use blockchain_types::*;
use blockchain_types::prelude::Transaction;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AttachmentError {
    #[error("serialization error: {0}")]
    Serialization(String),
    
    #[error("deserialization error: {0}")]
    Deserialization(String),
    
    #[error("invalid attachment type for transaction")]
    InvalidType,
    
    #[error("missing required field: {0}")]
    MissingField(String),
    
    #[error("value out of range: {0}")]
    OutOfRange(String),
}

pub type AttachmentResult<T> = std::result::Result<T, AttachmentError>;

/// 交易附件统一枚举（用于内部数据结构，不用于二进制序列化）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Attachment {
    #[default]
    None,
    Payment(PaymentAttachment),
    AssetTransfer(AssetTransferAttachment),
    AssetIssuance(AssetIssuanceAttachment),
    ContractDeployment(ContractDeploymentAttachment),
    ContractInvocation(ContractInvocationAttachment),
    Lease(LeaseAttachment),
    SetProperty(SetPropertyAttachment),
    Message(MessageAttachment),
}

impl Attachment {
    pub fn pack(&self) -> AttachmentResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| AttachmentError::Serialization(e.to_string()))
    }
    
    pub fn unpack(bytes: &[u8]) -> AttachmentResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| AttachmentError::Deserialization(e.to_string()))
    }
    
    pub fn from_tx(tx: &Transaction) -> AttachmentResult<Self> {
        if tx.attachment_bytes.is_empty() {
            return Ok(Self::None);
        }
        
        Self::unpack(&tx.attachment_bytes)
    }
}

// ============ 支付交易附件 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentAttachment {
    pub message: Option<String>,
    pub message_is_text: bool,
}

impl PaymentAttachment {
    pub fn new(message: Option<String>, is_text: bool) -> Self {
        Self {
            message,
            message_is_text: is_text,
        }
    }
    
    pub fn validate(&self) -> AttachmentResult<()> {
        if let Some(ref msg) = self.message {
            if msg.len() > MAX_ARBITRARY_MESSAGE_LENGTH {
                return Err(AttachmentError::OutOfRange(
                    format!("message length {} exceeds maximum {}", 
                        msg.len(), MAX_ARBITRARY_MESSAGE_LENGTH)
                ));
            }
        }
        Ok(())
    }
}

// ============ 资产转移附件 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTransferAttachment {
    pub asset_id: AssetId,
    pub quantity: u64,
    pub comment: Option<String>,
}

impl AssetTransferAttachment {
    pub fn new(asset_id: AssetId, quantity: u64) -> Self {
        Self {
            asset_id,
            quantity,
            comment: None,
        }
    }
    
    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }
    
    pub fn validate(&self) -> AttachmentResult<()> {
        if self.quantity > MAX_ASSET_QUANTITY_QNT {
            return Err(AttachmentError::OutOfRange(
                format!("quantity {} exceeds maximum", self.quantity)
            ));
        }
        
        if let Some(ref comment) = self.comment {
            if comment.len() > MAX_ASSET_TRANSFER_COMMENT_LENGTH {
                return Err(AttachmentError::OutOfRange(
                    format!("comment length {} exceeds maximum", comment.len())
                ));
            }
        }
        
        Ok(())
    }
}

// ============ 资产发行附件 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIssuanceAttachment {
    pub name: String,
    pub description: String,
    pub quantity: u64,
    pub decimals: u8,
    pub mintable: bool,
}

impl AssetIssuanceAttachment {
    pub fn new(name: String, quantity: u64, decimals: u8) -> Self {
        Self {
            name,
            description: String::new(),
            quantity,
            decimals,
            mintable: false,
        }
    }
    
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }
    
    pub fn validate(&self) -> AttachmentResult<()> {
        if self.name.len() < MIN_ASSET_NAME_LENGTH || self.name.len() > MAX_ASSET_NAME_LENGTH {
            return Err(AttachmentError::OutOfRange(
                format!("asset name length must be {}-{}", 
                    MIN_ASSET_NAME_LENGTH, MAX_ASSET_NAME_LENGTH)
            ));
        }
        
        if self.description.len() > MAX_ASSET_DESCRIPTION_LENGTH {
            return Err(AttachmentError::OutOfRange(
                format!("description length {} exceeds maximum", self.description.len())
            ));
        }
        
        if self.quantity > MAX_ASSET_QUANTITY_QNT {
            return Err(AttachmentError::OutOfRange(
                format!("quantity {} exceeds maximum", self.quantity)
            ));
        }
        
        if self.decimals > 8 {
            return Err(AttachmentError::OutOfRange(
                "decimals must be 0-8".to_string()
            ));
        }
        
        Ok(())
    }
}

// ============ 合约部署附件 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDeploymentAttachment {
    pub name: String,
    pub code: Vec<u8>,
    pub constructor_params: Vec<u8>,
    pub gas_limit: u64,
}

impl ContractDeploymentAttachment {
    pub fn new(name: String, code: Vec<u8>) -> Self {
        Self {
            name,
            code,
            constructor_params: vec![],
            gas_limit: 1_000_000,
        }
    }
    
    pub fn with_params(mut self, params: Vec<u8>) -> Self {
        self.constructor_params = params;
        self
    }
    
    pub fn with_gas_limit(mut self, limit: u32) -> Self {
        self.gas_limit = limit as u64;
        self
    }
    
    pub fn validate(&self) -> AttachmentResult<()> {
        if self.name.is_empty() || self.name.len() > MAX_CONTRACT_NAME_LENGTH {
            return Err(AttachmentError::OutOfRange(
                format!("contract name length must be 1-{}", MAX_CONTRACT_NAME_LENGTH)
            ));
        }
        
        if self.code.is_empty() {
            return Err(AttachmentError::MissingField("code".to_string()));
        }
        
        if self.code.len() > MAX_PRUNABLE_MESSAGE_LENGTH {
            return Err(AttachmentError::OutOfRange(
                format!("code size {} exceeds maximum", self.code.len())
            ));
        }
        
        Ok(())
    }
}

// ============ 合约调用附件 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInvocationAttachment {
    pub contract_id: u64,
    pub method: String,
    pub args: Vec<u8>,
    pub gas_limit: u64,
}

impl ContractInvocationAttachment {
    pub fn new(contract_id: u64, method: String, args: Vec<u8>) -> Self {
        Self {
            contract_id,
            method,
            args,
            gas_limit: 500_000,
        }
    }
    
    pub fn with_gas_limit(mut self, limit: u32) -> Self {
        self.gas_limit = limit as u64;
        self
    }
    
    pub fn validate(&self) -> AttachmentResult<()> {
        if self.method.is_empty() {
            return Err(AttachmentError::MissingField("method".to_string()));
        }
        
        if self.args.len() > MAX_CONTRACT_PARAMS_LENGTH {
            return Err(AttachmentError::OutOfRange(
                format!("args length {} exceeds maximum", self.args.len())
            ));
        }
        
        Ok(())
    }
}

// ============ 租赁附件 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseAttachment {
    pub period: u32,
}

impl LeaseAttachment {
    pub fn new(period: u32) -> Self {
        Self { period }
    }
    
    pub fn validate(&self) -> AttachmentResult<()> {
        if self.period == 0 {
            return Err(AttachmentError::OutOfRange(
                "lease period must be greater than 0".to_string()
            ));
        }
        
        Ok(())
    }
}

// ============ 设置属性附件 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPropertyAttachment {
    pub property: String,
    pub value: String,
}

impl SetPropertyAttachment {
    pub fn new(property: String, value: String) -> Self {
        Self { property, value }
    }
    
    pub fn validate(&self) -> AttachmentResult<()> {
        if self.property.is_empty() || self.property.len() > MAX_ACCOUNT_PROPERTY_NAME_LENGTH {
            return Err(AttachmentError::OutOfRange(
                format!("property name length must be 1-{}", MAX_ACCOUNT_PROPERTY_NAME_LENGTH)
            ));
        }
        
        if self.value.len() > MAX_ACCOUNT_PROPERTY_VALUE_LENGTH {
            return Err(AttachmentError::OutOfRange(
                format!("value length {} exceeds maximum", self.value.len())
            ));
        }
        
        Ok(())
    }
}

// ============ 消息附件 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub message: Vec<u8>,
    pub is_text: bool,
    pub is_encrypted: bool,
}

impl MessageAttachment {
    pub fn new_text(message: String) -> Self {
        Self {
            message: message.into_bytes(),
            is_text: true,
            is_encrypted: false,
        }
    }
    
    pub fn new_binary(message: Vec<u8>) -> Self {
        Self {
            message,
            is_text: false,
            is_encrypted: false,
        }
    }
    
    pub fn encrypted(mut self) -> Self {
        self.is_encrypted = true;
        self
    }
    
    pub fn validate(&self) -> AttachmentResult<()> {
        let max_len = if self.is_encrypted {
            MAX_ENCRYPTED_MESSAGE_LENGTH
        } else {
            MAX_ARBITRARY_MESSAGE_LENGTH
        };
        
        if self.message.len() > max_len {
            return Err(AttachmentError::OutOfRange(
                format!("message length {} exceeds maximum {}", self.message.len(), max_len)
            ));
        }
        
        Ok(())
    }
}

// ============ 数据库模型 ============

#[derive(Debug, Clone)]
pub struct TransactionDbModel {
    pub id: i64,
    pub version: i32,
    pub type_id: i32,
    pub subtype: i32,
    pub timestamp: i32,
    pub deadline: i32,
    pub sender_id: i64,
    pub recipient_id: Option<i64>,
    pub amount: i64,
    pub fee: i64,
    pub height: i32,
    pub block_id: i64,
    pub signature: Vec<u8>,
    pub full_hash: Vec<u8>,
    pub attachment_bytes: Vec<u8>,
    pub referenced_transaction_id: Option<i64>,
}

impl TransactionDbModel {
    pub fn from_transaction(tx: &Transaction, block_height: u32, block_id: u64) -> Self {
        Self {
            id: 0,
            version: tx.version as i32,
            type_id: u8::from(tx.type_id) as i32,
            subtype: tx.subtype as i32,
            timestamp: tx.timestamp as i32,
            deadline: tx.deadline as i32,
            sender_id: tx.sender_id as i64,
            recipient_id: tx.recipient_id.map(|id| id as i64),
            amount: tx.amount as i64,
            fee: tx.fee as i64,
            height: block_height as i32,
            block_id: block_id as i64,
            signature: tx.signature.0.to_vec(),
            full_hash: tx.full_hash.0.to_vec(),
            attachment_bytes: tx.attachment_bytes.clone(),
            referenced_transaction_id: None,
        }
    }
    
    pub fn to_transaction(&self) -> AttachmentResult<Transaction> {
        Ok(Transaction {
            id: 0,
            version: self.version as u8,
            type_id: TransactionType::from_type(self.type_id as u8),
            subtype: self.subtype as u8,
            timestamp: self.timestamp as u32,
            deadline: self.deadline as u16,
            sender_public_key: Hash256([0u8; 32]),
            sender_id: self.sender_id as u64,
            recipient_id: self.recipient_id.map(|id| id as u64),
            amount: self.amount as u64,
            fee: self.fee as u64,
            height: self.height as u32,
            block_id: self.block_id as u64,
            block_timestamp: 0,
            transaction_index: 0,
            signature: {
                let mut arr = [0u8; 64];
                arr.copy_from_slice(&self.signature[..64]);
                Signature(arr)
            },
            full_hash: {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&self.full_hash[..32]);
                Hash256(arr)
            },
            referenced_transaction_full_hash: None,
            attachment_bytes: self.attachment_bytes.clone(),
            pruned_attachment_bytes: 0,
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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_attachment() {
        let attachment = PaymentAttachment::new(Some("test message".to_string()), true);
        assert!(attachment.validate().is_ok());
        
        let attachment_enum = Attachment::Payment(attachment);
        let packed = attachment_enum.pack().unwrap();
        let unpacked = Attachment::unpack(&packed).unwrap();
        
        match unpacked {
            Attachment::Payment(p) => {
                assert_eq!(p.message, Some("test message".to_string()));
            }
            _ => panic!("wrong type"),
        }
    }

    #[test]
    fn test_asset_transfer_attachment() {
        let attachment = AssetTransferAttachment::new(123, 1000);
        assert!(attachment.validate().is_ok());
    }

    #[test]
    fn test_asset_issuance_attachment() {
        let attachment = AssetIssuanceAttachment::new("TEST".to_string(), 1_000_000, 8);
        assert!(attachment.validate().is_ok());
    }

    #[test]
    fn test_contract_deployment_attachment() {
        let attachment = ContractDeploymentAttachment::new(
            "test_contract".to_string(),
            vec![1, 2, 3, 4],
        );
        assert!(attachment.validate().is_ok());
    }

    #[test]
    fn test_attachment_pack_unpack() {
        let attachment = Attachment::Payment(PaymentAttachment::new(Some("hello".to_string()), true));
        
        let packed = attachment.pack().unwrap();
        let unpacked = Attachment::unpack(&packed).unwrap();
        
        match unpacked {
            Attachment::Payment(p) => {
                assert_eq!(p.message, Some("hello".to_string()));
            }
            _ => panic!("wrong type"),
        }
    }

    #[test]
    fn test_transaction_db_model() {
        let tx = Transaction {
            id: 0,
            version: 1,
            type_id: TransactionType::Payment,
            subtype: 0,
            timestamp: 1000,
            deadline: 100,
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
            full_hash: Hash256([1u8; 32]),
            referenced_transaction_full_hash: None,
            attachment_bytes: vec![],
            pruned_attachment_bytes: 0,
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
        };
        
        let model = TransactionDbModel::from_transaction(&tx, 100, 1);
        assert_eq!(model.sender_id, 123);
        assert_eq!(model.recipient_id, Some(456));
        
        let restored = model.to_transaction().unwrap();
        assert_eq!(restored.sender_id, tx.sender_id);
        assert_eq!(restored.recipient_id, tx.recipient_id);
    }
}
