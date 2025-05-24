//! Dashboard System
//!
//! This module provides functionality for creating and managing dashboards
//! that visualize metrics, logs, traces, and alerts.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{error, info};

use super::{ComponentHealthStatus, MonitoringError};

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Enable dashboard
    pub enabled: bool,
    /// Dashboard host
    pub host: String,
    /// Dashboard port
    pub port: u16,
    /// Dashboard title
    pub title: String,
    /// Dashboard description
    pub description: Option<String>,
    /// Dashboard refresh interval in seconds
    pub refresh_interval: u64,
    /// Dashboard theme
    pub theme: String,
    /// Dashboard logo URL
    pub logo_url: Option<String>,
    /// Dashboard favicon URL
    pub favicon_url: Option<String>,
    /// Dashboard static files directory
    pub static_dir: PathBuf,
    /// Dashboard templates directory
    pub templates_dir: PathBuf,
    /// Dashboard custom CSS URL
    pub custom_css_url: Option<String>,
    /// Dashboard custom JS URL
    pub custom_js_url: Option<String>,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "127.0.0.1".to_string(),
            port: 8080,
            title: "IntelliRouter Dashboard".to_string(),
            description: Some("Monitoring dashboard for IntelliRouter".to_string()),
            refresh_interval: 30,
            theme: "light".to_string(),
            logo_url: None,
            favicon_url: None,
            static_dir: PathBuf::from("dashboard/static"),
            templates_dir: PathBuf::from("dashboard/templates"),
            custom_css_url: None,
            custom_js_url: None,
        }
    }
}

/// Dashboard panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPanel {
    /// Panel ID
    pub id: String,
    /// Panel title
    pub title: String,
    /// Panel description
    pub description: Option<String>,
    /// Panel type
    pub panel_type: String,
    /// Panel data source
    pub data_source: String,
    /// Panel query
    pub query: Option<String>,
    /// Panel width
    pub width: u32,
    /// Panel height
    pub height: u32,
    /// Panel position X
    pub position_x: u32,
    /// Panel position Y
    pub position_y: u32,
    /// Panel options
    pub options: HashMap<String, String>,
    /// Panel data
    pub data: Option<serde_json::Value>,
}

impl DashboardPanel {
    /// Create a new dashboard panel
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        panel_type: impl Into<String>,
        data_source: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            panel_type: panel_type.into(),
            data_source: data_source.into(),
            query: None,
            width: 6,
            height: 6,
            position_x: 0,
            position_y: 0,
            options: HashMap::new(),
            data: None,
        }
    }

    /// Set the panel description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the panel query
    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// Set the panel dimensions
    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set the panel position
    pub fn with_position(mut self, x: u32, y: u32) -> Self {
        self.position_x = x;
        self.position_y = y;
        self
    }

    /// Add an option to the panel
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }

    /// Set the panel data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

/// Dashboard view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardView {
    /// View ID
    pub id: String,
    /// View title
    pub title: String,
    /// View description
    pub description: Option<String>,
    /// View panels
    pub panels: Vec<String>,
    /// View layout
    pub layout: String,
    /// View theme
    pub theme: String,
    /// View options
    pub options: HashMap<String, String>,
}

impl DashboardView {
    /// Create a new dashboard view
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            panels: Vec::new(),
            layout: "grid".to_string(),
            theme: "light".to_string(),
            options: HashMap::new(),
        }
    }

    /// Set the view description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a panel to the view
    pub fn with_panel(mut self, panel_id: impl Into<String>) -> Self {
        self.panels.push(panel_id.into());
        self
    }

    /// Set the view layout
    pub fn with_layout(mut self, layout: impl Into<String>) -> Self {
        self.layout = layout.into();
        self
    }

    /// Set the view theme
    pub fn with_theme(mut self, theme: impl Into<String>) -> Self {
        self.theme = theme.into();
        self
    }

    /// Add an option to the view
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }
}

/// Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    /// Dashboard ID
    pub id: String,
    /// Dashboard title
    pub title: String,
    /// Dashboard description
    pub description: Option<String>,
    /// Dashboard panels
    pub panels: HashMap<String, DashboardPanel>,
    /// Dashboard views
    pub views: HashMap<String, DashboardView>,
    /// Dashboard default view
    pub default_view: String,
    /// Dashboard options
    pub options: HashMap<String, String>,
}

impl Dashboard {
    /// Create a new dashboard
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            id: id_str.clone(),
            title: title.into(),
            description: None,
            panels: HashMap::new(),
            views: HashMap::new(),
            default_view: "main".to_string(),
            options: HashMap::new(),
        }
    }

    /// Set the dashboard description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a panel to the dashboard
    pub fn with_panel(mut self, panel: DashboardPanel) -> Self {
        self.panels.insert(panel.id.clone(), panel);
        self
    }

    /// Add a view to the dashboard
    pub fn with_view(mut self, view: DashboardView) -> Self {
        self.views.insert(view.id.clone(), view);
        self
    }

    /// Set the default view
    pub fn with_default_view(mut self, view_id: impl Into<String>) -> Self {
        self.default_view = view_id.into();
        self
    }

    /// Add an option to the dashboard
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }
}

/// Dashboard server
#[derive(Debug)]
pub struct DashboardServer {
    /// Dashboard configuration
    config: DashboardConfig,
    /// Dashboards
    dashboards: Arc<RwLock<HashMap<String, Dashboard>>>,
    /// Server handle
    #[allow(dead_code)]
    server_handle: Option<tokio::task::JoinHandle<()>>,
}

impl DashboardServer {
    /// Create a new dashboard server
    pub fn new(config: DashboardConfig) -> Self {
        Self {
            config,
            dashboards: Arc::new(RwLock::new(HashMap::new())),
            server_handle: None,
        }
    }

    /// Initialize the dashboard server
    pub async fn initialize(&self) -> Result<(), MonitoringError> {
        if !self.config.enabled {
            info!("Dashboard server is disabled");
            return Ok(());
        }

        info!("Initializing dashboard server");

        // Create static directory if it doesn't exist
        if !self.config.static_dir.exists() {
            std::fs::create_dir_all(&self.config.static_dir).map_err(|e| {
                MonitoringError::DashboardError(format!("Failed to create static directory: {}", e))
            })?;
        }

        // Create templates directory if it doesn't exist
        if !self.config.templates_dir.exists() {
            std::fs::create_dir_all(&self.config.templates_dir).map_err(|e| {
                MonitoringError::DashboardError(format!(
                    "Failed to create templates directory: {}",
                    e
                ))
            })?;
        }

        Ok(())
    }

    /// Start the dashboard server
    pub async fn start(&mut self) -> Result<(), MonitoringError> {
        if !self.config.enabled {
            return Ok(());
        }

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
                MonitoringError::DashboardError(format!("Failed to parse socket address: {}", e))
            })?;

        // Start server
        let dashboards = Arc::clone(&self.dashboards);
        let config = self.config.clone();
        let server_handle = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            if let Err(e) = axum::serve(listener, router).await {
                error!("Dashboard server error: {}", e);
            }
        });

        self.server_handle = Some(server_handle);

        info!("Dashboard server started");

        Ok(())
    }

    /// Stop the dashboard server
    pub async fn stop(&mut self) -> Result<(), MonitoringError> {
        if !self.config.enabled {
            return Ok(());
        }

        info!("Stopping dashboard server");

        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }

        info!("Dashboard server stopped");

        Ok(())
    }

    /// Create router
    fn create_router(&self) -> Router {
        Router::new()
            // API routes
            .route("/api/dashboards", get(Self::get_dashboards))
            .route("/api/dashboards/:id", get(Self::get_dashboard))
            .route(
                "/api/dashboards/:id/panels",
                get(Self::get_dashboard_panels),
            )
            .route("/api/dashboards/:id/views", get(Self::get_dashboard_views))
            .route(
                "/api/dashboards/:id/panels/:panel_id",
                get(Self::get_dashboard_panel),
            )
            .route(
                "/api/dashboards/:id/views/:view_id",
                get(Self::get_dashboard_view),
            )
            // HTML routes
            .route("/", get(Self::index_handler))
            .route("/dashboards", get(Self::dashboards_handler))
            .route("/dashboards/:id", get(Self::dashboard_handler))
            .route(
                "/dashboards/:id/views/:view_id",
                get(Self::dashboard_view_handler),
            )
            // State
            .with_state((Arc::clone(&self.dashboards), self.config.clone()))
    }

    /// Index handler
    async fn index_handler() -> impl IntoResponse {
        Html("<html><body><h1>IntelliRouter Dashboard</h1></body></html>")
    }

    /// Dashboards handler
    async fn dashboards_handler() -> impl IntoResponse {
        Html("<html><body><h1>Dashboards</h1></body></html>")
    }

    /// Dashboard handler
    async fn dashboard_handler(Path(id): Path<String>) -> impl IntoResponse {
        Html(format!(
            "<html><body><h1>Dashboard: {}</h1></body></html>",
            id
        ))
    }

    /// Dashboard view handler
    async fn dashboard_view_handler(
        Path((id, view_id)): Path<(String, String)>,
    ) -> impl IntoResponse {
        Html(format!(
            "<html><body><h1>Dashboard: {}, View: {}</h1></body></html>",
            id, view_id
        ))
    }

    /// Get dashboards
    async fn get_dashboards(
        State((dashboards, _)): State<(Arc<RwLock<HashMap<String, Dashboard>>>, DashboardConfig)>,
    ) -> impl IntoResponse {
        let dashboards = dashboards.read().await;
        Json(dashboards.clone())
    }

    /// Get dashboard
    async fn get_dashboard(
        State((dashboards, _)): State<(Arc<RwLock<HashMap<String, Dashboard>>>, DashboardConfig)>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let dashboards = dashboards.read().await;
        if let Some(dashboard) = dashboards.get(&id) {
            Json(Some(dashboard.clone()))
        } else {
            Json(None)
        }
    }

    /// Get dashboard panels
    async fn get_dashboard_panels(
        State((dashboards, _)): State<(Arc<RwLock<HashMap<String, Dashboard>>>, DashboardConfig)>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let dashboards = dashboards.read().await;
        if let Some(dashboard) = dashboards.get(&id) {
            Json(dashboard.panels.clone())
        } else {
            Json(HashMap::new())
        }
    }

    /// Get dashboard views
    async fn get_dashboard_views(
        State((dashboards, _)): State<(Arc<RwLock<HashMap<String, Dashboard>>>, DashboardConfig)>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let dashboards = dashboards.read().await;
        if let Some(dashboard) = dashboards.get(&id) {
            Json(dashboard.views.clone())
        } else {
            Json(HashMap::new())
        }
    }

    /// Get dashboard panel
    async fn get_dashboard_panel(
        State((dashboards, _)): State<(Arc<RwLock<HashMap<String, Dashboard>>>, DashboardConfig)>,
        Path((id, panel_id)): Path<(String, String)>,
    ) -> impl IntoResponse {
        let dashboards = dashboards.read().await;
        if let Some(dashboard) = dashboards.get(&id) {
            if let Some(panel) = dashboard.panels.get(&panel_id) {
                Json(Some(panel.clone()))
            } else {
                Json(None)
            }
        } else {
            Json(None)
        }
    }

    /// Get dashboard view
    async fn get_dashboard_view(
        State((dashboards, _)): State<(Arc<RwLock<HashMap<String, Dashboard>>>, DashboardConfig)>,
        Path((id, view_id)): Path<(String, String)>,
    ) -> impl IntoResponse {
        let dashboards = dashboards.read().await;
        if let Some(dashboard) = dashboards.get(&id) {
            if let Some(view) = dashboard.views.get(&view_id) {
                Json(Some(view.clone()))
            } else {
                Json(None)
            }
        } else {
            Json(None)
        }
    }

    /// Add a dashboard
    pub async fn add_dashboard(&self, dashboard: Dashboard) -> Result<(), MonitoringError> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut dashboards = self.dashboards.write().await;
        dashboards.insert(dashboard.id.clone(), dashboard);
        Ok(())
    }

    /// Remove a dashboard
    pub async fn remove_dashboard(&self, dashboard_id: &str) -> Result<(), MonitoringError> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut dashboards = self.dashboards.write().await;
        dashboards.remove(dashboard_id);
        Ok(())
    }

    /// Get a dashboard
    pub async fn get_dashboard_by_id(&self, dashboard_id: &str) -> Option<Dashboard> {
        let dashboards = self.dashboards.read().await;
        dashboards.get(dashboard_id).cloned()
    }

    /// Get all dashboards
    pub async fn get_all_dashboards(&self) -> HashMap<String, Dashboard> {
        let dashboards = self.dashboards.read().await;
        dashboards.clone()
    }

    /// Run a health check
    pub async fn health_check(&self) -> Result<ComponentHealthStatus, MonitoringError> {
        let healthy = self.config.enabled;
        let message = if healthy {
            Some("Dashboard server is healthy".to_string())
        } else {
            Some("Dashboard server is disabled".to_string())
        };

        let dashboards = self.dashboards.read().await;
        let details = serde_json::json!({
            "dashboards_count": dashboards.len(),
            "host": self.config.host,
            "port": self.config.port,
            "refresh_interval": self.config.refresh_interval,
            "theme": self.config.theme,
        });

        Ok(ComponentHealthStatus {
            name: "DashboardServer".to_string(),
            healthy,
            message,
            details: Some(details),
        })
    }
}

/// Dashboard manager
#[derive(Debug)]
pub struct DashboardManager {
    /// Dashboard configuration
    config: DashboardConfig,
    /// Dashboard server
    server: Arc<RwLock<DashboardServer>>,
}

impl DashboardManager {
    /// Create a new dashboard manager
    pub fn new(config: DashboardConfig) -> Self {
        let server = Arc::new(RwLock::new(DashboardServer::new(config.clone())));

        Self { config, server }
    }

    /// Initialize the dashboard manager
    pub async fn initialize(&self) -> Result<(), MonitoringError> {
        if !self.config.enabled {
            info!("Dashboard manager is disabled");
            return Ok(());
        }

        info!("Initializing dashboard manager");
        let server = self.server.read().await;
        server.initialize().await?;
        // Additional initialization logic would go here
        Ok(())
    }

    /// Start the dashboard manager
    pub async fn start(&self) -> Result<(), MonitoringError> {
        if !self.config.enabled {
            return Ok(());
        }

        info!("Starting dashboard manager");
        let mut server = self.server.write().await;
        server.start().await?;
        // Additional start logic would go here
        Ok(())
    }

    /// Stop the dashboard manager
    pub async fn stop(&self) -> Result<(), MonitoringError> {
        if !self.config.enabled {
            return Ok(());
        }

        info!("Stopping dashboard manager");
        let mut server = self.server.write().await;
        server.stop().await?;
        // Additional stop logic would go here
        Ok(())
    }

    /// Add a dashboard
    pub async fn add_dashboard(&self, dashboard: Dashboard) -> Result<(), MonitoringError> {
        if !self.config.enabled {
            return Ok(());
        }

        let server = self.server.read().await;
        server.add_dashboard(dashboard).await?;
        Ok(())
    }

    /// Remove a dashboard
    pub async fn remove_dashboard(&self, dashboard_id: &str) -> Result<(), MonitoringError> {
        if !self.config.enabled {
            return Ok(());
        }

        let server = self.server.read().await;
        server.remove_dashboard(dashboard_id).await?;
        Ok(())
    }

    /// Get a dashboard
    pub async fn get_dashboard(&self, dashboard_id: &str) -> Option<Dashboard> {
        let server = self.server.read().await;
        let dashboards = server.dashboards.read().await;
        dashboards.get(dashboard_id).cloned()
    }

    /// Get all dashboards
    pub async fn get_all_dashboards(&self) -> HashMap<String, Dashboard> {
        let server = self.server.read().await;
        server.get_all_dashboards().await
    }

    /// Run a health check
    pub async fn health_check(&self) -> Result<ComponentHealthStatus, MonitoringError> {
        let server = self.server.read().await;
        let server_status = server.health_check().await?;

        let healthy = self.config.enabled && server_status.healthy;
        let message = if healthy {
            Some("Dashboard manager is healthy".to_string())
        } else if !self.config.enabled {
            Some("Dashboard manager is disabled".to_string())
        } else {
            Some("Dashboard manager is unhealthy".to_string())
        };

        let details = serde_json::json!({
            "server_status": server_status,
        });

        Ok(ComponentHealthStatus {
            name: "DashboardManager".to_string(),
            healthy,
            message,
            details: Some(details),
        })
    }
}
