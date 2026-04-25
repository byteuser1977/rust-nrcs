//! 交易数据结构定义

use super::*;
use serde::{Serialize, Deserialize};

pub const TYPE_PAYMENT: u8 = 0;
pub const TYPE_MESSAGING: u8 = 1;
pub const TYPE_COLORED_COINS: u8 = 2;
pub const TYPE_DIGITAL_GOODS: u8 = 3;
pub const TYPE_ACCOUNT_CONTROL: u8 = 4;
pub const TYPE_MONETARY_SYSTEM: u8 = 5;
pub const TYPE_DATA: u8 = 6;
pub const TYPE_SHUFFLING: u8 = 7;
pub const TYPE_ALIASES: u8 = 8;
pub const TYPE_VOTING: u8 = 9;
pub const TYPE_ACCOUNT_PROPERTY: u8 = 10;
pub const TYPE_COIN_EXCHANGE: u8 = 11;
pub const TYPE_LIGHT_CONTRACT: u8 = 12;

pub const SUBTYPE_PAYMENT_ORDINARY_PAYMENT: u8 = 0;

pub const TRANSACTION_VERSION: u8 = 1;

/// 交易类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransactionType {
    Payment,
    Messaging,
    ColoredCoins,
    DigitalGoods,
    AccountControl,
    MonetarySystem,
    Data,
    Shuffling,
    Aliases,
    Voting,
    AccountProperty,
    CoinExchange,
    LightContract,
    Unknown,
}

impl TransactionType {
    pub fn from_type(type_byte: u8) -> Self {
        match type_byte {
            TYPE_PAYMENT => TransactionType::Payment,
            TYPE_MESSAGING => TransactionType::Messaging,
            TYPE_COLORED_COINS => TransactionType::ColoredCoins,
            TYPE_DIGITAL_GOODS => TransactionType::DigitalGoods,
            TYPE_ACCOUNT_CONTROL => TransactionType::AccountControl,
            TYPE_MONETARY_SYSTEM => TransactionType::MonetarySystem,
            TYPE_DATA => TransactionType::Data,
            TYPE_SHUFFLING => TransactionType::Shuffling,
            TYPE_ALIASES => TransactionType::Aliases,
            TYPE_VOTING => TransactionType::Voting,
            TYPE_ACCOUNT_PROPERTY => TransactionType::AccountProperty,
            TYPE_COIN_EXCHANGE => TransactionType::CoinExchange,
            TYPE_LIGHT_CONTRACT => TransactionType::LightContract,
            _ => TransactionType::Unknown,
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            TransactionType::Payment => TYPE_PAYMENT,
            TransactionType::Messaging => TYPE_MESSAGING,
            TransactionType::ColoredCoins => TYPE_COLORED_COINS,
            TransactionType::DigitalGoods => TYPE_DIGITAL_GOODS,
            TransactionType::AccountControl => TYPE_ACCOUNT_CONTROL,
            TransactionType::MonetarySystem => TYPE_MONETARY_SYSTEM,
            TransactionType::Data => TYPE_DATA,
            TransactionType::Shuffling => TYPE_SHUFFLING,
            TransactionType::Aliases => TYPE_ALIASES,
            TransactionType::Voting => TYPE_VOTING,
            TransactionType::AccountProperty => TYPE_ACCOUNT_PROPERTY,
            TransactionType::CoinExchange => TYPE_COIN_EXCHANGE,
            TransactionType::LightContract => TYPE_LIGHT_CONTRACT,
            TransactionType::Unknown => 255,
        }
    }
}

impl From<TransactionType> for u8 {
    fn from(t: TransactionType) -> u8 {
        t.to_byte()
    }
}

/// 交易结构体
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub id: u64,
    pub version: u8,
    pub type_id: TransactionType,
    pub subtype: u8,
    pub timestamp: Timestamp,
    pub deadline: u16,
    pub sender_public_key: Hash256,
    pub sender_id: AccountId,
    pub recipient_id: Option<AccountId>,
    pub amount: Amount,
    pub fee: Amount,
    pub height: Height,
    pub block_id: BlockId,
    pub block_timestamp: Timestamp,
    pub transaction_index: u16,
    pub signature: Signature,
    pub full_hash: Hash256,
    pub referenced_transaction_full_hash: Option<Hash256>,
    pub attachment_bytes: Vec<u8>,
    pub phased: bool,
    pub has_message: bool,
    pub has_encrypted_message: bool,
    pub has_public_key_announcement: bool,
    pub has_prunable_attachment: bool,
    pub ec_block_height: Option<u32>,
    pub ec_block_id: Option<u64>,
    pub has_encrypttoself_message: bool,
    pub has_prunable_encrypted_message: bool,
}

impl Default for Transaction {
    fn default() -> Self {
        Self {
            id: 0,
            version: TRANSACTION_VERSION,
            type_id: TransactionType::Payment,
            subtype: 0,
            timestamp: 0,
            deadline: 32767,
            sender_public_key: Hash256([0u8; 32]),
            sender_id: 0,
            recipient_id: None,
            amount: 0,
            fee: 0,
            height: 0,
            block_id: 0,
            block_timestamp: 0,
            transaction_index: 0,
            signature: Signature([0u8; 64]),
            full_hash: Hash256([0u8; 32]),
            referenced_transaction_full_hash: None,
            attachment_bytes: vec![],
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
}

impl Transaction {
    pub fn new(
        type_id: TransactionType,
        sender_id: AccountId,
        recipient_id: Option<AccountId>,
        amount: Amount,
        fee: Amount,
        timestamp: Timestamp,
        deadline: u16,
    ) -> Self {
        Self {
            id: 0,
            version: TRANSACTION_VERSION,
            type_id,
            subtype: 0,
            timestamp,
            deadline,
            sender_public_key: Hash256([0u8; 32]),
            sender_id,
            recipient_id,
            amount,
            fee,
            height: 0,
            block_id: 0,
            block_timestamp: 0,
            transaction_index: 0,
            signature: Signature([0u8; 64]),
            full_hash: Hash256([0u8; 32]),
            referenced_transaction_full_hash: None,
            attachment_bytes: vec![],
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

    pub fn from_json(json: &serde_json::Value) -> Result<Self> {
        let obj = json.as_object().ok_or_else(|| {
            BlockchainError::InvalidTransaction("transaction JSON must be an object".to_string())
        })?;

        let get_u64 = |key: &str| -> Result<u64> {
            match obj.get(key) {
                Some(serde_json::Value::Number(n)) => {
                    n.as_u64().ok_or_else(|| {
                        BlockchainError::InvalidTransaction(format!("invalid {} value", key))
                    })
                }
                Some(serde_json::Value::String(s)) => {
                    s.parse::<u64>().map_err(|_| {
                        BlockchainError::InvalidTransaction(format!("invalid {} string", key))
                    })
                }
                _ => Ok(0),
            }
        };

        let get_u32 = |key: &str| -> Result<u32> {
            Ok(get_u64(key)? as u32)
        };

        let get_u16 = |key: &str| -> Result<u16> {
            Ok(get_u64(key)? as u16)
        };

        let get_u8 = |key: &str| -> Result<u8> {
            Ok(get_u64(key)? as u8)
        };

        let get_hex_bytes = |key: &str| -> Result<Vec<u8>> {
            match obj.get(key) {
                Some(serde_json::Value::String(s)) => {
                    hex::decode(s).map_err(|_| {
                        BlockchainError::InvalidTransaction(format!("invalid hex for {}", key))
                    })
                }
                _ => Ok(vec![]),
            }
        };

        let get_hash256 = |key: &str| -> Result<Hash256> {
            let bytes = get_hex_bytes(key)?;
            if bytes.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Ok(Hash256(arr))
            } else {
                Ok(Hash256([0u8; 32]))
            }
        };

        let get_hash256_opt = |key: &str| -> Result<Option<Hash256>> {
            let bytes = get_hex_bytes(key)?;
            if bytes.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Ok(Some(Hash256(arr)))
            } else {
                Ok(None)
            }
        };

        let get_signature = |key: &str| -> Result<Signature> {
            let bytes = get_hex_bytes(key)?;
            if bytes.len() == 64 {
                let mut arr = [0u8; 64];
                arr.copy_from_slice(&bytes);
                Ok(Signature(arr))
            } else {
                Ok(Signature([0u8; 64]))
            }
        };

        let type_byte = get_u8("type")?;
        let subtype = get_u8("subtype")?;
        let version = obj.get("version")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .unwrap_or(1);

        let sender_public_key = get_hash256("senderPublicKey")?;
        let sender_id = Self::public_key_to_account_id(&sender_public_key.0);

        let recipient_id = match obj.get("recipient") {
            Some(serde_json::Value::String(s)) => s.parse::<u64>().ok(),
            Some(serde_json::Value::Number(n)) => n.as_u64(),
            _ => None,
        };

        let amount = get_u64("amountNQT")?;
        let fee = get_u64("feeNQT")?;
        let timestamp = get_u32("timestamp")?;
        let deadline = get_u16("deadline")?;

        let signature = get_signature("signature")?;
        let full_hash = get_hash256("fullHash")?;
        let referenced_transaction_full_hash = get_hash256_opt("referencedTransactionFullHash")?;

        let ec_block_height = obj.get("ecBlockHeight")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);
        let ec_block_id = obj.get("ecBlockId")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok());

        let attachment_bytes = match obj.get("attachment") {
            Some(serde_json::Value::Object(att)) => {
                serde_json::to_vec(att).unwrap_or_default()
            }
            _ => vec![],
        };

        let has_message = obj.get("attachment")
            .and_then(|a| a.get("message"))
            .is_some();
        let has_encrypted_message = obj.get("attachment")
            .and_then(|a| a.get("encryptedMessage"))
            .is_some();
        let has_public_key_announcement = obj.get("attachment")
            .and_then(|a| a.get("recipientPublicKey"))
            .is_some();

        let mut tx = Self {
            id: 0,
            version,
            type_id: TransactionType::from_type(type_byte),
            subtype,
            timestamp,
            deadline,
            sender_public_key,
            sender_id,
            recipient_id,
            amount,
            fee,
            height: 0,
            block_id: 0,
            block_timestamp: 0,
            transaction_index: 0,
            signature,
            full_hash,
            referenced_transaction_full_hash,
            attachment_bytes,
            phased: false,
            has_message,
            has_encrypted_message,
            has_public_key_announcement,
            has_prunable_attachment: false,
            ec_block_height,
            ec_block_id,
            has_encrypttoself_message: false,
            has_prunable_encrypted_message: false,
        };

        tx.id = tx.calculate_id();

        Ok(tx)
    }

    pub fn public_key_to_account_id(public_key: &[u8; 32]) -> AccountId {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&public_key[..8]);
        u64::from_le_bytes(buf)
    }

    pub fn calculate_id(&self) -> u64 {
        match self.calculate_full_hash() {
            Ok(full_hash) => {
                let mut id_bytes = [0u8; 8];
                id_bytes.copy_from_slice(&full_hash.0[..8]);
                
                id_bytes.reverse();
                
                u64::from_be_bytes(id_bytes)
            }
            Err(_) => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(&self.signature.0);
                let hash = hasher.finalize();
                
                let mut id_bytes = [0u8; 8];
                id_bytes.copy_from_slice(&hash[..8]);
                u64::from_le_bytes(id_bytes)
            }
        }
    }

    pub fn calculate_full_hash(&self) -> Result<Hash256> {
        use sha2::{Digest, Sha256};
        
        let data = self.serialize_for_signing();
        
        let mut hasher = Sha256::new();
        hasher.update(&self.signature.0);
        let signature_hash = hasher.finalize();
        
        let mut hasher = Sha256::new();
        hasher.update(&data);
        hasher.update(&signature_hash);
        let hash = hasher.finalize();
        
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hash);
        
        Ok(Hash256(arr))
    }

    pub fn compute_hash(&self) -> Result<Hash256> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(&self.serialize_for_signing());
        hasher.update(&self.signature.0);
        let hash = hasher.finalize();

        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hash);
        Ok(Hash256(arr))
    }

    pub fn serialize_for_signing(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.type_id.to_byte().to_le_bytes());
        buf.extend_from_slice(&((self.subtype & 0x0f) | (self.version << 4)).to_le_bytes());
        buf.extend_from_slice(&self.timestamp.to_le_bytes());
        buf.extend_from_slice(&self.deadline.to_le_bytes());
        buf.extend_from_slice(&self.sender_public_key.0);

        if let Some(recipient) = self.recipient_id {
            buf.extend_from_slice(&recipient.to_le_bytes());
        } else {
            buf.extend_from_slice(&[0u8; 8]);
        }

        buf.extend_from_slice(&self.amount.to_le_bytes());
        buf.extend_from_slice(&self.fee.to_le_bytes());

        if let Some(ref hash) = self.referenced_transaction_full_hash {
            buf.extend_from_slice(&hash.0);
        } else {
            buf.extend_from_slice(&[0u8; 32]);
        }

        buf.extend_from_slice(&self.attachment_bytes);
        buf
    }

    pub fn verify_signature(&self) -> bool {
        use ed25519_dalek::{Verifier, Signature as EdSignature, PublicKey};

        let public_key = match PublicKey::from_bytes(&self.sender_public_key.0) {
            Ok(pk) => pk,
            Err(_) => return false,
        };

        let signature = match EdSignature::from_bytes(&self.signature.0) {
            Ok(sig) => sig,
            Err(_) => return false,
        };

        let message = self.serialize_for_signing();
        public_key.verify(&message, &signature).is_ok()
    }

    pub fn validate_basic(&self) -> Result<()> {
        if self.version < 1 {
            return Err(BlockchainError::InvalidTransaction("invalid transaction version".to_string()));
        }

        if self.id == 0 {
            return Err(BlockchainError::InvalidTransaction("invalid transaction id 0".to_string()));
        }

        if self.deadline == 0 {
            return Err(BlockchainError::InvalidTransaction("deadline cannot be zero".to_string()));
        }

        if self.sender_id == 0 {
            return Err(BlockchainError::InvalidTransaction("sender_id cannot be zero".to_string()));
        }

        Ok(())
    }

    pub fn is_payment(&self) -> bool {
        matches!(self.type_id, TransactionType::Payment)
    }

    pub fn size(&self) -> usize {
        200 + self.attachment_bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new(
            TransactionType::Payment,
            1234567890,
            Some(9876543210),
            1_000_000_000,
            100_000,
            1_704_000_000,
            32767,
        );

        assert_eq!(tx.type_id, TransactionType::Payment);
        assert_eq!(tx.amount, 1_000_000_000);
        assert_eq!(tx.sender_id, 1234567890);
        assert!(matches!(tx.recipient_id, Some(9876543210)));
    }

    #[test]
    fn test_transaction_type_conversion() {
        assert_eq!(TransactionType::from_type(TYPE_PAYMENT), TransactionType::Payment);
        assert_eq!(TransactionType::from_type(TYPE_MESSAGING), TransactionType::Messaging);
        assert_eq!(TransactionType::Payment.to_byte(), TYPE_PAYMENT);
    }
}
