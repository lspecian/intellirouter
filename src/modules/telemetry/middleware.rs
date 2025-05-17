use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

use super::telemetry::TelemetryManager;

/// Middleware for logging HTTP requests and responses
pub async fn telemetry_middleware<B>(
    State(telemetry): State<Arc<TelemetryManager>>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Generate a request ID
    let request_id = Uuid::new_v4().to_string();

    // Extract path and method
    let path = request.uri().path().to_string();
    let method = request.method().to_string();

    // Start the timer
    let start_time = telemetry.start_request_timer();

    // Log the request
    tracing::info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        "Request started"
    );

    // Process the request
    let response = next.run(request).await;

    // Extract status code
    let status = response.status().as_u16();

    // Record metrics
    telemetry.record_request_metrics(&path, &method, status, start_time);

    Ok(response)
}
