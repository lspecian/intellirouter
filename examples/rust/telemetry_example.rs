use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use intellirouter::modules::telemetry::{
    create_cost_calculator, init_telemetry, telemetry_middleware, LlmCallMetrics, RoutingMetrics,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize telemetry
    let metrics_addr = SocketAddr::from(([127, 0, 0, 1], 9091));
    let telemetry = init_telemetry(
        "intellirouter-example",
        "development",
        env!("CARGO_PKG_VERSION"),
        metrics_addr,
    )?;

    // Create cost calculator
    let cost_calculator = create_cost_calculator();

    // Create the router
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/llm", post(llm_handler))
        .route("/routing", post(routing_handler))
        // Add telemetry middleware
        .layer(from_fn_with_state(telemetry.clone(), telemetry_middleware))
        .with_state(AppState {
            telemetry,
            cost_calculator,
        });

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);
    println!("Metrics available on http://{}/metrics", metrics_addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    telemetry: Arc<intellirouter::modules::telemetry::TelemetryManager>,
    cost_calculator: Arc<intellirouter::modules::telemetry::CostCalculator>,
}

async fn root_handler() -> &'static str {
    "Hello, World!"
}

async fn llm_handler(axum::extract::State(state): axum::extract::State<AppState>) -> &'static str {
    // Simulate an LLM API call
    let start = Instant::now();

    // Simulate processing time
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Record LLM call metrics
    let prompt_tokens = 100;
    let completion_tokens = 50;
    let total_tokens = prompt_tokens + completion_tokens;

    // Calculate cost
    let cost = state
        .cost_calculator
        .calculate_cost("gpt-4", prompt_tokens, completion_tokens)
        .unwrap_or(0.0);

    let metrics = LlmCallMetrics {
        model_id: "gpt-4".to_string(),
        prompt_tokens,
        completion_tokens,
        total_tokens,
        latency_ms: start.elapsed().as_millis() as u64,
        estimated_cost: cost,
        success: true,
        error_message: None,
    };

    state.telemetry.record_llm_call(metrics);

    "LLM call completed"
}

async fn routing_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> &'static str {
    // Simulate a routing decision
    let start = Instant::now();

    // Simulate processing time
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Record routing metrics
    let metrics = RoutingMetrics {
        request_id: uuid::Uuid::new_v4().to_string(),
        selected_model: "gpt-4".to_string(),
        candidate_count: 3,
        decision_time_ms: start.elapsed().as_millis() as u64,
        success: true,
        error_message: None,
    };

    state.telemetry.record_routing_decision(metrics);

    "Routing decision completed"
}
