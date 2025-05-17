//! Persona Layer Service Health Check
//!
//! This module implements health check functionality specific to the Persona Layer service.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;
use tracing::{debug, error, info};

use crate::modules::health::{DiagnosticsProvider, HealthCheckManager, HttpDependencyChecker};
use crate::modules::persona_layer::manager::PersonaManager;

/// Persona Layer diagnostics provider
#[derive(Debug)]
pub struct PersonaLayerDiagnosticsProvider {
    /// Persona manager
    persona_manager: Arc<PersonaManager>,
}

impl PersonaLayerDiagnosticsProvider {
    /// Create a new persona layer diagnostics provider
    pub fn new(persona_manager: Arc<PersonaManager>) -> Self {
        Self { persona_manager }
    }
}

#[async_trait::async_trait]
impl DiagnosticsProvider for PersonaLayerDiagnosticsProvider {
    async fn get_diagnostics(
        &self,
        verbosity: u8,
    ) -> Result<HashMap<String, serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
        let mut diagnostics = HashMap::new();

        // Basic diagnostics (verbosity level 1)
        let personas = self.persona_manager.list_personas().await?;
        diagnostics.insert("total_personas".to_string(), json!(personas.len()));

        let active_personas = personas.iter().filter(|p| p.active).count();
        diagnostics.insert("active_personas".to_string(), json!(active_personas));

        let inactive_personas = personas.len() - active_personas;
        diagnostics.insert("inactive_personas".to_string(), json!(inactive_personas));

        // Add guardrail stats
        let guardrails = self.persona_manager.list_guardrails().await?;
        diagnostics.insert("total_guardrails".to_string(), json!(guardrails.len()));

        let active_guardrails = guardrails.iter().filter(|g| g.active).count();
        diagnostics.insert("active_guardrails".to_string(), json!(active_guardrails));

        // Add usage statistics
        let usage_stats = self.persona_manager.get_usage_stats().await?;
        diagnostics.insert(
            "total_requests".to_string(),
            json!(usage_stats.total_requests),
        );

        diagnostics.insert(
            "guardrail_blocks".to_string(),
            json!(usage_stats.guardrail_blocks),
        );

        diagnostics.insert(
            "average_response_time_ms".to_string(),
            json!(usage_stats.average_response_time_ms),
        );

        // More detailed diagnostics for higher verbosity levels
        if verbosity >= 2 {
            // Add persona details
            let persona_details: Vec<serde_json::Value> = personas
                .iter()
                .map(|p| {
                    json!({
                        "id": p.id,
                        "name": p.name,
                        "description": p.description,
                        "active": p.active,
                        "version": p.version,
                        "guardrails_count": p.guardrails.len(),
                    })
                })
                .collect();

            diagnostics.insert("personas".to_string(), json!(persona_details));

            // Add guardrail details
            let guardrail_details: Vec<serde_json::Value> = guardrails
                .iter()
                .map(|g| {
                    json!({
                        "id": g.id,
                        "name": g.name,
                        "type": g.guardrail_type,
                        "active": g.active,
                        "block_count": g.block_count,
                    })
                })
                .collect();

            diagnostics.insert("guardrails".to_string(), json!(guardrail_details));
        }

        // Even more detailed diagnostics for highest verbosity level
        if verbosity >= 3 {
            // Add recent persona usage
            let recent_usage = self.persona_manager.get_recent_persona_usage(10).await?;
            let usage_details: Vec<serde_json::Value> = recent_usage
                .iter()
                .map(|u| {
                    json!({
                        "persona_id": u.persona_id,
                        "timestamp": u.timestamp,
                        "request_type": u.request_type,
                        "response_time_ms": u.response_time_ms,
                        "guardrail_checks": u.guardrail_checks,
                        "guardrail_blocks": u.guardrail_blocks,
                    })
                })
                .collect();

            diagnostics.insert("recent_usage".to_string(), json!(usage_details));

            // Add recent guardrail blocks
            let recent_blocks = self.persona_manager.get_recent_guardrail_blocks(10).await?;
            let block_details: Vec<serde_json::Value> = recent_blocks
                .iter()
                .map(|b| {
                    json!({
                        "guardrail_id": b.guardrail_id,
                        "persona_id": b.persona_id,
                        "timestamp": b.timestamp,
                        "reason": b.reason,
                        "content_snippet": b.content_snippet,
                    })
                })
                .collect();

            diagnostics.insert("recent_blocks".to_string(), json!(block_details));

            // Add performance metrics
            diagnostics.insert(
                "performance_metrics".to_string(),
                json!({
                    "average_prompt_tokens": self.persona_manager.get_performance_metrics().await?.average_prompt_tokens,
                    "average_completion_tokens": self.persona_manager.get_performance_metrics().await?.average_completion_tokens,
                    "average_guardrail_check_time_ms": self.persona_manager.get_performance_metrics().await?.average_guardrail_check_time_ms,
                    "average_persona_application_time_ms": self.persona_manager.get_performance_metrics().await?.average_persona_application_time_ms,
                }),
            );
        }

        Ok(diagnostics)
    }
}

/// Create a health check manager for the Persona Layer service
pub fn create_persona_layer_health_manager(
    persona_manager: Arc<PersonaManager>,
    redis_url: Option<String>,
    router_endpoint: Option<String>,
) -> HealthCheckManager {
    let mut manager = HealthCheckManager::new("PersonaLayer", env!("CARGO_PKG_VERSION"), None);

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

    // Add persona layer diagnostics provider
    let diagnostics_provider = Arc::new(PersonaLayerDiagnosticsProvider::new(persona_manager));
    manager.set_diagnostics_provider(diagnostics_provider);

    manager
}
