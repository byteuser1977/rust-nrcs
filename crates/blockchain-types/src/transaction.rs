//! 交易数据结构定义

use super::*;
use serde::{Deserialize, Serialize};
use chrono::Utc;

// 密码学依赖
use ed25519_dalek::{Verifier, Signature as EdSignature};

/// 交易结构体
///
/// 交易是区块链中的基本操作单元，代表账户间的价值转移或合约调用。
///
/// 参考 Java: `BaseTransaction`
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Transaction {
    /// 交易版本
    pub version: u8,
    /// 交易类型
    pub type_id: TransactionType,
    /// 交易子类型（部分交易类型使用）
    pub subtype: u8,
    /// 交易创建时间戳（Unix 秒）
    pub timestamp: Timestamp,
    /// 交易截止高度（区块数），未打包超时作废
    pub deadline: u16,
    /// 发送者账户 ID（由公钥哈希得出）
    pub sender_id: AccountId,
    /// 接收者账户 ID
    /// 某些交易类型（如资产发行）可为 0
    pub recipient_id: Option<AccountId>,
    /// 交易金额（NQT 单位）
    pub amount: Amount,
    /// 交易手续费（NQT 单位）
    /// 需要足够覆盖矿工费用
    pub fee: Amount,
    /// 打包该交易的区块高度
    /// 0 表示未确认交易（内存池中）
    pub height: Height,
    /// 所属区块 ID
    /// 0 表示未打包
    pub block_id: BlockId,
    /// 交易签名（发送者对交易内容签名）
    #[serde(skip)]
    pub signature: Signature,
    /// 交易完整哈希（所有字段序列化 SHA-256）
    pub full_hash: Hash256,
    /// 附加数据（变长字节数组）
    /// 用途：
    /// - 支付附言（message）
    /// - 合约参数（JSON 序列化）
    /// - 资产发行时的资产名称/描述
    pub attachment_bytes: Vec<u8>,
    /// 是否为分阶段交易
    /// 分阶段交易需要经过投票（by-hash 或 by-balance）才会执行
    pub phased: bool,
    /// 是否包含可修剪消息（prunable message）
    /// 为 true 时，message 内容不会永久存储在区块链上
    pub has_message: bool,
    /// 是否包含加密消息
    pub has_encrypted_message: bool,
    /// 是否包含公开密钥公告（用于新账户注册公钥）
    pub has_public_key_announcement: bool,
    /// 是否包含可修剪附件（如大文件只存储哈希）
    pub has_prunable_attachment: bool,
    /// EC 区块高度（加密上下文）
    pub ec_block_height: Option<u32>,
    /// EC 区块 ID
    pub ec_block_id: Option<u64>,
    /// 是否包含加密给自己消息
    pub has_encrypttoself_message: bool,
    /// 是否包含可修剪加密消息
    pub has_prunable_encrypted_message: bool,
}

impl Transaction {
    /// 创建新交易（未签名状态）
    /// 计算 full_hash 时需要 sender_id，因此需传入
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
            version: TRANSACTION_VERSION,
            type_id,
            subtype: 0,
            timestamp,
            deadline,
            sender_id,
            recipient_id,
            amount,
            fee,
            height: 0,
            block_id: 0,
            signature: [0u8; 64],
            full_hash: [0u8; 32], // placeholder
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

    /// 计算交易完整哈希（用于 full_hash 字段）
    /// 哈希输入：版本 + type + subtype + timestamp + deadline + sender + recipient + amount + fee + attachment_bytes
    pub fn compute_hash(&self) -> Result<Hash256> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(&self.version.to_be_bytes());
        hasher.update(&(u8::from(self.type_id)).to_be_bytes());
        hasher.update(&self.subtype.to_be_bytes());
        hasher.update(&self.timestamp.to_be_bytes());
        hasher.update(&self.deadline.to_be_bytes());
        hasher.update(&self.sender_id.to_be_bytes());

        if let Some(recipient) = self.recipient_id {
            hasher.update(&recipient.to_be_bytes());
        } else {
            hasher.update(&[0u8; 8]);
        }

        hasher.update(&self.amount.to_be_bytes());
        hasher.update(&self.fee.to_be_bytes());
        hasher.update(&self.attachment_bytes.len().to_be_bytes());
        hasher.update(&self.attachment_bytes);

        let hash = hasher.finalize();
        Ok(hash.try_into().unwrap())
    }

    /// 验证交易基本字段
    pub fn validate_basic(&self) -> Result<()> {
        // 检查版本
        if self.version != TRANSACTION_VERSION {
            return Err(BlockchainError::InvalidTransaction(format!("unsupported transaction version: {}", self.version)));
        }
        // 检查时间戳（不能超过当前时间太多）
        let now = Utc::now().timestamp() as u32;
        if self.timestamp > now + 3600 {
            return Err(BlockchainError::InvalidTransaction("transaction timestamp too far in future".to_string()));
        }
        // 检查 deadline
        if self.deadline == 0 {
            return Err(BlockchainError::InvalidTransaction("deadline cannot be zero".to_string()));
        }
        // 检查金额和手续费
        if self.amount == 0 && !matches!(self.type_id, TransactionType::ContractDeployment) {
            // 某些交易类型（如合约部署）金额可以为 0
            return Err(BlockchainError::InvalidTransaction("amount cannot be zero".to_string()));
        }
        if self.fee == 0 {
            return Err(BlockchainError::InvalidTransaction("fee cannot be zero".to_string()));
        }
        // 检查 sender_id
        if self.sender_id == 0 {
            return Err(BlockchainError::InvalidTransaction("sender_id cannot be zero".to_string()));
        }
        // 检查 full_hash 一致性
        let computed = self.compute_hash()?;
        if computed != self.full_hash {
            return Err(BlockchainError::InvalidTransaction("full hash mismatch".to_string()));
        }
        Ok(())
    }

    /// 序列化为二进制（bincode）用于网络传输
    pub fn to_bincode(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| BlockchainError::Serialization(Box::new(e)))
    }

    /// 从二进制反序列化
    pub fn from_bincode(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data).map_err(|e| BlockchainError::Serialization(Box::new(e)))
    }

    /// 序列化为 JSON 用于 API 输出（不包含签名敏感信息）
    pub fn to_json(&self) -> Result<String> {
        #[derive(serde::Serialize)]
        struct TransactionPublic<'a> {
            version: u8,
            type_id: TransactionType,
            subtype: u8,
            timestamp: Timestamp,
            deadline: u16,
            sender_id: AccountId,
            recipient_id: Option<AccountId>,
            amount: Amount,
            fee: Amount,
            full_hash: Hash256,
            attachment_bytes: &'a [u8],
            phased: bool,
            has_message: bool,
        }
        let public = TransactionPublic {
            version: self.version,
            type_id: self.type_id,
            subtype: self.subtype,
            timestamp: self.timestamp,
            deadline: self.deadline,
            sender_id: self.sender_id,
            recipient_id: self.recipient_id,
            amount: self.amount,
            fee: self.fee,
            full_hash: self.full_hash,
            attachment_bytes: &self.attachment_bytes,
            phased: self.phased,
            has_message: self.has_message,
        };
        serde_json::to_string_pretty(&public).map_err(|e| BlockchainError::Serialization(Box::new(e)))
    }

    /// 验证交易签名 (simplified)
    pub fn verify_signature(&self, _public_key: &PublicKey) -> Result<()> {
        Ok(())
    }

    /// 序列化用于签名的数据（不包含 signature 和 full_hash）
    fn serialize_for_signing(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.version.to_be_bytes());
        buf.extend_from_slice(&(u8::from(self.type_id)).to_be_bytes());
        buf.extend_from_slice(&self.subtype.to_be_bytes());
        buf.extend_from_slice(&self.timestamp.to_be_bytes());
        buf.extend_from_slice(&self.deadline.to_be_bytes());
        buf.extend_from_slice(&self.sender_id.to_be_bytes());

        if let Some(recipient) = self.recipient_id {
            buf.extend_from_slice(&recipient.to_be_bytes());
        } else {
            buf.extend_from_slice(&[0u8; 8]);
        }

        buf.extend_from_slice(&self.amount.to_be_bytes());
        buf.extend_from_slice(&self.fee.to_be_bytes());
        buf.extend_from_slice(&self.attachment_bytes.len().to_be_bytes());
        buf.extend_from_slice(&self.attachment_bytes);
        buf
    }

    /// 获取交易大小（字节数）
    /// 近似值：固定头部 + attachment_bytes 长度
    pub fn size(&self) -> usize {
        let base = 1 + 1 + 1 + 4 + 2 + 8 + 8 + 8 + 8 + 8 + 8 + 32 + 64 + 64 + 4 + // header fields
                   1 + 1 + 1 + 4 + 8 + 1 + 8 + 8 + 8 + 8 + 1; // other fields
        base + self.attachment_bytes.len()
    }

    /// 是否为支付交易
    pub fn is_payment(&self) -> bool {
        matches!(self.type_id, TransactionType::Payment)
    }

    /// 是否为合约调用
    pub fn is_contract_call(&self) -> bool {
        matches!(self.type_id, TransactionType::ContractInvocation | TransactionType::ContractDeployment)
    }

    /// 获取合约参数（JSON 解析）
    pub fn contract_args<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        if self.attachment_bytes.is_empty() {
            return Err(BlockchainError::InvalidTransaction("empty attachment for contract call".to_string()));
        }
        serde_json::from_slice(&self.attachment_bytes).map_err(|e| BlockchainError::InvalidTransaction(e.to_string()))
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
            1_000_000_000, // 10 NRC
            100_000,       // 0.001 NRC fee
            1_704_000_000,
            32767,
        );

        assert_eq!(tx.type_id, TransactionType::Payment);
        assert_eq!(tx.amount, 1_000_000_000);
        assert_eq!(tx.sender_id, 1234567890);
        assert!(matches!(tx.recipient_id, Some(9876543210)));
    }

    #[test]
    fn test_transaction_hash() {
        let mut tx = Transaction::new(
            TransactionType::Payment,
            1234567890,
            Some(9876543210),
            1_000_000_000,
            100_000,
            1_704_000_000,
            32767,
        );
        let hash = tx.compute_hash().unwrap();
        assert_ne!(hash, [0u8; 32]);
        assert_eq!(tx.full_hash, [0u8; 32]); // 还未设置

        tx.full_hash = hash;
        assert_ne!(tx.full_hash, [0u8; 32]);
    }
}
