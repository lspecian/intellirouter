//! Tests for the round-robin routing strategy

use std::sync::Arc;
use std::time::Duration;

use crate::modules::model_registry::{
    connectors::{ChatCompletionRequest, ChatMessage, MessageRole},
    ModelMetadata, ModelStatus, ModelType,
};
use crate::modules::router_core::{
    strategies::{RoundRobinConfig, RoundRobinStrategy},
    RoutingRequest, RoutingStrategyTrait,
};
use crate::test_utils::mocks::MockModelRegistry;

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

    let mut request = RoutingRequest::new(chat_request);
    request.timeout = Duration::from_secs(10);
    request
}

fn create_test_model(id: &str, provider: &str, model_type: ModelType) -> ModelMetadata {
    let mut model = ModelMetadata::new(
        id.to_string(),
        format!("Test Model {}", id),
        provider.to_string(),
        "1.0".to_string(),
        "https://example.com".to_string(),
    );

    // Set model as available
    model.set_status(ModelStatus::Available);
    model.set_model_type(model_type);

    // Set capabilities
    model.capabilities.max_context_length = 4096;
    model.capabilities.supports_streaming = true;
    model.capabilities.supports_function_calling = true;

    model
}

#[tokio::test]
async fn test_round_robin_selection() {
    // Create a mock registry with test models
    let mut registry = MockModelRegistry::new();

    // Add test models
    let model1 = create_test_model("model1", "provider1", ModelType::TextGeneration);
    let model2 = create_test_model("model2", "provider1", ModelType::TextGeneration);
    let model3 = create_test_model("model3", "provider2", ModelType::TextGeneration);

    registry.add_model(model1.clone());
    registry.add_model(model2.clone());
    registry.add_model(model3.clone());

    // Create a round-robin strategy
    let config = RoundRobinConfig::default();
    let strategy = RoundRobinStrategy::new(config);

    // Create a test request
    let request = create_test_request();

    // Test round-robin selection
    // First call should select model1
    let selected1 = strategy.select_model(&request, &registry).await.unwrap();
    assert_eq!(selected1.id, "model1");

    // Second call should select model2
    let selected2 = strategy.select_model(&request, &registry).await.unwrap();
    assert_eq!(selected2.id, "model2");

    // Third call should select model3
    let selected3 = strategy.select_model(&request, &registry).await.unwrap();
    assert_eq!(selected3.id, "model3");

    // Fourth call should wrap around to model1
    let selected4 = strategy.select_model(&request, &registry).await.unwrap();
    assert_eq!(selected4.id, "model1");
}

#[tokio::test]
async fn test_weighted_round_robin() {
    // Create a mock registry with test models
    let mut registry = MockModelRegistry::new();

    // Add test models
    let model1 = create_test_model("model1", "provider1", ModelType::TextGeneration);
    let model2 = create_test_model("model2", "provider1", ModelType::TextGeneration);

    registry.add_model(model1.clone());
    registry.add_model(model2.clone());

    // Create a weighted round-robin strategy
    let mut config = RoundRobinConfig::default();
    config.weighted = true;
    config.model_weights.insert("model1".to_string(), 2);
    config.model_weights.insert("model2".to_string(), 1);

    let strategy = RoundRobinStrategy::new(config);

    // Create a test request
    let request = create_test_request();

    // Test weighted round-robin selection
    // First call should select model1
    let selected1 = strategy.select_model(&request, &registry).await.unwrap();
    assert_eq!(selected1.id, "model1");

    // Second call should select model1 again (due to weight of 2)
    let selected2 = strategy.select_model(&request, &registry).await.unwrap();
    assert_eq!(selected2.id, "model1");

    // Third call should select model2
    let selected3 = strategy.select_model(&request, &registry).await.unwrap();
    assert_eq!(selected3.id, "model2");

    // Fourth call should wrap around to model1
    let selected4 = strategy.select_model(&request, &registry).await.unwrap();
    assert_eq!(selected4.id, "model1");
}

#[tokio::test]
async fn test_provider_weights() {
    // Create a mock registry with test models
    let mut registry = MockModelRegistry::new();

    // Add test models from different providers
    let model1 = create_test_model("model1", "provider1", ModelType::TextGeneration);
    let model2 = create_test_model("model2", "provider2", ModelType::TextGeneration);

    registry.add_model(model1.clone());
    registry.add_model(model2.clone());

    // Create a weighted round-robin strategy with provider weights
    let mut config = RoundRobinConfig::default();
    config.weighted = true;
    config.provider_weights.insert("provider1".to_string(), 3);
    config.provider_weights.insert("provider2".to_string(), 1);

    let strategy = RoundRobinStrategy::new(config);

    // Create a test request
    let request = create_test_request();

    // Test provider-weighted round-robin selection
    // First call should select model1
    let selected1 = strategy.select_model(&request, &registry).await.unwrap();
    assert_eq!(selected1.id, "model1");

    // Second call should select model1 again (due to provider weight of 3)
    let selected2 = strategy.select_model(&request, &registry).await.unwrap();
    assert_eq!(selected2.id, "model1");

    // Third call should select model1 again
    let selected3 = strategy.select_model(&request, &registry).await.unwrap();
    assert_eq!(selected3.id, "model1");

    // Fourth call should select model2
    let selected4 = strategy.select_model(&request, &registry).await.unwrap();
    assert_eq!(selected4.id, "model2");

    // Fifth call should wrap around to model1
    let selected5 = strategy.select_model(&request, &registry).await.unwrap();
    assert_eq!(selected5.id, "model1");
}

#[tokio::test]
async fn test_routing_metadata() {
    // Create a mock registry with a test model
    let mut registry = MockModelRegistry::new();
    let model = create_test_model("model1", "provider1", ModelType::TextGeneration);
    registry.add_model(model.clone());

    // Create a round-robin strategy
    let config = RoundRobinConfig::default();
    let strategy = RoundRobinStrategy::new(config);

    // Create a test request
    let request = create_test_request();

    // Select a model
    let selected = strategy.select_model(&request, &registry).await.unwrap();

    // Get routing metadata
    let start_time = std::time::Instant::now();
    let metadata = strategy.get_routing_metadata(&selected, start_time, 1, false);

    // Verify metadata
    assert_eq!(metadata.selected_model_id, "model1");
    assert_eq!(metadata.strategy_name, "round_robin");
    assert_eq!(metadata.attempts, 1);
    assert_eq!(metadata.is_fallback, false);
    assert_eq!(metadata.selection_criteria, Some("round_robin".to_string()));
    assert!(metadata.additional_metadata.contains_key("current_index"));
}
