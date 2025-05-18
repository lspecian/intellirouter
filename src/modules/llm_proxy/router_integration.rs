//! Router Integration for LLM Proxy
//!
//! This module integrates the router_core with the LLM proxy,
//! allowing requests to be routed to the appropriate model backend.

use futures::StreamExt;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::modules::model_registry::connectors::{ChatCompletionRequest, ChatCompletionResponse};
use crate::modules::model_registry::storage::ModelRegistry;
use crate::modules::router_core::{
    ErrorCategory, Router, RouterError, RouterImpl, RoutingContext, RoutingRequest, RoutingResponse,
};

/// Service for routing chat completion requests
pub struct RouterService {
    /// Router implementation
    router: Arc<RouterImpl>,
}

impl RouterService {
    /// Create a new router service
    pub fn new(router: Arc<RouterImpl>) -> Self {
        Self { router }
    }

    /// Route a chat completion request to the appropriate model
    pub async fn route_request(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, RouterError> {
        debug!("Routing request for model: {}", request.model);

        // Create routing context
        let context = RoutingContext::new(request.clone());

        // Create routing request
        let routing_request =
            RoutingRequest::new(request.clone()).with_preferred_model(request.model.clone());

        // Route the request
        let routing_response = self.router.route(routing_request).await?;

        // Return the response
        Ok(routing_response.response)
    }

    /// Route a streaming chat completion request
    pub async fn route_streaming_request(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<
        impl futures::Stream<Item = Result<String, RouterError>> + Send + 'static,
        RouterError,
    > {
        debug!("Routing streaming request for model: {}", request.model);

        // Create routing context
        let context = RoutingContext::new(request.clone());

        // Create routing request
        let routing_request =
            RoutingRequest::new(request.clone()).with_preferred_model(request.model.clone());

        // Route the request
        let routing_response = self.router.route(routing_request).await?;

        // Get the selected model
        let model_id = routing_response.metadata.selected_model_id.clone();

        // Get the connector for the selected model
        let registry = self.router.get_registry();
        let connector = registry.get_connector(&model_id).ok_or_else(|| {
            RouterError::NoSuitableModel(format!("No connector found for model: {}", model_id))
        })?;

        // Generate streaming response
        let stream = connector
            .generate_streaming(request.clone())
            .await
            .map_err(|e| RouterError::ConnectorError(e.to_string()))?;

        // Map connector errors to router errors and convert chunks to strings
        let mapped_stream = stream.map(|result| match result {
            Ok(chunk) => {
                // Convert the chunk to a JSON string
                match serde_json::to_string(&chunk) {
                    Ok(json) => Ok(json),
                    Err(e) => Err(RouterError::SerializationError(e.to_string())),
                }
            }
            Err(e) => Err(RouterError::ConnectorError(e.to_string())),
        });

        Ok(mapped_stream)
    }
}

/// Create a router service with a mock backend for testing
pub fn create_mock_router_service() -> RouterService {
    use crate::modules::llm_proxy::MockModelBackend;
    use crate::modules::model_registry::{ModelMetadata, ModelStatus, ModelType};
    use crate::modules::router_core::{RouterConfig, RoutingStrategy};
    use std::collections::HashMap;

    // Create a model registry
    let registry = Arc::new(ModelRegistry::new());

    // Create mock models
    let models = vec![
        ModelMetadata::new(
            "gpt-3.5-turbo".to_string(),
            "GPT-3.5 Turbo".to_string(),
            "openai".to_string(),
            "1.0".to_string(),
            "https://api.openai.com/v1/chat/completions".to_string(),
        ),
        ModelMetadata::new(
            "gpt-4".to_string(),
            "GPT-4".to_string(),
            "openai".to_string(),
            "1.0".to_string(),
            "https://api.openai.com/v1/chat/completions".to_string(),
        ),
        ModelMetadata::new(
            "claude-3-sonnet".to_string(),
            "Claude 3 Sonnet".to_string(),
            "anthropic".to_string(),
            "1.0".to_string(),
            "https://api.anthropic.com/v1/messages".to_string(),
        ),
    ];

    // Register models
    for mut model in models {
        model.status = ModelStatus::Available;
        model.model_type = ModelType::TextGeneration; // Use TextGeneration instead of ChatCompletion
        registry.register_model(model.clone()).unwrap();

        // Create and register mock backend
        let backend =
            MockModelBackend::new(model.id.clone(), model.name.clone(), model.provider.clone());
        registry.register_connector(&model.id, Arc::new(backend));
        // No need to unwrap as the method doesn't return a Result
    }

    // Create router config
    let config = RouterConfig {
        strategy: RoutingStrategy::RoundRobin,
        strategy_config: None,
        fallback_strategies: vec![RoutingStrategy::ContentBased],
        retry_policy: crate::modules::router_core::RetryPolicy::default(),
        circuit_breaker: crate::modules::router_core::CircuitBreakerConfig::default(),
        degraded_service_mode: crate::modules::router_core::DegradedServiceMode::default(),
        retryable_errors: std::collections::HashSet::new(),
        cache_routing_decisions: true,
        max_cache_size: 100,
        collect_metrics: true,
        ..Default::default()
    };

    // Create router
    let router = RouterImpl::new(config, registry).unwrap();

    // Create router service
    RouterService::new(Arc::new(router))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::model_registry::connectors::{ChatMessage, MessageRole};

    #[tokio::test]
    async fn test_router_service_route_request() {
        // Create router service
        let service = create_mock_router_service();

        // Create test request
        let request = ChatCompletionRequest {
            model: "gpt-3.5-turbo".to_string(),
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

        // Route the request
        let response = service.route_request(&request).await.unwrap();

        // Verify the response
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.role, MessageRole::Assistant);
        assert!(response.choices[0]
            .message
            .content
            .contains("Hello, world!"));
    }
}
