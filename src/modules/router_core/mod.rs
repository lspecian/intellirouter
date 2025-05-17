//! Router Core Module
//!
//! This module contains the core routing logic for IntelliRouter.
//! It determines which model or service should handle a given request
//! based on various criteria such as content, user preferences, and system load.
//! The module provides a flexible and extensible framework for implementing
//! different routing strategies.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::modules::model_registry::{
    storage::ModelRegistry, ChatCompletionRequest, ChatCompletionResponse, ConnectorError,
    ModelFilter, ModelMetadata, ModelStatus, ModelType, RegistryError,
};

#[cfg(test)]
mod unit_tests;

pub mod registry_integration;
pub mod retry;
pub mod router;
pub mod strategies;
// Tests are in the tests directory

// Re-export types for easier access
pub use registry_integration::RegistryIntegration;
pub use retry::{CircuitBreakerConfig, DegradedServiceMode, ErrorCategory, RetryPolicy};
pub use router::RouterImpl;
pub use strategies::BaseStrategy;

//------------------------------------------------------------------------------
// Error Types
//------------------------------------------------------------------------------

/// Error types for the router core module
#[derive(Error, Debug, Clone)]
pub enum RouterError {
    /// No suitable model found for routing
    #[error("No suitable model found: {0}")]
    NoSuitableModel(String),

    /// Model registry error
    #[error("Model registry error: {0}")]
    RegistryError(#[from] RegistryError),

    /// Model connector error
    #[error("Model connector error: {0}")]
    ConnectorError(String),

    /// Strategy configuration error
    #[error("Strategy configuration error: {0}")]
    StrategyConfigError(String),

    /// Invalid request error
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Routing timeout error
    #[error("Routing timeout: {0}")]
    Timeout(String),

    /// Fallback error (when all fallbacks fail)
    #[error("All fallbacks failed: {0}")]
    FallbackError(String),

    /// Other errors
    #[error("Error: {0}")]
    Other(String),
}

impl From<ConnectorError> for RouterError {
    fn from(error: ConnectorError) -> Self {
        RouterError::ConnectorError(error.to_string())
    }
}

//------------------------------------------------------------------------------
// Routing Strategy Types
//------------------------------------------------------------------------------

/// Routing strategy options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingStrategy {
    /// Round-robin distribution across models
    RoundRobin,

    /// Load-balanced distribution based on model availability and capacity
    LoadBalanced,

    /// Content-based routing using request analysis
    ContentBased,

    /// Cost-optimized routing to minimize token costs
    CostOptimized,

    /// Latency-optimized routing for fastest response times
    LatencyOptimized,

    /// Custom strategy (requires custom implementation)
    Custom,
}

impl fmt::Display for RoutingStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoutingStrategy::RoundRobin => write!(f, "RoundRobin"),
            RoutingStrategy::LoadBalanced => write!(f, "LoadBalanced"),
            RoutingStrategy::ContentBased => write!(f, "ContentBased"),
            RoutingStrategy::CostOptimized => write!(f, "CostOptimized"),
            RoutingStrategy::LatencyOptimized => write!(f, "LatencyOptimized"),
            RoutingStrategy::Custom => write!(f, "Custom"),
        }
    }
}

//------------------------------------------------------------------------------
// Routing Context and Request/Response Types
//------------------------------------------------------------------------------

/// Routing context containing information used during routing
#[derive(Debug, Clone)]
pub struct RoutingContext {
    /// Original chat completion request
    pub request: ChatCompletionRequest,

    /// User ID (if available)
    pub user_id: Option<String>,

    /// Organization ID (if available)
    pub org_id: Option<String>,

    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Request priority (higher values indicate higher priority)
    pub priority: u8,

    /// Request tags for categorization
    pub tags: Vec<String>,

    /// Additional context parameters
    pub parameters: HashMap<String, String>,
}

impl RoutingContext {
    /// Create a new routing context from a chat completion request
    pub fn new(request: ChatCompletionRequest) -> Self {
        Self {
            request,
            user_id: None,
            org_id: None,
            timestamp: chrono::Utc::now(),
            priority: 0,
            tags: Vec::new(),
            parameters: HashMap::new(),
        }
    }

    /// Add a parameter to the routing context
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    /// Set the user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set the organization ID
    pub fn with_org_id(mut self, org_id: impl Into<String>) -> Self {
        self.org_id = Some(org_id.into());
        self
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// Routing request wrapping a chat completion request with routing metadata
#[derive(Debug, Clone)]
pub struct RoutingRequest {
    /// Routing context
    pub context: RoutingContext,

    /// Model filter for selecting eligible models
    pub model_filter: Option<ModelFilter>,

    /// Preferred model ID (if any)
    pub preferred_model_id: Option<String>,

    /// Excluded model IDs
    pub excluded_model_ids: Vec<String>,

    /// Maximum routing attempts
    pub max_attempts: u32,

    /// Routing timeout
    pub timeout: Duration,
}

impl RoutingRequest {
    /// Create a new routing request from a chat completion request
    pub fn new(request: ChatCompletionRequest) -> Self {
        Self {
            context: RoutingContext::new(request),
            model_filter: None,
            preferred_model_id: None,
            excluded_model_ids: Vec::new(),
            max_attempts: 3,
            timeout: Duration::from_secs(30),
        }
    }

    /// Set a model filter
    pub fn with_model_filter(mut self, filter: ModelFilter) -> Self {
        self.model_filter = Some(filter);
        self
    }

    /// Set a preferred model ID
    pub fn with_preferred_model(mut self, model_id: impl Into<String>) -> Self {
        self.preferred_model_id = Some(model_id.into());
        self
    }

    /// Add an excluded model ID
    pub fn exclude_model(mut self, model_id: impl Into<String>) -> Self {
        self.excluded_model_ids.push(model_id.into());
        self
    }

    /// Set the maximum number of routing attempts
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Set the routing timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Metadata about a routing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetadata {
    /// ID of the selected model
    pub selected_model_id: String,

    /// Name of the strategy that made the selection
    pub strategy_name: String,

    /// Timestamp when routing started
    pub routing_start_time: chrono::DateTime<chrono::Utc>,

    /// Timestamp when routing completed
    pub routing_end_time: chrono::DateTime<chrono::Utc>,

    /// Total routing time in milliseconds
    pub routing_time_ms: u64,

    /// Number of models considered during routing
    pub models_considered: u32,

    /// Number of routing attempts made
    pub attempts: u32,

    /// Whether this was a fallback selection
    pub is_fallback: bool,

    /// Selection criteria used (e.g., "lowest_cost", "lowest_latency")
    pub selection_criteria: Option<String>,

    /// Additional metadata about the routing decision
    pub additional_metadata: HashMap<String, String>,
}

/// Routing response containing the chat completion response and routing metadata
#[derive(Debug, Clone)]
pub struct RoutingResponse {
    /// Original chat completion response
    pub response: ChatCompletionResponse,

    /// Metadata about the routing decision
    pub metadata: RoutingMetadata,
}

//------------------------------------------------------------------------------
// Strategy Configuration Types
//------------------------------------------------------------------------------

/// Base configuration for all routing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    /// Strategy-specific parameters
    pub parameters: HashMap<String, serde_json::Value>,

    /// Fallback strategy to use if this strategy fails
    pub fallback_strategy: Option<Box<StrategyConfig>>,

    /// Maximum number of fallback attempts
    pub max_fallback_attempts: u32,

    /// Whether to include models with limited status
    pub include_limited_models: bool,

    /// Whether to include models with high latency
    pub include_high_latency_models: bool,

    /// Timeout for strategy execution in milliseconds
    pub timeout_ms: u64,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            parameters: HashMap::new(),
            fallback_strategy: None,
            max_fallback_attempts: 2,
            include_limited_models: false,
            include_high_latency_models: false,
            timeout_ms: 5000,
        }
    }
}

/// Round-robin strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundRobinConfig {
    /// Base strategy configuration
    #[serde(flatten)]
    pub base: StrategyConfig,

    /// Whether to weight the distribution by model capacity
    pub weighted: bool,
}

impl Default for RoundRobinConfig {
    fn default() -> Self {
        Self {
            base: StrategyConfig::default(),
            weighted: false,
        }
    }
}

/// Load-balanced strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancedConfig {
    /// Base strategy configuration
    #[serde(flatten)]
    pub base: StrategyConfig,

    /// Weight factor for model capacity (0.0 to 1.0)
    pub capacity_weight: f32,

    /// Weight factor for current load (0.0 to 1.0)
    pub load_weight: f32,

    /// Weight factor for model performance (0.0 to 1.0)
    pub performance_weight: f32,
}

impl Default for LoadBalancedConfig {
    fn default() -> Self {
        Self {
            base: StrategyConfig::default(),
            capacity_weight: 0.4,
            load_weight: 0.4,
            performance_weight: 0.2,
        }
    }
}

/// Content-based strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBasedConfig {
    /// Base strategy configuration
    #[serde(flatten)]
    pub base: StrategyConfig,

    /// Whether to analyze message content for routing
    pub analyze_content: bool,

    /// Whether to consider model capabilities for specific content types
    pub match_capabilities: bool,

    /// Whether to consider language detection for routing
    pub language_detection: bool,

    /// Maximum content length to analyze (in characters)
    pub max_analysis_length: usize,
}

impl Default for ContentBasedConfig {
    fn default() -> Self {
        Self {
            base: StrategyConfig::default(),
            analyze_content: true,
            match_capabilities: true,
            language_detection: true,
            max_analysis_length: 1000,
        }
    }
}

/// Cost-optimized strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostOptimizedConfig {
    /// Base strategy configuration
    #[serde(flatten)]
    pub base: StrategyConfig,

    /// Maximum cost per 1K tokens (input)
    pub max_cost_per_1k_input: Option<f64>,

    /// Maximum cost per 1K tokens (output)
    pub max_cost_per_1k_output: Option<f64>,

    /// Whether to estimate token count for cost calculation
    pub estimate_token_count: bool,

    /// Weight factor for balancing cost vs. performance (0.0 to 1.0, higher values favor performance)
    pub performance_cost_balance: f32,
}

impl Default for CostOptimizedConfig {
    fn default() -> Self {
        Self {
            base: StrategyConfig::default(),
            max_cost_per_1k_input: None,
            max_cost_per_1k_output: None,
            estimate_token_count: true,
            performance_cost_balance: 0.3,
        }
    }
}

/// Latency-optimized strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyOptimizedConfig {
    /// Base strategy configuration
    #[serde(flatten)]
    pub base: StrategyConfig,

    /// Maximum acceptable latency in milliseconds
    pub max_latency_ms: Option<f64>,

    /// Whether to use historical latency data
    pub use_historical_data: bool,

    /// Weight factor for balancing latency vs. quality (0.0 to 1.0, higher values favor quality)
    pub quality_latency_balance: f32,
}

impl Default for LatencyOptimizedConfig {
    fn default() -> Self {
        Self {
            base: StrategyConfig::default(),
            max_latency_ms: None,
            use_historical_data: true,
            quality_latency_balance: 0.3,
        }
    }
}

/// Custom strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomStrategyConfig {
    /// Base strategy configuration
    #[serde(flatten)]
    pub base: StrategyConfig,

    /// Strategy implementation identifier
    pub implementation_id: String,

    /// Custom configuration (JSON value)
    pub custom_config: serde_json::Value,
}

//------------------------------------------------------------------------------
// Router Configuration
//------------------------------------------------------------------------------

/// Router configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Primary routing strategy
    pub strategy: RoutingStrategy,

    /// Strategy-specific configuration
    pub strategy_config: Option<StrategyConfig>,

    /// Fallback strategies in order of preference
    pub fallback_strategies: Vec<RoutingStrategy>,

    /// Global timeout for routing operations in milliseconds
    pub global_timeout_ms: u64,

    /// Maximum number of routing attempts
    pub max_routing_attempts: u32,

    /// Whether to cache routing decisions
    pub cache_routing_decisions: bool,

    /// Maximum size of the routing decision cache
    pub max_cache_size: usize,

    /// Whether to collect routing metrics
    pub collect_metrics: bool,

    /// Retry policy
    pub retry_policy: RetryPolicy,

    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,

    /// Degraded service mode
    pub degraded_service_mode: DegradedServiceMode,

    /// Error categories that should be retried
    pub retryable_errors: HashSet<ErrorCategory>,

    /// Additional configuration parameters
    pub additional_config: HashMap<String, String>,
}

impl Default for RouterConfig {
    fn default() -> Self {
        // Default retryable error categories
        let mut retryable_errors = HashSet::new();
        retryable_errors.insert(ErrorCategory::Network);
        retryable_errors.insert(ErrorCategory::Timeout);
        retryable_errors.insert(ErrorCategory::RateLimit);
        retryable_errors.insert(ErrorCategory::Server);

        Self {
            strategy: RoutingStrategy::ContentBased,
            strategy_config: Some(StrategyConfig::default()),
            fallback_strategies: vec![RoutingStrategy::LoadBalanced, RoutingStrategy::RoundRobin],
            global_timeout_ms: 10000,
            max_routing_attempts: 3,
            cache_routing_decisions: true,
            max_cache_size: 1000,
            collect_metrics: true,
            retry_policy: RetryPolicy::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            degraded_service_mode: DegradedServiceMode::default(),
            retryable_errors,
            additional_config: HashMap::new(),
        }
    }
}

//------------------------------------------------------------------------------
// Strategy Trait
//------------------------------------------------------------------------------

/// Trait for routing strategies
#[async_trait]
pub trait RoutingStrategyTrait: Send + Sync + std::fmt::Debug {
    /// Get the strategy name
    fn name(&self) -> &'static str;

    /// Get the strategy type
    fn strategy_type(&self) -> RoutingStrategy;

    /// Select a model for the given request
    async fn select_model(
        &self,
        request: &RoutingRequest,
        registry: &ModelRegistry,
    ) -> Result<ModelMetadata, RouterError>;

    /// Handle a routing failure
    async fn handle_failure(
        &self,
        request: &RoutingRequest,
        failed_model_id: &str,
        error: &RouterError,
        registry: &ModelRegistry,
    ) -> Result<ModelMetadata, RouterError>;

    /// Get metadata about the routing decision
    fn get_routing_metadata(
        &self,
        model: &ModelMetadata,
        start_time: Instant,
        attempts: u32,
        is_fallback: bool,
    ) -> RoutingMetadata {
        RoutingMetadata {
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
        }
    }
}

//------------------------------------------------------------------------------
// Router Interface
//------------------------------------------------------------------------------

/// Router interface
#[async_trait]
pub trait Router: Send + Sync {
    /// Initialize the router with the specified configuration
    fn init(&mut self, config: RouterConfig) -> Result<(), RouterError>;

    /// Route a request to the appropriate model
    async fn route(&self, request: RoutingRequest) -> Result<RoutingResponse, RouterError>;

    /// Get the current router configuration
    fn get_config(&self) -> &RouterConfig;

    /// Update the router configuration
    fn update_config(&mut self, config: RouterConfig) -> Result<(), RouterError>;

    /// Get routing metrics
    fn get_metrics(&self) -> HashMap<String, serde_json::Value>;

    /// Clear the routing decision cache
    fn clear_cache(&mut self);
}

//------------------------------------------------------------------------------
// Module Functions
//------------------------------------------------------------------------------

/// Initialize the router with the specified configuration
pub fn init(config: RouterConfig) -> Result<(), RouterError> {
    // Get the global registry
    let registry = crate::modules::model_registry::global_registry().registry();
    let mut router = RouterImpl::new(config.clone(), registry)?;
    router.init(config)
}

/// Route a request to the appropriate model or service
pub async fn route_request(request: &str) -> Result<String, RouterError> {
    // Parse the request
    let chat_request: crate::modules::model_registry::connectors::ChatCompletionRequest =
        serde_json::from_str(request)
            .map_err(|e| RouterError::InvalidRequest(format!("Invalid request: {}", e)))?;

    // Create a routing request
    let routing_request = RoutingRequest::new(chat_request);

    // Get the router
    let registry = crate::modules::model_registry::global_registry().registry();
    let router = RouterImpl::new(RouterConfig::default(), registry)?;

    // Route the request
    let response = router.route(routing_request).await?;

    // Serialize the response
    let response_str = serde_json::to_string(&response.response)
        .map_err(|e| RouterError::Other(format!("Failed to serialize response: {}", e)))?;

    Ok(response_str)
}
