//! RAG Manager Service Health Check
//!
//! This module implements health check functionality specific to the RAG Manager service.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;
use tracing::{debug, error, info};

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
        let sources = self.rag_manager.list_sources().await?;
        diagnostics.insert("total_sources".to_string(), json!(sources.len()));

        let file_sources = sources.iter().filter(|s| s.source_type == "file").count();
        diagnostics.insert("file_sources".to_string(), json!(file_sources));

        let api_sources = sources.iter().filter(|s| s.source_type == "api").count();
        diagnostics.insert("api_sources".to_string(), json!(api_sources));

        let database_sources = sources
            .iter()
            .filter(|s| s.source_type == "database")
            .count();
        diagnostics.insert("database_sources".to_string(), json!(database_sources));

        // Add vector database stats
        let vector_db_stats = self.rag_manager.get_vector_db_stats().await?;
        diagnostics.insert(
            "vector_db_collections".to_string(),
            json!(vector_db_stats.collections),
        );

        diagnostics.insert(
            "vector_db_total_vectors".to_string(),
            json!(vector_db_stats.total_vectors),
        );

        // More detailed diagnostics for higher verbosity levels
        if verbosity >= 2 {
            // Add source details
            let source_details: Vec<serde_json::Value> = sources
                .iter()
                .map(|s| {
                    json!({
                        "id": s.id,
                        "name": s.name,
                        "source_type": s.source_type,
                        "document_count": s.document_count,
                        "last_updated": s.last_updated,
                    })
                })
                .collect();

            diagnostics.insert("sources".to_string(), json!(source_details));

            // Add retrieval statistics
            let retrieval_stats = self.rag_manager.get_retrieval_stats().await?;
            diagnostics.insert(
                "retrieval_stats".to_string(),
                json!({
                    "total_retrievals": retrieval_stats.total_retrievals,
                    "successful_retrievals": retrieval_stats.successful_retrievals,
                    "failed_retrievals": retrieval_stats.failed_retrievals,
                    "average_retrieval_time_ms": retrieval_stats.average_retrieval_time_ms,
                    "average_results_per_query": retrieval_stats.average_results_per_query,
                }),
            );
        }

        // Even more detailed diagnostics for highest verbosity level
        if verbosity >= 3 {
            // Add collection details
            let collections = self.rag_manager.list_collections().await?;
            let collection_details: Vec<serde_json::Value> = collections
                .iter()
                .map(|c| {
                    json!({
                        "name": c.name,
                        "vector_count": c.vector_count,
                        "dimension": c.dimension,
                        "metadata_schema": c.metadata_schema,
                    })
                })
                .collect();

            diagnostics.insert("collections".to_string(), json!(collection_details));

            // Add embedding model information
            diagnostics.insert(
                "embedding_model".to_string(),
                json!({
                    "name": self.rag_manager.get_embedding_model_info().await?.name,
                    "dimension": self.rag_manager.get_embedding_model_info().await?.dimension,
                    "provider": self.rag_manager.get_embedding_model_info().await?.provider,
                }),
            );

            // Add recent queries
            let recent_queries = self.rag_manager.get_recent_queries(10).await?;
            let query_details: Vec<serde_json::Value> = recent_queries
                .iter()
                .map(|q| {
                    json!({
                        "query": q.query,
                        "timestamp": q.timestamp,
                        "result_count": q.result_count,
                        "execution_time_ms": q.execution_time_ms,
                    })
                })
                .collect();

            diagnostics.insert("recent_queries".to_string(), json!(query_details));
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
