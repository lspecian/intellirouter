//! Integration test module
//!
//! This module contains integration tests for the IntelliRouter application.

// API integration tests
pub mod api;

// Database integration tests
pub mod database;

// External services integration tests
pub mod external_services;

// General integration tests
pub mod general_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_all_integration_tests() {
        // This test can be used to run all integration tests together
        // It's currently a placeholder
    }
}
