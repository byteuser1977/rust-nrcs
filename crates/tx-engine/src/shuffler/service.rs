//! Shuffler Service
//!
//! 对应 Java: Shuffler.java, Shuffling.java

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;

use blockchain_types::{AccountId, Height};
use orm::models::ShufflingModel;
use crate::shuffler::{
    ShufflingStage, ShufflingEvent, ShufflingState,
    ShufflingParticipant,
};

/// Shuffler Error
#[derive(Debug, Error)]
pub enum ShufflerError {
    #[error("shuffler limit exceeded: {0}")]
    LimitExceeded(String),
    
    #[error("invalid recipient: {0}")]
    InvalidRecipient(String),
    
    #[error("invalid stage: {0}")]
    InvalidStage(String),
    
    #[error("duplicate shuffler: {0}")]
    DuplicateShuffler(String),
    
    #[error("controlled account: {0}")]
    ControlledAccount(String),
    
    #[error("database error: {0}")]
    Database(String),
    
    #[error("validation error: {0}")]
    Validation(String),
}

pub type ShufflerResult<T> = Result<T, ShufflerError>;

/// Shuffler
///
/// 对应 Java: Shuffler
#[derive(Debug, Clone)]
pub struct Shuffler {
    pub account_id: AccountId,
    pub recipient_public_key: Vec<u8>,
    pub shuffling_full_hash: Vec<u8>,
    pub failed_transaction: Option<Vec<u8>>,
    pub failure_cause: Option<String>,
}

impl Shuffler {
    pub fn new(account_id: AccountId, recipient_public_key: Vec<u8>, shuffling_full_hash: Vec<u8>) -> Self {
        Self {
            account_id,
            recipient_public_key,
            shuffling_full_hash,
            failed_transaction: None,
            failure_cause: None,
        }
    }
}

/// Shuffling Listener
pub type ShufflingListener = Box<dyn Fn(&ShufflingState) + Send + Sync>;

/// Shuffler Service
///
/// 对应 Java: Shuffler static methods + Shuffling static methods
pub struct ShufflerService {
    max_shufflers: usize,
    shufflings_map: Arc<RwLock<HashMap<String, HashMap<AccountId, Shuffler>>>>,
    expirations: Arc<RwLock<HashMap<u32, HashSet<String>>>>,
    listeners: Vec<(ShufflingEvent, ShufflingListener)>,
}

impl Default for ShufflerService {
    fn default() -> Self {
        Self::new()
    }
}

impl ShufflerService {
    pub fn new() -> Self {
        Self {
            max_shufflers: 100,
            shufflings_map: Arc::new(RwLock::new(HashMap::new())),
            expirations: Arc::new(RwLock::new(HashMap::new())),
            listeners: Vec::new(),
        }
    }

    pub fn with_max_shufflers(max_shufflers: usize) -> Self {
        Self {
            max_shufflers,
            shufflings_map: Arc::new(RwLock::new(HashMap::new())),
            expirations: Arc::new(RwLock::new(HashMap::new())),
            listeners: Vec::new(),
        }
    }

    pub fn add_listener(&mut self, listener: ShufflingListener, event_type: ShufflingEvent) {
        self.listeners.push((event_type, listener));
    }

    async fn notify_listeners(&self, shuffling: &ShufflingState, event_type: ShufflingEvent) {
        for (event, listener) in &self.listeners {
            if *event == event_type {
                listener(shuffling);
            }
        }
    }

    pub async fn add_or_get_shuffler(
        &self,
        account_id: AccountId,
        recipient_public_key: Vec<u8>,
        shuffling_full_hash: Vec<u8>,
    ) -> ShufflerResult<Shuffler> {
        let hash = hex::encode(&shuffling_full_hash);
        
        let mut map = self.shufflings_map.write().await;
        
        if map.len() >= self.max_shufflers {
            return Err(ShufflerError::LimitExceeded(
                format!("Cannot run more than {} shufflers on the same node", self.max_shufflers)
            ));
        }
        
        let shuffler_map = map.entry(hash.clone()).or_insert_with(HashMap::new);
        
        if recipient_public_key.is_empty() {
            return Ok(shuffler_map.get(&account_id).cloned().unwrap_or_else(|| {
                Shuffler::new(account_id, recipient_public_key.clone(), shuffling_full_hash.clone())
            }));
        }
        
        if let Some(existing) = shuffler_map.get(&account_id) {
            if existing.recipient_public_key != recipient_public_key {
                return Err(ShufflerError::DuplicateShuffler(
                    "A shuffler with different recipientPublicKey already started".to_string()
                ));
            }
            if existing.shuffling_full_hash != shuffling_full_hash {
                return Err(ShufflerError::DuplicateShuffler(
                    "A shuffler with different shufflingFullHash already started".to_string()
                ));
            }
            return Ok(existing.clone());
        }
        
        let shuffler = Shuffler::new(account_id, recipient_public_key, shuffling_full_hash);
        shuffler_map.insert(account_id, shuffler.clone());
        
        Ok(shuffler)
    }

    pub async fn get_all_shufflers(&self) -> Vec<Shuffler> {
        let map = self.shufflings_map.read().await;
        map.values().flat_map(|m| m.values().cloned()).collect()
    }

    pub async fn get_shuffling_shufflers(&self, shuffling_full_hash: &[u8]) -> Vec<Shuffler> {
        let hash = hex::encode(shuffling_full_hash);
        let map = self.shufflings_map.read().await;
        map.get(&hash)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default()
    }

    pub async fn get_account_shufflers(&self, account_id: AccountId) -> Vec<Shuffler> {
        let map = self.shufflings_map.read().await;
        map.values()
            .filter_map(|m| m.get(&account_id).cloned())
            .collect()
    }

    pub async fn stop_shuffler(&self, account_id: AccountId, shuffling_full_hash: &[u8]) -> Option<Shuffler> {
        let hash = hex::encode(shuffling_full_hash);
        let mut map = self.shufflings_map.write().await;
        map.get_mut(&hash).and_then(|m| m.remove(&account_id))
    }

    pub async fn stop_all_shufflers(&self) {
        let mut map = self.shufflings_map.write().await;
        map.clear();
    }

    pub async fn schedule_expiration(&self, shuffling: &ShufflingState, current_height: Height) {
        let expiration_height = current_height + 720;
        let hash = hex::encode(shuffling.get_state_hash());
        
        let mut expirations = self.expirations.write().await;
        expirations
            .entry(expiration_height)
            .or_insert_with(HashSet::new)
            .insert(hash);
    }

    pub async fn clear_expiration(&self, shuffling: &ShufflingState) {
        let hash = hex::encode(shuffling.get_state_hash());
        let mut expirations = self.expirations.write().await;
        
        for shuffling_ids in expirations.values_mut() {
            if shuffling_ids.remove(&hash) {
                return;
            }
        }
    }

    pub async fn process_block_expirations(&self, height: Height) {
        let mut expirations = self.expirations.write().await;
        let mut map = self.shufflings_map.write().await;
        
        if let Some(expired) = expirations.remove(&height) {
            for hash in expired {
                map.remove(&hash);
            }
        }
    }

    pub async fn get_participants_hash(participants: &[ShufflingParticipant]) -> Vec<u8> {
        use crypto::sha256;
        
        let mut data = Vec::new();
        for participant in participants {
            data.extend_from_slice(&participant.account_id.to_le_bytes());
        }
        
        sha256(&data).to_vec()
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_shuffling(
        &self,
        id: u64,
        issuer_id: AccountId,
        amount: i64,
        participant_count: i16,
        holding_id: Option<u64>,
        holding_type: i16,
        registration_period: i16,
    ) -> ShufflingState {
        let mut state = ShufflingState::new(id, issuer_id, amount, participant_count, holding_id, holding_type);
        state.blocks_remaining = Some(registration_period);
        state
    }

    pub async fn add_participant(
        &self,
        shuffling: &mut ShufflingState,
        participant: ShufflingParticipant,
    ) {
        shuffling.registrant_count += 1;
        
        if shuffling.registrant_count >= shuffling.participant_count {
            shuffling.stage = ShufflingStage::Processing;
            shuffling.blocks_remaining = Some(1440);
            self.notify_listeners(shuffling, ShufflingEvent::ProcessingAssigned).await;
        } else {
            shuffling.assignee_account_id = Some(participant.account_id);
        }
    }

    pub async fn update_participant_data(
        &self,
        shuffling: &mut ShufflingState,
        participant_id: AccountId,
        data: Vec<Vec<u8>>,
        next_account_id: AccountId,
    ) {
        if data.is_empty() {
            self.cancel_by(shuffling, participant_id).await;
            return;
        }
        
        shuffling.assignee_account_id = Some(next_account_id);
        shuffling.blocks_remaining = Some(1440);
        self.notify_listeners(shuffling, ShufflingEvent::ProcessingAssigned).await;
    }

    pub async fn verify_participant(&self, shuffling: &mut ShufflingState, _account_id: AccountId, verified_count: i16) {
        if verified_count >= shuffling.participant_count {
            self.distribute(shuffling).await;
        }
    }

    pub async fn cancel_by(&self, shuffling: &mut ShufflingState, participant_id: AccountId) {
        let starting_blame = shuffling.stage != ShufflingStage::Blame;
        
        if starting_blame {
            shuffling.stage = ShufflingStage::Blame;
            shuffling.assignee_account_id = Some(participant_id);
            shuffling.blocks_remaining = Some(1440 + shuffling.participant_count);
            self.notify_listeners(shuffling, ShufflingEvent::BlameStarted).await;
        }
    }

    async fn distribute(&self, shuffling: &mut ShufflingState) {
        shuffling.stage = ShufflingStage::Done;
        shuffling.blocks_remaining = Some(0);
        shuffling.assignee_account_id = None;
        self.notify_listeners(shuffling, ShufflingEvent::Done).await;
    }

    pub async fn cancel(&self, shuffling: &mut ShufflingState, blamed_account_id: AccountId) {
        shuffling.stage = ShufflingStage::Cancelled;
        shuffling.blocks_remaining = Some(0);
        shuffling.assignee_account_id = Some(blamed_account_id);
        self.notify_listeners(shuffling, ShufflingEvent::Cancelled).await;
    }
}

impl ShufflingState {
    pub fn from_model(model: &ShufflingModel) -> Self {
        Self {
            id: model.id as u64,
            holding_id: model.holding_id.map(|id| id as u64),
            holding_type: model.holding_type,
            issuer_id: model.issuer_id as u64,
            amount: model.amount,
            participant_count: model.participant_count,
            blocks_remaining: model.blocks_remaining,
            stage: ShufflingStage::from_code(model.stage).unwrap_or(ShufflingStage::Registration),
            assignee_account_id: model.assignee_account_id.map(|id| id as u64),
            registrant_count: model.registrant_count,
            recipient_public_keys: model.recipient_public_keys.as_ref().map(|s| {
                s.split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| hex::decode(s).unwrap_or_default())
                    .collect()
            }),
            height: model.height as Height,
        }
    }

    pub fn to_model(&self) -> ShufflingModel {
        ShufflingModel {
            db_id: 0,
            id: self.id as i64,
            holding_id: self.holding_id.map(|id| id as i64),
            holding_type: self.holding_type,
            issuer_id: self.issuer_id as i64,
            amount: self.amount,
            participant_count: self.participant_count,
            blocks_remaining: self.blocks_remaining,
            stage: self.stage.code(),
            assignee_account_id: self.assignee_account_id.map(|id| id as i64),
            registrant_count: self.registrant_count,
            recipient_public_keys: self.recipient_public_keys.as_ref().map(|keys| {
                keys.iter()
                    .map(hex::encode)
                    .collect::<Vec<_>>()
                    .join(",")
            }),
            height: self.height as i32,
            latest: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_shuffler() {
        let service = ShufflerService::new();
        
        let shuffler = service.add_or_get_shuffler(
            100,
            vec![1, 2, 3],
            vec![4, 5, 6],
        ).await.unwrap();
        
        assert_eq!(shuffler.account_id, 100);
        assert_eq!(shuffler.recipient_public_key, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_get_all_shufflers() {
        let service = ShufflerService::new();
        
        service.add_or_get_shuffler(100, vec![1, 2, 3], vec![4, 5, 6]).await.unwrap();
        service.add_or_get_shuffler(200, vec![7, 8, 9], vec![10, 11, 12]).await.unwrap();
        
        let all = service.get_all_shufflers().await;
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_stop_shuffler() {
        let service = ShufflerService::new();
        
        service.add_or_get_shuffler(100, vec![1, 2, 3], vec![4, 5, 6]).await.unwrap();
        
        let stopped = service.stop_shuffler(100, &[4, 5, 6]).await;
        assert!(stopped.is_some());
        
        let all = service.get_all_shufflers().await;
        assert!(all.is_empty());
    }

    #[test]
    fn test_shuffling_state_from_model() {
        let model = ShufflingModel {
            db_id: 0,
            id: 123,
            holding_id: None,
            holding_type: 0,
            issuer_id: 100,
            amount: 1000,
            participant_count: 3,
            blocks_remaining: Some(10),
            stage: 0,
            assignee_account_id: Some(100),
            registrant_count: 1,
            recipient_public_keys: None,
            height: 1000,
            latest: true,
        };
        
        let state = ShufflingState::from_model(&model);
        assert_eq!(state.id, 123);
        assert_eq!(state.issuer_id, 100);
        assert_eq!(state.stage, ShufflingStage::Registration);
    }
}
