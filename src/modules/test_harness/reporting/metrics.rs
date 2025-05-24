//! Metrics module for collecting and analyzing test metrics

use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter metric
    Counter,
    /// Gauge metric
    Gauge,
    /// Histogram metric
    Histogram,
    /// Summary metric
    Summary,
    /// Timer metric
    Timer,
}

/// Metric aggregation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricAggregation {
    /// Sum aggregation
    Sum,
    /// Average aggregation
    Average,
    /// Minimum aggregation
    Min,
    /// Maximum aggregation
    Max,
    /// Percentile aggregation
    Percentile(u8),
    /// Count aggregation
    Count,
}

/// Metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name
    pub name: String,
    /// Metric description
    pub description: Option<String>,
    /// Metric type
    pub metric_type: MetricType,
    /// Metric value
    pub value: f64,
    /// Metric unit
    pub unit: Option<String>,
    /// Metric tags
    pub tags: HashMap<String, String>,
    /// Metric timestamp
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl Metric {
    /// Create a new metric
    pub fn new(name: impl Into<String>, metric_type: MetricType, value: f64) -> Self {
        Self {
            name: name.into(),
            description: None,
            metric_type,
            value,
            unit: None,
            tags: HashMap::new(),
            timestamp: Some(chrono::Utc::now()),
        }
    }

    /// Set the metric description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the metric unit
    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// Set the metric timestamp
    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
}

/// Metric collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricCollection {
    /// Metrics
    pub metrics: Vec<Metric>,
}

impl MetricCollection {
    /// Create a new metric collection
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
        }
    }

    /// Add a metric
    pub fn add_metric(&mut self, metric: Metric) {
        self.metrics.push(metric);
    }

    /// Get a metric by name
    pub fn get_metric(&self, name: &str) -> Option<&Metric> {
        self.metrics.iter().find(|m| m.name == name)
    }

    /// Get metrics by type
    pub fn get_metrics_by_type(&self, metric_type: MetricType) -> Vec<&Metric> {
        self.metrics
            .iter()
            .filter(|m| m.metric_type == metric_type)
            .collect()
    }

    /// Get metrics by tag
    pub fn get_metrics_by_tag(&self, key: &str, value: &str) -> Vec<&Metric> {
        self.metrics
            .iter()
            .filter(|m| m.tags.get(key).map_or(false, |v| v == value))
            .collect()
    }

    /// Aggregate metrics
    pub fn aggregate(&self, name: &str, aggregation: MetricAggregation) -> Option<f64> {
        let metrics: Vec<&Metric> = self.metrics.iter().filter(|m| m.name == name).collect();

        if metrics.is_empty() {
            return None;
        }

        match aggregation {
            MetricAggregation::Sum => Some(metrics.iter().map(|m| m.value).sum()),
            MetricAggregation::Average => {
                let sum: f64 = metrics.iter().map(|m| m.value).sum();
                Some(sum / metrics.len() as f64)
            }
            MetricAggregation::Min => metrics
                .iter()
                .map(|m| m.value)
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)),
            MetricAggregation::Max => metrics
                .iter()
                .map(|m| m.value)
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)),
            MetricAggregation::Percentile(p) => {
                let mut values: Vec<f64> = metrics.iter().map(|m| m.value).collect();
                values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                let index = (values.len() as f64 * (p as f64 / 100.0)) as usize;
                values.get(index).copied()
            }
            MetricAggregation::Count => Some(metrics.len() as f64),
        }
    }
}

impl Default for MetricCollection {
    fn default() -> Self {
        Self::new()
    }
}

/// Time metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeMetrics {
    /// Total duration
    pub total_duration: Duration,
    /// Setup duration
    pub setup_duration: Option<Duration>,
    /// Teardown duration
    pub teardown_duration: Option<Duration>,
    /// Test durations
    pub test_durations: HashMap<String, Duration>,
}

impl TimeMetrics {
    /// Create new time metrics
    pub fn new(total_duration: Duration) -> Self {
        Self {
            total_duration,
            setup_duration: None,
            teardown_duration: None,
            test_durations: HashMap::new(),
        }
    }

    /// Set the setup duration
    pub fn with_setup_duration(mut self, duration: Duration) -> Self {
        self.setup_duration = Some(duration);
        self
    }

    /// Set the teardown duration
    pub fn with_teardown_duration(mut self, duration: Duration) -> Self {
        self.teardown_duration = Some(duration);
        self
    }

    /// Add a test duration
    pub fn add_test_duration(&mut self, test_id: impl Into<String>, duration: Duration) {
        self.test_durations.insert(test_id.into(), duration);
    }

    /// Get the average test duration
    pub fn average_test_duration(&self) -> Option<Duration> {
        if self.test_durations.is_empty() {
            return None;
        }

        let total_nanos: u128 = self.test_durations.values().map(|d| d.as_nanos()).sum();

        Some(Duration::from_nanos(
            (total_nanos / self.test_durations.len() as u128) as u64,
        ))
    }
}

/// Assertion metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionMetrics {
    /// Total assertions
    pub total_assertions: usize,
    /// Passed assertions
    pub passed_assertions: usize,
    /// Failed assertions
    pub failed_assertions: usize,
    /// Warning assertions
    pub warning_assertions: usize,
}

impl AssertionMetrics {
    /// Create new assertion metrics
    pub fn new(
        total_assertions: usize,
        passed_assertions: usize,
        failed_assertions: usize,
        warning_assertions: usize,
    ) -> Self {
        Self {
            total_assertions,
            passed_assertions,
            failed_assertions,
            warning_assertions,
        }
    }

    /// Get the pass rate
    pub fn pass_rate(&self) -> f64 {
        if self.total_assertions == 0 {
            0.0
        } else {
            self.passed_assertions as f64 / self.total_assertions as f64
        }
    }

    /// Get the fail rate
    pub fn fail_rate(&self) -> f64 {
        if self.total_assertions == 0 {
            0.0
        } else {
            self.failed_assertions as f64 / self.total_assertions as f64
        }
    }

    /// Get the warning rate
    pub fn warning_rate(&self) -> f64 {
        if self.total_assertions == 0 {
            0.0
        } else {
            self.warning_assertions as f64 / self.total_assertions as f64
        }
    }
}

/// Test metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetrics {
    /// Total tests
    pub total_tests: usize,
    /// Passed tests
    pub passed_tests: usize,
    /// Failed tests
    pub failed_tests: usize,
    /// Skipped tests
    pub skipped_tests: usize,
    /// Running tests
    pub running_tests: usize,
    /// Pending tests
    pub pending_tests: usize,
    /// Blocked tests
    pub blocked_tests: usize,
    /// Error tests
    pub error_tests: usize,
    /// Time metrics
    pub time_metrics: TimeMetrics,
    /// Assertion metrics
    pub assertion_metrics: AssertionMetrics,
}

impl TestMetrics {
    /// Create new test metrics
    pub fn new(
        total_tests: usize,
        passed_tests: usize,
        failed_tests: usize,
        skipped_tests: usize,
        running_tests: usize,
        pending_tests: usize,
        blocked_tests: usize,
        error_tests: usize,
        time_metrics: TimeMetrics,
        assertion_metrics: AssertionMetrics,
    ) -> Self {
        Self {
            total_tests,
            passed_tests,
            failed_tests,
            skipped_tests,
            running_tests,
            pending_tests,
            blocked_tests,
            error_tests,
            time_metrics,
            assertion_metrics,
        }
    }

    /// Get the pass rate
    pub fn pass_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            self.passed_tests as f64 / self.total_tests as f64
        }
    }

    /// Get the fail rate
    pub fn fail_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            self.failed_tests as f64 / self.total_tests as f64
        }
    }

    /// Get the skip rate
    pub fn skip_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            self.skipped_tests as f64 / self.total_tests as f64
        }
    }

    /// Get the running rate
    pub fn running_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            self.running_tests as f64 / self.total_tests as f64
        }
    }

    /// Get the pending rate
    pub fn pending_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            self.pending_tests as f64 / self.total_tests as f64
        }
    }

    /// Get the blocked rate
    pub fn blocked_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            self.blocked_tests as f64 / self.total_tests as f64
        }
    }

    /// Get the error rate
    pub fn error_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            self.error_tests as f64 / self.total_tests as f64
        }
    }
}
