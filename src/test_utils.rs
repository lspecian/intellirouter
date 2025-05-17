//! Test utilities for IntelliRouter
//!
//! This module provides common utilities and helpers for testing IntelliRouter components.
//! It includes mock implementations, test fixtures, and helper functions.

use std::sync::Once;

// Initialize logging for tests
static INIT: Once = Once::new();
pub fn init_test_logging() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter("intellirouter=debug,test=debug")
            .with_test_writer()
            .init();
    });
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

/// Mock implementations for testing
pub mod mocks {
    use mockall::predicate::*;
    use mockall::*;

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
    mock! {
        pub ModelRegistry {
            pub fn new() -> Self;
            pub fn get_model(&self, id: &str) -> Result<crate::modules::model_registry::ModelMetadata, crate::modules::model_registry::RegistryError>;
            pub fn register_model(&self, metadata: crate::modules::model_registry::ModelMetadata) -> Result<(), crate::modules::model_registry::RegistryError>;
            pub fn update_model(&self, metadata: crate::modules::model_registry::ModelMetadata) -> Result<(), crate::modules::model_registry::RegistryError>;
            pub fn remove_model(&self, id: &str) -> Result<(), crate::modules::model_registry::RegistryError>;
            pub fn list_models(&self) -> Vec<crate::modules::model_registry::ModelMetadata>;
            pub fn find_models(&self, filter: crate::modules::model_registry::ModelFilter) -> Vec<crate::modules::model_registry::ModelMetadata>;
        }
    }
}
