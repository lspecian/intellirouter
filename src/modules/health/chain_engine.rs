//! Chain Engine Service Health Check
//!
//! This module implements health check functionality specific to the Chain Engine service.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;
use tracing::{debug, error, info};

use crate::modules::chain_engine::core::ChainEngineCore;
use crate::modules::health::{DiagnosticsProvider, HealthCheckManager, HttpDependencyChecker};

/// Chain Engine diagnostics provider
#[derive(Debug)]
pub struct ChainEngineDiagnosticsProvider {
    /// Chain engine core
    chain_engine: Arc<ChainEngineCore>,
}

impl ChainEngineDiagnosticsProvider {
    /// Create a new chain engine diagnostics provider
    pub fn new(chain_engine: Arc<ChainEngineCore>) -> Self {
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
            json!(stats.average_execution_time_ms),
        );

        // More detailed diagnostics for higher verbosity levels
        if verbosity >= 2 {
            // Add executor-specific statistics
            let executor_stats = self.chain_engine.get_executor_stats();
            let executor_details: Vec<serde_json::Value> = executor_stats
                .iter()
                .map(|(executor_type, stats)| {
                    json!({
                        "executor_type": executor_type,
                        "executions": stats.executions,
                        "successes": stats.successes,
                        "failures": stats.failures,
                        "average_time_ms": stats.average_time_ms,
                    })
                })
                .collect();

            diagnostics.insert("executor_stats".to_string(), json!(executor_details));

            // Add recent executions
            let recent_executions = self.chain_engine.get_recent_executions(10);
            let execution_details: Vec<serde_json::Value> = recent_executions
                .iter()
                .map(|execution| {
                    json!({
                        "chain_id": execution.chain_id,
                        "status": execution.status.to_string(),
                        "execution_time_ms": execution.execution_time_ms,
                        "timestamp": execution.timestamp,
                    })
                })
                .collect();

            diagnostics.insert("recent_executions".to_string(), json!(execution_details));
        }

        // Even more detailed diagnostics for highest verbosity level
        if verbosity >= 3 {
            // Add registered chain definitions
            let chain_definitions = self.chain_engine.list_chain_definitions();
            let chain_details: Vec<serde_json::Value> = chain_definitions
                .iter()
                .map(|def| {
                    json!({
                        "id": def.id,
                        "name": def.name,
                        "version": def.version,
                        "steps": def.steps.len(),
                    })
                })
                .collect();

            diagnostics.insert("chain_definitions".to_string(), json!(chain_details));

            // Add memory usage statistics
            diagnostics.insert(
                "memory_usage".to_string(),
                json!({
                    "context_cache_size": self.chain_engine.get_context_cache_size(),
                    "result_cache_size": self.chain_engine.get_result_cache_size(),
                }),
            );
        }

        Ok(diagnostics)
    }
}

/// Create a health check manager for the Chain Engine service
pub fn create_chain_engine_health_manager(
    chain_engine: Arc<ChainEngineCore>,
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
