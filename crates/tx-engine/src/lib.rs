//! Transaction Engine
//!
//! 交易处理引擎，负责：
//! - 交易验证
//! - 交易执行
//! - 未确认交易池管理
//! - 交易广播
//! - 交易附件打包/解包

pub mod types;
pub mod attachment;
pub mod tx_types;
pub mod validation;
pub mod processor;
pub mod mempool;
pub mod broadcast;
pub mod trade_matcher;

pub use types::{TxPriority, TxReceiptInfo, TxStatus};
pub use attachment::{
    Attachment, AttachmentError, AttachmentResult,
    PaymentAttachment, AssetTransferAttachment, AssetIssuanceAttachment,
    ContractDeploymentAttachment, ContractInvocationAttachment,
    LeaseAttachment, SetPropertyAttachment, MessageAttachment,
    TransactionDbModel,
};
pub use tx_types::{
    TxTypeHandler, TxTypeRegistry, TxTypeError, TxTypeResult,
    TxExecutionContext,
    PaymentHandler, ColoredCoinsHandler, LightContractHandler,
    MessagingHandler, DataHandler,
};
pub use validation::{
    TransactionValidator, TxValidationError, TxValidationResult,
    PaymentValidator, AssetTransferValidator, LeaseValidator, ContractValidator,
};
pub use processor::{TransactionProcessor, DatabaseTransactionProcessor, ProcessorError, ProcessorResult};
pub use mempool::{Mempool, MempoolConfig, MempoolError, MempoolStats};
pub use broadcast::{
    TxBroadcaster, BroadcastConfig, BroadcastError, BroadcastResult,
    BroadcastedTx, TxConfirmationTracker,
};

pub mod prelude {
    pub use crate::{
        TxPriority,
        TxReceiptInfo,
        TxStatus,
        Attachment,
        PaymentAttachment,
        AssetTransferAttachment,
        AssetIssuanceAttachment,
        ContractDeploymentAttachment,
        ContractInvocationAttachment,
        LeaseAttachment,
        SetPropertyAttachment,
        MessageAttachment,
        TransactionDbModel,
        TxTypeHandler,
        TxTypeRegistry,
        TxExecutionContext,
        TransactionValidator,
        TransactionProcessor,
        DatabaseTransactionProcessor,
        Mempool,
        MempoolConfig,
        TxBroadcaster,
        BroadcastConfig,
    };
}