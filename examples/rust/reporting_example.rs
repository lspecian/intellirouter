//! Reporting System Example
//!
//! This example demonstrates how to use the IntelliRouter reporting system.

use intellirouter::modules::test_harness::{
    assert::{assert_that, AssertionResult},
    reporting::{
        ExportFormat, ReportConfig, ReportGenerator, ReportManager, TestResult, TestRun, TestStatus,
    },
    types::TestHarnessError,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("IntelliRouter Reporting System Example");
    println!("======================================");

    // Create a report configuration
    let report_config = ReportConfig {
        title: "IntelliRouter Test Report".to_string(),
        description: Some("Example test report for IntelliRouter".to_string()),
        output_dir: PathBuf::from("reports"),
        formats: vec![
            ExportFormat::Html,
            ExportFormat::Json,
            ExportFormat::Markdown,
        ],
        ..ReportConfig::default()
    };
    println!("Created report configuration");

    // Create a report generator
    let report_generator = Arc::new(ReportGenerator::new(report_config));
    println!("Created report generator");

    // Create a report manager
    let report_manager = ReportManager::new("reports", Arc::clone(&report_generator));
    println!("Created report manager");

    // Start a test run
    let run_id = format!("run-{}", Utc::now().timestamp());
    report_manager
        .start_run(&run_id, "Example Test Run")
        .await?;
    println!("Started test run: {}", run_id);

    // Create some test results
    println!("\nCreating test results...");

    // Test 1: Passed test
    let test1 = TestResult::new("test-1", "Example Test 1", TestStatus::Passed)
        .with_description("This is an example of a passed test")
        .with_duration(Duration::from_millis(100))
        .with_tag("example")
        .with_tag("passed")
        .with_category("unit")
        .with_suite("example-suite")
        .with_file("src/example.rs")
        .with_line(42)
        .with_assertion(assert_that(42).is_equal_to(42))
        .with_assertion(assert_that("hello").contains("he"))
        .with_assertion(assert_that(true).is_true());

    println!("Created test result 1: {}", test1.name);
    report_manager.add_result(test1).await?;

    // Test 2: Failed test
    let test2 = TestResult::new("test-2", "Example Test 2", TestStatus::Failed)
        .with_description("This is an example of a failed test")
        .with_duration(Duration::from_millis(200))
        .with_tag("example")
        .with_tag("failed")
        .with_category("unit")
        .with_suite("example-suite")
        .with_file("src/example.rs")
        .with_line(84)
        .with_assertion(assert_that(42).is_equal_to(42))
        .with_assertion(assert_that("hello").contains("world"))
        .with_assertion(assert_that(false).is_true())
        .with_output("Test output:\nExpected true but got false");

    println!("Created test result 2: {}", test2.name);
    report_manager.add_result(test2).await?;

    // Test 3: Skipped test
    let test3 = TestResult::new("test-3", "Example Test 3", TestStatus::Skipped)
        .with_description("This is an example of a skipped test")
        .with_tag("example")
        .with_tag("skipped")
        .with_category("integration")
        .with_suite("example-suite");

    println!("Created test result 3: {}", test3.name);
    report_manager.add_result(test3).await?;

    // Test 4: Test with children
    let child1 = TestResult::new("test-4-1", "Child Test 1", TestStatus::Passed)
        .with_duration(Duration::from_millis(50))
        .with_assertion(assert_that(1).is_equal_to(1));

    let child2 = TestResult::new("test-4-2", "Child Test 2", TestStatus::Failed)
        .with_duration(Duration::from_millis(75))
        .with_assertion(assert_that(2).is_equal_to(3));

    let test4 = TestResult::new("test-4", "Example Test 4", TestStatus::Failed)
        .with_description("This is an example of a test with children")
        .with_duration(Duration::from_millis(300))
        .with_tag("example")
        .with_tag("parent")
        .with_category("integration")
        .with_suite("example-suite")
        .with_child(child1)
        .with_child(child2);

    println!("Created test result 4: {}", test4.name);
    report_manager.add_result(test4).await?;

    // End the test run
    let test_run = report_manager.end_run().await?;
    println!("\nEnded test run: {}", test_run.id);
    println!("Test run summary:");
    println!("  Total tests: {}", test_run.test_count());
    println!("  Passed tests: {}", test_run.passed_count());
    println!("  Failed tests: {}", test_run.failed_count());
    println!("  Skipped tests: {}", test_run.skipped_count());
    println!("  Pass rate: {:.2}%", test_run.pass_rate() * 100.0);

    // Generate reports
    println!("\nGenerating reports...");
    report_manager.generate_report().await?;
    println!("Reports generated successfully in the 'reports' directory");

    // Print report paths
    println!("\nReport files:");
    println!("  HTML: reports/report_{}.html", run_id);
    println!("  JSON: reports/report_{}.json", run_id);
    println!("  Markdown: reports/report_{}.md", run_id);

    println!("\nReporting system example completed successfully!");
    Ok(())
}
