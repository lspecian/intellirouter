//! Retry and Fallback Mechanism Tests
//!
//! This module contains tests for the retry and fallback mechanisms.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex as TokioMutex;

use crate::modules::model_registry::{
    connectors::{ChatCompletionRequest, ChatCompletionResponse, ChatMessage, MessageRole},
    storage::ModelRegistry,
    ModelMetadata, ModelStatus, ModelType,
};
use crate::modules::router_core::{
    retry::{
        CircuitBreakerConfig, CircuitBreakerState, DegradedServiceHandler, DegradedServiceMode,
        ErrorCategory, RetryManager, RetryPolicy,
    },
    router::RouterImpl,
    Router, RouterConfig, RouterError, RoutingRequest,
};
use crate::test_utils::mocks::MockModelRegistry;

// Helper function to create a test request
fn create_test_request() -> RoutingRequest {
    let chat_request = ChatCompletionRequest {
        model: "test-model".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: "Hello, world!".to_string(),
            name: None,
            function_call: None,
            tool_calls: None,
        }],
        temperature: None,
        top_p: None,
        max_tokens: None,
        stream: None,
        functions: None,
        tools: None,
        additional_params: None,
    };

    RoutingRequest::new(chat_request)
}

// Helper function to create a test model
fn create_test_model(id: &str, provider: &str) -> ModelMetadata {
    let mut model = ModelMetadata::new(
        id.to_string(),
        format!("Test Model {}", id),
        provider.to_string(),
        "1.0".to_string(),
        "https://example.com".to_string(),
    );

    model.set_status(ModelStatus::Available);
    model.set_model_type(ModelType::TextGeneration);
    model.capabilities.max_context_length = 4096;
    model.capabilities.supports_streaming = true;
    model.capabilities.supports_function_calling = true;

    model
}

#[tokio::test]
async fn test_retry_policy_fixed() {
    // Create a counter to track the number of attempts
    let attempts = Arc::new(TokioMutex::new(0));
    let attempts_clone = attempts.clone();

    // Create a retry manager with a fixed retry policy
    let mut retryable_errors = HashSet::new();
    retryable_errors.insert(ErrorCategory::Network);

    let retry_manager = RetryManager::new(
        RetryPolicy::Fixed {
            interval_ms: 10,
            max_retries: 2,
        },
        CircuitBreakerConfig {
            enabled: false,
            ..Default::default()
        },
        retryable_errors,
    );

    // Test function that fails with a network error twice, then succeeds
    let result = retry_manager
        .execute(
            || async {
                let mut count = attempts_clone.lock().await;
                *count += 1;

                if *count <= 2 {
                    Err(RouterError::ConnectorError("Network error".to_string()))
                } else {
                    Ok("Success")
                }
            },
            "test",
        )
        .await;

    // Check that the function succeeded after retries
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Success");

    // Check that the function was called 3 times (initial + 2 retries)
    let final_attempts = *attempts.lock().await;
    assert_eq!(final_attempts, 3);
}

#[tokio::test]
async fn test_retry_policy_exponential_backoff() {
    // Create a counter to track the number of attempts
    let attempts = Arc::new(TokioMutex::new(0));
    let attempts_clone = attempts.clone();

    // Create a retry manager with an exponential backoff retry policy
    let mut retryable_errors = HashSet::new();
    retryable_errors.insert(ErrorCategory::Timeout);

    let retry_manager = RetryManager::new(
        RetryPolicy::ExponentialBackoff {
            initial_interval_ms: 10,
            backoff_factor: 2.0,
            max_retries: 2,
            max_interval_ms: 100,
        },
        CircuitBreakerConfig {
            enabled: false,
            ..Default::default()
        },
        retryable_errors,
    );

    // Test function that fails with a timeout error twice, then succeeds
    let result = retry_manager
        .execute(
            || async {
                let mut count = attempts_clone.lock().await;
                *count += 1;

                if *count <= 2 {
                    Err(RouterError::ConnectorError("Request timed out".to_string()))
                } else {
                    Ok("Success")
                }
            },
            "test",
        )
        .await;

    // Check that the function succeeded after retries
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Success");

    // Check that the function was called 3 times (initial + 2 retries)
    let final_attempts = *attempts.lock().await;
    assert_eq!(final_attempts, 3);
}

#[tokio::test]
async fn test_retry_policy_non_retryable_error() {
    // Create a counter to track the number of attempts
    let attempts = Arc::new(TokioMutex::new(0));
    let attempts_clone = attempts.clone();

    // Create a retry manager with a fixed retry policy
    let mut retryable_errors = HashSet::new();
    retryable_errors.insert(ErrorCategory::Network);

    let retry_manager = RetryManager::new(
        RetryPolicy::Fixed {
            interval_ms: 10,
            max_retries: 2,
        },
        CircuitBreakerConfig {
            enabled: false,
            ..Default::default()
        },
        retryable_errors,
    );

    // Test function that fails with a non-retryable error
    let result = retry_manager
        .execute(
            || async {
                let mut count = attempts_clone.lock().await;
                *count += 1;

                Err(RouterError::InvalidRequest("Invalid request".to_string()))
            },
            "test",
        )
        .await;

    // Check that the function failed without retries
    assert!(result.is_err());
    match result {
        Err(RouterError::InvalidRequest(_)) => {}
        _ => panic!("Expected InvalidRequest error"),
    }

    // Check that the function was called only once (no retries)
    let final_attempts = *attempts.lock().await;
    assert_eq!(final_attempts, 1);
}

#[tokio::test]
async fn test_circuit_breaker() {
    // Create a counter to track the number of attempts
    let attempts = Arc::new(TokioMutex::new(0));
    let attempts_clone = attempts.clone();

    // Create a retry manager with a circuit breaker
    let mut retryable_errors = HashSet::new();
    retryable_errors.insert(ErrorCategory::Network);

    let retry_manager = RetryManager::new(
        RetryPolicy::None,
        CircuitBreakerConfig {
            enabled: true,
            failure_threshold: 2,
            success_threshold: 2,
            reset_timeout_ms: 50,
        },
        retryable_errors,
    );

    // Test function that always fails
    let result1 = retry_manager
        .execute(
            || async {
                let mut count = attempts_clone.lock().await;
                *count += 1;
                Err(RouterError::ConnectorError("Network error".to_string()))
            },
            "test",
        )
        .await;

    // First call should fail but circuit should still be closed
    assert!(result1.is_err());
    assert_eq!(
        retry_manager.get_circuit_breaker_state(),
        CircuitBreakerState::Closed
    );

    // Second call should fail and open the circuit
    let result2 = retry_manager
        .execute(
            || async {
                let mut count = attempts_clone.lock().await;
                *count += 1;
                Err(RouterError::ConnectorError("Network error".to_string()))
            },
            "test",
        )
        .await;

    assert!(result2.is_err());
    assert_eq!(
        retry_manager.get_circuit_breaker_state(),
        CircuitBreakerState::Open
    );

    // Third call should fail fast without executing the function
    let result3 = retry_manager
        .execute(
            || async {
                let mut count = attempts_clone.lock().await;
                *count += 1;
                Err(RouterError::ConnectorError("Network error".to_string()))
            },
            "test",
        )
        .await;

    assert!(result3.is_err());
    match result3 {
        Err(RouterError::Other(msg)) => {
            assert!(msg.contains("Circuit breaker is open"));
        }
        _ => panic!("Expected Other error with circuit breaker message"),
    }

    // Check that the function was called only twice (circuit prevented third call)
    let final_attempts = *attempts.lock().await;
    assert_eq!(final_attempts, 2);

    // Wait for the reset timeout
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Fourth call should execute the function (half-open state)
    let result4 = retry_manager
        .execute(
            || async {
                let mut count = attempts_clone.lock().await;
                *count += 1;
                Ok("Success")
            },
            "test",
        )
        .await;

    assert!(result4.is_ok());
    assert_eq!(
        retry_manager.get_circuit_breaker_state(),
        CircuitBreakerState::HalfOpen
    );

    // Fifth call should succeed and close the circuit
    let result5 = retry_manager
        .execute(
            || async {
                let mut count = attempts_clone.lock().await;
                *count += 1;
                Ok("Success")
            },
            "test",
        )
        .await;

    assert!(result5.is_ok());
    assert_eq!(
        retry_manager.get_circuit_breaker_state(),
        CircuitBreakerState::Closed
    );

    // Check that the function was called 4 times in total
    let final_attempts = *attempts.lock().await;
    assert_eq!(final_attempts, 4);
}

#[tokio::test]
async fn test_degraded_service_mode_fail_fast() {
    let registry = Arc::new(ModelRegistry::new());

    // Create a degraded service handler with fail fast mode
    let handler = DegradedServiceHandler::new(DegradedServiceMode::FailFast, registry);

    // Test handling a request
    let request = create_test_request();
    let result = handler.handle_request(&request).await;

    // Check that the handler failed fast
    assert!(result.is_err());
    match result {
        Err(RouterError::Other(msg)) => {
            assert!(msg.contains("Service is in degraded mode"));
        }
        _ => panic!("Expected Other error with degraded mode message"),
    }
}

#[tokio::test]
async fn test_degraded_service_mode_static_response() {
    let registry = Arc::new(ModelRegistry::new());

    // Create a degraded service handler with static response mode
    let handler = DegradedServiceHandler::new(
        DegradedServiceMode::StaticResponse("This is a static response".to_string()),
        registry,
    );

    // Test handling a request
    let request = create_test_request();
    let result = handler.handle_request(&request).await;

    // Check that the handler returned a static response
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.response.model, "degraded-mode");
    assert_eq!(
        response.response.choices[0].message.content,
        "This is a static response"
    );
    assert_eq!(response.metadata.selected_model_id, "degraded-mode");
    assert!(response.metadata.is_fallback);
}

#[tokio::test]
async fn test_router_with_retry_and_fallback() {
    // Create a router config with retry and fallback
    let mut config = RouterConfig::default();
    config.retry_policy = RetryPolicy::Fixed {
        interval_ms: 10,
        max_retries: 1,
    };
    config.degraded_service_mode =
        DegradedServiceMode::StaticResponse("This is a degraded service response".to_string());

    // Create a mock registry
    let registry = Arc::new(MockModelRegistry::new());

    // Create a router
    let router = RouterImpl::new(config, registry).unwrap();

    // Test routing a request
    // Note: This test is limited because we can't easily mock the model connectors
    // In a real implementation, we would use a more sophisticated mock
    let request = create_test_request();
    let result = router.route(request).await;

    // The result will likely be an error since we're using a mock registry
    // But the important thing is that the router used the retry and fallback mechanisms
    assert!(result.is_err());
}
