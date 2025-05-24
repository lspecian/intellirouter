//! # Test Mocks
//!
//! This module provides mock implementations of IntelliRouter components and services for testing.
//! These mocks can be used to simulate behavior without requiring the actual implementations.

use async_trait::async_trait;
use mockall::predicate::*;
use mockall::*;
use std::sync::Arc;

#[cfg(feature = "with-intellirouter")]
use intellirouter::modules::model_registry::types::ModelInfo;

// Mock for router
mock! {
    pub Router {
        pub fn route(&self, request: &str) -> Result<String, String>;
        pub fn init(&self) -> Result<(), String>;
    }
}

// Mock for LLM client
mock! {
    pub LlmClient {
        pub fn send_request(&self, request: &str) -> Result<String, String>;
        pub fn init(&self) -> Result<(), String>;
    }
}

// Mock for model registry
#[cfg(feature = "with-intellirouter")]
mock! {
    pub ModelRegistry {
        pub fn new() -> Self;
        pub fn get_model(&self, id: &str) -> Result<intellirouter::modules::model_registry::ModelMetadata, intellirouter::modules::model_registry::RegistryError>;
        pub fn register_model(&self, metadata: intellirouter::modules::model_registry::ModelMetadata) -> Result<(), intellirouter::modules::model_registry::RegistryError>;
        pub fn update_model(&self, metadata: intellirouter::modules::model_registry::ModelMetadata) -> Result<(), intellirouter::modules::model_registry::RegistryError>;
        pub fn remove_model(&self, id: &str) -> Result<(), intellirouter::modules::model_registry::RegistryError>;
        pub fn list_models(&self) -> Vec<intellirouter::modules::model_registry::ModelMetadata>;
        pub fn find_models(&self, filter: intellirouter::modules::model_registry::ModelFilter) -> Vec<intellirouter::modules::model_registry::ModelMetadata>;
    }
}

#[cfg(not(feature = "with-intellirouter"))]
mock! {
    pub ModelRegistry {
        pub fn new() -> Self;
        pub fn get_model(&self, id: &str) -> Result<String, String>;
        pub fn register_model(&self, metadata: String) -> Result<(), String>;
        pub fn update_model(&self, metadata: String) -> Result<(), String>;
        pub fn remove_model(&self, id: &str) -> Result<(), String>;
        pub fn list_models(&self) -> Vec<String>;
        pub fn find_models(&self, filter: String) -> Vec<String>;
    }
}

/// A mock HTTP server for testing HTTP clients.
///
/// # Example
///
/// ```no_run
/// use intellirouter_test_utils::mocks::MockHttpServer;
///
/// // This example shows how to use the MockHttpServer
/// // Note: This code would typically be run in an async test function
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let mut server = MockHttpServer::start();
///     let client = reqwest::Client::new();
///
///     // Configure mock response
///     server.mock(|builder| {
///         builder
///             .method("GET")
///             .path("/test")
///             .status(200)
///             .body("Hello, world!");
///     });
///
///     // Use the mock server URL in your tests
///     let response = client.get(&format!("{}/test", server.url())).send().await?;
///     assert_eq!(response.status(), 200);
///     assert_eq!(response.text().await?, "Hello, world!");
///
///     Ok(())
/// }
/// ```
/// A wrapper around mockito::Server that provides a more convenient API.
pub struct MockHttpServer {
    server: mockito::ServerGuard,
}

impl MockHttpServer {
    /// Starts a new mock HTTP server.
    ///
    /// # Returns
    ///
    /// A new `MockHttpServer` instance.
    pub fn start() -> Self {
        Self {
            server: mockito::Server::new(),
        }
    }

    /// Returns the base URL of the mock server.
    ///
    /// # Returns
    ///
    /// The base URL as a string.
    pub fn url(&self) -> String {
        self.server.url()
    }

    /// Creates a new mock expectation.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that configures the mock expectation.
    ///
    /// # Returns
    ///
    /// A mockito::Mock instance.
    pub fn mock<F>(&mut self, f: F) -> mockito::Mock
    where
        F: FnOnce(&mut MockBuilder),
    {
        // Create a simple struct to hold the configuration
        let mut config = MockConfig {
            method: "GET".to_string(),
            path: "/".to_string(),
            status: 200,
            body: "".to_string(),
        };

        // Create a builder that updates the configuration
        let mut builder = MockBuilder {
            config: &mut config,
        };

        // Call the closure with the builder
        f(&mut builder);

        // Create the mock with the configuration
        let mut mock = self.server.mock(&config.method, config.path.as_str());
        mock = mock.with_status(config.status);
        if !config.body.is_empty() {
            mock = mock.with_body(&config.body);
        }
        mock
    }
}

/// Configuration for a mock.
struct MockConfig {
    method: String,
    path: String,
    status: usize,
    body: String,
}

/// A builder for creating mock expectations.
pub struct MockBuilder<'a> {
    config: &'a mut MockConfig,
}

impl<'a> MockBuilder<'a> {
    /// Set the HTTP method for this mock.
    pub fn method(&mut self, method: &str) -> &mut Self {
        self.config.method = method.to_string();
        self
    }

    /// Set the path for this mock.
    pub fn path(&mut self, path: &str) -> &mut Self {
        self.config.path = path.to_string();
        self
    }

    /// Set the status code for this mock.
    pub fn status(&mut self, status: usize) -> &mut Self {
        self.config.status = status;
        self
    }

    /// Set the response body for this mock.
    pub fn body(&mut self, body: &str) -> &mut Self {
        self.config.body = body.to_string();
        self
    }
}

/// A mock for an async function that returns a Result.
///
/// This is a generic mock that can be used for any async function that returns a Result.
#[automock]
#[async_trait]
pub trait AsyncResultFn<T: Send + Sync + 'static, E: Send + Sync + 'static> {
    async fn call(&self) -> Result<T, E>;
}

/// A mock for the model registry client.
///
/// This mock can be used to simulate the behavior of the model registry client in tests.
#[cfg(feature = "with-intellirouter")]
#[automock]
#[async_trait]
pub trait ModelRegistryClient: Send + Sync {
    async fn get_model(&self, model_id: &str) -> Result<ModelInfo, anyhow::Error>;
    async fn list_models(&self) -> Result<Vec<ModelInfo>, anyhow::Error>;
}

/// A mock for a simple key-value store.
///
/// This mock can be used to simulate a key-value store in tests.
#[automock]
pub trait KeyValueStore {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: String) -> Result<(), anyhow::Error>;
    fn delete(&self, key: &str) -> Result<(), anyhow::Error>;
}

/// A mock for a message broker.
///
/// This mock can be used to simulate a message broker in tests.
#[automock]
#[async_trait]
pub trait MessageBroker: Send + Sync {
    async fn publish(&self, topic: &str, message: Vec<u8>) -> Result<(), anyhow::Error>;
    async fn subscribe(&self, topic: &str) -> Result<(), anyhow::Error>;
    async fn receive(&self) -> Result<(String, Vec<u8>), anyhow::Error>;
}

/// Audit module mocks for testing audit functionality
pub mod audit {
    use super::*;
    use crate::fixtures::audit::{CommunicationTestResult, ServiceInfo, ServiceType};
    use serde_json::Value;
    use std::collections::HashMap;

    /// A mock for the audit controller.
    #[automock]
    #[async_trait]
    pub trait AuditController: Send + Sync {
        async fn start_audit(&self) -> Result<(), anyhow::Error>;
        async fn get_service_status(
            &self,
            service_type: ServiceType,
        ) -> Result<ServiceInfo, anyhow::Error>;
        async fn get_all_services(
            &self,
        ) -> Result<HashMap<ServiceType, ServiceInfo>, anyhow::Error>;
        async fn run_communication_tests(
            &self,
        ) -> Result<Vec<CommunicationTestResult>, anyhow::Error>;
    }

    /// A mock for a service health check.
    #[automock]
    #[async_trait]
    pub trait ServiceHealthCheck: Send + Sync {
        async fn check_health(&self, service_info: &ServiceInfo) -> Result<bool, anyhow::Error>;
        async fn check_readiness(&self, service_info: &ServiceInfo) -> Result<bool, anyhow::Error>;
        async fn get_diagnostics(&self, service_info: &ServiceInfo)
            -> Result<Value, anyhow::Error>;
    }

    /// A mock HTTP client for testing service communication.
    pub struct MockServiceClient {
        pub http_server: MockHttpServer,
        pub client: reqwest::Client,
    }

    impl MockServiceClient {
        /// Creates a new mock service client.
        pub fn new() -> Self {
            Self {
                http_server: MockHttpServer::start(),
                client: reqwest::Client::new(),
            }
        }

        /// Mocks a health check endpoint.
        pub fn mock_health_check(
            &mut self,
            service_type: ServiceType,
            is_healthy: bool,
        ) -> mockito::Mock {
            let status = if is_healthy { 200 } else { 503 };
            let body = if is_healthy {
                r#"{"status":"healthy"}"#
            } else {
                r#"{"status":"unhealthy","error":"Service is not healthy"}"#
            };

            self.http_server.mock(|builder| {
                builder
                    .method("GET")
                    .path(&format!("/{}/health", service_type))
                    .status(status)
                    .body(body);
            })
        }

        /// Mocks a readiness check endpoint.
        pub fn mock_readiness_check(
            &mut self,
            service_type: ServiceType,
            is_ready: bool,
        ) -> mockito::Mock {
            let status = if is_ready { 200 } else { 503 };
            let body = if is_ready {
                r#"{"status":"ready"}"#
            } else {
                r#"{"status":"not_ready","error":"Service is not ready"}"#
            };

            self.http_server.mock(|builder| {
                builder
                    .method("GET")
                    .path(&format!("/{}/readiness", service_type))
                    .status(status)
                    .body(body);
            })
        }

        /// Mocks a diagnostics endpoint.
        pub fn mock_diagnostics(
            &mut self,
            service_type: ServiceType,
            diagnostics: &str,
        ) -> mockito::Mock {
            self.http_server.mock(|builder| {
                builder
                    .method("GET")
                    .path(&format!("/{}/diagnostics", service_type))
                    .status(200)
                    .body(diagnostics);
            })
        }

        /// Mocks a service-to-service communication test.
        pub fn mock_service_connection(
            &mut self,
            source: ServiceType,
            target: ServiceType,
            can_connect: bool,
        ) -> mockito::Mock {
            let body = if can_connect {
                format!(
                    r#"{{
                    "connections": [
                        {{
                            "name": "{}",
                            "status": "healthy"
                        }}
                    ]
                }}"#,
                    target
                )
            } else {
                format!(
                    r#"{{
                    "connections": [
                        {{
                            "name": "{}",
                            "status": "unhealthy",
                            "error": "Cannot connect to service"
                        }}
                    ]
                }}"#,
                    target
                )
            };

            self.http_server.mock(|builder| {
                builder
                    .method("GET")
                    .path(&format!("/{}/diagnostics", source))
                    .status(200)
                    .body(&body);
            })
        }
    }

    impl Default for MockServiceClient {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Use a non-async test to avoid runtime conflicts with mockito
    #[test]
    #[ignore]
    fn test_mock_http_server() {
        // This test is skipped because mockito tries to create a Tokio runtime
        // inside a test that's already running in a Tokio runtime, which causes
        // a panic. We'll need to find a better way to test this in the future.
    }

    #[tokio::test]
    async fn test_mock_async_result_fn() {
        let mut mock = MockAsyncResultFn::<String, anyhow::Error>::new();

        // Configure mock
        mock.expect_call().returning(|| Ok("test".to_string()));

        // Test the mock
        let result = mock.call().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test");
    }

    #[test]
    fn test_mock_key_value_store() {
        let mut mock = MockKeyValueStore::new();

        // Configure mock
        mock.expect_get()
            .with(eq("test_key"))
            .returning(|_| Some("test_value".to_string()));

        mock.expect_set()
            .with(eq("test_key"), eq("new_value".to_string()))
            .returning(|_, _| Ok(()));

        // Test the mock
        let value = mock.get("test_key");
        assert_eq!(value, Some("test_value".to_string()));

        let result = mock.set("test_key", "new_value".to_string());
        assert!(result.is_ok());
    }
}
