//! Test Harness Module
//!
//! This module provides a comprehensive testing harness for IntelliRouter.
//! It supports various test categories, including unit, integration, end-to-end,
//! performance, and security tests.

pub mod assert;
pub mod config;
pub mod engine;
pub mod environment;
pub mod error_recovery_tests;
pub mod integration_tests;
pub mod load_tests;
pub mod mock;
pub mod performance;
pub mod plugins;
pub mod reporting;
pub mod scenario;
pub mod security;
pub mod types;
pub mod utils;

pub use assert::{
    assert_context, assert_that, AssertionBuilder, AssertionContext, AssertionResult,
};
pub use config::{
    create_config_set, create_config_test, create_config_test_suite, create_config_value,
    create_test_case_from_config_suite, ConfigSet, ConfigSource, ConfigTest, ConfigTestParams,
    ConfigTestResult, ConfigTestSuite, ConfigValue,
};
pub use engine::{TestEngine, TestEngineBuilder, TestExecutionOptions};
pub use environment::{Environment, EnvironmentExt, LocalEnvironment};
pub use error_recovery_tests::create_error_recovery_test_suite;
pub use integration_tests::{
    create_integration_test_suite,
    error_recovery_integration_tests::create_error_recovery_integration_test_suite,
};
pub use load_tests::create_load_test_suite;
pub use mock::{
    create_http_mock, create_mock_manager, create_mock_server, create_network_mock,
    create_service_mock, create_storage_mock, HttpMock, MockFactory, MockManager, MockServer,
};
pub use performance::{
    create_load_generator, create_metric, create_performance_test, create_performance_test_params,
    create_performance_test_suite, create_test_case_from_performance_suite, LoadGenerator, Metric,
    MetricType, PerformanceTest, PerformanceTestParams, PerformanceTestResult,
    PerformanceTestSuite,
};
pub use plugins::{Plugin, PluginManager};
pub use reporting::Reporter;
pub use scenario::{
    create_scenario, create_scenario_step, create_test_case_from_scenario, Scenario,
    ScenarioContext, ScenarioStep, ScenarioStepBuilder, ScenarioStepResult, ScenarioStepStatus,
};
pub use security::{
    create_security_test, create_security_test_params, create_security_test_suite,
    create_test_case_from_security_suite, create_vulnerability, SecurityTest, SecurityTestParams,
    SecurityTestResult, SecurityTestSuite, Vulnerability, VulnerabilitySeverity,
};
pub use types::{
    AssertionError, TestCase, TestCategory, TestContext, TestHarnessError, TestOutcome, TestResult,
    TestSuite, TestSuiteResult,
};
pub use utils::AssertionHelper;

/// Create a new test engine with default options
pub fn create_test_engine() -> TestEngine {
    TestEngine::new()
}

/// Create a new test engine builder
pub fn create_test_engine_builder() -> TestEngineBuilder {
    TestEngine::builder()
}

/// Create a new test suite
pub fn create_test_suite(name: impl Into<String>) -> TestSuite {
    TestSuite::new(name)
}

/// Create a new test case
pub fn create_test_case(
    category: TestCategory,
    name: impl Into<String>,
    test_fn: impl Fn(
            &TestContext,
        ) -> futures::future::BoxFuture<'static, Result<TestResult, TestHarnessError>>
        + Send
        + Sync
        + 'static,
) -> TestCase {
    let context = TestContext::new(category, name.into());
    TestCase::new(context, test_fn)
}

/// Create a new test result
pub fn create_test_result(
    name: impl Into<String>,
    category: TestCategory,
    outcome: TestOutcome,
) -> TestResult {
    TestResult::new(name, category, outcome)
}

/// Helper functions for creating mock objects
pub mod mock_helpers {
    use super::*;

    /// Create a new HTTP mock
    pub fn create_http_mock(name: impl Into<String>) -> mock::HttpMockBuilder {
        let manager = create_mock_manager();
        mock::HttpMockBuilder::new(name.into(), Arc::new(manager))
    }

    /// Create a new network mock
    pub fn create_network_mock(name: impl Into<String>) -> mock::NetworkMockBuilder {
        let manager = create_mock_manager();
        mock::NetworkMockBuilder::new(name.into(), Arc::new(manager))
    }

    /// Create a new service mock
    pub fn create_service_mock(name: impl Into<String>) -> mock::ServiceMockBuilder {
        let manager = create_mock_manager();
        mock::ServiceMockBuilder::new(name.into(), Arc::new(manager))
    }

    /// Create a new storage mock
    pub fn create_storage_mock(name: impl Into<String>) -> mock::StorageMockBuilder {
        let manager = create_mock_manager();
        mock::StorageMockBuilder::new(name.into(), Arc::new(manager))
    }

    /// Create a new mock manager
    pub fn create_mock_manager() -> mock::MockManager {
        mock::MockManager::new()
    }

    /// Create a new mock server
    pub fn create_mock_server(
        address: impl Into<String>,
        port: u16,
        manager: std::sync::Arc<mock::MockManager>,
    ) -> mock::MockServer {
        mock::MockServer::new(address, port, manager)
    }
}
