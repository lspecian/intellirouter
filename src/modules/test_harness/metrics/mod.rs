//! Metrics Module
//!
//! This module provides functionality for collecting and analyzing metrics.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::modules::test_harness::types::TestHarnessError;

/// Metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl fmt::Display for MetricType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricType::Counter => write!(f, "counter"),
            MetricType::Gauge => write!(f, "gauge"),
            MetricType::Histogram => write!(f, "histogram"),
            MetricType::Summary => write!(f, "summary"),
            MetricType::Timer => write!(f, "timer"),
        }
    }
}

/// Metric aggregation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricAggregation {
    /// Sum aggregation
    Sum,
    /// Average aggregation
    Average,
    /// Minimum aggregation
    Min,
    /// Maximum aggregation
    Max,
    /// Count aggregation
    Count,
    /// Percentile aggregation
    Percentile(u8),
}

impl fmt::Display for MetricAggregation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricAggregation::Sum => write!(f, "sum"),
            MetricAggregation::Average => write!(f, "avg"),
            MetricAggregation::Min => write!(f, "min"),
            MetricAggregation::Max => write!(f, "max"),
            MetricAggregation::Count => write!(f, "count"),
            MetricAggregation::Percentile(p) => write!(f, "p{}", p),
        }
    }
}

/// Metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric ID
    pub id: String,
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
    /// Metric timestamp
    pub timestamp: DateTime<Utc>,
    /// Metric tags
    pub tags: HashMap<String, String>,
    /// Metric values (for histogram and summary)
    pub values: Option<Vec<f64>>,
    /// Metric aggregations
    pub aggregations: HashMap<MetricAggregation, f64>,
}

impl Metric {
    /// Create a new metric
    pub fn new(id: impl Into<String>, value: f64) -> Self {
        Self {
            id: id.into(),
            name: id.into(),
            description: None,
            metric_type: MetricType::Gauge,
            value,
            unit: None,
            timestamp: Utc::now(),
            tags: HashMap::new(),
            values: None,
            aggregations: HashMap::new(),
        }
    }

    /// Set the metric name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the metric description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the metric type
    pub fn with_type(mut self, metric_type: MetricType) -> Self {
        self.metric_type = metric_type;
        self
    }

    /// Set the metric unit
    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Set the metric timestamp
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Add a tag to the metric
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// Add multiple tags to the metric
    pub fn with_tags(
        mut self,
        tags: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (key, value) in tags {
            self.tags.insert(key.into(), value.into());
        }
        self
    }

    /// Set the metric values
    pub fn with_values(mut self, values: Vec<f64>) -> Self {
        self.values = Some(values);
        self
    }

    /// Add an aggregation to the metric
    pub fn with_aggregation(mut self, aggregation: MetricAggregation, value: f64) -> Self {
        self.aggregations.insert(aggregation, value);
        self
    }

    /// Add multiple aggregations to the metric
    pub fn with_aggregations(
        mut self,
        aggregations: impl IntoIterator<Item = (MetricAggregation, f64)>,
    ) -> Self {
        for (aggregation, value) in aggregations {
            self.aggregations.insert(aggregation, value);
        }
        self
    }

    /// Get the metric ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the metric name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the metric description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get the metric type
    pub fn metric_type(&self) -> MetricType {
        self.metric_type
    }

    /// Get the metric value
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Get the metric unit
    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    /// Get the metric timestamp
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Get the metric tags
    pub fn tags(&self) -> &HashMap<String, String> {
        &self.tags
    }

    /// Get the metric values
    pub fn values(&self) -> Option<&[f64]> {
        self.values.as_deref()
    }

    /// Get the metric aggregations
    pub fn aggregations(&self) -> &HashMap<MetricAggregation, f64> {
        &self.aggregations
    }

    /// Get an aggregation value
    pub fn aggregation(&self, aggregation: MetricAggregation) -> Option<f64> {
        self.aggregations.get(&aggregation).copied()
    }

    /// Calculate aggregations
    pub fn calculate_aggregations(&mut self) {
        if let Some(values) = &self.values {
            if !values.is_empty() {
                // Calculate sum
                let sum = values.iter().sum();
                self.aggregations.insert(MetricAggregation::Sum, sum);

                // Calculate average
                let avg = sum / values.len() as f64;
                self.aggregations.insert(MetricAggregation::Average, avg);

                // Calculate min
                if let Some(min) = values.iter().copied().reduce(f64::min) {
                    self.aggregations.insert(MetricAggregation::Min, min);
                }

                // Calculate max
                if let Some(max) = values.iter().copied().reduce(f64::max) {
                    self.aggregations.insert(MetricAggregation::Max, max);
                }

                // Calculate count
                self.aggregations
                    .insert(MetricAggregation::Count, values.len() as f64);

                // Calculate percentiles
                let mut sorted_values = values.clone();
                sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                for p in &[50, 90, 95, 99] {
                    let idx = (sorted_values.len() as f64 * (*p as f64 / 100.0)) as usize;
                    if idx < sorted_values.len() {
                        self.aggregations
                            .insert(MetricAggregation::Percentile(*p), sorted_values[idx]);
                    }
                }
            }
        }
    }
}

/// Test metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetrics {
    /// Total tests
    pub total: usize,
    /// Passed tests
    pub passed: usize,
    /// Failed tests
    pub failed: usize,
    /// Skipped tests
    pub skipped: usize,
    /// Pass rate
    pub pass_rate: f64,
}

impl TestMetrics {
    /// Create new test metrics
    pub fn new(total: usize, passed: usize, failed: usize, skipped: usize) -> Self {
        let pass_rate = if total > 0 {
            passed as f64 / total as f64
        } else {
            0.0
        };

        Self {
            total,
            passed,
            failed,
            skipped,
            pass_rate,
        }
    }
}

/// Assertion metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionMetrics {
    /// Total assertions
    pub total: usize,
    /// Passed assertions
    pub passed: usize,
    /// Failed assertions
    pub failed: usize,
    /// Warning assertions
    pub warnings: usize,
    /// Pass rate
    pub pass_rate: f64,
}

impl AssertionMetrics {
    /// Create new assertion metrics
    pub fn new(total: usize, passed: usize, failed: usize, warnings: usize) -> Self {
        let pass_rate = if total > 0 {
            passed as f64 / total as f64
        } else {
            0.0
        };

        Self {
            total,
            passed,
            failed,
            warnings,
            pass_rate,
        }
    }
}

/// Time metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeMetrics {
    /// Total duration
    pub total_duration: Duration,
    /// Average duration
    pub avg_duration: Duration,
    /// Min duration
    pub min_duration: Duration,
    /// Max duration
    pub max_duration: Duration,
    /// Median duration
    pub median_duration: Duration,
    /// P90 duration
    pub p90_duration: Duration,
    /// P95 duration
    pub p95_duration: Duration,
    /// P99 duration
    pub p99_duration: Duration,
}

impl TimeMetrics {
    /// Create new time metrics
    pub fn new(durations: &[Duration]) -> Self {
        if durations.is_empty() {
            return Self {
                total_duration: Duration::from_secs(0),
                avg_duration: Duration::from_secs(0),
                min_duration: Duration::from_secs(0),
                max_duration: Duration::from_secs(0),
                median_duration: Duration::from_secs(0),
                p90_duration: Duration::from_secs(0),
                p95_duration: Duration::from_secs(0),
                p99_duration: Duration::from_secs(0),
            };
        }

        let total_duration = durations.iter().sum();
        let avg_duration = total_duration / durations.len() as u32;
        let min_duration = *durations.iter().min().unwrap_or(&Duration::from_secs(0));
        let max_duration = *durations.iter().max().unwrap_or(&Duration::from_secs(0));

        let mut sorted_durations = durations.to_vec();
        sorted_durations.sort();

        let median_idx = durations.len() / 2;
        let median_duration = sorted_durations[median_idx];

        let p90_idx = (durations.len() as f64 * 0.9) as usize;
        let p90_duration = sorted_durations[p90_idx.min(durations.len() - 1)];

        let p95_idx = (durations.len() as f64 * 0.95) as usize;
        let p95_duration = sorted_durations[p95_idx.min(durations.len() - 1)];

        let p99_idx = (durations.len() as f64 * 0.99) as usize;
        let p99_duration = sorted_durations[p99_idx.min(durations.len() - 1)];

        Self {
            total_duration,
            avg_duration,
            min_duration,
            max_duration,
            median_duration,
            p90_duration,
            p95_duration,
            p99_duration,
        }
    }
}

/// Metric collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricCollection {
    /// Metrics
    metrics: HashMap<String, Metric>,
}

impl MetricCollection {
    /// Create a new metric collection
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    /// Add a metric to the collection
    pub fn add_metric(&mut self, metric: Metric) {
        self.metrics.insert(metric.id.clone(), metric);
    }

    /// Get a metric by ID
    pub fn get_metric(&self, id: &str) -> Option<&Metric> {
        self.metrics.get(id)
    }

    /// Get all metrics
    pub fn metrics(&self) -> &HashMap<String, Metric> {
        &self.metrics
    }

    /// Get metrics by type
    pub fn metrics_by_type(&self, metric_type: MetricType) -> Vec<&Metric> {
        self.metrics
            .values()
            .filter(|m| m.metric_type == metric_type)
            .collect()
    }

    /// Get metrics by tag
    pub fn metrics_by_tag(&self, key: &str, value: &str) -> Vec<&Metric> {
        self.metrics
            .values()
            .filter(|m| m.tags.get(key).map_or(false, |v| v == value))
            .collect()
    }

    /// Calculate aggregations for all metrics
    pub fn calculate_aggregations(&mut self) {
        for metric in self.metrics.values_mut() {
            metric.calculate_aggregations();
        }
    }
}

impl Default for MetricCollection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric() {
        let mut metric = Metric::new("test-metric", 42.0)
            .with_name("Test Metric")
            .with_description("Test metric description")
            .with_type(MetricType::Counter)
            .with_unit("count")
            .with_tag("category", "test")
            .with_tag("priority", "high")
            .with_values(vec![1.0, 2.0, 3.0, 4.0, 5.0]);

        assert_eq!(metric.id(), "test-metric");
        assert_eq!(metric.name(), "Test Metric");
        assert_eq!(metric.description(), Some("Test metric description"));
        assert_eq!(metric.metric_type(), MetricType::Counter);
        assert_eq!(metric.value(), 42.0);
        assert_eq!(metric.unit(), Some("count"));
        assert_eq!(metric.tags().get("category"), Some(&"test".to_string()));
        assert_eq!(metric.tags().get("priority"), Some(&"high".to_string()));
        assert_eq!(metric.values(), Some(&[1.0, 2.0, 3.0, 4.0, 5.0][..]));

        metric.calculate_aggregations();

        assert_eq!(metric.aggregation(MetricAggregation::Sum), Some(15.0));
        assert_eq!(metric.aggregation(MetricAggregation::Average), Some(3.0));
        assert_eq!(metric.aggregation(MetricAggregation::Min), Some(1.0));
        assert_eq!(metric.aggregation(MetricAggregation::Max), Some(5.0));
        assert_eq!(metric.aggregation(MetricAggregation::Count), Some(5.0));
        assert_eq!(
            metric.aggregation(MetricAggregation::Percentile(50)),
            Some(3.0)
        );
        assert_eq!(
            metric.aggregation(MetricAggregation::Percentile(90)),
            Some(5.0)
        );
    }

    #[test]
    fn test_metric_collection() {
        let mut collection = MetricCollection::new();

        let metric1 = Metric::new("metric-1", 10.0)
            .with_type(MetricType::Counter)
            .with_tag("category", "test");

        let metric2 = Metric::new("metric-2", 20.0)
            .with_type(MetricType::Gauge)
            .with_tag("category", "test");

        let metric3 = Metric::new("metric-3", 30.0)
            .with_type(MetricType::Counter)
            .with_tag("category", "production");

        collection.add_metric(metric1);
        collection.add_metric(metric2);
        collection.add_metric(metric3);

        assert_eq!(collection.metrics().len(), 3);
        assert_eq!(collection.get_metric("metric-1").unwrap().value(), 10.0);
        assert_eq!(collection.get_metric("metric-2").unwrap().value(), 20.0);
        assert_eq!(collection.get_metric("metric-3").unwrap().value(), 30.0);

        let counters = collection.metrics_by_type(MetricType::Counter);
        assert_eq!(counters.len(), 2);
        assert!(counters.iter().any(|m| m.id() == "metric-1"));
        assert!(counters.iter().any(|m| m.id() == "metric-3"));

        let test_metrics = collection.metrics_by_tag("category", "test");
        assert_eq!(test_metrics.len(), 2);
        assert!(test_metrics.iter().any(|m| m.id() == "metric-1"));
        assert!(test_metrics.iter().any(|m| m.id() == "metric-2"));
    }
}
