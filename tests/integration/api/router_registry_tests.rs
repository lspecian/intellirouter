//! Integration tests for the Router and Model Registry
//!
//! These tests verify that the Router works correctly with the Model Registry.

use intellirouter::{
    modules::{
        model_registry::{
            connectors::{ChatCompletionRequest, ChatMessage, MessageRole},
            ModelFilter, ModelMetadata, ModelRegistry, ModelStatus, ModelType,
        },
        router_core::{RouterConfig, RouterImpl, RoutingRequest, RoutingStrategy, StrategyConfig},
    },
    test_utils::init_test_logging,
};
use std::sync::Arc;
use std::time::Duration;

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
async fn test_router_with_registry() {
    // Initialize test environment
    init_test_logging();

    // Create a registry with test models
    let registry = Arc::new(ModelRegistry::new());

    // Add test models
    registry
        .register_model(create_test_model("model1", "provider1"))
        .unwrap();
    registry
        .register_model(create_test_model("model2", "provider1"))
        .unwrap();
    registry
        .register_model(create_test_model("model3", "provider2"))
        .unwrap();

    // Create a router
    let config = RouterConfig::default();
    let router = RouterImpl::new(config, registry.clone()).unwrap();

    // Update from registry
    let result = router.update_from_registry().await;
    assert!(result.is_ok());

    // Get filtered models
    let request = create_test_request();
    let result = router.get_filtered_models(&request).await;
    assert!(result.is_ok());

    let models = result.unwrap();
    assert_eq!(models.len(), 3);
}

#[tokio::test]
async fn test_router_with_different_strategies() {
    // Initialize test environment
    init_test_logging();

    // Create a registry with test models
    let registry = Arc::new(ModelRegistry::new());

    // Add test models
    registry
        .register_model(create_test_model("model1", "provider1"))
        .unwrap();
    registry
        .register_model(create_test_model("model2", "provider1"))
        .unwrap();
    registry
        .register_model(create_test_model("model3", "provider2"))
        .unwrap();

    // Test with round-robin strategy
    let mut config = RouterConfig::default();
    config.strategy = RoutingStrategy::RoundRobin;

    let router = RouterImpl::new(config, registry.clone()).unwrap();

    // Update from registry
    let result = router.update_from_registry().await;
    assert!(result.is_ok());

    // Test with content-based strategy
    let mut config = RouterConfig::default();
    config.strategy = RoutingStrategy::ContentBased;

    let router = RouterImpl::new(config, registry.clone()).unwrap();

    // Update from registry
    let result = router.update_from_registry().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_router_with_model_filtering() {
    // Initialize test environment
    init_test_logging();

    // Create a registry with models of different types
    let registry = Arc::new(ModelRegistry::new());

    // Add test models
    let mut text_model = create_test_model("text-model", "provider1");
    text_model.set_model_type(ModelType::TextGeneration);

    let mut embedding_model = create_test_model("embedding-model", "provider1");
    embedding_model.set_model_type(ModelType::Embedding);

    let mut multimodal_model = create_test_model("multimodal-model", "provider2");
    multimodal_model.set_model_type(ModelType::MultiModal);

    registry.register_model(text_model).unwrap();
    registry.register_model(embedding_model).unwrap();
    registry.register_model(multimodal_model).unwrap();

    // Create a router
    let config = RouterConfig::default();
    let router = RouterImpl::new(config, registry.clone()).unwrap();

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
async fn test_router_with_model_status_changes() {
    // Initialize test environment
    init_test_logging();

    // Create a registry with some models
    let registry = Arc::new(ModelRegistry::new());

    // Add test models
    registry
        .register_model(create_test_model("model1", "provider1"))
        .unwrap();
    registry
        .register_model(create_test_model("model2", "provider1"))
        .unwrap();

    // Create a router
    let config = RouterConfig::default();
    let router = RouterImpl::new(config, registry.clone()).unwrap();

    // Update from registry
    let result = router.update_from_registry().await;
    assert!(result.is_ok());

    // Get filtered models (should return both models)
    let request = create_test_request();
    let result = router.get_filtered_models(&request).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);

    // Change status of model1 to Unavailable
    registry
        .update_model_status("model1", ModelStatus::Unavailable)
        .unwrap();

    // Get filtered models again (should only return model2)
    let request = create_test_request();
    let result = router.get_filtered_models(&request).await;
    assert!(result.is_ok());

    let models = result.unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "model2");
}

#[tokio::test]
async fn test_router_with_model_capabilities() {
    // Initialize test environment
    init_test_logging();

    // Create a registry with models having different capabilities
    let registry = Arc::new(ModelRegistry::new());

    // Add test models
    let mut model1 = create_test_model("model1", "provider1");
    model1.capabilities.supports_function_calling = true;
    model1.capabilities.max_context_length = 4096;

    let mut model2 = create_test_model("model2", "provider1");
    model2.capabilities.supports_function_calling = false;
    model2.capabilities.max_context_length = 8192;

    let mut model3 = create_test_model("model3", "provider2");
    model3.capabilities.supports_function_calling = true;
    model3.capabilities.max_context_length = 16384;

    registry.register_model(model1).unwrap();
    registry.register_model(model2).unwrap();
    registry.register_model(model3).unwrap();

    // Create a router
    let config = RouterConfig::default();
    let router = RouterImpl::new(config, registry.clone()).unwrap();

    // Create a request with a filter for models with function calling
    let mut request = create_test_request();
    request.model_filter = Some(ModelFilter::new().with_function_calling(true));

    // Get filtered models
    let result = router.get_filtered_models(&request).await;
    assert!(result.is_ok());

    let models = result.unwrap();
    assert_eq!(models.len(), 2);
    assert!(models.iter().any(|m| m.id == "model1"));
    assert!(models.iter().any(|m| m.id == "model3"));

    // Create a request with a filter for models with large context
    let mut request = create_test_request();
    request.model_filter = Some(ModelFilter::new().with_min_context_length(10000));

    // Get filtered models
    let result = router.get_filtered_models(&request).await;
    assert!(result.is_ok());

    let models = result.unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "model3");
}

#[tokio::test]
async fn test_router_performance_with_many_models() {
    // Initialize test environment
    init_test_logging();

    // Create a registry with many models
    let registry = Arc::new(ModelRegistry::new());

    // Add 100 test models
    for i in 0..100 {
        let provider = if i % 3 == 0 {
            "provider1"
        } else if i % 3 == 1 {
            "provider2"
        } else {
            "provider3"
        };

        let mut model = create_test_model(&format!("model{}", i), provider);

        // Vary capabilities
        model.capabilities.max_context_length = 4096 * (i % 4 + 1);
        model.capabilities.supports_streaming = i % 2 == 0;
        model.capabilities.supports_function_calling = i % 3 == 0;

        registry.register_model(model).unwrap();
    }

    // Create a router
    let config = RouterConfig::default();
    let router = RouterImpl::new(config, registry.clone()).unwrap();

    // Update from registry
    let start = std::time::Instant::now();
    let result = router.update_from_registry().await;
    let update_duration = start.elapsed();

    assert!(result.is_ok());
    println!("Update from registry took: {:?}", update_duration);

    // Get filtered models with various filters
    let filters = vec![
        ModelFilter::new().with_provider("provider1".to_string()),
        ModelFilter::new().with_function_calling(true),
        ModelFilter::new().with_min_context_length(8192),
    ];

    for (i, filter) in filters.iter().enumerate() {
        let mut request = create_test_request();
        request.model_filter = Some(filter.clone());

        let start = std::time::Instant::now();
        let result = router.get_filtered_models(&request).await;
        let filter_duration = start.elapsed();

        assert!(result.is_ok());
        println!("Filter {} took: {:?}", i, filter_duration);
    }
}
