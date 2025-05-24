//! # IntelliRouter Test Utilities
//!
//! This crate provides test utilities, fixtures, and mocks for testing the IntelliRouter project.
//! It is designed to be used as a dev-dependency in the main IntelliRouter crate and other related crates.
//!
//! ## Features
//!
//! - **Fixtures**: Common test data and fixtures for testing
//! - **Mocks**: Mock implementations of IntelliRouter components and services
//! - **Helpers**: Helper functions and utilities for testing
//!
//! ## Usage
//!
//! Add this crate as a dev-dependency in your Cargo.toml:
//!
//! ```toml
//! [dev-dependencies]
//! intellirouter-test-utils = { path = "../intellirouter-test-utils" }
//! ```
//!
//! Then import and use the utilities in your tests:
//!
//! ```rust
//! use intellirouter_test_utils::fixtures;
//! use intellirouter_test_utils::mocks;
//! use intellirouter_test_utils::helpers;
//! ```

// Re-export modules
pub mod fixtures;
pub mod helpers;
pub mod mocks;

// Export common utilities
pub use fixtures::*;
pub use helpers::*;
pub use mocks::*;

/// Initializes the test environment.
///
/// This function sets up common test environment configurations like tracing,
/// temporary directories, and other test prerequisites.
pub fn init_test_env() {
    // Initialize tracing for tests
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_test_writer()
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_test_env() {
        // Just make sure it doesn't panic
        init_test_env();
    }
}
