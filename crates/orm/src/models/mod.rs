//! Database models organized by category
//!
//! All models are derived from `sqlx::FromRow` and include
//! `to_domain` / `from_domain` methods for domain object conversion.

pub mod block;
pub mod transaction;
pub mod account;
pub mod asset;
pub mod currency;
pub mod order;
pub mod alias;
pub mod account_ext;
pub mod asset_ext;
pub mod phasing;
pub mod poll;
pub mod goods;
pub mod tagged_data;
pub mod shuffling;
pub mod prunable;
pub mod misc;

// Re-export all models for backward compatibility
pub use block::*;
pub use transaction::*;
pub use account::*;
pub use asset::*;
pub use currency::*;
pub use order::*;
pub use alias::*;
pub use account_ext::*;
pub use asset_ext::*;
pub use phasing::*;
pub use poll::*;
pub use goods::*;
pub use tagged_data::*;
pub use shuffling::*;
pub use prunable::*;
pub use misc::*;
