//! Phasing Module
//!
//! 对应 Java: PhasingPoll.java, PhasingVote.java
//!
//! 分阶段投票系统，支持交易的延迟确认和投票验证

pub mod types;
pub mod service;

pub use types::{PhasingPoll, PhasingVote, PhasingPollResult, HashFunction};
pub use service::{PhasingService, PhasingPollRepository};
