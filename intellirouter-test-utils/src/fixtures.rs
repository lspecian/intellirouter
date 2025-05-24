//! # Test Fixtures
//!
//! This module provides common test fixtures for IntelliRouter tests.
//! Fixtures are pre-defined test data that can be used across different tests.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Creates a temporary directory for test files.
///
/// # Returns
///
/// A `tempfile::TempDir` that will be automatically cleaned up when dropped.
pub fn temp_test_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

/// Sample configuration for testing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Name of the test configuration
    pub name: String,
    /// Test-specific settings
    pub settings: HashMap<String, String>,
}

impl Default for TestConfig {
    fn default() -> Self {
        let mut settings = HashMap::new();
        settings.insert("test_mode".to_string(), "true".to_string());
        settings.insert("log_level".to_string(), "debug".to_string());

        Self {
            name: format!("test-config-{}", Uuid::new_v4()),
            settings,
        }
    }
}

/// Creates a sample request payload for testing.
///
/// # Returns
///
/// A JSON string containing a sample request payload.
pub fn sample_request_payload() -> String {
    r#"{
        "model": "test-model",
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful assistant."
            },
            {
                "role": "user",
                "content": "Hello, world!"
            }
        ],
        "temperature": 0.7,
        "max_tokens": 100
    }"#
    .to_string()
}

/// Creates a sample response payload for testing.
///
/// # Returns
///
/// A JSON string containing a sample response payload.
pub fn sample_response_payload() -> String {
    r#"{
        "id": "test-response-id",
        "object": "chat.completion",
        "created": 1677858242,
        "model": "test-model",
        "choices": [
            {
                "message": {
                    "role": "assistant",
                    "content": "Hello! How can I assist you today?"
                },
                "finish_reason": "stop",
                "index": 0
            }
        ],
        "usage": {
            "prompt_tokens": 25,
            "completion_tokens": 12,
            "total_tokens": 37
        }
    }"#
    .to_string()
}

/// Audit module fixtures for testing audit functionality
pub mod audit {
    use chrono::Utc;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    /// Service type for testing
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
    pub enum ServiceType {
        /// Router service
        Router,
        /// Chain Engine service
        ChainEngine,
        /// RAG Manager service
        RagManager,
        /// Persona Layer service
        PersonaLayer,
        /// Redis service
        Redis,
        /// ChromaDB service
        ChromaDb,
        /// Model Registry service
        ModelRegistry,
        /// Memory service
        Memory,
        /// Orchestrator service
        Orchestrator,
    }

    impl std::fmt::Display for ServiceType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ServiceType::Router => write!(f, "Router"),
                ServiceType::ChainEngine => write!(f, "ChainEngine"),
                ServiceType::RagManager => write!(f, "RagManager"),
                ServiceType::PersonaLayer => write!(f, "PersonaLayer"),
                ServiceType::Redis => write!(f, "Redis"),
                ServiceType::ChromaDb => write!(f, "ChromaDb"),
                ServiceType::ModelRegistry => write!(f, "ModelRegistry"),
                ServiceType::Memory => write!(f, "Memory"),
                ServiceType::Orchestrator => write!(f, "Orchestrator"),
            }
        }
    }

    /// Service status for testing
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
    pub enum ServiceStatus {
        /// Service is not started
        NotStarted,
        /// Service is starting
        Starting,
        /// Service is running
        Running,
        /// Service is active and healthy
        Active,
        /// Service is inactive
        Inactive,
        /// Service is running but in a degraded state
        Degraded,
        /// Service failed to start
        Failed,
        /// Service is shutting down
        ShuttingDown,
        /// Service is stopped
        Stopped,
    }

    /// Service information for testing
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct ServiceInfo {
        /// Service type
        pub service_type: ServiceType,
        /// Service name
        pub name: String,
        /// Service status
        pub status: ServiceStatus,
        /// Service host
        pub host: String,
        /// Service port
        pub port: u16,
        /// Main service endpoint
        pub endpoint: String,
        /// Service health endpoint
        pub health_endpoint: String,
        /// Service readiness endpoint
        pub readiness_endpoint: String,
        /// Service diagnostics endpoint
        pub diagnostics_endpoint: String,
        /// Service start time
        pub start_time: Option<chrono::DateTime<chrono::Utc>>,
        /// Service ready time
        pub ready_time: Option<chrono::DateTime<chrono::Utc>>,
        /// Service dependencies
        pub dependencies: Vec<ServiceType>,
    }

    impl ServiceInfo {
        /// Create a new service info for testing
        pub fn new(service_type: ServiceType, host: &str, port: u16) -> Self {
            let base_url = format!("http://{}:{}", host, port);
            let name = format!("test-{:?}", service_type).to_lowercase();

            Self {
                service_type,
                name,
                status: ServiceStatus::NotStarted,
                host: host.to_string(),
                port,
                endpoint: base_url.clone(),
                health_endpoint: format!("{}/health", base_url),
                readiness_endpoint: format!("{}/readiness", base_url),
                diagnostics_endpoint: format!("{}/diagnostics", base_url),
                start_time: None,
                ready_time: None,
                dependencies: Vec::new(),
            }
        }

        /// Set the service status
        pub fn with_status(mut self, status: ServiceStatus) -> Self {
            self.status = status;
            self
        }

        /// Set the service dependencies
        pub fn with_dependencies(mut self, dependencies: Vec<ServiceType>) -> Self {
            self.dependencies = dependencies;
            self
        }
    }

    /// Create a sample set of services for testing
    pub fn sample_services() -> HashMap<ServiceType, ServiceInfo> {
        let mut services = HashMap::new();

        services.insert(
            ServiceType::Router,
            ServiceInfo::new(ServiceType::Router, "localhost", 8080)
                .with_status(ServiceStatus::Running)
                .with_dependencies(vec![ServiceType::Redis]),
        );

        services.insert(
            ServiceType::ChainEngine,
            ServiceInfo::new(ServiceType::ChainEngine, "localhost", 8081)
                .with_status(ServiceStatus::Running)
                .with_dependencies(vec![ServiceType::Redis, ServiceType::Router]),
        );

        services.insert(
            ServiceType::RagManager,
            ServiceInfo::new(ServiceType::RagManager, "localhost", 8082)
                .with_status(ServiceStatus::Running)
                .with_dependencies(vec![
                    ServiceType::Redis,
                    ServiceType::Router,
                    ServiceType::ChromaDb,
                ]),
        );

        services.insert(
            ServiceType::Redis,
            ServiceInfo::new(ServiceType::Redis, "localhost", 6379)
                .with_status(ServiceStatus::Running),
        );

        services
    }

    /// Communication test result for testing
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct CommunicationTestResult {
        /// Source service
        pub source: ServiceType,
        /// Target service
        pub target: ServiceType,
        /// Test status
        pub success: bool,
        /// Error message if the test failed
        pub error: Option<String>,
        /// Response time in milliseconds
        pub response_time_ms: Option<u64>,
        /// Test timestamp
        pub timestamp: chrono::DateTime<chrono::Utc>,
    }

    impl CommunicationTestResult {
        /// Create a new successful communication test result
        pub fn success(source: ServiceType, target: ServiceType, response_time_ms: u64) -> Self {
            Self {
                source,
                target,
                success: true,
                error: None,
                response_time_ms: Some(response_time_ms),
                timestamp: Utc::now(),
            }
        }

        /// Create a new failed communication test result
        pub fn failure(source: ServiceType, target: ServiceType, error: &str) -> Self {
            Self {
                source,
                target,
                success: false,
                error: Some(error.to_string()),
                response_time_ms: None,
                timestamp: Utc::now(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temp_test_dir() {
        let dir = temp_test_dir();
        assert!(dir.path().exists());
    }

    #[test]
    fn test_test_config_default() {
        let config = TestConfig::default();
        assert!(config.name.starts_with("test-config-"));
        assert_eq!(config.settings.get("test_mode"), Some(&"true".to_string()));
    }

    #[test]
    fn test_sample_request_payload() {
        let payload = sample_request_payload();
        assert!(payload.contains("test-model"));
        assert!(payload.contains("Hello, world!"));
    }

    #[test]
    fn test_sample_response_payload() {
        let payload = sample_response_payload();
        assert!(payload.contains("test-response-id"));
        assert!(payload.contains("Hello! How can I assist you today?"));
    }
}
