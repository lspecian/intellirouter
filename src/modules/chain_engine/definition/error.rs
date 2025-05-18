//! Error types for chain definitions
//!
//! This file contains error types specific to chain definitions.

// This file is intentionally left minimal as the main error types
// are defined in the parent module's error.rs file.
// We're creating this as a placeholder for future definition-specific errors.

// Re-export the error types from the parent module
pub use crate::modules::chain_engine::error::{ChainError, ChainResult};
