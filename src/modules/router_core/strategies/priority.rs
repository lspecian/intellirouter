//! Priority-based Routing Strategy
//!
//! This module implements a priority-based routing strategy that selects models
//! based on configurable priorities for models, providers, and model types.

use std::collections::HashMap;
use std::time::Instant;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::modules::model_registry::{storage::ModelRegistry, ModelMetadata, ModelType};
use crate::modules::router_core::{
    BaseStrategy, RouterError, RoutingMetadata, RoutingRequest, RoutingStrategy,
    RoutingStrategyTrait, StrategyConfig,
};

/// Priority configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityConfig {
    /// Base strategy configuration
    #[serde(flatten)]
    pub base: StrategyConfig,
    /// Model priorities (model ID -> priority)
    pub model_priorities: HashMap<String, u32>,
    /// Provider priorities (provider name -> priority)
    pub provider_priorities: HashMap<String, u32>,
    /// Type priorities (model type -> priority)
    pub type_priorities: HashMap<String, u32>,
    /// Default priority for models not explicitly prioritized
    pub default_priority: u32,
}

impl Default for PriorityConfig {
    fn default() -> Self {
        Self {
            base: StrategyConfig::default(),
            model_priorities: HashMap::new(),
            provider_priorities: HashMap::new(),
            type_priorities: HashMap::new(),
            default_priority: 0,
        }
    }
}

/// Priority-based routing strategy
pub struct PriorityStrategy {
    /// Base strategy
    base: BaseStrategy,
    /// Priority configuration
    config: PriorityConfig,
}

impl PriorityStrategy {
    /// Create a new priority-based strategy
    pub fn new(config: PriorityConfig) -> Self {
        Self {
            base: BaseStrategy::new(
                "priority",
                RoutingStrategy::ContentBased,
                config.base.clone(),
            ),
            config,
        }
    }

    /// Get the priority for a model
    fn get_model_priority(&self, model: &ModelMetadata) -> u32 {
        // Check for explicit model ID priority
        if let Some(priority) = self.config.model_priorities.get(&model.id) {
            return *priority;
        }

        // Check for provider priority
        if let Some(priority) = self.config.provider_priorities.get(&model.provider) {
            return *priority;
        }

        // Check for type priority
        let type_str = match model.model_type {
            ModelType::TextGeneration => "text",
            ModelType::Embedding => "embedding",
            ModelType::ImageGeneration => "image",
            ModelType::AudioProcessing => "audio",
            ModelType::MultiModal => "multimodal",
            ModelType::Other(ref s) => s,
        };
        if let Some(priority) = self.config.type_priorities.get(type_str) {
            return *priority;
        }

        // Use default priority
        self.config.default_priority
    }
}

#[async_trait]
impl RoutingStrategyTrait for PriorityStrategy {
    fn name(&self) -> &'static str {
        self.base.name()
    }

    fn strategy_type(&self) -> RoutingStrategy {
        self.base.strategy_type()
    }

    async fn select_model(
        &self,
        request: &RoutingRequest,
        registry: &ModelRegistry,
    ) -> Result<ModelMetadata, RouterError> {
        debug!("Selecting model using priority strategy");

        // Get filtered models from base strategy
        let models = self.base.filter_models(request, registry).await?;

        // Sort models by priority
        let mut prioritized_models = models.clone();
        prioritized_models.sort_by(|a, b| {
            let a_priority = self.get_model_priority(a);
            let b_priority = self.get_model_priority(b);
            // Higher priority comes first
            b_priority.cmp(&a_priority)
        });

        // Select the highest priority model
        if let Some(model) = prioritized_models.first() {
            info!(
                "Selected model: {} with priority {}",
                model.id,
                self.get_model_priority(model)
            );
            Ok(model.clone())
        } else {
            Err(RouterError::NoSuitableModel(
                "No suitable model found after priority filtering".to_string(),
            ))
        }
    }

    async fn handle_failure(
        &self,
        request: &RoutingRequest,
        failed_model_id: &str,
        error: &RouterError,
        registry: &ModelRegistry,
    ) -> Result<ModelMetadata, RouterError> {
        // Delegate to base strategy
        self.base
            .handle_failure(request, failed_model_id, error, registry)
            .await
    }

    fn get_routing_metadata(
        &self,
        model: &ModelMetadata,
        start_time: Instant,
        attempts: u32,
        is_fallback: bool,
    ) -> RoutingMetadata {
        // Get base metadata
        let mut metadata = self
            .base
            .get_routing_metadata(model, start_time, attempts, is_fallback);

        // Add priority-specific metadata
        metadata.selection_criteria = Some("priority".to_string());
        metadata.additional_metadata.insert(
            "model_priority".to_string(),
            self.get_model_priority(model).to_string(),
        );

        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::model_registry::{
        connectors::{ChatCompletionRequest, ChatMessage, MessageRole},
        ModelStatus,
    };
    use crate::test_utils::mocks::MockModelRegistry;
    use std::time::Duration;

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

    #[test]
    fn test_priority_strategy_creation() {
        let config = PriorityConfig::default();
        let strategy = PriorityStrategy::new(config);

        assert_eq!(strategy.name(), "priority");
        assert_eq!(strategy.strategy_type(), RoutingStrategy::ContentBased);
    }

    #[test]
    fn test_get_model_priority() {
        let mut config = PriorityConfig::default();
        config.default_priority = 1;
        config.model_priorities.insert("model1".to_string(), 10);
        config
            .provider_priorities
            .insert("provider1".to_string(), 5);
        config.type_priorities.insert("text".to_string(), 3);

        let strategy = PriorityStrategy::new(config);

        // Test model ID priority
        let model1 = create_test_model("model1", "provider2", ModelType::TextGeneration);
        assert_eq!(strategy.get_model_priority(&model1), 10);

        // Test provider priority
        let model2 = create_test_model("model2", "provider1", ModelType::TextGeneration);
        assert_eq!(strategy.get_model_priority(&model2), 5);

        // Test type priority
        let model3 = create_test_model("model3", "provider2", ModelType::TextGeneration);
        assert_eq!(strategy.get_model_priority(&model3), 3);

        // Test default priority
        let model4 = create_test_model("model4", "provider2", ModelType::Embedding);
        assert_eq!(strategy.get_model_priority(&model4), 1);
    }

    // Note: We're skipping the async tests for now since the MockModelRegistry
    // doesn't have the methods we need. In a real implementation, we would
    // create a more complete mock or use the actual ModelRegistry.
}
