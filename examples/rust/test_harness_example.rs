//! Test Harness Example
//!
//! This example demonstrates how to use the IntelliRouter test harness.

use futures::future::BoxFuture;
use futures::FutureExt;
use intellirouter::modules::test_harness::{
    AssertionError, AssertionHelper, AssertionResult, ConsoleReporter, Environment,
    LocalEnvironment, PluginManager, Reporter, TestCase, TestCategory, TestContext, TestEngine,
    TestEngineBuilder, TestExecutionOptions, TestOutcome, TestResult, TestSuite, TestSuiteResult,
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the environment
    let environment = Box::new(LocalEnvironment::new());

    // Create a plugin manager
    let plugin_manager = Arc::new(PluginManager::new());

    // Create a reporter
    let reporter = Arc::new(ConsoleReporter::new());

    // Create test execution options
    let options = TestExecutionOptions {
        max_parallel_tests: 4,
        default_timeout: Duration::from_secs(30),
        fail_fast: false,
        include_skipped: true,
        retry_failed: false,
        max_retries: 0,
        include_categories: Some(vec![TestCategory::Unit, TestCategory::Integration]),
        exclude_categories: None,
        include_tags: None,
        exclude_tags: None,
        include_tests: None,
        exclude_tests: None,
        shuffle: false,
        shuffle_seed: None,
    };

    // Create a test engine
    let engine = TestEngine::builder()
        .with_options(options)
        .with_environment(environment)
        .with_plugin_manager(plugin_manager)
        .with_reporter(reporter)
        .build();

    // Create test cases
    let test_case1 = TestCase::new(
        TestContext::new(TestCategory::Unit, "test_example_1".to_string()),
        |_ctx| {
            async {
                // This test will pass
                println!("Running test_example_1");
                Ok(TestResult::new(
                    "test_example_1",
                    TestCategory::Unit,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    );

    let test_case2 = TestCase::new(
        TestContext::new(TestCategory::Unit, "test_example_2".to_string()),
        |_ctx| {
            async {
                // This test will fail
                println!("Running test_example_2");
                Ok(
                    TestResult::new("test_example_2", TestCategory::Unit, TestOutcome::Failed)
                        .with_error("Example failure".to_string()),
                )
            }
            .boxed()
        },
    );

    let test_case3 = TestCase::new(
        TestContext::new(TestCategory::Integration, "test_example_3".to_string()),
        |_ctx| {
            async {
                // This test will pass with assertions
                println!("Running test_example_3");

                // Use the assertion helper
                let assertion1 =
                    AssertionHelper::assert_eq(2 + 2, 4, "2 + 2 should equal 4").unwrap();

                let assertion2 = AssertionHelper::assert_true(true, "true should be true").unwrap();

                let assertion3 = AssertionHelper::assert_contains(
                    "hello world",
                    "world",
                    "should contain world",
                )
                .unwrap();

                let mut result = TestResult::new(
                    "test_example_3",
                    TestCategory::Integration,
                    TestOutcome::Passed,
                );

                // Add assertions to the result
                result = result
                    .with_assertion(assertion1)
                    .with_assertion(assertion2)
                    .with_assertion(assertion3);

                Ok(result)
            }
            .boxed()
        },
    );

    // Create a test suite
    let test_suite = TestSuite::new("Example Test Suite")
        .with_description("A simple example test suite")
        .with_test_case(test_case1)
        .with_test_case(test_case2)
        .with_test_case(test_case3)
        .with_parallel(true);

    // Run the test suite
    let result = engine.run_suite(test_suite).await?;

    // Print the results
    println!("\nTest Suite Results:");
    println!("Total Tests: {}", result.total_tests());
    println!("Passed: {}", result.passed);
    println!("Failed: {}", result.failed);
    println!("Skipped: {}", result.skipped);
    println!("Timed Out: {}", result.timed_out);
    println!("Panicked: {}", result.panicked);
    println!(
        "Success Rate: {:.2}%",
        (result.passed as f64 / result.total_tests() as f64) * 100.0
    );

    // Create output directory for reports
    std::fs::create_dir_all("reports")?;

    // Create formatters for different formats
    let json_formatter = intellirouter::modules::test_harness::reporting::JsonFormatter::new();
    let html_formatter = intellirouter::modules::test_harness::reporting::HtmlFormatter::new();
    let markdown_formatter =
        intellirouter::modules::test_harness::reporting::MarkdownFormatter::new();

    // Format the results
    let config = intellirouter::modules::test_harness::reporting::FormatterConfig::default();
    let json_report = json_formatter.format_suite_result(&result, &config)?;
    let html_report = html_formatter.format_suite_result(&result, &config)?;
    let markdown_report = markdown_formatter.format_suite_result(&result, &config)?;

    // Save the reports
    std::fs::write("reports/example_report.json", json_report)?;
    std::fs::write("reports/example_report.html", html_report)?;
    std::fs::write("reports/example_report.md", markdown_report)?;

    println!("\nReports saved to:");
    println!("- reports/example_report.json");
    println!("- reports/example_report.html");
    println!("- reports/example_report.md");

    Ok(())
}
