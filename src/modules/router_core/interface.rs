//! Router Interface
//!
//! This module defines the interface for the router.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::modules::model_registry::storage::ModelRegistry;
use crate::modules::router_core::config::RouterConfig;
use crate::modules::router_core::errors::RouterError;
use crate::modules::router_core::request::RoutingRequest;
use crate::modules::router_core::response::RoutingResponse;

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

    /// Get the model registry
    fn get_registry(&self) -> Arc<ModelRegistry>;

    /// Validate service health before handling requests
    async fn validate_service_health(&self) -> Result<(), RouterError>;
}
