//! ORM (Object-Relational Mapping) layer for NRCS blockchain
//!
//! Provides database models and repository traits for blockchain entities.
//! Uses SQLx for compile-time query verification and async operations.

pub mod models;
pub mod repository;
pub mod genesis; // genesis block creation

pub use models::*;
pub use repository::*;
pub use genesis::*;