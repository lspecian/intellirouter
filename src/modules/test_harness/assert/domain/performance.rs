//! Performance assertions for the assertion framework.
//!
//! This module provides assertions for performance testing.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::modules::test_harness::assert::core::{
    assert_that, AssertThat, AssertionOutcome, AssertionResult,
};
use crate::modules::test_harness::types::TestHarnessError;

/// Represents a performance metric for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    /// The metric name.
    pub name: String,
    /// The metric value.
    pub value: f64,
    /// The metric unit.
    pub unit: String,
    /// The metric timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The metric metadata.
    pub metadata: Value,
}

impl PerformanceMetric {
    /// Creates a new performance metric.
    pub fn new(name: &str, value: f64, unit: &str) -> Self {
        Self {
            name: name.to_string(),
            value,
            unit: unit.to_string(),
            timestamp: chrono::Utc::now(),
            metadata: Value::Null,
        }
    }

    /// Sets the metric timestamp.
    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Sets the metric metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Represents a performance test result for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestResult {
    /// The test name.
    pub name: String,
    /// The test description.
    pub description: String,
    /// The test metrics.
    pub metrics: Vec<PerformanceMetric>,
    /// The test start time.
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// The test end time.
    pub end_time: chrono::DateTime<chrono::Utc>,
    /// The test duration.
    pub duration: Duration,
    /// The test parameters.
    pub parameters: Value,
    /// The test metadata.
    pub metadata: Value,
}

impl PerformanceTestResult {
    /// Creates a new performance test result.
    pub fn new(name: &str) -> Self {
        let now = chrono::Utc::now();
        Self {
            name: name.to_string(),
            description: "".to_string(),
            metrics: Vec::new(),
            start_time: now,
            end_time: now,
            duration: Duration::default(),
            parameters: Value::Null,
            metadata: Value::Null,
        }
    }

    /// Sets the test description.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Adds a metric to the test result.
    pub fn with_metric(mut self, metric: PerformanceMetric) -> Self {
        self.metrics.push(metric);
        self
    }

    /// Sets the test start time.
    pub fn with_start_time(mut self, start_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.start_time = start_time;
        self
    }

    /// Sets the test end time.
    pub fn with_end_time(mut self, end_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.end_time = end_time;
        self
    }

    /// Sets the test duration.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the test parameters.
    pub fn with_parameters(mut self, parameters: Value) -> Self {
        self.parameters = parameters;
        self
    }

    /// Sets the test metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Gets a metric by name.
    pub fn get_metric(&self, name: &str) -> Option<&PerformanceMetric> {
        self.metrics.iter().find(|m| m.name == name)
    }

    /// Gets a metric value by name.
    pub fn get_metric_value(&self, name: &str) -> Option<f64> {
        self.get_metric(name).map(|m| m.value)
    }
}

/// Represents a load test result for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestResult {
    /// The test name.
    pub name: String,
    /// The test description.
    pub description: String,
    /// The test metrics.
    pub metrics: HashMap<String, Vec<PerformanceMetric>>,
    /// The test start time.
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// The test end time.
    pub end_time: chrono::DateTime<chrono::Utc>,
    /// The test duration.
    pub duration: Duration,
    /// The test parameters.
    pub parameters: Value,
    /// The test metadata.
    pub metadata: Value,
    /// The test summary.
    pub summary: HashMap<String, f64>,
}

impl LoadTestResult {
    /// Creates a new load test result.
    pub fn new(name: &str) -> Self {
        let now = chrono::Utc::now();
        Self {
            name: name.to_string(),
            description: "".to_string(),
            metrics: HashMap::new(),
            start_time: now,
            end_time: now,
            duration: Duration::default(),
            parameters: Value::Null,
            metadata: Value::Null,
            summary: HashMap::new(),
        }
    }

    /// Sets the test description.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Adds a metric to the test result.
    pub fn with_metric(mut self, category: &str, metric: PerformanceMetric) -> Self {
        self.metrics
            .entry(category.to_string())
            .or_insert_with(Vec::new)
            .push(metric);
        self
    }

    /// Sets the test start time.
    pub fn with_start_time(mut self, start_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.start_time = start_time;
        self
    }

    /// Sets the test end time.
    pub fn with_end_time(mut self, end_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.end_time = end_time;
        self
    }

    /// Sets the test duration.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the test parameters.
    pub fn with_parameters(mut self, parameters: Value) -> Self {
        self.parameters = parameters;
        self
    }

    /// Sets the test metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Adds a summary value to the test result.
    pub fn with_summary_value(mut self, name: &str, value: f64) -> Self {
        self.summary.insert(name.to_string(), value);
        self
    }

    /// Gets a metric by category and name.
    pub fn get_metric(&self, category: &str, name: &str) -> Option<&PerformanceMetric> {
        self.metrics
            .get(category)
            .and_then(|metrics| metrics.iter().find(|m| m.name == name))
    }

    /// Gets a metric value by category and name.
    pub fn get_metric_value(&self, category: &str, name: &str) -> Option<f64> {
        self.get_metric(category, name).map(|m| m.value)
    }

    /// Gets a summary value by name.
    pub fn get_summary_value(&self, name: &str) -> Option<f64> {
        self.summary.get(name).copied()
    }

    /// Calculates summary statistics for the test result.
    pub fn calculate_summary(&mut self) {
        // Calculate latency statistics
        if let Some(latency_metrics) = self.metrics.get("latency") {
            let latency_values: Vec<f64> = latency_metrics.iter().map(|m| m.value).collect();
            if !latency_values.is_empty() {
                // Calculate min, max, avg, p50, p90, p95, p99
                let mut sorted_values = latency_values.clone();
                sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                let min = sorted_values.first().unwrap_or(&0.0);
                let max = sorted_values.last().unwrap_or(&0.0);
                let avg = latency_values.iter().sum::<f64>() / latency_values.len() as f64;
                let p50 = percentile(&sorted_values, 50.0);
                let p90 = percentile(&sorted_values, 90.0);
                let p95 = percentile(&sorted_values, 95.0);
                let p99 = percentile(&sorted_values, 99.0);

                self.summary.insert("latency_min".to_string(), *min);
                self.summary.insert("latency_max".to_string(), *max);
                self.summary.insert("latency_avg".to_string(), avg);
                self.summary.insert("latency_p50".to_string(), p50);
                self.summary.insert("latency_p90".to_string(), p90);
                self.summary.insert("latency_p95".to_string(), p95);
                self.summary.insert("latency_p99".to_string(), p99);
            }
        }

        // Calculate throughput
        if let Some(throughput_metrics) = self.metrics.get("throughput") {
            let throughput_values: Vec<f64> = throughput_metrics.iter().map(|m| m.value).collect();
            if !throughput_values.is_empty() {
                let avg = throughput_values.iter().sum::<f64>() / throughput_values.len() as f64;
                self.summary.insert("throughput_avg".to_string(), avg);
            }
        }

        // Calculate error rate
        if let Some(error_metrics) = self.metrics.get("errors") {
            let error_count = error_metrics.len();
            let total_requests = self.metrics.get("requests").map(|m| m.len()).unwrap_or(0);
            if total_requests > 0 {
                let error_rate = error_count as f64 / total_requests as f64 * 100.0;
                self.summary.insert("error_rate".to_string(), error_rate);
            }
        }
    }
}

/// Calculates a percentile value from a sorted array of values.
fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    let index = (p / 100.0 * sorted_values.len() as f64).ceil() as usize - 1;
    let index = std::cmp::min(index, sorted_values.len() - 1);
    sorted_values[index]
}

/// Assertions for performance testing.
#[derive(Debug, Clone)]
pub struct PerformanceAssertions;

impl PerformanceAssertions {
    /// Creates a new performance assertions instance.
    pub fn new() -> Self {
        Self
    }

    /// Asserts that a metric has a specific value.
    pub fn assert_metric_value(
        &self,
        result: &PerformanceTestResult,
        metric_name: &str,
        expected: f64,
    ) -> AssertionResult {
        match result.get_metric_value(metric_name) {
            Some(value) => {
                if (value - expected).abs() < f64::EPSILON {
                    AssertionResult::new(
                        &format!("Metric '{}' has value {}", metric_name, expected),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("Metric '{}' has value {}", metric_name, expected),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            &format!("Metric '{}' has unexpected value", metric_name),
                            &format!("{}", expected),
                            &format!("{}", value),
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("Metric '{}' has value {}", metric_name, expected),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Metric '{}' not found", metric_name),
                    &format!("{}", expected),
                    "No such metric",
                ),
            ),
        }
    }

    /// Asserts that a metric is less than a specific value.
    pub fn assert_metric_less_than(
        &self,
        result: &PerformanceTestResult,
        metric_name: &str,
        max: f64,
    ) -> AssertionResult {
        match result.get_metric_value(metric_name) {
            Some(value) => {
                if value < max {
                    AssertionResult::new(
                        &format!("Metric '{}' < {}", metric_name, max),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("Metric '{}' < {}", metric_name, max),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            &format!("Metric '{}' is not less than maximum", metric_name),
                            &format!("< {}", max),
                            &format!("{}", value),
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("Metric '{}' < {}", metric_name, max),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Metric '{}' not found", metric_name),
                    &format!("< {}", max),
                    "No such metric",
                ),
            ),
        }
    }

    /// Asserts that a metric is greater than a specific value.
    pub fn assert_metric_greater_than(
        &self,
        result: &PerformanceTestResult,
        metric_name: &str,
        min: f64,
    ) -> AssertionResult {
        match result.get_metric_value(metric_name) {
            Some(value) => {
                if value > min {
                    AssertionResult::new(
                        &format!("Metric '{}' > {}", metric_name, min),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("Metric '{}' > {}", metric_name, min),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            &format!("Metric '{}' is not greater than minimum", metric_name),
                            &format!("> {}", min),
                            &format!("{}", value),
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("Metric '{}' > {}", metric_name, min),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Metric '{}' not found", metric_name),
                    &format!("> {}", min),
                    "No such metric",
                ),
            ),
        }
    }

    /// Asserts that a metric is between two values.
    pub fn assert_metric_between(
        &self,
        result: &PerformanceTestResult,
        metric_name: &str,
        min: f64,
        max: f64,
    ) -> AssertionResult {
        match result.get_metric_value(metric_name) {
            Some(value) => {
                if value >= min && value <= max {
                    AssertionResult::new(
                        &format!("Metric '{}' between {} and {}", metric_name, min, max),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("Metric '{}' between {} and {}", metric_name, min, max),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            &format!("Metric '{}' is not between min and max", metric_name),
                            &format!("between {} and {}", min, max),
                            &format!("{}", value),
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("Metric '{}' between {} and {}", metric_name, min, max),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Metric '{}' not found", metric_name),
                    &format!("between {} and {}", min, max),
                    "No such metric",
                ),
            ),
        }
    }

    /// Asserts that a load test summary value is less than a specific value.
    pub fn assert_summary_less_than(
        &self,
        result: &LoadTestResult,
        name: &str,
        max: f64,
    ) -> AssertionResult {
        match result.get_summary_value(name) {
            Some(value) => {
                if value < max {
                    AssertionResult::new(
                        &format!("Summary '{}' < {}", name, max),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("Summary '{}' < {}", name, max),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            &format!("Summary '{}' is not less than maximum", name),
                            &format!("< {}", max),
                            &format!("{}", value),
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("Summary '{}' < {}", name, max),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Summary '{}' not found", name),
                    &format!("< {}", max),
                    "No such summary value",
                ),
            ),
        }
    }

    /// Asserts that a load test summary value is greater than a specific value.
    pub fn assert_summary_greater_than(
        &self,
        result: &LoadTestResult,
        name: &str,
        min: f64,
    ) -> AssertionResult {
        match result.get_summary_value(name) {
            Some(value) => {
                if value > min {
                    AssertionResult::new(
                        &format!("Summary '{}' > {}", name, min),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("Summary '{}' > {}", name, min),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            &format!("Summary '{}' is not greater than minimum", name),
                            &format!("> {}", min),
                            &format!("{}", value),
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("Summary '{}' > {}", name, min),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Summary '{}' not found", name),
                    &format!("> {}", min),
                    "No such summary value",
                ),
            ),
        }
    }

    /// Asserts that a load test summary value is between two values.
    pub fn assert_summary_between(
        &self,
        result: &LoadTestResult,
        name: &str,
        min: f64,
        max: f64,
    ) -> AssertionResult {
        match result.get_summary_value(name) {
            Some(value) => {
                if value >= min && value <= max {
                    AssertionResult::new(
                        &format!("Summary '{}' between {} and {}", name, min, max),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("Summary '{}' between {} and {}", name, min, max),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            &format!("Summary '{}' is not between min and max", name),
                            &format!("between {} and {}", min, max),
                            &format!("{}", value),
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("Summary '{}' between {} and {}", name, min, max),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Summary '{}' not found", name),
                    &format!("between {} and {}", min, max),
                    "No such summary value",
                ),
            ),
        }
    }

    /// Asserts that the average latency is less than a specific value.
    pub fn assert_avg_latency_less_than(
        &self,
        result: &LoadTestResult,
        max_ms: f64,
    ) -> AssertionResult {
        self.assert_summary_less_than(result, "latency_avg", max_ms)
    }

    /// Asserts that the 95th percentile latency is less than a specific value.
    pub fn assert_p95_latency_less_than(
        &self,
        result: &LoadTestResult,
        max_ms: f64,
    ) -> AssertionResult {
        self.assert_summary_less_than(result, "latency_p95", max_ms)
    }

    /// Asserts that the 99th percentile latency is less than a specific value.
    pub fn assert_p99_latency_less_than(
        &self,
        result: &LoadTestResult,
        max_ms: f64,
    ) -> AssertionResult {
        self.assert_summary_less_than(result, "latency_p99", max_ms)
    }

    /// Asserts that the average throughput is greater than a specific value.
    pub fn assert_avg_throughput_greater_than(
        &self,
        result: &LoadTestResult,
        min_rps: f64,
    ) -> AssertionResult {
        self.assert_summary_greater_than(result, "throughput_avg", min_rps)
    }

    /// Asserts that the error rate is less than a specific value.
    pub fn assert_error_rate_less_than(
        &self,
        result: &LoadTestResult,
        max_percent: f64,
    ) -> AssertionResult {
        self.assert_summary_less_than(result, "error_rate", max_percent)
    }
}

impl Default for PerformanceAssertions {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new performance assertions instance.
pub fn create_performance_assertions() -> PerformanceAssertions {
    PerformanceAssertions::new()
}

/// Creates a new performance metric.
pub fn create_performance_metric(name: &str, value: f64, unit: &str) -> PerformanceMetric {
    PerformanceMetric::new(name, value, unit)
}

/// Creates a new performance test result.
pub fn create_performance_test_result(name: &str) -> PerformanceTestResult {
    PerformanceTestResult::new(name)
}

/// Creates a new load test result.
pub fn create_load_test_result(name: &str) -> LoadTestResult {
    LoadTestResult::new(name)
}
