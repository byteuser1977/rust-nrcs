//! ORM (Object-Relational Mapping) layer for NRCS blockchain
//!
//! Provides database models and repository traits for blockchain entities.
//! Uses SQLx for compile-time query verification and async operations.
//!
//! # Features
//! - **sqlite**: SQLite backend support
//! - **postgres**: PostgreSQL backend support
//!
//! # Configuration
//! Database type is auto-detected from connection URL:
//! - `sqlite://path/to/db.db` → SQLite
//! - `postgres://user:pass@host:5432/db` → PostgreSQL

pub mod models;
pub mod repository;
pub mod genesis;
pub mod transaction;
pub mod connection;

pub use models::*;
pub use repository::*;
pub use genesis::*;
pub use transaction::*;
pub use connection::*;