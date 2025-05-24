//! Object-safe Environment trait implementation

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::config::{EnvironmentConfig, EnvironmentType};
use super::{ResourceUsage, ServiceResourceUsage};
use crate::modules::test_harness::types::TestHarnessError;

/// Object-safe Environment trait for different environment implementations
#[async_trait]
pub trait Environment: Send + Sync {
    /// Get the environment type
    fn env_type(&self) -> EnvironmentType;

    /// Get the environment ID
    fn id(&self) -> &str;

    /// Get the environment name
    fn name(&self) -> &str;

    /// Get the environment description
    fn description(&self) -> Option<&str>;

    /// Get the environment base directory
    fn base_dir(&self) -> &Path;

    /// Get the environment artifacts directory
    fn artifacts_dir(&self) -> &Path;

    /// Get the environment logs directory
    fn logs_dir(&self) -> &Path;

    /// Get the environment configuration
    fn config(&self) -> &EnvironmentConfig;

    /// Get an environment variable
    fn get_env_var(&self, key: &str) -> Option<&str>;

    /// Set an environment variable
    fn set_env_var_string(&mut self, key: String, value: String);

    /// Get a property from the environment
    fn get_property(&self, key: &str) -> Option<&serde_json::Value>;

    /// Get a typed property from the environment as JSON string
    fn get_property_json(&self, key: &str) -> Result<Option<String>, TestHarnessError>;

    /// Set up the environment
    async fn setup(&mut self) -> Result<(), TestHarnessError>;

    /// Tear down the environment
    async fn teardown(&mut self) -> Result<(), TestHarnessError>;

    /// Get a service endpoint
    fn get_service_endpoint(&self, service: &str) -> Option<String>;

    /// Wait for a service to be ready
    async fn wait_for_service(
        &self,
        service: &str,
        timeout_secs: u64,
    ) -> Result<(), TestHarnessError>;

    /// Wait for all services to be ready
    async fn wait_for_all_services(&self, timeout_secs: u64) -> Result<(), TestHarnessError>;

    /// Create a subdirectory in the artifacts directory
    fn create_artifacts_subdir_path(&self, subdir: PathBuf) -> Result<PathBuf, TestHarnessError>;

    /// Get the database URL
    fn database_url(&self) -> Option<&str>;

    /// Get the Redis URL
    fn redis_url(&self) -> Option<&str>;

    /// Get the Kafka URL
    fn kafka_url(&self) -> Option<&str>;

    /// Get the Elasticsearch URL
    fn elasticsearch_url(&self) -> Option<&str>;

    /// Get the S3 URL
    fn s3_url(&self) -> Option<&str>;

    /// Get the environment state
    async fn get_state(&self, key: &str) -> Option<serde_json::Value>;

    /// Set a value in the environment state as JSON string
    async fn set_state_json(&self, key: String, value_json: String)
        -> Result<(), TestHarnessError>;

    /// Get a typed value from the environment state as JSON string
    async fn get_state_json(&self, key: &str) -> Result<Option<String>, TestHarnessError>;

    /// Clear the environment state
    async fn clear_state(&self) -> Result<(), TestHarnessError>;

    /// Execute a command in the environment
    async fn execute_command(
        &self,
        command: &str,
        args: &[&str],
    ) -> Result<(String, String), TestHarnessError>;

    /// Copy a file to the environment
    async fn copy_file_to_path(
        &self,
        local_path: PathBuf,
        remote_path: PathBuf,
    ) -> Result<(), TestHarnessError>;

    /// Copy a file from the environment
    async fn copy_file_from_path(
        &self,
        remote_path: PathBuf,
        local_path: PathBuf,
    ) -> Result<(), TestHarnessError>;

    /// Check if a service is healthy
    async fn is_service_healthy(&self, service: &str) -> Result<bool, TestHarnessError>;

    /// Get service logs
    async fn get_service_logs(
        &self,
        service: &str,
        lines: Option<usize>,
    ) -> Result<String, TestHarnessError>;

    /// Get resource usage
    async fn get_resource_usage(&self) -> Result<ResourceUsage, TestHarnessError>;

    /// Take a snapshot of the environment
    async fn take_snapshot(&self, name: &str) -> Result<(), TestHarnessError>;

    /// Restore a snapshot
    async fn restore_snapshot(&self, name: &str) -> Result<(), TestHarnessError>;

    /// List available snapshots
    async fn list_snapshots(&self) -> Result<Vec<String>, TestHarnessError>;
}

/// Extension trait for Environment with generic methods
pub trait EnvironmentExt: Environment {
    /// Set an environment variable
    fn set_env_var(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.set_env_var_string(key.into(), value.into());
    }

    /// Get a typed property from the environment
    fn get_property_as<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<Option<T>, TestHarnessError> {
        let json = self.get_property_json(key)?;
        match json {
            Some(json_str) => {
                let value = serde_json::from_str(&json_str)
                    .map_err(TestHarnessError::DeserializationError)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Create a subdirectory in the artifacts directory
    fn create_artifacts_subdir(
        &self,
        subdir: impl AsRef<Path>,
    ) -> Result<PathBuf, TestHarnessError> {
        self.create_artifacts_subdir_path(subdir.as_ref().to_path_buf())
    }

    /// Set a value in the environment state
    async fn set_state<T: Serialize>(
        &self,
        key: impl Into<String>,
        value: T,
    ) -> Result<(), TestHarnessError> {
        let json = serde_json::to_string(&value).map_err(TestHarnessError::SerializationError)?;
        self.set_state_json(key.into(), json).await
    }

    /// Get a typed value from the environment state
    async fn get_state_as<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<Option<T>, TestHarnessError> {
        let json = self.get_state_json(key).await?;
        match json {
            Some(json_str) => {
                let value = serde_json::from_str(&json_str)
                    .map_err(TestHarnessError::DeserializationError)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Copy a file to the environment
    async fn copy_file_to(
        &self,
        local_path: impl AsRef<Path>,
        remote_path: impl AsRef<Path>,
    ) -> Result<(), TestHarnessError> {
        self.copy_file_to_path(
            local_path.as_ref().to_path_buf(),
            remote_path.as_ref().to_path_buf(),
        )
        .await
    }

    /// Copy a file from the environment
    async fn copy_file_from(
        &self,
        remote_path: impl AsRef<Path>,
        local_path: impl AsRef<Path>,
    ) -> Result<(), TestHarnessError> {
        self.copy_file_from_path(
            remote_path.as_ref().to_path_buf(),
            local_path.as_ref().to_path_buf(),
        )
        .await
    }
}

// Implement EnvironmentExt for all types that implement Environment
impl<T: Environment + ?Sized> EnvironmentExt for T {}
