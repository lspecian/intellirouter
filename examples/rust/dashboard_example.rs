//! Dashboard Example
//!
//! This example demonstrates how to use the IntelliRouter dashboard and metrics system.

use intellirouter::modules::test_harness::{
    assert::{assert_that, AssertionResult},
    dashboard::{Dashboard, DashboardConfig, DashboardPanel, DashboardView},
    metrics::{Metric, MetricCollection, MetricType},
    reporting::{
        DashboardServer, DashboardServerConfig, ExportFormat, HtmlExporter, JsonExporter,
        MarkdownExporter, TestResult, TestRun,
    },
    types::{TestCategory, TestHarnessError, TestOutcome},
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("IntelliRouter Dashboard Example");
    println!("===============================");

    // Create a dashboard server configuration
    let server_config = DashboardServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        title: "IntelliRouter Test Dashboard".to_string(),
        description: Some("Example test dashboard for IntelliRouter".to_string()),
        static_dir: PathBuf::from("src/modules/test_harness/reporting/static"),
        data_dir: PathBuf::from("dashboard/data"),
        refresh_interval: 30,
        theme: "default".to_string(),
        logo: None,
        metadata: HashMap::new(),
        enable_websocket: true,
        enable_auth: false,
        auth_username: None,
        auth_password_hash: None,
        enable_https: false,
        https_cert: None,
        https_key: None,
    };
    println!("Created dashboard server configuration");

    // Create a dashboard server
    let dashboard_server = DashboardServer::new(server_config);
    println!("Created dashboard server");

    // Create test runs
    println!("\nCreating test runs...");

    // Create test run 1
    let run_id1 = format!("run-{}-1", Utc::now().timestamp());
    let mut test_run1 = TestRun::new(&run_id1, "Example Test Run 1")
        .with_start_time(Utc::now())
        .with_duration(Duration::from_secs(5));

    // Add test results to run 1
    let test1 = TestResult::new("test-1", TestCategory::Unit, TestOutcome::Passed)
        .with_name("Example Test 1")
        .with_duration(Duration::from_millis(100))
        .with_assertion(assert_that(42).is_equal_to(42));

    let test2 = TestResult::new("test-2", TestCategory::Unit, TestOutcome::Failed)
        .with_name("Example Test 2")
        .with_duration(Duration::from_millis(200))
        .with_assertion(assert_that("hello").contains("world"))
        .with_error("Expected 'hello' to contain 'world'".to_string());

    test_run1 = test_run1.with_result(test1).with_result(test2);
    test_run1.calculate_counts();
    test_run1 = test_run1.with_end_time(Utc::now());

    println!("Created test run 1: {}", test_run1.id);

    // Create test run 2
    let run_id2 = format!("run-{}-2", Utc::now().timestamp());
    let mut test_run2 = TestRun::new(&run_id2, "Example Test Run 2")
        .with_start_time(Utc::now())
        .with_duration(Duration::from_secs(8));

    // Add test results to run 2
    let test3 = TestResult::new("test-3", TestCategory::Integration, TestOutcome::Passed)
        .with_name("Example Test 3")
        .with_duration(Duration::from_millis(150))
        .with_assertion(assert_that(true).is_true());

    let test4 = TestResult::new("test-4", TestCategory::Integration, TestOutcome::Passed)
        .with_name("Example Test 4")
        .with_duration(Duration::from_millis(250))
        .with_assertion(assert_that("world").contains("world"));

    let test5 = TestResult::new("test-5", TestCategory::Integration, TestOutcome::Skipped)
        .with_name("Example Test 5")
        .with_duration(Duration::from_millis(50));

    test_run2 = test_run2
        .with_result(test3)
        .with_result(test4)
        .with_result(test5);
    test_run2.calculate_counts();
    test_run2 = test_run2.with_end_time(Utc::now());

    println!("Created test run 2: {}", test_run2.id);

    // Add test runs to the dashboard server
    dashboard_server.add_test_run(test_run1.clone()).await?;
    dashboard_server.add_test_run(test_run2.clone()).await?;
    println!("Added test runs to the dashboard server");

    // Start the dashboard server in a separate task
    let server_handle = tokio::spawn(async move {
        if let Err(e) = dashboard_server.start().await {
            eprintln!("Error starting dashboard server: {}", e);
        }
    });

    println!("\nDashboard server started at http://127.0.0.1:8080");
    println!("Press Ctrl+C to stop the server");

    // Wait for the server to exit
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            println!("Received Ctrl+C, shutting down...");
        }
        Err(err) => {
            eprintln!("Error waiting for Ctrl+C: {}", err);
        }
    }

    // Abort the server task
    server_handle.abort();

    println!("\nDashboard example completed successfully!");
    Ok(())
}
