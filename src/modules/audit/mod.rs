//! Audit Controller Service
//!
//! This module implements a central audit controller service that orchestrates
//! the testing process, monitors services, and validates communication between
//! system components.
//!
//! The audit controller is responsible for:
//! - Orchestrating the boot sequence of services
//! - Monitoring service health
//! - Validating communication between components
//! - Executing test flows to verify system integration
//! - Collecting metrics on system performance
//! - Providing comprehensive logging of the audit process

mod boot_orchestrator;
mod cli;
mod communication_tests;
mod metrics;
mod report;
mod reporting;
mod service_discovery;
mod test_executor;
mod types;
mod validation;

pub use boot_orchestrator::BootOrchestrator;
pub use cli::run_audit_cli;
pub use metrics::MetricsCollector;
pub use report::{AuditReport, ReportFormat};
pub use reporting::{
    DashboardConfig, DashboardServer, ExportFormat, ReportExporter, ReportGenerator,
    SystemTopology, TopologyEdge, TopologyNode,
};
pub use service_discovery::ServiceDiscovery;
pub use test_executor::TestExecutor;
pub use types::*;
// Re-export ValidationConfig from validation module instead of types
pub use validation::{ValidationConfig, ValidationResult, ValidationWorkflow};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Main Audit Controller that orchestrates the entire audit process
#[derive(Debug)]
pub struct AuditController {
    /// Configuration for the audit controller
    _config: AuditConfig,
    /// Boot sequence orchestrator
    boot_orchestrator: BootOrchestrator,
    /// Service discovery manager
    service_discovery: ServiceDiscovery,
    /// Test executor for running test flows
    test_executor: TestExecutor,
    /// Metrics collector
    metrics_collector: MetricsCollector,
    /// Validation workflow
    validation_workflow: ValidationWorkflow,
    /// Shared audit report
    report: Arc<RwLock<AuditReport>>,
    /// Report generator
    report_generator: Option<ReportGenerator>,
    /// Dashboard server
    dashboard_server: Option<DashboardServer>,
}

impl AuditController {
    /// Create a new audit controller with the given configuration
    pub fn new(config: AuditConfig) -> Self {
        let report = Arc::new(RwLock::new(AuditReport::new()));
        let service_discovery =
            ServiceDiscovery::new(config.discovery_config.clone(), Arc::clone(&report));

        // Create a map of services for the validation workflow
        let mut services = HashMap::new();
        services.insert(
            ServiceType::Router,
            ServiceInfo::new(ServiceType::Router, "router", 8080),
        );
        services.insert(
            ServiceType::ChainEngine,
            ServiceInfo::new(ServiceType::ChainEngine, "orchestrator", 8080),
        );
        services.insert(
            ServiceType::RagManager,
            ServiceInfo::new(ServiceType::RagManager, "rag-injector", 8080),
        );
        services.insert(
            ServiceType::PersonaLayer,
            ServiceInfo::new(ServiceType::PersonaLayer, "summarizer", 8080),
        );
        services.insert(
            ServiceType::Redis,
            ServiceInfo::new(ServiceType::Redis, "redis", 6379),
        );
        services.insert(
            ServiceType::ChromaDb,
            ServiceInfo::new(ServiceType::ChromaDb, "chromadb", 8000),
        );

        // Use the validation config from the AuditConfig if provided, or create a default one
        let validation_config = config
            .validation_config
            .clone() // Clone to avoid partial move
            .unwrap_or_else(|| validation::ValidationConfig::default());

        Self {
            boot_orchestrator: BootOrchestrator::new(
                config.boot_config.clone(),
                Arc::clone(&report),
            ),
            service_discovery: service_discovery.clone(),
            test_executor: TestExecutor::new(config.test_config.clone(), Arc::clone(&report)),
            metrics_collector: MetricsCollector::new(
                config.metrics_config.clone(),
                Arc::clone(&report),
            ),
            validation_workflow: ValidationWorkflow::new(
                validation_config,
                service_discovery,
                Arc::clone(&report),
                services,
            ),
            _config: config,
            report,
            report_generator: None,
            dashboard_server: None,
        }
    }

    /// Run the complete audit process
    pub async fn run_audit(&self) -> Result<AuditReport, AuditError> {
        info!("Starting audit process");

        // Step 1: Boot sequence orchestration
        info!("Step 1: Orchestrating boot sequence");
        self.boot_orchestrator.orchestrate_boot_sequence().await?;

        // Step 2: Service discovery validation
        info!("Step 2: Validating service discovery");
        self.service_discovery.validate_service_discovery().await?;

        // Step 3: Communication tests
        info!("Step 3: Running communication tests");
        self.service_discovery.validate_communication().await?;

        // Step 4: Execute test flows
        info!("Step 4: Executing test flows");
        self.test_executor.execute_test_flows().await?;

        // Step 5: Run comprehensive validation workflow
        info!("Step 5: Running comprehensive validation workflow");
        let validation_results = self.validation_workflow.run_validation().await?;

        // Add validation results to the report
        {
            let mut report = self.report.write().await;
            for result in validation_results {
                if result.success {
                    report.add_success(format!("Validation '{}' passed", result.validation_type));
                } else if let Some(error) = &result.error {
                    report.add_error(format!(
                        "Validation '{}' failed: {}",
                        result.validation_type, error
                    ));
                } else {
                    report.add_error(format!("Validation '{}' failed", result.validation_type));
                }
            }
        }

        // Step 6: Collect and analyze metrics
        info!("Step 6: Collecting and analyzing metrics");
        self.metrics_collector.collect_metrics().await?;

        // Generate final report
        info!("Generating final audit report");
        let report = self.report.read().await.clone();

        info!("Audit process completed successfully");

        // Generate report if report generator is configured
        if let Some(report_generator) = &self.report_generator {
            info!("Generating report");
            report_generator.generate_report().await?;
        }

        Ok(report)
    }

    /// Get a reference to the current audit report
    pub async fn get_report(&self) -> AuditReport {
        self.report.read().await.clone()
    }

    /// Generate and save the audit report to a file
    pub async fn save_report(&self, path: &str, format: ReportFormat) -> Result<(), AuditError> {
        let report = self.report.read().await.clone();
        report.save(path, format)
    }

    /// Configure report generator
    pub fn with_report_generator(mut self, report_generator: ReportGenerator) -> Self {
        self.report_generator = Some(report_generator);
        self
    }

    /// Configure dashboard server
    pub fn with_dashboard(mut self, dashboard_config: DashboardConfig) -> Self {
        let dashboard_server = DashboardServer::new(dashboard_config, Arc::clone(&self.report));
        self.dashboard_server = Some(dashboard_server);
        self
    }

    /// Start the dashboard server
    pub async fn start_dashboard(&self) -> Result<(), AuditError> {
        if let Some(dashboard_server) = &self.dashboard_server {
            dashboard_server.start().await?;
            Ok(())
        } else {
            Err(AuditError::ReportGenerationError(
                "Dashboard server not configured".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_controller_creation() {
        let config = AuditConfig::default();
        let controller = AuditController::new(config);
        assert!(controller._config.enabled);
    }
}
