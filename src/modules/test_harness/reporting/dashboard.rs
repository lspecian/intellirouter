//! Dashboard module for visualizing test results
//!
//! This module provides functionality for creating and managing dashboards
//! for visualizing test results and metrics.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::modules::test_harness::metrics::{Metric, MetricCollection};
use crate::modules::test_harness::types::TestHarnessError;

/// Test run for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRun {
    /// Run ID
    pub id: String,
    /// Run name
    pub name: String,
    /// Run description
    pub description: Option<String>,
    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// End time
    pub end_time: chrono::DateTime<chrono::Utc>,
    /// Duration
    pub duration: std::time::Duration,
    /// Test results
    pub results: Vec<super::TestResult>,
    /// Passed count
    pub passed_count: usize,
    /// Failed count
    pub failed_count: usize,
    /// Skipped count
    pub skipped_count: usize,
    /// Timed out count
    pub timed_out_count: usize,
    /// Panicked count
    pub panicked_count: usize,
    /// Tags
    pub tags: HashMap<String, String>,
}

impl TestRun {
    /// Create a new test run
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            start_time: chrono::Utc::now(),
            end_time: chrono::Utc::now(),
            duration: std::time::Duration::from_secs(0),
            results: Vec::new(),
            passed_count: 0,
            failed_count: 0,
            skipped_count: 0,
            timed_out_count: 0,
            panicked_count: 0,
            tags: HashMap::new(),
        }
    }

    /// Set the run description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the start time
    pub fn with_start_time(mut self, start_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.start_time = start_time;
        self
    }

    /// Set the end time
    pub fn with_end_time(mut self, end_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.end_time = end_time;
        self
    }

    /// Set the duration
    pub fn with_duration(mut self, duration: std::time::Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Add a test result
    pub fn with_result(mut self, result: super::TestResult) -> Self {
        self.results.push(result);
        self
    }

    /// Add multiple test results
    pub fn with_results(mut self, results: Vec<super::TestResult>) -> Self {
        self.results.extend(results);
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// Add multiple tags
    pub fn with_tags(
        mut self,
        tags: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (key, value) in tags {
            self.tags.insert(key.into(), value.into());
        }
        self
    }

    /// Get the total number of tests
    pub fn test_count(&self) -> usize {
        self.results.len()
    }

    /// Get the number of passed tests
    pub fn passed_count(&self) -> usize {
        self.passed_count
    }

    /// Get the number of failed tests
    pub fn failed_count(&self) -> usize {
        self.failed_count
    }

    /// Get the number of skipped tests
    pub fn skipped_count(&self) -> usize {
        self.skipped_count
    }

    /// Get the number of timed out tests
    pub fn timed_out_count(&self) -> usize {
        self.timed_out_count
    }

    /// Get the number of panicked tests
    pub fn panicked_count(&self) -> usize {
        self.panicked_count
    }

    /// Get the pass rate
    pub fn pass_rate(&self) -> f64 {
        if self.test_count() == 0 {
            return 0.0;
        }
        self.passed_count as f64 / self.test_count() as f64
    }

    /// Calculate counts
    pub fn calculate_counts(&mut self) {
        self.passed_count = 0;
        self.failed_count = 0;
        self.skipped_count = 0;
        self.timed_out_count = 0;
        self.panicked_count = 0;

        for result in &self.results {
            match result.outcome {
                crate::modules::test_harness::types::TestOutcome::Passed => {
                    self.passed_count += 1;
                }
                crate::modules::test_harness::types::TestOutcome::Failed => {
                    self.failed_count += 1;
                }
                crate::modules::test_harness::types::TestOutcome::Skipped => {
                    self.skipped_count += 1;
                }
                crate::modules::test_harness::types::TestOutcome::TimedOut => {
                    self.timed_out_count += 1;
                }
                crate::modules::test_harness::types::TestOutcome::Panicked => {
                    self.panicked_count += 1;
                }
            }
        }
    }
}

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
    pub refresh_interval: Option<u64>,
    /// Dashboard theme
    pub theme: Option<String>,
    /// Dashboard logo
    pub logo: Option<String>,
    /// Dashboard metadata
    pub metadata: HashMap<String, String>,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            title: "Test Dashboard".to_string(),
            description: None,
            output_dir: PathBuf::from("dashboard"),
            refresh_interval: Some(30),
            theme: Some("default".to_string()),
            logo: None,
            metadata: HashMap::new(),
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
    /// Panel data
    pub data: serde_json::Value,
    /// Panel width
    pub width: u32,
    /// Panel height
    pub height: u32,
    /// Panel position x
    pub position_x: u32,
    /// Panel position y
    pub position_y: u32,
}

impl DashboardPanel {
    /// Create a new dashboard panel
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        panel_type: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            panel_type: panel_type.into(),
            data,
            width: 12,
            height: 6,
            position_x: 0,
            position_y: 0,
        }
    }

    /// Set the panel description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the panel width
    pub fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Set the panel height
    pub fn with_height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    /// Set the panel position
    pub fn with_position(mut self, x: u32, y: u32) -> Self {
        self.position_x = x;
        self.position_y = y;
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
    pub theme: Option<String>,
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
            theme: None,
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
        self.theme = Some(theme.into());
        self
    }
}

/// Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    /// Dashboard configuration
    pub config: DashboardConfig,
    /// Dashboard panels
    pub panels: HashMap<String, DashboardPanel>,
    /// Dashboard views
    pub views: HashMap<String, DashboardView>,
    /// Dashboard test runs
    pub test_runs: Vec<TestRun>,
    /// Dashboard metrics
    pub metrics: MetricCollection,
}

impl Dashboard {
    /// Create a new dashboard
    pub fn new(config: DashboardConfig) -> Self {
        Self {
            config,
            panels: HashMap::new(),
            views: HashMap::new(),
            test_runs: Vec::new(),
            metrics: MetricCollection::new(),
        }
    }

    /// Add a panel to the dashboard
    pub fn add_panel(&mut self, panel: DashboardPanel) {
        self.panels.insert(panel.id.clone(), panel);
    }

    /// Add a view to the dashboard
    pub fn add_view(&mut self, view: DashboardView) {
        self.views.insert(view.id.clone(), view);
    }

    /// Add a test run to the dashboard
    pub fn add_test_run(&mut self, test_run: TestRun) {
        self.test_runs.push(test_run);
    }

    /// Add a metric to the dashboard
    pub fn add_metric(&mut self, metric: Metric) {
        self.metrics.add_metric(metric);
    }

    /// Calculate metrics
    pub fn calculate_metrics(&mut self) {
        // Calculate pass rate
        let total_tests = self.test_runs.iter().map(|r| r.test_count()).sum::<usize>();
        let passed_tests = self
            .test_runs
            .iter()
            .map(|r| r.passed_count())
            .sum::<usize>();
        let pass_rate = if total_tests > 0 {
            passed_tests as f64 / total_tests as f64
        } else {
            0.0
        };

        // Add pass rate metric
        let pass_rate_metric = Metric::new("pass_rate", pass_rate)
            .with_name("Pass Rate")
            .with_description("Test pass rate")
            .with_type(crate::modules::test_harness::metrics::MetricType::Gauge)
            .with_unit("percentage")
            .with_timestamp(chrono::Utc::now());

        self.metrics.add_metric(pass_rate_metric);

        // Add test count metric
        let test_count_metric = Metric::new("test_count", total_tests as f64)
            .with_name("Test Count")
            .with_description("Total number of tests")
            .with_type(crate::modules::test_harness::metrics::MetricType::Gauge)
            .with_unit("count")
            .with_timestamp(chrono::Utc::now());

        self.metrics.add_metric(test_count_metric);

        // Add passed tests metric
        let passed_tests_metric = Metric::new("passed_tests", passed_tests as f64)
            .with_name("Passed Tests")
            .with_description("Number of passed tests")
            .with_type(crate::modules::test_harness::metrics::MetricType::Gauge)
            .with_unit("count")
            .with_timestamp(chrono::Utc::now());

        self.metrics.add_metric(passed_tests_metric);

        // Add failed tests metric
        let failed_tests = self
            .test_runs
            .iter()
            .map(|r| r.failed_count())
            .sum::<usize>();
        let failed_tests_metric = Metric::new("failed_tests", failed_tests as f64)
            .with_name("Failed Tests")
            .with_description("Number of failed tests")
            .with_type(crate::modules::test_harness::metrics::MetricType::Gauge)
            .with_unit("count")
            .with_timestamp(chrono::Utc::now());

        self.metrics.add_metric(failed_tests_metric);

        // Add skipped tests metric
        let skipped_tests = self
            .test_runs
            .iter()
            .map(|r| r.skipped_count())
            .sum::<usize>();
        let skipped_tests_metric = Metric::new("skipped_tests", skipped_tests as f64)
            .with_name("Skipped Tests")
            .with_description("Number of skipped tests")
            .with_type(crate::modules::test_harness::metrics::MetricType::Gauge)
            .with_unit("count")
            .with_timestamp(chrono::Utc::now());

        self.metrics.add_metric(skipped_tests_metric);

        // Calculate aggregations
        self.metrics.calculate_aggregations();
    }

    /// Get test runs
    pub fn test_runs(&self) -> &[TestRun] {
        &self.test_runs
    }

    /// Get metrics
    pub fn metrics(&self) -> &MetricCollection {
        &self.metrics
    }
}
