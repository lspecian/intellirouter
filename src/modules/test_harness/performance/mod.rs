//! Performance Testing Framework
//!
//! This module provides utilities for performance testing, including
//! load testing, stress testing, and benchmarking.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::modules::test_harness::types::{
    TestCategory, TestContext, TestHarnessError, TestOutcome, TestResult,
};

/// Performance metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    /// Latency in milliseconds
    Latency,
    /// Throughput in requests per second
    Throughput,
    /// Error rate as a percentage
    ErrorRate,
    /// CPU usage as a percentage
    CpuUsage,
    /// Memory usage in megabytes
    MemoryUsage,
    /// Disk usage in megabytes
    DiskUsage,
    /// Network usage in megabytes
    NetworkUsage,
    /// Custom metric
    Custom(u32),
}

impl fmt::Display for MetricType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricType::Latency => write!(f, "Latency"),
            MetricType::Throughput => write!(f, "Throughput"),
            MetricType::ErrorRate => write!(f, "ErrorRate"),
            MetricType::CpuUsage => write!(f, "CpuUsage"),
            MetricType::MemoryUsage => write!(f, "MemoryUsage"),
            MetricType::DiskUsage => write!(f, "DiskUsage"),
            MetricType::NetworkUsage => write!(f, "NetworkUsage"),
            MetricType::Custom(id) => write!(f, "Custom({})", id),
        }
    }
}

/// Performance metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name
    pub name: String,
    /// Metric type
    pub metric_type: MetricType,
    /// Metric value
    pub value: f64,
    /// Metric unit
    pub unit: String,
    /// Metric timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Metric tags
    pub tags: HashMap<String, String>,
}

impl Metric {
    /// Create a new metric
    pub fn new(
        name: impl Into<String>,
        metric_type: MetricType,
        value: f64,
        unit: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            metric_type,
            value,
            unit: unit.into(),
            timestamp: chrono::Utc::now(),
            tags: HashMap::new(),
        }
    }

    /// Add a tag to the metric
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// Add multiple tags to the metric
    pub fn with_tags(mut self, tags: HashMap<String, String>) -> Self {
        self.tags.extend(tags);
        self
    }
}

/// Performance test parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestParams {
    /// Number of concurrent users
    pub concurrent_users: usize,
    /// Test duration
    pub duration: Duration,
    /// Ramp-up time
    pub ramp_up_time: Duration,
    /// Ramp-down time
    pub ramp_down_time: Duration,
    /// Think time between requests
    pub think_time: Duration,
    /// Request timeout
    pub request_timeout: Duration,
    /// Maximum requests per second
    pub max_requests_per_second: Option<usize>,
    /// Custom parameters
    pub custom_params: HashMap<String, serde_json::Value>,
}

impl Default for PerformanceTestParams {
    fn default() -> Self {
        Self {
            concurrent_users: 10,
            duration: Duration::from_secs(60),
            ramp_up_time: Duration::from_secs(10),
            ramp_down_time: Duration::from_secs(10),
            think_time: Duration::from_millis(100),
            request_timeout: Duration::from_secs(10),
            max_requests_per_second: None,
            custom_params: HashMap::new(),
        }
    }
}

impl PerformanceTestParams {
    /// Create new performance test parameters
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of concurrent users
    pub fn with_concurrent_users(mut self, concurrent_users: usize) -> Self {
        self.concurrent_users = concurrent_users;
        self
    }

    /// Set the test duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Set the ramp-up time
    pub fn with_ramp_up_time(mut self, ramp_up_time: Duration) -> Self {
        self.ramp_up_time = ramp_up_time;
        self
    }

    /// Set the ramp-down time
    pub fn with_ramp_down_time(mut self, ramp_down_time: Duration) -> Self {
        self.ramp_down_time = ramp_down_time;
        self
    }

    /// Set the think time between requests
    pub fn with_think_time(mut self, think_time: Duration) -> Self {
        self.think_time = think_time;
        self
    }

    /// Set the request timeout
    pub fn with_request_timeout(mut self, request_timeout: Duration) -> Self {
        self.request_timeout = request_timeout;
        self
    }

    /// Set the maximum requests per second
    pub fn with_max_requests_per_second(mut self, max_requests_per_second: usize) -> Self {
        self.max_requests_per_second = Some(max_requests_per_second);
        self
    }

    /// Add a custom parameter
    pub fn with_custom_param(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Result<Self, TestHarnessError> {
        let value = serde_json::to_value(value).map_err(TestHarnessError::SerializationError)?;
        self.custom_params.insert(key.into(), value);
        Ok(self)
    }
}

/// Performance test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestResult {
    /// Test name
    pub name: String,
    /// Test description
    pub description: Option<String>,
    /// Test parameters
    pub params: PerformanceTestParams,
    /// Test metrics
    pub metrics: Vec<Metric>,
    /// Test start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Test end time
    pub end_time: chrono::DateTime<chrono::Utc>,
    /// Test duration
    pub duration: Duration,
    /// Test outcome
    pub outcome: TestOutcome,
    /// Error message if the test failed
    pub error: Option<String>,
    /// Test summary
    pub summary: HashMap<String, f64>,
}

impl PerformanceTestResult {
    /// Create a new performance test result
    pub fn new(
        name: impl Into<String>,
        params: PerformanceTestParams,
        outcome: TestOutcome,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            name: name.into(),
            description: None,
            params,
            metrics: Vec::new(),
            start_time: now,
            end_time: now,
            duration: Duration::from_secs(0),
            outcome,
            error: None,
            summary: HashMap::new(),
        }
    }

    /// Set the test description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a metric
    pub fn with_metric(mut self, metric: Metric) -> Self {
        self.metrics.push(metric);
        self
    }

    /// Add multiple metrics
    pub fn with_metrics(mut self, metrics: Vec<Metric>) -> Self {
        self.metrics.extend(metrics);
        self
    }

    /// Set the test start time
    pub fn with_start_time(mut self, start_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.start_time = start_time;
        self
    }

    /// Set the test end time
    pub fn with_end_time(mut self, end_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.end_time = end_time;
        self
    }

    /// Set the test duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Set the error message
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Add a summary value
    pub fn with_summary_value(mut self, key: impl Into<String>, value: f64) -> Self {
        self.summary.insert(key.into(), value);
        self
    }

    /// Calculate summary statistics
    pub fn calculate_summary(&mut self) {
        // Group metrics by type
        let mut metrics_by_type: HashMap<MetricType, Vec<&Metric>> = HashMap::new();
        for metric in &self.metrics {
            metrics_by_type
                .entry(metric.metric_type)
                .or_default()
                .push(metric);
        }

        // Calculate summary statistics for each metric type
        for (metric_type, metrics) in metrics_by_type {
            match metric_type {
                MetricType::Latency => {
                    // Calculate min, max, avg, p50, p90, p95, p99 latency
                    let mut values: Vec<f64> = metrics.iter().map(|m| m.value).collect();
                    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                    if !values.is_empty() {
                        let min = values[0];
                        let max = values[values.len() - 1];
                        let avg = values.iter().sum::<f64>() / values.len() as f64;
                        let p50 = percentile(&values, 50.0);
                        let p90 = percentile(&values, 90.0);
                        let p95 = percentile(&values, 95.0);
                        let p99 = percentile(&values, 99.0);

                        self.summary.insert("latency_min".to_string(), min);
                        self.summary.insert("latency_max".to_string(), max);
                        self.summary.insert("latency_avg".to_string(), avg);
                        self.summary.insert("latency_p50".to_string(), p50);
                        self.summary.insert("latency_p90".to_string(), p90);
                        self.summary.insert("latency_p95".to_string(), p95);
                        self.summary.insert("latency_p99".to_string(), p99);
                    }
                }
                MetricType::Throughput => {
                    // Calculate min, max, avg throughput
                    let values: Vec<f64> = metrics.iter().map(|m| m.value).collect();
                    if !values.is_empty() {
                        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
                        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                        let avg = values.iter().sum::<f64>() / values.len() as f64;

                        self.summary.insert("throughput_min".to_string(), min);
                        self.summary.insert("throughput_max".to_string(), max);
                        self.summary.insert("throughput_avg".to_string(), avg);
                    }
                }
                MetricType::ErrorRate => {
                    // Calculate avg error rate
                    let values: Vec<f64> = metrics.iter().map(|m| m.value).collect();
                    if !values.is_empty() {
                        let avg = values.iter().sum::<f64>() / values.len() as f64;
                        self.summary.insert("error_rate_avg".to_string(), avg);
                    }
                }
                _ => {
                    // Calculate min, max, avg for other metrics
                    let values: Vec<f64> = metrics.iter().map(|m| m.value).collect();
                    if !values.is_empty() {
                        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
                        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                        let avg = values.iter().sum::<f64>() / values.len() as f64;

                        let metric_name = metric_type.to_string().to_lowercase();
                        self.summary.insert(format!("{}_min", metric_name), min);
                        self.summary.insert(format!("{}_max", metric_name), max);
                        self.summary.insert(format!("{}_avg", metric_name), avg);
                    }
                }
            }
        }

        // Calculate total requests, successful requests, failed requests
        let total_requests = self.metrics.len();
        let successful_requests = self
            .metrics
            .iter()
            .filter(|m| m.metric_type == MetricType::Latency && m.value >= 0.0)
            .count();
        let failed_requests = total_requests - successful_requests;

        self.summary
            .insert("total_requests".to_string(), total_requests as f64);
        self.summary.insert(
            "successful_requests".to_string(),
            successful_requests as f64,
        );
        self.summary
            .insert("failed_requests".to_string(), failed_requests as f64);

        // Calculate requests per second
        if self.duration.as_secs() > 0 {
            let rps = total_requests as f64 / self.duration.as_secs_f64();
            self.summary.insert("requests_per_second".to_string(), rps);
        }
    }
}

/// Calculate percentile value
fn percentile(values: &[f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let index = (p / 100.0 * values.len() as f64).ceil() as usize - 1;
    values[index.min(values.len() - 1)]
}

/// Performance test interface
#[async_trait]
pub trait PerformanceTest: Send + Sync {
    /// Get the test name
    fn name(&self) -> &str;

    /// Get the test description
    fn description(&self) -> Option<&str>;

    /// Execute the test with the given parameters
    async fn execute(
        &self,
        params: &PerformanceTestParams,
    ) -> Result<PerformanceTestResult, TestHarnessError>;
}

/// Performance test builder
pub struct PerformanceTestBuilder {
    /// Test name
    name: String,
    /// Test description
    description: Option<String>,
    /// Test execution function
    execute_fn: Option<
        Box<
            dyn Fn(
                    &PerformanceTestParams,
                )
                    -> BoxFuture<'static, Result<PerformanceTestResult, TestHarnessError>>
                + Send
                + Sync,
        >,
    >,
}

impl PerformanceTestBuilder {
    /// Create a new performance test builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            execute_fn: None,
        }
    }

    /// Set the test description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the test execution function
    pub fn with_execute_fn(
        mut self,
        execute_fn: impl Fn(
                &PerformanceTestParams,
            ) -> BoxFuture<'static, Result<PerformanceTestResult, TestHarnessError>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        self.execute_fn = Some(Box::new(execute_fn));
        self
    }

    /// Build the performance test
    pub fn build(self) -> Box<dyn PerformanceTest> {
        let execute_fn = self.execute_fn.unwrap_or_else(|| {
            Box::new(|params| {
                async move {
                    Ok(PerformanceTestResult::new(
                        self.name.clone(),
                        params.clone(),
                        TestOutcome::Passed,
                    ))
                }
                .boxed()
            })
        });

        Box::new(BasicPerformanceTest {
            name: self.name,
            description: self.description,
            execute_fn,
        })
    }
}

/// Basic performance test implementation
struct BasicPerformanceTest {
    /// Test name
    name: String,
    /// Test description
    description: Option<String>,
    /// Test execution function
    execute_fn: Box<
        dyn Fn(
                &PerformanceTestParams,
            ) -> BoxFuture<'static, Result<PerformanceTestResult, TestHarnessError>>
            + Send
            + Sync,
    >,
}

#[async_trait]
impl PerformanceTest for BasicPerformanceTest {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    async fn execute(
        &self,
        params: &PerformanceTestParams,
    ) -> Result<PerformanceTestResult, TestHarnessError> {
        (self.execute_fn)(params).await
    }
}

/// Performance test suite
#[derive(Debug)]
pub struct PerformanceTestSuite {
    /// Suite name
    pub name: String,
    /// Suite description
    pub description: Option<String>,
    /// Test parameters
    pub params: Vec<PerformanceTestParams>,
    /// Tests to run
    pub tests: Vec<Box<dyn PerformanceTest>>,
    /// Whether to run tests in parallel
    pub parallel: bool,
    /// Whether to fail fast on the first test failure
    pub fail_fast: bool,
}

impl PerformanceTestSuite {
    /// Create a new performance test suite
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            params: Vec::new(),
            tests: Vec::new(),
            parallel: false,
            fail_fast: false,
        }
    }

    /// Set the suite description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add test parameters
    pub fn with_params(mut self, params: PerformanceTestParams) -> Self {
        self.params.push(params);
        self
    }

    /// Add multiple test parameters
    pub fn with_multiple_params(mut self, params: Vec<PerformanceTestParams>) -> Self {
        self.params.extend(params);
        self
    }

    /// Add a test
    pub fn with_test(mut self, test: Box<dyn PerformanceTest>) -> Self {
        self.tests.push(test);
        self
    }

    /// Add multiple tests
    pub fn with_tests(mut self, tests: Vec<Box<dyn PerformanceTest>>) -> Self {
        self.tests.extend(tests);
        self
    }

    /// Set whether to run tests in parallel
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Set whether to fail fast on the first test failure
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Execute the test suite
    pub async fn execute(&self) -> Result<Vec<PerformanceTestResult>, TestHarnessError> {
        info!("Executing performance test suite: {}", self.name);

        let mut results = Vec::new();

        for params in &self.params {
            info!(
                "Running performance tests with {} concurrent users",
                params.concurrent_users
            );

            for test in &self.tests {
                info!("Executing test: {}", test.name());

                let start_time = Instant::now();
                let result = match test.execute(params).await {
                    Ok(mut result) => {
                        // Calculate summary statistics
                        result.calculate_summary();
                        result
                    }
                    Err(e) => {
                        error!("Test failed: {}: {}", test.name(), e);
                        let result = PerformanceTestResult::new(
                            test.name(),
                            params.clone(),
                            TestOutcome::Failed,
                        )
                        .with_error(format!("Test execution error: {}", e))
                        .with_duration(start_time.elapsed());

                        results.push(result.clone());

                        if self.fail_fast {
                            return Ok(results);
                        } else {
                            continue;
                        }
                    }
                };

                // Check result
                if result.outcome == TestOutcome::Failed {
                    if self.fail_fast {
                        results.push(result);
                        return Ok(results);
                    }
                }

                results.push(result);
            }
        }

        Ok(results)
    }
}

/// Create a test case from a performance test suite
pub fn create_test_case_from_performance_suite(
    suite: PerformanceTestSuite,
) -> crate::modules::test_harness::types::TestCase {
    let suite_name = suite.name.clone();

    crate::modules::test_harness::types::TestCase::new(
        TestContext::new(TestCategory::Performance, suite_name.clone()),
        move |_| {
            let suite = suite.clone();
            async move {
                let start_time = Instant::now();
                let start_datetime = chrono::Utc::now();

                let results = suite.execute().await?;

                let duration = start_time.elapsed();
                let end_datetime = chrono::Utc::now();

                let all_passed = results.iter().all(|r| r.outcome == TestOutcome::Passed);
                let outcome = if all_passed {
                    TestOutcome::Passed
                } else {
                    TestOutcome::Failed
                };

                let mut test_result =
                    TestResult::new(&suite_name, TestCategory::Performance, outcome)
                        .with_start_time(start_datetime)
                        .with_end_time(end_datetime)
                        .with_duration(duration);

                // Add performance test results as custom data
                test_result = test_result
                    .with_custom_data("performance_test_results", &results)
                    .map_err(|e| {
                        TestHarnessError::ExecutionError(format!(
                            "Failed to add performance test results: {}",
                            e
                        ))
                    })?;

                // Add error message if any tests failed
                if !all_passed {
                    let failed_tests: Vec<String> = results
                        .iter()
                        .filter(|r| r.outcome == TestOutcome::Failed)
                        .map(|r| {
                            if let Some(error) = &r.error {
                                format!("{}: {}", r.name, error)
                            } else {
                                format!("{}: Failed", r.name)
                            }
                        })
                        .collect();

                    test_result = test_result.with_error(format!(
                        "Performance test suite failed with {} failed tests: {}",
                        failed_tests.len(),
                        failed_tests.join(", ")
                    ));
                }

                Ok(test_result)
            }
            .boxed()
        },
    )
}

/// Load generator for performance testing
pub struct LoadGenerator {
    /// Load generator name
    name: String,
    /// Load generator description
    description: Option<String>,
    /// Metrics collector
    metrics: Arc<Mutex<Vec<Metric>>>,
}

impl LoadGenerator {
    /// Create a new load generator
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            metrics: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Set the load generator description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Generate load with the given parameters
    pub async fn generate_load<F, Fut>(
        &self,
        params: &PerformanceTestParams,
        request_fn: F,
    ) -> Result<Vec<Metric>, TestHarnessError>
    where
        F: Fn() -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<Duration, TestHarnessError>> + Send + 'static,
    {
        info!(
            "Generating load with {} concurrent users for {:?}",
            params.concurrent_users, params.duration
        );

        let start_time = Instant::now();
        let metrics = Arc::clone(&self.metrics);

        // Create a rate limiter if max_requests_per_second is specified
        let rate_limiter = if let Some(max_rps) = params.max_requests_per_second {
            Some(Arc::new(tokio::sync::Semaphore::new(max_rps)))
        } else {
            None
        };

        // Create a vector of futures for each user
        let mut user_futures = Vec::new();
        for user_id in 0..params.concurrent_users {
            let request_fn = request_fn.clone();
            let metrics = Arc::clone(&metrics);
            let rate_limiter = rate_limiter.clone();
            let params = params.clone();

            let user_future = tokio::spawn(async move {
                // Calculate start time based on ramp-up
                let user_start_delay =
                    if params.concurrent_users > 1 && params.ramp_up_time.as_millis() > 0 {
                        let ramp_up_per_user =
                            params.ramp_up_time.as_millis() / (params.concurrent_users as u128 - 1);
                        Duration::from_millis((ramp_up_per_user * user_id as u128) as u64)
                    } else {
                        Duration::from_millis(0)
                    };

                // Wait for user start time
                if user_start_delay.as_millis() > 0 {
                    tokio::time::sleep(user_start_delay).await;
                }

                let user_start_time = Instant::now();
                let mut request_count = 0;
                let mut error_count = 0;

                // Run until test duration is reached
                while user_start_time.elapsed() < params.duration {
                    // Acquire rate limiter permit if needed
                    let _permit = if let Some(limiter) = &rate_limiter {
                        Some(limiter.acquire().await.unwrap())
                    } else {
                        None
                    };

                    // Execute the request
                    let request_start = Instant::now();
                    let result =
                        match tokio::time::timeout(params.request_timeout, request_fn()).await {
                            Ok(Ok(latency)) => {
                                // Record successful request
                                let metric = Metric::new(
                                    "request_latency",
                                    MetricType::Latency,
                                    latency.as_millis() as f64,
                                    "ms",
                                )
                                .with_tag("user_id", user_id.to_string())
                                .with_tag("request_id", request_count.to_string());

                                let mut metrics = metrics.lock().await;
                                metrics.push(metric);

                                request_count += 1;
                                Ok(())
                            }
                            Ok(Err(e)) => {
                                // Record failed request
                                let metric =
                                    Metric::new("request_error", MetricType::Latency, -1.0, "ms")
                                        .with_tag("user_id", user_id.to_string())
                                        .with_tag("request_id", request_count.to_string())
                                        .with_tag("error", e.to_string());

                                let mut metrics = metrics.lock().await;
                                metrics.push(metric);

                                request_count += 1;
                                error_count += 1;
                                Err(e)
                            }
                            Err(_) => {
                                // Record timeout
                                let metric =
                                    Metric::new("request_timeout", MetricType::Latency, -1.0, "ms")
                                        .with_tag("user_id", user_id.to_string())
                                        .with_tag("request_id", request_count.to_string())
                                        .with_tag("error", "Request timed out");

                                let mut metrics = metrics.lock().await;
                                metrics.push(metric);

                                request_count += 1;
                                error_count += 1;
                                Err(TestHarnessError::TimeoutError(
                                    "Request timed out".to_string(),
                                ))
                            }
                        };

                    // Think time between requests
                    if params.think_time.as_millis() > 0 {
                        tokio::time::sleep(params.think_time).await;
                    }
                }

                // Record user metrics
                let user_duration = user_start_time.elapsed();
                let throughput = if user_duration.as_secs_f64() > 0.0 {
                    request_count as f64 / user_duration.as_secs_f64()
                } else {
                    0.0
                };

                let error_rate = if request_count > 0 {
                    error_count as f64 / request_count as f64 * 100.0
                } else {
                    0.0
                };

                let throughput_metric =
                    Metric::new("user_throughput", MetricType::Throughput, throughput, "rps")
                        .with_tag("user_id", user_id.to_string());

                let error_rate_metric =
                    Metric::new("user_error_rate", MetricType::ErrorRate, error_rate, "%")
                        .with_tag("user_id", user_id.to_string());

                let mut metrics = metrics.lock().await;
                metrics.push(throughput_metric);
                metrics.push(error_rate_metric);
            });

            user_futures.push(user_future);
        }

        // Wait for all users to complete
        futures::future::join_all(user_futures).await;

        // Calculate overall metrics
        let test_duration = start_time.elapsed();
        let metrics = metrics.lock().await;
        let metrics_vec = metrics.clone();

        // Calculate overall throughput
        let total_requests = metrics_vec
            .iter()
            .filter(|m| m.metric_type == MetricType::Latency)
            .count();

        let overall_throughput = if test_duration.as_secs_f64() > 0.0 {
            total_requests as f64 / test_duration.as_secs_f64()
        } else {
            0.0
        };

        let overall_throughput_metric = Metric::new(
            "overall_throughput",
            MetricType::Throughput,
            overall_throughput,
            "rps",
        );

        // Calculate overall error rate
        let error_count = metrics_vec
            .iter()
            .filter(|m| m.metric_type == MetricType::Latency && m.value < 0.0)
            .count();

        let overall_error_rate = if total_requests > 0 {
            error_count as f64 / total_requests as f64 * 100.0
        } else {
            0.0
        };

        let overall_error_rate_metric = Metric::new(
            "overall_error_rate",
            MetricType::ErrorRate,
            overall_error_rate,
            "%",
        );

        // Add overall metrics to the result
        let mut result = metrics_vec.clone();
        result.push(overall_throughput_metric);
        result.push(overall_error_rate_metric);

        Ok(result)
    }
}

/// Create a new performance test
pub fn create_performance_test(name: impl Into<String>) -> PerformanceTestBuilder {
    PerformanceTestBuilder::new(name)
}

/// Create new performance test parameters
pub fn create_performance_test_params() -> PerformanceTestParams {
    PerformanceTestParams::new()
}

/// Create a new performance test suite
pub fn create_performance_test_suite(name: impl Into<String>) -> PerformanceTestSuite {
    PerformanceTestSuite::new(name)
}

/// Create a new load generator
pub fn create_load_generator(name: impl Into<String>) -> LoadGenerator {
    LoadGenerator::new(name)
}

/// Create a new metric
pub fn create_metric(
    name: impl Into<String>,
    metric_type: MetricType,
    value: f64,
    unit: impl Into<String>,
) -> Metric {
    Metric::new(name, metric_type, value, unit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future;

    #[tokio::test]
    async fn test_performance_test_params() {
        let params = create_performance_test_params()
            .with_concurrent_users(20)
            .with_duration(Duration::from_secs(30))
            .with_ramp_up_time(Duration::from_secs(5))
            .with_ramp_down_time(Duration::from_secs(5))
            .with_think_time(Duration::from_millis(50))
            .with_request_timeout(Duration::from_secs(5))
            .with_max_requests_per_second(100);

        assert_eq!(params.concurrent_users, 20);
        assert_eq!(params.duration, Duration::from_secs(30));
        assert_eq!(params.ramp_up_time, Duration::from_secs(5));
        assert_eq!(params.ramp_down_time, Duration::from_secs(5));
        assert_eq!(params.think_time, Duration::from_millis(50));
        assert_eq!(params.request_timeout, Duration::from_secs(5));
        assert_eq!(params.max_requests_per_second, Some(100));
    }

    #[tokio::test]
    async fn test_performance_test() {
        // Create a performance test
        let test = create_performance_test("test_performance")
            .with_description("Test performance test")
            .with_execute_fn(|params| {
                async move {
                    // Create some test metrics
                    let metrics = vec![
                        create_metric("latency", MetricType::Latency, 100.0, "ms"),
                        create_metric("latency", MetricType::Latency, 200.0, "ms"),
                        create_metric("latency", MetricType::Latency, 300.0, "ms"),
                        create_metric("throughput", MetricType::Throughput, 10.0, "rps"),
                    ];

                    let mut result = PerformanceTestResult::new(
                        "test_performance",
                        params.clone(),
                        TestOutcome::Passed,
                    )
                    .with_metrics(metrics)
                    .with_duration(Duration::from_secs(1));

                    result.calculate_summary();

                    Ok(result)
                }
                .boxed()
            })
            .build();

        // Create test parameters
        let params = create_performance_test_params()
            .with_concurrent_users(1)
            .with_duration(Duration::from_secs(1));

        // Execute the test
        let result = test.execute(&params).await.unwrap();

        // Check the result
        assert_eq!(result.name, "test_performance");
        assert_eq!(result.outcome, TestOutcome::Passed);
        assert_eq!(result.metrics.len(), 4);

        // Check summary statistics
        assert!(result.summary.contains_key("latency_min"));
        assert!(result.summary.contains_key("latency_max"));
        assert!(result.summary.contains_key("latency_avg"));
        assert!(result.summary.contains_key("throughput_avg"));
    }

    #[tokio::test]
    async fn test_load_generator() {
        // Create a load generator
        let load_generator = create_load_generator("test_load_generator");

        // Create test parameters
        let params = create_performance_test_params()
            .with_concurrent_users(2)
            .with_duration(Duration::from_millis(100))
            .with_think_time(Duration::from_millis(10));

        // Generate load
        let metrics = load_generator
            .generate_load(&params, || async {
                // Simulate a request with random latency
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok(Duration::from_millis(10))
            })
            .await
            .unwrap();

        // Check that metrics were collected
        assert!(!metrics.is_empty());
        assert!(metrics.iter().any(|m| m.metric_type == MetricType::Latency));
        assert!(metrics
            .iter()
            .any(|m| m.metric_type == MetricType::Throughput));
        assert!(metrics
            .iter()
            .any(|m| m.metric_type == MetricType::ErrorRate));
    }
}
