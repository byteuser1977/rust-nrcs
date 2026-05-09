//! Common utilities and shared functionality for NRCS blockchain
//!
//! This crate provides reusable components that can be used across
//! all modules in the rust-nrcs project, including:
//! - Log masking utilities for sensitive data protection
//! - Common error types
//! - Shared helper functions

pub mod log_masker;

pub use log_masker::{LogMasker, MaskStrategy};
