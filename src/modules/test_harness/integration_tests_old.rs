//! Integration Tests Module
//!
//! This module provides test cases for integration testing between components.

use std::collections::HashMap;
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

/// Create a test suite for integration tests
pub fn create_integration_test_suite() -> TestSuite {
    let mut suite = TestSuite::new("Integration Tests")
        .with_description("Tests for integration between components");

    // Add test cases
    suite = suite
        .with_test_case(create_router_model_registry_test_case())
        .with_test_case(create_router_connector_test_case())
        .with_test_case(create_health_check_integration_test_case())
        .with_test_case(create_end_to_end_request_flow_test_case());

    suite
}

/// Create a test case for router and model registry integration
fn create_router_model_registry_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(
            TestCategory::Integration,
            "router_model_registry_test".to_string(),
        ),
        |ctx| {
            async move {
                info!("Running router and model registry integration test");

                // Create a model registry with test models
                let registry = Arc::new(create_test_model_registry());

                // Create a router configuration
                let router_config = RouterConfig {
                    strategy: RoutingStrategy::ModelPreference,
                    fallback_strategies: vec![RoutingStrategy::Random],
                    global_timeout_ms: 5000,
                    max_routing_attempts: 3,
                    cache_routing_decisions: true,
                    max_cache_size: 100,
                    collect_metrics: true,
                    retry_policy: RetryPolicy::ExponentialBackoff {
                        initial_interval_ms: 100,
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
                    crate::modules::router_core::Router::new(registry.clone(), router_config);

                // Create a request
                let request = create_test_routing_request("test-model-1");

                // Route the request
                let result = router.route(request).await;

                // Verify the result
                AssertionHelper::assert_true(
                    result.is_ok(),
                    "Router should successfully route the request",
                )?;

                let response = result.unwrap();
                AssertionHelper::assert_eq(
                    response.metadata.selected_model_id,
                    "test-model-1",
                    "Selected model should match the requested model",
                )?;

                // Test with a non-existent model
                let request = create_test_routing_request("non-existent-model");
                let result = router.route(request).await;

                // Verify that it falls back to the default model
                AssertionHelper::assert_true(
                    result.is_ok(),
                    "Router should fall back to the default model",
                )?;

                let response = result.unwrap();
                AssertionHelper::assert_eq(
                    response.metadata.is_fallback,
                    true,
                    "Response should be marked as fallback",
                )?;

                AssertionHelper::assert_eq(
                    response.metadata.selected_model_id,
                    "fallback-model",
                    "Selected model should be the fallback model",
                )?;

                Ok(TestResult::new(
                    "router_model_registry_test",
                    TestCategory::Integration,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Create a test case for router and connector integration
fn create_router_connector_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(
            TestCategory::Integration,
            "router_connector_test".to_string(),
        ),
        |ctx| {
            async move {
                info!("Running router and connector integration test");

                // Create a model registry with test models and connectors
                let registry = Arc::new(create_test_model_registry_with_connectors());

                // Create a router configuration
                let router_config = RouterConfig {
                    strategy: RoutingStrategy::ModelPreference,
                    fallback_strategies: vec![RoutingStrategy::Random],
                    global_timeout_ms: 5000,
                    max_routing_attempts: 3,
                    cache_routing_decisions: true,
                    max_cache_size: 100,
                    collect_metrics: true,
                    retry_policy: RetryPolicy::ExponentialBackoff {
                        initial_interval_ms: 100,
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
                    crate::modules::router_core::Router::new(registry.clone(), router_config);

                // Create a request for a model with a working connector
                let request = create_test_routing_request("test-model-1");

                // Route the request
                let result = router.route(request).await;

                // Verify the result
                AssertionHelper::assert_true(
                    result.is_ok(),
                    "Router should successfully route the request to a working connector",
                )?;

                let response = result.unwrap();
                AssertionHelper::assert_eq(
                    response.metadata.selected_model_id,
                    "test-model-1",
                    "Selected model should match the requested model",
                )?;

                // Create a request for a model with a failing connector
                let request = create_test_routing_request("failing-model");

                // Route the request
                let result = router.route(request).await;

                // Verify that it falls back to another model
                AssertionHelper::assert_true(
                    result.is_ok(),
                    "Router should fall back to another model when connector fails",
                )?;

                let response = result.unwrap();
                AssertionHelper::assert_eq(
                    response.metadata.is_fallback,
                    true,
                    "Response should be marked as fallback",
                )?;

                AssertionHelper::assert_ne(
                    response.metadata.selected_model_id,
                    "failing-model",
                    "Selected model should not be the failing model",
                )?;

                Ok(TestResult::new(
                    "router_connector_test",
                    TestCategory::Integration,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Create a test case for health check integration
fn create_health_check_integration_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(
            TestCategory::Integration,
            "health_check_integration_test".to_string(),
        ),
        |ctx| {
            async move {
                info!("Running health check integration test");

                // Create a model registry with test models
                let registry = Arc::new(create_test_model_registry());

                // Create a router configuration
                let router_config = RouterConfig {
                    strategy: RoutingStrategy::ModelPreference,
                    fallback_strategies: vec![RoutingStrategy::Random],
                    global_timeout_ms: 5000,
                    max_routing_attempts: 3,
                    cache_routing_decisions: true,
                    max_cache_size: 100,
                    collect_metrics: true,
                    retry_policy: RetryPolicy::ExponentialBackoff {
                        initial_interval_ms: 100,
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

                // Create a health check manager
                let health_manager = crate::modules::health::create_router_health_manager(
                    registry.clone(),
                    router_config,
                    None,
                );

                // Check health
                let health_result = health_manager.check_health().await;

                // Verify the result
                AssertionHelper::assert_true(
                    health_result.is_healthy,
                    "Health check should report healthy",
                )?;

                // Get diagnostics
                let diagnostics = health_manager.get_diagnostics(2).await.unwrap();

                // Verify diagnostics
                AssertionHelper::assert_true(
                    diagnostics.contains_key("total_models"),
                    "Diagnostics should include total_models",
                )?;

                AssertionHelper::assert_true(
                    diagnostics.contains_key("available_models"),
                    "Diagnostics should include available_models",
                )?;

                AssertionHelper::assert_true(
                    diagnostics.contains_key("routing_strategy"),
                    "Diagnostics should include routing_strategy",
                )?;

                // Verify that the number of models matches
                let total_models = diagnostics["total_models"].as_u64().unwrap();
                AssertionHelper::assert_eq(
                    total_models,
                    3, // test-model-1, test-model-2, fallback-model
                    "Total models should match expected count",
                )?;

                Ok(TestResult::new(
                    "health_check_integration_test",
                    TestCategory::Integration,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Create a test case for end-to-end request flow
fn create_end_to_end_request_flow_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(
            TestCategory::Integration,
            "end_to_end_request_flow_test".to_string(),
        ),
        |ctx| {
            async move {
                info!("Running end-to-end request flow test");

                // Create a model registry with test models and connectors
                let registry = Arc::new(create_test_model_registry_with_connectors());

                // Create a router configuration
                let router_config = RouterConfig {
                    strategy: RoutingStrategy::ModelPreference,
                    fallback_strategies: vec![RoutingStrategy::Random],
                    global_timeout_ms: 5000,
                    max_routing_attempts: 3,
                    cache_routing_decisions: true,
                    max_cache_size: 100,
                    collect_metrics: true,
                    retry_policy: RetryPolicy::ExponentialBackoff {
                        initial_interval_ms: 100,
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
                    crate::modules::router_core::Router::new(registry.clone(), router_config);

                // Create a request
                let request = create_test_routing_request("test-model-1");

                // Route the request
                let result = router.route(request).await;

                // Verify the result
                AssertionHelper::assert_true(
                    result.is_ok(),
                    "Router should successfully route the request",
                )?;

                let response = result.unwrap();
                AssertionHelper::assert_eq(
                    response.metadata.selected_model_id,
                    "test-model-1",
                    "Selected model should match the requested model",
                )?;

                // Verify the response content
                AssertionHelper::assert_true(
                    !response.response.choices.is_empty(),
                    "Response should contain choices",
                )?;

                let message = &response.response.choices[0].message;
                AssertionHelper::assert_eq(
                    message.role,
                    MessageRole::Assistant,
                    "Message role should be Assistant",
                )?;

                AssertionHelper::assert_true(
                    !message.content.is_empty(),
                    "Message content should not be empty",
                )?;

                // Test the full flow with a sequence of requests
                let models = vec!["test-model-1", "test-model-2", "fallback-model"];
                let mut conversation = Vec::new();

                for (i, model) in models.iter().enumerate() {
                    // Add the previous response to the conversation
                    if i > 0 {
                        conversation.push(ChatMessage {
                            role: MessageRole::Assistant,
                            content: format!("Response {}", i - 1),
                            name: None,
                            function_call: None,
                            tool_calls: None,
                        });
                    }

                    // Add a new user message
                    conversation.push(ChatMessage {
                        role: MessageRole::User,
                        content: format!("User message {}", i),
                        name: None,
                        function_call: None,
                        tool_calls: None,
                    });

                    // Create a request with the conversation
                    let request = RoutingRequest {
                        context: RoutingContext {
                            request: ChatCompletionRequest {
                                model: model.to_string(),
                                messages: conversation.clone(),
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
                            request_id: format!("test-request-{}", i),
                            timestamp: chrono::Utc::now(),
                            additional_context: serde_json::Map::new(),
                        },
                        constraints: None,
                    };

                    // Route the request
                    let result = router.route(request).await;
                    AssertionHelper::assert_true(
                        result.is_ok(),
                        &format!("Router should successfully route request {}", i),
                    )?;
                }

                Ok(TestResult::new(
                    "end_to_end_request_flow_test",
                    TestCategory::Integration,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Create a test model registry
fn create_test_model_registry() -> ModelRegistry {
    let mut registry = ModelRegistry::new();

    // Add test models
    let models = vec![
        ModelMetadata {
            id: "test-model-1".to_string(),
            name: "Test Model 1".to_string(),
            provider: "test".to_string(),
            version: "1.0".to_string(),
            capabilities: vec!["text".to_string()],
            status: ModelStatus::Available,
            properties: serde_json::Map::new(),
        },
        ModelMetadata {
            id: "test-model-2".to_string(),
            name: "Test Model 2".to_string(),
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
    ];

    for model in models {
        registry.add_model(model).unwrap();
    }

    registry
}

/// Create a test model registry with connectors
fn create_test_model_registry_with_connectors() -> ModelRegistry {
    let mut registry = create_test_model_registry();

    // Add connectors
    registry
        .register_connector("test-model-1", Box::new(TestConnector::new(false)))
        .unwrap();
    registry
        .register_connector("test-model-2", Box::new(TestConnector::new(false)))
        .unwrap();
    registry
        .register_connector("fallback-model", Box::new(TestConnector::new(false)))
        .unwrap();
    registry
        .register_connector("failing-model", Box::new(TestConnector::new(true)))
        .unwrap();

    registry
}

/// Create a test routing request
fn create_test_routing_request(model: &str) -> RoutingRequest {
    RoutingRequest {
        context: RoutingContext {
            request: ChatCompletionRequest {
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

/// Test connector
struct TestConnector {
    should_fail: bool,
}

impl TestConnector {
    fn new(should_fail: bool) -> Self {
        Self { should_fail }
    }
}

#[async_trait]
impl ModelConnector for TestConnector {
    async fn generate(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ConnectorError> {
        if self.should_fail {
            return Err(ConnectorError::NetworkError(
                "Simulated connector failure".to_string(),
            ));
        }

        // Create a response based on the request
        let model = request.model.clone();
        let user_message = request.messages.last().map_or("", |m| m.content.as_str());

        Ok(ChatCompletionResponse {
            id: format!("response-{}", chrono::Utc::now().timestamp()),
            model,
            created: chrono::Utc::now().timestamp() as u64,
            choices: vec![ChatCompletionChoice {
                index: 0,
                message: ChatMessage {
                    role: MessageRole::Assistant,
                    content: format!("Response to: {}", user_message),
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
