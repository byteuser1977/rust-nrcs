//! Shuffler 外部测试
//!
//! 测试 ShufflerService、ShufflingState、ShufflingStage 等洗牌交易功能

use tx_engine::shuffler::{
    ShufflerService, ShufflerError, ShufflingStage, ShufflingState,
};
use orm::models::ShufflingModel;

#[test]
fn test_shuffling_stage_from_code() {
    assert_eq!(ShufflingStage::from_code(0), Some(ShufflingStage::Registration));
    assert_eq!(ShufflingStage::from_code(1), Some(ShufflingStage::Processing));
    assert_eq!(ShufflingStage::from_code(2), Some(ShufflingStage::Verification));
    assert_eq!(ShufflingStage::from_code(3), Some(ShufflingStage::Blame));
    assert_eq!(ShufflingStage::from_code(4), Some(ShufflingStage::Done));
    assert_eq!(ShufflingStage::from_code(5), Some(ShufflingStage::Cancelled));
    assert_eq!(ShufflingStage::from_code(99), None);
}

#[test]
fn test_shuffling_stage_code() {
    assert_eq!(ShufflingStage::Registration.code(), 0);
    assert_eq!(ShufflingStage::Processing.code(), 1);
    assert_eq!(ShufflingStage::Verification.code(), 2);
    assert_eq!(ShufflingStage::Blame.code(), 3);
    assert_eq!(ShufflingStage::Done.code(), 4);
    assert_eq!(ShufflingStage::Cancelled.code(), 5);
}

#[test]
fn test_shuffling_stage_code_roundtrip() {
    for stage in [
        ShufflingStage::Registration,
        ShufflingStage::Processing,
        ShufflingStage::Verification,
        ShufflingStage::Blame,
        ShufflingStage::Done,
        ShufflingStage::Cancelled,
    ] {
        assert_eq!(ShufflingStage::from_code(stage.code()), Some(stage));
    }
}

#[test]
fn test_shuffling_state_new() {
    let state = ShufflingState::new(1, 100, 1000, 3, None, 0);
    assert_eq!(state.id, 1);
    assert_eq!(state.issuer_id, 100);
    assert_eq!(state.amount, 1000);
    assert_eq!(state.participant_count, 3);
    assert_eq!(state.stage, ShufflingStage::Registration);
    assert!(state.blocks_remaining.is_none());
    assert_eq!(state.registrant_count, 1);
    assert_eq!(state.assignee_account_id, Some(100));
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

    state.stage = ShufflingStage::Processing;
    assert!(!state.is_finished());
}

#[test]
fn test_shuffling_state_is_full() {
    let state = ShufflingState::new(1, 100, 1000, 3, None, 0);

    assert!(!state.is_full(0, 1000), "Registration is small");
    assert!(state.is_full(0, 10), "Very small max payload");
}

#[test]
fn test_shuffling_state_get_state_hash() {
    let state = ShufflingState::new(1, 100, 1000, 3, None, 0);
    let hash = state.get_state_hash();
    assert!(!hash.is_empty());
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
    assert_eq!(state.amount, 1000);
}

#[test]
fn test_shuffling_state_to_model() {
    let state = ShufflingState::new(123, 100, 1000, 3, None, 0);
    let model = state.to_model();

    assert_eq!(model.id, 123);
    assert_eq!(model.issuer_id, 100);
    assert_eq!(model.amount, 1000);
    assert_eq!(model.stage, 0);
    assert!(model.latest);
}

#[test]
fn test_shuffling_state_roundtrip() {
    let state = ShufflingState::new(456, 200, 5000, 5, Some(999), 1);
    let model = state.to_model();
    let restored = ShufflingState::from_model(&model);

    assert_eq!(restored.id, state.id);
    assert_eq!(restored.issuer_id, state.issuer_id);
    assert_eq!(restored.amount, state.amount);
    assert_eq!(restored.participant_count, state.participant_count);
}

#[tokio::test]
async fn test_shuffler_service_add() {
    let service = ShufflerService::new();

    let shuffler = service.add_or_get_shuffler(100, vec![1, 2, 3], vec![4, 5, 6]).await.unwrap();
    assert_eq!(shuffler.account_id, 100);
    assert_eq!(shuffler.recipient_public_key, vec![1, 2, 3]);
}

#[tokio::test]
async fn test_shuffler_service_get_all() {
    let service = ShufflerService::new();

    service.add_or_get_shuffler(100, vec![1, 2, 3], vec![4, 5, 6]).await.unwrap();
    service.add_or_get_shuffler(200, vec![7, 8, 9], vec![10, 11, 12]).await.unwrap();

    let all = service.get_all_shufflers().await;
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn test_shuffler_service_get_same_account_different_hash() {
    let service = ShufflerService::new();

    service.add_or_get_shuffler(100, vec![1, 2, 3], vec![4, 5, 6]).await.unwrap();
    service.add_or_get_shuffler(100, vec![7, 8, 9], vec![10, 11, 12]).await.unwrap();

    let all = service.get_all_shufflers().await;
    assert_eq!(all.len(), 2, "Different hash should be separate shufflers");
}

#[tokio::test]
async fn test_shuffler_service_duplicate() {
    let service = ShufflerService::new();

    service.add_or_get_shuffler(100, vec![1, 2, 3], vec![4, 5, 6]).await.unwrap();
    let result = service.add_or_get_shuffler(100, vec![7, 8, 9], vec![4, 5, 6]).await;
    assert!(matches!(result, Err(ShufflerError::DuplicateShuffler(_))));
}

#[tokio::test]
async fn test_shuffler_service_same_shuffler_twice() {
    let service = ShufflerService::new();

    let s1 = service.add_or_get_shuffler(100, vec![1, 2, 3], vec![4, 5, 6]).await.unwrap();
    let s2 = service.add_or_get_shuffler(100, vec![1, 2, 3], vec![4, 5, 6]).await.unwrap();

    assert_eq!(s1.account_id, s2.account_id);
    assert_eq!(service.get_all_shufflers().await.len(), 1);
}

#[tokio::test]
async fn test_shuffler_service_empty_recipient() {
    let service = ShufflerService::new();

    let shuffler = service.add_or_get_shuffler(100, vec![], vec![4, 5, 6]).await.unwrap();
    assert_eq!(shuffler.account_id, 100);
    assert!(shuffler.recipient_public_key.is_empty());
}

#[tokio::test]
async fn test_shuffler_service_stop() {
    let service = ShufflerService::new();

    service.add_or_get_shuffler(100, vec![1, 2, 3], vec![4, 5, 6]).await.unwrap();

    let stopped = service.stop_shuffler(100, &[4, 5, 6]).await;
    assert!(stopped.is_some());

    let all = service.get_all_shufflers().await;
    assert!(all.is_empty());
}

#[tokio::test]
async fn test_shuffler_service_stop_nonexistent() {
    let service = ShufflerService::new();

    let stopped = service.stop_shuffler(999, &[1, 2, 3]).await;
    assert!(stopped.is_none());
}

#[tokio::test]
async fn test_shuffler_service_stop_all() {
    let service = ShufflerService::new();

    service.add_or_get_shuffler(100, vec![], vec![4, 5, 6]).await.unwrap();
    service.add_or_get_shuffler(200, vec![], vec![7, 8, 9]).await.unwrap();

    service.stop_all_shufflers().await;
    assert!(service.get_all_shufflers().await.is_empty());
}

#[tokio::test]
async fn test_shuffler_service_get_shuffling_shufflers() {
    let service = ShufflerService::new();

    service.add_or_get_shuffler(100, vec![1], vec![4, 5, 6]).await.unwrap();
    service.add_or_get_shuffler(200, vec![2], vec![4, 5, 6]).await.unwrap();
    service.add_or_get_shuffler(300, vec![3], vec![7, 8, 9]).await.unwrap();

    let shufflers = service.get_shuffling_shufflers(&[4, 5, 6]).await;
    assert_eq!(shufflers.len(), 2);
}

#[tokio::test]
async fn test_shuffler_service_get_account_shufflers() {
    let service = ShufflerService::new();

    service.add_or_get_shuffler(100, vec![1], vec![4, 5, 6]).await.unwrap();
    service.add_or_get_shuffler(100, vec![2], vec![7, 8, 9]).await.unwrap();
    service.add_or_get_shuffler(200, vec![3], vec![4, 5, 6]).await.unwrap();

    let shufflers = service.get_account_shufflers(100).await;
    assert_eq!(shufflers.len(), 2);
}

#[tokio::test]
async fn test_shuffler_service_limit() {
    let service = ShufflerService::with_max_shufflers(2);

    service.add_or_get_shuffler(100, vec![1], vec![4, 5, 6]).await.unwrap();
    service.add_or_get_shuffler(200, vec![2], vec![7, 8, 9]).await.unwrap();

    let result = service.add_or_get_shuffler(300, vec![3], vec![10, 11, 12]).await;
    assert!(matches!(result, Err(ShufflerError::LimitExceeded(_))));
}

#[tokio::test]
async fn test_shuffler_service_create_shuffling() {
    let service = ShufflerService::new();

    let shuffling = service.create_shuffling(1, 100, 1000, 3, None, 0, 100).await;
    assert_eq!(shuffling.id, 1);
    assert_eq!(shuffling.issuer_id, 100);
    assert_eq!(shuffling.amount, 1000);
    assert_eq!(shuffling.stage, ShufflingStage::Registration);
    assert_eq!(shuffling.blocks_remaining, Some(100));
}

#[tokio::test]
async fn test_shuffler_service_add_participant() {
    let service = ShufflerService::new();

    let mut shuffling = service.create_shuffling(1, 100, 1000, 3, None, 0, 100).await;
    service.add_participant(&mut shuffling, tx_engine::shuffler::ShufflingParticipant {
        shuffling_id: 1,
        account_id: 200,
        next_account_id: None,
        participant_index: 1,
        state: tx_engine::shuffler::ParticipantState::Registered,
        blame_data: None,
        key_seeds: None,
        data: None,
        data_transaction_full_hash: None,
        height: 0,
    }).await;

    assert_eq!(shuffling.registrant_count, 2);
    assert_eq!(shuffling.stage, ShufflingStage::Registration);
}

#[tokio::test]
async fn test_shuffler_service_cancel_by() {
    let service = ShufflerService::new();

    let mut shuffling = service.create_shuffling(1, 100, 1000, 3, None, 0, 100).await;
    service.cancel_by(&mut shuffling, 200).await;

    assert_eq!(shuffling.stage, ShufflingStage::Blame);
    assert_eq!(shuffling.assignee_account_id, Some(200));
}

#[tokio::test]
async fn test_shuffler_service_schedule_expiration() {
    let service = ShufflerService::new();

    let shuffling = service.create_shuffling(1, 100, 1000, 3, None, 0, 100).await;
    service.schedule_expiration(&shuffling, 100).await;
}

#[tokio::test]
async fn test_shuffler_service_process_block_expirations() {
    let service = ShufflerService::new();

    let shuffling = service.create_shuffling(1, 100, 1000, 3, None, 0, 100).await;
    let state_hash = shuffling.get_state_hash();
    service.schedule_expiration(&shuffling, 100).await;

    service.add_or_get_shuffler(100, vec![1, 2, 3], state_hash.clone()).await.unwrap();
    assert_eq!(service.get_all_shufflers().await.len(), 1);

    service.process_block_expirations(820).await;
    assert!(service.get_all_shufflers().await.is_empty(), "Should expire at height 100+720=820");
}
