use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::modules::telemetry::{
    create_cost_calculator, init_telemetry, telemetry_middleware, LlmCallMetrics, RoutingMetrics,
    TelemetryManager,
};

/// Application state for sharing between handlers
#[derive(Clone)]
pub struct AppState {
    pub telemetry: Arc<TelemetryManager>,
    pub cost_calculator: Arc<crate::modules::telemetry::CostCalculator>,
}

use super::routes::{chat_completions, chat_completions_stream};

/// Create a router with telemetry middleware
pub fn create_router_with_telemetry(
    telemetry: Arc<TelemetryManager>,
    cost_calculator: Arc<crate::modules::telemetry::CostCalculator>,
) -> Router {
    let app_state = AppState {
        telemetry: telemetry.clone(),
        cost_calculator,
    };

    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/chat/completions/stream", post(chat_completions_stream))
        // Add telemetry middleware
        .layer(from_fn_with_state(telemetry, telemetry_middleware))
        .with_state(app_state)
}

/// Initialize telemetry and create a router
pub async fn create_server() -> Result<Router, Box<dyn std::error::Error>> {
    // Initialize telemetry
    let metrics_addr = SocketAddr::from(([0, 0, 0, 0], 9091));
    let telemetry = init_telemetry(
        "intellirouter",
        &std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()),
        env!("CARGO_PKG_VERSION"),
        metrics_addr,
    )?;

    // Create cost calculator
    let cost_calculator = create_cost_calculator();

    // Create the router
    let router = create_router_with_telemetry(telemetry, cost_calculator);

    Ok(router)
}

/// Record LLM call metrics
pub fn record_llm_metrics(
    telemetry: &Arc<TelemetryManager>,
    cost_calculator: &Arc<crate::modules::telemetry::CostCalculator>,
    model_id: &str,
    prompt_tokens: usize,
    completion_tokens: usize,
    latency_ms: u64,
    success: bool,
    error_message: Option<String>,
) {
    let total_tokens = prompt_tokens + completion_tokens;

    // Calculate cost
    let cost = cost_calculator
        .calculate_cost(model_id, prompt_tokens, completion_tokens)
        .unwrap_or(0.0);

    let metrics = LlmCallMetrics {
        model_id: model_id.to_string(),
        prompt_tokens,
        completion_tokens,
        total_tokens,
        latency_ms,
        estimated_cost: cost,
        success,
        error_message,
    };

    telemetry.record_llm_call(metrics);
}

/// Record routing decision metrics
pub fn record_routing_metrics(
    telemetry: &Arc<TelemetryManager>,
    request_id: &str,
    selected_model: &str,
    candidate_count: usize,
    decision_time_ms: u64,
    success: bool,
    error_message: Option<String>,
) {
    let metrics = RoutingMetrics {
        request_id: request_id.to_string(),
        selected_model: selected_model.to_string(),
        candidate_count,
        decision_time_ms,
        success,
        error_message,
    };

    telemetry.record_routing_decision(metrics);
}
