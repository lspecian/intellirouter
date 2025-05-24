//! Router Service Health Check
//!
//! This module implements health check functionality specific to the Router service.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;

use crate::modules::health::{DiagnosticsProvider, HealthCheckManager};
use crate::modules::model_registry::storage::ModelRegistry;
use crate::modules::router_core::{RetryPolicy, RouterConfig};

/// Router diagnostics provider
#[derive(Debug)]
pub struct RouterDiagnosticsProvider {
    /// Model registry
    model_registry: Arc<ModelRegistry>,
    /// Router configuration
    router_config: RouterConfig,
}

impl RouterDiagnosticsProvider {
    /// Create a new router diagnostics provider
    pub fn new(model_registry: Arc<ModelRegistry>, router_config: RouterConfig) -> Self {
        Self {
            model_registry,
            router_config,
        }
    }
}

#[async_trait::async_trait]
impl DiagnosticsProvider for RouterDiagnosticsProvider {
    async fn get_diagnostics(
        &self,
        verbosity: u8,
    ) -> Result<HashMap<String, serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
        let mut diagnostics = HashMap::new();

        // Basic diagnostics (verbosity level 1)
        let models = self.model_registry.list_models();
        diagnostics.insert("total_models".to_string(), json!(models.len()));

        let available_models = models.iter().filter(|m| m.status.is_available()).count();
        diagnostics.insert("available_models".to_string(), json!(available_models));

        let unavailable_models = models.len() - available_models;
        diagnostics.insert("unavailable_models".to_string(), json!(unavailable_models));

        // Add routing strategy information
        diagnostics.insert(
            "routing_strategy".to_string(),
            json!(self.router_config.strategy.to_string()),
        );

        // Add retry policy information
        // RetryPolicy is an enum, not a struct with enabled field
        diagnostics.insert(
            "retry_enabled".to_string(),
            json!(match self.router_config.retry_policy {
                RetryPolicy::None => false,
                _ => true,
            }),
        );

        // RetryPolicy is an enum, max_retries depends on the variant
        diagnostics.insert(
            "max_retries".to_string(),
            json!(match &self.router_config.retry_policy {
                RetryPolicy::None => 0,
                RetryPolicy::Fixed { max_retries, .. } => *max_retries,
                RetryPolicy::ExponentialBackoff { max_retries, .. } => *max_retries,
            }),
        );

        // More detailed diagnostics for higher verbosity levels
        if verbosity >= 2 {
            // Add model details
            let model_details: Vec<serde_json::Value> = models
                .iter()
                .map(|m| {
                    json!({
                        "id": m.id,
                        "name": m.name,
                        "provider": m.provider,
                        "version": m.version,
                        "status": m.status.to_string(),
                    })
                })
                .collect();

            diagnostics.insert("models".to_string(), json!(model_details));

            // Add circuit breaker configuration
            diagnostics.insert(
                "circuit_breaker".to_string(),
                json!({
                    "enabled": self.router_config.circuit_breaker.enabled,
                    "failure_threshold": self.router_config.circuit_breaker.failure_threshold,
                    "reset_timeout_ms": self.router_config.circuit_breaker.reset_timeout_ms,
                    "success_threshold": self.router_config.circuit_breaker.success_threshold,
                }),
            );
        }

        // Even more detailed diagnostics for highest verbosity level
        if verbosity >= 3 {
            // Add full router configuration
            diagnostics.insert(
                "full_config".to_string(),
                json!({
                    "strategy": self.router_config.strategy.to_string(),
                    "fallback_strategies": self.router_config.fallback_strategies.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                    "global_timeout_ms": self.router_config.global_timeout_ms,
                    "max_routing_attempts": self.router_config.max_routing_attempts,
                    "cache_routing_decisions": self.router_config.cache_routing_decisions,
                    "max_cache_size": self.router_config.max_cache_size,
                    "collect_metrics": self.router_config.collect_metrics,
                }),
            );
        }

        Ok(diagnostics)
    }
}

/// Create a health check manager for the Router service
pub fn create_router_health_manager(
    model_registry: Arc<ModelRegistry>,
    router_config: RouterConfig,
    redis_url: Option<String>,
) -> HealthCheckManager {
    let mut manager = HealthCheckManager::new("Router", env!("CARGO_PKG_VERSION"), None);

    // Add Redis dependency checker if Redis URL is provided
    if let Some(redis_url) = redis_url {
        let redis_checker = Arc::new(crate::modules::health::RedisDependencyChecker::new(
            redis_url,
        ));
        manager.add_dependency_checker(redis_checker);
    }

    // Add model registry diagnostics provider
    let diagnostics_provider = Arc::new(RouterDiagnosticsProvider::new(
        model_registry,
        router_config,
    ));
    manager.set_diagnostics_provider(diagnostics_provider);

    manager
}
