//! 交易类型处理模块
//!
//! 对应 Java: TransactionType 及其子类
//!
//! 负责:
//! - 不同类型交易的验证
//! - 不同类型交易的执行
//! - 不同类型交易的撤销

use blockchain_types::*;
use blockchain_types::prelude::Transaction;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TxTypeError {
    #[error("unsupported transaction type: {0:?}")]
    UnsupportedType(TransactionType),
    
    #[error("invalid attachment: {0}")]
    InvalidAttachment(String),
    
    #[error("invalid amount: {0}")]
    InvalidAmount(u64),
    
    #[error("missing recipient")]
    MissingRecipient,
    
    #[error("asset not found: {0}")]
    AssetNotFound(AssetId),
    
    #[error("insufficient asset balance")]
    InsufficientAssetBalance,
}

pub type TxTypeResult<T> = std::result::Result<T, TxTypeError>;

pub trait TxTypeHandler: Send + Sync {
    fn tx_type(&self) -> TransactionType;
    
    fn validate(&self, tx: &Transaction) -> TxTypeResult<()>;
    
    fn apply(&self, tx: &Transaction, state: &mut TxExecutionContext) -> TxTypeResult<()>;
    
    fn undo(&self, tx: &Transaction, state: &mut TxExecutionContext) -> TxTypeResult<()>;
}

#[derive(Debug, Clone, Default)]
pub struct TxExecutionContext {
    pub sender_balance: u64,
    pub recipient_balance: u64,
    pub sender_id: AccountId,
    pub recipient_id: Option<AccountId>,
    pub height: u32,
    pub timestamp: u32,
}

impl TxExecutionContext {
    pub fn new(
        sender_id: AccountId,
        sender_balance: u64,
        height: u32,
        timestamp: u32,
    ) -> Self {
        Self {
            sender_balance,
            recipient_balance: 0,
            sender_id,
            recipient_id: None,
            height,
            timestamp,
        }
    }
    
    pub fn set_recipient(&mut self, recipient_id: AccountId, balance: u64) {
        self.recipient_id = Some(recipient_id);
        self.recipient_balance = balance;
    }
}

pub struct PaymentHandler;

impl PaymentHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PaymentHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl TxTypeHandler for PaymentHandler {
    fn tx_type(&self) -> TransactionType {
        TransactionType::Payment
    }
    
    fn validate(&self, tx: &Transaction) -> TxTypeResult<()> {
        if tx.amount > MAX_BALANCE_NQT {
            return Err(TxTypeError::InvalidAmount(tx.amount));
        }
        
        if tx.recipient_id.is_none() {
            return Err(TxTypeError::MissingRecipient);
        }
        
        Ok(())
    }
    
    fn apply(&self, tx: &Transaction, state: &mut TxExecutionContext) -> TxTypeResult<()> {
        let total = tx.amount + tx.fee;
        
        if state.sender_balance < total {
            return Err(TxTypeError::InvalidAmount(state.sender_balance));
        }
        
        state.sender_balance -= total;
        
        if let Some(_recipient) = state.recipient_id {
            state.recipient_balance += tx.amount;
        }
        
        Ok(())
    }
    
    fn undo(&self, tx: &Transaction, state: &mut TxExecutionContext) -> TxTypeResult<()> {
        let total = tx.amount + tx.fee;
        
        state.sender_balance += total;
        
        if let Some(_recipient) = state.recipient_id {
            if state.recipient_balance >= tx.amount {
                state.recipient_balance -= tx.amount;
            }
        }
        
        Ok(())
    }
}

pub struct ColoredCoinsHandler;

impl ColoredCoinsHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ColoredCoinsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl TxTypeHandler for ColoredCoinsHandler {
    fn tx_type(&self) -> TransactionType {
        TransactionType::ColoredCoins
    }
    
    fn validate(&self, tx: &Transaction) -> TxTypeResult<()> {
        if tx.recipient_id.is_none() {
            return Err(TxTypeError::MissingRecipient);
        }
        
        if tx.attachment_bytes.is_empty() {
            return Err(TxTypeError::InvalidAttachment(
                "colored coins transfer requires asset info".to_string()
            ));
        }
        
        Ok(())
    }
    
    fn apply(&self, tx: &Transaction, state: &mut TxExecutionContext) -> TxTypeResult<()> {
        if state.sender_balance < tx.fee {
            return Err(TxTypeError::InvalidAmount(state.sender_balance));
        }
        
        state.sender_balance -= tx.fee;
        
        Ok(())
    }
    
    fn undo(&self, tx: &Transaction, state: &mut TxExecutionContext) -> TxTypeResult<()> {
        state.sender_balance += tx.fee;
        Ok(())
    }
}

pub struct LightContractHandler {
    max_code_size: usize,
}

impl LightContractHandler {
    pub fn new() -> Self {
        Self {
            max_code_size: MAX_PRUNABLE_MESSAGE_LENGTH,
        }
    }
}

impl Default for LightContractHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl TxTypeHandler for LightContractHandler {
    fn tx_type(&self) -> TransactionType {
        TransactionType::LightContract
    }
    
    fn validate(&self, tx: &Transaction) -> TxTypeResult<()> {
        if tx.attachment_bytes.len() > self.max_code_size {
            return Err(TxTypeError::InvalidAttachment(
                format!("code size {} exceeds maximum {}", 
                    tx.attachment_bytes.len(), self.max_code_size)
            ));
        }
        
        Ok(())
    }
    
    fn apply(&self, tx: &Transaction, state: &mut TxExecutionContext) -> TxTypeResult<()> {
        if state.sender_balance < tx.fee {
            return Err(TxTypeError::InvalidAmount(state.sender_balance));
        }
        
        state.sender_balance -= tx.fee;
        
        Ok(())
    }
    
    fn undo(&self, tx: &Transaction, state: &mut TxExecutionContext) -> TxTypeResult<()> {
        state.sender_balance += tx.fee;
        Ok(())
    }
}

pub struct MessagingHandler;

impl MessagingHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MessagingHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl TxTypeHandler for MessagingHandler {
    fn tx_type(&self) -> TransactionType {
        TransactionType::Messaging
    }
    
    fn validate(&self, _tx: &Transaction) -> TxTypeResult<()> {
        Ok(())
    }
    
    fn apply(&self, tx: &Transaction, state: &mut TxExecutionContext) -> TxTypeResult<()> {
        if state.sender_balance < tx.fee {
            return Err(TxTypeError::InvalidAmount(state.sender_balance));
        }
        
        state.sender_balance -= tx.fee;
        
        Ok(())
    }
    
    fn undo(&self, tx: &Transaction, state: &mut TxExecutionContext) -> TxTypeResult<()> {
        state.sender_balance += tx.fee;
        Ok(())
    }
}

pub struct DataHandler;

impl DataHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DataHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl TxTypeHandler for DataHandler {
    fn tx_type(&self) -> TransactionType {
        TransactionType::Data
    }
    
    fn validate(&self, _tx: &Transaction) -> TxTypeResult<()> {
        Ok(())
    }
    
    fn apply(&self, tx: &Transaction, state: &mut TxExecutionContext) -> TxTypeResult<()> {
        if state.sender_balance < tx.fee {
            return Err(TxTypeError::InvalidAmount(state.sender_balance));
        }
        
        state.sender_balance -= tx.fee;
        
        Ok(())
    }
    
    fn undo(&self, tx: &Transaction, state: &mut TxExecutionContext) -> TxTypeResult<()> {
        state.sender_balance += tx.fee;
        Ok(())
    }
}

pub struct TxTypeRegistry {
    handlers: std::collections::HashMap<TransactionType, Box<dyn TxTypeHandler>>,
}

impl TxTypeRegistry {
    pub fn new() -> Self {
        let mut handlers: std::collections::HashMap<TransactionType, Box<dyn TxTypeHandler>> = 
            std::collections::HashMap::new();
        
        handlers.insert(TransactionType::Payment, Box::new(PaymentHandler::new()));
        handlers.insert(TransactionType::ColoredCoins, Box::new(ColoredCoinsHandler::new()));
        handlers.insert(TransactionType::LightContract, Box::new(LightContractHandler::new()));
        handlers.insert(TransactionType::Messaging, Box::new(MessagingHandler::new()));
        handlers.insert(TransactionType::Data, Box::new(DataHandler::new()));
        
        Self { handlers }
    }
    
    pub fn get_handler(&self, tx_type: TransactionType) -> Option<&dyn TxTypeHandler> {
        self.handlers.get(&tx_type).map(|h| h.as_ref())
    }
    
    pub fn validate(&self, tx: &Transaction) -> TxTypeResult<()> {
        let handler = self.get_handler(tx.type_id)
            .ok_or_else(|| TxTypeError::UnsupportedType(tx.type_id))?;
        
        handler.validate(tx)
    }
    
    pub fn apply(&self, tx: &Transaction, state: &mut TxExecutionContext) -> TxTypeResult<()> {
        let handler = self.get_handler(tx.type_id)
            .ok_or_else(|| TxTypeError::UnsupportedType(tx.type_id))?;
        
        handler.apply(tx, state)
    }
    
    pub fn undo(&self, tx: &Transaction, state: &mut TxExecutionContext) -> TxTypeResult<()> {
        let handler = self.get_handler(tx.type_id)
            .ok_or_else(|| TxTypeError::UnsupportedType(tx.type_id))?;
        
        handler.undo(tx, state)
    }
}

impl Default for TxTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_transaction(tx_type: TransactionType) -> Transaction {
        Transaction {
            id: 0,
            version: TRANSACTION_VERSION,
            type_id: tx_type,
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
            attachment_bytes: vec![1, 2, 3],
            phased: false,
            has_message: false,
            has_encrypted_message: false,
            has_public_key_announcement: false,
            has_prunable_attachment: false,
            ec_block_height: None,
            ec_block_id: None,
            has_encrypttoself_message: false,
            has_prunable_encrypted_message: false,
        }
    }

    #[test]
    fn test_payment_handler() {
        let handler = PaymentHandler::new();
        let tx = create_test_transaction(TransactionType::Payment);
        
        assert_eq!(handler.tx_type(), TransactionType::Payment);
        assert!(handler.validate(&tx).is_ok());
    }

    #[test]
    fn test_payment_handler_apply() {
        let handler = PaymentHandler::new();
        let tx = create_test_transaction(TransactionType::Payment);
        let mut state = TxExecutionContext::new(123, 2000, 1, 1000);
        state.set_recipient(456, 0);
        
        assert!(handler.apply(&tx, &mut state).is_ok());
        assert_eq!(state.sender_balance, 990);
        assert_eq!(state.recipient_balance, 1000);
    }

    #[test]
    fn test_colored_coins_handler() {
        let handler = ColoredCoinsHandler::new();
        let tx = create_test_transaction(TransactionType::ColoredCoins);
        
        assert_eq!(handler.tx_type(), TransactionType::ColoredCoins);
        assert!(handler.validate(&tx).is_ok());
    }

    #[test]
    fn test_light_contract_handler() {
        let handler = LightContractHandler::new();
        let tx = create_test_transaction(TransactionType::LightContract);
        
        assert_eq!(handler.tx_type(), TransactionType::LightContract);
        assert!(handler.validate(&tx).is_ok());
    }

    #[test]
    fn test_type_registry() {
        let registry = TxTypeRegistry::new();
        
        let tx = create_test_transaction(TransactionType::Payment);
        assert!(registry.validate(&tx).is_ok());
        
        let mut state = TxExecutionContext::new(123, 2000, 1, 1000);
        state.set_recipient(456, 0);
        assert!(registry.apply(&tx, &mut state).is_ok());
    }
}
