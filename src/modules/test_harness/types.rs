//! Test Harness Types
//!
//! This module defines the core data structures and enums used by the test harness.

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Test categories supported by the test harness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TestCategory {
    /// Unit tests that test individual components in isolation
    Unit,
    /// Integration tests that test how components work together
    Integration,
    /// End-to-end tests that test the entire system
    EndToEnd,
    /// Performance tests that measure system performance
    Performance,
    /// Security tests that verify system security
    Security,
    /// Custom test category
    Custom(u32),
}

impl fmt::Display for TestCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestCategory::Unit => write!(f, "Unit"),
            TestCategory::Integration => write!(f, "Integration"),
            TestCategory::EndToEnd => write!(f, "End-to-End"),
            TestCategory::Performance => write!(f, "Performance"),
            TestCategory::Security => write!(f, "Security"),
            TestCategory::Custom(id) => write!(f, "Custom({})", id),
        }
    }
}

/// Test outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TestOutcome {
    /// Test passed
    Passed,
    /// Test failed
    Failed,
    /// Test was skipped
    Skipped,
    /// Test timed out
    TimedOut,
    /// Test panicked
    Panicked,
}

impl fmt::Display for TestOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestOutcome::Passed => write!(f, "Passed"),
            TestOutcome::Failed => write!(f, "Failed"),
            TestOutcome::Skipped => write!(f, "Skipped"),
            TestOutcome::TimedOut => write!(f, "Timed Out"),
            TestOutcome::Panicked => write!(f, "Panicked"),
        }
    }
}

/// Test harness errors
#[derive(Error, Debug)]
pub enum TestHarnessError {
    /// Plugin error
    #[error("Plugin error: {0}")]
    PluginError(String),
    /// Environment error
    #[error("Environment error: {0}")]
    EnvironmentError(String),
    /// Test execution error
    #[error("Test execution error: {0}")]
    ExecutionError(String),
    /// Reporting error
    #[error("Reporting error: {0}")]
    ReportingError(String),
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    /// HTTP error
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    /// Timeout error
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    /// Assertion error
    #[error("Assertion error: {0}")]
    AssertionError(String),
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Test context that is passed to test cases
#[derive(Debug, Clone)]
pub struct TestContext {
    /// Test category
    pub category: TestCategory,
    /// Test name
    pub name: String,
    /// Test description
    pub description: Option<String>,
    /// Test tags
    pub tags: Vec<String>,
    /// Test parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Test artifacts directory
    pub artifacts_dir: Option<PathBuf>,
    /// Test start time
    pub start_time: Option<DateTime<Utc>>,
    /// Test timeout
    pub timeout: Option<Duration>,
    /// Custom test data
    pub custom_data: HashMap<String, serde_json::Value>,
}

impl TestContext {
    /// Create a new test context
    pub fn new(category: TestCategory, name: String) -> Self {
        Self {
            category,
            name,
            description: None,
            tags: Vec::new(),
            parameters: HashMap::new(),
            artifacts_dir: None,
            start_time: None,
            timeout: None,
            custom_data: HashMap::new(),
        }
    }

    /// Set the test description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a tag to the test
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags to the test
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags.extend(tags.into_iter().map(|t| t.into()));
        self
    }

    /// Add a parameter to the test
    pub fn with_parameter(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Result<Self, TestHarnessError> {
        let value = serde_json::to_value(value).map_err(TestHarnessError::SerializationError)?;
        self.parameters.insert(key.into(), value);
        Ok(self)
    }

    /// Set the artifacts directory
    pub fn with_artifacts_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.artifacts_dir = Some(dir.into());
        self
    }

    /// Set the test timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Add custom data to the test
    pub fn with_custom_data(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Result<Self, TestHarnessError> {
        let value = serde_json::to_value(value).map_err(TestHarnessError::SerializationError)?;
        self.custom_data.insert(key.into(), value);
        Ok(self)
    }
}

/// Test priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TestPriority {
    /// Critical tests that must pass for the system to be considered functional
    Critical = 0,
    /// High priority tests
    High = 1,
    /// Medium priority tests
    Medium = 2,
    /// Low priority tests
    Low = 3,
}

impl Default for TestPriority {
    fn default() -> Self {
        TestPriority::Medium
    }
}

impl fmt::Display for TestPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestPriority::Critical => write!(f, "Critical"),
            TestPriority::High => write!(f, "High"),
            TestPriority::Medium => write!(f, "Medium"),
            TestPriority::Low => write!(f, "Low"),
        }
    }
}

/// Standardized test interface for all test categories
#[async_trait::async_trait]
pub trait TestInterface: Send + Sync {
    /// Get the test category
    fn category(&self) -> TestCategory;

    /// Get the test name
    fn name(&self) -> &str;

    /// Get the test description
    fn description(&self) -> Option<&str>;

    /// Get the test tags
    fn tags(&self) -> &[String];

    /// Get the test priority
    fn priority(&self) -> TestPriority;

    /// Get the test dependencies
    fn dependencies(&self) -> &[String];

    /// Execute the test
    async fn execute(&self, context: &TestContext) -> Result<TestResult, TestHarnessError>;

    /// Set up the test environment
    async fn setup(&self, context: &TestContext) -> Result<(), TestHarnessError>;

    /// Clean up the test environment
    async fn teardown(&self, context: &TestContext) -> Result<(), TestHarnessError>;

    /// Check if the test should be skipped
    async fn should_skip(&self, context: &TestContext) -> Result<bool, TestHarnessError>;

    /// Get the test timeout
    fn timeout(&self) -> Option<Duration>;

    /// Get the test metadata
    fn metadata(&self) -> HashMap<String, serde_json::Value>;
}

/// Test case definition
#[derive(Debug)]
pub struct TestCase {
    /// Test context
    pub context: TestContext,
    /// Test function
    pub test_fn: Box<
        dyn Fn(
                &TestContext,
            )
                -> futures::future::BoxFuture<'static, Result<TestResult, TestHarnessError>>
            + Send
            + Sync,
    >,
    /// Setup function
    pub setup_fn: Option<
        Box<
            dyn Fn(
                    &TestContext,
                )
                    -> futures::future::BoxFuture<'static, Result<(), TestHarnessError>>
                + Send
                + Sync,
        >,
    >,
    /// Teardown function
    pub teardown_fn: Option<
        Box<
            dyn Fn(
                    &TestContext,
                )
                    -> futures::future::BoxFuture<'static, Result<(), TestHarnessError>>
                + Send
                + Sync,
        >,
    >,
    /// Whether to run this test in parallel
    pub parallel: bool,
    /// Dependencies on other tests
    pub dependencies: Vec<String>,
    /// Test priority
    pub priority: TestPriority,
}

impl TestCase {
    /// Create a new test case
    pub fn new(
        context: TestContext,
        test_fn: impl Fn(
                &TestContext,
            )
                -> futures::future::BoxFuture<'static, Result<TestResult, TestHarnessError>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        Self {
            context,
            test_fn: Box::new(test_fn),
            setup_fn: None,
            teardown_fn: None,
            parallel: false,
            dependencies: Vec::new(),
            priority: TestPriority::Medium,
        }
    }

    /// Set the setup function
    pub fn with_setup(
        mut self,
        setup_fn: impl Fn(&TestContext) -> futures::future::BoxFuture<'static, Result<(), TestHarnessError>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        self.setup_fn = Some(Box::new(setup_fn));
        self
    }

    /// Set the teardown function
    pub fn with_teardown(
        mut self,
        teardown_fn: impl Fn(&TestContext) -> futures::future::BoxFuture<'static, Result<(), TestHarnessError>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        self.teardown_fn = Some(Box::new(teardown_fn));
        self
    }

    /// Enable parallel execution
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Add a dependency on another test
    pub fn with_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependencies.push(dependency.into());
        self
    }

    /// Add multiple dependencies on other tests
    pub fn with_dependencies(
        mut self,
        dependencies: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.dependencies
            .extend(dependencies.into_iter().map(|d| d.into()));
        self
    }

    /// Set the test priority
    pub fn with_priority(mut self, priority: TestPriority) -> Self {
        self.priority = priority;
        self
    }
}

#[async_trait::async_trait]
impl TestInterface for TestCase {
    fn category(&self) -> TestCategory {
        self.context.category
    }

    fn name(&self) -> &str {
        &self.context.name
    }

    fn description(&self) -> Option<&str> {
        self.context.description.as_deref()
    }

    fn tags(&self) -> &[String] {
        &self.context.tags
    }

    fn priority(&self) -> TestPriority {
        self.priority
    }

    fn dependencies(&self) -> &[String] {
        &self.dependencies
    }

    async fn execute(&self, context: &TestContext) -> Result<TestResult, TestHarnessError> {
        (self.test_fn)(context).await
    }

    async fn setup(&self, context: &TestContext) -> Result<(), TestHarnessError> {
        if let Some(setup_fn) = &self.setup_fn {
            setup_fn(context).await
        } else {
            Ok(())
        }
    }

    async fn teardown(&self, context: &TestContext) -> Result<(), TestHarnessError> {
        if let Some(teardown_fn) = &self.teardown_fn {
            teardown_fn(context).await
        } else {
            Ok(())
        }
    }

    async fn should_skip(&self, _context: &TestContext) -> Result<bool, TestHarnessError> {
        // Default implementation: don't skip
        Ok(false)
    }

    fn timeout(&self) -> Option<Duration> {
        self.context.timeout
    }

    fn metadata(&self) -> HashMap<String, serde_json::Value> {
        self.context.custom_data.clone()
    }
}

/// Test suite definition
#[derive(Debug)]
pub struct TestSuite {
    /// Suite name
    pub name: String,
    /// Suite description
    pub description: Option<String>,
    /// Test cases in this suite
    pub test_cases: Vec<TestCase>,
    /// Setup function for the entire suite
    pub setup_fn: Option<
        Box<
            dyn Fn() -> futures::future::BoxFuture<'static, Result<(), TestHarnessError>>
                + Send
                + Sync,
        >,
    >,
    /// Teardown function for the entire suite
    pub teardown_fn: Option<
        Box<
            dyn Fn() -> futures::future::BoxFuture<'static, Result<(), TestHarnessError>>
                + Send
                + Sync,
        >,
    >,
    /// Whether to run tests in parallel
    pub parallel: bool,
    /// Dependencies on other suites
    pub dependencies: Vec<String>,
    /// Suite priority
    pub priority: TestPriority,
}

impl TestSuite {
    /// Create a new test suite
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            test_cases: Vec::new(),
            setup_fn: None,
            teardown_fn: None,
            parallel: false,
            dependencies: Vec::new(),
            priority: TestPriority::Medium,
        }
    }

    /// Set the suite description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a test case to the suite
    pub fn with_test_case(mut self, test_case: TestCase) -> Self {
        self.test_cases.push(test_case);
        self
    }

    /// Add multiple test cases to the suite
    pub fn with_test_cases(mut self, test_cases: impl IntoIterator<Item = TestCase>) -> Self {
        self.test_cases.extend(test_cases);
        self
    }

    /// Set the setup function for the entire suite
    pub fn with_setup(
        mut self,
        setup_fn: impl Fn() -> futures::future::BoxFuture<'static, Result<(), TestHarnessError>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        self.setup_fn = Some(Box::new(setup_fn));
        self
    }

    /// Set the teardown function for the entire suite
    pub fn with_teardown(
        mut self,
        teardown_fn: impl Fn() -> futures::future::BoxFuture<'static, Result<(), TestHarnessError>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        self.teardown_fn = Some(Box::new(teardown_fn));
        self
    }

    /// Enable parallel execution of tests in this suite
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Add a dependency on another suite
    pub fn with_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependencies.push(dependency.into());
        self
    }

    /// Add multiple dependencies on other suites
    pub fn with_dependencies(
        mut self,
        dependencies: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.dependencies
            .extend(dependencies.into_iter().map(|d| d.into()));
        self
    }

    /// Set the suite priority
    pub fn with_priority(mut self, priority: TestPriority) -> Self {
        self.priority = priority;
        self
    }
}

/// Test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test name
    pub name: String,
    /// Test category
    pub category: TestCategory,
    /// Test outcome
    pub outcome: TestOutcome,
    /// Error message if the test failed
    pub error: Option<String>,
    /// Test duration
    pub duration: Duration,
    /// Test start time
    pub start_time: DateTime<Utc>,
    /// Test end time
    pub end_time: DateTime<Utc>,
    /// Test artifacts
    pub artifacts: HashMap<String, String>,
    /// Test metrics
    pub metrics: HashMap<String, f64>,
    /// Test logs
    pub logs: Vec<String>,
    /// Custom test data
    pub custom_data: HashMap<String, serde_json::Value>,
    /// Assertion results
    #[serde(default)]
    pub assertions: Vec<AssertionResult>,
}

impl TestResult {
    /// Create a new test result
    pub fn new(name: impl Into<String>, category: TestCategory, outcome: TestOutcome) -> Self {
        let now = Utc::now();
        Self {
            name: name.into(),
            category,
            outcome,
            error: None,
            duration: Duration::from_secs(0),
            start_time: now,
            end_time: now,
            artifacts: HashMap::new(),
            metrics: HashMap::new(),
            logs: Vec::new(),
            custom_data: HashMap::new(),
            assertions: Vec::new(),
        }
    }

    /// Set the error message
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Set the test duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Set the test start time
    pub fn with_start_time(mut self, start_time: DateTime<Utc>) -> Self {
        self.start_time = start_time;
        self
    }

    /// Set the test end time
    pub fn with_end_time(mut self, end_time: DateTime<Utc>) -> Self {
        self.end_time = end_time;
        self
    }

    /// Add an artifact to the test result
    pub fn with_artifact(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.artifacts.insert(key.into(), value.into());
        self
    }

    /// Add a metric to the test result
    pub fn with_metric(mut self, key: impl Into<String>, value: f64) -> Self {
        self.metrics.insert(key.into(), value);
        self
    }

    /// Add a log entry to the test result
    pub fn with_log(mut self, log: impl Into<String>) -> Self {
        self.logs.push(log.into());
        self
    }

    /// Add custom data to the test result
    pub fn with_custom_data(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Result<Self, TestHarnessError> {
        let value = serde_json::to_value(value).map_err(TestHarnessError::SerializationError)?;
        self.custom_data.insert(key.into(), value);
        Ok(self)
    }

    /// Add an assertion result to the test result
    pub fn with_assertion(mut self, assertion: AssertionResult) -> Self {
        self.assertions.push(assertion);
        self
    }
}

/// Test suite result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteResult {
    /// Suite name
    pub name: String,
    /// Test results
    pub test_results: Vec<TestResult>,
    /// Suite start time
    pub start_time: DateTime<Utc>,
    /// Suite end time
    pub end_time: DateTime<Utc>,
    /// Suite duration
    pub duration: Duration,
    /// Number of passed tests
    pub passed: usize,
    /// Number of failed tests
    pub failed: usize,
    /// Number of skipped tests
    pub skipped: usize,
    /// Number of timed out tests
    pub timed_out: usize,
    /// Number of panicked tests
    pub panicked: usize,
}

impl TestSuiteResult {
    /// Create a new test suite result
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            name: name.into(),
            test_results: Vec::new(),
            start_time: now,
            end_time: now,
            duration: Duration::from_secs(0),
            passed: 0,
            failed: 0,
            skipped: 0,
            timed_out: 0,
            panicked: 0,
        }
    }

    /// Add a test result to the suite result
    pub fn add_test_result(&mut self, result: TestResult) {
        match result.outcome {
            TestOutcome::Passed => self.passed += 1,
            TestOutcome::Failed => self.failed += 1,
            TestOutcome::Skipped => self.skipped += 1,
            TestOutcome::TimedOut => self.timed_out += 1,
            TestOutcome::Panicked => self.panicked += 1,
        }
        self.test_results.push(result);
    }

    /// Set the suite start time
    pub fn with_start_time(mut self, start_time: DateTime<Utc>) -> Self {
        self.start_time = start_time;
        self
    }

    /// Set the suite end time
    pub fn with_end_time(mut self, end_time: DateTime<Utc>) -> Self {
        self.end_time = end_time;
        self
    }

    /// Set the suite duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed == 0 && self.timed_out == 0 && self.panicked == 0
    }

    /// Get the total number of tests
    pub fn total_tests(&self) -> usize {
        self.passed + self.failed + self.skipped + self.timed_out + self.panicked
    }
}

/// Assertion error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionError {
    /// Error message
    pub message: String,
    /// Expected value
    pub expected: Option<String>,
    /// Actual value
    pub actual: Option<String>,
    /// Source location
    pub location: Option<String>,
}

impl AssertionError {
    /// Create a new assertion error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            expected: None,
            actual: None,
            location: None,
        }
    }

    /// Set the expected value
    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }

    /// Set the actual value
    pub fn with_actual(mut self, actual: impl Into<String>) -> Self {
        self.actual = Some(actual.into());
        self
    }

    /// Set the source location
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }
}

/// Assertion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResult {
    /// Assertion name
    pub name: String,
    /// Whether the assertion passed
    pub passed: bool,
    /// Error message if the assertion failed
    pub error: Option<String>,
    /// Expected value
    pub expected: Option<String>,
    /// Actual value
    pub actual: Option<String>,
}

impl AssertionResult {
    /// Create a new assertion result
    pub fn new(name: impl Into<String>, passed: bool) -> Self {
        Self {
            name: name.into(),
            passed,
            error: None,
            expected: None,
            actual: None,
        }
    }

    /// Set the error message
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Set the expected value
    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }

    /// Set the actual value
    pub fn with_actual(mut self, actual: impl Into<String>) -> Self {
        self.actual = Some(actual.into());
        self
    }

    /// Check if the assertion passed
    pub fn passed(&self) -> bool {
        self.passed
    }

    /// Check if the assertion failed
    pub fn failed(&self) -> bool {
        !self.passed
    }

    /// Check if the assertion is a warning
    pub fn is_warning(&self) -> bool {
        // Implement your warning logic here
        // For example, you might have a specific error message format for warnings
        if let Some(error) = &self.error {
            error.starts_with("WARNING:")
        } else {
            false
        }
    }

    /// Get the assertion message
    pub fn message(&self) -> &str {
        if let Some(error) = &self.error {
            error
        } else {
            &self.name
        }
    }

    /// Get the expected value
    pub fn expected(&self) -> Option<&str> {
        self.expected.as_deref()
    }

    /// Get the actual value
    pub fn actual(&self) -> Option<&str> {
        self.actual.as_deref()
    }
}

/// Assertion trait for creating custom assertions
pub trait Assertion: Send + Sync {
    /// Assert a condition
    fn assert(&self, context: &TestContext) -> AssertionResult;
    /// Get the assertion name
    fn name(&self) -> &str;
}
