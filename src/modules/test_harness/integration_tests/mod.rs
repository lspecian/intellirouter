//! Integration Tests Module
//!
//! This module provides integration tests between components with error scenarios.

pub mod error_recovery_integration_tests;

use crate::modules::test_harness::{TestCategory, TestSuite};

/// Create a test suite for integration tests
pub fn create_integration_test_suite() -> TestSuite {
    let mut suite = TestSuite::new("Integration Tests")
        .with_description("Tests for integration between components");

    // Add test cases from submodules
    suite = suite.with_test_case(
        error_recovery_integration_tests::create_router_retry_integration_test_case(),
    );

    suite
}
