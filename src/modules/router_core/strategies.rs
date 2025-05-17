//! Routing Strategy Implementations
//!
//! This module contains implementations of various routing strategies for the IntelliRouter.
//! It provides a base strategy with common functionality that can be extended by specific
//! strategy implementations.

use std::collections::HashMap;
use std::time::Instant;

use async_trait::async_trait;

// Strategy implementations
pub mod priority;
pub mod round_robin;

// Re-export types for easier access
pub use priority::{PriorityConfig, PriorityStrategy};
pub use round_robin::{RoundRobinConfig, RoundRobinStrategy};
use tracing::{debug, info, warn};

use crate::modules::model_registry::{
    connectors::{ChatCompletionRequest, ChatMessage, MessageRole},
    storage::ModelRegistry,
    ModelMetadata, ModelStatus,
};

use super::{
    RouterError, RoutingMetadata, RoutingRequest, RoutingStrategy, RoutingStrategyTrait,
    StrategyConfig,
};

/// Base strategy implementation providing common functionality for all routing strategies
#[derive(Debug)]
pub struct BaseStrategy {
    /// Strategy name
    name: &'static str,
    /// Strategy type
    strategy_type: RoutingStrategy,
    /// Strategy configuration
    config: StrategyConfig,
}

impl BaseStrategy {
    /// Create a new base strategy
    pub fn new(name: &'static str, strategy_type: RoutingStrategy, config: StrategyConfig) -> Self {
        Self {
            name,
            strategy_type,
            config,
        }
    }

    /// Filter models based on request criteria
    async fn filter_models(
        &self,
        request: &RoutingRequest,
        registry: &ModelRegistry,
    ) -> Result<Vec<ModelMetadata>, RouterError> {
        debug!("Filtering models for request");

        // Start with all available models
        let mut models = registry.find_available_models();

        // If no models are available, return an error
        if models.is_empty() {
            return Err(RouterError::NoSuitableModel(
                "No available models found in registry".to_string(),
            ));
        }

        // Apply model filter if provided
        if let Some(filter) = &request.model_filter {
            models = registry.find_models(filter);
            if models.is_empty() {
                return Err(RouterError::NoSuitableModel(
                    "No models match the provided filter criteria".to_string(),
                ));
            }
        }

        // Filter out excluded models
        if !request.excluded_model_ids.is_empty() {
            models.retain(|model| !request.excluded_model_ids.contains(&model.id));
            if models.is_empty() {
                return Err(RouterError::NoSuitableModel(
                    "All available models are excluded by request".to_string(),
                ));
            }
        }

        // Filter out models with limited status if configured
        if !self.config.include_limited_models {
            models.retain(|model| model.status != ModelStatus::Limited);
            if models.is_empty() {
                return Err(RouterError::NoSuitableModel(
                    "No models available with required status".to_string(),
                ));
            }
        }

        // Apply additional suitability checks
        models.retain(|model| self.is_model_suitable(model, request));
        if models.is_empty() {
            return Err(RouterError::NoSuitableModel(
                "No suitable models found after applying all criteria".to_string(),
            ));
        }

        // Prioritize preferred model if specified
        if let Some(preferred_id) = &request.preferred_model_id {
            if let Some(preferred_model) = models.iter().find(|m| &m.id == preferred_id) {
                // Move preferred model to the front
                let preferred_model = preferred_model.clone();
                models.retain(|m| m.id != preferred_model.id);
                models.insert(0, preferred_model);
            }
        }

        debug!("Found {} suitable models", models.len());
        Ok(models)
    }

    /// Check if a model is suitable for a request
    fn is_model_suitable(&self, model: &ModelMetadata, request: &RoutingRequest) -> bool {
        // Check if model is available
        if !model.is_available() {
            return false;
        }

        // Check context length requirements
        let estimated_tokens = self.estimate_token_count(request);
        if estimated_tokens > model.capabilities.max_context_length {
            return false;
        }

        // Check if model supports streaming if requested
        if request.context.request.stream.unwrap_or(false) && !model.capabilities.supports_streaming
        {
            return false;
        }

        // Check if model supports function calling if requested
        if request.context.request.functions.is_some()
            && !model.capabilities.supports_function_calling
        {
            return false;
        }

        // Check if model supports tools if requested
        if request.context.request.tools.is_some() && !model.capabilities.supports_function_calling
        {
            return false;
        }

        // Check for high latency models if configured to exclude them
        if !self.config.include_high_latency_models {
            if let Some(avg_latency) = model.capabilities.performance.avg_latency_ms {
                // Consider high latency if > 1000ms
                if avg_latency > 1000.0 {
                    return false;
                }
            }
        }

        // All checks passed
        true
    }

    /// Estimate token count for a request
    fn estimate_token_count(&self, request: &RoutingRequest) -> usize {
        // Simple estimation based on message content lengths
        // In a real implementation, this would use a proper tokenizer
        let mut total_tokens = 0;

        for message in &request.context.request.messages {
            // Rough estimate: 1 token per 4 characters
            total_tokens += message.content.len() / 4;

            // Add tokens for function calls if present
            if let Some(func_call) = &message.function_call {
                total_tokens += func_call.name.len() / 4;
                total_tokens += func_call.arguments.len() / 4;
            }

            // Add tokens for tool calls if present
            if let Some(tool_calls) = &message.tool_calls {
                for tool_call in tool_calls {
                    total_tokens += tool_call.function.name.len() / 4;
                    total_tokens += tool_call.function.arguments.len() / 4;
                }
            }
        }

        // Add a buffer for safety
        total_tokens += 100;

        total_tokens
    }

    /// Handle fallback strategies
    async fn try_fallback(
        &self,
        request: &RoutingRequest,
        failed_model_id: &str,
        error: &RouterError,
        registry: &ModelRegistry,
    ) -> Result<ModelMetadata, RouterError> {
        // Check if we have a fallback strategy configured
        if let Some(fallback_config) = &self.config.fallback_strategy {
            info!(
                "Attempting fallback strategy for failed model: {}",
                failed_model_id
            );

            // Create a new request with the failed model excluded
            let mut fallback_request = request.clone();
            fallback_request
                .excluded_model_ids
                .push(failed_model_id.to_string());

            // Get available models with the fallback strategy's configuration
            let fallback_strategy =
                BaseStrategy::new("fallback", self.strategy_type, (**fallback_config).clone());

            // Try to select a model using the fallback strategy
            match fallback_strategy
                .select_model(&fallback_request, registry)
                .await
            {
                Ok(model) => {
                    info!("Fallback succeeded, selected model: {}", model.id);
                    return Ok(model);
                }
                Err(fallback_error) => {
                    warn!("Fallback strategy failed: {}", fallback_error);
                    return Err(RouterError::FallbackError(format!(
                        "Original error: {}. Fallback error: {}",
                        error, fallback_error
                    )));
                }
            }
        }

        // No fallback strategy or fallback failed
        Err(RouterError::NoSuitableModel(format!(
            "No fallback available for failed model: {}. Error: {}",
            failed_model_id, error
        )))
    }
}

#[async_trait]
impl super::RoutingStrategyTrait for BaseStrategy {
    fn name(&self) -> &'static str {
        self.name
    }

    fn strategy_type(&self) -> RoutingStrategy {
        self.strategy_type
    }

    async fn select_model(
        &self,
        request: &RoutingRequest,
        registry: &ModelRegistry,
    ) -> Result<ModelMetadata, RouterError> {
        debug!("Selecting model using {} strategy", self.name);

        // Filter models based on request criteria
        let models = self.filter_models(request, registry).await?;

        // Default implementation: select the first suitable model
        // Specific strategies will override this with their own selection logic
        if let Some(model) = models.first() {
            info!("Selected model: {}", model.id);
            Ok(model.clone())
        } else {
            // This should not happen since filter_models already checks for empty results
            Err(RouterError::NoSuitableModel(
                "No suitable model found after filtering".to_string(),
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
        warn!(
            "Handling failure for model {} with error: {}",
            failed_model_id, error
        );

        // Try fallback strategy
        self.try_fallback(request, failed_model_id, error, registry)
            .await
    }

    fn get_routing_metadata(
        &self,
        model: &ModelMetadata,
        start_time: Instant,
        attempts: u32,
        is_fallback: bool,
    ) -> RoutingMetadata {
        // Create metadata directly instead of calling the trait method to avoid recursion
        let mut metadata = RoutingMetadata {
            selected_model_id: model.id.clone(),
            strategy_name: self.name().to_string(),
            routing_start_time: chrono::Utc::now()
                - chrono::Duration::from_std(start_time.elapsed()).unwrap_or_default(),
            routing_end_time: chrono::Utc::now(),
            routing_time_ms: start_time.elapsed().as_millis() as u64,
            models_considered: 1,
            attempts,
            is_fallback,
            selection_criteria: None,
            additional_metadata: HashMap::new(),
        };

        // Add strategy-specific metadata
        metadata
            .additional_metadata
            .insert("strategy_type".to_string(), self.strategy_type.to_string());

        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::model_registry::{
        connectors::{ChatCompletionRequest, ChatMessage, MessageRole},
        ModelFilter,
    };
    use crate::modules::router_core::RoutingStrategyTrait;
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

    // Note: We're skipping the async tests for now since the MockModelRegistry
    // doesn't have the methods we need. In a real implementation, we would
    // create a more complete mock or use the actual ModelRegistry.

    #[test]
    fn test_base_strategy_creation() {
        let config = StrategyConfig::default();
        let strategy = BaseStrategy::new("test", RoutingStrategy::RoundRobin, config);

        assert_eq!(strategy.name(), "test");
        assert_eq!(strategy.strategy_type(), RoutingStrategy::RoundRobin);
    }

    #[test]
    fn test_is_model_suitable() {
        let config = StrategyConfig::default();
        let strategy = BaseStrategy::new("test", RoutingStrategy::RoundRobin, config);

        let mut model = ModelMetadata::new(
            "test-model".to_string(),
            "Test Model".to_string(),
            "test-provider".to_string(),
            "1.0".to_string(),
            "https://example.com".to_string(),
        );

        // Set model as available
        model.set_status(ModelStatus::Available);

        // Set capabilities
        model.capabilities.max_context_length = 4096;
        model.capabilities.supports_streaming = true;
        model.capabilities.supports_function_calling = true;

        let mut request = create_test_request();

        // Test with streaming request
        request.context.request.stream = Some(true);
        assert!(strategy.is_model_suitable(&model, &request));

        // Test with non-streaming model
        model.capabilities.supports_streaming = false;
        assert!(!strategy.is_model_suitable(&model, &request));

        // Reset streaming capability and test with functions
        model.capabilities.supports_streaming = true;
        request.context.request.stream = None;
        request.context.request.functions = Some(vec![]);
        assert!(strategy.is_model_suitable(&model, &request));

        // Test with model that doesn't support functions
        model.capabilities.supports_function_calling = false;
        assert!(!strategy.is_model_suitable(&model, &request));
    }
}
