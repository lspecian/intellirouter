//! Chain Engine Service Health Check
//!
//! This module implements health check functionality specific to the Chain Engine service.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;
use tracing::{debug, error, info};

use crate::modules::chain_engine::ChainEngine;
use crate::modules::health::{DiagnosticsProvider, HealthCheckManager, HttpDependencyChecker};

/// Chain Engine diagnostics provider
#[derive(Debug)]
pub struct ChainEngineDiagnosticsProvider {
    /// Chain engine
    chain_engine: Arc<ChainEngine>,
}

impl ChainEngineDiagnosticsProvider {
    /// Create a new chain engine diagnostics provider
    pub fn new(chain_engine: Arc<ChainEngine>) -> Self {
        Self { chain_engine }
    }
}

#[async_trait::async_trait]
impl DiagnosticsProvider for ChainEngineDiagnosticsProvider {
    async fn get_diagnostics(
        &self,
        verbosity: u8,
    ) -> Result<HashMap<String, serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
        let mut diagnostics = HashMap::new();

        // Basic diagnostics (verbosity level 1)
        let stats = self.chain_engine.get_execution_stats();

        diagnostics.insert(
            "total_chains_executed".to_string(),
            json!(stats.total_executions),
        );

        diagnostics.insert(
            "successful_executions".to_string(),
            json!(stats.successful_executions),
        );

        diagnostics.insert(
            "failed_executions".to_string(),
            json!(stats.failed_executions),
        );

        diagnostics.insert(
            "average_execution_time_ms".to_string(),
            json!(stats.avg_execution_time_ms),
        );

        // More detailed diagnostics for higher verbosity levels
        if verbosity >= 2 {
            // Stub implementation for executor-specific statistics
            // TODO: Implement get_executor_stats in ChainEngine
            let executor_details: Vec<serde_json::Value> = Vec::new();
            diagnostics.insert("executor_stats".to_string(), json!(executor_details));

            // Stub implementation for recent executions
            // TODO: Implement get_recent_executions in ChainEngine
            let execution_details: Vec<serde_json::Value> = Vec::new();
            diagnostics.insert("recent_executions".to_string(), json!(execution_details));
        }

        // Even more detailed diagnostics for highest verbosity level
        if verbosity >= 3 {
            // Add registered chain definitions
            // TODO: Implement list_chain_definitions in ChainEngine
            let chain_details: Vec<serde_json::Value> = Vec::new();
            diagnostics.insert("chain_definitions".to_string(), json!(chain_details));

            // Add memory usage statistics
            // TODO: Implement get_context_cache_size and get_result_cache_size in ChainEngine
            diagnostics.insert(
                "memory_usage".to_string(),
                json!({
                    "context_cache_size": 0,
                    "result_cache_size": 0,
                }),
            );
        }

        Ok(diagnostics)
    }
}

/// Create a health check manager for the Chain Engine service
pub fn create_chain_engine_health_manager(
    chain_engine: Arc<ChainEngine>,
    redis_url: Option<String>,
    router_endpoint: Option<String>,
) -> HealthCheckManager {
    let mut manager = HealthCheckManager::new("ChainEngine", env!("CARGO_PKG_VERSION"), None);

    // Add Redis dependency checker if Redis URL is provided
    if let Some(redis_url) = redis_url {
        let redis_checker = Arc::new(crate::modules::health::RedisDependencyChecker::new(
            redis_url,
        ));
        manager.add_dependency_checker(redis_checker);
    }

    // Add Router dependency checker if Router endpoint is provided
    if let Some(router_endpoint) = router_endpoint {
        let router_health_url = format!("{}/health", router_endpoint);
        let router_checker = Arc::new(HttpDependencyChecker::new("router", router_health_url, 200));
        manager.add_dependency_checker(router_checker);
    }

    // Add chain engine diagnostics provider
    let diagnostics_provider = Arc::new(ChainEngineDiagnosticsProvider::new(chain_engine));
    manager.set_diagnostics_provider(diagnostics_provider);

    manager
}
