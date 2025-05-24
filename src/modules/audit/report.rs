//! Audit Report
//!
//! This module is responsible for generating and managing the audit report.

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;

use super::types::{
    AuditError, CommunicationTestResult, MetricDataPoint, MetricType, ServiceStatus, ServiceType,
    TestResult,
};

/// Report format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ReportFormat {
    /// JSON format
    Json,
    /// Markdown format
    Markdown,
    /// HTML format
    Html,
}

/// Metric analysis
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricAnalysis {
    /// Service type
    pub service: ServiceType,
    /// Metric type
    pub metric_type: MetricType,
    /// Average value
    pub average_value: f64,
    /// Description
    pub description: String,
}

/// Audit report
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditReport {
    /// Report timestamp
    pub timestamp: DateTime<Utc>,
    /// Overall success status
    pub success: bool,
    /// Service statuses
    pub service_statuses: HashMap<ServiceType, ServiceStatus>,
    /// Success messages
    pub successes: Vec<String>,
    /// Warning messages
    pub warnings: Vec<String>,
    /// Error messages
    pub errors: Vec<String>,
    /// Test results
    pub test_results: Vec<TestResult>,
    /// Communication test results
    pub communication_tests: Vec<CommunicationTestResult>,
    /// Metrics
    pub metrics: Vec<MetricDataPoint>,
    /// Metric analyses
    pub metric_analyses: Vec<MetricAnalysis>,
}

impl AuditReport {
    /// Create a new audit report
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
            success: true,
            service_statuses: HashMap::new(),
            successes: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
            test_results: Vec::new(),
            communication_tests: Vec::new(),
            metrics: Vec::new(),
            metric_analyses: Vec::new(),
        }
    }

    /// Add a service status
    pub fn add_service_status(&mut self, service: ServiceType, status: ServiceStatus) {
        self.service_statuses.insert(service, status);

        // If any service is failed, the overall status is failed
        if status == ServiceStatus::Failed {
            self.success = false;
        }
    }

    /// Add a success message
    pub fn add_success(&mut self, message: impl Into<String>) {
        self.successes.push(message.into());
    }

    /// Add a warning message
    pub fn add_warning(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }

    /// Add an error message
    pub fn add_error(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
        self.success = false;
    }

    /// Add a test result
    pub fn add_test_result(&mut self, result: TestResult) {
        if !result.success {
            self.success = false;
        }
        self.test_results.push(result);
    }

    /// Add a communication test result
    pub fn add_communication_test(&mut self, result: CommunicationTestResult) {
        if !result.success {
            self.success = false;
        }
        self.communication_tests.push(result);
    }

    /// Add a metric data point
    pub fn add_metric(&mut self, metric: MetricDataPoint) {
        self.metrics.push(metric);
    }

    /// Add a metric analysis
    pub fn add_metric_analysis(
        &mut self,
        service: ServiceType,
        metric_type: MetricType,
        average_value: f64,
        description: impl Into<String>,
    ) {
        self.metric_analyses.push(MetricAnalysis {
            service,
            metric_type,
            average_value,
            description: description.into(),
        });
    }

    /// Add a section to the report
    pub fn add_section(&mut self, name: impl Into<String>, value: serde_json::Value) {
        // For now, we'll just add a success message with the section name and value
        // In a real implementation, this would add a section to the report
        let message = format!("Section {}: {}", name.into(), value);
        self.add_success(message);
    }

    /// Get all metrics
    pub fn get_metrics(&self) -> &[MetricDataPoint] {
        &self.metrics
    }

    /// Get the number of test results
    pub fn get_test_count(&self) -> usize {
        self.test_results.len()
    }

    /// Get the number of errors
    pub fn get_error_count(&self) -> usize {
        self.errors.len()
    }

    /// Check if the report has errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if the report contains a test with the given name
    pub fn contains_test(&self, test_name: &str) -> bool {
        self.test_results
            .iter()
            .any(|test| test.test_flow.to_string() == test_name)
    }

    /// Save the report to a file
    pub fn save(&self, path: &str, format: ReportFormat) -> Result<(), AuditError> {
        let path = Path::new(path);

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AuditError::ReportGenerationError(format!("Failed to create directories: {}", e))
            })?;
        }

        // Generate the report content
        let content = match format {
            ReportFormat::Json => self.to_json()?,
            ReportFormat::Markdown => self.to_markdown(),
            ReportFormat::Html => self.to_html(),
        };

        // Write the content to the file
        let mut file = File::create(path).map_err(|e| {
            AuditError::ReportGenerationError(format!("Failed to create file: {}", e))
        })?;

        file.write_all(content.as_bytes()).map_err(|e| {
            AuditError::ReportGenerationError(format!("Failed to write to file: {}", e))
        })?;

        info!("Report saved to {}", path.display());

        Ok(())
    }

    /// Convert the report to JSON
    fn to_json(&self) -> Result<String, AuditError> {
        serde_json::to_string_pretty(self).map_err(|e| {
            AuditError::ReportGenerationError(format!("Failed to serialize report to JSON: {}", e))
        })
    }

    /// Convert the report to Markdown
    fn to_markdown(&self) -> String {
        let mut markdown = String::new();

        // Title
        markdown.push_str("# IntelliRouter Audit Report\n\n");

        // Summary
        markdown.push_str("## Summary\n\n");
        markdown.push_str(&format!("- **Timestamp**: {}\n", self.timestamp));
        markdown.push_str(&format!(
            "- **Status**: {}\n",
            if self.success {
                "✅ Success"
            } else {
                "❌ Failure"
            }
        ));
        markdown.push_str(&format!(
            "- **Services**: {}\n",
            self.service_statuses.len()
        ));
        markdown.push_str(&format!("- **Tests**: {}\n", self.test_results.len()));
        markdown.push_str(&format!(
            "- **Communication Tests**: {}\n",
            self.communication_tests.len()
        ));
        markdown.push_str(&format!("- **Metrics**: {}\n", self.metrics.len()));
        markdown.push_str("\n");

        // Service Statuses
        markdown.push_str("## Service Statuses\n\n");
        markdown.push_str("| Service | Status |\n");
        markdown.push_str("|---------|--------|\n");

        for (service, status) in &self.service_statuses {
            let status_icon = match status {
                ServiceStatus::Running => "✅ Running",
                ServiceStatus::Failed => "❌ Failed",
                ServiceStatus::NotStarted => "⏳ Not Started",
                ServiceStatus::Starting => "🔄 Starting",
                ServiceStatus::ShuttingDown => "🔄 Shutting Down",
                ServiceStatus::Stopped => "⏹️ Stopped",
                ServiceStatus::Active => "✅ Active",
                ServiceStatus::Inactive => "⏹️ Inactive",
                ServiceStatus::Degraded => "⚠️ Degraded",
            };

            markdown.push_str(&format!("| {} | {} |\n", service, status_icon));
        }

        markdown.push_str("\n");

        // Successes
        if !self.successes.is_empty() {
            markdown.push_str("## Successes\n\n");

            for success in &self.successes {
                markdown.push_str(&format!("- ✅ {}\n", success));
            }

            markdown.push_str("\n");
        }

        // Warnings
        if !self.warnings.is_empty() {
            markdown.push_str("## Warnings\n\n");

            for warning in &self.warnings {
                markdown.push_str(&format!("- ⚠️ {}\n", warning));
            }

            markdown.push_str("\n");
        }

        // Errors
        if !self.errors.is_empty() {
            markdown.push_str("## Errors\n\n");

            for error in &self.errors {
                markdown.push_str(&format!("- ❌ {}\n", error));
            }

            markdown.push_str("\n");
        }

        // Test Results
        if !self.test_results.is_empty() {
            markdown.push_str("## Test Results\n\n");
            markdown.push_str("| Test | Status | Duration (ms) | Timestamp |\n");
            markdown.push_str("|------|--------|--------------|----------|\n");

            for result in &self.test_results {
                let status_icon = if result.success { "✅" } else { "❌" };

                markdown.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    result.test_flow, status_icon, result.duration_ms, result.timestamp
                ));
            }

            markdown.push_str("\n");
        }

        // Communication Tests
        if !self.communication_tests.is_empty() {
            markdown.push_str("## Communication Tests\n\n");
            markdown.push_str("| Source | Target | Status | Response Time (ms) |\n");
            markdown.push_str("|--------|--------|--------|-------------------|\n");

            for result in &self.communication_tests {
                let status_icon = if result.success { "✅" } else { "❌" };
                let response_time = result
                    .response_time_ms
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "-".to_string());

                markdown.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    result.source, result.target, status_icon, response_time
                ));
            }

            markdown.push_str("\n");
        }

        // Metric Analyses
        if !self.metric_analyses.is_empty() {
            markdown.push_str("## Metric Analyses\n\n");
            markdown.push_str("| Service | Metric | Value | Description |\n");
            markdown.push_str("|---------|--------|-------|-------------|\n");

            for analysis in &self.metric_analyses {
                markdown.push_str(&format!(
                    "| {} | {} | {:.2} | {} |\n",
                    analysis.service,
                    analysis.metric_type,
                    analysis.average_value,
                    analysis.description
                ));
            }

            markdown.push_str("\n");
        }

        markdown
    }

    /// Convert the report to HTML
    fn to_html(&self) -> String {
        let mut html = String::new();

        // HTML header
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n");
        html.push_str("<head>\n");
        html.push_str("  <meta charset=\"UTF-8\">\n");
        html.push_str(
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        html.push_str("  <title>IntelliRouter Audit Report</title>\n");
        html.push_str("  <style>\n");
        html.push_str("    body { font-family: Arial, sans-serif; margin: 0; padding: 20px; }\n");
        html.push_str("    h1 { color: #333; }\n");
        html.push_str("    h2 { color: #555; margin-top: 30px; }\n");
        html.push_str(
            "    table { border-collapse: collapse; width: 100%; margin-bottom: 20px; }\n",
        );
        html.push_str("    th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        html.push_str("    th { background-color: #f2f2f2; }\n");
        html.push_str("    tr:nth-child(even) { background-color: #f9f9f9; }\n");
        html.push_str("    .success { color: green; }\n");
        html.push_str("    .warning { color: orange; }\n");
        html.push_str("    .error { color: red; }\n");
        html.push_str("    .summary { display: flex; flex-wrap: wrap; margin-bottom: 20px; }\n");
        html.push_str("    .summary-item { margin-right: 20px; margin-bottom: 10px; }\n");
        html.push_str("  </style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");

        // Title
        html.push_str("  <h1>IntelliRouter Audit Report</h1>\n");

        // Summary
        html.push_str("  <h2>Summary</h2>\n");
        html.push_str("  <div class=\"summary\">\n");
        html.push_str(&format!(
            "    <div class=\"summary-item\"><strong>Timestamp:</strong> {}</div>\n",
            self.timestamp
        ));
        html.push_str(&format!(
            "    <div class=\"summary-item\"><strong>Status:</strong> <span class=\"{}\">{}</span></div>\n",
            if self.success { "success" } else { "error" },
            if self.success { "Success" } else { "Failure" }
        ));
        html.push_str(&format!(
            "    <div class=\"summary-item\"><strong>Services:</strong> {}</div>\n",
            self.service_statuses.len()
        ));
        html.push_str(&format!(
            "    <div class=\"summary-item\"><strong>Tests:</strong> {}</div>\n",
            self.test_results.len()
        ));
        html.push_str(&format!(
            "    <div class=\"summary-item\"><strong>Communication Tests:</strong> {}</div>\n",
            self.communication_tests.len()
        ));
        html.push_str(&format!(
            "    <div class=\"summary-item\"><strong>Metrics:</strong> {}</div>\n",
            self.metrics.len()
        ));
        html.push_str("  </div>\n");

        // Service Statuses
        html.push_str("  <h2>Service Statuses</h2>\n");
        html.push_str("  <table>\n");
        html.push_str("    <tr><th>Service</th><th>Status</th></tr>\n");

        for (service, status) in &self.service_statuses {
            let (status_class, status_text) = match status {
                ServiceStatus::Running => ("success", "✅ Running"),
                ServiceStatus::Failed => ("error", "❌ Failed"),
                ServiceStatus::NotStarted => ("", "⏳ Not Started"),
                ServiceStatus::Starting => ("", "🔄 Starting"),
                ServiceStatus::ShuttingDown => ("", "🔄 Shutting Down"),
                ServiceStatus::Stopped => ("", "⏹️ Stopped"),
                ServiceStatus::Active => ("success", "✅ Active"),
                ServiceStatus::Inactive => ("", "⏹️ Inactive"),
                ServiceStatus::Degraded => ("warning", "⚠️ Degraded"),
            };

            html.push_str(&format!(
                "    <tr><td>{}</td><td class=\"{}\">{}</td></tr>\n",
                service, status_class, status_text
            ));
        }

        html.push_str("  </table>\n");

        // Successes
        if !self.successes.is_empty() {
            html.push_str("  <h2>Successes</h2>\n");
            html.push_str("  <ul>\n");

            for success in &self.successes {
                html.push_str(&format!("    <li class=\"success\">✅ {}</li>\n", success));
            }

            html.push_str("  </ul>\n");
        }

        // Warnings
        if !self.warnings.is_empty() {
            html.push_str("  <h2>Warnings</h2>\n");
            html.push_str("  <ul>\n");

            for warning in &self.warnings {
                html.push_str(&format!("    <li class=\"warning\">⚠️ {}</li>\n", warning));
            }

            html.push_str("  </ul>\n");
        }

        // Errors
        if !self.errors.is_empty() {
            html.push_str("  <h2>Errors</h2>\n");
            html.push_str("  <ul>\n");

            for error in &self.errors {
                html.push_str(&format!("    <li class=\"error\">❌ {}</li>\n", error));
            }

            html.push_str("  </ul>\n");
        }

        // Test Results
        if !self.test_results.is_empty() {
            html.push_str("  <h2>Test Results</h2>\n");
            html.push_str("  <table>\n");
            html.push_str("    <tr><th>Test</th><th>Status</th><th>Duration (ms)</th><th>Timestamp</th></tr>\n");

            for result in &self.test_results {
                let status_class = if result.success { "success" } else { "error" };
                let status_icon = if result.success { "✅" } else { "❌" };

                html.push_str(&format!(
                    "    <tr><td>{}</td><td class=\"{}\">{}</td><td>{}</td><td>{}</td></tr>\n",
                    result.test_flow,
                    status_class,
                    status_icon,
                    result.duration_ms,
                    result.timestamp
                ));
            }

            html.push_str("  </table>\n");
        }

        // Communication Tests
        if !self.communication_tests.is_empty() {
            html.push_str("  <h2>Communication Tests</h2>\n");
            html.push_str("  <table>\n");
            html.push_str("    <tr><th>Source</th><th>Target</th><th>Status</th><th>Response Time (ms)</th></tr>\n");

            for result in &self.communication_tests {
                let status_class = if result.success { "success" } else { "error" };
                let status_icon = if result.success { "✅" } else { "❌" };
                let response_time = result
                    .response_time_ms
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "-".to_string());

                html.push_str(&format!(
                    "    <tr><td>{}</td><td>{}</td><td class=\"{}\">{}</td><td>{}</td></tr>\n",
                    result.source, result.target, status_class, status_icon, response_time
                ));
            }

            html.push_str("  </table>\n");
        }

        // Metric Analyses
        if !self.metric_analyses.is_empty() {
            html.push_str("  <h2>Metric Analyses</h2>\n");
            html.push_str("  <table>\n");
            html.push_str(
                "    <tr><th>Service</th><th>Metric</th><th>Value</th><th>Description</th></tr>\n",
            );

            for analysis in &self.metric_analyses {
                html.push_str(&format!(
                    "    <tr><td>{}</td><td>{}</td><td>{:.2}</td><td>{}</td></tr>\n",
                    analysis.service,
                    analysis.metric_type,
                    analysis.average_value,
                    analysis.description
                ));
            }

            html.push_str("  </table>\n");
        }

        // HTML footer
        html.push_str("</body>\n");
        html.push_str("</html>\n");

        html
    }
}

impl Default for AuditReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_creation() {
        let report = AuditReport::new();

        assert!(report.success);
        assert!(report.service_statuses.is_empty());
        assert!(report.successes.is_empty());
        assert!(report.warnings.is_empty());
        assert!(report.errors.is_empty());
        assert!(report.test_results.is_empty());
        assert!(report.communication_tests.is_empty());
        assert!(report.metrics.is_empty());
        assert!(report.metric_analyses.is_empty());
    }

    #[test]
    fn test_report_add_error() {
        let mut report = AuditReport::new();

        assert!(report.success);

        report.add_error("Test error");

        assert!(!report.success);
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.errors[0], "Test error");
    }
}
