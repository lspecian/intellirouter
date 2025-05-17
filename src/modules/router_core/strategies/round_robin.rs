//! Round-Robin Routing Strategy
//!
//! This module implements a round-robin routing strategy that distributes requests
//! evenly across available models. It can optionally weight the distribution based
//! on model capacity or custom weights.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::modules::model_registry::{storage::ModelRegistry, ModelMetadata};
use crate::modules::router_core::{
    BaseStrategy, RouterError, RoutingMetadata, RoutingRequest, RoutingStrategy,
    RoutingStrategyTrait, StrategyConfig,
};

/// Round-robin routing strategy
#[derive(Debug)]
pub struct RoundRobinStrategy {
    /// Base strategy
    base: BaseStrategy,
    /// Round-robin configuration
    config: RoundRobinConfig,
    /// Current index for round-robin selection
    current_index: AtomicUsize,
}

/// Round-robin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundRobinConfig {
    /// Base strategy configuration
    #[serde(flatten)]
    pub base: StrategyConfig,
    /// Whether to weight the distribution by model capacity
    pub weighted: bool,
    /// Model weights (model ID -> weight)
    pub model_weights: HashMap<String, u32>,
    /// Provider weights (provider name -> weight)
    pub provider_weights: HashMap<String, u32>,
    /// Default weight for models not explicitly weighted
    pub default_weight: u32,
}

impl Default for RoundRobinConfig {
    fn default() -> Self {
        Self {
            base: StrategyConfig::default(),
            weighted: false,
            model_weights: HashMap::new(),
            provider_weights: HashMap::new(),
            default_weight: 1,
        }
    }
}

impl RoundRobinStrategy {
    /// Create a new round-robin strategy
    pub fn new(config: RoundRobinConfig) -> Self {
        Self {
            base: BaseStrategy::new(
                "round_robin",
                RoutingStrategy::RoundRobin,
                config.base.clone(),
            ),
            config,
            current_index: AtomicUsize::new(0),
        }
    }

    /// Get the weight for a model
    fn get_model_weight(&self, model: &ModelMetadata) -> u32 {
        // Check for explicit model ID weight
        if let Some(weight) = self.config.model_weights.get(&model.id) {
            return *weight;
        }

        // Check for provider weight
        if let Some(weight) = self.config.provider_weights.get(&model.provider) {
            return *weight;
        }

        // Use default weight
        self.config.default_weight
    }

    /// Get weighted models for distribution
    fn get_weighted_models(&self, models: &[ModelMetadata]) -> Vec<ModelMetadata> {
        if !self.config.weighted {
            return models.to_vec();
        }

        let mut weighted_models = Vec::new();
        for model in models {
            let weight = self.get_model_weight(model);
            for _ in 0..weight {
                weighted_models.push(model.clone());
            }
        }

        weighted_models
    }
}

#[async_trait]
impl RoutingStrategyTrait for RoundRobinStrategy {
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
        debug!("Selecting model using round-robin strategy");

        // Get filtered models from base strategy
        let models = self.base.filter_models(request, registry).await?;

        // If no models are available, return an error
        if models.is_empty() {
            return Err(RouterError::NoSuitableModel(
                "No suitable models found after filtering".to_string(),
            ));
        }

        // Apply weighting if configured
        let selection_pool = if self.config.weighted {
            self.get_weighted_models(&models)
        } else {
            models.clone()
        };

        // If the selection pool is empty (shouldn't happen), return an error
        if selection_pool.is_empty() {
            return Err(RouterError::NoSuitableModel(
                "No suitable models found after applying weights".to_string(),
            ));
        }

        // Get the next index in a thread-safe way
        let index = self.current_index.fetch_add(1, Ordering::SeqCst) % selection_pool.len();

        // Select the model at the current index
        let model = selection_pool[index].clone();

        info!(
            "Selected model: {} (index {} of {})",
            model.id,
            index,
            selection_pool.len()
        );

        Ok(model)
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

        // Add round-robin-specific metadata
        metadata.selection_criteria = Some("round_robin".to_string());
        metadata.additional_metadata.insert(
            "current_index".to_string(),
            self.current_index.load(Ordering::SeqCst).to_string(),
        );

        if self.config.weighted {
            metadata
                .additional_metadata
                .insert("weighted".to_string(), "true".to_string());
            metadata.additional_metadata.insert(
                "model_weight".to_string(),
                self.get_model_weight(model).to_string(),
            );
        }

        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::model_registry::{
        connectors::{ChatCompletionRequest, ChatMessage, MessageRole},
        ModelStatus, ModelType,
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
    fn test_round_robin_strategy_creation() {
        let config = RoundRobinConfig::default();
        let strategy = RoundRobinStrategy::new(config);

        assert_eq!(strategy.name(), "round_robin");
        assert_eq!(strategy.strategy_type(), RoutingStrategy::RoundRobin);
    }

    #[test]
    fn test_get_model_weight() {
        let mut config = RoundRobinConfig::default();
        config.default_weight = 1;
        config.model_weights.insert("model1".to_string(), 10);
        config.provider_weights.insert("provider1".to_string(), 5);

        let strategy = RoundRobinStrategy::new(config);

        // Test model ID weight
        let model1 = create_test_model("model1", "provider2", ModelType::TextGeneration);
        assert_eq!(strategy.get_model_weight(&model1), 10);

        // Test provider weight
        let model2 = create_test_model("model2", "provider1", ModelType::TextGeneration);
        assert_eq!(strategy.get_model_weight(&model2), 5);

        // Test default weight
        let model3 = create_test_model("model3", "provider2", ModelType::TextGeneration);
        assert_eq!(strategy.get_model_weight(&model3), 1);
    }

    #[test]
    fn test_get_weighted_models() {
        let mut config = RoundRobinConfig::default();
        config.weighted = true;
        config.model_weights.insert("model1".to_string(), 3);
        config.model_weights.insert("model2".to_string(), 1);
        config.default_weight = 1;

        let strategy = RoundRobinStrategy::new(config);

        let model1 = create_test_model("model1", "provider1", ModelType::TextGeneration);
        let model2 = create_test_model("model2", "provider1", ModelType::TextGeneration);
        let model3 = create_test_model("model3", "provider1", ModelType::TextGeneration);

        let models = vec![model1, model2, model3];
        let weighted_models = strategy.get_weighted_models(&models);

        // model1 should appear 3 times, model2 once, and model3 once (default weight)
        assert_eq!(weighted_models.len(), 5);

        // Count occurrences of each model
        let model1_count = weighted_models.iter().filter(|m| m.id == "model1").count();
        let model2_count = weighted_models.iter().filter(|m| m.id == "model2").count();
        let model3_count = weighted_models.iter().filter(|m| m.id == "model3").count();

        assert_eq!(model1_count, 3);
        assert_eq!(model2_count, 1);
        assert_eq!(model3_count, 1);
    }

    // Note: We're skipping the async tests for now since the MockModelRegistry
    // doesn't have the methods we need. In a real implementation, we would
    // create a more complete mock or use the actual ModelRegistry.
}
