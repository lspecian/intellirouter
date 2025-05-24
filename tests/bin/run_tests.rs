//! Test Runner
//!
//! This binary runs the enhanced tests for error conditions, recovery scenarios,
//! load testing, concurrency testing, and integration tests.
//!
//! This binary is only available when the `test-utils` feature is enabled.
#![cfg(feature = "test-utils")]

use intellirouter;
use std::env;
use std::process;
use tokio;

// Define our own simplified test harness types and functions
use std::future::Future;
use std::pin::Pin;

/// Test outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestOutcome {
    /// Test passed
    Passed,
    /// Test failed
    Failed,
    /// Test skipped
    Skipped,
}

/// Test category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestCategory {
    /// Integration tests
    Integration,
    /// Error recovery tests
    ErrorRecovery,
    /// Load tests
    Load,
}

/// Test result
#[derive(Debug)]
pub struct TestResult {
    /// Test name
    pub name: String,
    /// Test category
    pub category: TestCategory,
    /// Test outcome
    pub outcome: TestOutcome,
}

/// Test case
pub struct TestCase {
    /// Test name
    pub name: String,
    /// Test category
    pub category: TestCategory,
    /// Test function
    pub test_fn: Box<
        dyn Fn() -> Pin<Box<dyn Future<Output = Result<TestResult, String>> + Send>> + Send + Sync,
    >,
}

/// Test suite
pub struct TestSuite {
    /// Suite name
    pub name: String,
    /// Suite description
    pub description: Option<String>,
    /// Test cases
    pub test_cases: Vec<TestCase>,
}

impl Clone for TestSuite {
    fn clone(&self) -> Self {
        // We can't clone the test cases directly, so we create an empty vector
        // This is fine for our use case since we only need to clone the suite
        // to add test cases to it, not to clone the test cases themselves
        Self {
            name: self.name.clone(),
            description: self.description.clone(),
            test_cases: Vec::new(),
        }
    }
}

impl TestSuite {
    /// Create a new test suite
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            test_cases: Vec::new(),
        }
    }

    /// Set the suite description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a test case
    pub fn with_test_case(mut self, test_case: TestCase) -> Self {
        self.test_cases.push(test_case);
        self
    }
}

/// Test suite result
pub struct TestSuiteResult {
    /// Suite name
    pub name: String,
    /// Number of passed tests
    pub passed: usize,
    /// Number of failed tests
    pub failed: usize,
    /// Number of skipped tests
    pub skipped: usize,
}

/// Test engine
pub struct TestEngine;

impl TestEngine {
    /// Create a new test engine
    pub fn new() -> Self {
        Self
    }

    /// Run a test suite
    pub async fn run_suite(&self, mut suite: TestSuite) -> Result<TestSuiteResult, String> {
        println!(
            "Running test suite: {} with {} test cases",
            suite.name,
            suite.test_cases.len()
        );

        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        // Take ownership of the test cases
        let test_cases = std::mem::take(&mut suite.test_cases);
        println!("Taken {} test cases from suite", test_cases.len());
        for test_case in test_cases {
            println!("Running test: {}", test_case.name);

            match (test_case.test_fn)().await {
                Ok(result) => match result.outcome {
                    TestOutcome::Passed => {
                        println!("Test passed: {}", test_case.name);
                        passed += 1;
                    }
                    TestOutcome::Failed => {
                        println!("Test failed: {}", test_case.name);
                        failed += 1;
                    }
                    TestOutcome::Skipped => {
                        println!("Test skipped: {}", test_case.name);
                        skipped += 1;
                    }
                },
                Err(e) => {
                    println!("Test error: {} - {}", test_case.name, e);
                    failed += 1;
                }
            }
        }

        Ok(TestSuiteResult {
            name: suite.name,
            passed,
            failed,
            skipped,
        })
    }
}

/// Create a test suite for error recovery tests
pub fn create_error_recovery_test_suite() -> TestSuite {
    TestSuite::new("Error Recovery Tests")
        .with_description("Tests for error conditions and recovery scenarios")
}

/// Create a test suite for load testing
pub fn create_load_test_suite() -> TestSuite {
    TestSuite::new("Load Tests").with_description("Tests for load testing and concurrency testing")
}

/// Create a test suite for integration tests
pub fn create_integration_test_suite() -> TestSuite {
    println!("Creating integration test suite");

    // Create a test case for the circuit breaker
    let circuit_breaker_test = TestCase {
        name: "Circuit Breaker State Transitions".to_string(),
        category: TestCategory::Integration,
        test_fn: Box::new(|| {
            Box::pin(async {
                use intellirouter::modules::ipc::resilient::circuit_breaker::{
                    CircuitBreaker, CircuitState,
                };
                use intellirouter::modules::ipc::IpcError;
                use intellirouter::modules::router_core::retry::{
                    CircuitBreakerConfig, DegradedServiceMode,
                };
                use std::time::Duration;
                use tokio::time::sleep;

                println!("Starting circuit breaker test");

                // Create a circuit breaker with a custom configuration for testing
                // Using small thresholds and timeout for faster testing
                let config = CircuitBreakerConfig {
                    failure_threshold: 3,  // Open after 3 failures
                    success_threshold: 2,  // Close after 2 successes
                    reset_timeout_ms: 100, // 100ms timeout for half-open transition
                    enabled: true,
                };

                let degraded_mode = DegradedServiceMode::FailFast;
                let circuit_breaker = CircuitBreaker::new(config, degraded_mode);

                // Verify initial state is Closed
                assert_eq!(
                    circuit_breaker.state(),
                    CircuitState::Closed,
                    "Circuit breaker should start in Closed state"
                );
                assert!(
                    circuit_breaker.allow_execution(),
                    "Circuit breaker should allow execution in Closed state"
                );

                println!("Circuit breaker initialized in Closed state");

                // Simulate failures to trigger circuit to open
                println!("Simulating failures to open the circuit");
                let error = IpcError::ConnectionError("Test connection error".to_string());

                for i in 1..=3 {
                    circuit_breaker.record_failure(&error);
                    println!("Recorded failure {}", i);
                }

                // Verify circuit is now Open
                assert_eq!(
                    circuit_breaker.state(),
                    CircuitState::Open,
                    "Circuit breaker should be Open after 3 failures"
                );
                assert!(
                    !circuit_breaker.allow_execution(),
                    "Circuit breaker should not allow execution in Open state"
                );

                println!("Circuit breaker is now Open");

                // Wait for reset timeout to elapse
                println!("Waiting for reset timeout to elapse (100ms)");
                sleep(Duration::from_millis(150)).await;

                // Verify circuit transitions to HalfOpen and allows execution
                assert!(
                    circuit_breaker.allow_execution(),
                    "Circuit breaker should allow execution after timeout"
                );
                assert_eq!(
                    circuit_breaker.state(),
                    CircuitState::HalfOpen,
                    "Circuit breaker should be HalfOpen after timeout"
                );

                println!("Circuit breaker is now HalfOpen");

                // Simulate successes to close the circuit
                println!("Simulating successes to close the circuit");
                for i in 1..=2 {
                    circuit_breaker.record_success();
                    println!("Recorded success {}", i);
                }

                // Verify circuit is now Closed
                assert_eq!(
                    circuit_breaker.state(),
                    CircuitState::Closed,
                    "Circuit breaker should be Closed after 2 successes"
                );
                assert!(
                    circuit_breaker.allow_execution(),
                    "Circuit breaker should allow execution in Closed state"
                );

                println!("Circuit breaker is now Closed");

                // Test failure in HalfOpen state
                println!("Testing failure in HalfOpen state");

                // First, get back to HalfOpen state
                for _ in 1..=3 {
                    circuit_breaker.record_failure(&error);
                }
                sleep(Duration::from_millis(150)).await;

                // Call allow_execution() which triggers the transition to HalfOpen if timeout elapsed
                let allowed = circuit_breaker.allow_execution();
                println!("After timeout, allow_execution() returned: {}", allowed);

                assert!(
                    allowed,
                    "Circuit breaker should allow execution after timeout"
                );
                assert_eq!(
                    circuit_breaker.state(),
                    CircuitState::HalfOpen,
                    "Circuit breaker should be HalfOpen"
                );

                // Now record a failure in HalfOpen state
                circuit_breaker.record_failure(&error);

                // Verify circuit is Open again
                assert_eq!(
                    circuit_breaker.state(),
                    CircuitState::Open,
                    "Circuit breaker should be Open after failure in HalfOpen state"
                );

                println!("Circuit breaker test completed successfully");

                Ok(TestResult {
                    name: "Circuit Breaker State Transitions".to_string(),
                    category: TestCategory::Integration,
                    outcome: TestOutcome::Passed,
                })
            })
        }),
    };

    // Create the suite and add the test case
    let suite = TestSuite::new("Integration Tests")
        .with_description("Tests for integration between components")
        .with_test_case(circuit_breaker_test);

    println!(
        "Integration test suite created with {} test cases",
        suite.test_cases.len()
    );

    suite
}
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "test" => {
            // Run tests
            if args.len() < 3 {
                error!("Missing test suite name");
                print_usage();
                process::exit(1);
            }

            let suite_names = args[2].split(',').collect::<Vec<_>>();
            if suite_names.is_empty() {
                error!("No test suites specified");
                print_usage();
                process::exit(1);
            }

            // Create test engine
            let engine = TestEngine::new();

            // No need to create test suites in advance anymore

            // Run specified test suites
            let mut success = true;
            for suite_name in suite_names {
                info!("Running test suite: {}", suite_name);

                // Create and run the appropriate test suite directly
                let result = match suite_name {
                    "error_recovery" => {
                        let suite = create_error_recovery_test_suite();
                        engine.run_suite(suite).await
                    }
                    "load" => {
                        let suite = create_load_test_suite();
                        engine.run_suite(suite).await
                    }
                    "integration" => {
                        let suite = create_integration_test_suite();
                        println!(
                            "Running integration suite with {} test cases",
                            suite.test_cases.len()
                        );
                        engine.run_suite(suite).await
                    }
                    "all" => {
                        let suite = create_all_test_suite();
                        engine.run_suite(suite).await
                    }
                    _ => {
                        error!("Unknown test suite: {}", suite_name);
                        print_usage();
                        process::exit(1);
                    }
                };

                match result {
                    Ok(result) => {
                        info!(
                            "Test suite {} completed: {} passed, {} failed, {} skipped",
                            suite_name, result.passed, result.failed, result.skipped
                        );
                        if result.failed > 0 {
                            success = false;
                        }
                    }
                    Err(e) => {
                        error!("Failed to run test suite {}: {}", suite_name, e);
                        success = false;
                    }
                }
            }

            if !success {
                process::exit(1);
            }
        }
        "list" => {
            // List available test suites
            info!("Available test suites:");
            info!("  error_recovery - Tests for error conditions and recovery scenarios");
            info!("  load - Tests for load testing and concurrency testing");
            info!("  integration - Tests for integration between components");
            info!("  all - All test suites combined");
        }
        _ => {
            error!("Unknown command: {}", args[1]);
            print_usage();
            process::exit(1);
        }
    }
}

/// Print usage information
fn print_usage() {
    println!("Usage: run_tests <command> [options]");
    println!();
    println!("Commands:");
    println!("  test <suite>   Run the specified test suite");
    println!("  list           List available test suites");
    println!();
    println!("Options:");
    println!("  <suite>        Comma-separated list of test suites to run");
    println!("                 Available suites: error_recovery, load, integration, all");
}

/// Create a test suite with all tests
fn create_all_test_suite() -> TestSuite {
    let mut suite = TestSuite::new("All Tests").with_description("All enhanced tests combined");

    // Add all test cases from other suites
    let error_recovery_suite = create_error_recovery_test_suite();
    let load_suite = create_load_test_suite();

    // We don't need to create a new integration suite here since we already have one
    // Just use the test cases from the existing suites

    for test_case in error_recovery_suite.test_cases {
        suite = suite.with_test_case(test_case);
    }

    for test_case in load_suite.test_cases {
        suite = suite.with_test_case(test_case);
    }

    // Get the integration test cases directly
    let integration_suite = create_integration_test_suite();
    for test_case in integration_suite.test_cases {
        suite = suite.with_test_case(test_case);
    }

    suite
}
