//! Web server for the dashboard
//!
//! This module provides a web server for the dashboard.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use axum::{
    extract::{Path as AxumPath, Query, State, WebSocketUpgrade},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

use super::dashboard::{Dashboard, DashboardConfig, TestRun};
use super::exporters::{ExportFormat, Exporter, HtmlExporter, JsonExporter, MarkdownExporter};
use super::renderers::{HtmlRenderer, Renderer, RendererConfig};
use crate::modules::test_harness::metrics::{Metric, MetricCollection};
use crate::modules::test_harness::types::TestHarnessError;

/// Dashboard server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardServerConfig {
    /// Host to bind to
    pub host: String,
    /// Port to bind to
    pub port: u16,
    /// Dashboard title
    pub title: String,
    /// Dashboard description
    pub description: Option<String>,
    /// Static files directory
    pub static_dir: PathBuf,
    /// Data directory
    pub data_dir: PathBuf,
    /// Dashboard refresh interval in seconds
    pub refresh_interval: u64,
    /// Dashboard theme
    pub theme: String,
    /// Dashboard logo
    pub logo: Option<String>,
    /// Dashboard metadata
    pub metadata: HashMap<String, String>,
    /// Enable WebSocket for real-time updates
    pub enable_websocket: bool,
    /// Enable authentication
    pub enable_auth: bool,
    /// Authentication username
    pub auth_username: Option<String>,
    /// Authentication password hash
    pub auth_password_hash: Option<String>,
    /// Enable HTTPS
    pub enable_https: bool,
    /// HTTPS certificate
    pub https_cert: Option<PathBuf>,
    /// HTTPS key
    pub https_key: Option<PathBuf>,
}

impl Default for DashboardServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            title: "Test Dashboard".to_string(),
            description: None,
            static_dir: PathBuf::from("static"),
            data_dir: PathBuf::from("data"),
            refresh_interval: 30,
            theme: "default".to_string(),
            logo: None,
            metadata: HashMap::new(),
            enable_websocket: true,
            enable_auth: false,
            auth_username: None,
            auth_password_hash: None,
            enable_https: false,
            https_cert: None,
            https_key: None,
        }
    }
}

/// Dashboard server state
#[derive(Debug, Clone)]
pub struct DashboardServerState {
    /// Dashboard configuration
    pub config: DashboardServerConfig,
    /// Dashboard
    pub dashboard: Arc<RwLock<Dashboard>>,
    /// Notification sender
    pub notification_tx: broadcast::Sender<String>,
}

/// Dashboard server
#[derive(Debug)]
pub struct DashboardServer {
    /// Server state
    state: DashboardServerState,
}

impl DashboardServer {
    /// Create a new dashboard server
    pub fn new(config: DashboardServerConfig) -> Self {
        // Create the data directory if it doesn't exist
        std::fs::create_dir_all(&config.data_dir).unwrap_or_else(|e| {
            eprintln!("Error creating data directory: {}", e);
        });

        // Create a dashboard
        let dashboard_config = DashboardConfig {
            title: config.title.clone(),
            description: config.description.clone(),
            output_dir: config.data_dir.clone(),
            refresh_interval: Some(config.refresh_interval),
            theme: Some(config.theme.clone()),
            logo: config.logo.clone(),
            metadata: config.metadata.clone(),
        };

        let dashboard = Dashboard::new(dashboard_config);

        // Create a notification channel
        let (notification_tx, _) = broadcast::channel(100);

        Self {
            state: DashboardServerState {
                config,
                dashboard: Arc::new(RwLock::new(dashboard)),
                notification_tx,
            },
        }
    }

    /// Add a test run to the dashboard
    pub async fn add_test_run(&self, test_run: TestRun) -> Result<(), TestHarnessError> {
        // Add the test run to the dashboard
        {
            let mut dashboard = self.state.dashboard.write().unwrap();
            dashboard.add_test_run(test_run.clone());
            dashboard.calculate_metrics();
        }

        // Send a notification
        let notification = serde_json::to_string(&test_run).unwrap_or_else(|e| {
            eprintln!("Error serializing test run: {}", e);
            "{}".to_string()
        });

        let _ = self.state.notification_tx.send(notification);

        Ok(())
    }

    /// Add a metric to the dashboard
    pub async fn add_metric(&self, metric: Metric) -> Result<(), TestHarnessError> {
        // Add the metric to the dashboard
        {
            let mut dashboard = self.state.dashboard.write().unwrap();
            dashboard.add_metric(metric.clone());
        }

        // Send a notification
        let notification = serde_json::to_string(&metric).unwrap_or_else(|e| {
            eprintln!("Error serializing metric: {}", e);
            "{}".to_string()
        });

        let _ = self.state.notification_tx.send(notification);

        Ok(())
    }

    /// Start the dashboard server
    pub async fn start(&self) -> Result<(), TestHarnessError> {
        // Create the router
        let router = self.create_router();

        // Create the address to bind to
        let addr = format!("{}:{}", self.state.config.host, self.state.config.port)
            .parse::<SocketAddr>()
            .map_err(|e| TestHarnessError::ConfigError(format!("Invalid address: {}", e)))?;

        // Start the server
        println!("Starting dashboard server at http://{}", addr);

        if self.state.config.enable_https {
            // HTTPS server
            if let (Some(cert), Some(key)) =
                (&self.state.config.https_cert, &self.state.config.https_key)
            {
                // Load the certificate and key
                let cert = std::fs::read(cert).map_err(|e| {
                    TestHarnessError::ConfigError(format!("Error reading certificate: {}", e))
                })?;
                let key = std::fs::read(key).map_err(|e| {
                    TestHarnessError::ConfigError(format!("Error reading key: {}", e))
                })?;

                // Create the TLS configuration
                let config = rustls::ServerConfig::builder()
                    .with_safe_defaults()
                    .with_no_client_auth()
                    .with_single_cert(
                        rustls_pemfile::certs(&mut cert.as_slice())
                            .map_err(|e| {
                                TestHarnessError::ConfigError(format!(
                                    "Error parsing certificate: {}",
                                    e
                                ))
                            })?
                            .into_iter()
                            .map(rustls::Certificate)
                            .collect(),
                        rustls::PrivateKey(
                            rustls_pemfile::pkcs8_private_keys(&mut key.as_slice())
                                .map_err(|e| {
                                    TestHarnessError::ConfigError(format!(
                                        "Error parsing key: {}",
                                        e
                                    ))
                                })?
                                .into_iter()
                                .next()
                                .ok_or_else(|| {
                                    TestHarnessError::ConfigError(
                                        "No private key found".to_string(),
                                    )
                                })?
                                .secret_pkcs8_der()
                                .to_vec(),
                        ),
                    )
                    .map_err(|e| {
                        TestHarnessError::ConfigError(format!("Error creating TLS config: {}", e))
                    })?;

                // Start the HTTPS server
                axum::Server::bind(&addr)
                    .serve(router.into_make_service())
                    .await
                    .map_err(|e| {
                        TestHarnessError::ServerError(format!("Error starting server: {}", e))
                    })?;
            } else {
                return Err(TestHarnessError::ConfigError(
                    "HTTPS is enabled but certificate or key is missing".to_string(),
                ));
            }
        } else {
            // HTTP server
            axum::Server::bind(&addr)
                .serve(router.into_make_service())
                .await
                .map_err(|e| {
                    TestHarnessError::ServerError(format!("Error starting server: {}", e))
                })?;
        }

        Ok(())
    }

    /// Create the router
    fn create_router(&self) -> Router {
        // Create the state
        let state = self.state.clone();

        // Create the CORS layer
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        // Create the router
        let router = Router::new()
            .route("/", get(index_handler))
            .route("/api/test-runs", get(test_runs_handler))
            .route("/api/test-run/:id", get(test_run_handler))
            .route("/api/metrics", get(metrics_handler))
            .route("/api/trends", get(trends_handler))
            .route("/api/flaky-tests", get(flaky_tests_handler))
            .route("/api/export", get(export_handler))
            .route("/ws", get(websocket_handler))
            .nest_service(
                "/static",
                ServeDir::new(&state.config.static_dir).append_index_html_on_directories(true),
            )
            .layer(TraceLayer::new_for_http())
            .layer(cors)
            .with_state(state);

        router
    }
}

/// Index handler
async fn index_handler(State(state): State<DashboardServerState>) -> impl IntoResponse {
    // Create the HTML renderer
    let renderer = HtmlRenderer::new();

    // Render the dashboard
    let dashboard = state.dashboard.read().unwrap();
    let html = renderer.render(&dashboard).unwrap_or_else(|e| {
        eprintln!("Error rendering dashboard: {}", e);
        format!("<html><body><h1>Error</h1><p>{}</p></body></html>", e)
    });

    // Return the HTML
    Html(html)
}

/// Test runs handler
async fn test_runs_handler(State(state): State<DashboardServerState>) -> impl IntoResponse {
    // Get the test runs
    let dashboard = state.dashboard.read().unwrap();
    let test_runs = dashboard.test_runs().to_vec();

    // Return the test runs as JSON
    Json(test_runs)
}

/// Test run handler
async fn test_run_handler(
    State(state): State<DashboardServerState>,
    AxumPath(id): AxumPath<String>,
) -> impl IntoResponse {
    // Get the test run
    let dashboard = state.dashboard.read().unwrap();
    let test_run = dashboard
        .test_runs()
        .iter()
        .find(|run| run.id == id)
        .cloned();

    // Return the test run as JSON
    match test_run {
        Some(run) => (StatusCode::OK, Json(run)),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Test run not found: {}", id)
            })),
        ),
    }
}

/// Metrics handler
async fn metrics_handler(State(state): State<DashboardServerState>) -> impl IntoResponse {
    // Get the metrics
    let dashboard = state.dashboard.read().unwrap();
    let metrics = dashboard.metrics().clone();

    // Return the metrics as JSON
    Json(metrics)
}

/// Trends handler
async fn trends_handler(State(state): State<DashboardServerState>) -> impl IntoResponse {
    // Get the dashboard
    let dashboard = state.dashboard.read().unwrap();

    // Calculate trends
    let mut pass_rate_trend = Vec::new();
    let mut test_count_trend = Vec::new();
    let mut duration_trend = Vec::new();
    let mut failure_trend = Vec::new();
    let mut flaky_test_trend = Vec::new();

    // Sort test runs by start time
    let mut test_runs = dashboard.test_runs().to_vec();
    test_runs.sort_by(|a, b| a.start_time.cmp(&b.start_time));

    // Calculate trends
    for run in &test_runs {
        let timestamp = run.start_time.timestamp_millis();
        let pass_rate = run.pass_rate();
        let test_count = run.test_count();
        let duration = run.duration;
        let failures = run.failed_count;

        pass_rate_trend.push((timestamp, pass_rate));
        test_count_trend.push((timestamp, test_count));
        duration_trend.push((timestamp, duration));
        failure_trend.push((timestamp, failures));
    }

    // Return the trends as JSON
    Json(serde_json::json!({
        "pass_rate_trend": pass_rate_trend,
        "test_count_trend": test_count_trend,
        "duration_trend": duration_trend,
        "failure_trend": failure_trend,
        "flaky_test_trend": flaky_test_trend
    }))
}

/// Flaky tests handler
async fn flaky_tests_handler(State(state): State<DashboardServerState>) -> impl IntoResponse {
    // Get the dashboard
    let dashboard = state.dashboard.read().unwrap();

    // Calculate flaky tests
    let mut flaky_tests = Vec::new();

    // Group test results by name
    let mut test_results = HashMap::new();
    for run in dashboard.test_runs() {
        for result in &run.results {
            let entry = test_results
                .entry(result.name.clone())
                .or_insert_with(Vec::new);
            entry.push(result);
        }
    }

    // Calculate flakiness
    for (name, results) in test_results {
        if results.len() < 2 {
            continue;
        }

        let total_runs = results.len();
        let passed_runs = results
            .iter()
            .filter(|r| r.outcome == crate::modules::test_harness::types::TestOutcome::Passed)
            .count();
        let failed_runs = results
            .iter()
            .filter(|r| r.outcome == crate::modules::test_harness::types::TestOutcome::Failed)
            .count();

        // A test is considered flaky if it has both passed and failed runs
        if passed_runs > 0 && failed_runs > 0 {
            let flakiness_rate = failed_runs as f64 / total_runs as f64;

            // Get the last failure and success
            let last_failure = results
                .iter()
                .filter(|r| r.outcome == crate::modules::test_harness::types::TestOutcome::Failed)
                .map(|r| r.end_time)
                .max();

            let last_success = results
                .iter()
                .filter(|r| r.outcome == crate::modules::test_harness::types::TestOutcome::Passed)
                .map(|r| r.end_time)
                .max();

            // Get the category and suite
            let category = results.first().map(|r| r.category).unwrap_or_default();
            let suite = results.first().map(|r| r.suite.clone()).unwrap_or_default();

            flaky_tests.push(serde_json::json!({
                "id": name.clone(),
                "name": name,
                "category": category,
                "suite": suite,
                "flakiness_rate": flakiness_rate,
                "total_runs": total_runs,
                "passed_runs": passed_runs,
                "failed_runs": failed_runs,
                "last_failure": last_failure,
                "last_success": last_success
            }));
        }
    }

    // Return the flaky tests as JSON
    Json(flaky_tests)
}

/// Export handler
async fn export_handler(
    State(state): State<DashboardServerState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // Get the format
    let format = params.get("format").map(|f| f.as_str()).unwrap_or("html");

    // Get the dashboard
    let dashboard = state.dashboard.read().unwrap();

    // Create the renderer
    let renderer = match format {
        "html" => Box::new(HtmlRenderer::new()) as Box<dyn Renderer>,
        "json" => {
            // Return the dashboard as JSON
            return (
                StatusCode::OK,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                )],
                serde_json::to_string(&dashboard).unwrap_or_else(|e| {
                    eprintln!("Error serializing dashboard: {}", e);
                    "{}".to_string()
                }),
            )
                .into_response();
        }
        _ => Box::new(HtmlRenderer::new()) as Box<dyn Renderer>,
    };

    // Render the dashboard
    let content = renderer.render(&dashboard).unwrap_or_else(|e| {
        eprintln!("Error rendering dashboard: {}", e);
        format!("<html><body><h1>Error</h1><p>{}</p></body></html>", e)
    });

    // Set the content type
    let content_type = match format {
        "html" => "text/html",
        "json" => "application/json",
        _ => "text/html",
    };

    // Return the content
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, HeaderValue::from_static(content_type))],
        content,
    )
        .into_response()
}

/// WebSocket handler
async fn websocket_handler(
    State(state): State<DashboardServerState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    // Upgrade the connection to a WebSocket
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

/// Handle a WebSocket connection
async fn handle_websocket(socket: axum::extract::ws::WebSocket, state: DashboardServerState) {
    // Split the socket
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to notifications
    let mut rx = state.notification_tx.subscribe();

    // Spawn a task to send notifications
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // Send the notification
            if let Err(e) = sender.send(axum::extract::ws::Message::Text(msg)).await {
                eprintln!("Error sending WebSocket message: {}", e);
                break;
            }
        }
    });

    // Wait for the client to disconnect
    while let Some(Ok(_)) = receiver.next().await {}

    // Abort the send task
    send_task.abort();
}
