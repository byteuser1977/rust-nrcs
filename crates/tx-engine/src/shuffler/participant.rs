//! Shuffling Participant
//!
//! 对应 Java: ShufflingParticipant.java

use serde::{Deserialize, Serialize};
use blockchain_types::{AccountId, Height};

/// Participant State
///
/// 对应 Java: ShufflingParticipantState
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParticipantState {
    Registered = 0,
    Processed = 1,
    Verified = 2,
    Cancelled = 3,
}

impl ParticipantState {
    pub fn from_code(code: i16) -> Option<Self> {
        match code {
            0 => Some(ParticipantState::Registered),
            1 => Some(ParticipantState::Processed),
            2 => Some(ParticipantState::Verified),
            3 => Some(ParticipantState::Cancelled),
            _ => None,
        }
    }

    pub fn code(&self) -> i16 {
        *self as i16
    }
}

/// Shuffling Participant
///
/// 对应 Java: ShufflingParticipant
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShufflingParticipant {
    pub shuffling_id: u64,
    pub account_id: AccountId,
    pub next_account_id: Option<AccountId>,
    pub participant_index: i16,
    pub state: ParticipantState,
    pub blame_data: Option<Vec<Vec<u8>>>,
    pub key_seeds: Option<Vec<Vec<u8>>>,
    pub data: Option<Vec<Vec<u8>>>,
    pub data_transaction_full_hash: Option<Vec<u8>>,
    pub height: Height,
}

impl ShufflingParticipant {
    pub fn new(shuffling_id: u64, account_id: AccountId, participant_index: i16) -> Self {
        Self {
            shuffling_id,
            account_id,
            next_account_id: None,
            participant_index,
            state: ParticipantState::Registered,
            blame_data: None,
            key_seeds: None,
            data: None,
            data_transaction_full_hash: None,
            height: 0,
        }
    }

    pub fn set_next_account_id(&mut self, next_account_id: AccountId) {
        self.next_account_id = Some(next_account_id);
    }

    pub fn set_data(&mut self, data: Vec<Vec<u8>>, timestamp: i32) {
        self.data = Some(data);
        self.data_transaction_full_hash = Some(timestamp.to_le_bytes().to_vec());
    }

    pub fn set_processed(&mut self, full_hash: Vec<u8>) {
        self.state = ParticipantState::Processed;
        self.data_transaction_full_hash = Some(full_hash);
    }

    pub fn verify(&mut self) {
        self.state = ParticipantState::Verified;
    }

    pub fn cancel(&mut self, blame_data: Vec<Vec<u8>>, key_seeds: Vec<Vec<u8>>) {
        self.state = ParticipantState::Cancelled;
        self.blame_data = Some(blame_data);
        self.key_seeds = Some(key_seeds);
    }

    pub fn is_verified(&self) -> bool {
        self.state == ParticipantState::Verified
    }

    pub fn is_cancelled(&self) -> bool {
        self.state == ParticipantState::Cancelled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_participant_state_from_code() {
        assert_eq!(ParticipantState::from_code(0), Some(ParticipantState::Registered));
        assert_eq!(ParticipantState::from_code(1), Some(ParticipantState::Processed));
        assert_eq!(ParticipantState::from_code(2), Some(ParticipantState::Verified));
        assert_eq!(ParticipantState::from_code(3), Some(ParticipantState::Cancelled));
        assert_eq!(ParticipantState::from_code(99), None);
    }

    #[test]
    fn test_participant_new() {
        let participant = ShufflingParticipant::new(1, 100, 0);
        assert_eq!(participant.shuffling_id, 1);
        assert_eq!(participant.account_id, 100);
        assert_eq!(participant.participant_index, 0);
        assert_eq!(participant.state, ParticipantState::Registered);
    }

    #[test]
    fn test_participant_verify() {
        let mut participant = ShufflingParticipant::new(1, 100, 0);
        assert!(!participant.is_verified());
        
        participant.verify();
        assert!(participant.is_verified());
    }
}
