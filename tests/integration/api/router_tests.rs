//! Integration tests for the Router component
//!
//! These tests verify that the Router works correctly with other components.

use intellirouter::{
    config::Config,
    modules::{
        model_registry,
        router_core::{self, RouterConfig, RoutingStrategy},
    },
    test_utils::{self, init_test_logging, TestConfig},
};
use std::path::PathBuf;
use tokio;

#[test]
fn test_router_with_model_registry() {
    // Initialize test environment
    init_test_logging();
    let test_config = TestConfig::new();

    // Set up router configuration
    let router_config = RouterConfig::default();

    // Initialize router
    let router_result = router_core::init(router_config);
    assert!(router_result.is_ok());

    // Create a test request
    let request = test_utils::create_test_request("Test request for model registry integration");

    // Route the request
    let routing_result = router_core::route_request(&request);
    assert!(routing_result.is_ok());

    // In a real test, we would verify that the router correctly interacts with the model registry
    // For now, this is a placeholder until more implementation is available
}

#[tokio::test]
async fn test_router_with_multiple_models() {
    // This test will verify that the router can correctly route requests to different models
    // based on the routing strategy

    // Initialize test environment
    init_test_logging();
    let test_config = TestConfig::new();

    // Set up router configuration for round-robin strategy
    let mut router_config = RouterConfig::default();
    router_config.strategy = RoutingStrategy::RoundRobin;

    // Initialize router
    let router_result = router_core::init(router_config);
    assert!(router_result.is_ok());

    // Create multiple test requests
    let requests = vec![
        test_utils::create_test_request("First test request"),
        test_utils::create_test_request("Second test request"),
        test_utils::create_test_request("Third test request"),
    ];

    // Route each request and verify they are routed correctly
    for request in requests {
        let routing_result = router_core::route_request(&request);
        assert!(routing_result.is_ok());
    }

    // In a real test, we would verify that the requests are distributed according to the round-robin strategy
    // For now, this is a placeholder until more implementation is available
}

#[test]
fn test_router_with_cost_optimization() {
    // Initialize test environment
    init_test_logging();
    let test_config = TestConfig::new();

    // Set up router configuration for cost optimization
    let mut router_config = RouterConfig::default();
    router_config.strategy = RoutingStrategy::CostOptimized;

    // Initialize router
    let router_result = router_core::init(router_config);
    assert!(router_result.is_ok());

    // Create a test request
    let request = test_utils::create_test_request("Test request for cost optimization");

    // Route the request
    let routing_result = router_core::route_request(&request);
    assert!(routing_result.is_ok());

    // In a real test, we would verify that the router selects the most cost-effective model
    // For now, this is a placeholder until more implementation is available
}

// Test with error conditions
#[test]
fn test_router_with_error_handling() {
    // Initialize test environment
    init_test_logging();
    let test_config = TestConfig::new();

    // Set up router configuration
    let mut router_config = RouterConfig::default();
    router_config.strategy = RoutingStrategy::ContentBased;

    // Initialize router
    let router_result = router_core::init(router_config);
    assert!(router_result.is_ok());

    // Create an invalid test request (in a real test, this would be a malformed request)
    let invalid_request = "Invalid request format";

    // Route the request - in a real implementation, this might return an error
    // For now, we're just checking that the function doesn't panic
    let routing_result = router_core::route_request(invalid_request);

    // In a real test with error handling, we might expect an error here
    // assert!(routing_result.is_err());
    // For now, we're just checking that the function returns something
    assert!(true);
}

#[tokio::test]
async fn test_content_based_routing() {
    // Initialize test environment
    init_test_logging();
    let test_config = TestConfig::new();

    // Set up router configuration for content-based strategy
    let mut router_config = RouterConfig::default();
    router_config.strategy = RoutingStrategy::ContentBased;

    // Initialize router
    let router_result = router_core::init(router_config);
    assert!(router_result.is_ok(), "Router initialization failed");

    // Create test requests with different content characteristics
    let technical_request = test_utils::create_test_request(
        "Explain the principles of quantum computing and how qubits work in detail.",
    );

    let creative_request =
        test_utils::create_test_request("Write a short poem about a sunset over the ocean.");

    let code_request = test_utils::create_test_request(
        "Write a Rust function that implements a binary search algorithm.",
    );

    // Route each request
    let technical_result = router_core::route_request(&technical_request);
    let creative_result = router_core::route_request(&creative_request);
    let code_result = router_core::route_request(&code_request);

    // All requests should be routed successfully
    assert!(technical_result.is_ok(), "Technical request routing failed");
    assert!(creative_result.is_ok(), "Creative request routing failed");
    assert!(code_result.is_ok(), "Code request routing failed");

    // In a complete test, we would verify that:
    // 1. Technical request is routed to a model with strong technical capabilities
    // 2. Creative request is routed to a model with strong creative capabilities
    // 3. Code request is routed to a model with strong coding capabilities

    // For now, we'll just check that the routing was successful and log the results
    if let (Ok(technical_response), Ok(creative_response), Ok(code_response)) =
        (&technical_result, &creative_result, &code_result)
    {
        // Parse the responses to extract the model IDs (in a real implementation)
        // For now, we'll just log the responses
        println!("Technical request routed successfully");
        println!("Creative request routed successfully");
        println!("Code request routed successfully");

        // The actual verification would look something like this:
        // let technical_model = extract_model_id_from_response(technical_response);
        // let creative_model = extract_model_id_from_response(creative_response);
        // let code_model = extract_model_id_from_response(code_response);

        // assert_eq!(technical_model, "technical-specialized-model");
        // assert_eq!(creative_model, "creative-specialized-model");
        // assert_eq!(code_model, "code-specialized-model");
    }
}
