//! Types for the Audit Controller Service
//!
//! This module defines the core data structures used by the audit controller.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// Import ValidationConfig from validation module
use super::validation::ValidationConfig;

// ValidationConfig moved to validation/config.rs to resolve circular dependency
// Re-exported through validation/mod.rs

/// Configuration for the Audit Controller
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditConfig {
    /// Whether the audit controller is enabled
    pub enabled: bool,
    /// Boot sequence orchestration configuration
    pub boot_config: BootConfig,
    /// Service discovery configuration
    pub discovery_config: DiscoveryConfig,
    /// Test execution configuration
    pub test_config: TestConfig,
    /// Metrics collection configuration
    pub metrics_config: MetricsConfig,
    /// Validation workflow configuration
    pub validation_config: Option<ValidationConfig>,
    /// Log level for the audit controller
    pub log_level: LogLevel,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            boot_config: BootConfig::default(),
            discovery_config: DiscoveryConfig::default(),
            test_config: TestConfig::default(),
            metrics_config: MetricsConfig::default(),
            validation_config: None,
            log_level: LogLevel::Info,
        }
    }
}

/// Boot sequence orchestration configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BootConfig {
    /// Maximum time to wait for all services to start
    pub max_boot_time_secs: u64,
    /// Time to wait between service startup checks
    pub check_interval_ms: u64,
    /// Whether to fail if any service fails to start
    pub fail_fast: bool,
    /// Service boot order
    pub service_order: Vec<ServiceType>,
    /// Timeout for individual service startup
    pub service_timeout_secs: u64,
}

impl Default for BootConfig {
    fn default() -> Self {
        Self {
            max_boot_time_secs: 300, // 5 minutes
            check_interval_ms: 1000, // 1 second
            fail_fast: true,
            service_order: vec![
                ServiceType::Redis,
                ServiceType::ChromaDb,
                ServiceType::Router,
                ServiceType::ChainEngine,
                ServiceType::RagManager,
                ServiceType::PersonaLayer,
            ],
            service_timeout_secs: 60, // 1 minute
        }
    }
}

/// Service discovery configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiscoveryConfig {
    /// Maximum time to wait for service discovery
    pub discovery_timeout_secs: u64,
    /// Whether to validate all services can discover each other
    pub validate_all_connections: bool,
    /// Timeout for individual connection checks
    pub connection_timeout_ms: u64,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            discovery_timeout_secs: 60, // 1 minute
            validate_all_connections: true,
            connection_timeout_ms: 5000, // 5 seconds
        }
    }
}

/// Test execution configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestConfig {
    /// Test flows to execute
    pub test_flows: Vec<TestFlow>,
    /// Maximum time to wait for test execution
    pub test_timeout_secs: u64,
    /// Whether to fail if any test fails
    pub fail_fast: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            test_flows: vec![TestFlow::BasicChainExecution],
            test_timeout_secs: 120, // 2 minutes
            fail_fast: true,
        }
    }
}

/// Metrics collection configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricsConfig {
    /// Whether to collect metrics
    pub collect_metrics: bool,
    /// Metrics collection interval in milliseconds
    pub collection_interval_ms: u64,
    /// Duration to collect metrics for
    pub collection_duration_secs: u64,
    /// Types of metrics to collect
    pub metric_types: Vec<MetricType>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            collect_metrics: true,
            collection_interval_ms: 1000, // 1 second
            collection_duration_secs: 60, // 1 minute
            metric_types: vec![
                MetricType::Latency,
                MetricType::Throughput,
                MetricType::ErrorRate,
                MetricType::ResourceUsage,
            ],
        }
    }
}

/// Service type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ServiceType {
    /// Router service
    Router,
    /// Chain Engine service
    ChainEngine,
    /// RAG Manager service
    RagManager,
    /// Persona Layer service
    PersonaLayer,
    /// Redis service
    Redis,
    /// ChromaDB service
    ChromaDb,
    /// Model Registry service
    ModelRegistry,
    /// Memory service
    Memory,
    /// Orchestrator service
    Orchestrator,
}

impl fmt::Display for ServiceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceType::Router => write!(f, "router"),
            ServiceType::ChainEngine => write!(f, "orchestrator"),
            ServiceType::RagManager => write!(f, "rag-injector"),
            ServiceType::PersonaLayer => write!(f, "summarizer"),
            ServiceType::Redis => write!(f, "redis"),
            ServiceType::ChromaDb => write!(f, "chromadb"),
            ServiceType::ModelRegistry => write!(f, "model-registry"),
            ServiceType::Memory => write!(f, "memory"),
            ServiceType::Orchestrator => write!(f, "orchestrator-service"),
        }
    }
}

/// Service status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ServiceStatus {
    /// Service is not started
    NotStarted,
    /// Service is starting
    Starting,
    /// Service is running
    Running,
    /// Service is active and healthy
    Active,
    /// Service is inactive
    Inactive,
    /// Service is running but in a degraded state
    Degraded,
    /// Service failed to start
    Failed,
    /// Service is shutting down
    ShuttingDown,
    /// Service is stopped
    Stopped,
}

impl fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceStatus::NotStarted => write!(f, "Not Started"),
            ServiceStatus::Starting => write!(f, "Starting"),
            ServiceStatus::Running => write!(f, "Running"),
            ServiceStatus::Active => write!(f, "Active"),
            ServiceStatus::Inactive => write!(f, "Inactive"),
            ServiceStatus::Degraded => write!(f, "Degraded"),
            ServiceStatus::Failed => write!(f, "Failed"),
            ServiceStatus::ShuttingDown => write!(f, "Shutting Down"),
            ServiceStatus::Stopped => write!(f, "Stopped"),
        }
    }
}

/// Test flow type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum TestFlow {
    /// Basic chain execution test
    BasicChainExecution,
    /// RAG integration test
    RagIntegration,
    /// Persona layer integration test
    PersonaLayerIntegration,
    /// End-to-end flow test
    EndToEndFlow,
}

impl fmt::Display for TestFlow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestFlow::BasicChainExecution => write!(f, "Basic Chain Execution"),
            TestFlow::RagIntegration => write!(f, "RAG Integration"),
            TestFlow::PersonaLayerIntegration => write!(f, "Persona Layer Integration"),
            TestFlow::EndToEndFlow => write!(f, "End-to-End Flow"),
        }
    }
}

/// Metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Hash)]
pub enum MetricType {
    /// Latency metrics
    Latency,
    /// Throughput metrics
    Throughput,
    /// Error rate metrics
    ErrorRate,
    /// Resource usage metrics
    ResourceUsage,
}

impl fmt::Display for MetricType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricType::Latency => write!(f, "Latency"),
            MetricType::Throughput => write!(f, "Throughput"),
            MetricType::ErrorRate => write!(f, "Error Rate"),
            MetricType::ResourceUsage => write!(f, "Resource Usage"),
        }
    }
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum LogLevel {
    /// Debug log level
    Debug,
    /// Info log level
    Info,
    /// Warning log level
    Warn,
    /// Error log level
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

/// Service information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceInfo {
    /// Service type
    pub service_type: ServiceType,
    /// Service name
    pub name: String,
    /// Service status
    pub status: ServiceStatus,
    /// Service host
    pub host: String,
    /// Service port
    pub port: u16,
    /// Main service endpoint
    pub endpoint: String,
    /// Service health endpoint
    pub health_endpoint: String,
    /// Service readiness endpoint
    pub readiness_endpoint: String,
    /// Service diagnostics endpoint
    pub diagnostics_endpoint: String,
    /// Service start time
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Service ready time
    pub ready_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Service dependencies
    pub dependencies: Vec<ServiceType>,
}

impl ServiceInfo {
    /// Create a new service info
    pub fn new(service_type: ServiceType, host: &str, port: u16) -> Self {
        let base_url = format!("http://{}:{}", host, port);
        // Generate a name based on service type
        let name = format!("{}", service_type);

        Self {
            service_type,
            name,
            status: ServiceStatus::NotStarted,
            host: host.to_string(),
            port,
            endpoint: base_url.clone(),
            health_endpoint: format!("{}/health", base_url),
            readiness_endpoint: format!("{}/readiness", base_url),
            diagnostics_endpoint: format!("{}/diagnostics", base_url),
            start_time: None,
            ready_time: None,
            dependencies: match service_type {
                ServiceType::Router => vec![ServiceType::Redis],
                ServiceType::ChainEngine => vec![ServiceType::Redis, ServiceType::Router],
                ServiceType::RagManager => vec![
                    ServiceType::Redis,
                    ServiceType::Router,
                    ServiceType::ChromaDb,
                ],
                ServiceType::PersonaLayer => vec![ServiceType::Redis, ServiceType::Router],
                ServiceType::Redis => vec![],
                ServiceType::ChromaDb => vec![],
                ServiceType::ModelRegistry => vec![ServiceType::Redis],
                ServiceType::Memory => vec![ServiceType::Redis],
                ServiceType::Orchestrator => vec![ServiceType::Redis, ServiceType::Router],
            },
        }
    }
}

/// Communication test result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommunicationTestResult {
    /// Source service
    pub source: ServiceType,
    /// Target service
    pub target: ServiceType,
    /// Test status
    pub success: bool,
    /// Error message if the test failed
    pub error: Option<String>,
    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,
    /// Test timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Test result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestResult {
    /// Test flow
    pub test_flow: TestFlow,
    /// Test status
    pub success: bool,
    /// Error message if the test failed
    pub error: Option<String>,
    /// Test duration in milliseconds
    pub duration_ms: u64,
    /// Test timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Test details
    pub details: HashMap<String, serde_json::Value>,
}

/// Metric data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricDataPoint {
    /// Metric type
    pub metric_type: MetricType,
    /// Service type
    pub service: ServiceType,
    /// Metric value
    pub value: f64,
    /// Metric timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Audit error
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    /// Boot sequence error
    #[error("Boot sequence error: {0}")]
    BootSequenceError(String),

    /// Service discovery error
    #[error("Service discovery error: {0}")]
    ServiceDiscoveryError(String),

    /// Communication test error
    #[error("Communication test error: {0}")]
    CommunicationTestError(String),

    /// Test execution error
    #[error("Test execution error: {0}")]
    TestExecutionError(String),

    /// Metrics collection error
    #[error("Metrics collection error: {0}")]
    MetricsCollectionError(String),

    /// Report generation error
    #[error("Report generation error: {0}")]
    ReportGenerationError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// HTTP error
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Timeout error
    #[error("Timeout error: {0}")]
    TimeoutError(String),
}
