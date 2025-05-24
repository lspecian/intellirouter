# IntelliRouter Test Harness

The IntelliRouter Test Harness is a comprehensive, modular testing framework designed to support all test categories (unit, integration, end-to-end, performance, security) with a plugin-based architecture for extensibility.

## Features

- **Modular Test Execution Engine**: Supports different test categories with a consistent interface
- **Plugin-Based Architecture**: Easily extend the test harness with custom functionality
- **Environment Management**: Automated setup and teardown of test environments
- **Test Data Management**: Utilities for loading and managing test data
- **Mocking/Stubbing Framework**: Create mock objects for testing
- **Assertion Libraries**: Comprehensive assertion utilities
- **Reporting System**: Generate detailed test reports in various formats
- **Parallel Execution**: Run tests in parallel for faster execution
- **Dependency Resolution**: Automatically resolve test dependencies

## Usage

### Basic Example

```rust
use intellirouter::modules::test_harness::{
    engine::TestEngine,
    types::{TestCase, TestCategory, TestContext, TestOutcome, TestResult, TestSuite},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a test engine
    let engine = TestEngine::new();

    // Create a test case
    let test_case = TestCase::new(
        TestContext::new(TestCategory::Unit, "test_example".to_string()),
        |_ctx| {
            // Test implementation
            Ok(TestResult::new(
                "test_example",
                TestCategory::Unit,
                TestOutcome::Passed,
            ))
        },
    );

    // Create a test suite
    let test_suite = TestSuite::new("Example Test Suite")
        .with_description("A simple example test suite")
        .with_test_case(test_case);

    // Run the test suite
    let result = engine.run_suite(test_suite).await?;

    // Print the results
    println!("Total Tests: {}", result.total_tests());
    println!("Passed: {}", result.passed);
    println!("Failed: {}", result.failed);

    Ok(())
}
```

### Advanced Example

For a more advanced example, see [examples/rust/test_harness_example.rs](../examples/rust/test_harness_example.rs).

## Components

### Test Engine

The test engine is responsible for executing tests and managing the test lifecycle. It supports parallel and sequential execution of tests, with dependency resolution.

```rust
let engine = TestEngine::builder()
    .with_options(options)
    .with_environment(environment)
    .with_plugin_manager(plugin_manager)
    .with_reporter(reporter)
    .build();
```

### Test Cases

Test cases define individual tests to be executed. They include a test context and a test function.

```rust
let test_case = TestCase::new(
    TestContext::new(TestCategory::Unit, "test_example".to_string()),
    |ctx| {
        // Test implementation
        Ok(TestResult::new(
            "test_example",
            TestCategory::Unit,
            TestOutcome::Passed,
        ))
    },
);
```

### Test Suites

Test suites group related test cases together. They can have setup and teardown functions, and can be executed in parallel.

```rust
let test_suite = TestSuite::new("Example Test Suite")
    .with_description("A simple example test suite")
    .with_test_case(test_case1)
    .with_test_case(test_case2)
    .with_parallel(true);
```

### Environment Management

The environment manager handles setting up and tearing down test environments. It supports temporary directories, environment variables, and Docker services.

```rust
let environment = Environment::builder()
    .with_temp_dir(true)
    .with_cleanup(true)
    .with_env_var("TEST_ENV", "example")
    .build()?;
```

### Plugin System

The plugin system allows extending the test harness with custom functionality. Plugins can add new test types, assertions, reporters, and more.

```rust
let plugin_manager = Arc::new(PluginManager::with_environment(environment.clone()));
plugin_manager.register_plugin(Arc::new(MyCustomPlugin::new()))?;
```

### Reporting

The reporting system generates detailed test reports in various formats, including JSON, HTML, Markdown, and more.

```rust
let reporter = Arc::new(ConsoleReporter::new());
let report = Report::builder()
    .with_title("Example Test Report")
    .with_description("A report for the example test suite")
    .with_suite_result(result)
    .build();
report.save("example_report.json", ReportFormat::Json)?;
```

### Dashboard

The dashboard system provides a web-based UI for visualizing test results, metrics, and trends. It includes:

- Interactive dashboards showing test coverage, pass/fail rates, and performance metrics
- Historical reporting with trend analysis
- Test flakiness detection
- Notification mechanisms for test failures
- Integration with CI/CD pipelines
- APIs for programmatic access to test results

```rust
// Create a dashboard server configuration
let server_config = DashboardServerConfig {
    host: "127.0.0.1".to_string(),
    port: 8080,
    title: "IntelliRouter Test Dashboard".to_string(),
    description: Some("Test dashboard for IntelliRouter".to_string()),
    static_dir: PathBuf::from("src/modules/test_harness/reporting/static"),
    data_dir: PathBuf::from("dashboard/data"),
    refresh_interval: 30,
    theme: "default".to_string(),
    enable_websocket: true,
    // ... other configuration options
};

// Create a dashboard server
let dashboard_server = DashboardServer::new(server_config);

// Add test runs to the dashboard
dashboard_server.add_test_run(test_run).await?;

// Start the dashboard server
tokio::spawn(async move {
    if let Err(e) = dashboard_server.start().await {
        eprintln!("Error starting dashboard server: {}", e);
    }
});

println!("Dashboard server started at http://127.0.0.1:8080");
```

For a complete example, see [examples/rust/dashboard_example.rs](../examples/rust/dashboard_example.rs).

### Assertions

The assertion library provides utilities for making assertions in tests.

```rust
AssertionHelper::assert_eq(2 + 2, 4, "2 + 2 should equal 4")?;
AssertionHelper::assert_true(true, "true should be true")?;
AssertionHelper::assert_contains("hello world", "world", "should contain world")?;
```

## Test Categories

The test harness supports the following test categories:

- **Unit**: Test individual components in isolation
- **Integration**: Test how components work together
- **EndToEnd**: Test the entire system
- **Performance**: Measure system performance
- **Security**: Verify system security
- **Custom**: Define custom test categories

## Test Outcomes

Tests can have the following outcomes:

- **Passed**: The test passed successfully
- **Failed**: The test failed
- **Skipped**: The test was skipped
- **TimedOut**: The test timed out
- **Panicked**: The test panicked

## Configuration

The test harness can be configured with various options:

```rust
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
```

## Integration with Existing Tests

The test harness can be integrated with existing tests by creating adapters for different test frameworks. For example, you can create an adapter for the standard Rust test framework:

```rust
pub struct RustTestAdapter;

impl RustTestAdapter {
    pub fn run_test<F>(name: &str, test_fn: F) -> TestResult
    where
        F: FnOnce() -> Result<(), Box<dyn std::error::Error>> + Send + 'static,
    {
        let context = TestContext::new(TestCategory::Unit, name.to_string());
        let test_case = TestCase::new(context, move |_| {
            match test_fn() {
                Ok(()) => Ok(TestResult::new(name, TestCategory::Unit, TestOutcome::Passed)),
                Err(e) => Ok(TestResult::new(name, TestCategory::Unit, TestOutcome::Failed)
                    .with_error(e.to_string())),
            }
        });
        
        let engine = TestEngine::new();
        let suite = TestSuite::new("Rust Test Suite").with_test_case(test_case);
        
        match tokio::runtime::Runtime::new().unwrap().block_on(engine.run_suite(suite)) {
            Ok(result) => {
                if result.failed > 0 {
                    result.test_results[0].clone()
                } else {
                    TestResult::new(name, TestCategory::Unit, TestOutcome::Passed)
                }
            },
            Err(e) => TestResult::new(name, TestCategory::Unit, TestOutcome::Failed)
                .with_error(e.to_string()),
        }
    }
}
```

## Extending the Test Harness

The test harness can be extended with custom plugins. Plugins can add new test types, assertions, reporters, and more.

```rust
pub struct MyCustomPlugin {
    name: String,
    version: String,
    description: String,
}

#[async_trait]
impl Plugin for MyCustomPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    async fn initialize(&self, _environment: &Environment) -> Result<(), TestHarnessError> {
        // Initialize the plugin
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<(), TestHarnessError> {
        // Shut down the plugin
        Ok(())
    }
    
    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability::TestType("custom".to_string())]
    }
    
    async fn execute_command(
        &self,
        command: &str,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value, TestHarnessError> {
        // Execute a command
        match command {
            "create_test_suite" => {
                // Create a test suite
                let name = args["name"].as_str().unwrap();
                let suite = TestSuite::new(name);
                Ok(serde_json::to_value(suite).unwrap())
            }
            _ => Err(TestHarnessError::PluginError(format!(
                "Unknown command: {}",
                command
            ))),
        }
    }
    
    fn config_schema(&self) -> Option<serde_json::Value> {
        // Return the plugin configuration schema
        None
    }
    
    async fn configure(&self, _config: &serde_json::Value) -> Result<(), TestHarnessError> {
        // Configure the plugin
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}
```

## Future Enhancements

- **Enhanced CI/CD Integration**: Deeper integration with CI/CD systems for automated testing
- **Test Coverage**: Integration with code coverage tools
- **Test Generation**: Automatic generation of tests from code
- **Test Prioritization**: Prioritize tests based on code changes
- **Advanced Test Flakiness Analysis**: Enhanced analysis of flaky tests with root cause identification
- **Test Impact Analysis**: Analyze the impact of code changes on tests
- **Mobile Dashboard**: Mobile-friendly version of the dashboard
- **Custom Dashboard Widgets**: Allow users to create custom dashboard widgets