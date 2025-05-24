//! Continuous Improvement and Monitoring System
//!
//! This module provides a comprehensive system for ongoing reliability enhancement
//! of IntelliRouter. It builds upon the existing test harness, audit, and telemetry
//! systems to provide centralized monitoring, logging, distributed tracing, alerting,
//! and continuous feedback loops.

mod alerting;
mod dashboard;
mod distributed_tracing;
mod feedback;
mod logging;
mod metrics;

pub use alerting::{Alert, AlertConfig, AlertManager, AlertSeverity, AlertingSystem};
pub use dashboard::{
    DashboardConfig, DashboardManager, DashboardPanel, DashboardServer, DashboardView,
};
pub use distributed_tracing::{Span, SpanContext, Tracer, TracingConfig, TracingSystem};
pub use feedback::{
    AnalysisFinding, AnalysisRecommendation, ContinuousImprovementSystem, FeedbackLoop,
    ImprovementSuggestion,
};
pub use logging::{LogConfig, LogFormat, LogLevel, LoggingSystem};
pub use metrics::{Metric, MetricConfig, MetricsCollector, MetricsSystem};

use std::sync::Arc;
use tracing::{error, info};

use crate::modules::audit::AuditController;
use crate::modules::telemetry::TelemetryManager;
#[cfg(feature = "test-harness")]
use crate::modules::test_harness::engine::TestEngine;

/// Main monitoring system that integrates all components
#[derive(Debug)]
pub struct MonitoringSystem {
    /// Configuration for the monitoring system
    config: MonitoringConfig,
    /// Metrics collection system
    metrics_system: Arc<MetricsSystem>,
    /// Logging system
    logging_system: Arc<LoggingSystem>,
    /// Distributed tracing system
    tracing_system: Arc<TracingSystem>,
    /// Alerting system
    alerting_system: Arc<AlertingSystem>,
    /// Dashboard system
    dashboard_system: Arc<DashboardManager>,
    /// Continuous improvement system
    improvement_system: Arc<ContinuousImprovementSystem>,
    /// Telemetry manager
    telemetry_manager: Option<Arc<TelemetryManager>>,
    /// Audit controller
    audit_controller: Option<Arc<AuditController>>,
    /// Test engine
    #[cfg(feature = "test-harness")]
    test_engine: Option<Arc<TestEngine>>,
    #[cfg(not(feature = "test-harness"))]
    test_engine: Option<Arc<()>>, // Placeholder when test-harness is not enabled
}

/// Configuration for the monitoring system
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Enable the monitoring system
    pub enabled: bool,
    /// Metrics configuration
    pub metrics_config: MetricConfig,
    /// Logging configuration
    pub log_config: LogConfig,
    /// Tracing configuration
    pub tracing_config: TracingConfig,
    /// Alerting configuration
    pub alert_config: AlertConfig,
    /// Dashboard configuration
    pub dashboard_config: DashboardConfig,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            metrics_config: MetricConfig::default(),
            log_config: LogConfig::default(),
            tracing_config: TracingConfig::default(),
            alert_config: AlertConfig::default(),
            dashboard_config: DashboardConfig::default(),
        }
    }
}

impl MonitoringSystem {
    /// Create a new monitoring system with the given configuration
    pub fn new(config: MonitoringConfig) -> Self {
        let metrics_system = Arc::new(MetricsSystem::new(config.metrics_config.clone()));
        let logging_system = Arc::new(LoggingSystem::new(config.log_config.clone()));
        let tracing_system = Arc::new(TracingSystem::new(config.tracing_config.clone()));
        let alerting_system = Arc::new(AlertingSystem::new(config.alert_config.clone()));
        let dashboard_system = Arc::new(DashboardManager::new(config.dashboard_config.clone()));
        let improvement_system = Arc::new(ContinuousImprovementSystem::new());

        Self {
            config,
            metrics_system,
            logging_system,
            tracing_system,
            alerting_system,
            dashboard_system,
            improvement_system,
            telemetry_manager: None,
            audit_controller: None,
            test_engine: None,
        }
    }

    /// Initialize the monitoring system
    pub async fn initialize(&mut self) -> Result<(), MonitoringError> {
        info!("Initializing monitoring system");

        // Initialize metrics system
        self.metrics_system.initialize().await?;

        // Initialize logging system
        self.logging_system.initialize().await?;

        // Initialize tracing system
        self.tracing_system.initialize().await?;

        // Initialize alerting system
        self.alerting_system.initialize().await?;

        // Initialize dashboard system
        self.dashboard_system.initialize().await?;

        // Initialize continuous improvement system
        self.improvement_system.initialize().await?;

        info!("Monitoring system initialized successfully");

        Ok(())
    }

    /// Start the monitoring system
    pub async fn start(&self) -> Result<(), MonitoringError> {
        if !self.config.enabled {
            info!("Monitoring system is disabled, not starting");
            return Ok(());
        }

        info!("Starting monitoring system");

        // Start metrics collection
        self.metrics_system.start().await?;

        // Start logging system
        self.logging_system.start().await?;

        // Start tracing system
        self.tracing_system.start().await?;

        // Start alerting system
        self.alerting_system.start().await?;

        // Start dashboard system
        self.dashboard_system.start().await?;

        // Start continuous improvement system
        self.improvement_system.start().await?;

        info!("Monitoring system started successfully");

        Ok(())
    }

    /// Stop the monitoring system
    pub async fn stop(&self) -> Result<(), MonitoringError> {
        info!("Stopping monitoring system");

        // Stop metrics collection
        self.metrics_system.stop().await?;

        // Stop logging system
        self.logging_system.stop().await?;

        // Stop tracing system
        self.tracing_system.stop().await?;

        // Stop alerting system
        self.alerting_system.stop().await?;

        // Stop dashboard system
        self.dashboard_system.stop().await?;

        // Stop continuous improvement system
        self.improvement_system.stop().await?;

        info!("Monitoring system stopped successfully");

        Ok(())
    }

    /// Integrate with telemetry manager
    pub fn with_telemetry(&mut self, telemetry: Arc<TelemetryManager>) -> &mut Self {
        self.telemetry_manager = Some(telemetry);
        self
    }

    /// Integrate with audit controller
    pub fn with_audit_controller(&mut self, audit_controller: Arc<AuditController>) -> &mut Self {
        self.audit_controller = Some(audit_controller);
        self
    }

    /// Integrate with test engine
    #[cfg(feature = "test-harness")]
    pub fn with_test_engine(&mut self, test_engine: Arc<TestEngine>) -> &mut Self {
        self.test_engine = Some(test_engine);
        self
    }

    /// Placeholder for when test-harness feature is not enabled
    #[cfg(not(feature = "test-harness"))]
    pub fn with_test_engine(&mut self, _test_engine: Arc<()>) -> &mut Self {
        // Do nothing when test-harness is not enabled
        self
    }

    /// Get a reference to the metrics system
    pub fn metrics_system(&self) -> Arc<MetricsSystem> {
        Arc::clone(&self.metrics_system)
    }

    /// Get a reference to the logging system
    pub fn logging_system(&self) -> Arc<LoggingSystem> {
        Arc::clone(&self.logging_system)
    }

    /// Get a reference to the tracing system
    pub fn tracing_system(&self) -> Arc<TracingSystem> {
        Arc::clone(&self.tracing_system)
    }

    /// Get a reference to the alerting system
    pub fn alerting_system(&self) -> Arc<AlertingSystem> {
        Arc::clone(&self.alerting_system)
    }

    /// Get a reference to the dashboard system
    pub fn dashboard_system(&self) -> Arc<DashboardManager> {
        Arc::clone(&self.dashboard_system)
    }

    /// Get a reference to the continuous improvement system
    pub fn improvement_system(&self) -> Arc<ContinuousImprovementSystem> {
        Arc::clone(&self.improvement_system)
    }

    /// Run a health check on all monitoring components
    pub async fn health_check(&self) -> Result<MonitoringHealthStatus, MonitoringError> {
        info!("Running monitoring system health check");

        let metrics_status = self.metrics_system.health_check().await?;
        let logging_status = self.logging_system.health_check().await?;
        let tracing_status = self.tracing_system.health_check().await?;
        let alerting_status = self.alerting_system.health_check().await?;
        let dashboard_status = self.dashboard_system.health_check().await?;
        let improvement_status = self.improvement_system.health_check().await?;

        let overall_status = if metrics_status.healthy
            && logging_status.healthy
            && tracing_status.healthy
            && alerting_status.healthy
            && dashboard_status.healthy
            && improvement_status.healthy
        {
            true
        } else {
            false
        };

        let health_status = MonitoringHealthStatus {
            healthy: overall_status,
            metrics_status,
            logging_status,
            tracing_status,
            alerting_status,
            dashboard_status,
            improvement_status,
        };

        info!(
            "Monitoring system health check completed: {}",
            if health_status.healthy {
                "healthy"
            } else {
                "unhealthy"
            }
        );

        Ok(health_status)
    }
}

/// Health status for the monitoring system
#[derive(Debug, Clone)]
pub struct MonitoringHealthStatus {
    /// Overall health status
    pub healthy: bool,
    /// Metrics system health status
    pub metrics_status: ComponentHealthStatus,
    /// Logging system health status
    pub logging_status: ComponentHealthStatus,
    /// Tracing system health status
    pub tracing_status: ComponentHealthStatus,
    /// Alerting system health status
    pub alerting_status: ComponentHealthStatus,
    /// Dashboard system health status
    pub dashboard_status: ComponentHealthStatus,
    /// Continuous improvement system health status
    pub improvement_status: ComponentHealthStatus,
}

/// Health status for a monitoring component
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComponentHealthStatus {
    /// Component name
    pub name: String,
    /// Component health status
    pub healthy: bool,
    /// Component status message
    pub message: Option<String>,
    /// Component status details
    pub details: Option<serde_json::Value>,
}

/// Monitoring system error
#[derive(Debug, thiserror::Error)]
pub enum MonitoringError {
    /// Initialization error
    #[error("Initialization error: {0}")]
    InitializationError(String),
    /// Metrics error
    #[error("Metrics error: {0}")]
    MetricsError(String),
    /// Logging error
    #[error("Logging error: {0}")]
    LoggingError(String),
    /// Tracing error
    #[error("Tracing error: {0}")]
    TracingError(String),
    /// Alerting error
    #[error("Alerting error: {0}")]
    AlertingError(String),
    /// Dashboard error
    #[error("Dashboard error: {0}")]
    DashboardError(String),
    /// Improvement error
    #[error("Improvement error: {0}")]
    ImprovementError(String),
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// JSON error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}
