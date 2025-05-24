//! End-to-End Tests for IntelliRouter
//!
//! This file is the main entry point for running end-to-end tests.
//! It includes the e2e module which contains all the end-to-end tests.

// Import the e2e module
mod e2e;

// Re-export the e2e module for easier access
pub use e2e::*;

#[cfg(test)]
mod tests {
    #[test]
    fn e2e_tests_entry_point() {
        // This test ensures that the e2e tests entry point is properly included in the build
        assert!(true);
    }
}
