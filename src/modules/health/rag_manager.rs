//! RAG Manager Service Health Check
//!
//! This module implements health check functionality specific to the RAG Manager service.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;

use crate::modules::health::{DiagnosticsProvider, HealthCheckManager, HttpDependencyChecker};
use crate::modules::rag_manager::manager::RagManager;

/// RAG Manager diagnostics provider
#[derive(Debug)]
pub struct RagManagerDiagnosticsProvider {
    /// RAG manager
    rag_manager: Arc<RagManager>,
}

impl RagManagerDiagnosticsProvider {
    /// Create a new RAG manager diagnostics provider
    pub fn new(rag_manager: Arc<RagManager>) -> Self {
        Self { rag_manager }
    }
}

#[async_trait::async_trait]
impl DiagnosticsProvider for RagManagerDiagnosticsProvider {
    async fn get_diagnostics(
        &self,
        verbosity: u8,
    ) -> Result<HashMap<String, serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
        let mut diagnostics = HashMap::new();

        // Basic diagnostics (verbosity level 1)
        let sources = self.rag_manager.list_sources();
        diagnostics.insert("total_sources".to_string(), json!(sources.len()));

        // Count sources by type
        let file_sources = sources
            .iter()
            .filter(|(_, source_type, _)| source_type == "file")
            .count();
        let api_sources = sources
            .iter()
            .filter(|(_, source_type, _)| source_type == "api")
            .count();
        let database_sources = sources
            .iter()
            .filter(|(_, source_type, _)| source_type == "database")
            .count();

        diagnostics.insert("file_sources".to_string(), json!(file_sources));
        diagnostics.insert("api_sources".to_string(), json!(api_sources));
        diagnostics.insert("database_sources".to_string(), json!(database_sources));

        // Add vector database stats
        let vector_db_stats = self.rag_manager.get_vector_db_stats().await;
        let total_collections = vector_db_stats.len();
        let total_vectors: usize = vector_db_stats.values().sum();

        diagnostics.insert(
            "vector_db_collections".to_string(),
            json!(total_collections),
        );
        diagnostics.insert("vector_db_total_vectors".to_string(), json!(total_vectors));

        // More detailed diagnostics for higher verbosity levels
        if verbosity >= 2 {
            // Add source details
            let source_details: Vec<serde_json::Value> = sources
                .iter()
                .map(|(name, source_type, metadata)| {
                    let mut source_json = serde_json::Map::new();
                    source_json.insert("name".to_string(), json!(name));
                    source_json.insert("type".to_string(), json!(source_type));

                    let metadata_json = metadata
                        .iter()
                        .map(|(k, v)| (k.clone(), json!(v)))
                        .collect::<serde_json::Map<String, serde_json::Value>>();

                    source_json.insert("metadata".to_string(), json!(metadata_json));
                    serde_json::Value::Object(source_json)
                })
                .collect();

            diagnostics.insert("sources".to_string(), json!(source_details));

            // Add retrieval statistics
            let retrieval_stats = self.rag_manager.get_retrieval_stats();
            diagnostics.insert("retrieval_stats".to_string(), json!(retrieval_stats));
        }

        // Even more detailed diagnostics for highest verbosity level
        if verbosity >= 3 {
            // Add collection details
            let collections = self.rag_manager.list_collections().await;
            diagnostics.insert("collections".to_string(), json!(collections));

            // Add embedding model information
            let embedding_model_info = self.rag_manager.get_embedding_model_info();
            diagnostics.insert("embedding_model".to_string(), json!(embedding_model_info));

            // Add recent queries
            let recent_queries = self.rag_manager.get_recent_queries();
            diagnostics.insert("recent_queries".to_string(), json!(recent_queries));
        }

        Ok(diagnostics)
    }
}

/// Create a health check manager for the RAG Manager service
pub fn create_rag_manager_health_manager(
    rag_manager: Arc<RagManager>,
    redis_url: Option<String>,
    router_endpoint: Option<String>,
    vector_db_url: Option<String>,
) -> HealthCheckManager {
    let mut manager = HealthCheckManager::new("RagManager", env!("CARGO_PKG_VERSION"), None);

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

    // Add Vector DB dependency checker if Vector DB URL is provided
    if let Some(vector_db_url) = vector_db_url {
        let vector_db_health_url = format!("{}/api/v1/heartbeat", vector_db_url);
        let vector_db_checker = Arc::new(HttpDependencyChecker::new(
            "vector_db",
            vector_db_health_url,
            200,
        ));
        manager.add_dependency_checker(vector_db_checker);
    }

    // Add RAG manager diagnostics provider
    let diagnostics_provider = Arc::new(RagManagerDiagnosticsProvider::new(rag_manager));
    manager.set_diagnostics_provider(diagnostics_provider);

    manager
}
