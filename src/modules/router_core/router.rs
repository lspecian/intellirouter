//! Router Implementation
//!
//! This module contains the concrete implementation of the Router trait
//! that leverages the strategy interfaces to make routing decisions.

use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::modules::router_core::config::StrategyConfig;
use crate::modules::router_core::RegistryIntegration;

use lru::LruCache;
use tracing::{debug, error, info, warn};

use crate::modules::model_registry::{
    connectors::{ChatCompletionRequest, ModelConnector},
    storage::ModelRegistry,
    ModelMetadata,
};

use super::{
    retry::{
        CircuitBreakerConfig, DegradedServiceHandler, DegradedServiceMode, RetryManager,
        RetryPolicy,
    },
    strategies::{PriorityConfig, PriorityStrategy, RoundRobinConfig, RoundRobinStrategy},
    BaseStrategy, Router, RouterConfig, RouterError, RoutingMetadata, RoutingRequest,
    RoutingResponse, RoutingStrategy, RoutingStrategyTrait,
};

/// Router implementation
#[derive(Debug)]
pub struct RouterImpl {
    /// Router configuration
    config: RouterConfig,
    /// Active routing strategy
    strategy: Box<dyn RoutingStrategyTrait>,
    /// Fallback strategies
    fallback_strategies: Vec<Box<dyn RoutingStrategyTrait>>,
    /// Model registry
    registry: Arc<ModelRegistry>,
    /// Registry integration
    registry_integration: RegistryIntegration,
    /// Routing metrics
    metrics: Mutex<HashMap<String, serde_json::Value>>,
    /// Routing decision cache
    cache: Mutex<LruCache<String, ModelMetadata>>,
    /// Retry manager
    retry_manager: RetryManager,
    /// Degraded service handler
    degraded_service_handler: DegradedServiceHandler,
}

impl RouterImpl {
    /// Create a new router
    pub fn new(config: RouterConfig, registry: Arc<ModelRegistry>) -> Result<Self, RouterError> {
        let retry_manager = RetryManager::new(
            config.retry_policy.clone(),
            config.circuit_breaker.clone(),
            config.retryable_errors.clone(),
        );

        let degraded_service_handler =
            DegradedServiceHandler::new(config.degraded_service_mode.clone(), registry.clone());

        let registry_integration = RegistryIntegration::new(registry.clone());

        let mut router = Self {
            config: config.clone(),
            strategy: Box::new(BaseStrategy::new(
                "default",
                RoutingStrategy::ContentBased,
                StrategyConfig::default(),
            )),
            fallback_strategies: Vec::new(),
            registry,
            registry_integration,
            metrics: Mutex::new(HashMap::new()),
            cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(config.max_cache_size).unwrap(),
            )),
            retry_manager,
            degraded_service_handler,
        };

        // Initialize with config
        router.init(config)?;

        Ok(router)
    }

    /// Update the router with the latest model information from the registry
    pub async fn update_from_registry(&self) -> Result<(), RouterError> {
        let mut metrics = self.metrics.lock().unwrap();
        self.registry_integration.update_metrics(&mut metrics).await
    }

    /// Subscribe to model registry updates
    pub async fn subscribe_to_registry_updates(&self) -> Result<(), RouterError> {
        self.registry_integration
            .subscribe_to_registry_updates()
            .await
    }

    /// Get a filtered list of models based on request criteria
    pub async fn get_filtered_models(
        &self,
        request: &RoutingRequest,
    ) -> Result<Vec<ModelMetadata>, RouterError> {
        self.registry_integration.get_filtered_models(request).await
    }

    /// Create a strategy based on the strategy type
    fn create_strategy(
        &self,
        strategy_type: &RoutingStrategy,
        config: &Option<StrategyConfig>,
    ) -> Result<Box<dyn RoutingStrategyTrait>, RouterError> {
        let base_config = config.clone().unwrap_or_default();

        match strategy_type {
            RoutingStrategy::RoundRobin => {
                let config = RoundRobinConfig {
                    base: base_config,
                    weighted: false,
                    model_weights: HashMap::new(),
                    provider_weights: HashMap::new(),
                    default_weight: 1,
                };
                Ok(Box::new(RoundRobinStrategy::new(config)))
            }
            RoutingStrategy::ContentBased => {
                let config = PriorityConfig {
                    base: base_config,
                    model_priorities: HashMap::new(),
                    provider_priorities: HashMap::new(),
                    type_priorities: HashMap::new(),
                    default_priority: 0,
                };
                Ok(Box::new(PriorityStrategy::new(config)))
            }
            // For now, we'll use the base strategy for other strategy types
            // In a real implementation, we would implement all strategy types
            RoutingStrategy::LoadBalanced
            | RoutingStrategy::CostOptimized
            | RoutingStrategy::LatencyOptimized => Ok(Box::new(BaseStrategy::new(
                "fallback",
                *strategy_type,
                base_config,
            ))),
            RoutingStrategy::Custom => Err(RouterError::StrategyConfigError(
                "Custom strategy requires specific implementation".to_string(),
            )),
        }
    }

    /// Try a strategy with retries
    async fn try_strategy_with_retries(
        &self,
        strategy: &dyn RoutingStrategyTrait,
        request: &RoutingRequest,
        start_time: Instant,
        is_fallback: bool,
    ) -> Result<RoutingResponse, RouterError> {
        let strategy_name = strategy.name().to_string();

        // Use the retry manager to execute the strategy
        let response = self
            .retry_manager
            .execute(
                || async {
                    // Select model using strategy
                    let model = strategy.select_model(request, &*self.registry).await?;

                    // Create metadata
                    let metadata =
                        strategy.get_routing_metadata(&model, start_time, 1, is_fallback);

                    // Create response
                    let response = self.create_response(request, model, metadata).await?;

                    Ok::<RoutingResponse, RouterError>(response)
                },
                &format!("strategy {}", strategy_name),
            )
            .await?;

        // Cache the result if enabled
        if self.config.cache_routing_decisions {
            let cache_key = self.generate_cache_key(request);
            self.add_to_cache(cache_key, response.metadata.selected_model_id.clone());
        }

        // Update metrics
        self.update_metrics(&response);

        Ok(response)
    }

    /// Create a response from a model and metadata
    async fn create_response(
        &self,
        request: &RoutingRequest,
        model: ModelMetadata,
        metadata: RoutingMetadata,
    ) -> Result<RoutingResponse, RouterError> {
        // Get the model connector
        let connector = self.registry.get_connector(&model.id).ok_or_else(|| {
            RouterError::NoSuitableModel(format!("No connector found for model: {}", model.id))
        })?;

        // Send the request to the model
        let response = connector
            .generate(request.context.request.clone())
            .await
            .map_err(|e| RouterError::ConnectorError(e.to_string()))?;

        // Create routing response
        Ok(RoutingResponse { response, metadata })
    }

    /// Generate a cache key for a request
    fn generate_cache_key(&self, request: &RoutingRequest) -> String {
        // Simple cache key based on request content
        // In a real implementation, this would be more sophisticated
        let mut hasher = DefaultHasher::new();
        for message in &request.context.request.messages {
            message.content.hash(&mut hasher);
        }
        format!("{:x}", hasher.finish())
    }

    /// Get a model from the cache
    fn get_from_cache(&self, key: &str) -> Option<ModelMetadata> {
        let mut cache = self.cache.lock().unwrap();
        cache.get(key).cloned()
    }

    /// Add a model to the cache
    fn add_to_cache(&self, key: String, model_id: String) {
        let mut cache = self.cache.lock().unwrap();
        if let Ok(model) = self.registry.get_model(&model_id) {
            cache.put(key, model);
        }
    }

    /// Update routing metrics
    fn update_metrics(&self, response: &RoutingResponse) {
        if !self.config.collect_metrics {
            return;
        }

        let mut metrics = self.metrics.lock().unwrap();

        // Update request count
        let request_count = metrics
            .entry("request_count".to_string())
            .or_insert_with(|| serde_json::Value::Number(serde_json::Number::from(0)));
        if let serde_json::Value::Number(n) = request_count {
            if let Some(i) = n.as_u64() {
                *request_count = serde_json::Value::Number(serde_json::Number::from(i + 1));
            }
        }

        // Update model usage
        let model_usage = metrics
            .entry("model_usage".to_string())
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
        if let serde_json::Value::Object(map) = model_usage {
            let model_id = &response.metadata.selected_model_id;
            let count = map
                .entry(model_id.clone())
                .or_insert_with(|| serde_json::Value::Number(serde_json::Number::from(0)));
            if let serde_json::Value::Number(n) = count {
                if let Some(i) = n.as_u64() {
                    *count = serde_json::Value::Number(serde_json::Number::from(i + 1));
                }
            }
        }

        // Update average routing time
        let avg_routing_time = metrics
            .entry("avg_routing_time_ms".to_string())
            .or_insert_with(|| serde_json::Value::Number(serde_json::Number::from(0)));
        if let serde_json::Value::Number(n) = avg_routing_time {
            if let Some(i) = n.as_u64() {
                let new_avg = (i + response.metadata.routing_time_ms) / 2;
                *avg_routing_time = serde_json::Value::Number(serde_json::Number::from(new_avg));
            }
        }

        // Update fallback usage
        if response.metadata.is_fallback {
            let fallback_count = metrics
                .entry("fallback_count".to_string())
                .or_insert_with(|| serde_json::Value::Number(serde_json::Number::from(0)));
            if let serde_json::Value::Number(n) = fallback_count {
                if let Some(i) = n.as_u64() {
                    *fallback_count = serde_json::Value::Number(serde_json::Number::from(i + 1));
                }
            }
        }

        // Update strategy usage
        let strategy_usage = metrics
            .entry("strategy_usage".to_string())
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
        if let serde_json::Value::Object(map) = strategy_usage {
            let strategy_name = &response.metadata.strategy_name;
            let count = map
                .entry(strategy_name.clone())
                .or_insert_with(|| serde_json::Value::Number(serde_json::Number::from(0)));
            if let serde_json::Value::Number(n) = count {
                if let Some(i) = n.as_u64() {
                    *count = serde_json::Value::Number(serde_json::Number::from(i + 1));
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl Router for RouterImpl {
    fn init(&mut self, config: RouterConfig) -> Result<(), RouterError> {
        // Update configuration
        self.config = config.clone();

        // Create strategy based on configuration
        self.strategy = self.create_strategy(&config.strategy, &config.strategy_config)?;

        // Create fallback strategies
        self.fallback_strategies.clear();
        for strategy_type in &config.fallback_strategies {
            let strategy = self.create_strategy(strategy_type, &config.strategy_config)?;
            self.fallback_strategies.push(strategy);
        }

        // Update retry manager
        self.retry_manager = RetryManager::new(
            config.retry_policy.clone(),
            config.circuit_breaker.clone(),
            config.retryable_errors.clone(),
        );

        // Update degraded service handler
        self.degraded_service_handler = DegradedServiceHandler::new(
            config.degraded_service_mode.clone(),
            self.registry.clone(),
        );

        // Clear cache if needed
        if !config.cache_routing_decisions {
            self.clear_cache();
        }

        Ok(())
    }

    async fn route(&self, request: RoutingRequest) -> Result<RoutingResponse, RouterError> {
        // Start timing
        let start_time = Instant::now();

        // Check cache if enabled
        if self.config.cache_routing_decisions {
            let cache_key = self.generate_cache_key(&request);
            if let Some(model) = self.get_from_cache(&cache_key) {
                debug!("Cache hit for request: {}", cache_key);

                // Create metadata
                let metadata = self
                    .strategy
                    .get_routing_metadata(&model, start_time, 0, false);

                // Create response
                let response = self.create_response(&request, model, metadata).await?;

                return Ok(response);
            }
        }

        // Get filtered models based on request criteria
        let filtered_models = self.get_filtered_models(&request).await?;

        // If no models are available, return an error
        if filtered_models.is_empty() {
            return Err(RouterError::NoSuitableModel(
                "No suitable models found after filtering".to_string(),
            ));
        }

        // Try primary strategy with retries
        debug!("Trying primary strategy: {}", self.strategy.name());
        let result = self
            .try_strategy_with_retries(&*self.strategy, &request, start_time, false)
            .await;

        // If primary strategy fails, try fallbacks
        if let Err(error) = result {
            warn!("Primary strategy failed: {}", error);

            // Try fallback strategies
            for (i, fallback) in self.fallback_strategies.iter().enumerate() {
                debug!("Trying fallback strategy {}: {}", i + 1, fallback.name());
                let fallback_result = self
                    .try_strategy_with_retries(&**fallback, &request, start_time, true)
                    .await;

                if fallback_result.is_ok() {
                    info!("Fallback strategy {} succeeded", fallback.name());
                    return fallback_result;
                }

                warn!("Fallback strategy {} failed", fallback.name());
            }

            // All strategies failed, try degraded service mode
            info!("All strategies failed, trying degraded service mode");
            let degraded_result = self.degraded_service_handler.handle_request(&request).await;

            // If degraded service mode fails, return the original error
            if degraded_result.is_err() {
                warn!("Degraded service mode failed");
                return Err(RouterError::FallbackError(format!(
                    "All strategies and degraded service mode failed. Original error: {}",
                    error
                )));
            }

            info!("Degraded service mode succeeded");
            return degraded_result;
        }

        result
    }

    fn get_config(&self) -> &RouterConfig {
        &self.config
    }

    fn update_config(&mut self, config: RouterConfig) -> Result<(), RouterError> {
        self.init(config)
    }

    fn get_metrics(&self) -> HashMap<String, serde_json::Value> {
        let metrics = self.metrics.lock().unwrap();
        metrics.clone()
    }

    fn clear_cache(&mut self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        debug!("Router cache cleared");
    }

    /// Get the model registry
    fn get_registry(&self) -> Arc<ModelRegistry> {
        self.registry.clone()
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
    fn test_router_creation() {
        let config = RouterConfig::default();
        let registry = Arc::new(ModelRegistry::new());

        let router = RouterImpl::new(config, registry);
        assert!(router.is_ok());
    }

    #[test]
    fn test_strategy_creation() {
        let config = RouterConfig::default();
        let registry = Arc::new(ModelRegistry::new());
        let router = RouterImpl::new(config, registry).unwrap();

        // Test RoundRobin strategy creation
        let strategy = router.create_strategy(&RoutingStrategy::RoundRobin, &None);
        assert!(strategy.is_ok());
        assert_eq!(
            strategy.unwrap().strategy_type(),
            RoutingStrategy::RoundRobin
        );

        // Test ContentBased strategy creation
        let strategy = router.create_strategy(&RoutingStrategy::ContentBased, &None);
        assert!(strategy.is_ok());
        assert_eq!(
            strategy.unwrap().strategy_type(),
            RoutingStrategy::ContentBased
        );

        // Test Custom strategy creation (should fail)
        let strategy = router.create_strategy(&RoutingStrategy::Custom, &None);
        assert!(strategy.is_err());
    }

    #[test]
    fn test_cache_key_generation() {
        let config = RouterConfig::default();
        let registry = Arc::new(ModelRegistry::new());
        let router = RouterImpl::new(config, registry).unwrap();

        let request = create_test_request();
        let key = router.generate_cache_key(&request);

        // The same request should generate the same key
        let key2 = router.generate_cache_key(&request);
        assert_eq!(key, key2);

        // Different requests should generate different keys
        let mut different_request = create_test_request();
        different_request.context.request.messages[0].content = "Different content".to_string();
        let different_key = router.generate_cache_key(&different_request);
        assert_ne!(key, different_key);
    }

    #[test]
    fn test_metrics_update() {
        let config = RouterConfig::default();
        let registry = Arc::new(ModelRegistry::new());
        let router = RouterImpl::new(config, registry).unwrap();

        // Create a test response
        let model = create_test_model("gpt-4", "openai");
        let metadata = RoutingMetadata {
            selected_model_id: "gpt-4".to_string(),
            strategy_name: "test_strategy".to_string(),
            routing_start_time: chrono::Utc::now(),
            routing_end_time: chrono::Utc::now(),
            routing_time_ms: 100,
            models_considered: 1,
            attempts: 1,
            is_fallback: false,
            selection_criteria: None,
            additional_metadata: HashMap::new(),
        };

        let response = RoutingResponse {
            response: crate::modules::model_registry::connectors::ChatCompletionResponse {
                id: "test-id".to_string(),
                model: "gpt-4".to_string(),
                created: 0,
                choices: vec![],
                usage: None,
            },
            metadata,
        };

        // Update metrics
        router.update_metrics(&response);

        // Check metrics
        let metrics = router.get_metrics();
        assert!(metrics.contains_key("request_count"));
        assert!(metrics.contains_key("model_usage"));
        assert!(metrics.contains_key("avg_routing_time_ms"));

        // Check model usage
        if let serde_json::Value::Object(map) = &metrics["model_usage"] {
            assert!(map.contains_key("gpt-4"));
        } else {
            panic!("Expected model_usage to be an object");
        }

        // Check strategy usage
        if let serde_json::Value::Object(map) = &metrics["strategy_usage"] {
            assert!(map.contains_key("test_strategy"));
        } else {
            panic!("Expected strategy_usage to be an object");
        }
    }
}
