//! Configuration Testing Framework
//!
//! This module provides utilities for testing different configurations
//! of the IntelliRouter system.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::modules::test_harness::assert::{AssertionContext, AssertionResult};
use crate::modules::test_harness::types::{
    TestCategory, TestContext, TestHarnessError, TestOutcome, TestResult,
};

/// Configuration source
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigSource {
    /// Configuration from a file
    File(PathBuf),
    /// Configuration from environment variables
    Environment,
    /// Configuration from command line arguments
    CommandLine,
    /// Configuration from a string
    String(String),
    /// Configuration from a JSON value
    Json(serde_json::Value),
    /// Configuration from a YAML value
    Yaml(String),
    /// Configuration from a TOML value
    Toml(String),
}

impl fmt::Display for ConfigSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigSource::File(path) => write!(f, "File({})", path.display()),
            ConfigSource::Environment => write!(f, "Environment"),
            ConfigSource::CommandLine => write!(f, "CommandLine"),
            ConfigSource::String(_) => write!(f, "String"),
            ConfigSource::Json(_) => write!(f, "Json"),
            ConfigSource::Yaml(_) => write!(f, "Yaml"),
            ConfigSource::Toml(_) => write!(f, "Toml"),
        }
    }
}

/// Configuration value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValue {
    /// Configuration key
    pub key: String,
    /// Configuration value
    pub value: serde_json::Value,
    /// Configuration source
    pub source: ConfigSource,
}

impl ConfigValue {
    /// Create a new configuration value
    pub fn new(
        key: impl Into<String>,
        value: impl Serialize,
        source: ConfigSource,
    ) -> Result<Self, TestHarnessError> {
        let value = serde_json::to_value(value).map_err(TestHarnessError::SerializationError)?;
        Ok(Self {
            key: key.into(),
            value,
            source,
        })
    }

    /// Get the value as a specific type
    pub fn as_type<T: for<'de> Deserialize<'de>>(&self) -> Result<T, TestHarnessError> {
        serde_json::from_value(self.value.clone()).map_err(TestHarnessError::SerializationError)
    }
}

/// Configuration set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSet {
    /// Configuration name
    pub name: String,
    /// Configuration description
    pub description: Option<String>,
    /// Configuration values
    pub values: HashMap<String, ConfigValue>,
}

impl ConfigSet {
    /// Create a new configuration set
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            values: HashMap::new(),
        }
    }

    /// Set the configuration description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a configuration value
    pub fn with_value(mut self, value: ConfigValue) -> Self {
        self.values.insert(value.key.clone(), value);
        self
    }

    /// Add multiple configuration values
    pub fn with_values(mut self, values: Vec<ConfigValue>) -> Self {
        for value in values {
            self.values.insert(value.key.clone(), value);
        }
        self
    }

    /// Get a configuration value
    pub fn get_value(&self, key: &str) -> Option<&ConfigValue> {
        self.values.get(key)
    }

    /// Get a configuration value as a specific type
    pub fn get_value_as<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<Option<T>, TestHarnessError> {
        if let Some(value) = self.values.get(key) {
            let typed_value = value.as_type()?;
            Ok(Some(typed_value))
        } else {
            Ok(None)
        }
    }

    /// Merge with another configuration set
    pub fn merge(&mut self, other: &ConfigSet) {
        for (key, value) in &other.values {
            self.values.insert(key.clone(), value.clone());
        }
    }

    /// Create a new configuration set by merging with another
    pub fn merged_with(mut self, other: &ConfigSet) -> Self {
        self.merge(other);
        self
    }
}

/// Configuration test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTestResult {
    /// Test name
    pub name: String,
    /// Test description
    pub description: Option<String>,
    /// Configuration set used for the test
    pub config_set: ConfigSet,
    /// Test outcome
    pub outcome: TestOutcome,
    /// Error message if the test failed
    pub error: Option<String>,
    /// Test duration
    pub duration: std::time::Duration,
    /// Assertion results
    pub assertions: Vec<AssertionResult>,
}

impl ConfigTestResult {
    /// Create a new configuration test result
    pub fn new(name: impl Into<String>, config_set: ConfigSet, outcome: TestOutcome) -> Self {
        Self {
            name: name.into(),
            description: None,
            config_set,
            outcome,
            error: None,
            duration: std::time::Duration::from_secs(0),
            assertions: Vec::new(),
        }
    }

    /// Set the test description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the error message
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Set the test duration
    pub fn with_duration(mut self, duration: std::time::Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Add an assertion result
    pub fn with_assertion(mut self, assertion: AssertionResult) -> Self {
        self.assertions.push(assertion);
        self
    }

    /// Add multiple assertion results
    pub fn with_assertions(mut self, assertions: Vec<AssertionResult>) -> Self {
        self.assertions.extend(assertions);
        self
    }

    /// Check if the test passed
    pub fn passed(&self) -> bool {
        self.outcome == TestOutcome::Passed && self.assertions.iter().all(|a| a.passed())
    }

    /// Check if the test failed
    pub fn failed(&self) -> bool {
        self.outcome == TestOutcome::Failed || self.assertions.iter().any(|a| a.failed())
    }
}

/// Configuration test interface
#[async_trait]
pub trait ConfigTest: Send + Sync {
    /// Get the test name
    fn name(&self) -> &str;

    /// Get the test description
    fn description(&self) -> Option<&str>;

    /// Execute the test with a configuration set
    async fn execute(&self, config_set: &ConfigSet) -> Result<ConfigTestResult, TestHarnessError>;
}

/// Configuration test builder
pub struct ConfigTestBuilder {
    /// Test name
    name: String,
    /// Test description
    description: Option<String>,
    /// Test execution function
    execute_fn: Option<
        Box<
            dyn Fn(&ConfigSet) -> BoxFuture<'static, Result<ConfigTestResult, TestHarnessError>>
                + Send
                + Sync,
        >,
    >,
}

impl ConfigTestBuilder {
    /// Create a new configuration test builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            execute_fn: None,
        }
    }

    /// Set the test description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the test execution function
    pub fn with_execute_fn(
        mut self,
        execute_fn: impl Fn(&ConfigSet) -> BoxFuture<'static, Result<ConfigTestResult, TestHarnessError>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        self.execute_fn = Some(Box::new(execute_fn));
        self
    }

    /// Build the configuration test
    pub fn build(self) -> Box<dyn ConfigTest> {
        let execute_fn = self.execute_fn.unwrap_or_else(|| {
            Box::new(|config_set| {
                async move {
                    Ok(ConfigTestResult::new(
                        self.name.clone(),
                        config_set.clone(),
                        TestOutcome::Passed,
                    ))
                }
                .boxed()
            })
        });

        Box::new(BasicConfigTest {
            name: self.name,
            description: self.description,
            execute_fn,
        })
    }
}

/// Basic configuration test implementation
struct BasicConfigTest {
    /// Test name
    name: String,
    /// Test description
    description: Option<String>,
    /// Test execution function
    execute_fn: Box<
        dyn Fn(&ConfigSet) -> BoxFuture<'static, Result<ConfigTestResult, TestHarnessError>>
            + Send
            + Sync,
    >,
}

#[async_trait]
impl ConfigTest for BasicConfigTest {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    async fn execute(&self, config_set: &ConfigSet) -> Result<ConfigTestResult, TestHarnessError> {
        (self.execute_fn)(config_set).await
    }
}

/// Configuration test suite
#[derive(Debug)]
pub struct ConfigTestSuite {
    /// Suite name
    pub name: String,
    /// Suite description
    pub description: Option<String>,
    /// Configuration sets to test
    pub config_sets: Vec<ConfigSet>,
    /// Tests to run
    pub tests: Vec<Box<dyn ConfigTest>>,
    /// Whether to run tests in parallel
    pub parallel: bool,
    /// Whether to fail fast on the first test failure
    pub fail_fast: bool,
}

impl ConfigTestSuite {
    /// Create a new configuration test suite
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            config_sets: Vec::new(),
            tests: Vec::new(),
            parallel: false,
            fail_fast: false,
        }
    }

    /// Set the suite description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a configuration set
    pub fn with_config_set(mut self, config_set: ConfigSet) -> Self {
        self.config_sets.push(config_set);
        self
    }

    /// Add multiple configuration sets
    pub fn with_config_sets(mut self, config_sets: Vec<ConfigSet>) -> Self {
        self.config_sets.extend(config_sets);
        self
    }

    /// Add a test
    pub fn with_test(mut self, test: Box<dyn ConfigTest>) -> Self {
        self.tests.push(test);
        self
    }

    /// Add multiple tests
    pub fn with_tests(mut self, tests: Vec<Box<dyn ConfigTest>>) -> Self {
        self.tests.extend(tests);
        self
    }

    /// Set whether to run tests in parallel
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Set whether to fail fast on the first test failure
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Execute the test suite
    pub async fn execute(&self) -> Result<Vec<ConfigTestResult>, TestHarnessError> {
        info!("Executing configuration test suite: {}", self.name);

        let mut results = Vec::new();

        for config_set in &self.config_sets {
            info!("Testing configuration set: {}", config_set.name);

            for test in &self.tests {
                info!("Executing test: {}", test.name());

                let start_time = std::time::Instant::now();
                let result = match test.execute(config_set).await {
                    Ok(result) => result,
                    Err(e) => {
                        error!("Test failed: {}: {}", test.name(), e);
                        let result = ConfigTestResult::new(
                            test.name(),
                            config_set.clone(),
                            TestOutcome::Failed,
                        )
                        .with_error(format!("Test execution error: {}", e))
                        .with_duration(start_time.elapsed());

                        results.push(result.clone());

                        if self.fail_fast {
                            return Ok(results);
                        } else {
                            continue;
                        }
                    }
                };

                // Check result
                if result.failed() {
                    if self.fail_fast {
                        results.push(result);
                        return Ok(results);
                    }
                }

                results.push(result);
            }
        }

        Ok(results)
    }
}

/// Create a test case from a configuration test suite
pub fn create_test_case_from_config_suite(
    suite: ConfigTestSuite,
) -> crate::modules::test_harness::types::TestCase {
    let suite_name = suite.name.clone();

    crate::modules::test_harness::types::TestCase::new(
        TestContext::new(TestCategory::Integration, suite_name.clone()),
        move |_| {
            let suite = suite.clone();
            async move {
                let start_time = std::time::Instant::now();
                let start_datetime = chrono::Utc::now();

                let results = suite.execute().await?;

                let duration = start_time.elapsed();
                let end_datetime = chrono::Utc::now();

                let all_passed = results.iter().all(|r| r.passed());
                let outcome = if all_passed {
                    TestOutcome::Passed
                } else {
                    TestOutcome::Failed
                };

                let mut test_result =
                    TestResult::new(&suite_name, TestCategory::Integration, outcome)
                        .with_start_time(start_datetime)
                        .with_end_time(end_datetime)
                        .with_duration(duration);

                // Add config test results as custom data
                test_result = test_result
                    .with_custom_data("config_test_results", &results)
                    .map_err(|e| {
                        TestHarnessError::ExecutionError(format!(
                            "Failed to add config test results: {}",
                            e
                        ))
                    })?;

                // Add error message if any tests failed
                if !all_passed {
                    let failed_tests: Vec<String> = results
                        .iter()
                        .filter(|r| r.failed())
                        .map(|r| {
                            if let Some(error) = &r.error {
                                format!("{}: {}", r.name, error)
                            } else {
                                format!("{}: Failed", r.name)
                            }
                        })
                        .collect();

                    test_result = test_result.with_error(format!(
                        "Configuration test suite failed with {} failed tests: {}",
                        failed_tests.len(),
                        failed_tests.join(", ")
                    ));
                }

                Ok(test_result)
            }
            .boxed()
        },
    )
}

/// Create a new configuration test
pub fn create_config_test(name: impl Into<String>) -> ConfigTestBuilder {
    ConfigTestBuilder::new(name)
}

/// Create a new configuration set
pub fn create_config_set(name: impl Into<String>) -> ConfigSet {
    ConfigSet::new(name)
}

/// Create a new configuration value
pub fn create_config_value(
    key: impl Into<String>,
    value: impl Serialize,
    source: ConfigSource,
) -> Result<ConfigValue, TestHarnessError> {
    ConfigValue::new(key, value, source)
}

/// Create a new configuration test suite
pub fn create_config_test_suite(name: impl Into<String>) -> ConfigTestSuite {
    ConfigTestSuite::new(name)
}

/// Load configuration from a file
pub async fn load_config_from_file(path: impl AsRef<Path>) -> Result<ConfigSet, TestHarnessError> {
    let path = path.as_ref();
    let file_name = path
        .file_name()
        .ok_or_else(|| {
            TestHarnessError::ConfigError(format!("Invalid file path: {}", path.display()))
        })?
        .to_string_lossy();

    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| TestHarnessError::IoError(e))?;

    let config_set = match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => {
            let json: serde_json::Value =
                serde_json::from_str(&content).map_err(TestHarnessError::SerializationError)?;
            let mut config_set = ConfigSet::new(file_name.to_string());

            if let serde_json::Value::Object(obj) = &json {
                for (key, value) in obj {
                    let config_value = ConfigValue::new(
                        key.clone(),
                        value.clone(),
                        ConfigSource::File(path.to_path_buf()),
                    )?;
                    config_set = config_set.with_value(config_value);
                }
            }

            config_set
        }
        Some("yaml") | Some("yml") => {
            let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| TestHarnessError::SerializationError(serde_json::Error::custom(e)))?;
            let json = serde_json::to_value(yaml).map_err(TestHarnessError::SerializationError)?;
            let mut config_set = ConfigSet::new(file_name.to_string());

            if let serde_json::Value::Object(obj) = &json {
                for (key, value) in obj {
                    let config_value = ConfigValue::new(
                        key.clone(),
                        value.clone(),
                        ConfigSource::File(path.to_path_buf()),
                    )?;
                    config_set = config_set.with_value(config_value);
                }
            }

            config_set
        }
        Some("toml") => {
            let toml: toml::Value = toml::from_str(&content)
                .map_err(|e| TestHarnessError::SerializationError(serde_json::Error::custom(e)))?;
            let json = serde_json::to_value(toml).map_err(TestHarnessError::SerializationError)?;
            let mut config_set = ConfigSet::new(file_name.to_string());

            if let serde_json::Value::Object(obj) = &json {
                for (key, value) in obj {
                    let config_value = ConfigValue::new(
                        key.clone(),
                        value.clone(),
                        ConfigSource::File(path.to_path_buf()),
                    )?;
                    config_set = config_set.with_value(config_value);
                }
            }

            config_set
        }
        _ => {
            return Err(TestHarnessError::ConfigError(format!(
                "Unsupported file format: {}",
                path.display()
            )));
        }
    };

    Ok(config_set)
}

/// Load configuration from environment variables
pub fn load_config_from_env(prefix: Option<&str>) -> Result<ConfigSet, TestHarnessError> {
    let mut config_set = ConfigSet::new("Environment");

    for (key, value) in std::env::vars() {
        if let Some(prefix) = prefix {
            if !key.starts_with(prefix) {
                continue;
            }
        }

        let config_value = ConfigValue::new(key.clone(), value, ConfigSource::Environment)?;
        config_set = config_set.with_value(config_value);
    }

    Ok(config_set)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future;

    #[tokio::test]
    async fn test_config_set() {
        // Create a configuration set
        let config_set = create_config_set("test_config")
            .with_description("Test configuration")
            .with_value(
                create_config_value("key1", "value1", ConfigSource::String("test".to_string()))
                    .unwrap(),
            )
            .with_value(
                create_config_value("key2", 42, ConfigSource::String("test".to_string())).unwrap(),
            );

        // Check values
        assert_eq!(config_set.name, "test_config");
        assert_eq!(
            config_set.description,
            Some("Test configuration".to_string())
        );
        assert_eq!(config_set.values.len(), 2);

        // Get values
        let value1 = config_set.get_value("key1").unwrap();
        let value2 = config_set.get_value("key2").unwrap();

        assert_eq!(value1.key, "key1");
        assert_eq!(value1.value.as_str().unwrap(), "value1");

        assert_eq!(value2.key, "key2");
        assert_eq!(value2.value.as_i64().unwrap(), 42);

        // Get typed values
        let typed_value1: String = config_set.get_value_as("key1").unwrap().unwrap();
        let typed_value2: i64 = config_set.get_value_as("key2").unwrap().unwrap();

        assert_eq!(typed_value1, "value1");
        assert_eq!(typed_value2, 42);
    }

    #[tokio::test]
    async fn test_config_test() {
        // Create a configuration test
        let test = create_config_test("test_config_test")
            .with_description("Test configuration test")
            .with_execute_fn(|config_set| {
                async move {
                    // Check that the configuration set has the expected values
                    let value1 = config_set.get_value("key1");
                    let value2 = config_set.get_value("key2");

                    if value1.is_none() || value2.is_none() {
                        return Ok(ConfigTestResult::new(
                            "test_config_test",
                            config_set.clone(),
                            TestOutcome::Failed,
                        )
                        .with_error("Missing required configuration values"));
                    }

                    Ok(ConfigTestResult::new(
                        "test_config_test",
                        config_set.clone(),
                        TestOutcome::Passed,
                    ))
                }
                .boxed()
            })
            .build();

        // Create a configuration set
        let config_set = create_config_set("test_config")
            .with_value(
                create_config_value("key1", "value1", ConfigSource::String("test".to_string()))
                    .unwrap(),
            )
            .with_value(
                create_config_value("key2", 42, ConfigSource::String("test".to_string())).unwrap(),
            );

        // Execute the test
        let result = test.execute(&config_set).await.unwrap();

        // Check the result
        assert_eq!(result.name, "test_config_test");
        assert_eq!(result.outcome, TestOutcome::Passed);
    }

    #[tokio::test]
    async fn test_config_test_suite() {
        // Create a configuration test
        let test = create_config_test("test_config_test")
            .with_description("Test configuration test")
            .with_execute_fn(|config_set| {
                async move {
                    // Check that the configuration set has the expected values
                    let value1 = config_set.get_value("key1");
                    let value2 = config_set.get_value("key2");

                    if value1.is_none() || value2.is_none() {
                        return Ok(ConfigTestResult::new(
                            "test_config_test",
                            config_set.clone(),
                            TestOutcome::Failed,
                        )
                        .with_error("Missing required configuration values"));
                    }

                    Ok(ConfigTestResult::new(
                        "test_config_test",
                        config_set.clone(),
                        TestOutcome::Passed,
                    ))
                }
                .boxed()
            })
            .build();

        // Create configuration sets
        let config_set1 = create_config_set("test_config1")
            .with_value(
                create_config_value("key1", "value1", ConfigSource::String("test".to_string()))
                    .unwrap(),
            )
            .with_value(
                create_config_value("key2", 42, ConfigSource::String("test".to_string())).unwrap(),
            );

        let config_set2 = create_config_set("test_config2").with_value(
            create_config_value("key1", "value1", ConfigSource::String("test".to_string()))
                .unwrap(),
        );

        // Create a test suite
        let suite = create_config_test_suite("test_suite")
            .with_description("Test configuration test suite")
            .with_config_set(config_set1)
            .with_config_set(config_set2)
            .with_test(test);

        // Execute the suite
        let results = suite.execute().await.unwrap();

        // Check the results
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].outcome, TestOutcome::Passed);
        assert_eq!(results[1].outcome, TestOutcome::Failed);
    }
}
