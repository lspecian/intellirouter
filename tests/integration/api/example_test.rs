//! Example Integration Test
//!
//! This file demonstrates how to write integration tests using the test-utils crate.

use intellirouter::{
    config::Config,
    modules::router_core::{RouterConfig, RoutingStrategy},
};
use intellirouter_test_utils::{
    fixtures::create_test_model, helpers::assert_response_valid, init_test_env,
    mocks::create_mock_router,
};
use std::sync::Arc;

/// Example test that demonstrates how to use the test-utils crate
#[tokio::test]
async fn test_example() {
    // Initialize test environment
    init_test_env();

    // Use fixtures from test-utils
    let model = create_test_model("test-model", "test-provider");

    // Use mocks from test-utils
    let router = create_mock_router();

    // Perform test
    let result = router.route_request("test request").await;

    // Use helpers from test-utils for assertions
    assert_response_valid(&result);
}

/// Example test that shows how to test error conditions
#[tokio::test]
async fn test_example_error() {
    // Initialize test environment
    init_test_env();

    // Create a router with an invalid configuration
    let config = RouterConfig {
        strategy: RoutingStrategy::Unknown,
        ..Default::default()
    };

    // This should result in an error
    let result = intellirouter::modules::router_core::init(config);
    assert!(result.is_err());
}
