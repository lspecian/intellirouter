//! LLM Proxy Server
//!
//! This module implements an Axum web server that provides OpenAI-compatible API endpoints
//! for various LLM providers.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::{error, info};

use super::{routes, telemetry_integration, websocket, Provider};
use crate::config::Config;
use crate::modules::telemetry::{
    create_cost_calculator, init_telemetry, CostCalculator, TelemetryManager,
};

/// Configuration for the LLM Proxy server
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Host address to bind to
    pub host: String,
    /// Port to listen on
    pub port: u16,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Enable CORS
    pub cors_enabled: bool,
    /// CORS allowed origins
    pub cors_allowed_origins: Vec<String>,
    /// Redis URL for health checks
    pub redis_url: Option<String>,
}

impl ServerConfig {
    /// Create a new server configuration from the global config
    pub fn from_config(config: &Config) -> Self {
        Self {
            host: config.server.host.to_string(),
            port: config.server.port,
            max_connections: config.server.max_connections,
            request_timeout_secs: config.server.request_timeout_secs,
            cors_enabled: config.server.cors_enabled,
            cors_allowed_origins: config.server.cors_allowed_origins.clone(),
            redis_url: config.memory.redis_url.clone(),
        }
    }

    /// Get the socket address for the server
    pub fn socket_addr(&self) -> Result<SocketAddr, String> {
        let addr = format!("{}:{}", self.host, self.port);
        addr.parse::<SocketAddr>()
            .map_err(|e| format!("Failed to parse socket address: {}", e))
    }
}

/// Application state shared across all routes
#[derive(Debug, Clone)]
pub struct AppState {
    /// The LLM provider to use
    pub provider: Provider,
    /// Server configuration
    pub config: ServerConfig,
    /// Shared application state
    pub shared: Arc<Mutex<SharedState>>,
    /// Telemetry manager
    pub telemetry: Option<Arc<TelemetryManager>>,
    /// Cost calculator
    pub cost_calculator: Option<Arc<CostCalculator>>,
}

/// Shared mutable state
#[derive(Debug)]
pub struct SharedState {
    /// Number of active connections
    pub active_connections: usize,
    /// Whether the server is shutting down
    pub shutting_down: bool,
}

impl SharedState {
    /// Create a new shared state
    pub fn new() -> Self {
        Self {
            active_connections: 0,
            shutting_down: false,
        }
    }
}

/// Start the LLM Proxy server
pub async fn start_server(config: ServerConfig, provider: Provider) -> Result<(), String> {
    info!(
        "Starting LLM Proxy server on {}:{}",
        config.host, config.port
    );

    // Create shared state
    let shared_state = SharedState::new();

    // Initialize telemetry (optional)
    let (telemetry, cost_calculator) = match init_telemetry_components().await {
        Ok((t, c)) => (Some(t), Some(c)),
        Err(e) => {
            error!("Failed to initialize telemetry: {}", e);
            (None, None)
        }
    };

    // Create app state
    let app_state = AppState {
        provider,
        config: config.clone(),
        shared: Arc::new(Mutex::new(shared_state)),
        telemetry,
        cost_calculator,
    };

    // Create health check manager
    let registry_api = crate::modules::model_registry::api::ModelRegistryApi::new();
    let registry = registry_api.registry();
    let health_manager = crate::modules::health::router::create_router_health_manager(
        registry,
        crate::modules::router_core::RouterConfig::default(),
        Some(
            config
                .redis_url
                .clone()
                .unwrap_or_else(|| "redis://localhost:6379".to_string()),
        ),
    );

    let health_router = health_manager.create_router();

    // Create router
    let app = if let (Some(telemetry), Some(cost_calculator)) = (
        app_state.telemetry.clone(),
        app_state.cost_calculator.clone(),
    ) {
        // Create router with telemetry middleware
        telemetry_integration::create_router_with_telemetry(telemetry, cost_calculator)
            .merge(health_router)
    } else {
        // Create router without telemetry
        create_router(app_state.clone()).merge(health_router)
    };

    // Get socket address
    let addr = config.socket_addr()?;

    // Create TCP listener
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind to address: {}", e))?;

    info!("LLM Proxy server listening on {}", addr);

    // Start server
    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}

/// Initialize telemetry components
async fn init_telemetry_components(
) -> Result<(Arc<TelemetryManager>, Arc<CostCalculator>), Box<dyn std::error::Error>> {
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

    Ok((telemetry, cost_calculator))
}

/// Create the Axum router with all routes
pub fn create_router(state: AppState) -> Router {
    // Create a basic router without state first
    let router = Router::new()
        // Legacy health check endpoint (simple version)
        .route("/health/simple", get(health_check))
        // Chat completions endpoints
        .route(
            "/v1/chat/completions",
            post(super::routes::chat_completions),
        )
        .route(
            "/v1/chat/completions/stream",
            post(super::routes::chat_completions_stream),
        );

    // If telemetry is available, create a router with telemetry state
    if let (Some(telemetry), Some(cost_calculator)) =
        (state.telemetry.clone(), state.cost_calculator.clone())
    {
        // Create router with telemetry state
        telemetry_integration::create_router_with_telemetry(telemetry, cost_calculator)
    } else {
        // Return the basic router with state
        router.with_state(state)
    }
}

/// Simple health check endpoint (legacy)
async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "service": "LLM Proxy",
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": chrono::Utc::now().to_rfc3339()
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_server_config_from_config() {
        let config = Config::default();
        let server_config = ServerConfig::from_config(&config);

        assert_eq!(server_config.host, config.server.host.to_string());
        assert_eq!(server_config.port, config.server.port);
        assert_eq!(server_config.max_connections, config.server.max_connections);
        assert_eq!(
            server_config.request_timeout_secs,
            config.server.request_timeout_secs
        );
        assert_eq!(server_config.cors_enabled, config.server.cors_enabled);
        assert_eq!(
            server_config.cors_allowed_origins,
            config.server.cors_allowed_origins
        );
    }

    #[test]
    fn test_server_config_socket_addr() {
        let mut config = ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 1000,
            request_timeout_secs: 30,
            cors_enabled: false,
            cors_allowed_origins: vec!["*".to_string()],
            redis_url: None,
        };

        let addr = config.socket_addr().unwrap();
        assert_eq!(addr.to_string(), "127.0.0.1:8080");

        // Test with IPv6 address
        config.host = "::1".to_string();
        let addr = config.socket_addr().unwrap();
        assert_eq!(addr.to_string(), "[::1]:8080");
    }

    #[test]
    fn test_shared_state_new() {
        let state = SharedState::new();
        assert_eq!(state.active_connections, 0);
        assert_eq!(state.shutting_down, false);
    }

    #[test]
    fn test_app_state_creation() {
        let config = ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 1000,
            request_timeout_secs: 30,
            cors_enabled: false,
            cors_allowed_origins: vec!["*".to_string()],
            redis_url: None,
        };

        let app_state = AppState {
            provider: Provider::OpenAI,
            config: config.clone(),
            shared: Arc::new(Mutex::new(SharedState::new())),
            telemetry: None,
            cost_calculator: None,
        };

        assert_eq!(app_state.provider as u8, Provider::OpenAI as u8);
        assert_eq!(app_state.config.host, "127.0.0.1");
        assert_eq!(app_state.config.port, 8080);
        assert!(app_state.telemetry.is_none());
        assert!(app_state.cost_calculator.is_none());
    }
}
