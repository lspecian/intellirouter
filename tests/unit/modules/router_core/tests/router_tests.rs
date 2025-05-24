//! Router Implementation Tests
//!
//! This module contains tests for the router implementation.

use std::collections::HashMap;
use std::sync::Arc;

use crate::modules::model_registry::{
    connectors::{ChatCompletionRequest, ChatMessage, MessageRole},
    storage::ModelRegistry,
    ModelMetadata, ModelStatus, ModelType,
};
use crate::modules::router_core::{
    router::RouterImpl, Router, RouterConfig, RouterError, RoutingRequest, RoutingStrategy,
    StrategyConfig,
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

#[test]
fn test_router_initialization() {
    let config = RouterConfig::default();
    let registry = Arc::new(ModelRegistry::new());

    let router = RouterImpl::new(config.clone(), registry);
    assert!(router.is_ok());

    let mut router = router.unwrap();

    // Test initialization with different strategies
    let mut round_robin_config = config.clone();
    round_robin_config.strategy = RoutingStrategy::RoundRobin;
    let result = router.init(round_robin_config);
    assert!(result.is_ok());
    assert_eq!(router.get_config().strategy, RoutingStrategy::RoundRobin);

    let mut content_based_config = config.clone();
    content_based_config.strategy = RoutingStrategy::ContentBased;
    let result = router.init(content_based_config);
    assert!(result.is_ok());
    assert_eq!(router.get_config().strategy, RoutingStrategy::ContentBased);
}

#[test]
fn test_router_config_update() {
    let config = RouterConfig::default();
    let registry = Arc::new(ModelRegistry::new());

    let router = RouterImpl::new(config.clone(), registry).unwrap();

    // Test initial config
    assert_eq!(router.get_config().strategy, RoutingStrategy::ContentBased);
    assert!(router.get_config().cache_routing_decisions);

    // Create a new config
    let mut new_config = config.clone();
    new_config.strategy = RoutingStrategy::RoundRobin;
    new_config.cache_routing_decisions = false;

    // Update the config
    let mut router = router;
    let result = router.update_config(new_config);
    assert!(result.is_ok());

    // Check the updated config
    assert_eq!(router.get_config().strategy, RoutingStrategy::RoundRobin);
    assert!(!router.get_config().cache_routing_decisions);
}

#[test]
fn test_router_metrics() {
    let config = RouterConfig::default();
    let registry = Arc::new(ModelRegistry::new());

    let router = RouterImpl::new(config, registry).unwrap();

    // Initially, metrics should be empty
    let metrics = router.get_metrics();
    assert!(metrics.is_empty());

    // After routing, metrics should be populated
    // Note: We can't easily test this without mocking the model connector
    // This would be tested in integration tests
}

#[test]
fn test_router_cache() {
    let mut config = RouterConfig::default();
    config.cache_routing_decisions = true;
    config.max_cache_size = 10;

    let registry = Arc::new(ModelRegistry::new());
    let mut router = RouterImpl::new(config, registry).unwrap();

    // Clear the cache
    router.clear_cache();

    // Metrics should still be empty
    let metrics = router.get_metrics();
    assert!(metrics.is_empty());
}

// Note: More comprehensive tests would require mocking the model connector
// and testing the actual routing logic. This would be part of integration tests.
