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
        let personas = self.persona_manager.list_personas();
        // No need for ? operator as list_personas doesn't return a Result
        diagnostics.insert("total_personas".to_string(), json!(personas.len()));

        // Since Persona doesn't have an active field, we'll assume all personas are active
        let active_personas = personas.len();
        diagnostics.insert("active_personas".to_string(), json!(active_personas));

        let inactive_personas = 0;
        diagnostics.insert("inactive_personas".to_string(), json!(inactive_personas));

        // Stub implementation for guardrail stats
        // TODO: Implement list_guardrails in PersonaManager
        diagnostics.insert("total_guardrails".to_string(), json!(0));
        diagnostics.insert("active_guardrails".to_string(), json!(0));

        // Stub implementation for usage statistics
        // TODO: Implement get_usage_stats in PersonaManager
        diagnostics.insert("total_requests".to_string(), json!(0));

        // Stub implementation for guardrail blocks
        // TODO: Implement get_usage_stats in PersonaManager
        diagnostics.insert("guardrail_blocks".to_string(), json!(0));

        diagnostics.insert("average_response_time_ms".to_string(), json!(0));

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
                        "active": true, // Assuming all personas are active
                        "examples_count": p.few_shot_examples.len(),
                        "guardrails_count": p.guardrails.len(),
                    })
                })
                .collect();

            diagnostics.insert("personas".to_string(), json!(persona_details));

            // Add guardrail details
            // TODO: Implement list_guardrails in PersonaManager
            let guardrail_details: Vec<serde_json::Value> = Vec::new();
            diagnostics.insert("guardrails".to_string(), json!(guardrail_details));
        }

        // Even more detailed diagnostics for highest verbosity level
        if verbosity >= 3 {
            // Stub implementation for recent persona usage
            // TODO: Implement get_recent_persona_usage in PersonaManager
            let usage_details: Vec<serde_json::Value> = Vec::new();
            diagnostics.insert("recent_usage".to_string(), json!(usage_details));

            // Stub implementation for recent guardrail blocks
            // TODO: Implement get_recent_guardrail_blocks in PersonaManager
            let block_details: Vec<serde_json::Value> = Vec::new();

            diagnostics.insert("recent_blocks".to_string(), json!(block_details));

            // Stub implementation for performance metrics
            // TODO: Implement get_performance_metrics in PersonaManager
            diagnostics.insert(
                "performance_metrics".to_string(),
                json!({
                    "average_prompt_tokens": 0,
                    "average_completion_tokens": 0,
                    "average_guardrail_check_time_ms": 0,
                    "average_persona_application_time_ms": 0,
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
