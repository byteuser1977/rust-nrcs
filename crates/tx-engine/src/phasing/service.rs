//! Phasing Service
//!
//! 对应 Java: PhasingPoll.java (service methods)

use std::sync::Arc;
use async_trait::async_trait;

use blockchain_types::{AccountId, Height, Result};

use crate::phasing::types::{PhasingPoll, PhasingVote, PhasingPollResult};
use crate::restrictions::types::VotingModel;

/// Phasing Poll Repository Trait
#[async_trait]
pub trait PhasingPollRepository: Send + Sync {
    async fn find_by_id(&self, id: u64) -> Result<Option<PhasingPoll>>;
    async fn insert(&self, poll: &PhasingPoll) -> Result<()>;
    async fn find_finishing_at_height(&self, height: Height) -> Result<Vec<PhasingPoll>>;
    async fn find_by_voter(&self, voter_id: AccountId, limit: i64) -> Result<Vec<PhasingPoll>>;
    async fn find_by_account(&self, account_id: AccountId, limit: i64) -> Result<Vec<PhasingPoll>>;
}

/// Phasing Vote Repository Trait
#[async_trait]
pub trait PhasingVoteRepository: Send + Sync {
    async fn find_by_poll(&self, poll_id: u64) -> Result<Vec<PhasingVote>>;
    async fn insert(&self, vote: &PhasingVote) -> Result<()>;
    async fn count_votes(&self, poll_id: u64) -> Result<i64>;
}

/// Phasing Poll Result Repository Trait
#[async_trait]
pub trait PhasingPollResultRepository: Send + Sync {
    async fn find_by_id(&self, id: u64) -> Result<Option<PhasingPollResult>>;
    async fn insert(&self, result: &PhasingPollResult) -> Result<()>;
    async fn find_approved_at_height(&self, height: Height) -> Result<Vec<PhasingPollResult>>;
}

/// Phasing Service
///
/// 对应 Java: PhasingPoll static methods
pub struct PhasingService {
    poll_repo: Arc<dyn PhasingPollRepository>,
    vote_repo: Arc<dyn PhasingVoteRepository>,
    result_repo: Arc<dyn PhasingPollResultRepository>,
}

impl PhasingService {
    pub fn new(
        poll_repo: Arc<dyn PhasingPollRepository>,
        vote_repo: Arc<dyn PhasingVoteRepository>,
        result_repo: Arc<dyn PhasingPollResultRepository>,
    ) -> Self {
        Self {
            poll_repo,
            vote_repo,
            result_repo,
        }
    }

    pub async fn get_poll(&self, id: u64) -> Result<Option<PhasingPoll>> {
        self.poll_repo.find_by_id(id).await
    }

    pub async fn get_finishing_transactions(&self, height: Height) -> Result<Vec<PhasingPoll>> {
        self.poll_repo.find_finishing_at_height(height).await
    }

    pub async fn get_voter_phased_transactions(
        &self,
        voter_id: AccountId,
        limit: i64,
    ) -> Result<Vec<PhasingPoll>> {
        self.poll_repo.find_by_voter(voter_id, limit).await
    }

    pub async fn get_account_phased_transactions(
        &self,
        account_id: AccountId,
        limit: i64,
    ) -> Result<Vec<PhasingPoll>> {
        self.poll_repo.find_by_account(account_id, limit).await
    }

    pub async fn add_poll(&self, poll: &PhasingPoll) -> Result<()> {
        self.poll_repo.insert(poll).await
    }

    pub async fn add_vote(&self, vote: &PhasingVote) -> Result<()> {
        self.vote_repo.insert(vote).await
    }

    pub async fn count_votes(&self, poll_id: u64) -> Result<i64> {
        self.vote_repo.count_votes(poll_id).await
    }

    pub async fn finish_poll(&self, poll_id: u64, result: i64, approved: bool, height: Height) -> Result<()> {
        let poll_result = PhasingPollResult::new(poll_id, result, approved, height);
        self.result_repo.insert(&poll_result).await
    }

    pub async fn get_result(&self, poll_id: u64) -> Result<Option<PhasingPollResult>> {
        self.result_repo.find_by_id(poll_id).await
    }

    pub async fn get_approved_at_height(&self, height: Height) -> Result<Vec<PhasingPollResult>> {
        self.result_repo.find_approved_at_height(height).await
    }

    pub async fn count_votes_for_poll(
        &self,
        poll: &PhasingPoll,
        current_height: Height,
    ) -> Result<i64> {
        let voting_model = VotingModel::from_code(poll.voting_model)
            .unwrap_or(VotingModel::None);

        if voting_model == VotingModel::None {
            return Ok(0);
        }

        if voting_model.is_balance_independent() {
            return self.vote_repo.count_votes(poll.id).await;
        }

        let votes = self.vote_repo.find_by_poll(poll.id).await?;
        let mut cumulative_weight: i64 = 0;

        for vote in votes {
            let weight = self.calculate_vote_weight(&vote, poll, current_height).await?;
            cumulative_weight += weight;
        }

        Ok(cumulative_weight)
    }

    async fn calculate_vote_weight(
        &self,
        _vote: &PhasingVote,
        poll: &PhasingPoll,
        _current_height: Height,
    ) -> Result<i64> {
        let voting_model = VotingModel::from_code(poll.voting_model)
            .unwrap_or(VotingModel::None);

        match voting_model {
            VotingModel::Account => Ok(1),
            VotingModel::NqtBalance | VotingModel::Asset | VotingModel::Currency => {
                Ok(poll.min_balance.unwrap_or(0))
            }
            _ => Ok(1),
        }
    }

    pub async fn is_approved(&self, poll: &PhasingPoll, current_height: Height) -> Result<bool> {
        if let Some(result) = self.get_result(poll.id).await? {
            return Ok(result.approved);
        }

        let vote_count = self.count_votes_for_poll(poll, current_height).await?;
        let quorum = poll.quorum.unwrap_or(0);

        Ok(if quorum > 0 {
            vote_count >= quorum
        } else {
            vote_count > 0
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPollRepository;

    #[async_trait]
    impl PhasingPollRepository for MockPollRepository {
        async fn find_by_id(&self, _id: u64) -> Result<Option<PhasingPoll>> {
            Ok(None)
        }
        async fn insert(&self, _poll: &PhasingPoll) -> Result<()> {
            Ok(())
        }
        async fn find_finishing_at_height(&self, _height: Height) -> Result<Vec<PhasingPoll>> {
            Ok(Vec::new())
        }
        async fn find_by_voter(&self, _voter_id: AccountId, _limit: i64) -> Result<Vec<PhasingPoll>> {
            Ok(Vec::new())
        }
        async fn find_by_account(&self, _account_id: AccountId, _limit: i64) -> Result<Vec<PhasingPoll>> {
            Ok(Vec::new())
        }
    }

    struct MockVoteRepository;

    #[async_trait]
    impl PhasingVoteRepository for MockVoteRepository {
        async fn find_by_poll(&self, _poll_id: u64) -> Result<Vec<PhasingVote>> {
            Ok(Vec::new())
        }
        async fn insert(&self, _vote: &PhasingVote) -> Result<()> {
            Ok(())
        }
        async fn count_votes(&self, _poll_id: u64) -> Result<i64> {
            Ok(0)
        }
    }

    struct MockResultRepository;

    #[async_trait]
    impl PhasingPollResultRepository for MockResultRepository {
        async fn find_by_id(&self, _id: u64) -> Result<Option<PhasingPollResult>> {
            Ok(None)
        }
        async fn insert(&self, _result: &PhasingPollResult) -> Result<()> {
            Ok(())
        }
        async fn find_approved_at_height(&self, _height: Height) -> Result<Vec<PhasingPollResult>> {
            Ok(Vec::new())
        }
    }

    #[tokio::test]
    async fn test_phasing_service_creation() {
        let service = PhasingService::new(
            Arc::new(MockPollRepository),
            Arc::new(MockVoteRepository),
            Arc::new(MockResultRepository),
        );

        let poll = service.get_poll(1).await;
        assert!(poll.is_ok());
    }
}
