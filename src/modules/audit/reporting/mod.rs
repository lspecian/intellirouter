//! Reporting Mechanism
//!
//! This module provides visual representation of system topology, test results,
//! performance metrics, and detailed error information.

mod dashboard;
mod exporters;
mod topology;
mod visualization;

pub use dashboard::{DashboardConfig, DashboardServer};
pub use exporters::{ExportFormat, ReportExporter};
pub use topology::{SystemTopology, TopologyEdge, TopologyNode};
pub use visualization::{
    ErrorVisualizer, PerformanceVisualizer, TestResultVisualizer, TopologyVisualizer,
};

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::report::AuditReport;
use super::types::{AuditError, ServiceStatus, ServiceType};

/// Report Generator
///
/// Responsible for generating comprehensive reports in various formats
/// and visualizing system topology, test results, and performance metrics.
#[derive(Debug)]
pub struct ReportGenerator {
    /// Shared audit report
    report: Arc<RwLock<AuditReport>>,
    /// System topology visualizer
    topology_visualizer: TopologyVisualizer,
    /// Test result visualizer
    test_result_visualizer: TestResultVisualizer,
    /// Performance visualizer
    performance_visualizer: PerformanceVisualizer,
    /// Error visualizer
    error_visualizer: ErrorVisualizer,
    /// Report exporter
    exporter: ReportExporter,
    /// Dashboard server
    dashboard_server: Option<DashboardServer>,
}

impl ReportGenerator {
    /// Create a new report generator
    pub fn new(report: Arc<RwLock<AuditReport>>) -> Self {
        Self {
            report: Arc::clone(&report),
            topology_visualizer: TopologyVisualizer::new(),
            test_result_visualizer: TestResultVisualizer::new(),
            performance_visualizer: PerformanceVisualizer::new(),
            error_visualizer: ErrorVisualizer::new(),
            exporter: ReportExporter::new(),
            dashboard_server: None,
        }
    }

    /// Initialize the dashboard server
    pub fn with_dashboard(mut self, config: DashboardConfig) -> Self {
        self.dashboard_server = Some(DashboardServer::new(config, Arc::clone(&self.report)));
        self
    }

    /// Generate a comprehensive report
    pub async fn generate_report(&self) -> Result<(), AuditError> {
        info!("Generating comprehensive report");

        let report = self.report.read().await;

        // Generate system topology visualization
        let topology = self.generate_system_topology(&report).await?;

        // Generate test results visualization
        let test_results = self.test_result_visualizer.visualize(&report).await?;

        // Generate performance metrics visualization
        let performance_metrics = self.performance_visualizer.visualize(&report).await?;

        // Generate error visualization
        let error_details = self.error_visualizer.visualize(&report).await?;

        info!("Report generation completed");

        Ok(())
    }

    /// Generate system topology
    async fn generate_system_topology(
        &self,
        report: &AuditReport,
    ) -> Result<SystemTopology, AuditError> {
        info!("Generating system topology");

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
            self.get_communication_status(report, ServiceType::Router, ServiceType::ChainEngine),
        ));

        // Router -> RagManager
        topology.add_edge(TopologyEdge::new(
            ServiceType::Router,
            ServiceType::RagManager,
            self.get_communication_status(report, ServiceType::Router, ServiceType::RagManager),
        ));

        // Router -> PersonaLayer
        topology.add_edge(TopologyEdge::new(
            ServiceType::Router,
            ServiceType::PersonaLayer,
            self.get_communication_status(report, ServiceType::Router, ServiceType::PersonaLayer),
        ));

        // ChainEngine -> RagManager
        topology.add_edge(TopologyEdge::new(
            ServiceType::ChainEngine,
            ServiceType::RagManager,
            self.get_communication_status(
                report,
                ServiceType::ChainEngine,
                ServiceType::RagManager,
            ),
        ));

        // ChainEngine -> PersonaLayer
        topology.add_edge(TopologyEdge::new(
            ServiceType::ChainEngine,
            ServiceType::PersonaLayer,
            self.get_communication_status(
                report,
                ServiceType::ChainEngine,
                ServiceType::PersonaLayer,
            ),
        ));

        // RagManager -> ChromaDb
        topology.add_edge(TopologyEdge::new(
            ServiceType::RagManager,
            ServiceType::ChromaDb,
            self.get_communication_status(report, ServiceType::RagManager, ServiceType::ChromaDb),
        ));

        // All services -> Redis
        topology.add_edge(TopologyEdge::new(
            ServiceType::Router,
            ServiceType::Redis,
            self.get_communication_status(report, ServiceType::Router, ServiceType::Redis),
        ));

        topology.add_edge(TopologyEdge::new(
            ServiceType::ChainEngine,
            ServiceType::Redis,
            self.get_communication_status(report, ServiceType::ChainEngine, ServiceType::Redis),
        ));

        topology.add_edge(TopologyEdge::new(
            ServiceType::RagManager,
            ServiceType::Redis,
            self.get_communication_status(report, ServiceType::RagManager, ServiceType::Redis),
        ));

        topology.add_edge(TopologyEdge::new(
            ServiceType::PersonaLayer,
            ServiceType::Redis,
            self.get_communication_status(report, ServiceType::PersonaLayer, ServiceType::Redis),
        ));

        // Visualize the topology
        self.topology_visualizer.visualize(&topology).await?;

        Ok(topology)
    }

    /// Get communication status between two services
    fn get_communication_status(
        &self,
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

    /// Export report to a file
    pub async fn export_report(&self, path: &str, format: ExportFormat) -> Result<(), AuditError> {
        info!("Exporting report to {}", path);

        let report = self.report.read().await;
        self.exporter.export(&report, path, format).await?;

        info!("Report exported successfully");

        Ok(())
    }

    /// Start the dashboard server
    pub async fn start_dashboard(&self) -> Result<(), AuditError> {
        if let Some(dashboard) = &self.dashboard_server {
            info!("Starting dashboard server");
            dashboard.start().await?;
            info!("Dashboard server started");
            Ok(())
        } else {
            Err(AuditError::ReportGenerationError(
                "Dashboard server not configured".to_string(),
            ))
        }
    }
}
