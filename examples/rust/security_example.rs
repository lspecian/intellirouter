//! Security Testing Example
//!
//! This example demonstrates how to use the IntelliRouter security testing system.

use intellirouter::modules::test_harness::{
    reporting::{ReportConfig, ReportGenerator, ReportManager, TestStatus},
    security::{
        SecurityTestConfig, SecurityTestRunner, SecurityTestType, Vulnerability,
        VulnerabilitySeverity,
    },
    types::TestHarnessError,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("IntelliRouter Security Testing Example");
    println!("======================================");

    // Create a report configuration
    let report_config = ReportConfig {
        title: "IntelliRouter Security Report".to_string(),
        description: Some("Example security report for IntelliRouter".to_string()),
        output_dir: PathBuf::from("reports"),
        formats: vec![intellirouter::modules::test_harness::reporting::ExportFormat::Html],
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
    let run_id = format!("security-{}", Utc::now().timestamp());
    report_manager
        .start_run(&run_id, "Security Test Run")
        .await?;
    println!("Started security test run: {}", run_id);

    // Create security test configurations
    println!("\nCreating security test configurations...");

    // Dependency scanning test
    let dependency_scan_config = SecurityTestConfig::new(
        "dependency-scan",
        "Dependency Vulnerability Scan",
        SecurityTestType::DependencyScanning,
        "Cargo.toml",
    )
    .with_description("Scan dependencies for known vulnerabilities")
    .with_timeout(Duration::from_secs(30))
    .with_severity_threshold(VulnerabilitySeverity::High)
    .with_tag("dependencies")
    .with_tag("security");

    println!("Created dependency scanning test configuration");

    // Secret scanning test
    let secret_scan_config = SecurityTestConfig::new(
        "secret-scan",
        "Secret Scanning",
        SecurityTestType::SecretScanning,
        ".",
    )
    .with_description("Scan codebase for hardcoded secrets")
    .with_timeout(Duration::from_secs(30))
    .with_severity_threshold(VulnerabilitySeverity::Medium)
    .with_tag("secrets")
    .with_tag("security");

    println!("Created secret scanning test configuration");

    // Create security test functions
    println!("\nCreating security test functions...");

    // Dependency scanning function
    let dependency_scan_fn = || {
        // Simulate dependency scanning
        println!("  Scanning dependencies for vulnerabilities...");

        // Simulate finding vulnerabilities
        let vulnerabilities = vec![
            Vulnerability::new(
                "DEP-001",
                "Outdated Dependency",
                "Using an outdated version of a dependency with known vulnerabilities",
                VulnerabilitySeverity::Medium,
            )
            .with_location("Cargo.toml")
            .with_line(42)
            .with_cve_id("CVE-2023-12345")
            .with_cvss_score(5.5)
            .with_remediation("Update to the latest version"),
            Vulnerability::new(
                "DEP-002",
                "Vulnerable Dependency",
                "Using a dependency with a known security vulnerability",
                VulnerabilitySeverity::High,
            )
            .with_location("Cargo.toml")
            .with_line(56)
            .with_cve_id("CVE-2023-67890")
            .with_cvss_score(8.2)
            .with_remediation("Replace with a secure alternative"),
        ];

        Ok(vulnerabilities)
    };

    // Secret scanning function
    let secret_scan_fn = || {
        // Simulate secret scanning
        println!("  Scanning codebase for hardcoded secrets...");

        // Simulate finding secrets
        let vulnerabilities = vec![
            Vulnerability::new(
                "SEC-001",
                "Hardcoded API Key",
                "API key hardcoded in source code",
                VulnerabilitySeverity::High,
            )
            .with_location("src/config.rs")
            .with_line(123)
            .with_cwe_id("CWE-798")
            .with_remediation("Move to environment variables or secure storage"),
            Vulnerability::new(
                "SEC-002",
                "Hardcoded Password",
                "Password hardcoded in test file",
                VulnerabilitySeverity::Medium,
            )
            .with_location("tests/integration_test.rs")
            .with_line(45)
            .with_cwe_id("CWE-798")
            .with_remediation("Use test fixtures or environment variables"),
            Vulnerability::new(
                "SEC-003",
                "Hardcoded JWT Secret",
                "JWT signing secret hardcoded in source code",
                VulnerabilitySeverity::High,
            )
            .with_location("src/auth.rs")
            .with_line(78)
            .with_cwe_id("CWE-798")
            .with_remediation("Move to secure configuration management"),
        ];

        Ok(vulnerabilities)
    };

    // Create security test runners
    println!("\nCreating security test runners...");

    let dependency_scan_runner =
        SecurityTestRunner::new(dependency_scan_config, dependency_scan_fn);
    println!("Created dependency scanning test runner");

    let secret_scan_runner = SecurityTestRunner::new(secret_scan_config, secret_scan_fn);
    println!("Created secret scanning test runner");

    // Run security tests
    println!("\nRunning security tests...");

    println!("Running dependency scanning test...");
    let dependency_scan_result = dependency_scan_runner.run().await?;
    println!("Dependency scanning test completed");
    println!("  Status: {}", dependency_scan_result.status);
    println!(
        "  Vulnerabilities found: {}",
        dependency_scan_result.vulnerabilities.len()
    );

    for (i, vuln) in dependency_scan_result.vulnerabilities.iter().enumerate() {
        println!(
            "  Vulnerability #{}: {} [{}]",
            i + 1,
            vuln.name,
            vuln.severity
        );
        println!("    Description: {}", vuln.description);
        if let Some(location) = &vuln.location {
            if let Some(line) = vuln.line {
                println!("    Location: {}:{}", location, line);
            } else {
                println!("    Location: {}", location);
            }
        }
        if let Some(remediation) = &vuln.remediation {
            println!("    Remediation: {}", remediation);
        }
    }

    println!("\nRunning secret scanning test...");
    let secret_scan_result = secret_scan_runner.run().await?;
    println!("Secret scanning test completed");
    println!("  Status: {}", secret_scan_result.status);
    println!(
        "  Vulnerabilities found: {}",
        secret_scan_result.vulnerabilities.len()
    );

    for (i, vuln) in secret_scan_result.vulnerabilities.iter().enumerate() {
        println!(
            "  Vulnerability #{}: {} [{}]",
            i + 1,
            vuln.name,
            vuln.severity
        );
        println!("    Description: {}", vuln.description);
        if let Some(location) = &vuln.location {
            if let Some(line) = vuln.line {
                println!("    Location: {}:{}", location, line);
            } else {
                println!("    Location: {}", location);
            }
        }
        if let Some(remediation) = &vuln.remediation {
            println!("    Remediation: {}", remediation);
        }
    }

    // Add security test results to the test run
    println!("\nAdding security test results to test run...");

    let dependency_scan_test_result = dependency_scan_result.to_test_result();
    report_manager
        .add_result(dependency_scan_test_result)
        .await?;
    println!("Added dependency scanning test result to test run");

    let secret_scan_test_result = secret_scan_result.to_test_result();
    report_manager.add_result(secret_scan_test_result).await?;
    println!("Added secret scanning test result to test run");

    // End the test run
    let test_run = report_manager.end_run().await?;
    println!("\nEnded security test run: {}", test_run.id);

    // Generate reports
    println!("\nGenerating security reports...");
    report_manager.generate_report().await?;
    println!("Security reports generated successfully in the 'reports' directory");

    println!("\nSecurity testing example completed successfully!");
    Ok(())
}
