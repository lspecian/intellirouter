//! Dashboard Server
//!
//! This module provides a web-based dashboard for viewing audit reports.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use tower_http::services::ServeDir;
// Removed incorrect hyper Server import - will be replaced with correct import if needed
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{error, info};

use super::exporters::ExportFormat;
use super::topology::{SystemTopology, TopologyEdge, TopologyNode};
use crate::modules::audit::report::AuditReport;
use crate::modules::audit::types::{AuditError, ServiceStatus, ServiceType};

/// Dashboard configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DashboardConfig {
    /// Dashboard host
    pub host: String,
    /// Dashboard port
    pub port: u16,
    /// Static files directory
    pub static_dir: String,
    /// Enable history
    pub enable_history: bool,
    /// Maximum history entries
    pub max_history: usize,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8090,
            static_dir: "reports/static".to_string(),
            enable_history: true,
            max_history: 10,
        }
    }
}

/// Dashboard state
#[derive(Debug)]
struct DashboardState {
    /// Current report
    report: Arc<RwLock<AuditReport>>,
    /// Report history
    history: Arc<RwLock<Vec<HistoricalReport>>>,
    /// Dashboard configuration
    _config: DashboardConfig,
}

/// Historical report
#[derive(Debug, Clone, Deserialize, Serialize)]
struct HistoricalReport {
    /// Report ID
    id: String,
    /// Report timestamp
    timestamp: DateTime<Utc>,
    /// Overall success status
    success: bool,
    /// Report summary
    summary: String,
}

/// Dashboard server
#[derive(Debug)]
pub struct DashboardServer {
    /// Dashboard configuration
    config: DashboardConfig,
    /// Dashboard state
    state: Arc<DashboardState>,
}

impl DashboardServer {
    /// Create a new dashboard server
    pub fn new(config: DashboardConfig, report: Arc<RwLock<AuditReport>>) -> Self {
        let state = Arc::new(DashboardState {
            report: Arc::clone(&report),
            history: Arc::new(RwLock::new(Vec::new())),
            _config: config.clone(),
        });

        Self { config, state }
    }

    /// Start the dashboard server
    pub async fn start(&self) -> Result<(), AuditError> {
        info!(
            "Starting dashboard server on {}:{}",
            self.config.host, self.config.port
        );

        // Create router
        let router = self.create_router();

        // Create socket address
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .map_err(|e| {
                AuditError::ReportGenerationError(format!("Failed to parse socket address: {}", e))
            })?;

        // Start server
        tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            if let Err(e) = axum::serve::serve(listener, router).await {
                error!("Dashboard server error: {}", e);
            }
        });

        info!("Dashboard server started");

        Ok(())
    }

    /// Create router
    fn create_router(&self) -> Router {
        let state = Arc::clone(&self.state);

        Router::new()
            // API routes
            .route("/api/report", get(Self::get_current_report))
            .route("/api/report/history", get(Self::get_report_history))
            .route("/api/report/history/:id", get(Self::get_historical_report))
            .route("/api/topology", get(Self::get_topology))
            .route("/api/services", get(Self::get_services))
            .route("/api/tests", get(Self::get_tests))
            .route("/api/metrics", get(Self::get_metrics))
            .route("/api/errors", get(Self::get_errors))
            .route("/api/export", post(Self::export_report))
            // HTML routes
            .route("/", get(Self::index_handler))
            .route("/dashboard", get(Self::dashboard_handler))
            .route("/topology", get(Self::topology_handler))
            .route("/services", get(Self::services_handler))
            .route("/tests", get(Self::tests_handler))
            .route("/metrics", get(Self::metrics_handler))
            .route("/errors", get(Self::errors_handler))
            .route("/history", get(Self::history_handler))
            // Static files
            .nest_service("/static", ServeDir::new(&self.config.static_dir))
            // State
            .with_state(state)
    }

    /// Index handler
    async fn index_handler() -> impl IntoResponse {
        Self::dashboard_handler().await
    }

    /// Dashboard handler
    async fn dashboard_handler() -> impl IntoResponse {
        Html(include_str!("../templates/dashboard.html"))
    }

    /// Topology handler
    async fn topology_handler() -> impl IntoResponse {
        Html(include_str!("../templates/topology.html"))
    }

    /// Services handler
    async fn services_handler() -> impl IntoResponse {
        Html(include_str!("../templates/services.html"))
    }

    /// Tests handler
    async fn tests_handler() -> impl IntoResponse {
        Html(include_str!("../templates/tests.html"))
    }

    /// Metrics handler
    async fn metrics_handler() -> impl IntoResponse {
        Html(include_str!("../templates/metrics.html"))
    }

    /// Errors handler
    async fn errors_handler() -> impl IntoResponse {
        Html(include_str!("../templates/errors.html"))
    }

    /// History handler
    async fn history_handler() -> impl IntoResponse {
        Html(include_str!("../templates/history.html"))
    }

    /// Get current report
    async fn get_current_report(
        State(state): State<Arc<DashboardState>>,
    ) -> Result<Json<AuditReport>, StatusCode> {
        let report = state.report.read().await.clone();
        Ok(Json(report))
    }

    /// Get report history
    async fn get_report_history(
        State(state): State<Arc<DashboardState>>,
    ) -> Result<Json<Vec<HistoricalReport>>, StatusCode> {
        let history = state.history.read().await.clone();
        Ok(Json(history))
    }

    /// Get historical report
    async fn get_historical_report(
        State(_state): State<Arc<DashboardState>>,
        Path(_id): Path<String>,
    ) -> Result<Json<Option<AuditReport>>, StatusCode> {
        // In a real implementation, this would fetch the historical report from storage
        // For now, we'll just return None
        Ok(Json(None))
    }

    /// Get system topology
    async fn get_topology(
        State(state): State<Arc<DashboardState>>,
    ) -> Result<Json<SystemTopology>, StatusCode> {
        let report = state.report.read().await.clone();

        let mut topology = SystemTopology::new();

        // Add nodes for each service
        for (service_type, status) in &report.service_statuses {
            topology.add_node(TopologyNode::new(*service_type, *status));
        }

        // Add edges based on service dependencies
        // Router -> ChainEngine
        topology.add_edge(TopologyEdge::new(
            ServiceType::Router,
            ServiceType::ChainEngine,
            Self::get_communication_status(&report, ServiceType::Router, ServiceType::ChainEngine),
        ));

        // Router -> RagManager
        topology.add_edge(TopologyEdge::new(
            ServiceType::Router,
            ServiceType::RagManager,
            Self::get_communication_status(&report, ServiceType::Router, ServiceType::RagManager),
        ));

        // Router -> PersonaLayer
        topology.add_edge(TopologyEdge::new(
            ServiceType::Router,
            ServiceType::PersonaLayer,
            Self::get_communication_status(&report, ServiceType::Router, ServiceType::PersonaLayer),
        ));

        // ChainEngine -> RagManager
        topology.add_edge(TopologyEdge::new(
            ServiceType::ChainEngine,
            ServiceType::RagManager,
            Self::get_communication_status(
                &report,
                ServiceType::ChainEngine,
                ServiceType::RagManager,
            ),
        ));

        // ChainEngine -> PersonaLayer
        topology.add_edge(TopologyEdge::new(
            ServiceType::ChainEngine,
            ServiceType::PersonaLayer,
            Self::get_communication_status(
                &report,
                ServiceType::ChainEngine,
                ServiceType::PersonaLayer,
            ),
        ));

        // RagManager -> ChromaDb
        topology.add_edge(TopologyEdge::new(
            ServiceType::RagManager,
            ServiceType::ChromaDb,
            Self::get_communication_status(&report, ServiceType::RagManager, ServiceType::ChromaDb),
        ));

        // All services -> Redis
        topology.add_edge(TopologyEdge::new(
            ServiceType::Router,
            ServiceType::Redis,
            Self::get_communication_status(&report, ServiceType::Router, ServiceType::Redis),
        ));

        topology.add_edge(TopologyEdge::new(
            ServiceType::ChainEngine,
            ServiceType::Redis,
            Self::get_communication_status(&report, ServiceType::ChainEngine, ServiceType::Redis),
        ));

        topology.add_edge(TopologyEdge::new(
            ServiceType::RagManager,
            ServiceType::Redis,
            Self::get_communication_status(&report, ServiceType::RagManager, ServiceType::Redis),
        ));

        topology.add_edge(TopologyEdge::new(
            ServiceType::PersonaLayer,
            ServiceType::Redis,
            Self::get_communication_status(&report, ServiceType::PersonaLayer, ServiceType::Redis),
        ));

        Ok(Json(topology))
    }

    /// Get communication status between two services
    fn get_communication_status(
        report: &AuditReport,
        source: ServiceType,
        target: ServiceType,
    ) -> bool {
        for test in &report.communication_tests {
            if test.source == source && test.target == target {
                return test.success;
            }
        }
        false
    }

    /// Get services
    async fn get_services(
        State(state): State<Arc<DashboardState>>,
    ) -> Result<Json<HashMap<ServiceType, ServiceStatus>>, StatusCode> {
        let report = state.report.read().await;
        Ok(Json(report.service_statuses.clone()))
    }

    /// Get tests
    async fn get_tests(
        State(state): State<Arc<DashboardState>>,
    ) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
        let report = state.report.read().await.clone();

        // Convert test results to JSON
        let test_results = report
            .test_results
            .iter()
            .map(|result| {
                serde_json::json!({
                    "test_flow": result.test_flow.to_string(),
                    "success": result.success,
                    "error": result.error,
                    "duration_ms": result.duration_ms,
                    "timestamp": result.timestamp,
                    "details": result.details,
                })
            })
            .collect();

        Ok(Json(test_results))
    }

    /// Get metrics
    async fn get_metrics(
        State(state): State<Arc<DashboardState>>,
    ) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
        let report = state.report.read().await.clone();

        // Convert metrics to JSON
        let metrics = report
            .metrics
            .iter()
            .map(|metric| {
                serde_json::json!({
                    "metric_type": metric.metric_type.to_string(),
                    "service": metric.service.to_string(),
                    "value": metric.value,
                    "timestamp": metric.timestamp,
                })
            })
            .collect();

        Ok(Json(metrics))
    }

    /// Get errors
    async fn get_errors(
        State(state): State<Arc<DashboardState>>,
    ) -> Result<Json<Vec<String>>, StatusCode> {
        let report = state.report.read().await.clone();
        Ok(Json(report.errors))
    }

    /// Export report
    async fn export_report(
        State(_state): State<Arc<DashboardState>>,
        Json(payload): Json<ExportRequest>,
    ) -> Result<Json<ExportResponse>, StatusCode> {
        // In a real implementation, this would export the report to a file
        // and return the file path or download URL

        Ok(Json(ExportResponse {
            success: true,
            message: format!("Report exported in {:?} format", payload.format),
            download_url: Some(format!("/api/download/{}", payload.filename)),
        }))
    }
}

/// Export request
#[derive(Debug, Deserialize)]
struct ExportRequest {
    /// Export format
    format: ExportFormat,
    /// Filename
    filename: String,
}

/// Export response
#[derive(Debug, Serialize)]
struct ExportResponse {
    /// Success status
    success: bool,
    /// Message
    message: String,
    /// Download URL
    download_url: Option<String>,
}
