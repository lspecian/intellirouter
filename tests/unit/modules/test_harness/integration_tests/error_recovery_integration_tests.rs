//! Error Recovery Integration Tests
//!
//! This module provides integration tests for error conditions and recovery scenarios
//! between different components of the system.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::modules::model_registry::{
    connectors::{
        ChatCompletionChoice, ChatCompletionRequest, ChatCompletionResponse, ChatMessage,
        ConnectorError, MessageRole, ModelConnector,
    },
    storage::ModelRegistry,
    ModelMetadata, ModelStatus,
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

/// Create a test suite for error recovery integration tests
pub fn create_error_recovery_integration_test_suite() -> TestSuite {
    let mut suite = TestSuite::new("Error Recovery Integration Tests").with_description(
        "Integration tests for error conditions and recovery scenarios between components",
    );

    // Add test cases
    suite = suite
        .with_test_case(create_router_retry_integration_test_case())
        .with_test_case(create_circuit_breaker_integration_test_case())
        .with_test_case(create_degraded_service_integration_test_case())
        .with_test_case(create_component_failure_recovery_test_case());

    suite
}

/// Create a test case for router retry integration
fn create_router_retry_integration_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(
            TestCategory::Integration,
            "router_retry_integration_test".to_string(),
        ),
        |ctx| {
            async move {
                info!("Running router retry integration test");

                // Create a model registry with test models
                let registry = Arc::new(create_test_model_registry());

                // Create a failing connector that will succeed after a few retries
                let fail_count = Arc::new(Mutex::new(2)); // Fail twice, then succeed
                let connector = Box::new(MockFailingConnector::new(fail_count.clone()));

                // Register the connector with the registry
                registry
                    .register_connector("test-model", connector)
                    .unwrap();

                // Create a router configuration with retry policy
                let router_config = RouterConfig {
                    strategy: RoutingStrategy::ModelPreference,
                    fallback_strategies: vec![RoutingStrategy::Random],
                    global_timeout_ms: 5000,
                    max_routing_attempts: 3,
                    cache_routing_decisions: true,
                    max_cache_size: 100,
                    collect_metrics: true,
                    retry_policy: RetryPolicy::ExponentialBackoff {
                        initial_interval_ms: 10, // Short for testing
                        backoff_factor: 2.0,
                        max_retries: 3,
                        max_interval_ms: 1000,
                    },
                    circuit_breaker: CircuitBreakerConfig {
                        failure_threshold: 5,
                        success_threshold: 3,
                        reset_timeout_ms: 30000,
                        enabled: true,
                    },
                    degraded_service_mode: DegradedServiceMode::DefaultModel(
                        "fallback-model".to_string(),
                    ),
                };

                // Create a router
                let router =
                    crate::modules::router_core::RouterImpl::new(registry.clone(), router_config);

                // Create a request
                let request = create_test_routing_request("test-model");

                // Route the request
                let result = router.route(request).await;

                // Verify the result
                AssertionHelper::assert_true(
                    result.is_ok(),
                    "Router should successfully route the request after retries",
                )?;

                let response = result.unwrap();
                AssertionHelper::assert_eq(
                    response.metadata.selected_model_id,
                    "test-model",
                    "Selected model should match the requested model",
                )?;

                AssertionHelper::assert_eq(
                    response.metadata.attempts,
                    3, // Initial attempt + 2 retries
                    "Number of attempts should match expected count",
                )?;

                // Verify the fail count is 0
                let final_count = *fail_count.lock().await;
                AssertionHelper::assert_eq(final_count, 0, "Fail count should be 0 after retries")?;

                Ok(TestResult::new(
                    "router_retry_integration_test",
                    TestCategory::Integration,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Create a test case for circuit breaker integration
fn create_circuit_breaker_integration_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(
            TestCategory::Integration,
            "circuit_breaker_integration_test".to_string(),
        ),
        |ctx| {
            async move {
                info!("Running circuit breaker integration test");

                // Create a model registry with test models
                let registry = Arc::new(create_test_model_registry());

                // Create a consistently failing connector
                let connector = Box::new(MockAlwaysFailingConnector::new());

                // Register the connector with the registry
                registry
                    .register_connector("failing-model", connector)
                    .unwrap();

                // Create a router configuration with a low circuit breaker threshold
                let router_config = RouterConfig {
                    strategy: RoutingStrategy::ModelPreference,
                    fallback_strategies: vec![RoutingStrategy::Random],
                    global_timeout_ms: 5000,
                    max_routing_attempts: 3,
                    cache_routing_decisions: true,
                    max_cache_size: 100,
                    collect_metrics: true,
                    retry_policy: RetryPolicy::ExponentialBackoff {
                        initial_interval_ms: 10, // Short for testing
                        backoff_factor: 2.0,
                        max_retries: 3,
                        max_interval_ms: 1000,
                    },
                    circuit_breaker: CircuitBreakerConfig {
                        failure_threshold: 2, // Open after 2 failures
                        success_threshold: 2,
                        reset_timeout_ms: 100, // Short for testing
                        enabled: true,
                    },
                    degraded_service_mode: DegradedServiceMode::DefaultModel(
                        "fallback-model".to_string(),
                    ),
                };

                // Create a router
                let router =
                    crate::modules::router_core::RouterImpl::new(registry.clone(), router_config);

                // Create a request
                let request = create_test_routing_request("failing-model");

                // First request - should fail but be retried
                let result1 = router.route(request.clone()).await;
                AssertionHelper::assert_true(
                    result1.is_ok(),
                    "First request should fall back to the default model",
                )?;

                let response1 = result1.unwrap();
                AssertionHelper::assert_eq(
                    response1.metadata.is_fallback,
                    true,
                    "Response should be marked as fallback",
                )?;

                // Second request - should fail but be retried
                let result2 = router.route(request.clone()).await;
                AssertionHelper::assert_true(
                    result2.is_ok(),
                    "Second request should fall back to the default model",
                )?;

                // Third request - circuit should be open, so it should immediately fall back
                let result3 = router.route(request.clone()).await;
                AssertionHelper::assert_true(
                    result3.is_ok(),
                    "Third request should immediately fall back due to open circuit",
                )?;

                let response3 = result3.unwrap();
                AssertionHelper::assert_eq(
                    response3.metadata.attempts,
                    1, // No retries, immediate fallback
                    "Number of attempts should be 1 with open circuit",
                )?;

                // Wait for the reset timeout
                tokio::time::sleep(Duration::from_millis(150)).await;

                // Register a working connector for the failing model
                registry
                    .register_connector("failing-model", Box::new(MockConnector::new()))
                    .unwrap();

                // Fourth request - circuit should be half-open, so it should try again
                let result4 = router.route(request.clone()).await;
                AssertionHelper::assert_true(
                    result4.is_ok(),
                    "Fourth request should succeed in half-open state",
                )?;

                let response4 = result4.unwrap();
                AssertionHelper::assert_eq(
                    response4.metadata.is_fallback,
                    false,
                    "Response should not be marked as fallback",
                )?;

                Ok(TestResult::new(
                    "circuit_breaker_integration_test",
                    TestCategory::Integration,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Create a test case for degraded service integration
fn create_degraded_service_integration_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(
            TestCategory::Integration,
            "degraded_service_integration_test".to_string(),
        ),
        |ctx| {
            async move {
                info!("Running degraded service integration test");

                // Create a model registry with test models
                let registry = Arc::new(create_test_model_registry());

                // Create router configurations with different degraded service modes
                let static_response_config = RouterConfig {
                    strategy: RoutingStrategy::ModelPreference,
                    fallback_strategies: vec![RoutingStrategy::Random],
                    global_timeout_ms: 5000,
                    max_routing_attempts: 3,
                    cache_routing_decisions: true,
                    max_cache_size: 100,
                    collect_metrics: true,
                    retry_policy: RetryPolicy::None, // No retries for this test
                    circuit_breaker: CircuitBreakerConfig {
                        failure_threshold: 5,
                        success_threshold: 3,
                        reset_timeout_ms: 30000,
                        enabled: false, // Disable circuit breaker for this test
                    },
                    degraded_service_mode: DegradedServiceMode::StaticResponse(
                        "Service is in degraded mode".to_string(),
                    ),
                };

                let default_model_config = RouterConfig {
                    strategy: RoutingStrategy::ModelPreference,
                    fallback_strategies: vec![RoutingStrategy::Random],
                    global_timeout_ms: 5000,
                    max_routing_attempts: 3,
                    cache_routing_decisions: true,
                    max_cache_size: 100,
                    collect_metrics: true,
                    retry_policy: RetryPolicy::None, // No retries for this test
                    circuit_breaker: CircuitBreakerConfig {
                        failure_threshold: 5,
                        success_threshold: 3,
                        reset_timeout_ms: 30000,
                        enabled: false, // Disable circuit breaker for this test
                    },
                    degraded_service_mode: DegradedServiceMode::DefaultModel(
                        "fallback-model".to_string(),
                    ),
                };

                let fail_fast_config = RouterConfig {
                    strategy: RoutingStrategy::ModelPreference,
                    fallback_strategies: vec![RoutingStrategy::Random],
                    global_timeout_ms: 5000,
                    max_routing_attempts: 3,
                    cache_routing_decisions: true,
                    max_cache_size: 100,
                    collect_metrics: true,
                    retry_policy: RetryPolicy::None, // No retries for this test
                    circuit_breaker: CircuitBreakerConfig {
                        failure_threshold: 5,
                        success_threshold: 3,
                        reset_timeout_ms: 30000,
                        enabled: false, // Disable circuit breaker for this test
                    },
                    degraded_service_mode: DegradedServiceMode::FailFast,
                };

                // Create routers with different degraded service modes
                let static_response_router = crate::modules::router_core::RouterImpl::new(
                    registry.clone(),
                    static_response_config,
                );
                let default_model_router = crate::modules::router_core::RouterImpl::new(
                    registry.clone(),
                    default_model_config,
                );
                let fail_fast_router = crate::modules::router_core::RouterImpl::new(
                    registry.clone(),
                    fail_fast_config,
                );

                // Create a request for a non-existent model
                let request = create_test_routing_request("non-existent-model");

                // Test static response mode
                let static_result = static_response_router.route(request.clone()).await;
                AssertionHelper::assert_true(
                    static_result.is_ok(),
                    "Static response mode should handle the request",
                )?;

                let static_response = static_result.unwrap();
                AssertionHelper::assert_eq(
                    static_response.metadata.is_fallback,
                    true,
                    "Response should be marked as fallback",
                )?;

                AssertionHelper::assert_eq(
                    static_response.response.choices[0].message.content,
                    "Service is in degraded mode",
                    "Response content should match the static response",
                )?;

                // Test default model mode
                let default_result = default_model_router.route(request.clone()).await;
                AssertionHelper::assert_true(
                    default_result.is_ok(),
                    "Default model mode should handle the request",
                )?;

                let default_response = default_result.unwrap();
                AssertionHelper::assert_eq(
                    default_response.metadata.is_fallback,
                    true,
                    "Response should be marked as fallback",
                )?;

                AssertionHelper::assert_eq(
                    default_response.metadata.selected_model_id,
                    "fallback-model",
                    "Selected model should be the fallback model",
                )?;

                // Test fail fast mode
                let fail_fast_result = fail_fast_router.route(request.clone()).await;
                AssertionHelper::assert_true(
                    fail_fast_result.is_err(),
                    "Fail fast mode should fail the request",
                )?;

                Ok(TestResult::new(
                    "degraded_service_integration_test",
                    TestCategory::Integration,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Create a test case for component failure recovery
fn create_component_failure_recovery_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(
            TestCategory::Integration,
            "component_failure_recovery_test".to_string(),
        ),
        |ctx| {
            async move {
                info!("Running component failure recovery test");

                // Create a model registry with test models
                let registry = Arc::new(create_test_model_registry());

                // Create a connector that simulates different types of failures
                let connector = Box::new(MockMultiFailureConnector::new());

                // Register the connector with the registry
                registry
                    .register_connector("multi-failure-model", connector)
                    .unwrap();

                // Create a router configuration with retry policy
                let router_config = RouterConfig {
                    strategy: RoutingStrategy::ModelPreference,
                    fallback_strategies: vec![RoutingStrategy::Random],
                    global_timeout_ms: 5000,
                    max_routing_attempts: 3,
                    cache_routing_decisions: true,
                    max_cache_size: 100,
                    collect_metrics: true,
                    retry_policy: RetryPolicy::ExponentialBackoff {
                        initial_interval_ms: 10, // Short for testing
                        backoff_factor: 2.0,
                        max_retries: 5, // Allow more retries for this test
                        max_interval_ms: 1000,
                    },
                    circuit_breaker: CircuitBreakerConfig {
                        failure_threshold: 10, // Higher threshold for this test
                        success_threshold: 3,
                        reset_timeout_ms: 30000,
                        enabled: true,
                    },
                    degraded_service_mode: DegradedServiceMode::DefaultModel(
                        "fallback-model".to_string(),
                    ),
                };

                // Create a router
                let router =
                    crate::modules::router_core::RouterImpl::new(registry.clone(), router_config);

                // Create a request
                let request = create_test_routing_request("multi-failure-model");

                // Route the request
                let result = router.route(request).await;

                // Verify the result
                AssertionHelper::assert_true(
                    result.is_ok(),
                    "Router should successfully route the request after recovering from failures",
                )?;

                let response = result.unwrap();
                AssertionHelper::assert_eq(
                    response.metadata.selected_model_id,
                    "multi-failure-model",
                    "Selected model should match the requested model",
                )?;

                AssertionHelper::assert_true(
                    response.metadata.attempts > 1,
                    "Number of attempts should be greater than 1",
                )?;

                Ok(TestResult::new(
                    "component_failure_recovery_test",
                    TestCategory::Integration,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Create a test routing request
fn create_test_routing_request(model: &str) -> RoutingRequest {
    use std::collections::HashMap;
    use std::time::Duration;

    // Create the chat completion request
    let chat_request = ChatCompletionRequest {
        model: model.to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: "Test message".to_string(),
            name: None,
            function_call: None,
            tool_calls: None,
        }],
        temperature: Some(0.7),
        top_p: Some(0.9),
        max_tokens: Some(100),
        stream: Some(false),
        functions: None,
        tools: None,
        additional_params: None,
    };

    // Create the routing context
    let mut parameters = HashMap::new();
    parameters.insert("request_id".to_string(), "test-request".to_string());
    parameters.insert("session_id".to_string(), "test-session".to_string());

    let context = RoutingContext {
        request: chat_request,
        user_id: Some("test-user".to_string()),
        org_id: None,
        timestamp: chrono::Utc::now(),
        priority: 0,
        tags: vec!["test".to_string()],
        parameters,
    };

    // Create the routing request
    RoutingRequest {
        context,
        model_filter: None,
        preferred_model_id: None,
        excluded_model_ids: Vec::new(),
        max_attempts: 3,
        timeout: Duration::from_secs(30),
    }
}

/// Create a test model registry
fn create_test_model_registry() -> ModelRegistry {
    let mut registry = ModelRegistry::new();

    // Add test models
    let models = vec![
        ModelMetadata {
            id: "test-model".to_string(),
            name: "Test Model".to_string(),
            provider: "test".to_string(),
            version: "1.0".to_string(),
            capabilities: vec!["text".to_string()],
            status: ModelStatus::Available,
            properties: serde_json::Map::new(),
        },
        ModelMetadata {
            id: "fallback-model".to_string(),
            name: "Fallback Model".to_string(),
            provider: "test".to_string(),
            version: "1.0".to_string(),
            capabilities: vec!["text".to_string()],
            status: ModelStatus::Available,
            properties: serde_json::Map::new(),
        },
        ModelMetadata {
            id: "multi-failure-model".to_string(),
            name: "Multi-Failure Model".to_string(),
            provider: "test".to_string(),
            version: "1.0".to_string(),
            capabilities: vec!["text".to_string()],
            status: ModelStatus::Available,
            properties: serde_json::Map::new(),
        },
    ];

    for model in models {
        registry.add_model(model).unwrap();
    }

    // Add a connector for the fallback model
    registry
        .register_connector("fallback-model", Box::new(MockConnector::new()))
        .unwrap();

    registry
}

/// Mock connector
struct MockConnector;

impl MockConnector {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ModelConnector for MockConnector {
    async fn generate(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ConnectorError> {
        Ok(ChatCompletionResponse {
            id: "mock-id".to_string(),
            model: request.model,
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

    async fn generate_streaming(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<crate::modules::model_registry::connectors::StreamingResponse, ConnectorError> {
        Err(ConnectorError::UnsupportedOperation(
            "Streaming not supported in mock connector".to_string(),
        ))
    }

    fn get_config(&self) -> &crate::modules::model_registry::connectors::ConnectorConfig {
        static CONFIG: once_cell::sync::Lazy<
            crate::modules::model_registry::connectors::ConnectorConfig,
        > = once_cell::sync::Lazy::new(|| {
            crate::modules::model_registry::connectors::ConnectorConfig::default()
        });
        &CONFIG
    }

    fn update_config(
        &mut self,
        _config: crate::modules::model_registry::connectors::ConnectorConfig,
    ) {
        // No-op for mock
    }

    fn provider_name(&self) -> &'static str {
        "mock"
    }

    fn supports_model(&self, _model_id: &str) -> bool {
        true
    }

    async fn list_models(&self) -> Result<Vec<String>, ConnectorError> {
        Ok(vec!["mock-model".to_string()])
    }
}

/// Mock failing connector
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
                "Simulated network error".to_string(),
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

    async fn generate_streaming(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<crate::modules::model_registry::connectors::StreamingResponse, ConnectorError> {
        Err(ConnectorError::UnsupportedOperation(
            "Streaming not supported in mock connector".to_string(),
        ))
    }

    fn get_config(&self) -> &crate::modules::model_registry::connectors::ConnectorConfig {
        static CONFIG: once_cell::sync::Lazy<
            crate::modules::model_registry::connectors::ConnectorConfig,
        > = once_cell::sync::Lazy::new(|| {
            crate::modules::model_registry::connectors::ConnectorConfig::default()
        });
        &CONFIG
    }

    fn update_config(
        &mut self,
        _config: crate::modules::model_registry::connectors::ConnectorConfig,
    ) {
        // No-op for mock
    }

    fn provider_name(&self) -> &'static str {
        "mock"
    }

    fn supports_model(&self, _model_id: &str) -> bool {
        true
    }

    async fn list_models(&self) -> Result<Vec<String>, ConnectorError> {
        Ok(vec!["mock-model".to_string()])
    }
}

/// Mock always failing connector
struct MockAlwaysFailingConnector;

impl MockAlwaysFailingConnector {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ModelConnector for MockAlwaysFailingConnector {
    async fn generate(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ConnectorError> {
        Err(ConnectorError::NetworkError(
            "Simulated persistent network error".to_string(),
        ))
    }

    async fn generate_streaming(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<crate::modules::model_registry::connectors::StreamingResponse, ConnectorError> {
        Err(ConnectorError::UnsupportedOperation(
            "Streaming not supported in mock connector".to_string(),
        ))
    }

    fn get_config(&self) -> &crate::modules::model_registry::connectors::ConnectorConfig {
        static CONFIG: once_cell::sync::Lazy<
            crate::modules::model_registry::connectors::ConnectorConfig,
        > = once_cell::sync::Lazy::new(|| {
            crate::modules::model_registry::connectors::ConnectorConfig::default()
        });
        &CONFIG
    }

    fn update_config(
        &mut self,
        _config: crate::modules::model_registry::connectors::ConnectorConfig,
    ) {
        // No-op for mock
    }

    fn provider_name(&self) -> &'static str {
        "mock"
    }

    fn supports_model(&self, _model_id: &str) -> bool {
        true
    }

    async fn list_models(&self) -> Result<Vec<String>, ConnectorError> {
        Ok(vec!["mock-model".to_string()])
    }
}

/// Mock connector that simulates different types of failures
struct MockMultiFailureConnector {
    failure_count: std::sync::atomic::AtomicUsize,
}

impl MockMultiFailureConnector {
    fn new() -> Self {
        Self {
            failure_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl ModelConnector for MockMultiFailureConnector {
    async fn generate(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ConnectorError> {
        let count = self
            .failure_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        match count % 4 {
            0 => Err(ConnectorError::NetworkError(
                "Simulated network error".to_string(),
            )),
            1 => Err(ConnectorError::TimeoutError(
                "Simulated timeout error".to_string(),
            )),
            2 => Err(ConnectorError::RateLimitError(
                "Simulated rate limit error".to_string(),
            )),
            _ => Ok(ChatCompletionResponse {
                id: "mock-id".to_string(),
                model: request.model,
                created: chrono::Utc::now().timestamp() as u64,
                choices: vec![ChatCompletionChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: "Mock response after multiple failures".to_string(),
                        name: None,
                        function_call: None,
                        tool_calls: None,
                    },
                    finish_reason: Some("stop".to_string()),
                }],
                usage: None,
            }),
        }
    }

    async fn generate_streaming(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<crate::modules::model_registry::connectors::StreamingResponse, ConnectorError> {
        Err(ConnectorError::UnsupportedOperation(
            "Streaming not supported in mock connector".to_string(),
        ))
    }

    fn get_config(&self) -> &crate::modules::model_registry::connectors::ConnectorConfig {
        static CONFIG: once_cell::sync::Lazy<
            crate::modules::model_registry::connectors::ConnectorConfig,
        > = once_cell::sync::Lazy::new(|| {
            crate::modules::model_registry::connectors::ConnectorConfig::default()
        });
        &CONFIG
    }

    fn update_config(
        &mut self,
        _config: crate::modules::model_registry::connectors::ConnectorConfig,
    ) {
        // No-op for mock
    }

    fn provider_name(&self) -> &'static str {
        "mock"
    }

    fn supports_model(&self, _model_id: &str) -> bool {
        true
    }

    async fn list_models(&self) -> Result<Vec<String>, ConnectorError> {
        Ok(vec!["mock-model".to_string()])
    }
}
