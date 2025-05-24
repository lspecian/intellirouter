//! Health Check Module
//!
//! This module provides standardized health check endpoints for all services
//! in the IntelliRouter system. It includes functionality for basic health checks,
//! readiness checks, and detailed diagnostics.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::error;

// Service-specific health check implementations
pub mod chain_engine;
pub mod persona_layer;
pub mod rag_manager;
pub mod router;

// Re-export service-specific health check functions
pub use chain_engine::create_chain_engine_health_manager;
pub use persona_layer::create_persona_layer_health_manager;
pub use rag_manager::create_rag_manager_health_manager;
pub use router::create_router_health_manager;

/// Health status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Service is healthy and fully operational
    Healthy,
    /// Service is operational but with reduced functionality
    Degraded,
    /// Service is not operational
    Unhealthy,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

/// Connection status for a dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    /// Name of the dependency
    pub name: String,
    /// Status of the connection
    pub status: HealthStatus,
    /// Last successful connection time
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
    /// Error message if the connection is not healthy
    pub error: Option<String>,
    /// Response time in milliseconds for the last check
    pub response_time_ms: Option<u64>,
    /// Additional details about the connection
    pub details: Option<HashMap<String, String>>,
}

/// Resource utilization information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// CPU usage percentage (0-100)
    pub cpu_percent: f32,
    /// Disk usage in bytes
    pub disk_bytes: Option<u64>,
    /// Network usage in bytes
    pub network_bytes: Option<u64>,
    /// Number of active connections
    pub active_connections: Option<u32>,
    /// Additional resource metrics
    pub additional_metrics: HashMap<String, serde_json::Value>,
}

/// Basic health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// Overall health status
    pub status: HealthStatus,
    /// Service name
    pub service: String,
    /// Service version
    pub version: String,
    /// Current timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Service uptime in seconds
    pub uptime_seconds: u64,
}

/// Readiness check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessCheckResponse {
    /// Overall health status
    pub status: HealthStatus,
    /// Service name
    pub service: String,
    /// Service version
    pub version: String,
    /// Current timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Service uptime in seconds
    pub uptime_seconds: u64,
    /// Connection status for dependencies
    pub connections: Vec<ConnectionStatus>,
    /// Resource utilization
    pub resources: ResourceUtilization,
}

/// Diagnostics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsResponse {
    /// Overall health status
    pub status: HealthStatus,
    /// Service name
    pub service: String,
    /// Service version
    pub version: String,
    /// Current timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Service uptime in seconds
    pub uptime_seconds: u64,
    /// Connection status for dependencies
    pub connections: Vec<ConnectionStatus>,
    /// Resource utilization
    pub resources: ResourceUtilization,
    /// Service-specific diagnostics
    pub diagnostics: HashMap<String, serde_json::Value>,
    /// Configuration information
    pub config: HashMap<String, serde_json::Value>,
    /// Recent errors or warnings
    pub recent_issues: Vec<String>,
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Warning threshold for memory usage (percentage)
    pub memory_warning_threshold: f32,
    /// Critical threshold for memory usage (percentage)
    pub memory_critical_threshold: f32,
    /// Warning threshold for CPU usage (percentage)
    pub cpu_warning_threshold: f32,
    /// Critical threshold for CPU usage (percentage)
    pub cpu_critical_threshold: f32,
    /// Timeout for dependency checks in milliseconds
    pub dependency_check_timeout_ms: u64,
    /// Verbosity level for diagnostics (0-3)
    pub diagnostics_verbosity: u8,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            memory_warning_threshold: 80.0,
            memory_critical_threshold: 95.0,
            cpu_warning_threshold: 70.0,
            cpu_critical_threshold: 90.0,
            dependency_check_timeout_ms: 1000,
            diagnostics_verbosity: 1,
        }
    }
}

/// Health check manager
#[derive(Debug, Clone)]
pub struct HealthCheckManager {
    /// Service name
    service_name: String,
    /// Service version
    service_version: String,
    /// Service start time
    start_time: Instant,
    /// Health check configuration
    config: HealthCheckConfig,
    /// Dependency checkers
    dependency_checkers: Vec<Arc<dyn DependencyChecker>>,
    /// Service-specific diagnostics provider
    diagnostics_provider: Option<Arc<dyn DiagnosticsProvider>>,
    /// Recent issues (errors or warnings)
    recent_issues: Arc<RwLock<Vec<String>>>,
    /// Maximum number of recent issues to keep
    max_recent_issues: usize,
}

impl HealthCheckManager {
    /// Create a new health check manager
    pub fn new(
        service_name: impl Into<String>,
        service_version: impl Into<String>,
        config: Option<HealthCheckConfig>,
    ) -> Self {
        Self {
            service_name: service_name.into(),
            service_version: service_version.into(),
            start_time: Instant::now(),
            config: config.unwrap_or_default(),
            dependency_checkers: Vec::new(),
            diagnostics_provider: None,
            recent_issues: Arc::new(RwLock::new(Vec::new())),
            max_recent_issues: 100,
        }
    }

    /// Add a dependency checker
    pub fn add_dependency_checker(&mut self, checker: Arc<dyn DependencyChecker>) {
        self.dependency_checkers.push(checker);
    }

    /// Set the diagnostics provider
    pub fn set_diagnostics_provider(&mut self, provider: Arc<dyn DiagnosticsProvider>) {
        self.diagnostics_provider = Some(provider);
    }

    /// Log an issue
    pub async fn log_issue(&self, issue: impl Into<String>) {
        let issue = issue.into();
        let mut issues = self.recent_issues.write().await;
        if issues.len() >= self.max_recent_issues {
            issues.remove(0);
        }
        issues.push(issue);
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Get current timestamp
    pub fn current_timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }

    /// Get resource utilization
    pub async fn get_resource_utilization(&self) -> ResourceUtilization {
        // Get system memory info
        let sys_info = match sys_info::mem_info() {
            Ok(info) => info,
            Err(e) => {
                error!("Failed to get system memory info: {}", e);
                return ResourceUtilization {
                    memory_bytes: 0,
                    cpu_percent: 0.0,
                    disk_bytes: None,
                    network_bytes: None,
                    active_connections: None,
                    additional_metrics: HashMap::new(),
                };
            }
        };

        // Calculate memory usage
        let memory_bytes = sys_info.total - sys_info.avail;

        // Get CPU load
        let cpu_percent = match sys_info::loadavg() {
            Ok(load) => (load.one * 100.0) as f32,
            Err(e) => {
                error!("Failed to get CPU load: {}", e);
                0.0
            }
        };

        ResourceUtilization {
            memory_bytes: memory_bytes * 1024, // Convert from KB to bytes
            cpu_percent,
            disk_bytes: None,
            network_bytes: None,
            active_connections: None,
            additional_metrics: HashMap::new(),
        }
    }

    /// Check dependencies
    pub async fn check_dependencies(&self) -> Vec<ConnectionStatus> {
        let mut results = Vec::new();

        for checker in &self.dependency_checkers {
            let timeout = Duration::from_millis(self.config.dependency_check_timeout_ms);

            let result = match tokio::time::timeout(timeout, checker.check()).await {
                Ok(Ok(status)) => status,
                Ok(Err(e)) => {
                    let error_msg = format!("Dependency check failed: {}", e);
                    error!("{}", error_msg);

                    ConnectionStatus {
                        name: checker.name().to_string(),
                        status: HealthStatus::Unhealthy,
                        last_success: None,
                        error: Some(error_msg),
                        response_time_ms: None,
                        details: None,
                    }
                }
                Err(_) => {
                    let error_msg = format!(
                        "Dependency check timed out after {} ms",
                        self.config.dependency_check_timeout_ms
                    );
                    error!("{}", error_msg);

                    ConnectionStatus {
                        name: checker.name().to_string(),
                        status: HealthStatus::Unhealthy,
                        last_success: None,
                        error: Some(error_msg),
                        response_time_ms: None,
                        details: None,
                    }
                }
            };

            results.push(result);
        }

        results
    }

    /// Get service-specific diagnostics
    pub async fn get_diagnostics(&self) -> HashMap<String, serde_json::Value> {
        if let Some(provider) = &self.diagnostics_provider {
            match provider
                .get_diagnostics(self.config.diagnostics_verbosity)
                .await
            {
                Ok(diagnostics) => diagnostics,
                Err(e) => {
                    error!("Failed to get diagnostics: {}", e);
                    let mut map = HashMap::new();
                    map.insert(
                        "error".to_string(),
                        serde_json::Value::String(format!("Failed to get diagnostics: {}", e)),
                    );
                    map
                }
            }
        } else {
            HashMap::new()
        }
    }

    /// Get recent issues
    pub async fn get_recent_issues(&self) -> Vec<String> {
        self.recent_issues.read().await.clone()
    }

    /// Get overall health status based on dependencies and resources
    pub async fn get_overall_status(
        &self,
        connections: &[ConnectionStatus],
        resources: &ResourceUtilization,
    ) -> HealthStatus {
        // Check for any unhealthy dependencies
        let has_unhealthy = connections
            .iter()
            .any(|c| c.status == HealthStatus::Unhealthy);
        if has_unhealthy {
            return HealthStatus::Unhealthy;
        }

        // Check for any degraded dependencies
        let has_degraded = connections
            .iter()
            .any(|c| c.status == HealthStatus::Degraded);

        // Check resource utilization
        let memory_percent = resources.memory_bytes as f32
            / (sys_info::mem_info().unwrap().total * 1024) as f32
            * 100.0;
        let cpu_percent = resources.cpu_percent;

        let resources_critical = memory_percent > self.config.memory_critical_threshold
            || cpu_percent > self.config.cpu_critical_threshold;

        let resources_warning = memory_percent > self.config.memory_warning_threshold
            || cpu_percent > self.config.cpu_warning_threshold;

        if resources_critical {
            HealthStatus::Unhealthy
        } else if has_degraded || resources_warning {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Get basic health check response
    pub async fn health_check(&self) -> HealthCheckResponse {
        HealthCheckResponse {
            status: HealthStatus::Healthy, // Basic health check always returns healthy if the service is running
            service: self.service_name.clone(),
            version: self.service_version.clone(),
            timestamp: self.current_timestamp(),
            uptime_seconds: self.uptime_seconds(),
        }
    }

    /// Get readiness check response
    pub async fn readiness_check(&self) -> ReadinessCheckResponse {
        let connections = self.check_dependencies().await;
        let resources = self.get_resource_utilization().await;
        let status = self.get_overall_status(&connections, &resources).await;

        ReadinessCheckResponse {
            status,
            service: self.service_name.clone(),
            version: self.service_version.clone(),
            timestamp: self.current_timestamp(),
            uptime_seconds: self.uptime_seconds(),
            connections,
            resources,
        }
    }

    /// Get diagnostics response
    pub async fn diagnostics(&self) -> DiagnosticsResponse {
        let connections = self.check_dependencies().await;
        let resources = self.get_resource_utilization().await;
        let status = self.get_overall_status(&connections, &resources).await;
        let diagnostics = self.get_diagnostics().await;
        let recent_issues = self.get_recent_issues().await;

        // Get configuration information
        let mut config = HashMap::new();
        config.insert(
            "memory_warning_threshold".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(self.config.memory_warning_threshold as f64).unwrap(),
            ),
        );
        config.insert(
            "memory_critical_threshold".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(self.config.memory_critical_threshold as f64).unwrap(),
            ),
        );
        config.insert(
            "cpu_warning_threshold".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(self.config.cpu_warning_threshold as f64).unwrap(),
            ),
        );
        config.insert(
            "cpu_critical_threshold".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(self.config.cpu_critical_threshold as f64).unwrap(),
            ),
        );
        config.insert(
            "dependency_check_timeout_ms".to_string(),
            serde_json::Value::Number(serde_json::Number::from(
                self.config.dependency_check_timeout_ms,
            )),
        );
        config.insert(
            "diagnostics_verbosity".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.config.diagnostics_verbosity)),
        );

        DiagnosticsResponse {
            status,
            service: self.service_name.clone(),
            version: self.service_version.clone(),
            timestamp: self.current_timestamp(),
            uptime_seconds: self.uptime_seconds(),
            connections,
            resources,
            diagnostics,
            config,
            recent_issues,
        }
    }

    /// Create Axum router with health check endpoints
    pub fn create_router(self) -> Router {
        let shared_state = Arc::new(self);

        Router::new()
            .route("/health", get(health_handler))
            .route("/readiness", get(readiness_handler))
            .route("/diagnostics", get(diagnostics_handler))
            .with_state(shared_state)
    }
}

/// Dependency checker trait
#[async_trait::async_trait]
pub trait DependencyChecker: Send + Sync + std::fmt::Debug {
    /// Get the name of the dependency
    fn name(&self) -> &str;

    /// Check the dependency
    async fn check(&self) -> Result<ConnectionStatus, Box<dyn std::error::Error + Send + Sync>>;
}

/// Diagnostics provider trait
#[async_trait::async_trait]
pub trait DiagnosticsProvider: Send + Sync + std::fmt::Debug {
    /// Get diagnostics information
    async fn get_diagnostics(
        &self,
        verbosity: u8,
    ) -> Result<HashMap<String, serde_json::Value>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Health check handler
async fn health_handler(State(state): State<Arc<HealthCheckManager>>) -> impl IntoResponse {
    let response = state.health_check().await;
    (StatusCode::OK, Json(response))
}

/// Readiness check handler
async fn readiness_handler(State(state): State<Arc<HealthCheckManager>>) -> impl IntoResponse {
    let response = state.readiness_check().await;

    let status_code = match response.status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK, // Still return 200 for degraded but include the status in the response
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(response))
}

/// Diagnostics handler
async fn diagnostics_handler(State(state): State<Arc<HealthCheckManager>>) -> impl IntoResponse {
    let response = state.diagnostics().await;

    let status_code = match response.status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK, // Still return 200 for degraded but include the status in the response
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(response))
}

/// Redis dependency checker
#[derive(Debug)]
pub struct RedisDependencyChecker {
    /// Redis URL
    redis_url: String,
    /// Last successful connection time
    last_success: std::sync::Mutex<Option<chrono::DateTime<chrono::Utc>>>,
}

impl RedisDependencyChecker {
    /// Create a new Redis dependency checker
    pub fn new(redis_url: impl Into<String>) -> Self {
        Self {
            redis_url: redis_url.into(),
            last_success: std::sync::Mutex::new(None),
        }
    }
}

#[async_trait::async_trait]
impl DependencyChecker for RedisDependencyChecker {
    fn name(&self) -> &str {
        "redis"
    }

    async fn check(&self) -> Result<ConnectionStatus, Box<dyn std::error::Error + Send + Sync>> {
        let start = Instant::now();

        // Try to connect to Redis
        let client = redis::Client::open(self.redis_url.clone())?;
        let mut conn = client.get_async_connection().await?;

        // Try a simple PING command
        let pong: String = redis::cmd("PING").query_async(&mut conn).await?;

        if pong != "PONG" {
            return Err(format!("Unexpected response from Redis: {}", pong).into());
        }

        let elapsed = start.elapsed();
        let now = chrono::Utc::now();

        // Update last success time
        *self.last_success.lock().unwrap() = Some(now);

        Ok(ConnectionStatus {
            name: self.name().to_string(),
            status: HealthStatus::Healthy,
            last_success: Some(now),
            error: None,
            response_time_ms: Some(elapsed.as_millis() as u64),
            details: None,
        })
    }
}

/// HTTP dependency checker
#[derive(Debug)]
pub struct HttpDependencyChecker {
    /// Service name
    name: String,
    /// URL to check
    url: String,
    /// Expected status code
    expected_status: u16,
    /// Last successful connection time
    last_success: std::sync::Mutex<Option<chrono::DateTime<chrono::Utc>>>,
}

impl HttpDependencyChecker {
    /// Create a new HTTP dependency checker
    pub fn new(name: impl Into<String>, url: impl Into<String>, expected_status: u16) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            expected_status,
            last_success: std::sync::Mutex::new(None),
        }
    }
}

#[async_trait::async_trait]
impl DependencyChecker for HttpDependencyChecker {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> Result<ConnectionStatus, Box<dyn std::error::Error + Send + Sync>> {
        let start = Instant::now();

        // Make HTTP request
        let client = reqwest::Client::new();
        let response = client.get(&self.url).send().await?;

        let status = response.status();
        let elapsed = start.elapsed();
        let now = chrono::Utc::now();

        if status.as_u16() != self.expected_status {
            return Err(format!(
                "Unexpected status code: {}, expected: {}",
                status.as_u16(),
                self.expected_status
            )
            .into());
        }

        // Update last success time
        *self.last_success.lock().unwrap() = Some(now);

        Ok(ConnectionStatus {
            name: self.name().to_string(),
            status: HealthStatus::Healthy,
            last_success: Some(now),
            error: None,
            response_time_ms: Some(elapsed.as_millis() as u64),
            details: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(HealthStatus::Degraded.to_string(), "degraded");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "unhealthy");
    }

    #[test]
    fn test_health_check_config_default() {
        let config = HealthCheckConfig::default();
        assert_eq!(config.memory_warning_threshold, 80.0);
        assert_eq!(config.memory_critical_threshold, 95.0);
        assert_eq!(config.cpu_warning_threshold, 70.0);
        assert_eq!(config.cpu_critical_threshold, 90.0);
        assert_eq!(config.dependency_check_timeout_ms, 1000);
        assert_eq!(config.diagnostics_verbosity, 1);
    }

    #[tokio::test]
    async fn test_health_check_manager_creation() {
        let manager = HealthCheckManager::new("test-service", "1.0.0", None);

        assert_eq!(manager.service_name, "test-service");
        assert_eq!(manager.service_version, "1.0.0");
        assert!(manager.dependency_checkers.is_empty());
        assert!(manager.diagnostics_provider.is_none());
    }
}
