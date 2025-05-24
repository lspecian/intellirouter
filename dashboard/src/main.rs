//! IntelliRouter Unified Project Dashboard
//!
//! This is the main entry point for the IntelliRouter dashboard server.
//! The dashboard provides a unified interface for monitoring the health and quality
//! of the IntelliRouter project, including code quality, performance benchmarking,
//! security audit, and documentation generation.

use chrono::{DateTime, Utc};
use rocket::fs::{relative, FileServer};
use rocket::{get, routes};
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

mod components;
mod data;
mod metrics;
mod utils;

use data::DashboardData;
use metrics::{
    CodeQualityMetrics, DocumentationMetrics, PerformanceMetrics, ProjectHealthMetrics,
    SecurityMetrics,
};

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Dashboard title
    pub title: String,
    /// Dashboard description
    pub description: Option<String>,
    /// Host to bind to
    pub host: String,
    /// Port to bind to
    pub port: u16,
    /// Path to static files
    pub static_dir: PathBuf,
    /// Path to data directory
    pub data_dir: PathBuf,
    /// Refresh interval in seconds
    pub refresh_interval: u64,
    /// Dashboard theme
    pub theme: String,
    /// Path to logo file
    pub logo: Option<PathBuf>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            title: "IntelliRouter Project Dashboard".to_string(),
            description: Some(
                "Unified dashboard for monitoring IntelliRouter project health".to_string(),
            ),
            host: "127.0.0.1".to_string(),
            port: 8080,
            static_dir: PathBuf::from("dashboard/static"),
            data_dir: PathBuf::from("dashboard/data"),
            refresh_interval: 60,
            theme: "default".to_string(),
            logo: None,
            metadata: HashMap::new(),
        }
    }
}

/// Dashboard state
struct DashboardState {
    config: DashboardConfig,
    data: Arc<Mutex<DashboardData>>,
    last_updated: Arc<Mutex<DateTime<Utc>>>,
}

/// Home page route
#[get("/")]
fn index(state: &rocket::State<DashboardState>) -> Template {
    let config = &state.config;
    let data = state.data.lock().unwrap();
    let last_updated = *state.last_updated.lock().unwrap();

    Template::render(
        "index",
        context! {
            title: &config.title,
            description: &config.description,
            refresh_interval: config.refresh_interval,
            theme: &config.theme,
            last_updated: last_updated.to_rfc3339(),
            code_quality: &data.code_quality,
            performance: &data.performance,
            security: &data.security,
            documentation: &data.documentation,
            project_health: &data.project_health,
        },
    )
}

/// Code quality page route
#[get("/code-quality")]
fn code_quality(state: &rocket::State<DashboardState>) -> Template {
    let config = &state.config;
    let data = state.data.lock().unwrap();

    Template::render(
        "code_quality",
        context! {
            title: &config.title,
            description: &config.description,
            refresh_interval: config.refresh_interval,
            theme: &config.theme,
            code_quality: &data.code_quality,
        },
    )
}

/// Performance page route
#[get("/performance")]
fn performance(state: &rocket::State<DashboardState>) -> Template {
    let config = &state.config;
    let data = state.data.lock().unwrap();

    Template::render(
        "performance",
        context! {
            title: &config.title,
            description: &config.description,
            refresh_interval: config.refresh_interval,
            theme: &config.theme,
            performance: &data.performance,
        },
    )
}

/// Security page route
#[get("/security")]
fn security(state: &rocket::State<DashboardState>) -> Template {
    let config = &state.config;
    let data = state.data.lock().unwrap();

    Template::render(
        "security",
        context! {
            title: &config.title,
            description: &config.description,
            refresh_interval: config.refresh_interval,
            theme: &config.theme,
            security: &data.security,
        },
    )
}

/// Documentation page route
#[get("/documentation")]
fn documentation(state: &rocket::State<DashboardState>) -> Template {
    let config = &state.config;
    let data = state.data.lock().unwrap();

    Template::render(
        "documentation",
        context! {
            title: &config.title,
            description: &config.description,
            refresh_interval: config.refresh_interval,
            theme: &config.theme,
            documentation: &data.documentation,
        },
    )
}

/// API route to get all metrics
#[get("/api/metrics")]
fn api_metrics(state: &rocket::State<DashboardState>) -> rocket::serde::json::Json<DashboardData> {
    let data = state.data.lock().unwrap();
    rocket::serde::json::Json(data.clone())
}

/// Background task to update metrics
async fn update_metrics(
    data: Arc<Mutex<DashboardData>>,
    last_updated: Arc<Mutex<DateTime<Utc>>>,
    config: DashboardConfig,
) {
    let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(config.refresh_interval));

    loop {
        interval.tick().await;

        // Update metrics
        let code_quality = metrics::collect_code_quality_metrics(&config.data_dir).await;
        let performance = metrics::collect_performance_metrics(&config.data_dir).await;
        let security = metrics::collect_security_metrics(&config.data_dir).await;
        let documentation = metrics::collect_documentation_metrics(&config.data_dir).await;
        let project_health = metrics::calculate_project_health(
            &code_quality,
            &performance,
            &security,
            &documentation,
        );

        // Update dashboard data
        let mut data_lock = data.lock().unwrap();
        data_lock.code_quality = code_quality;
        data_lock.performance = performance;
        data_lock.security = security;
        data_lock.documentation = documentation;
        data_lock.project_health = project_health;

        // Update last updated timestamp
        let mut last_updated_lock = last_updated.lock().unwrap();
        *last_updated_lock = Utc::now();

        println!("Metrics updated at {}", last_updated_lock);
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // Initialize logger
    env_logger::init();

    // Load configuration
    let config = DashboardConfig::default();

    // Create data directory if it doesn't exist
    std::fs::create_dir_all(&config.data_dir).expect("Failed to create data directory");

    // Initialize dashboard data
    let data = Arc::new(Mutex::new(DashboardData {
        code_quality: CodeQualityMetrics::default(),
        performance: PerformanceMetrics::default(),
        security: SecurityMetrics::default(),
        documentation: DocumentationMetrics::default(),
        project_health: ProjectHealthMetrics::default(),
    }));

    // Initialize last updated timestamp
    let last_updated = Arc::new(Mutex::new(Utc::now()));

    // Start background task to update metrics
    let data_clone = Arc::clone(&data);
    let last_updated_clone = Arc::clone(&last_updated);
    let config_clone = config.clone();
    tokio::spawn(async move {
        update_metrics(data_clone, last_updated_clone, config_clone).await;
    });

    // Start Rocket server
    let dashboard_state = DashboardState {
        config: config.clone(),
        data: Arc::clone(&data),
        last_updated: Arc::clone(&last_updated),
    };

    let _rocket = rocket::build()
        .mount(
            "/",
            routes![
                index,
                code_quality,
                performance,
                security,
                documentation,
                api_metrics
            ],
        )
        .mount("/static", FileServer::from(relative!("static")))
        .manage(dashboard_state)
        .attach(Template::fairing())
        .launch()
        .await?;

    Ok(())
}
