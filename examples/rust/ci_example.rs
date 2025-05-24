//! CI Integration Example
//!
//! This example demonstrates how to use the IntelliRouter CI integration system.

use intellirouter::modules::test_harness::{
    benchmark::{BenchmarkConfig, BenchmarkResult, BenchmarkRunner, BenchmarkType},
    ci::{CiConfig, CiEnvironment, CiProvider, CiRunResult, CiRunner},
    reporting::{ReportConfig, ReportGenerator, ReportManager, TestResult, TestRun, TestStatus},
    security::{
        SecurityTestConfig, SecurityTestResult, SecurityTestRunner, SecurityTestType,
        Vulnerability, VulnerabilitySeverity,
    },
    types::TestHarnessError,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("IntelliRouter CI Integration Example");
    println!("====================================");

    // Create a CI environment
    let ci_env = CiEnvironment::new(CiProvider::GitHubActions, "12345")
        .with_job_id("job-1")
        .with_workflow_id("workflow-1")
        .with_repository("intellirouter/intellirouter")
        .with_branch("main")
        .with_commit("abcdef123456")
        .with_pull_request("42")
        .with_tag("v1.0.0")
        .with_runner("ubuntu-latest");

    println!("Created CI environment:");
    println!("  Provider: {}", ci_env.provider);
    println!("  Build ID: {}", ci_env.build_id);
    println!("  Repository: {}", ci_env.repository.as_ref().unwrap());
    println!("  Branch: {}", ci_env.branch.as_ref().unwrap());
    println!("  Commit: {}", ci_env.commit.as_ref().unwrap());
    println!("  Pull Request: {}", ci_env.pull_request.as_ref().unwrap());

    // Create a CI configuration
    let ci_config = CiConfig {
        environment: Some(ci_env),
        output_dir: PathBuf::from("ci-reports"),
        report_formats: vec![
            intellirouter::modules::test_harness::reporting::ExportFormat::Html,
            intellirouter::modules::test_harness::reporting::ExportFormat::Json,
        ],
        fail_on_test_failure: true,
        fail_on_benchmark_regression: true,
        fail_on_security_vulnerability: true,
        upload_artifacts: true,
        artifact_retention_days: Some(30),
        timeout: Duration::from_secs(3600),
        parallel_jobs: 4,
        metadata: HashMap::new(),
    };
    println!("\nCreated CI configuration");

    // Create a report configuration
    let report_config = ReportConfig {
        title: "IntelliRouter CI Report".to_string(),
        description: Some("CI report for IntelliRouter".to_string()),
        output_dir: PathBuf::from("ci-reports"),
        formats: ci_config.report_formats.clone(),
        ..ReportConfig::default()
    };
    println!("Created report configuration");

    // Create a report generator
    let report_generator = Arc::new(ReportGenerator::new(report_config));
    println!("Created report generator");

    // Create a report manager
    let report_manager = Arc::new(ReportManager::new(
        "ci-reports",
        Arc::clone(&report_generator),
    ));
    println!("Created report manager");

    // Create a CI runner
    let ci_runner = CiRunner::new(ci_config.clone(), Arc::clone(&report_manager));
    println!("Created CI runner");

    // Simulate test runs
    println!("\nSimulating test runs...");

    // Create test run 1
    let run_id1 = format!("run-{}-1", Utc::now().timestamp());
    report_manager.start_run(&run_id1, "Unit Tests").await?;

    // Add test results to run 1
    let test1 = TestResult::new("test-1", "Example Test 1", TestStatus::Passed)
        .with_duration(Duration::from_millis(100));

    let test2 = TestResult::new("test-2", "Example Test 2", TestStatus::Failed)
        .with_duration(Duration::from_millis(200));

    report_manager.add_result(test1).await?;
    report_manager.add_result(test2).await?;

    let test_run1 = report_manager.end_run().await?;
    println!("Created test run 1: {}", test_run1.id);

    // Create test run 2
    let run_id2 = format!("run-{}-2", Utc::now().timestamp());
    report_manager
        .start_run(&run_id2, "Integration Tests")
        .await?;

    // Add test results to run 2
    let test3 = TestResult::new("test-3", "Example Test 3", TestStatus::Passed)
        .with_duration(Duration::from_millis(150));

    let test4 = TestResult::new("test-4", "Example Test 4", TestStatus::Passed)
        .with_duration(Duration::from_millis(250));

    report_manager.add_result(test3).await?;
    report_manager.add_result(test4).await?;

    let test_run2 = report_manager.end_run().await?;
    println!("Created test run 2: {}", test_run2.id);

    // Simulate benchmark results
    println!("\nSimulating benchmark results...");

    // Create benchmark result 1
    let benchmark_config1 = BenchmarkConfig::new(
        "throughput-bench",
        "Throughput Benchmark",
        BenchmarkType::Throughput,
    )
    .with_duration(Duration::from_secs(2));

    let benchmark_result1 = BenchmarkResult::new(benchmark_config1.clone())
        .with_total_operations(1000)
        .with_successful_operations(950)
        .with_failed_operations(50)
        .with_throughput(475.0)
        .with_duration(Duration::from_secs(2));

    println!(
        "Created benchmark result 1: {}",
        benchmark_result1.config.id
    );

    // Create benchmark result 2
    let benchmark_config2 =
        BenchmarkConfig::new("latency-bench", "Latency Benchmark", BenchmarkType::Latency)
            .with_duration(Duration::from_secs(2));

    let benchmark_result2 = BenchmarkResult::new(benchmark_config2.clone())
        .with_total_operations(500)
        .with_successful_operations(480)
        .with_failed_operations(20)
        .with_throughput(240.0)
        .with_duration(Duration::from_secs(2));

    println!(
        "Created benchmark result 2: {}",
        benchmark_result2.config.id
    );

    // Simulate security test results
    println!("\nSimulating security test results...");

    // Create security test result 1
    let security_config1 = SecurityTestConfig::new(
        "dependency-scan",
        "Dependency Vulnerability Scan",
        SecurityTestType::DependencyScanning,
        "Cargo.toml",
    )
    .with_description("Scan dependencies for known vulnerabilities");

    let vulnerability1 = Vulnerability::new(
        "DEP-001",
        "Outdated Dependency",
        "Using an outdated version of a dependency with known vulnerabilities",
        VulnerabilitySeverity::Medium,
    )
    .with_location("Cargo.toml")
    .with_line(42)
    .with_cve_id("CVE-2023-12345")
    .with_cvss_score(5.5)
    .with_remediation("Update to the latest version");

    let security_result1 = SecurityTestResult::new(security_config1.clone())
        .with_status(TestStatus::Failed)
        .with_vulnerability(vulnerability1)
        .with_duration(Duration::from_secs(5));

    println!(
        "Created security test result 1: {}",
        security_result1.config.id
    );

    // Create security test result 2
    let security_config2 = SecurityTestConfig::new(
        "secret-scan",
        "Secret Scanning",
        SecurityTestType::SecretScanning,
        ".",
    )
    .with_description("Scan codebase for hardcoded secrets");

    let security_result2 = SecurityTestResult::new(security_config2.clone())
        .with_status(TestStatus::Passed)
        .with_duration(Duration::from_secs(3));

    println!(
        "Created security test result 2: {}",
        security_result2.config.id
    );

    // Create a CI run result
    println!("\nCreating CI run result...");

    let ci_run_result = CiRunResult::new(ci_config)
        .with_start_time(Utc::now() - chrono::Duration::seconds(60))
        .with_end_time(Utc::now())
        .with_duration(Duration::from_secs(60))
        .with_status(TestStatus::Failed)
        .with_test_run(test_run1)
        .with_test_run(test_run2)
        .with_benchmark_result(benchmark_result1)
        .with_benchmark_result(benchmark_result2)
        .with_security_test_result(security_result1)
        .with_security_test_result(security_result2);

    println!("Created CI run result");
    println!("  Status: {}", ci_run_result.status);
    println!("  Duration: {:?}", ci_run_result.duration);
    println!("  Test runs: {}", ci_run_result.test_runs.len());
    println!(
        "  Benchmark results: {}",
        ci_run_result.benchmark_results.len()
    );
    println!(
        "  Security test results: {}",
        ci_run_result.security_test_results.len()
    );

    // Generate reports
    println!("\nGenerating CI reports...");
    report_manager.generate_report().await?;
    println!("CI reports generated successfully in the 'ci-reports' directory");

    println!("\nCI integration example completed successfully!");
    Ok(())
}
