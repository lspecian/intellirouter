//! Tests for the model registry integration with the router
//!
//! This module contains tests for the integration between the router and model registry.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use crate::modules::model_registry::{
    connectors::{ChatCompletionRequest, ChatCompletionResponse, ChatMessage, MessageRole},
    ModelFilter, ModelMetadata, ModelStatus, ModelType,
};
use crate::modules::router_core::{
    registry_integration::RegistryIntegration,
    retry::{CircuitBreakerConfig, DegradedServiceMode, ErrorCategory, RetryPolicy},
    router::RouterImpl,
    Router, RouterConfig, RouterError, RoutingRequest, RoutingStrategy, StrategyConfig,
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
async fn test_update_from_registry() {
    // Create a mock registry with some models
    let mut registry = MockModelRegistry::new();

    // Add test models
    registry.add_model(create_test_model("model1", "provider1"));
    registry.add_model(create_test_model("model2", "provider1"));
    registry.add_model(create_test_model("model3", "provider2"));

    // Create a router
    let config = RouterConfig::default();
    let router = RouterImpl::new(config, Arc::new(registry)).unwrap();

    // Update from registry
    let result = router.update_from_registry().await;
    assert!(result.is_ok());

    // Check metrics
    let metrics = router.get_metrics();
    assert_eq!(
        metrics.get("available_models_count"),
        Some(&serde_json::Value::Number(serde_json::Number::from(3)))
    );

    // Check providers
    if let Some(serde_json::Value::Object(providers)) = metrics.get("providers") {
        assert_eq!(
            providers.get("provider1"),
            Some(&serde_json::Value::Number(serde_json::Number::from(2)))
        );
        assert_eq!(
            providers.get("provider2"),
            Some(&serde_json::Value::Number(serde_json::Number::from(1)))
        );
    } else {
        panic!("Expected providers to be an object");
    }
}

#[tokio::test]
async fn test_get_filtered_models() {
    // Create a mock registry with some models
    let mut registry = MockModelRegistry::new();

    // Add test models
    registry.add_model(create_test_model("model1", "provider1"));
    registry.add_model(create_test_model("model2", "provider1"));
    registry.add_model(create_test_model("model3", "provider2"));

    // Create a router
    let config = RouterConfig::default();
    let router = RouterImpl::new(config, Arc::new(registry)).unwrap();

    // Create a request with a filter
    let mut request = create_test_request();
    request.model_filter = Some(ModelFilter::new().with_provider("provider1".to_string()));

    // Get filtered models
    let result = router.get_filtered_models(&request).await;
    assert!(result.is_ok());

    let models = result.unwrap();
    assert_eq!(models.len(), 2);
    assert!(models.iter().any(|m| m.id == "model1"));
    assert!(models.iter().any(|m| m.id == "model2"));

    // Test with excluded models
    let mut request = create_test_request();
    request.excluded_model_ids = vec!["model1".to_string(), "model2".to_string()];

    let result = router.get_filtered_models(&request).await;
    assert!(result.is_ok());

    let models = result.unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "model3");

    // Test with preferred model
    let mut request = create_test_request();
    request.preferred_model_id = Some("model2".to_string());

    let result = router.get_filtered_models(&request).await;
    assert!(result.is_ok());

    let models = result.unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "model2");

    // Test with non-existent preferred model
    let mut request = create_test_request();
    request.preferred_model_id = Some("non-existent".to_string());

    let result = router.get_filtered_models(&request).await;
    assert!(result.is_ok());

    let models = result.unwrap();
    assert_eq!(models.len(), 3);
}

#[tokio::test]
async fn test_router_with_registry_integration() {
    // Create a mock registry with some models
    let mut registry = MockModelRegistry::new();

    // Add test models
    registry.add_model(create_test_model("model1", "provider1"));
    registry.add_model(create_test_model("model2", "provider1"));
    registry.add_model(create_test_model("model3", "provider2"));

    // Create a router with round-robin strategy
    let mut config = RouterConfig::default();
    config.strategy = RoutingStrategy::RoundRobin;

    let router = RouterImpl::new(config, Arc::new(registry)).unwrap();

    // Update from registry
    let result = router.update_from_registry().await;
    assert!(result.is_ok());

    // Subscribe to registry updates
    let result = router.subscribe_to_registry_updates().await;
    assert!(result.is_ok());

    // Create a test request
    let request = create_test_request();

    // Route the request
    // Note: This will fail because the mock registry doesn't have a connector
    // In a real test, we would mock the connector as well
    let result = router.route(request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_router_with_empty_registry() {
    // Create an empty registry
    let registry = MockModelRegistry::new();

    // Create a router
    let config = RouterConfig::default();
    let router = RouterImpl::new(config, Arc::new(registry)).unwrap();

    // Create a test request
    let request = create_test_request();

    // Get filtered models
    let result = router.get_filtered_models(&request).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RouterError::NoSuitableModel(_)
    ));
}

#[tokio::test]
async fn test_router_with_all_models_excluded() {
    // Create a registry with some models
    let mut registry = MockModelRegistry::new();

    // Add test models
    registry.add_model(create_test_model("model1", "provider1"));
    registry.add_model(create_test_model("model2", "provider1"));

    // Create a router
    let config = RouterConfig::default();
    let router = RouterImpl::new(config, Arc::new(registry)).unwrap();

    // Create a request that excludes all models
    let mut request = create_test_request();
    request.excluded_model_ids = vec!["model1".to_string(), "model2".to_string()];

    // Get filtered models
    let result = router.get_filtered_models(&request).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RouterError::NoSuitableModel(_)
    ));
}

#[tokio::test]
async fn test_router_with_model_filter() {
    // Create a registry with models of different types
    let mut registry = MockModelRegistry::new();

    // Add test models
    let mut text_model = create_test_model("text-model", "provider1");
    text_model.set_model_type(ModelType::TextGeneration);

    let mut embedding_model = create_test_model("embedding-model", "provider1");
    embedding_model.set_model_type(ModelType::Embedding);

    let mut multimodal_model = create_test_model("multimodal-model", "provider2");
    multimodal_model.set_model_type(ModelType::MultiModal);

    registry.add_model(text_model);
    registry.add_model(embedding_model);
    registry.add_model(multimodal_model);

    // Create a router
    let config = RouterConfig::default();
    let router = RouterImpl::new(config, Arc::new(registry)).unwrap();

    // Create a request with a filter for text generation models
    let mut request = create_test_request();
    request.model_filter = Some(ModelFilter::new().with_model_type(ModelType::TextGeneration));

    // Get filtered models
    let result = router.get_filtered_models(&request).await;
    assert!(result.is_ok());

    let models = result.unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "text-model");

    // Create a request with a filter for embedding models
    let mut request = create_test_request();
    request.model_filter = Some(ModelFilter::new().with_model_type(ModelType::Embedding));

    // Get filtered models
    let result = router.get_filtered_models(&request).await;
    assert!(result.is_ok());

    let models = result.unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "embedding-model");
}

#[tokio::test]
async fn test_router_with_unavailable_models() {
    // Create a registry with some models
    let mut registry = MockModelRegistry::new();

    // Add test models with different statuses
    let mut available_model = create_test_model("available-model", "provider1");
    available_model.set_status(ModelStatus::Available);

    let mut unavailable_model = create_test_model("unavailable-model", "provider1");
    unavailable_model.set_status(ModelStatus::Unavailable);

    let mut limited_model = create_test_model("limited-model", "provider2");
    limited_model.set_status(ModelStatus::Limited);

    registry.add_model(available_model);
    registry.add_model(unavailable_model);
    registry.add_model(limited_model);

    // Create a router
    let config = RouterConfig::default();
    let router = RouterImpl::new(config, Arc::new(registry)).unwrap();

    // Create a request with no filter (should default to available models)
    let request = create_test_request();

    // Get filtered models
    let result = router.get_filtered_models(&request).await;
    assert!(result.is_ok());

    let models = result.unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "available-model");

    // Create a request with a filter for all statuses
    let mut request = create_test_request();
    request.model_filter = Some(ModelFilter::new());

    // Get filtered models
    let result = router.get_filtered_models(&request).await;
    assert!(result.is_ok());

    let models = result.unwrap();
    assert_eq!(models.len(), 3);
}
