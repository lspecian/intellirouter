//! Persona Layer Service Health Check
//!
//! This module implements health check functionality specific to the Persona Layer service.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;

use crate::modules::health::{DiagnosticsProvider, HealthCheckManager, HttpDependencyChecker};
use crate::modules::persona_layer::manager::PersonaManager;
use crate::modules::persona_layer::Guardrail;

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

        // Get guardrail stats
        let guardrails = self.persona_manager.list_guardrails();
        diagnostics.insert("total_guardrails".to_string(), json!(guardrails.len()));

        // Assuming all guardrails are active
        diagnostics.insert("active_guardrails".to_string(), json!(guardrails.len()));

        // Get usage statistics
        let usage_stats = self.persona_manager.get_usage_stats();

        // Extract specific metrics from usage stats
        let total_requests = usage_stats.get("total_requests").cloned().unwrap_or(0);
        let guardrail_blocks = usage_stats.get("guardrail_blocks").cloned().unwrap_or(0);
        let avg_response_time = usage_stats
            .get("average_response_time_ms")
            .cloned()
            .unwrap_or(0);

        diagnostics.insert("total_requests".to_string(), json!(total_requests));
        diagnostics.insert("guardrail_blocks".to_string(), json!(guardrail_blocks));
        diagnostics.insert(
            "average_response_time_ms".to_string(),
            json!(avg_response_time),
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
                        "active": true, // Assuming all personas are active
                        "examples_count": p.few_shot_examples.len(),
                        "guardrails_count": p.guardrails.len(),
                    })
                })
                .collect();

            diagnostics.insert("personas".to_string(), json!(persona_details));

            // Add guardrail details
            let guardrail_details: Vec<serde_json::Value> = guardrails
                .iter()
                .map(|(persona_id, persona_name, guardrail)| {
                    let guardrail_type = match guardrail {
                        Guardrail::ResponseFormat { .. } => "response_format",
                        Guardrail::ContentFilter { .. } => "content_filter",
                        Guardrail::TopicRestriction { .. } => "topic_restriction",
                    };

                    json!({
                        "persona_id": persona_id,
                        "persona_name": persona_name,
                        "type": guardrail_type,
                        "active": true, // Assuming all guardrails are active
                    })
                })
                .collect();

            diagnostics.insert("guardrails".to_string(), json!(guardrail_details));
        }

        // Even more detailed diagnostics for highest verbosity level
        if verbosity >= 3 {
            // Get recent persona usage
            let usage_details = self.persona_manager.get_recent_persona_usage();
            diagnostics.insert("recent_usage".to_string(), json!(usage_details));

            // Get recent guardrail blocks
            let block_details = self.persona_manager.get_recent_guardrail_blocks();
            diagnostics.insert("recent_blocks".to_string(), json!(block_details));

            // Get performance metrics
            let performance_metrics = self.persona_manager.get_performance_metrics();
            diagnostics.insert(
                "performance_metrics".to_string(),
                json!(performance_metrics),
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
