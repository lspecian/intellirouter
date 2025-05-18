//! Router Configuration
//!
//! This module defines the configuration structures for the router.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::modules::router_core::retry::{
    CircuitBreakerConfig, DegradedServiceMode, ErrorCategory, RetryPolicy,
};
use crate::modules::router_core::strategy::RoutingStrategy;

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
