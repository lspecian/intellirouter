//! Enhanced Testing Example
//!
//! This example demonstrates how to use the enhanced testing framework
//! for IntelliRouter, including scenario-based testing, configuration testing,
//! performance testing, and security testing.

use std::time::Duration;

use intellirouter::modules::test_harness::{
    assert_that,
    config::{create_config_set, create_config_test, create_config_test_suite, ConfigSource},
    create_test_engine, create_test_suite,
    performance::{
        create_load_generator, create_performance_test, create_performance_test_params,
        create_performance_test_suite, MetricType,
    },
    scenario::{create_scenario, create_scenario_step, ScenarioStepStatus},
    security::{
        create_security_test, create_security_test_params, create_security_test_suite,
        VulnerabilitySeverity,
    },
    TestCategory, TestContext, TestHarnessError, TestOutcome, TestResult,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running Enhanced Testing Example");

    // Create a test engine
    let engine = create_test_engine();

    // Create a test suite
    let mut suite = create_test_suite("Enhanced Testing Example")
        .with_description("Demonstrates the enhanced testing framework");

    // Add scenario-based test
    suite = suite.with_test_case(create_scenario_test_case().await?);

    // Add configuration test
    suite = suite.with_test_case(create_config_test_case().await?);

    // Add performance test
    suite = suite.with_test_case(create_performance_test_case().await?);

    // Add security test
    suite = suite.with_test_case(create_security_test_case().await?);

    // Run the test suite
    let result = engine.run_suite(suite).await?;

    // Print the results
    println!("Test suite completed:");
    println!("  Total tests: {}", result.total_tests());
    println!("  Passed: {}", result.passed);
    println!("  Failed: {}", result.failed);
    println!("  Skipped: {}", result.skipped);

    Ok(())
}

/// Create a scenario-based test case
async fn create_scenario_test_case(
) -> Result<intellirouter::modules::test_harness::TestCase, TestHarnessError> {
    println!("Creating scenario-based test case");

    // Create a scenario with multiple steps
    let scenario = create_scenario("API Request Flow")
        .with_description("Tests the flow of an API request through the system")
        .with_step(
            create_scenario_step("initialize_router")
                .with_description("Initialize the router")
                .with_execute_fn(|_| {
                    async move {
                        // Simulate initializing the router
                        tokio::time::sleep(Duration::from_millis(100)).await;

                        Ok(intellirouter::modules::test_harness::scenario::ScenarioStepResult::new(
                            "initialize_router",
                        )
                        .with_status(ScenarioStepStatus::Completed))
                    }
                    .boxed()
                })
                .build(),
        )
        .with_step(
            create_scenario_step("send_request")
                .with_description("Send a request to the router")
                .with_dependency("initialize_router")
                .with_execute_fn(|ctx| {
                    async move {
                        // Check that the previous step completed successfully
                        let prev_step = ctx.get_step_result("initialize_router").unwrap();
                        assert_eq!(prev_step.status, ScenarioStepStatus::Completed);

                        // Simulate sending a request
                        tokio::time::sleep(Duration::from_millis(100)).await;

                        // Create a step result with output data
                        let mut result = intellirouter::modules::test_harness::scenario::ScenarioStepResult::new(
                            "send_request",
                        )
                        .with_status(ScenarioStepStatus::Completed);

                        // Add assertions
                        let assertion = assert_that(true).is_true();
                        result = result.with_assertion(assertion);

                        Ok(result)
                    }
                    .boxed()
                })
                .build(),
        )
        .with_step(
            create_scenario_step("verify_response")
                .with_description("Verify the response from the router")
                .with_dependency("send_request")
                .with_execute_fn(|ctx| {
                    async move {
                        // Check that the previous step completed successfully
                        let prev_step = ctx.get_step_result("send_request").unwrap();
                        assert_eq!(prev_step.status, ScenarioStepStatus::Completed);

                        // Simulate verifying the response
                        tokio::time::sleep(Duration::from_millis(100)).await;

                        // Create a step result with assertions
                        let mut result = intellirouter::modules::test_harness::scenario::ScenarioStepResult::new(
                            "verify_response",
                        )
                        .with_status(ScenarioStepStatus::Completed);

                        // Add assertions
                        let assertion1 = assert_that(200).is_equal_to(200);
                        let assertion2 = assert_that("success").contains("succ");
                        result = result.with_assertion(assertion1).with_assertion(assertion2);

                        Ok(result)
                    }
                    .boxed()
                })
                .build(),
        );

    // Convert the scenario to a test case
    let test_case =
        intellirouter::modules::test_harness::scenario::create_test_case_from_scenario(scenario);

    Ok(test_case)
}

/// Create a configuration test case
async fn create_config_test_case(
) -> Result<intellirouter::modules::test_harness::TestCase, TestHarnessError> {
    println!("Creating configuration test case");

    // Create configuration sets
    let default_config = create_config_set("default")
        .with_description("Default configuration")
        .with_value(
            intellirouter::modules::test_harness::config::create_config_value(
                "max_connections",
                100,
                ConfigSource::String("default".to_string()),
            )?,
        )
        .with_value(
            intellirouter::modules::test_harness::config::create_config_value(
                "timeout",
                30,
                ConfigSource::String("default".to_string()),
            )?,
        );

    let high_load_config = create_config_set("high_load")
        .with_description("High load configuration")
        .with_value(
            intellirouter::modules::test_harness::config::create_config_value(
                "max_connections",
                1000,
                ConfigSource::String("high_load".to_string()),
            )?,
        )
        .with_value(
            intellirouter::modules::test_harness::config::create_config_value(
                "timeout",
                10,
                ConfigSource::String("high_load".to_string()),
            )?,
        );

    // Create configuration tests
    let max_connections_test = create_config_test("max_connections_test")
        .with_description("Test max connections configuration")
        .with_execute_fn(|config_set| {
            async move {
                // Get the max_connections value
                let max_connections: i32 =
                    config_set.get_value_as("max_connections")?.ok_or_else(|| {
                        TestHarnessError::ConfigError(
                            "Missing max_connections configuration".to_string(),
                        )
                    })?;

                // Check that max_connections is within valid range
                let mut result =
                    intellirouter::modules::test_harness::config::ConfigTestResult::new(
                        "max_connections_test",
                        config_set.clone(),
                        if max_connections > 0 && max_connections <= 10000 {
                            TestOutcome::Passed
                        } else {
                            TestOutcome::Failed
                        },
                    );

                if max_connections <= 0 || max_connections > 10000 {
                    result = result.with_error(format!(
                        "Invalid max_connections value: {}. Must be between 1 and 10000",
                        max_connections
                    ));
                }

                Ok(result)
            }
            .boxed()
        })
        .build();

    let timeout_test = create_config_test("timeout_test")
        .with_description("Test timeout configuration")
        .with_execute_fn(|config_set| {
            async move {
                // Get the timeout value
                let timeout: i32 = config_set.get_value_as("timeout")?.ok_or_else(|| {
                    TestHarnessError::ConfigError("Missing timeout configuration".to_string())
                })?;

                // Check that timeout is within valid range
                let mut result =
                    intellirouter::modules::test_harness::config::ConfigTestResult::new(
                        "timeout_test",
                        config_set.clone(),
                        if timeout > 0 && timeout <= 300 {
                            TestOutcome::Passed
                        } else {
                            TestOutcome::Failed
                        },
                    );

                if timeout <= 0 || timeout > 300 {
                    result = result.with_error(format!(
                        "Invalid timeout value: {}. Must be between 1 and 300",
                        timeout
                    ));
                }

                Ok(result)
            }
            .boxed()
        })
        .build();

    // Create a configuration test suite
    let config_suite = create_config_test_suite("Configuration Tests")
        .with_description("Tests for different configurations")
        .with_config_set(default_config)
        .with_config_set(high_load_config)
        .with_test(max_connections_test)
        .with_test(timeout_test);

    // Convert the configuration test suite to a test case
    let test_case =
        intellirouter::modules::test_harness::config::create_test_case_from_config_suite(
            config_suite,
        );

    Ok(test_case)
}

/// Create a performance test case
async fn create_performance_test_case(
) -> Result<intellirouter::modules::test_harness::TestCase, TestHarnessError> {
    println!("Creating performance test case");

    // Create performance test parameters
    let low_load_params = create_performance_test_params()
        .with_concurrent_users(10)
        .with_duration(Duration::from_secs(2))
        .with_ramp_up_time(Duration::from_millis(500))
        .with_think_time(Duration::from_millis(50));

    let high_load_params = create_performance_test_params()
        .with_concurrent_users(50)
        .with_duration(Duration::from_secs(2))
        .with_ramp_up_time(Duration::from_millis(500))
        .with_think_time(Duration::from_millis(20));

    // Create a performance test
    let latency_test = create_performance_test("latency_test")
        .with_description("Test request latency under load")
        .with_execute_fn(|params| {
            async move {
                // Create a load generator
                let load_generator = create_load_generator("test_load_generator");

                // Generate load
                let start_time = std::time::Instant::now();
                let metrics = load_generator
                    .generate_load(params, || async {
                        // Simulate a request with random latency
                        let latency = rand::random::<u64>() % 50 + 10;
                        tokio::time::sleep(Duration::from_millis(latency)).await;
                        Ok(Duration::from_millis(latency))
                    })
                    .await?;

                // Create a performance test result
                let mut result =
                    intellirouter::modules::test_harness::performance::PerformanceTestResult::new(
                        "latency_test",
                        params.clone(),
                        TestOutcome::Passed,
                    )
                    .with_metrics(metrics)
                    .with_duration(start_time.elapsed());

                // Calculate summary statistics
                result.calculate_summary();

                // Check if the test passed based on latency requirements
                if let Some(avg_latency) = result.summary.get("latency_avg") {
                    if *avg_latency > 100.0 {
                        result = result.with_outcome(TestOutcome::Failed).with_error(format!(
                            "Average latency ({} ms) exceeds threshold (100 ms)",
                            avg_latency
                        ));
                    }
                }

                Ok(result)
            }
            .boxed()
        })
        .build();

    // Create a performance test suite
    let perf_suite = create_performance_test_suite("Performance Tests")
        .with_description("Tests for performance under different loads")
        .with_params(low_load_params)
        .with_params(high_load_params)
        .with_test(latency_test);

    // Convert the performance test suite to a test case
    let test_case =
        intellirouter::modules::test_harness::performance::create_test_case_from_performance_suite(
            perf_suite,
        );

    Ok(test_case)
}

/// Create a security test case
async fn create_security_test_case(
) -> Result<intellirouter::modules::test_harness::TestCase, TestHarnessError> {
    println!("Creating security test case");

    // Create security test parameters
    let params = create_security_test_params("http://localhost:8080")
        .with_timeout(Duration::from_secs(30))
        .with_scope("/api")
        .with_excluded_path("/api/health");

    // Create a security test
    let injection_test = create_security_test("injection_test")
        .with_description("Test for SQL injection vulnerabilities")
        .with_execute_fn(|params| {
            async move {
                // Simulate a security scan
                tokio::time::sleep(Duration::from_millis(500)).await;

                // Create a security test result with simulated vulnerabilities
                let vulnerabilities = vec![
                    intellirouter::modules::test_harness::security::create_vulnerability(
                        "SQL-001",
                        "SQL Injection",
                        "Possible SQL injection in login form",
                        VulnerabilitySeverity::Medium,
                        "/api/login",
                    )
                    .with_evidence("' OR 1=1 --")
                    .with_remediation("Use prepared statements"),
                ];

                let mut result =
                    intellirouter::modules::test_harness::security::SecurityTestResult::new(
                        "injection_test",
                        params.clone(),
                        TestOutcome::Passed,
                    )
                    .with_vulnerabilities(vulnerabilities)
                    .with_duration(Duration::from_millis(500));

                // Calculate summary statistics
                if let Err(e) = result.calculate_summary() {
                    return Err(TestHarnessError::ExecutionError(format!(
                        "Failed to calculate summary: {}",
                        e
                    )));
                }

                // Check if the test passed based on vulnerability severity
                if result.has_critical_vulnerabilities() {
                    result = result
                        .with_outcome(TestOutcome::Failed)
                        .with_error("Critical vulnerabilities found");
                } else if result.has_high_vulnerabilities() {
                    result = result
                        .with_outcome(TestOutcome::Failed)
                        .with_error("High severity vulnerabilities found");
                }

                Ok(result)
            }
            .boxed()
        })
        .build();

    // Create a security test suite
    let security_suite = create_security_test_suite("Security Tests")
        .with_description("Tests for security vulnerabilities")
        .with_params(params)
        .with_test(injection_test);

    // Convert the security test suite to a test case
    let test_case =
        intellirouter::modules::test_harness::security::create_test_case_from_security_suite(
            security_suite,
        );

    Ok(test_case)
}
