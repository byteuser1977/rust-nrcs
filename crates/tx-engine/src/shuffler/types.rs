//! Shuffling Types
//!
//! 对应 Java: ShufflingStage, ShufflingEvent

use serde::{Deserialize, Serialize};
use blockchain_types::{AccountId, Height};

/// Shuffling Stage
///
/// 对应 Java: ShufflingStage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShufflingStage {
    Registration = 0,
    Processing = 1,
    Verification = 2,
    Blame = 3,
    Done = 4,
    Cancelled = 5,
}

impl ShufflingStage {
    pub fn from_code(code: i16) -> Option<Self> {
        match code {
            0 => Some(ShufflingStage::Registration),
            1 => Some(ShufflingStage::Processing),
            2 => Some(ShufflingStage::Verification),
            3 => Some(ShufflingStage::Blame),
            4 => Some(ShufflingStage::Done),
            5 => Some(ShufflingStage::Cancelled),
            _ => None,
        }
    }

    pub fn code(&self) -> i16 {
        *self as i16
    }

    pub fn get_hash(&self, state: &ShufflingState) -> Vec<u8> {
        use crypto::sha256;
        
        let mut data = Vec::new();
        data.extend_from_slice(&state.id.to_le_bytes());
        data.extend_from_slice(&state.stage.code().to_le_bytes());
        
        if let Some(ref keys) = state.recipient_public_keys {
            for key in keys {
                data.extend_from_slice(key);
            }
        }
        
        sha256(&data).to_vec()
    }
}

/// Shuffling Event
///
/// 对应 Java: ShufflingEvent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShufflingEvent {
    Created,
    ProcessingAssigned,
    ProcessingFinished,
    BlameStarted,
    Done,
    Cancelled,
}

/// Shuffling State
///
/// 对应 Java: Shuffling 字段
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShufflingState {
    pub id: u64,
    pub holding_id: Option<u64>,
    pub holding_type: i16,
    pub issuer_id: AccountId,
    pub amount: i64,
    pub participant_count: i16,
    pub blocks_remaining: Option<i16>,
    pub stage: ShufflingStage,
    pub assignee_account_id: Option<AccountId>,
    pub registrant_count: i16,
    pub recipient_public_keys: Option<Vec<Vec<u8>>>,
    pub height: Height,
}

impl ShufflingState {
    pub fn new(
        id: u64,
        issuer_id: AccountId,
        amount: i64,
        participant_count: i16,
        holding_id: Option<u64>,
        holding_type: i16,
    ) -> Self {
        Self {
            id,
            holding_id,
            holding_type,
            issuer_id,
            amount,
            participant_count,
            blocks_remaining: None,
            stage: ShufflingStage::Registration,
            assignee_account_id: Some(issuer_id),
            registrant_count: 1,
            recipient_public_keys: None,
            height: 0,
        }
    }

    pub fn is_full(&self, block_payload_length: usize, max_payload_length: usize) -> bool {
        let transaction_size = match self.stage {
            ShufflingStage::Registration => 1 + 32,
            _ => 16384,
        };
        block_payload_length + transaction_size > max_payload_length
    }

    pub fn get_state_hash(&self) -> Vec<u8> {
        self.stage.get_hash(self)
    }

    pub fn is_active(&self) -> bool {
        self.blocks_remaining.is_some()
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.stage, ShufflingStage::Done | ShufflingStage::Cancelled)
    }
}

/// Shuffling Creation Attachment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShufflingCreationAttachment {
    pub holding_id: Option<u64>,
    pub holding_type: i16,
    pub amount: i64,
    pub participant_count: i16,
    pub registration_period: i16,
}

/// Shuffling Registration Attachment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShufflingRegistrationAttachment {
    pub shuffling_full_hash: Vec<u8>,
}

/// Shuffling Processing Attachment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShufflingProcessingAttachment {
    pub shuffling_id: u64,
    pub data: Vec<Vec<u8>>,
    pub shuffling_state_hash: Vec<u8>,
}

/// Shuffling Recipients Attachment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShufflingRecipientsAttachment {
    pub shuffling_id: u64,
    pub recipient_public_keys: Vec<Vec<u8>>,
    pub shuffling_state_hash: Vec<u8>,
}

/// Shuffling Verification Attachment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShufflingVerificationAttachment {
    pub shuffling_id: u64,
    pub shuffling_state_hash: Vec<u8>,
}

/// Shuffling Cancellation Attachment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShufflingCancellationAttachment {
    pub shuffling_id: u64,
    pub blame_data: Vec<Vec<u8>>,
    pub key_seeds: Vec<Vec<u8>>,
    pub shuffling_state_hash: Vec<u8>,
    pub cancelling_account_id: AccountId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shuffling_stage_from_code() {
        assert_eq!(ShufflingStage::from_code(0), Some(ShufflingStage::Registration));
        assert_eq!(ShufflingStage::from_code(1), Some(ShufflingStage::Processing));
        assert_eq!(ShufflingStage::from_code(5), Some(ShufflingStage::Cancelled));
        assert_eq!(ShufflingStage::from_code(99), None);
    }

    #[test]
    fn test_shuffling_state_is_active() {
        let mut state = ShufflingState::new(1, 100, 1000, 3, None, 0);
        assert!(!state.is_active());
        
        state.blocks_remaining = Some(10);
        assert!(state.is_active());
    }

    #[test]
    fn test_shuffling_state_is_finished() {
        let mut state = ShufflingState::new(1, 100, 1000, 3, None, 0);
        assert!(!state.is_finished());
        
        state.stage = ShufflingStage::Done;
        assert!(state.is_finished());
        
        state.stage = ShufflingStage::Cancelled;
        assert!(state.is_finished());
    }
}
