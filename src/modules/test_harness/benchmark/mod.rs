//! Benchmark Module
//!
//! This module provides functionality for performance benchmarking of IntelliRouter components.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::metrics::{Metric, MetricCollection, MetricType, TimeMetrics};
use super::reporting::{TestResult, TestRun, TestStatus};
use crate::modules::test_harness::types::TestHarnessError;

/// Benchmark type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BenchmarkType {
    /// Throughput benchmark
    Throughput,
    /// Latency benchmark
    Latency,
    /// Resource usage benchmark
    ResourceUsage,
    /// Scalability benchmark
    Scalability,
    /// Stress benchmark
    Stress,
    /// Endurance benchmark
    Endurance,
}

impl fmt::Display for BenchmarkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BenchmarkType::Throughput => write!(f, "throughput"),
            BenchmarkType::Latency => write!(f, "latency"),
            BenchmarkType::ResourceUsage => write!(f, "resource_usage"),
            BenchmarkType::Scalability => write!(f, "scalability"),
            BenchmarkType::Stress => write!(f, "stress"),
            BenchmarkType::Endurance => write!(f, "endurance"),
        }
    }
}

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Benchmark ID
    pub id: String,
    /// Benchmark name
    pub name: String,
    /// Benchmark description
    pub description: Option<String>,
    /// Benchmark type
    pub benchmark_type: BenchmarkType,
    /// Benchmark duration
    pub duration: Duration,
    /// Benchmark warmup duration
    pub warmup_duration: Duration,
    /// Benchmark cooldown duration
    pub cooldown_duration: Duration,
    /// Benchmark iterations
    pub iterations: usize,
    /// Benchmark concurrency
    pub concurrency: usize,
    /// Benchmark rate limit (requests per second)
    pub rate_limit: Option<u64>,
    /// Benchmark timeout
    pub timeout: Duration,
    /// Benchmark parameters
    pub parameters: HashMap<String, String>,
    /// Benchmark tags
    pub tags: Vec<String>,
}

impl BenchmarkConfig {
    /// Create a new benchmark configuration
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        benchmark_type: BenchmarkType,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            benchmark_type,
            duration: Duration::from_secs(60),
            warmup_duration: Duration::from_secs(10),
            cooldown_duration: Duration::from_secs(10),
            iterations: 1,
            concurrency: 1,
            rate_limit: None,
            timeout: Duration::from_secs(30),
            parameters: HashMap::new(),
            tags: Vec::new(),
        }
    }

    /// Set the benchmark description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the benchmark duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Set the benchmark warmup duration
    pub fn with_warmup_duration(mut self, warmup_duration: Duration) -> Self {
        self.warmup_duration = warmup_duration;
        self
    }

    /// Set the benchmark cooldown duration
    pub fn with_cooldown_duration(mut self, cooldown_duration: Duration) -> Self {
        self.cooldown_duration = cooldown_duration;
        self
    }

    /// Set the benchmark iterations
    pub fn with_iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations;
        self
    }

    /// Set the benchmark concurrency
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency;
        self
    }

    /// Set the benchmark rate limit
    pub fn with_rate_limit(mut self, rate_limit: u64) -> Self {
        self.rate_limit = Some(rate_limit);
        self
    }

    /// Set the benchmark timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Add a parameter to the benchmark
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    /// Add multiple parameters to the benchmark
    pub fn with_parameters(
        mut self,
        parameters: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (key, value) in parameters {
            self.parameters.insert(key.into(), value.into());
        }
        self
    }

    /// Add a tag to the benchmark
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags to the benchmark
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for tag in tags {
            self.tags.push(tag.into());
        }
        self
    }
}

/// Benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Benchmark configuration
    pub config: BenchmarkConfig,
    /// Benchmark start time
    pub start_time: DateTime<Utc>,
    /// Benchmark end time
    pub end_time: DateTime<Utc>,
    /// Benchmark total duration
    pub total_duration: Duration,
    /// Benchmark actual duration (excluding warmup and cooldown)
    pub actual_duration: Duration,
    /// Benchmark total operations
    pub total_operations: u64,
    /// Benchmark successful operations
    pub successful_operations: u64,
    /// Benchmark failed operations
    pub failed_operations: u64,
    /// Benchmark operation durations
    pub operation_durations: Vec<Duration>,
    /// Benchmark throughput (operations per second)
    pub throughput: f64,
    /// Benchmark latency metrics
    pub latency: TimeMetrics,
    /// Benchmark error rate
    pub error_rate: f64,
    /// Benchmark metrics
    pub metrics: MetricCollection,
    /// Benchmark errors
    pub errors: Vec<String>,
}

impl BenchmarkResult {
    /// Create a new benchmark result
    pub fn new(config: BenchmarkConfig) -> Self {
        let now = Utc::now();

        Self {
            config,
            start_time: now,
            end_time: now,
            total_duration: Duration::from_secs(0),
            actual_duration: Duration::from_secs(0),
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            operation_durations: Vec::new(),
            throughput: 0.0,
            latency: TimeMetrics::new(&[]),
            error_rate: 0.0,
            metrics: MetricCollection::new(),
            errors: Vec::new(),
        }
    }

    /// Set the benchmark start time
    pub fn with_start_time(mut self, start_time: DateTime<Utc>) -> Self {
        self.start_time = start_time;
        self
    }

    /// Set the benchmark end time
    pub fn with_end_time(mut self, end_time: DateTime<Utc>) -> Self {
        self.end_time = end_time;
        self
    }

    /// Set the benchmark total duration
    pub fn with_total_duration(mut self, total_duration: Duration) -> Self {
        self.total_duration = total_duration;
        self
    }

    /// Set the benchmark actual duration
    pub fn with_actual_duration(mut self, actual_duration: Duration) -> Self {
        self.actual_duration = actual_duration;
        self
    }

    /// Set the benchmark total operations
    pub fn with_total_operations(mut self, total_operations: u64) -> Self {
        self.total_operations = total_operations;
        self
    }

    /// Set the benchmark successful operations
    pub fn with_successful_operations(mut self, successful_operations: u64) -> Self {
        self.successful_operations = successful_operations;
        self
    }

    /// Set the benchmark failed operations
    pub fn with_failed_operations(mut self, failed_operations: u64) -> Self {
        self.failed_operations = failed_operations;
        self
    }

    /// Set the benchmark operation durations
    pub fn with_operation_durations(mut self, operation_durations: Vec<Duration>) -> Self {
        self.operation_durations = operation_durations;
        self
    }

    /// Add an operation duration
    pub fn add_operation_duration(&mut self, duration: Duration) {
        self.operation_durations.push(duration);
    }

    /// Add a successful operation
    pub fn add_successful_operation(&mut self, duration: Duration) {
        self.total_operations += 1;
        self.successful_operations += 1;
        self.operation_durations.push(duration);
    }

    /// Add a failed operation
    pub fn add_failed_operation(&mut self, duration: Duration, error: impl Into<String>) {
        self.total_operations += 1;
        self.failed_operations += 1;
        self.operation_durations.push(duration);
        self.errors.push(error.into());
    }

    /// Add a metric to the benchmark
    pub fn add_metric(&mut self, metric: Metric) {
        self.metrics.add_metric(metric);
    }

    /// Calculate benchmark metrics
    pub fn calculate_metrics(&mut self) {
        // Calculate throughput
        if !self.actual_duration.is_zero() {
            self.throughput =
                self.successful_operations as f64 / self.actual_duration.as_secs_f64();
        }

        // Calculate latency metrics
        self.latency = TimeMetrics::new(&self.operation_durations);

        // Calculate error rate
        if self.total_operations > 0 {
            self.error_rate = self.failed_operations as f64 / self.total_operations as f64;
        }

        // Add metrics
        self.metrics.add_metric(
            Metric::new("throughput", self.throughput)
                .with_type(MetricType::Gauge)
                .with_unit("ops/sec")
                .with_tag("benchmark_type", self.config.benchmark_type.to_string()),
        );

        self.metrics.add_metric(
            Metric::new("error_rate", self.error_rate)
                .with_type(MetricType::Gauge)
                .with_unit("ratio")
                .with_tag("benchmark_type", self.config.benchmark_type.to_string()),
        );

        self.metrics.add_metric(
            Metric::new("total_operations", self.total_operations as f64)
                .with_type(MetricType::Counter)
                .with_unit("count")
                .with_tag("benchmark_type", self.config.benchmark_type.to_string()),
        );

        self.metrics.add_metric(
            Metric::new("successful_operations", self.successful_operations as f64)
                .with_type(MetricType::Counter)
                .with_unit("count")
                .with_tag("benchmark_type", self.config.benchmark_type.to_string()),
        );

        self.metrics.add_metric(
            Metric::new("failed_operations", self.failed_operations as f64)
                .with_type(MetricType::Counter)
                .with_unit("count")
                .with_tag("benchmark_type", self.config.benchmark_type.to_string()),
        );

        self.metrics.add_metric(
            Metric::new(
                "latency_avg",
                self.latency.avg_duration.as_secs_f64() * 1000.0,
            )
            .with_type(MetricType::Gauge)
            .with_unit("ms")
            .with_tag("benchmark_type", self.config.benchmark_type.to_string()),
        );

        self.metrics.add_metric(
            Metric::new(
                "latency_p50",
                self.latency.median_duration.as_secs_f64() * 1000.0,
            )
            .with_type(MetricType::Gauge)
            .with_unit("ms")
            .with_tag("benchmark_type", self.config.benchmark_type.to_string()),
        );

        self.metrics.add_metric(
            Metric::new(
                "latency_p90",
                self.latency.p90_duration.as_secs_f64() * 1000.0,
            )
            .with_type(MetricType::Gauge)
            .with_unit("ms")
            .with_tag("benchmark_type", self.config.benchmark_type.to_string()),
        );

        self.metrics.add_metric(
            Metric::new(
                "latency_p95",
                self.latency.p95_duration.as_secs_f64() * 1000.0,
            )
            .with_type(MetricType::Gauge)
            .with_unit("ms")
            .with_tag("benchmark_type", self.config.benchmark_type.to_string()),
        );

        self.metrics.add_metric(
            Metric::new(
                "latency_p99",
                self.latency.p99_duration.as_secs_f64() * 1000.0,
            )
            .with_type(MetricType::Gauge)
            .with_unit("ms")
            .with_tag("benchmark_type", self.config.benchmark_type.to_string()),
        );
    }

    /// Convert to a test result
    pub fn to_test_result(&self) -> TestResult {
        let status = if self.error_rate < 0.01 {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        };

        let mut result = TestResult::new(&self.config.id, &self.config.name, status)
            .with_description(self.config.description.clone().unwrap_or_default())
            .with_duration(self.total_duration)
            .with_start_time(self.start_time)
            .with_end_time(self.end_time)
            .with_metadata("benchmark_type", self.config.benchmark_type.to_string())
            .with_metadata("throughput", format!("{:.2} ops/sec", self.throughput))
            .with_metadata("error_rate", format!("{:.2}%", self.error_rate * 100.0))
            .with_metadata(
                "latency_avg",
                format!("{:.2} ms", self.latency.avg_duration.as_secs_f64() * 1000.0),
            )
            .with_metadata(
                "latency_p50",
                format!(
                    "{:.2} ms",
                    self.latency.median_duration.as_secs_f64() * 1000.0
                ),
            )
            .with_metadata(
                "latency_p90",
                format!("{:.2} ms", self.latency.p90_duration.as_secs_f64() * 1000.0),
            )
            .with_metadata(
                "latency_p95",
                format!("{:.2} ms", self.latency.p95_duration.as_secs_f64() * 1000.0),
            )
            .with_metadata(
                "latency_p99",
                format!("{:.2} ms", self.latency.p99_duration.as_secs_f64() * 1000.0),
            );

        // Add tags
        for tag in &self.config.tags {
            result = result.with_tag(tag);
        }

        // Add parameters as metadata
        for (key, value) in &self.config.parameters {
            result = result.with_metadata(key, value);
        }

        // Add errors as output
        if !self.errors.is_empty() {
            let errors = self.errors.join("\n");
            result = result.with_output(errors);
        }

        result
    }
}

/// Benchmark runner
pub struct BenchmarkRunner {
    /// Benchmark configuration
    config: BenchmarkConfig,
    /// Benchmark function
    benchmark_fn: Box<dyn Fn() -> Result<Duration, String> + Send + Sync>,
}

impl BenchmarkRunner {
    /// Create a new benchmark runner
    pub fn new(
        config: BenchmarkConfig,
        benchmark_fn: impl Fn() -> Result<Duration, String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            config,
            benchmark_fn: Box::new(benchmark_fn),
        }
    }

    /// Run the benchmark
    pub async fn run(&self) -> Result<BenchmarkResult, TestHarnessError> {
        info!(
            "Starting benchmark: {} ({})",
            self.config.name, self.config.id
        );

        let start_time = Utc::now();
        let benchmark_start = Instant::now();

        // Create the benchmark result
        let mut result = BenchmarkResult::new(self.config.clone()).with_start_time(start_time);

        // Warmup phase
        if !self.config.warmup_duration.is_zero() {
            info!("Warmup phase: {:?}", self.config.warmup_duration);

            let warmup_end = benchmark_start + self.config.warmup_duration;

            while Instant::now() < warmup_end {
                let _ = (self.benchmark_fn)();
            }
        }

        // Benchmark phase
        info!("Benchmark phase: {:?}", self.config.duration);

        let benchmark_end = benchmark_start + self.config.warmup_duration + self.config.duration;
        let actual_start = Instant::now();

        while Instant::now() < benchmark_end {
            match (self.benchmark_fn)() {
                Ok(duration) => {
                    result.add_successful_operation(duration);
                }
                Err(error) => {
                    result.add_failed_operation(Duration::from_secs(0), error);
                }
            }
        }

        let actual_end = Instant::now();
        result.actual_duration = actual_end - actual_start;

        // Cooldown phase
        if !self.config.cooldown_duration.is_zero() {
            info!("Cooldown phase: {:?}", self.config.cooldown_duration);

            let cooldown_end = benchmark_end + self.config.cooldown_duration;

            while Instant::now() < cooldown_end {
                let _ = (self.benchmark_fn)();
            }
        }

        // Finalize the benchmark
        let end_time = Utc::now();
        let total_duration = benchmark_start.elapsed();

        result.end_time = end_time;
        result.total_duration = total_duration;

        // Calculate metrics
        result.calculate_metrics();

        info!(
            "Benchmark completed: {} ({})",
            self.config.name, self.config.id
        );
        info!("  Throughput: {:.2} ops/sec", result.throughput);
        info!("  Error rate: {:.2}%", result.error_rate * 100.0);
        info!(
            "  Latency (avg): {:.2} ms",
            result.latency.avg_duration.as_secs_f64() * 1000.0
        );
        info!(
            "  Latency (p50): {:.2} ms",
            result.latency.median_duration.as_secs_f64() * 1000.0
        );
        info!(
            "  Latency (p90): {:.2} ms",
            result.latency.p90_duration.as_secs_f64() * 1000.0
        );
        info!(
            "  Latency (p95): {:.2} ms",
            result.latency.p95_duration.as_secs_f64() * 1000.0
        );
        info!(
            "  Latency (p99): {:.2} ms",
            result.latency.p99_duration.as_secs_f64() * 1000.0
        );

        Ok(result)
    }
}

/// Benchmark suite
pub struct BenchmarkSuite {
    /// Suite ID
    id: String,
    /// Suite name
    name: String,
    /// Suite description
    description: Option<String>,
    /// Benchmarks
    benchmarks: Vec<BenchmarkConfig>,
    /// Suite tags
    tags: Vec<String>,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            benchmarks: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// Set the suite description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a benchmark to the suite
    pub fn with_benchmark(mut self, benchmark: BenchmarkConfig) -> Self {
        self.benchmarks.push(benchmark);
        self
    }

    /// Add multiple benchmarks to the suite
    pub fn with_benchmarks(
        mut self,
        benchmarks: impl IntoIterator<Item = BenchmarkConfig>,
    ) -> Self {
        self.benchmarks.extend(benchmarks);
        self
    }

    /// Add a tag to the suite
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags to the suite
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for tag in tags {
            self.tags.push(tag.into());
        }
        self
    }

    /// Get the suite ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the suite name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the suite description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get the benchmarks
    pub fn benchmarks(&self) -> &[BenchmarkConfig] {
        &self.benchmarks
    }

    /// Get the suite tags
    pub fn tags(&self) -> &[String] {
        &self.tags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_benchmark_config() {
        let config = BenchmarkConfig::new("bench-1", "Test Benchmark", BenchmarkType::Throughput)
            .with_description("Test benchmark description")
            .with_duration(Duration::from_secs(30))
            .with_warmup_duration(Duration::from_secs(5))
            .with_cooldown_duration(Duration::from_secs(5))
            .with_iterations(3)
            .with_concurrency(10)
            .with_rate_limit(100)
            .with_timeout(Duration::from_secs(10))
            .with_parameter("param1", "value1")
            .with_parameter("param2", "value2")
            .with_tag("tag1")
            .with_tag("tag2");

        assert_eq!(config.id, "bench-1");
        assert_eq!(config.name, "Test Benchmark");
        assert_eq!(
            config.description,
            Some("Test benchmark description".to_string())
        );
        assert_eq!(config.benchmark_type, BenchmarkType::Throughput);
        assert_eq!(config.duration, Duration::from_secs(30));
        assert_eq!(config.warmup_duration, Duration::from_secs(5));
        assert_eq!(config.cooldown_duration, Duration::from_secs(5));
        assert_eq!(config.iterations, 3);
        assert_eq!(config.concurrency, 10);
        assert_eq!(config.rate_limit, Some(100));
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.parameters.get("param1"), Some(&"value1".to_string()));
        assert_eq!(config.parameters.get("param2"), Some(&"value2".to_string()));
        assert_eq!(config.tags, vec!["tag1".to_string(), "tag2".to_string()]);
    }

    #[tokio::test]
    async fn test_benchmark_runner() {
        let config = BenchmarkConfig::new("bench-1", "Test Benchmark", BenchmarkType::Throughput)
            .with_duration(Duration::from_millis(100))
            .with_warmup_duration(Duration::from_millis(10))
            .with_cooldown_duration(Duration::from_millis(10));

        let benchmark_fn = || {
            // Simulate some work
            thread::sleep(Duration::from_millis(1));
            Ok(Duration::from_millis(1))
        };

        let runner = BenchmarkRunner::new(config, benchmark_fn);
        let result = runner.run().await.unwrap();

        assert!(result.total_operations > 0);
        assert!(result.successful_operations > 0);
        assert_eq!(result.failed_operations, 0);
        assert!(result.throughput > 0.0);
        assert_eq!(result.error_rate, 0.0);
    }
}
