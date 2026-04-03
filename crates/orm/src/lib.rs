//! ORM (Object-Relational Mapping) layer for NRCS blockchain
//!
//! Provides database models and repository traits for blockchain entities.
//! Uses SQLx for compile-time query verification and async operations.

pub mod models;
pub mod repository;

pub use models::*;
pub use repository::*;