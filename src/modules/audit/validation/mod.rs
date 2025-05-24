//! Validation Module
//!
//! This module implements a comprehensive validation workflow that tests service discovery,
//! direct communication, end-to-end flows, data integrity, and error handling.

mod config;
mod types;
mod workflow;

// Validation type-specific modules
mod communication;
mod data_integrity;
mod discovery;
mod error_handling;
mod flows;
mod reporting;
mod security;

// Re-export main components
pub use config::ValidationConfig;
pub use types::ValidationResult;
pub use workflow::ValidationWorkflow;

// Re-export validation functions
