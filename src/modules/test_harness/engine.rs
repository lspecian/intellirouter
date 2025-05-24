//! Test Execution Engine
//!
//! This module provides the core test execution engine for the test harness.
//! It supports parallel and sequential execution of tests, with dependency resolution.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::Utc;
use futures::future::{join_all, BoxFuture, FutureExt};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use super::environment::Environment;
use super::plugins::PluginManager;
use super::reporting::Reporter;
use super::types::{
    TestCase, TestCategory, TestContext, TestHarnessError, TestInterface, TestOutcome,
    TestPriority, TestResult, TestSuite, TestSuiteResult,
};

/// Test execution options
#[derive(Debug, Clone)]
pub struct TestExecutionOptions {
    /// Maximum number of parallel tests
    pub max_parallel_tests: usize,
    /// Default test timeout
    pub default_timeout: Duration,
    /// Whether to fail fast on the first test failure
    pub fail_fast: bool,
    /// Whether to include skipped tests in the results
    pub include_skipped: bool,
    /// Whether to retry failed tests
    pub retry_failed: bool,
    /// Maximum number of retries for failed tests
    pub max_retries: usize,
    /// Categories to include
    pub include_categories: Option<Vec<TestCategory>>,
    /// Categories to exclude
    pub exclude_categories: Option<Vec<TestCategory>>,
    /// Tags to include
    pub include_tags: Option<Vec<String>>,
    /// Tags to exclude
    pub exclude_tags: Option<Vec<String>>,
    /// Test names to include
    pub include_tests: Option<Vec<String>>,
    /// Test names to exclude
    pub exclude_tests: Option<Vec<String>>,
    /// Whether to shuffle test execution order
    pub shuffle: bool,
    /// Random seed for shuffling
    pub shuffle_seed: Option<u64>,
    /// Whether to prioritize tests by priority
    pub prioritize: bool,
    /// Maximum priority level to run (inclusive)
    pub max_priority: Option<TestPriority>,
}

impl Default for TestExecutionOptions {
    fn default() -> Self {
        Self {
            max_parallel_tests: num_cpus::get(),
            default_timeout: Duration::from_secs(60),
            fail_fast: false,
            include_skipped: true,
            retry_failed: false,
            max_retries: 3,
            include_categories: None,
            exclude_categories: None,
            include_tags: None,
            exclude_tags: None,
            include_tests: None,
            exclude_tests: None,
            shuffle: false,
            shuffle_seed: None,
            prioritize: true,
            max_priority: None,
        }
    }
}

/// Test engine builder
#[derive(Default)]
pub struct TestEngineBuilder {
    options: TestExecutionOptions,
    environment: Option<dyn Environment>,
    plugin_manager: Option<Arc<PluginManager>>,
    reporter: Option<Arc<dyn Reporter>>,
}

impl TestEngineBuilder {
    /// Create a new test engine builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the test execution options
    pub fn with_options(mut self, options: TestExecutionOptions) -> Self {
        self.options = options;
        self
    }

    /// Set the test environment
    pub fn with_environment(mut self, environment: Box<dyn Environment>) -> Self {
        self.environment = Some(environment);
        self
    }

    /// Set the plugin manager
    pub fn with_plugin_manager(mut self, plugin_manager: Arc<PluginManager>) -> Self {
        self.plugin_manager = Some(plugin_manager);
        self
    }

    /// Set the reporter
    pub fn with_reporter(mut self, reporter: Arc<dyn Reporter>) -> Self {
        self.reporter = Some(reporter);
        self
    }

    /// Build the test engine
    pub fn build(self) -> TestEngine {
        let environment = self
            .environment
            .unwrap_or_else(|| Box::new(super::environment::LocalEnvironment::new()));
        let plugin_manager = self
            .plugin_manager
            .unwrap_or_else(|| Arc::new(PluginManager::new()));
        let reporter = self.reporter.unwrap_or_else(|| {
            Arc::new(super::reporting::ConsoleReporter::new()) as Arc<dyn Reporter>
        });

        TestEngine {
            options: self.options,
            environment,
            plugin_manager,
            reporter,
        }
    }
}

/// Test execution engine
pub struct TestEngine {
    /// Test execution options
    options: TestExecutionOptions,
    /// Test environment
    environment: Box<dyn Environment>,
    /// Plugin manager
    plugin_manager: Arc<PluginManager>,
    /// Reporter
    reporter: Arc<dyn Reporter>,
}

impl TestEngine {
    /// Create a new test engine with default options
    pub fn new() -> Self {
        TestEngineBuilder::new().build()
    }

    /// Create a new test engine builder
    pub fn builder() -> TestEngineBuilder {
        TestEngineBuilder::new()
    }

    /// Run a test suite
    pub async fn run_suite(&self, suite: TestSuite) -> Result<TestSuiteResult, TestHarnessError> {
        info!("Running test suite: {}", suite.name);
        let start_time = Instant::now();
        let start_datetime = Utc::now();

        // Initialize the suite result
        let mut suite_result = TestSuiteResult::new(&suite.name).with_start_time(start_datetime);

        // Run suite setup if provided
        if let Some(setup_fn) = &suite.setup_fn {
            info!("Running suite setup for: {}", suite.name);
            if let Err(e) = setup_fn().await {
                error!("Suite setup failed: {}", e);
                return Err(e);
            }
        }

        // Filter test cases based on options
        let test_cases = self.filter_test_cases(&suite.test_cases);
        if test_cases.is_empty() {
            warn!("No test cases to run after filtering");
            return Ok(suite_result);
        }

        // Sort test cases by dependencies and priority
        let sorted_test_cases = self.sort_test_cases(&test_cases)?;

        // Execute test cases
        let test_results = if suite.parallel {
            self.run_test_cases_parallel(&sorted_test_cases).await?
        } else {
            self.run_test_cases_sequential(&sorted_test_cases).await?
        };

        // Add test results to suite result
        for result in test_results {
            suite_result.add_test_result(result);
        }

        // Run suite teardown if provided
        if let Some(teardown_fn) = &suite.teardown_fn {
            info!("Running suite teardown for: {}", suite.name);
            if let Err(e) = teardown_fn().await {
                error!("Suite teardown failed: {}", e);
                // Don't fail the entire suite if teardown fails
                // but add a warning to the report
                self.reporter
                    .report_warning(&format!("Suite teardown failed for {}: {}", suite.name, e))
                    .await;
            }
        }

        // Finalize suite result
        let end_datetime = Utc::now();
        let duration = start_time.elapsed();
        suite_result = suite_result
            .with_end_time(end_datetime)
            .with_duration(duration);

        // Report suite result
        self.reporter.report_suite_result(&suite_result).await;

        info!(
            "Test suite {} completed in {:?}: {} passed, {} failed, {} skipped, {} timed out, {} panicked",
            suite.name,
            duration,
            suite_result.passed,
            suite_result.failed,
            suite_result.skipped,
            suite_result.timed_out,
            suite_result.panicked
        );

        Ok(suite_result)
    }

    /// Run multiple test suites
    pub async fn run_suites(
        &self,
        suites: Vec<TestSuite>,
    ) -> Result<Vec<TestSuiteResult>, TestHarnessError> {
        info!("Running {} test suites", suites.len());
        let start_time = Instant::now();

        // Sort suites by dependencies and priority
        let sorted_suites = self.sort_suites(&suites)?;

        // Execute suites
        let mut suite_results = Vec::new();
        for suite in sorted_suites {
            let result = self.run_suite(suite).await?;
            suite_results.push(result);

            // Check if we should fail fast
            if self.options.fail_fast && suite_results.last().map_or(false, |r| !r.all_passed()) {
                warn!("Stopping test execution due to fail_fast option");
                break;
            }
        }

        let duration = start_time.elapsed();
        let total_tests: usize = suite_results.iter().map(|r| r.total_tests()).sum();
        let passed: usize = suite_results.iter().map(|r| r.passed).sum();
        let failed: usize = suite_results.iter().map(|r| r.failed).sum();
        let skipped: usize = suite_results.iter().map(|r| r.skipped).sum();
        let timed_out: usize = suite_results.iter().map(|r| r.timed_out).sum();
        let panicked: usize = suite_results.iter().map(|r| r.panicked).sum();

        info!(
            "All test suites completed in {:?}: {} total tests, {} passed, {} failed, {} skipped, {} timed out, {} panicked",
            duration, total_tests, passed, failed, skipped, timed_out, panicked
        );

        Ok(suite_results)
    }

    /// Filter test cases based on options
    fn filter_test_cases(&self, test_cases: &[TestCase]) -> Vec<TestCase> {
        let mut filtered_cases = Vec::new();

        for test_case in test_cases {
            // Check if test should be included based on name
            if let Some(include_tests) = &self.options.include_tests {
                if !include_tests
                    .iter()
                    .any(|name| test_case.context.name.contains(name))
                {
                    continue;
                }
            }

            // Check if test should be excluded based on name
            if let Some(exclude_tests) = &self.options.exclude_tests {
                if exclude_tests
                    .iter()
                    .any(|name| test_case.context.name.contains(name))
                {
                    continue;
                }
            }

            // Check if test category should be included
            if let Some(include_categories) = &self.options.include_categories {
                if !include_categories.contains(&test_case.context.category) {
                    continue;
                }
            }

            // Check if test category should be excluded
            if let Some(exclude_categories) = &self.options.exclude_categories {
                if exclude_categories.contains(&test_case.context.category) {
                    continue;
                }
            }

            // Check if test tags should be included
            if let Some(include_tags) = &self.options.include_tags {
                if !test_case
                    .context
                    .tags
                    .iter()
                    .any(|tag| include_tags.contains(tag))
                {
                    continue;
                }
            }

            // Check if test tags should be excluded
            if let Some(exclude_tags) = &self.options.exclude_tags {
                if test_case
                    .context
                    .tags
                    .iter()
                    .any(|tag| exclude_tags.contains(tag))
                {
                    continue;
                }
            }

            // Check if test priority should be included
            if let Some(max_priority) = &self.options.max_priority {
                if &test_case.priority > max_priority {
                    continue;
                }
            }

            filtered_cases.push(test_case.clone());
        }

        // Optionally shuffle the test cases
        if self.options.shuffle {
            use rand::{seq::SliceRandom, SeedableRng};
            let seed = self.options.shuffle_seed.unwrap_or_else(|| {
                use std::time::{SystemTime, UNIX_EPOCH};
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            });
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            filtered_cases.shuffle(&mut rng);
        }

        filtered_cases
    }

    /// Sort test cases by dependencies and priority
    fn sort_test_cases(&self, test_cases: &[TestCase]) -> Result<Vec<TestCase>, TestHarnessError> {
        // First, sort by dependencies to ensure correct execution order
        let mut sorted_cases = self.sort_test_cases_by_dependencies(test_cases)?;

        // Then, if prioritization is enabled, sort by priority within each dependency level
        if self.options.prioritize {
            sorted_cases = self.prioritize_test_cases(sorted_cases);
        }

        Ok(sorted_cases)
    }

    /// Sort test cases by dependencies
    fn sort_test_cases_by_dependencies(
        &self,
        test_cases: &[TestCase],
    ) -> Result<Vec<TestCase>, TestHarnessError> {
        // Build a map of test names to indices
        let mut name_to_index = HashMap::new();
        for (i, test_case) in test_cases.iter().enumerate() {
            name_to_index.insert(test_case.context.name.clone(), i);
        }

        // Build a dependency graph
        let mut graph = vec![Vec::new(); test_cases.len()];
        let mut in_degree = vec![0; test_cases.len()];

        for (i, test_case) in test_cases.iter().enumerate() {
            for dep in &test_case.dependencies {
                if let Some(&dep_idx) = name_to_index.get(dep) {
                    graph[dep_idx].push(i);
                    in_degree[i] += 1;
                } else {
                    return Err(TestHarnessError::ExecutionError(format!(
                        "Test case {} depends on unknown test case {}",
                        test_case.context.name, dep
                    )));
                }
            }
        }

        // Topological sort
        let mut queue = VecDeque::new();
        for (i, &degree) in in_degree.iter().enumerate() {
            if degree == 0 {
                queue.push_back(i);
            }
        }

        let mut sorted_indices = Vec::new();
        while let Some(i) = queue.pop_front() {
            sorted_indices.push(i);
            for &j in &graph[i] {
                in_degree[j] -= 1;
                if in_degree[j] == 0 {
                    queue.push_back(j);
                }
            }
        }

        // Check for cycles
        if sorted_indices.len() != test_cases.len() {
            return Err(TestHarnessError::ExecutionError(
                "Cyclic dependencies detected in test cases".to_string(),
            ));
        }

        // Create sorted test cases
        let sorted_test_cases = sorted_indices
            .into_iter()
            .map(|i| test_cases[i].clone())
            .collect();

        Ok(sorted_test_cases)
    }

    /// Prioritize test cases while maintaining dependency order
    fn prioritize_test_cases(&self, test_cases: Vec<TestCase>) -> Vec<TestCase> {
        // Group test cases by their dependency level
        let mut dependency_levels: HashMap<usize, Vec<TestCase>> = HashMap::new();
        let mut current_level = 0;
        let mut remaining_cases: HashSet<String> = test_cases
            .iter()
            .map(|tc| tc.context.name.clone())
            .collect();

        while !remaining_cases.is_empty() {
            let mut current_level_cases = Vec::new();

            for test_case in &test_cases {
                if !remaining_cases.contains(&test_case.context.name) {
                    continue;
                }

                // Check if all dependencies are satisfied
                let deps_satisfied = test_case
                    .dependencies
                    .iter()
                    .all(|dep| !remaining_cases.contains(dep));

                if deps_satisfied {
                    current_level_cases.push(test_case.clone());
                    remaining_cases.remove(&test_case.context.name);
                }
            }

            if !current_level_cases.is_empty() {
                dependency_levels.insert(current_level, current_level_cases);
                current_level += 1;
            } else {
                // This should not happen if dependencies are properly resolved
                break;
            }
        }

        // Sort each level by priority
        let mut sorted_cases = Vec::new();
        for level in 0..current_level {
            if let Some(mut level_cases) = dependency_levels.remove(&level) {
                // Sort by priority (Critical first, Low last)
                level_cases.sort_by_key(|tc| tc.priority as u8);
                sorted_cases.extend(level_cases);
            }
        }

        sorted_cases
    }

    /// Sort suites by dependencies and priority
    fn sort_suites(&self, suites: &[TestSuite]) -> Result<Vec<TestSuite>, TestHarnessError> {
        // First, sort by dependencies to ensure correct execution order
        let mut sorted_suites = self.sort_suites_by_dependencies(suites)?;

        // Then, if prioritization is enabled, sort by priority within each dependency level
        if self.options.prioritize {
            sorted_suites = self.prioritize_suites(sorted_suites);
        }

        Ok(sorted_suites)
    }

    /// Prioritize suites while maintaining dependency order
    fn prioritize_suites(&self, suites: Vec<TestSuite>) -> Vec<TestSuite> {
        // Group suites by their dependency level
        let mut dependency_levels: HashMap<usize, Vec<TestSuite>> = HashMap::new();
        let mut current_level = 0;
        let mut remaining_suites: HashSet<String> = suites.iter().map(|s| s.name.clone()).collect();

        while !remaining_suites.is_empty() {
            let mut current_level_suites = Vec::new();

            for suite in &suites {
                if !remaining_suites.contains(&suite.name) {
                    continue;
                }

                // Check if all dependencies are satisfied
                let deps_satisfied = suite
                    .dependencies
                    .iter()
                    .all(|dep| !remaining_suites.contains(dep));

                if deps_satisfied {
                    current_level_suites.push(suite.clone());
                    remaining_suites.remove(&suite.name);
                }
            }

            if !current_level_suites.is_empty() {
                dependency_levels.insert(current_level, current_level_suites);
                current_level += 1;
            } else {
                // This should not happen if dependencies are properly resolved
                break;
            }
        }

        // Sort each level by priority
        let mut sorted_suites = Vec::new();
        for level in 0..current_level {
            if let Some(mut level_suites) = dependency_levels.remove(&level) {
                // Sort by priority (Critical first, Low last)
                level_suites.sort_by_key(|s| s.priority as u8);
                sorted_suites.extend(level_suites);
            }
        }

        sorted_suites
    }

    /// Sort suites by dependencies
    fn sort_suites_by_dependencies(
        &self,
        suites: &[TestSuite],
    ) -> Result<Vec<TestSuite>, TestHarnessError> {
        // Build a map of suite names to indices
        let mut name_to_index = HashMap::new();
        for (i, suite) in suites.iter().enumerate() {
            name_to_index.insert(suite.name.clone(), i);
        }

        // Build a dependency graph
        let mut graph = vec![Vec::new(); suites.len()];
        let mut in_degree = vec![0; suites.len()];

        for (i, suite) in suites.iter().enumerate() {
            for dep in &suite.dependencies {
                if let Some(&dep_idx) = name_to_index.get(dep) {
                    graph[dep_idx].push(i);
                    in_degree[i] += 1;
                } else {
                    return Err(TestHarnessError::ExecutionError(format!(
                        "Suite {} depends on unknown suite {}",
                        suite.name, dep
                    )));
                }
            }
        }

        // Topological sort
        let mut queue = VecDeque::new();
        for (i, &degree) in in_degree.iter().enumerate() {
            if degree == 0 {
                queue.push_back(i);
            }
        }

        let mut sorted_indices = Vec::new();
        while let Some(i) = queue.pop_front() {
            sorted_indices.push(i);
            for &j in &graph[i] {
                in_degree[j] -= 1;
                if in_degree[j] == 0 {
                    queue.push_back(j);
                }
            }
        }

        // Check for cycles
        if sorted_indices.len() != suites.len() {
            return Err(TestHarnessError::ExecutionError(
                "Cyclic dependencies detected in test suites".to_string(),
            ));
        }

        // Create sorted suites
        let sorted_suites = sorted_indices
            .into_iter()
            .map(|i| suites[i].clone())
            .collect();

        Ok(sorted_suites)
    }

    /// Run test cases sequentially
    async fn run_test_cases_sequential(
        &self,
        test_cases: &[TestCase],
    ) -> Result<Vec<TestResult>, TestHarnessError> {
        let mut results = Vec::new();

        for test_case in test_cases {
            let result = self.run_test_case(test_case).await?;
            results.push(result);

            // Check if we should fail fast
            if self.options.fail_fast
                && matches!(
                    results.last().unwrap().outcome,
                    TestOutcome::Failed | TestOutcome::TimedOut | TestOutcome::Panicked
                )
            {
                warn!("Stopping test execution due to fail_fast option");
                break;
            }
        }

        Ok(results)
    }

    /// Run test cases in parallel
    async fn run_test_cases_parallel(
        &self,
        test_cases: &[TestCase],
    ) -> Result<Vec<TestResult>, TestHarnessError> {
        // Create a channel for collecting results
        let (tx, mut rx) = mpsc::channel(test_cases.len());

        // Create a shared flag for fail-fast
        let should_stop = Arc::new(Mutex::new(false));

        // Create a set of futures for each test case
        let mut futures = Vec::new();
        for test_case in test_cases {
            let test_case = test_case.clone();
            let tx = tx.clone();
            let should_stop = Arc::clone(&should_stop);
            let fail_fast = self.options.fail_fast;

            let future = async move {
                // Check if we should stop
                if *should_stop.lock().unwrap() {
                    return;
                }

                // Run the test case
                let result = self.run_test_case(&test_case).await.unwrap_or_else(|e| {
                    let mut result = TestResult::new(
                        test_case.name(),
                        test_case.category(),
                        TestOutcome::Failed,
                    );
                    result = result.with_error(format!("Test execution error: {}", e));
                    result
                });

                // Check if we should stop future tests
                if fail_fast
                    && matches!(
                        result.outcome,
                        TestOutcome::Failed | TestOutcome::TimedOut | TestOutcome::Panicked
                    )
                {
                    let mut should_stop = should_stop.lock().unwrap();
                    *should_stop = true;
                }

                // Send the result
                let _ = tx.send(result).await;
            };

            futures.push(future.boxed());
        }

        // Run the futures with a limit on parallelism
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.options.max_parallel_tests));
        let mut limited_futures = Vec::new();

        for future in futures {
            let semaphore = Arc::clone(&semaphore);
            let limited_future = async move {
                let _permit = semaphore.acquire().await.unwrap();
                future.await;
            };
            limited_futures.push(limited_future.boxed());
        }

        // Execute all futures
        join_all(limited_futures).await;
        drop(tx); // Drop the sender to close the channel

        // Collect results
        let mut results = Vec::new();
        while let Some(result) = rx.recv().await {
            results.push(result);
        }

        // Sort results by start time to maintain deterministic order
        results.sort_by(|a, b| a.start_time.cmp(&b.start_time));

        Ok(results)
    }

    /// Run a single test case
    async fn run_test_case(&self, test_case: &TestCase) -> Result<TestResult, TestHarnessError> {
        let test_name = test_case.name();
        info!("Running test case: {}", test_name);

        let start_time = Instant::now();
        let start_datetime = Utc::now();

        // Create a mutable context for the test
        let mut context = test_case.context.clone();
        context.start_time = Some(start_datetime);

        // Set default timeout if not specified
        if context.timeout.is_none() {
            context.timeout = Some(self.options.default_timeout);
        }

        // Check if the test should be skipped
        let should_skip = test_case.should_skip(&context).await?;
        if should_skip {
            info!("Skipping test: {}", test_name);
            let result = TestResult::new(test_name, test_case.category(), TestOutcome::Skipped)
                .with_start_time(start_datetime)
                .with_end_time(Utc::now())
                .with_duration(start_time.elapsed());

            // Report the result
            self.reporter.report_test_result(&result).await;

            return Ok(result);
        }

        // Run setup
        if let Err(e) = test_case.setup(&context).await {
            error!("Test setup failed: {}", e);
            let mut result = TestResult::new(test_name, test_case.category(), TestOutcome::Failed)
                .with_start_time(start_datetime)
                .with_end_time(Utc::now())
                .with_duration(start_time.elapsed())
                .with_error(format!("Setup failed: {}", e));

            // Report the result
            self.reporter.report_test_result(&result).await;

            return Ok(result);
        }

        // Run the test with timeout
        let timeout_duration = test_case.timeout().unwrap_or(self.options.default_timeout);

        let test_result = match timeout(timeout_duration, test_case.execute(&context)).await {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                error!("Test failed with error: {}", e);
                TestResult::new(test_name, test_case.category(), TestOutcome::Failed)
                    .with_error(format!("Test error: {}", e))
            }
            Err(_) => {
                error!("Test timed out after {:?}", timeout_duration);
                TestResult::new(test_name, test_case.category(), TestOutcome::TimedOut)
                    .with_error(format!("Test timed out after {:?}", timeout_duration))
            }
        };

        // Run teardown
        if let Err(e) = test_case.teardown(&context).await {
            warn!("Test teardown failed: {}", e);
            // Don't fail the test if teardown fails, but log it
            self.reporter
                .report_warning(&format!("Teardown failed for {}: {}", test_name, e))
                .await;
        }

        // Finalize the result
        let end_datetime = Utc::now();
        let duration = start_time.elapsed();
        let mut final_result = test_result
            .with_start_time(start_datetime)
            .with_end_time(end_datetime)
            .with_duration(duration);

        // Report the result
        self.reporter.report_test_result(&final_result).await;

        // Log the result
        match final_result.outcome {
            TestOutcome::Passed => info!("Test passed: {} ({:?})", test_name, duration),
            TestOutcome::Failed => error!(
                "Test failed: {} ({:?}): {}",
                test_name,
                duration,
                final_result.error.as_deref().unwrap_or("No error message")
            ),
            TestOutcome::Skipped => info!("Test skipped: {}", test_name),
            TestOutcome::TimedOut => error!(
                "Test timed out: {} ({:?}): {}",
                test_name,
                duration,
                final_result.error.as_deref().unwrap_or("No error message")
            ),
            TestOutcome::Panicked => error!(
                "Test panicked: {} ({:?}): {}",
                test_name,
                duration,
                final_result.error.as_deref().unwrap_or("No error message")
            ),
        }

        Ok(final_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = TestEngine::new();
        assert_eq!(engine.options.max_parallel_tests, num_cpus::get());
    }

    #[tokio::test]
    async fn test_engine_builder() {
        let options = TestExecutionOptions {
            max_parallel_tests: 4,
            default_timeout: Duration::from_secs(30),
            ..Default::default()
        };

        let engine = TestEngine::builder().with_options(options).build();
        assert_eq!(engine.options.max_parallel_tests, 4);
        assert_eq!(engine.options.default_timeout, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_run_single_test_case() {
        let engine = TestEngine::new();

        let context = TestContext::new(TestCategory::Unit, "test_simple_pass".to_string());
        let test_case = TestCase::new(context, |_| {
            async {
                Ok(TestResult::new(
                    "test_simple_pass",
                    TestCategory::Unit,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        });

        let result = engine.run_test_case(&test_case).await.unwrap();
        assert_eq!(result.outcome, TestOutcome::Passed);
    }

    #[tokio::test]
    async fn test_run_test_case_with_setup_teardown() {
        let engine = TestEngine::new();

        let setup_called = Arc::new(AtomicBool::new(false));
        let teardown_called = Arc::new(AtomicBool::new(false));

        let setup_called_clone = Arc::clone(&setup_called);
        let teardown_called_clone = Arc::clone(&teardown_called);

        let context = TestContext::new(TestCategory::Unit, "test_with_setup_teardown".to_string());
        let test_case = TestCase::new(context, move |_| {
            async move {
                assert!(setup_called_clone.load(Ordering::SeqCst));
                Ok(TestResult::new(
                    "test_with_setup_teardown",
                    TestCategory::Unit,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        })
        .with_setup(move |_| {
            async move {
                setup_called.store(true, Ordering::SeqCst);
                Ok(())
            }
            .boxed()
        })
        .with_teardown(move |_| {
            async move {
                teardown_called.store(true, Ordering::SeqCst);
                Ok(())
            }
            .boxed()
        });

        let result = engine.run_test_case(&test_case).await.unwrap();
        assert_eq!(result.outcome, TestOutcome::Passed);
        assert!(teardown_called_clone.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_run_test_case_with_timeout() {
        let options = TestExecutionOptions {
            default_timeout: Duration::from_millis(100),
            ..Default::default()
        };

        let engine = TestEngine::builder().with_options(options).build();

        let context = TestContext::new(TestCategory::Unit, "test_timeout".to_string());
        let test_case = TestCase::new(context, |_| {
            async {
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok(TestResult::new(
                    "test_timeout",
                    TestCategory::Unit,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        });

        let result = engine.run_test_case(&test_case).await.unwrap();
        assert_eq!(result.outcome, TestOutcome::TimedOut);
    }

    #[tokio::test]
    async fn test_run_test_suite() {
        let engine = TestEngine::new();

        let mut suite = TestSuite::new("test_suite");

        let context1 = TestContext::new(TestCategory::Unit, "test_pass".to_string());
        let test_case1 = TestCase::new(context1, |_| {
            async {
                Ok(TestResult::new(
                    "test_pass",
                    TestCategory::Unit,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        });

        let context2 = TestContext::new(TestCategory::Unit, "test_fail".to_string());
        let test_case2 = TestCase::new(context2, |_| {
            async {
                Ok(
                    TestResult::new("test_fail", TestCategory::Unit, TestOutcome::Failed)
                        .with_error("Test failed".to_string()),
                )
            }
            .boxed()
        });

        suite = suite.with_test_case(test_case1).with_test_case(test_case2);

        let result = engine.run_suite(suite).await.unwrap();
        assert_eq!(result.passed, 1);
        assert_eq!(result.failed, 1);
    }
}
