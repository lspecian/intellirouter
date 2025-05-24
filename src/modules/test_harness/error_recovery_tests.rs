//! Error Recovery Tests
//!
//! This module provides test cases for error conditions and recovery scenarios.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::json;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::modules::model_registry::{
    connectors::{
        ChatCompletionChoice, ChatCompletionRequest, ChatCompletionResponse, ChatMessage,
        MessageRole, ModelConnector,
    },
    storage::ModelRegistry,
    ConnectorError, ModelMetadata, ModelStatus,
};
use crate::modules::router_core::{
    CircuitBreakerConfig, DegradedServiceMode, ErrorCategory, RetryManager, RetryPolicy,
    RouterConfig, RouterError, RoutingContext, RoutingMetadata, RoutingRequest, RoutingResponse,
    RoutingStrategy,
};
use crate::modules::test_harness::{
    AssertionHelper, TestCase, TestCategory, TestContext, TestEngine, TestOutcome, TestResult,
    TestSuite,
};

/// Create a test suite for error recovery tests
pub fn create_error_recovery_test_suite() -> TestSuite {
    let mut suite = TestSuite::new("Error Recovery Tests")
        .with_description("Tests for error conditions and recovery scenarios");

    // Add test cases
    suite = suite
        .with_test_case(create_retry_policy_test_case())
        .with_test_case(create_circuit_breaker_test_case())
        .with_test_case(create_degraded_service_test_case())
        .with_test_case(create_error_categorization_test_case())
        .with_test_case(create_recovery_from_timeout_test_case())
        .with_test_case(create_recovery_from_rate_limit_test_case())
        .with_test_case(create_recovery_from_network_error_test_case());

    suite
}

/// Create a test case for retry policy
fn create_retry_policy_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(TestCategory::Integration, "retry_policy_test".to_string()),
        |ctx| {
            async move {
                info!("Running retry policy test");

                // Create a mock connector that fails a certain number of times before succeeding
                let fail_count = Arc::new(Mutex::new(3));
                let mock_connector = MockFailingConnector::new(fail_count.clone());

                // Create a retry manager with exponential backoff
                let mut retryable_errors = HashSet::new();
                retryable_errors.insert(ErrorCategory::Network);
                retryable_errors.insert(ErrorCategory::Timeout);
                retryable_errors.insert(ErrorCategory::RateLimit);
                retryable_errors.insert(ErrorCategory::Server);

                let retry_policy = RetryPolicy::ExponentialBackoff {
                    initial_interval_ms: 10, // Short for testing
                    backoff_factor: 2.0,
                    max_retries: 5,
                    max_interval_ms: 1000,
                };

                let circuit_breaker_config = CircuitBreakerConfig {
                    failure_threshold: 5,
                    success_threshold: 3,
                    reset_timeout_ms: 30000,
                    enabled: true,
                };

                let retry_manager =
                    RetryManager::new(retry_policy, circuit_breaker_config, retryable_errors);

                // Execute a function with retries
                let result = retry_manager
                    .execute(
                        || async {
                            let mut lock = fail_count.lock().await;
                            if *lock > 0 {
                                *lock -= 1;
                                Err(RouterError::ConnectorError(
                                    "Network error: connection refused".to_string(),
                                ))
                            } else {
                                Ok("Success".to_string())
                            }
                        },
                        "retry_test",
                    )
                    .await;

                // Verify the result
                AssertionHelper::assert_true(result.is_ok(), "Retry should eventually succeed")?;
                AssertionHelper::assert_eq(
                    result.unwrap(),
                    "Success".to_string(),
                    "Result should be 'Success'",
                )?;

                // Verify the fail count is 0
                let final_count = *fail_count.lock().await;
                AssertionHelper::assert_eq(final_count, 0, "Fail count should be 0 after retries")?;

                Ok(TestResult::new(
                    "retry_policy_test",
                    TestCategory::Integration,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Create a test case for circuit breaker
fn create_circuit_breaker_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(
            TestCategory::Integration,
            "circuit_breaker_test".to_string(),
        ),
        |ctx| {
            async move {
                info!("Running circuit breaker test");

                // Create a circuit breaker with a low threshold for testing
                let config = CircuitBreakerConfig {
                    failure_threshold: 2,
                    success_threshold: 2,
                    reset_timeout_ms: 100, // Short for testing
                    enabled: true,
                };

                let mut retryable_errors = HashSet::new();
                retryable_errors.insert(ErrorCategory::Network);

                let retry_manager = RetryManager::new(RetryPolicy::None, config, retryable_errors);

                // Record failures to open the circuit
                let result1 = retry_manager
                    .execute(
                        || async {
                            Err::<String, _>(RouterError::ConnectorError(
                                "Network error".to_string(),
                            ))
                        },
                        "circuit_test_1",
                    )
                    .await;

                AssertionHelper::assert_true(result1.is_err(), "First request should fail")?;

                let result2 = retry_manager
                    .execute(
                        || async {
                            Err::<String, _>(RouterError::ConnectorError(
                                "Network error".to_string(),
                            ))
                        },
                        "circuit_test_2",
                    )
                    .await;

                AssertionHelper::assert_true(result2.is_err(), "Second request should fail")?;

                // Circuit should be open now, so this should fail fast
                let result3 = retry_manager
                    .execute(
                        || async {
                            // This shouldn't be called if circuit is open
                            Ok("Success".to_string())
                        },
                        "circuit_test_3",
                    )
                    .await;

                AssertionHelper::assert_true(
                    result3.is_err(),
                    "Third request should fail fast due to open circuit",
                )?;

                // Wait for the reset timeout
                tokio::time::sleep(Duration::from_millis(150)).await;

                // Circuit should be half-open now, so this should succeed
                let result4 = retry_manager
                    .execute(|| async { Ok("Success".to_string()) }, "circuit_test_4")
                    .await;

                AssertionHelper::assert_true(
                    result4.is_ok(),
                    "Fourth request should succeed in half-open state",
                )?;

                // Another success should close the circuit
                let result5 = retry_manager
                    .execute(|| async { Ok("Success".to_string()) }, "circuit_test_5")
                    .await;

                AssertionHelper::assert_true(
                    result5.is_ok(),
                    "Fifth request should succeed and close the circuit",
                )?;

                Ok(TestResult::new(
                    "circuit_breaker_test",
                    TestCategory::Integration,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Create a test case for degraded service mode
fn create_degraded_service_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(
            TestCategory::Integration,
            "degraded_service_test".to_string(),
        ),
        |ctx| {
            async move {
                info!("Running degraded service test");

                // Create a mock model registry
                let registry = Arc::new(create_mock_model_registry());

                // Create a degraded service handler with static response
                let static_response = "I'm sorry, the service is currently in degraded mode.";
                let degraded_handler = crate::modules::router_core::DegradedServiceHandler::new(
                    DegradedServiceMode::StaticResponse(static_response.to_string()),
                    registry.clone(),
                );

                // Create a request
                let request = create_mock_routing_request();

                // Handle the request in degraded mode
                let result = degraded_handler.handle_request(&request).await;

                // Verify the result
                AssertionHelper::assert_true(
                    result.is_ok(),
                    "Degraded service should handle the request",
                )?;

                let response = result.unwrap();
                AssertionHelper::assert_eq(
                    response.metadata.is_fallback,
                    true,
                    "Response should be marked as fallback",
                )?;

                AssertionHelper::assert_eq(
                    response.metadata.selected_model_id,
                    "degraded-mode",
                    "Selected model should be 'degraded-mode'",
                )?;

                AssertionHelper::assert_eq(
                    response.response.choices[0].message.content,
                    static_response,
                    "Response content should match the static response",
                )?;

                // Test with default model mode
                let degraded_handler = crate::modules::router_core::DegradedServiceHandler::new(
                    DegradedServiceMode::DefaultModel("mock-model".to_string()),
                    registry.clone(),
                );

                let result = degraded_handler.handle_request(&request).await;
                AssertionHelper::assert_true(
                    result.is_ok(),
                    "Degraded service with default model should handle the request",
                )?;

                // Test with fail fast mode
                let degraded_handler = crate::modules::router_core::DegradedServiceHandler::new(
                    DegradedServiceMode::FailFast,
                    registry.clone(),
                );

                let result = degraded_handler.handle_request(&request).await;
                AssertionHelper::assert_true(
                    result.is_err(),
                    "Degraded service with fail fast should fail",
                )?;

                Ok(TestResult::new(
                    "degraded_service_test",
                    TestCategory::Integration,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Create a test case for error categorization
fn create_error_categorization_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(TestCategory::Unit, "error_categorization_test".to_string()),
        |ctx| {
            async move {
                info!("Running error categorization test");

                // Test various error messages and their categories
                let test_cases = vec![
                    (
                        RouterError::ConnectorError("Network error".to_string()),
                        ErrorCategory::Network,
                    ),
                    (
                        RouterError::ConnectorError("connection refused".to_string()),
                        ErrorCategory::Network,
                    ),
                    (
                        RouterError::ConnectorError("Request timed out".to_string()),
                        ErrorCategory::Timeout,
                    ),
                    (
                        RouterError::Timeout("Operation timed out".to_string()),
                        ErrorCategory::Timeout,
                    ),
                    (
                        RouterError::ConnectorError("Rate limit exceeded".to_string()),
                        ErrorCategory::RateLimit,
                    ),
                    (
                        RouterError::ConnectorError("too many requests".to_string()),
                        ErrorCategory::RateLimit,
                    ),
                    (
                        RouterError::ConnectorError("Authentication failed".to_string()),
                        ErrorCategory::Authentication,
                    ),
                    (
                        RouterError::ConnectorError("unauthorized".to_string()),
                        ErrorCategory::Authentication,
                    ),
                    (
                        RouterError::ConnectorError("Invalid request".to_string()),
                        ErrorCategory::InvalidRequest,
                    ),
                    (
                        RouterError::InvalidRequest("Bad request".to_string()),
                        ErrorCategory::InvalidRequest,
                    ),
                    (
                        RouterError::ConnectorError("Internal server error".to_string()),
                        ErrorCategory::Server,
                    ),
                    (
                        RouterError::ConnectorError("server error".to_string()),
                        ErrorCategory::Server,
                    ),
                    (
                        RouterError::NoSuitableModel("No model found".to_string()),
                        ErrorCategory::ModelNotFound,
                    ),
                    (
                        RouterError::Other("Unknown error".to_string()),
                        ErrorCategory::Other,
                    ),
                ];

                for (error, expected_category) in test_cases {
                    let actual_category = error.category();
                    AssertionHelper::assert_eq(
                        actual_category,
                        expected_category,
                        &format!(
                            "Error '{}' should be categorized as {:?}",
                            error, expected_category
                        ),
                    )?;
                }

                // Test retryable errors
                let mut retryable_categories = HashSet::new();
                retryable_categories.insert(ErrorCategory::Network);
                retryable_categories.insert(ErrorCategory::Timeout);
                retryable_categories.insert(ErrorCategory::RateLimit);
                retryable_categories.insert(ErrorCategory::Server);

                let retryable_error = RouterError::ConnectorError("Network error".to_string());
                AssertionHelper::assert_true(
                    retryable_error.is_retryable(&retryable_categories),
                    "Network error should be retryable",
                )?;

                let non_retryable_error = RouterError::InvalidRequest("Bad request".to_string());
                AssertionHelper::assert_false(
                    non_retryable_error.is_retryable(&retryable_categories),
                    "Invalid request error should not be retryable",
                )?;

                Ok(TestResult::new(
                    "error_categorization_test",
                    TestCategory::Unit,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Create a mock routing request
fn create_mock_routing_request() -> RoutingRequest {
    RoutingRequest {
        context: RoutingContext {
            request: ChatCompletionRequest {
                model: "mock-model".to_string(),
                messages: vec![ChatMessage {
                    role: MessageRole::User,
                    content: "Hello".to_string(),
                    name: None,
                    function_call: None,
                    tool_calls: None,
                }],
                temperature: Some(0.7),
                top_p: Some(0.9),
                n: Some(1),
                stream: false,
                max_tokens: Some(100),
                presence_penalty: Some(0.0),
                frequency_penalty: Some(0.0),
                user: None,
            },
            user_id: Some("test-user".to_string()),
            session_id: Some("test-session".to_string()),
            request_id: "test-request".to_string(),
            timestamp: chrono::Utc::now(),
            additional_context: serde_json::Map::new(),
        },
        constraints: None,
    }
}

/// Mock connector
struct MockConnector;

#[async_trait]
impl ModelConnector for MockConnector {
    async fn generate(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ConnectorError> {
        Ok(ChatCompletionResponse {
            id: "mock-id".to_string(),
            model: "mock-model".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            choices: vec![ChatCompletionChoice {
                index: 0,
                message: ChatMessage {
                    role: MessageRole::Assistant,
                    content: "Mock response".to_string(),
                    name: None,
                    function_call: None,
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: None,
        })
    }
}

/// Mock failing connector that fails a certain number of times before succeeding
struct MockFailingConnector {
    fail_count: Arc<Mutex<usize>>,
}

impl MockFailingConnector {
    fn new(fail_count: Arc<Mutex<usize>>) -> Self {
        Self { fail_count }
    }
}

#[async_trait]
impl ModelConnector for MockFailingConnector {
    async fn generate(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ConnectorError> {
        let mut lock = self.fail_count.lock().await;
        if *lock > 0 {
            *lock -= 1;
            Err(ConnectorError::NetworkError(
                "Simulated connector failure".to_string(),
            ))
        } else {
            Ok(ChatCompletionResponse {
                id: "mock-id".to_string(),
                model: request.model,
                created: chrono::Utc::now().timestamp() as u64,
                choices: vec![ChatCompletionChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: "Mock response after recovery".to_string(),
                        name: None,
                        function_call: None,
                        tool_calls: None,
                    },
                    finish_reason: Some("stop".to_string()),
                }],
                usage: None,
            })
        }
    }
}

/// Create a test case for recovery from timeout
fn create_recovery_from_timeout_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(
            TestCategory::Integration,
            "recovery_from_timeout_test".to_string(),
        ),
        |ctx| {
            async move {
                info!("Running recovery from timeout test");

                // Create a mock connector that simulates timeouts
                let timeout_count = Arc::new(Mutex::new(2)); // Fail with timeout twice
                let mock_connector = MockTimeoutConnector::new(timeout_count.clone());

                // Create a retry manager with exponential backoff
                let mut retryable_errors = HashSet::new();
                retryable_errors.insert(ErrorCategory::Timeout);

                let retry_policy = RetryPolicy::ExponentialBackoff {
                    initial_interval_ms: 10, // Short for testing
                    backoff_factor: 2.0,
                    max_retries: 3,
                    max_interval_ms: 1000,
                };

                let circuit_breaker_config = CircuitBreakerConfig {
                    failure_threshold: 5,
                    success_threshold: 3,
                    reset_timeout_ms: 30000,
                    enabled: true,
                };

                let retry_manager =
                    RetryManager::new(retry_policy, circuit_breaker_config, retryable_errors);

                // Execute a function with retries
                let result = retry_manager
                    .execute(
                        || async {
                            let mut lock = timeout_count.lock().await;
                            if *lock > 0 {
                                *lock -= 1;
                                Err(RouterError::Timeout(
                                    "Request timed out after 5000ms".to_string(),
                                ))
                            } else {
                                Ok("Success after timeout recovery".to_string())
                            }
                        },
                        "timeout_recovery_test",
                    )
                    .await;

                // Verify the result
                AssertionHelper::assert_true(
                    result.is_ok(),
                    "Retry should eventually succeed after timeouts",
                )?;
                AssertionHelper::assert_eq(
                    result.unwrap(),
                    "Success after timeout recovery".to_string(),
                    "Result should be 'Success after timeout recovery'",
                )?;

                // Verify the timeout count is 0
                let final_count = *timeout_count.lock().await;
                AssertionHelper::assert_eq(
                    final_count,
                    0,
                    "Timeout count should be 0 after retries",
                )?;

                Ok(TestResult::new(
                    "recovery_from_timeout_test",
                    TestCategory::Integration,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Mock connector that simulates timeouts
struct MockTimeoutConnector {
    timeout_count: Arc<Mutex<usize>>,
}

impl MockTimeoutConnector {
    fn new(timeout_count: Arc<Mutex<usize>>) -> Self {
        Self { timeout_count }
    }
}

#[async_trait]
impl ModelConnector for MockTimeoutConnector {
    async fn generate(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ConnectorError> {
        let mut lock = self.timeout_count.lock().await;
        if *lock > 0 {
            *lock -= 1;
            Err(ConnectorError::TimeoutError(
                "Request timed out after 5000ms".to_string(),
            ))
        } else {
            Ok(ChatCompletionResponse {
                id: "mock-id".to_string(),
                model: request.model,
                created: chrono::Utc::now().timestamp() as u64,
                choices: vec![ChatCompletionChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: "Mock response after timeout recovery".to_string(),
                        name: None,
                        function_call: None,
                        tool_calls: None,
                    },
                    finish_reason: Some("stop".to_string()),
                }],
                usage: None,
            })
        }
    }
}

/// Create a test case for recovery from rate limit
fn create_recovery_from_rate_limit_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(
            TestCategory::Integration,
            "recovery_from_rate_limit_test".to_string(),
        ),
        |ctx| {
            async move {
                info!("Running recovery from rate limit test");

                // Create a mock connector that simulates rate limits
                let rate_limit_count = Arc::new(Mutex::new(2)); // Fail with rate limit twice
                let mock_connector = MockRateLimitConnector::new(rate_limit_count.clone());

                // Create a retry manager with exponential backoff
                let mut retryable_errors = HashSet::new();
                retryable_errors.insert(ErrorCategory::RateLimit);

                let retry_policy = RetryPolicy::ExponentialBackoff {
                    initial_interval_ms: 10, // Short for testing
                    backoff_factor: 2.0,
                    max_retries: 3,
                    max_interval_ms: 1000,
                };

                let circuit_breaker_config = CircuitBreakerConfig {
                    failure_threshold: 5,
                    success_threshold: 3,
                    reset_timeout_ms: 30000,
                    enabled: true,
                };

                let retry_manager =
                    RetryManager::new(retry_policy, circuit_breaker_config, retryable_errors);

                // Execute a function with retries
                let result = retry_manager
                    .execute(
                        || async {
                            let mut lock = rate_limit_count.lock().await;
                            if *lock > 0 {
                                *lock -= 1;
                                Err(RouterError::ConnectorError(
                                    "Rate limit exceeded: 100 requests per minute".to_string(),
                                ))
                            } else {
                                Ok("Success after rate limit recovery".to_string())
                            }
                        },
                        "rate_limit_recovery_test",
                    )
                    .await;

                // Verify the result
                AssertionHelper::assert_true(
                    result.is_ok(),
                    "Retry should eventually succeed after rate limits",
                )?;
                AssertionHelper::assert_eq(
                    result.unwrap(),
                    "Success after rate limit recovery".to_string(),
                    "Result should be 'Success after rate limit recovery'",
                )?;

                // Verify the rate limit count is 0
                let final_count = *rate_limit_count.lock().await;
                AssertionHelper::assert_eq(
                    final_count,
                    0,
                    "Rate limit count should be 0 after retries",
                )?;

                Ok(TestResult::new(
                    "recovery_from_rate_limit_test",
                    TestCategory::Integration,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Mock connector that simulates rate limits
struct MockRateLimitConnector {
    rate_limit_count: Arc<Mutex<usize>>,
}

impl MockRateLimitConnector {
    fn new(rate_limit_count: Arc<Mutex<usize>>) -> Self {
        Self { rate_limit_count }
    }
}

#[async_trait]
impl ModelConnector for MockRateLimitConnector {
    async fn generate(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ConnectorError> {
        let mut lock = self.rate_limit_count.lock().await;
        if *lock > 0 {
            *lock -= 1;
            Err(ConnectorError::RateLimitError(
                "Rate limit exceeded: 100 requests per minute".to_string(),
            ))
        } else {
            Ok(ChatCompletionResponse {
                id: "mock-id".to_string(),
                model: request.model,
                created: chrono::Utc::now().timestamp() as u64,
                choices: vec![ChatCompletionChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: "Mock response after rate limit recovery".to_string(),
                        name: None,
                        function_call: None,
                        tool_calls: None,
                    },
                    finish_reason: Some("stop".to_string()),
                }],
                usage: None,
            })
        }
    }
}

/// Create a test case for recovery from network error
fn create_recovery_from_network_error_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(
            TestCategory::Integration,
            "recovery_from_network_error_test".to_string(),
        ),
        |ctx| {
            async move {
                info!("Running recovery from network error test");

                // Create a mock connector that simulates network errors
                let network_error_count = Arc::new(Mutex::new(2)); // Fail with network error twice
                let mock_connector = MockNetworkErrorConnector::new(network_error_count.clone());

                // Create a retry manager with exponential backoff
                let mut retryable_errors = HashSet::new();
                retryable_errors.insert(ErrorCategory::Network);

                let retry_policy = RetryPolicy::ExponentialBackoff {
                    initial_interval_ms: 10, // Short for testing
                    backoff_factor: 2.0,
                    max_retries: 3,
                    max_interval_ms: 1000,
                };

                let circuit_breaker_config = CircuitBreakerConfig {
                    failure_threshold: 5,
                    success_threshold: 3,
                    reset_timeout_ms: 30000,
                    enabled: true,
                };

                let retry_manager =
                    RetryManager::new(retry_policy, circuit_breaker_config, retryable_errors);

                // Execute a function with retries
                let result = retry_manager
                    .execute(
                        || async {
                            let mut lock = network_error_count.lock().await;
                            if *lock > 0 {
                                *lock -= 1;
                                Err(RouterError::ConnectorError(
                                    "Network error: connection refused".to_string(),
                                ))
                            } else {
                                Ok("Success after network error recovery".to_string())
                            }
                        },
                        "network_error_recovery_test",
                    )
                    .await;

                // Verify the result
                AssertionHelper::assert_true(
                    result.is_ok(),
                    "Retry should eventually succeed after network errors",
                )?;
                AssertionHelper::assert_eq(
                    result.unwrap(),
                    "Success after network error recovery".to_string(),
                    "Result should be 'Success after network error recovery'",
                )?;

                // Verify the network error count is 0
                let final_count = *network_error_count.lock().await;
                AssertionHelper::assert_eq(
                    final_count,
                    0,
                    "Network error count should be 0 after retries",
                )?;

                Ok(TestResult::new(
                    "recovery_from_network_error_test",
                    TestCategory::Integration,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Mock connector that simulates network errors
struct MockNetworkErrorConnector {
    network_error_count: Arc<Mutex<usize>>,
}

impl MockNetworkErrorConnector {
    fn new(network_error_count: Arc<Mutex<usize>>) -> Self {
        Self {
            network_error_count,
        }
    }
}

#[async_trait]
impl ModelConnector for MockNetworkErrorConnector {
    async fn generate(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ConnectorError> {
        let mut lock = self.network_error_count.lock().await;
        if *lock > 0 {
            *lock -= 1;
            Err(ConnectorError::NetworkError(
                "Network error: connection refused".to_string(),
            ))
        } else {
            Ok(ChatCompletionResponse {
                id: "mock-id".to_string(),
                model: request.model,
                created: chrono::Utc::now().timestamp() as u64,
                choices: vec![ChatCompletionChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: "Mock response after network error recovery".to_string(),
                        name: None,
                        function_call: None,
                        tool_calls: None,
                    },
                    finish_reason: Some("stop".to_string()),
                }],
                usage: None,
            })
        }
    }
}

/// Create a test model registry
fn create_mock_model_registry() -> ModelRegistry {
    let mut registry = ModelRegistry::new();

    // Add test models
    let models = vec![
        ModelMetadata {
            id: "mock-model".to_string(),
            name: "Mock Model".to_string(),
            provider: "mock".to_string(),
            version: "1.0".to_string(),
            capabilities: vec!["text".to_string()],
            status: ModelStatus::Available,
            properties: serde_json::Map::new(),
        },
        ModelMetadata {
            id: "degraded-mode".to_string(),
            name: "Degraded Mode Model".to_string(),
            provider: "mock".to_string(),
            version: "1.0".to_string(),
            capabilities: vec!["text".to_string()],
            status: ModelStatus::Available,
            properties: serde_json::Map::new(),
        },
    ];

    for model in models {
        registry.add_model(model).unwrap();
    }

    // Add connectors
    registry
        .register_connector("mock-model", Box::new(MockConnector))
        .unwrap();
    registry
        .register_connector("degraded-mode", Box::new(MockConnector))
        .unwrap();

    registry
}
