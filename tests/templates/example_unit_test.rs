//! Example unit test demonstrating best practices
//!
//! This file shows how to write unit tests following IntelliRouter's test-first approach.

use intellirouter::modules::router_core::strategies::{ContentBasedStrategy, RoutingStrategy};
use intellirouter::modules::router_core::types::{Error, Request, Response};
use intellirouter_test_utils::fixtures::create_test_request;
use intellirouter_test_utils::mocks::create_mock_model_backend;

/// Test the basic functionality of the ContentBasedStrategy
#[test]
fn test_content_based_strategy_basic_routing() {
    // Arrange
    let strategy = ContentBasedStrategy::new();
    let request = create_test_request("Test content for model A");

    // Act
    let result = strategy.route(request);

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.model_id(), "model-a");
}

/// Test that the ContentBasedStrategy handles empty content correctly
#[test]
fn test_content_based_strategy_empty_content() {
    // Arrange
    let strategy = ContentBasedStrategy::new();
    let request = create_test_request("");

    // Act
    let result = strategy.route(request);

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.kind(), Error::EmptyContent);
}

/// Test that the ContentBasedStrategy handles invalid content correctly
#[test]
fn test_content_based_strategy_invalid_content() {
    // Arrange
    let strategy = ContentBasedStrategy::new();
    let request = create_test_request("@#$%^&*()");

    // Act
    let result = strategy.route(request);

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.kind(), Error::InvalidContent);
}

/// Test that the ContentBasedStrategy routes to the correct model based on content
#[test]
fn test_content_based_strategy_routes_to_correct_model() {
    // Arrange
    let strategy = ContentBasedStrategy::new();

    // Test cases with expected model IDs
    let test_cases = vec![
        ("Test content for model A", "model-a"),
        ("This should go to model B", "model-b"),
        ("Code generation request", "model-c"),
        ("Default content", "default-model"),
    ];

    // Act and Assert
    for (content, expected_model) in test_cases {
        let request = create_test_request(content);
        let result = strategy.route(request);
        assert!(result.is_ok(), "Failed to route: {}", content);
        let response = result.unwrap();
        assert_eq!(
            response.model_id(),
            expected_model,
            "Wrong model for content: {}",
            content
        );
    }
}

/// Test with a mock model backend
#[test]
fn test_content_based_strategy_with_mock_backend() {
    // Arrange
    let strategy = ContentBasedStrategy::new();
    let mock_backend = create_mock_model_backend();
    mock_backend
        .expect_process()
        .times(1)
        .returning(|request| Ok(Response::new("model-a", "Mocked response")));

    let request = create_test_request("Test content");

    // Act
    let routing_result = strategy.route(request.clone()).unwrap();
    let processing_result = mock_backend.process(request);

    // Assert
    assert!(processing_result.is_ok());
    let response = processing_result.unwrap();
    assert_eq!(response.model_id(), "model-a");
    assert_eq!(response.content(), "Mocked response");
}

/// Test with async functionality
#[tokio::test]
async fn test_content_based_strategy_async() {
    // Arrange
    let strategy = ContentBasedStrategy::new();
    let request = create_test_request("Test content for model A");

    // Act
    let result = strategy.route_async(request).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.model_id(), "model-a");
}

/// Test with error conditions
#[test]
fn test_content_based_strategy_error_handling() {
    // Arrange
    let strategy = ContentBasedStrategy::new();

    // Test cases with expected errors
    let test_cases = vec![
        ("", Error::EmptyContent),
        ("@#$%^&*()", Error::InvalidContent),
        ("ERROR", Error::RoutingFailed),
    ];

    // Act and Assert
    for (content, expected_error) in test_cases {
        let request = create_test_request(content);
        let result = strategy.route(request);
        assert!(result.is_err(), "Expected error for content: {}", content);
        let error = result.unwrap_err();
        assert_eq!(
            error.kind(),
            expected_error,
            "Wrong error for content: {}",
            content
        );
    }
}

/// Test with parameterized tests
#[test]
fn test_content_based_strategy_parameterized() {
    // Define a struct to hold test parameters
    struct TestCase {
        content: &'static str,
        expected_result: Result<&'static str, Error>,
    }

    // Define test cases
    let test_cases = vec![
        TestCase {
            content: "Test content for model A",
            expected_result: Ok("model-a"),
        },
        TestCase {
            content: "This should go to model B",
            expected_result: Ok("model-b"),
        },
        TestCase {
            content: "",
            expected_result: Err(Error::EmptyContent),
        },
        TestCase {
            content: "ERROR",
            expected_result: Err(Error::RoutingFailed),
        },
    ];

    // Arrange
    let strategy = ContentBasedStrategy::new();

    // Act and Assert
    for test_case in test_cases {
        let request = create_test_request(test_case.content);
        let result = strategy.route(request);

        match test_case.expected_result {
            Ok(expected_model) => {
                assert!(
                    result.is_ok(),
                    "Expected success for content: {}",
                    test_case.content
                );
                let response = result.unwrap();
                assert_eq!(
                    response.model_id(),
                    expected_model,
                    "Wrong model for content: {}",
                    test_case.content
                );
            }
            Err(expected_error) => {
                assert!(
                    result.is_err(),
                    "Expected error for content: {}",
                    test_case.content
                );
                let error = result.unwrap_err();
                assert_eq!(
                    error.kind(),
                    expected_error,
                    "Wrong error for content: {}",
                    test_case.content
                );
            }
        }
    }
}
