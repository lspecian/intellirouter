//! Routing Strategy Types and Traits
//!
//! This module defines the routing strategy types and traits used by the router.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::modules::model_registry::{ModelMetadata, ModelRegistry};
use crate::modules::router_core::errors::RouterError;
use crate::modules::router_core::request::RoutingRequest;
use crate::modules::router_core::response::RoutingMetadata;

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
