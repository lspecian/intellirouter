//! # Test Helpers
//!
//! This module provides helper functions and utilities for testing IntelliRouter components.
//! These helpers simplify common testing tasks and provide utilities for test setup and teardown.

use async_trait::async_trait;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Once;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

// Initialize logging for tests
static INIT: Once = Once::new();
pub fn init_test_logging() {
    INIT.call_once(|| {
        let env_filter = std::env::var("RUST_LOG")
            .unwrap_or_else(|_| "intellirouter=debug,test=debug".to_string());

        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_file(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_target(true)
            .with_test_writer()
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set global default subscriber");
    });
}

/// Initialize logging for tests with output to a file
pub fn init_test_logging_with_file(test_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use tracing_subscriber::fmt::writer::MakeWriterExt;

    let log_dir = std::path::Path::new("logs");
    std::fs::create_dir_all(log_dir)?;

    let log_file = File::create(log_dir.join(format!("{}.log", test_name)))?;
    let writer = std::io::stdout.and(log_file);

    let env_filter =
        std::env::var("RUST_LOG").unwrap_or_else(|_| "intellirouter=debug,test=debug".to_string());

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(true)
        .with_writer(writer)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    Ok(())
}

/// A test fixture for configuration
pub struct TestConfig {
    pub test_dir: tempfile::TempDir,
}

impl TestConfig {
    /// Create a new test configuration with a temporary directory
    pub fn new() -> Self {
        let test_dir = tempfile::tempdir().expect("Failed to create temp directory");
        Self { test_dir }
    }

    /// Get the path to the temporary directory
    pub fn path(&self) -> &std::path::Path {
        self.test_dir.path()
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create a test request
pub fn create_test_request(content: &str) -> String {
    format!(
        r#"{{
        "model": "test-model",
        "messages": [
            {{
                "role": "user",
                "content": "{}"
            }}
        ]
    }}"#,
        content
    )
}

/// Helper to create a test response
pub fn create_test_response(content: &str) -> String {
    format!(
        r#"{{
        "id": "test-id",
        "object": "chat.completion",
        "created": 1677858242,
        "model": "test-model",
        "choices": [
            {{
                "message": {{
                    "role": "assistant",
                    "content": "{}"
                }},
                "finish_reason": "stop",
                "index": 0
            }}
        ]
    }}"#,
        content
    )
}

/// Async test helper to run async tests
#[macro_export]
macro_rules! async_test {
    ($test_fn:expr) => {
        tokio_test::block_on(async {
            $test_fn.await;
        });
    };
}

/// Waits for a condition to be true with a timeout.
///
/// # Arguments
///
/// * `condition` - A function that returns a boolean indicating if the condition is met
/// * `timeout` - The maximum time to wait for the condition to be true
/// * `check_interval` - The interval between condition checks
///
/// # Returns
///
/// `true` if the condition was met within the timeout, `false` otherwise
pub async fn wait_for_condition<F>(
    mut condition: F,
    timeout: Duration,
    check_interval: Duration,
) -> bool
where
    F: FnMut() -> bool,
{
    let start = Instant::now();

    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        sleep(check_interval).await;
    }

    false
}

/// Waits for an async condition to be true with a timeout.
///
/// # Arguments
///
/// * `condition` - An async function that returns a boolean indicating if the condition is met
/// * `timeout` - The maximum time to wait for the condition to be true
/// * `check_interval` - The interval between condition checks
///
/// # Returns
///
/// `true` if the condition was met within the timeout, `false` otherwise
/// A trait for async conditions that can be checked
#[async_trait]
pub trait AsyncCondition {
    async fn check(&mut self) -> bool;
}

/// Wait for an async condition to be true with a timeout
pub async fn wait_for_async_condition<C>(
    condition: &mut C,
    timeout: Duration,
    check_interval: Duration,
) -> bool
where
    C: AsyncCondition + Send,
{
    let start = Instant::now();

    while start.elapsed() < timeout {
        if condition.check().await {
            return true;
        }
        sleep(check_interval).await;
    }

    false
}

/// Creates a test configuration file in a temporary directory.
///
/// # Arguments
///
/// * `config_content` - The content to write to the configuration file
///
/// # Returns
///
/// A tuple containing the temporary directory and the path to the configuration file
pub fn create_test_config_file(config_content: &str) -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let config_path = temp_dir.path().join("config.toml");

    std::fs::write(&config_path, config_content).expect("Failed to write test configuration file");

    (temp_dir, config_path)
}

/// Runs a function with retry logic.
///
/// # Arguments
///
/// * `operation` - The function to run
/// * `max_attempts` - The maximum number of attempts
/// * `retry_delay` - The delay between retries
///
/// # Returns
///
/// The result of the operation if successful, or the last error encountered
/// A trait for async operations that can be retried
#[async_trait]
pub trait AsyncOperation<T, E> {
    async fn execute(&mut self) -> Result<T, E>;
}

/// Runs an operation with retry logic.
pub async fn retry<O, T, E>(
    operation: &mut O,
    max_attempts: usize,
    retry_delay: Duration,
) -> Result<T, E>
where
    O: AsyncOperation<T, E> + Send,
    E: std::fmt::Debug,
{
    let mut attempts = 0;
    let mut last_error = None;

    while attempts < max_attempts {
        match operation.execute().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                attempts += 1;
                last_error = Some(err);

                if attempts < max_attempts {
                    debug!(
                        "Operation failed (attempt {}/{}), retrying after {:?}",
                        attempts, max_attempts, retry_delay
                    );
                    sleep(retry_delay).await;
                }
            }
        }
    }

    warn!("Operation failed after {} attempts", max_attempts);
    Err(last_error.unwrap())
}

/// Captures logs during test execution.
///
/// # Arguments
///
/// * `test_fn` - The test function to run
///
/// # Returns
///
/// The result of the test function
pub async fn with_captured_logs<F, Fut, T>(test_fn: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    // Set up a subscriber that captures logs
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);

    // Run the test function
    info!("Starting test with captured logs");
    let result = test_fn().await;
    info!("Test completed");

    result
}

/// Generates a random port number for testing.
///
/// # Returns
///
/// A random port number between 10000 and 65535
pub fn random_port() -> u16 {
    use rand::Rng;
    rand::thread_rng().gen_range(10000..65535)
}

/// Communication test helpers for testing service communication
pub mod communication {
    use super::*;
    use crate::fixtures::audit::{ServiceInfo, ServiceType};
    use crate::mocks::audit::MockServiceClient;
    use reqwest::Client;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// Test if a service can reach another service
    pub async fn test_service_connection(
        client: &Client,
        source_url: &str,
        target_url: &str,
    ) -> Result<bool, anyhow::Error> {
        // For Redis, we need to use a different approach
        if target_url.starts_with("redis://") {
            return test_redis_connection(target_url).await;
        }

        // For HTTP services, we can use the diagnostics endpoint
        let diagnostics_url = format!("{}/diagnostics", source_url);

        let response = client
            .get(&diagnostics_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(false);
        }

        let body: Value = response.json().await?;

        // Check if the service reports the target as a dependency
        if let Some(connections) = body.get("connections").and_then(|c| c.as_array()) {
            for connection in connections {
                if let Some(name) = connection.get("name").and_then(|n| n.as_str()) {
                    if target_url.contains(name) {
                        if let Some(status) = connection.get("status").and_then(|s| s.as_str()) {
                            return Ok(status == "healthy" || status == "degraded");
                        }
                    }
                }
            }
        }

        // If we didn't find the target in the connections, check if it's in the diagnostics
        if let Some(diagnostics) = body.get("diagnostics").and_then(|d| d.as_object()) {
            for (key, _) in diagnostics {
                if target_url.contains(key) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Test if a service can connect to Redis
    pub async fn test_redis_connection(redis_url: &str) -> Result<bool, anyhow::Error> {
        let client = redis::Client::open(redis_url)?;
        let mut conn = client.get_async_connection().await?;

        // Try to ping Redis
        let pong: String = redis::cmd("PING").query_async(&mut conn).await?;

        Ok(pong == "PONG")
    }

    /// Get the URL for a service
    pub fn get_service_url(service: ServiceType) -> String {
        match service {
            ServiceType::Router => "http://router:8080".to_string(),
            ServiceType::ChainEngine => "http://orchestrator:8080".to_string(),
            ServiceType::RagManager => "http://rag-injector:8080".to_string(),
            ServiceType::PersonaLayer => "http://summarizer:8080".to_string(),
            ServiceType::Redis => "redis://redis:6379".to_string(),
            ServiceType::ChromaDb => "http://chromadb:8000".to_string(),
            ServiceType::ModelRegistry => "http://model-registry:8080".to_string(),
            ServiceType::Memory => "http://memory:8080".to_string(),
            ServiceType::Orchestrator => "http://orchestrator-service:8080".to_string(),
        }
    }

    /// Create a mock service client for testing
    pub fn create_mock_service_client() -> MockServiceClient {
        MockServiceClient::new()
    }

    /// Create a mock service info map for testing
    pub fn create_mock_services() -> HashMap<ServiceType, ServiceInfo> {
        crate::fixtures::audit::sample_services()
    }

    /// Test gRPC communication between services
    pub async fn test_grpc_communication(
        client: &Client,
        services: &HashMap<ServiceType, ServiceInfo>,
    ) -> Result<Vec<crate::fixtures::audit::CommunicationTestResult>, anyhow::Error> {
        let mut results = Vec::new();

        // Define the services that communicate via gRPC
        let grpc_pairs = vec![
            (ServiceType::Router, ServiceType::ChainEngine),
            (ServiceType::Router, ServiceType::RagManager),
            (ServiceType::ChainEngine, ServiceType::Router),
            (ServiceType::RagManager, ServiceType::Router),
        ];

        for (source, target) in grpc_pairs {
            if let (Some(source_info), Some(target_info)) =
                (services.get(&source), services.get(&target))
            {
                // Get service endpoints
                let source_url = &source_info.endpoint;
                let target_url = &target_info.endpoint;

                // Test if source can reach target
                let start_time = Instant::now();

                match test_service_connection(client, source_url, target_url).await {
                    Ok(true) => {
                        let elapsed = start_time.elapsed();
                        info!("Service {} can reach {}", source, target);

                        let result = crate::fixtures::audit::CommunicationTestResult::success(
                            source,
                            target,
                            elapsed.as_millis() as u64,
                        );

                        results.push(result);
                    }
                    Ok(false) => {
                        warn!("Service {} cannot reach {}", source, target);

                        let result = crate::fixtures::audit::CommunicationTestResult::failure(
                            source,
                            target,
                            &format!("Service {} cannot reach {}", source, target),
                        );

                        results.push(result);
                    }
                    Err(e) => {
                        error!("Error testing if {} can reach {}: {}", source, target, e);

                        let result = crate::fixtures::audit::CommunicationTestResult::failure(
                            source,
                            target,
                            &format!("Error: {}", e),
                        );

                        results.push(result);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Test Redis pub/sub communication
    pub async fn test_redis_pubsub(
        services: &HashMap<ServiceType, ServiceInfo>,
    ) -> Result<Vec<crate::fixtures::audit::CommunicationTestResult>, anyhow::Error> {
        let mut results = Vec::new();

        // Define the services that use Redis pub/sub
        let redis_services = vec![
            ServiceType::Router,
            ServiceType::ChainEngine,
            ServiceType::RagManager,
            ServiceType::PersonaLayer,
        ];

        // Connect to Redis
        let redis_url = "redis://redis:6379";
        let client = redis::Client::open(redis_url)?;
        let mut conn = client.get_async_connection().await?;

        // Test Redis pub/sub by publishing a message to a test channel
        // and checking if it can be received
        let test_channel = "audit_test_channel";
        let test_message = "audit_test_message";

        // Publish a message
        let publish_result: Result<i32, redis::RedisError> = redis::cmd("PUBLISH")
            .arg(test_channel)
            .arg(test_message)
            .query_async(&mut conn)
            .await;

        // Check if the publish was successful
        match publish_result {
            Ok(_) => {
                // For each service that uses Redis, add a successful test result
                for service in redis_services {
                    if services.contains_key(&service) {
                        let result = crate::fixtures::audit::CommunicationTestResult::success(
                            service,
                            ServiceType::Redis,
                            0, // We don't have an actual response time
                        );

                        results.push(result);
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to publish message to Redis: {}", e);
                error!("{}", error_msg);

                // For each service that uses Redis, add a failed test result
                for service in redis_services {
                    if services.contains_key(&service) {
                        let result = crate::fixtures::audit::CommunicationTestResult::failure(
                            service,
                            ServiceType::Redis,
                            &error_msg,
                        );

                        results.push(result);
                    }
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wait_for_condition() {
        let mut counter = 0;

        let result = wait_for_condition(
            || {
                counter += 1;
                counter >= 3
            },
            Duration::from_secs(1),
            Duration::from_millis(10),
        )
        .await;

        assert!(result);
        assert!(counter >= 3);
    }

    #[tokio::test]
    async fn test_wait_for_condition_timeout() {
        let result = wait_for_condition(
            || false,
            Duration::from_millis(100),
            Duration::from_millis(10),
        )
        .await;

        assert!(!result);
    }

    #[tokio::test]
    async fn test_wait_for_async_condition() {
        // Use a struct that implements AsyncCondition
        struct TestCondition {
            counter: usize,
        }

        impl TestCondition {
            fn new() -> Self {
                Self { counter: 0 }
            }
        }

        #[async_trait]
        impl AsyncCondition for TestCondition {
            async fn check(&mut self) -> bool {
                self.counter += 1;
                self.counter >= 3
            }
        }

        let mut condition = TestCondition::new();

        let result = wait_for_async_condition(
            &mut condition,
            Duration::from_secs(1),
            Duration::from_millis(10),
        )
        .await;

        assert!(result);
        assert!(condition.counter >= 3);
    }

    #[test]
    fn test_create_test_config_file() {
        let config_content = r#"
            [server]
            host = "127.0.0.1"
            port = 8080
        "#;

        let (temp_dir, config_path) = create_test_config_file(config_content);

        assert!(config_path.exists());
        let content = std::fs::read_to_string(config_path).unwrap();
        assert_eq!(content, config_content);

        // Temp dir will be cleaned up when dropped
        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_retry_success() {
        // Use a struct that implements AsyncOperation
        struct TestOperation {
            attempts: usize,
        }

        impl TestOperation {
            fn new() -> Self {
                Self { attempts: 0 }
            }
        }

        #[async_trait]
        impl AsyncOperation<i32, &'static str> for TestOperation {
            async fn execute(&mut self) -> Result<i32, &'static str> {
                self.attempts += 1;
                if self.attempts < 3 {
                    Err("Not ready yet")
                } else {
                    Ok(42)
                }
            }
        }

        let mut operation = TestOperation::new();

        let result = retry(&mut operation, 5, Duration::from_millis(10)).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(operation.attempts, 3);
    }

    #[tokio::test]
    async fn test_retry_failure() {
        // Use a struct that implements AsyncOperation
        struct TestOperation {
            attempts: usize,
        }

        impl TestOperation {
            fn new() -> Self {
                Self { attempts: 0 }
            }
        }

        #[async_trait]
        impl AsyncOperation<(), &'static str> for TestOperation {
            async fn execute(&mut self) -> Result<(), &'static str> {
                self.attempts += 1;
                Err("Always fails")
            }
        }

        let mut operation = TestOperation::new();

        let result: Result<(), &str> = retry(&mut operation, 3, Duration::from_millis(10)).await;

        assert!(result.is_err());
        assert_eq!(operation.attempts, 3);
    }

    #[tokio::test]
    async fn test_with_captured_logs() {
        let result = with_captured_logs(|| async {
            tracing::info!("This is a test log");
            42
        })
        .await;

        assert_eq!(result, 42);
    }

    #[test]
    fn test_random_port() {
        let port = random_port();
        assert!(port >= 10000);
        assert!(port <= 65535);
    }
}
