use axum::{
    extract::State,
    http::Method,
    middleware::from_fn_with_state,
    response::{sse::Event, sse::Sse, IntoResponse},
    routing::{get, post, Router},
    Json,
};
use futures::stream::{self, Stream};
use futures::StreamExt;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::StreamExt as TokioStreamExt;
use tracing::{debug, error};

use crate::modules::telemetry::{
    create_cost_calculator, init_telemetry, telemetry_middleware, LlmCallMetrics, RoutingMetrics,
    TelemetryManager,
};

// Use the AppState from server.rs
pub use super::server::AppState;

use super::dto::{ApiError, ChatCompletionRequest, ChatCompletionResponse};
use super::service::ChatCompletionService;
use super::validation;
use crate::modules::router_core::RouterError;

/// Create a router with telemetry middleware
pub fn create_router_with_telemetry(
    telemetry: Arc<TelemetryManager>,
    cost_calculator: Arc<crate::modules::telemetry::CostCalculator>,
) -> Router {
    // Create a router with the actual handler functions from routes.rs
    let router = Router::new()
        .route(
            "/v1/chat/completions",
            post(super::routes::chat_completions),
        )
        .route(
            "/v1/chat/completions/stream",
            post(super::routes::chat_completions_stream),
        )
        // Add telemetry middleware
        .layer(from_fn_with_state(telemetry.clone(), telemetry_middleware));

    // Create a minimal app state with just the telemetry components
    let app_state = AppState {
        provider: super::Provider::OpenAI, // Default provider
        config: super::server::ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 1000,
            request_timeout_secs: 30,
            cors_enabled: false,
            cors_allowed_origins: vec!["*".to_string()],
            redis_url: None,
        },
        shared: std::sync::Arc::new(tokio::sync::Mutex::new(super::server::SharedState::new())),
        telemetry: Some(telemetry),
        cost_calculator: Some(cost_calculator),
    };

    router.with_state(app_state)
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
