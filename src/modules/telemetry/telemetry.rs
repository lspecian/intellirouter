use metrics::{counter, gauge, histogram};
use std::time::Instant;
use tracing::{debug, error, info, warn};

/// Metrics for an LLM API call
#[derive(Debug, Clone)]
pub struct LlmCallMetrics {
    /// ID of the model used
    pub model_id: String,
    /// Number of tokens in the prompt
    pub prompt_tokens: usize,
    /// Number of tokens in the completion
    pub completion_tokens: usize,
    /// Total number of tokens used
    pub total_tokens: usize,
    /// Latency in milliseconds
    pub latency_ms: u64,
    /// Estimated cost in USD
    pub estimated_cost: f64,
    /// Whether the call was successful
    pub success: bool,
    /// Error message if the call failed
    pub error_message: Option<String>,
}

/// Metrics for a routing decision
#[derive(Debug, Clone)]
pub struct RoutingMetrics {
    /// ID of the request
    pub request_id: String,
    /// ID of the selected model
    pub selected_model: String,
    /// Number of candidate models
    pub candidate_count: usize,
    /// Time taken to make the routing decision in milliseconds
    pub decision_time_ms: u64,
    /// Whether the routing was successful
    pub success: bool,
    /// Error message if the routing failed
    pub error_message: Option<String>,
}

/// Manager for telemetry data
#[derive(Debug)]
pub struct TelemetryManager {
    /// Name of the service
    pub service_name: String,
    /// Environment (e.g., "production", "development")
    pub environment: String,
    /// Version of the service
    pub version: String,
}

impl TelemetryManager {
    /// Create a new telemetry manager
    pub fn new(service_name: String, environment: String, version: String) -> Self {
        Self {
            service_name,
            environment,
            version,
        }
    }

    /// Set up logging with the tracing crate
    pub fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
        // Initialize tracing subscriber with JSON formatting for production
        // and pretty printing for development
        let env_filter = tracing_subscriber::EnvFilter::from_default_env();

        if std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) == "production" {
            // JSON formatting for production
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .json()
                .init();
        } else {
            // Pretty printing for development
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .pretty()
                .init();
        }

        Ok(())
    }

    /// Record metrics for an LLM API call
    pub fn record_llm_call(&self, metrics: LlmCallMetrics) {
        // Log the call
        if metrics.success {
            info!(
                model_id = %metrics.model_id,
                prompt_tokens = %metrics.prompt_tokens,
                completion_tokens = %metrics.completion_tokens,
                total_tokens = %metrics.total_tokens,
                latency_ms = %metrics.latency_ms,
                cost = %format!("${:.6}", metrics.estimated_cost),
                "LLM call completed successfully"
            );
        } else {
            error!(
                model_id = %metrics.model_id,
                prompt_tokens = %metrics.prompt_tokens,
                completion_tokens = %metrics.completion_tokens,
                total_tokens = %metrics.total_tokens,
                latency_ms = %metrics.latency_ms,
                cost = %format!("${:.6}", metrics.estimated_cost),
                error = %metrics.error_message.unwrap_or_else(|| "Unknown error".to_string()),
                "LLM call failed"
            );
        }

        // Record metrics
        counter!(
            "intellirouter.llm.calls", 1,
            "model" => metrics.model_id.clone(),
            "success" => metrics.success.to_string(),
            "service" => self.service_name.clone(),
            "env" => self.environment.clone()
        );

        gauge!(
            "intellirouter.llm.tokens.prompt", metrics.prompt_tokens as f64,
            "model" => metrics.model_id.clone(),
            "service" => self.service_name.clone(),
            "env" => self.environment.clone()
        );

        gauge!(
            "intellirouter.llm.tokens.completion", metrics.completion_tokens as f64,
            "model" => metrics.model_id.clone(),
            "service" => self.service_name.clone(),
            "env" => self.environment.clone()
        );

        gauge!(
            "intellirouter.llm.tokens.total", metrics.total_tokens as f64,
            "model" => metrics.model_id.clone(),
            "service" => self.service_name.clone(),
            "env" => self.environment.clone()
        );

        histogram!(
            "intellirouter.llm.latency", metrics.latency_ms as f64,
            "model" => metrics.model_id.clone(),
            "success" => metrics.success.to_string(),
            "service" => self.service_name.clone(),
            "env" => self.environment.clone()
        );

        gauge!(
            "intellirouter.llm.cost", metrics.estimated_cost,
            "model" => metrics.model_id.clone(),
            "service" => self.service_name.clone(),
            "env" => self.environment.clone()
        );
    }

    /// Record metrics for a routing decision
    pub fn record_routing_decision(&self, metrics: RoutingMetrics) {
        // Log the routing decision
        if metrics.success {
            info!(
                request_id = %metrics.request_id,
                selected_model = %metrics.selected_model,
                candidate_count = %metrics.candidate_count,
                decision_time_ms = %metrics.decision_time_ms,
                "Routing decision completed successfully"
            );
        } else {
            error!(
                request_id = %metrics.request_id,
                candidate_count = %metrics.candidate_count,
                decision_time_ms = %metrics.decision_time_ms,
                error = %metrics.error_message.unwrap_or_else(|| "Unknown error".to_string()),
                "Routing decision failed"
            );
        }

        // Record metrics
        counter!(
            "intellirouter.routing.decisions", 1,
            "success" => metrics.success.to_string(),
            "service" => self.service_name.clone(),
            "env" => self.environment.clone()
        );

        if metrics.success {
            counter!(
                "intellirouter.routing.model_selected", 1,
                "model" => metrics.selected_model.clone(),
                "service" => self.service_name.clone(),
                "env" => self.environment.clone()
            );
        }

        gauge!(
            "intellirouter.routing.candidate_count", metrics.candidate_count as f64,
            "service" => self.service_name.clone(),
            "env" => self.environment.clone()
        );

        histogram!(
            "intellirouter.routing.decision_time", metrics.decision_time_ms as f64,
            "success" => metrics.success.to_string(),
            "service" => self.service_name.clone(),
            "env" => self.environment.clone()
        );
    }

    /// Start a timer for measuring request duration
    pub fn start_request_timer(&self) -> Instant {
        Instant::now()
    }

    /// Record metrics for an HTTP request
    pub fn record_request_metrics(&self, path: &str, method: &str, status: u16, timer: Instant) {
        let duration = timer.elapsed();
        let duration_ms = duration.as_millis() as f64;

        // Log the request
        info!(
            method = %method,
            path = %path,
            status = %status,
            duration_ms = %duration_ms,
            "HTTP request completed"
        );

        // Record metrics
        counter!(
            "intellirouter.http.requests", 1,
            "path" => path.to_string(),
            "method" => method.to_string(),
            "status" => status.to_string(),
            "service" => self.service_name.clone(),
            "env" => self.environment.clone()
        );

        histogram!(
            "intellirouter.http.latency", duration_ms,
            "path" => path.to_string(),
            "method" => method.to_string(),
            "status" => status.to_string(),
            "service" => self.service_name.clone(),
            "env" => self.environment.clone()
        );
    }
}
