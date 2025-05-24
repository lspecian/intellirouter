//! Integration Tests Runner
//!
//! This file is the main entry point for running integration tests.
//! It includes the integration module and runs all tests in that module.

// Include the integration module
mod integration;

#[cfg(test)]
mod tests {
    use super::*;
    use intellirouter_test_utils::init_test_env;

    #[test]
    fn run_all_integration_tests() {
        // Initialize test environment
        init_test_env();

        // This test is a placeholder that ensures all integration tests are compiled
        // The actual tests are run individually by the test runner
        println!("Integration tests compiled successfully");
    }
}
