//! Test Environment Management
//!
//! This module provides functionality for managing test environments.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::types::TestHarnessError;

/// Environment trait for managing test environments
#[async_trait]
pub trait Environment: Send + Sync {
    /// Initialize the environment
    async fn initialize(&self) -> Result<(), TestHarnessError>;

    /// Cleanup the environment
    async fn cleanup(&self) -> Result<(), TestHarnessError>;

    /// Get a variable from the environment
    async fn get_variable(&self, name: &str) -> Option<String>;

    /// Set a variable in the environment
    async fn set_variable(&self, name: &str, value: &str) -> Result<(), TestHarnessError>;

    /// Get the working directory
    async fn get_working_directory(&self) -> PathBuf;

    /// Set the working directory
    async fn set_working_directory(&self, path: PathBuf) -> Result<(), TestHarnessError>;

    /// Get a resource from the environment
    async fn get_resource(&self, name: &str) -> Option<Vec<u8>>;

    /// Set a resource in the environment
    async fn set_resource(&self, name: &str, data: Vec<u8>) -> Result<(), TestHarnessError>;

    /// Create a temporary file
    async fn create_temp_file(&self, content: &[u8]) -> Result<PathBuf, TestHarnessError>;

    /// Create a temporary directory
    async fn create_temp_directory(&self) -> Result<PathBuf, TestHarnessError>;
}

/// Local environment implementation
pub struct LocalEnvironment {
    /// Working directory
    working_directory: RwLock<PathBuf>,
    /// Environment variables
    variables: RwLock<HashMap<String, String>>,
    /// Resources
    resources: RwLock<HashMap<String, Vec<u8>>>,
}

impl LocalEnvironment {
    /// Create a new local environment
    pub fn new() -> Self {
        Self {
            working_directory: RwLock::new(
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            ),
            variables: RwLock::new(HashMap::new()),
            resources: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Environment for LocalEnvironment {
    async fn initialize(&self) -> Result<(), TestHarnessError> {
        Ok(())
    }

    async fn cleanup(&self) -> Result<(), TestHarnessError> {
        self.variables.write().await.clear();
        self.resources.write().await.clear();
        Ok(())
    }

    async fn get_variable(&self, name: &str) -> Option<String> {
        self.variables.read().await.get(name).cloned()
    }

    async fn set_variable(&self, name: &str, value: &str) -> Result<(), TestHarnessError> {
        self.variables
            .write()
            .await
            .insert(name.to_string(), value.to_string());
        Ok(())
    }

    async fn get_working_directory(&self) -> PathBuf {
        self.working_directory.read().await.clone()
    }

    async fn set_working_directory(&self, path: PathBuf) -> Result<(), TestHarnessError> {
        *self.working_directory.write().await = path;
        Ok(())
    }

    async fn get_resource(&self, name: &str) -> Option<Vec<u8>> {
        self.resources.read().await.get(name).cloned()
    }

    async fn set_resource(&self, name: &str, data: Vec<u8>) -> Result<(), TestHarnessError> {
        self.resources.write().await.insert(name.to_string(), data);
        Ok(())
    }

    async fn create_temp_file(&self, content: &[u8]) -> Result<PathBuf, TestHarnessError> {
        let temp_dir = tempfile::tempdir().map_err(|e| TestHarnessError::IoError(e))?;
        let file_path = temp_dir.path().join("temp_file");
        tokio::fs::write(&file_path, content)
            .await
            .map_err(|e| TestHarnessError::IoError(e))?;
        Ok(file_path)
    }

    async fn create_temp_directory(&self) -> Result<PathBuf, TestHarnessError> {
        let temp_dir = tempfile::tempdir().map_err(|e| TestHarnessError::IoError(e))?;
        Ok(temp_dir.path().to_path_buf())
    }
}
