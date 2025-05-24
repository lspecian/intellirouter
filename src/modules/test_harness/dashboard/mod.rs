//! Dashboard Module
//!
//! This module provides functionality for creating dashboards to visualize test results.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{debug, error, info, warn};

use super::metrics::{Metric, MetricCollection};
use super::TestRun;
use crate::modules::test_harness::types::TestHarnessError;

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Dashboard title
    pub title: String,
    /// Dashboard description
    pub description: Option<String>,
    /// Dashboard output directory
    pub output_dir: PathBuf,
    /// Dashboard refresh interval in seconds
    pub refresh_interval: u64,
    /// Dashboard theme
    pub theme: String,
    /// Dashboard layout
    pub layout: String,
    /// Dashboard panels
    pub panels: Vec<DashboardPanelConfig>,
    /// Dashboard metadata
    pub metadata: HashMap<String, String>,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            title: "Test Dashboard".to_string(),
            description: None,
            output_dir: PathBuf::from("dashboard"),
            refresh_interval: 60,
            theme: "default".to_string(),
            layout: "grid".to_string(),
            panels: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

/// Dashboard panel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPanelConfig {
    /// Panel ID
    pub id: String,
    /// Panel title
    pub title: String,
    /// Panel description
    pub description: Option<String>,
    /// Panel type
    pub panel_type: String,
    /// Panel width
    pub width: u32,
    /// Panel height
    pub height: u32,
    /// Panel position X
    pub position_x: u32,
    /// Panel position Y
    pub position_y: u32,
    /// Panel data source
    pub data_source: String,
    /// Panel query
    pub query: Option<String>,
    /// Panel options
    pub options: HashMap<String, String>,
}

/// Dashboard panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPanel {
    /// Panel configuration
    pub config: DashboardPanelConfig,
    /// Panel data
    pub data: serde_json::Value,
}

impl DashboardPanel {
    /// Create a new dashboard panel
    pub fn new(config: DashboardPanelConfig, data: serde_json::Value) -> Self {
        Self { config, data }
    }

    /// Get the panel ID
    pub fn id(&self) -> &str {
        &self.config.id
    }

    /// Get the panel title
    pub fn title(&self) -> &str {
        &self.config.title
    }

    /// Get the panel description
    pub fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }

    /// Get the panel type
    pub fn panel_type(&self) -> &str {
        &self.config.panel_type
    }

    /// Get the panel width
    pub fn width(&self) -> u32 {
        self.config.width
    }

    /// Get the panel height
    pub fn height(&self) -> u32 {
        self.config.height
    }

    /// Get the panel position X
    pub fn position_x(&self) -> u32 {
        self.config.position_x
    }

    /// Get the panel position Y
    pub fn position_y(&self) -> u32 {
        self.config.position_y
    }

    /// Get the panel data source
    pub fn data_source(&self) -> &str {
        &self.config.data_source
    }

    /// Get the panel query
    pub fn query(&self) -> Option<&str> {
        self.config.query.as_deref()
    }

    /// Get the panel options
    pub fn options(&self) -> &HashMap<String, String> {
        &self.config.options
    }

    /// Get the panel data
    pub fn data(&self) -> &serde_json::Value {
        &self.data
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
            theme: "default".to_string(),
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

    /// Add multiple panels to the view
    pub fn with_panels(mut self, panel_ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for panel_id in panel_ids {
            self.panels.push(panel_id.into());
        }
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

    /// Add multiple options to the view
    pub fn with_options(
        mut self,
        options: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (key, value) in options {
            self.options.insert(key.into(), value.into());
        }
        self
    }
}

/// Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    /// Dashboard configuration
    pub config: DashboardConfig,
    /// Dashboard panels
    pub panels: Vec<DashboardPanel>,
    /// Dashboard views
    pub views: Vec<DashboardView>,
    /// Dashboard test runs
    pub test_runs: Vec<TestRun>,
    /// Dashboard metrics
    pub metrics: MetricCollection,
}

impl Dashboard {
    /// Create a new dashboard
    pub fn new(config: DashboardConfig, test_runs: Vec<TestRun>) -> Self {
        Self {
            config,
            panels: Vec::new(),
            views: Vec::new(),
            test_runs,
            metrics: MetricCollection::new(),
        }
    }

    /// Add a panel to the dashboard
    pub fn add_panel(&mut self, panel: DashboardPanel) {
        self.panels.push(panel);
    }

    /// Add a view to the dashboard
    pub fn add_view(&mut self, view: DashboardView) {
        self.views.push(view);
    }

    /// Add a test run to the dashboard
    pub fn add_test_run(&mut self, test_run: TestRun) {
        self.test_runs.push(test_run);
    }

    /// Add a metric to the dashboard
    pub fn add_metric(&mut self, metric: Metric) {
        self.metrics.add_metric(metric);
    }

    /// Get a panel by ID
    pub fn get_panel(&self, id: &str) -> Option<&DashboardPanel> {
        self.panels.iter().find(|p| p.id() == id)
    }

    /// Get a view by ID
    pub fn get_view(&self, id: &str) -> Option<&DashboardView> {
        self.views.iter().find(|v| v.id == id)
    }

    /// Get a test run by ID
    pub fn get_test_run(&self, id: &str) -> Option<&TestRun> {
        self.test_runs.iter().find(|r| r.id == id)
    }

    /// Get a metric by ID
    pub fn get_metric(&self, id: &str) -> Option<&Metric> {
        self.metrics.get_metric(id)
    }

    /// Get the dashboard title
    pub fn title(&self) -> &str {
        &self.config.title
    }

    /// Get the dashboard description
    pub fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }

    /// Get the dashboard theme
    pub fn theme(&self) -> &str {
        &self.config.theme
    }

    /// Get the dashboard layout
    pub fn layout(&self) -> &str {
        &self.config.layout
    }

    /// Get the dashboard refresh interval
    pub fn refresh_interval(&self) -> u64 {
        self.config.refresh_interval
    }

    /// Get the dashboard metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.config.metadata
    }

    /// Get the dashboard panels
    pub fn panels(&self) -> &[DashboardPanel] {
        &self.panels
    }

    /// Get the dashboard views
    pub fn views(&self) -> &[DashboardView] {
        &self.views
    }

    /// Get the dashboard test runs
    pub fn test_runs(&self) -> &[TestRun] {
        &self.test_runs
    }

    /// Get the dashboard metrics
    pub fn metrics(&self) -> &MetricCollection {
        &self.metrics
    }

    /// Calculate dashboard metrics
    pub fn calculate_metrics(&mut self) {
        // Calculate test metrics
        let total_tests = self.test_runs.iter().map(|r| r.test_count()).sum::<usize>();
        let passed_tests = self
            .test_runs
            .iter()
            .map(|r| r.passed_count())
            .sum::<usize>();
        let failed_tests = self
            .test_runs
            .iter()
            .map(|r| r.failed_count())
            .sum::<usize>();
        let skipped_tests = self
            .test_runs
            .iter()
            .map(|r| r.skipped_count())
            .sum::<usize>();

        let pass_rate = if total_tests > 0 {
            passed_tests as f64 / total_tests as f64
        } else {
            0.0
        };

        // Add metrics
        self.metrics
            .add_metric(Metric::new("total_tests", total_tests as f64));
        self.metrics
            .add_metric(Metric::new("passed_tests", passed_tests as f64));
        self.metrics
            .add_metric(Metric::new("failed_tests", failed_tests as f64));
        self.metrics
            .add_metric(Metric::new("skipped_tests", skipped_tests as f64));
        self.metrics.add_metric(Metric::new("pass_rate", pass_rate));

        // Calculate assertion metrics
        let total_assertions = self
            .test_runs
            .iter()
            .map(|r| r.assertion_count())
            .sum::<usize>();

        let passed_assertions = self
            .test_runs
            .iter()
            .map(|r| r.passed_assertion_count())
            .sum::<usize>();

        let failed_assertions = self
            .test_runs
            .iter()
            .map(|r| r.failed_assertion_count())
            .sum::<usize>();

        let assertion_pass_rate = if total_assertions > 0 {
            passed_assertions as f64 / total_assertions as f64
        } else {
            0.0
        };

        // Add assertion metrics
        self.metrics
            .add_metric(Metric::new("total_assertions", total_assertions as f64));
        self.metrics
            .add_metric(Metric::new("passed_assertions", passed_assertions as f64));
        self.metrics
            .add_metric(Metric::new("failed_assertions", failed_assertions as f64));
        self.metrics
            .add_metric(Metric::new("assertion_pass_rate", assertion_pass_rate));

        // Calculate time metrics
        let total_duration = self
            .test_runs
            .iter()
            .map(|r| r.duration.as_secs_f64())
            .sum::<f64>();

        let avg_duration = if self.test_runs.len() > 0 {
            total_duration / self.test_runs.len() as f64
        } else {
            0.0
        };

        // Add time metrics
        self.metrics
            .add_metric(Metric::new("total_duration", total_duration));
        self.metrics
            .add_metric(Metric::new("avg_duration", avg_duration));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::test_harness::reporting::TestStatus;
    use std::time::Duration;

    #[test]
    fn test_dashboard_panel() {
        let config = DashboardPanelConfig {
            id: "panel-1".to_string(),
            title: "Test Panel".to_string(),
            description: Some("Test panel description".to_string()),
            panel_type: "chart".to_string(),
            width: 12,
            height: 6,
            position_x: 0,
            position_y: 0,
            data_source: "test_runs".to_string(),
            query: Some("status=passed".to_string()),
            options: HashMap::new(),
        };

        let data = serde_json::json!({
            "labels": ["Passed", "Failed", "Skipped"],
            "values": [10, 2, 1]
        });

        let panel = DashboardPanel::new(config, data);

        assert_eq!(panel.id(), "panel-1");
        assert_eq!(panel.title(), "Test Panel");
        assert_eq!(panel.description(), Some("Test panel description"));
        assert_eq!(panel.panel_type(), "chart");
        assert_eq!(panel.width(), 12);
        assert_eq!(panel.height(), 6);
        assert_eq!(panel.position_x(), 0);
        assert_eq!(panel.position_y(), 0);
        assert_eq!(panel.data_source(), "test_runs");
        assert_eq!(panel.query(), Some("status=passed"));
    }

    #[test]
    fn test_dashboard_view() {
        let view = DashboardView::new("view-1", "Test View")
            .with_description("Test view description")
            .with_panel("panel-1")
            .with_panel("panel-2")
            .with_layout("grid")
            .with_theme("dark")
            .with_option("show_title", "true");

        assert_eq!(view.id, "view-1");
        assert_eq!(view.title, "Test View");
        assert_eq!(view.description, Some("Test view description".to_string()));
        assert_eq!(
            view.panels,
            vec!["panel-1".to_string(), "panel-2".to_string()]
        );
        assert_eq!(view.layout, "grid");
        assert_eq!(view.theme, "dark");
        assert_eq!(view.options.get("show_title"), Some(&"true".to_string()));
    }
}
