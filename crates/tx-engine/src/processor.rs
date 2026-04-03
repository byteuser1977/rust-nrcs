//! Transaction Processor
//!
//! 核心交易处理逻辑：
//! - 验证交易签名、余额、nonce、逻辑约束
//! - 应用交易到状态（修改账户余额、资产、合约存储）
//! - 返回交易收据

use async_trait::async_trait;
use std::sync::Arc;

use blockchain_types::*;
use orm::{AccountRepository, AccountAssetRepository, TransactionRepository, TransactionReceiptRepository, RepositoryError};
use thiserror::Error;

use crate::types::{TxReceiptInfo, TxStatus};

/// 交易处理器错误
#[derive(Debug, Error)]
pub enum ProcessorError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: Amount, need: Amount },

    #[error("account not found: {0}")]
    AccountNotFound(AccountId),

    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("blockchain error: {0}")]
    Blockchain(#[from] BlockchainError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type ProcessorResult<T> = Result<T, ProcessorError>;

/// TransactionProcessor trait
///
/// 负责交易的验证、执行和状态更新
#[async_trait]
pub trait TransactionProcessor: Send + Sync {
    /// 验证交易的有效性（签名、余额、nonce 等）
    async fn validate(&self, tx: &Transaction) -> ProcessorResult<()>;

    /// 应用交易到状态（修改账户余额、资产等）
    /// 不提交事务，需要调用者管理事务边界
    async fn apply(&self, tx: &Transaction) -> ProcessorResult<()>;

    /// 执行交易并返回收据
    /// 包含 apply 操作，并记录收据（gas 消耗、日志）
    async fn execute(&self, tx: &Transaction) -> ProcessorResult<TxReceiptInfo>;

    /// 批量验证交易
    async fn validate_batch(&self, txs: &[Transaction]) -> ProcessorResult<()> {
        for tx in txs {
            self.validate(tx).await?;
        }
        Ok(())
    }

    /// 批量执行交易
    async fn execute_batch(&self, txs: &[Transaction]) -> ProcessorResult<Vec<TxReceiptInfo>> {
        let mut receipts = Vec::new();
        for tx in txs {
            let receipt = self.execute(tx).await?;
            receipts.push(receipt);
        }
        Ok(receipts)
    }
}

/// 基于数据库的交易处理器实现
pub struct DatabaseTransactionProcessor {
    account_repo: Arc<dyn AccountRepository>,
    account_asset_repo: Arc<dyn AccountAssetRepository>,
    tx_repo: Arc<dyn TransactionRepository>,
    receipt_repo: Arc<dyn TransactionReceiptRepository>,
}

impl DatabaseTransactionProcessor {
    pub fn new(
        account_repo: Arc<dyn AccountRepository>,
        account_asset_repo: Arc<dyn AccountAssetRepository>,
        tx_repo: Arc<dyn TransactionRepository>,
        receipt_repo: Arc<dyn TransactionReceiptRepository>,
    ) -> Self {
        Self {
            account_repo,
            account_asset_repo,
            tx_repo,
            receipt_repo,
        }
    }

    /// 获取账户并提升错误类型
    async fn get_account(&self, account_id: AccountId) -> ProcessorResult<Account> {
        let model = self.account_repo
            .find_by_account_id(account_id as i64)
            .await?
            .ok_or_else(|| ProcessorError::AccountNotFound(account_id))?;
        model.to_domain().map_err(Into::into)
    }

    /// 更新账户余额
    async fn update_account_balance(&self, account_id: AccountId, balance: Amount, unconfirmed: Amount) -> ProcessorResult<()> {
        self.account_repo
            .update_balance(account_id as i64, balance as i64, unconfirmed as i64)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl TransactionProcessor for DatabaseTransactionProcessor {
    async fn validate(&self, tx: &Transaction) -> ProcessorResult<()> {
        // 1. 基础验证
        tx.validate_basic()?;

        // 2. 验证签名（需要发送者公钥）
        // TODO: 从 account_repo 获取发送者公钥
        // let account = self.get_account(tx.sender_id).await?;
        // if let Some(pub_key) = account.public_key {
        //     tx.verify_signature(&pub_key)?;
        // } else {
        //     return Err(ProcessorError::Validation("sender has no public key".to_string()));
        // }

        // 3. 检查发送者余额
        match tx.type_id {
            TransactionType::Payment | TransactionType::AssetTransfer => {
                let account = self.get_account(tx.sender_id).await?;
                let required = tx.amount + tx.fee;
                if !account.has_balance(required) {
                    return Err(ProcessorError::InsufficientBalance {
                        have: account.effective_balance(),
                        need: required,
                    });
                }
            }
            TransactionType::ContractInvocation | TransactionType::ContractDeployment => {
                let account = self.get_account(tx.sender_id).await?;
                if !account.has_balance(tx.fee) {
                    return Err(ProcessorError::InsufficientBalance {
                        have: account.effective_balance(),
                        need: tx.fee,
                    });
                }
            }
            _ => {} // 其他类型简化处理
        }

        // 4. 检查 deadline（是否过期）
        // let current_height = ... 需从 chain 状态获取
        // if tx.deadline > 0 && current_height > tx.deadline {
        //     return Err(ProcessorError::Validation("transaction deadline expired".to_string()));
        // }

        Ok(())
    }

    async fn apply(&self, tx: &Transaction) -> ProcessorResult<()> {
        // 所有数据库操作应该在一个事务中
        // TODO: 实现事务管理

        match tx.type_id {
            TransactionType::Payment => {
                // 普通转账：sender -> recipient
                let sender_id = tx.sender_id;
                let recipient_id = tx.recipient_id.ok_or_else(|| ProcessorError::Validation("payment requires recipient".to_string()))?;

                let sender = self.get_account(sender_id).await?;
                let total = tx.amount + tx.fee;

                if !sender.has_balance(total) {
                    return Err(ProcessorError::InsufficientBalance {
                        have: sender.effective_balance(),
                        need: total,
                    });
                }

                // 扣款
                let new_balance = sender.balance.saturating_sub(total);
                let new_unconfirmed = sender.unconfirmed_balance.saturating_sub(total);
                self.update_account_balance(sender_id, new_balance, new_unconfirmed).await?;

                // 增加收款方余额（unconfirmed）
                let recipient = self.get_account(recipient_id).await?;
                self.update_account_balance(recipient_id, recipient.balance, recipient.unconfirmed_balance.saturating_add(tx.amount)).await?;
            }

            TransactionType::AssetTransfer => {
                // 资产转移
                let sender_id = tx.sender_id;
                let recipient_id = tx.recipient_id.ok_or_else(|| ProcessorError::Validation("asset transfer requires recipient".to_string()))?;

                // 解析 attachment 获取 asset_id 和 quantity（简化）
                // TODO: 实现 asset transfer 的完整逻辑
            }

            TransactionType::ContractInvocation => {
                // 合约调用：扣除 gas fee，执行合约代码
                let sender_id = tx.sender_id;
                let account = self.get_account(sender_id).await?;
                let new_balance = account.balance.saturating_sub(tx.fee);
                let new_unconfirmed = account.unconfirmed_balance.saturating_sub(tx.fee);
                self.update_account_balance(sender_id, new_balance, new_unconfirmed).await?;

                // TODO: 调用合约引擎执行代码
                // let contract_addr = ... from attachment
                // let result = contract_engine.call(contract_addr, method, args)?;
            }

            TransactionType::ContractDeployment => {
                // 合约部署：扣除 fee，创建新合约记录
                let sender_id = tx.sender_id;
                let account = self.get_account(sender_id).await?;
                // TODO: 部署合约
            }

            _ => {
                return Err(ProcessorError::Validation(format!("unsupported transaction type: {:?}", tx.type_id)));
            }
        }

        Ok(())
    }

    async fn execute(&self, tx: &Transaction) -> ProcessorResult<TxReceiptInfo> {
        use chrono::Utc;

        // 1. 记录执行开始时间
        let start_time = Utc::now().timestamp() as u64;

        // 2. 执行交易
        self.apply(tx).await?;

        // 3. 计算 gas 消耗（简化固定值）
        let gas_used = match tx.type_id {
            TransactionType::Payment => 100_000,  // 10^5
            TransactionType::AssetTransfer => 200_000,
            TransactionType::ContractInvocation => {
                // TODO: 实际计算 gas 消耗
                500_000
            }
            TransactionType::ContractDeployment => 1_000_000,
            _ => 100_000,
        };

        // 4. 创建收据记录
        let receipt = TxReceipt {
            transaction_id: 0, // TODO: 从交易 full_hash 生成 ID
            status: 1, // success
            gas_used,
            logs: "[]".to_string(), // 暂无日志
            contract_address: None, // 仅合约调用有
            executed_at: Utc::now().timestamp() as Timestamp,
        };

        // 5. 保存到数据库
        // TODO: 实现 repository 插入

        // 6. 返回收据信息
        let receipt_info = TxReceiptInfo {
            transaction_id: 0,
            status: TxStatus::Success,
            block_height: None, // 打包后才填充
            gas_used,
            logs: vec![],
            contract_address: None,
            executed_at: receipt.executed_at,
        };

        Ok(receipt_info)
    }
}