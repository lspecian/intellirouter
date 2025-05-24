//! Data Cleanup Module
//!
//! This module provides functionality for cleaning up test data after test execution.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

use super::isolation::{IsolationContext, IsolationManager};
use super::store::DataStore;

/// Cleanup strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanupStrategy {
    /// Clean up immediately after test execution
    Immediate,
    /// Clean up at the end of the test run
    Deferred,
    /// Clean up manually (no automatic cleanup)
    Manual,
    /// Never clean up (keep all data)
    Never,
}

/// Cleanup scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanupScope {
    /// Clean up all data
    All,
    /// Clean up only test-specific data
    TestOnly,
    /// Clean up only temporary data
    TempOnly,
}

/// Cleanup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupConfig {
    /// Default cleanup strategy
    pub default_strategy: CleanupStrategy,
    /// Default cleanup scope
    pub default_scope: CleanupScope,
    /// Base directory for cleanup
    pub base_dir: Option<PathBuf>,
    /// Custom configuration
    pub custom: Option<serde_json::Value>,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            default_strategy: CleanupStrategy::Deferred,
            default_scope: CleanupScope::TestOnly,
            base_dir: None,
            custom: None,
        }
    }
}

/// Cleanup task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupTask {
    /// Task ID
    pub id: String,
    /// Task name
    pub name: String,
    /// Task description
    pub description: Option<String>,
    /// Cleanup strategy
    pub strategy: CleanupStrategy,
    /// Cleanup scope
    pub scope: CleanupScope,
    /// Associated context ID
    pub context_id: Option<String>,
    /// Resources to clean up
    pub resources: Vec<CleanupResource>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Execution timestamp
    pub executed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Success status
    pub success: Option<bool>,
    /// Error message
    pub error: Option<String>,
}

/// Cleanup resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResource {
    /// Resource type
    pub resource_type: String,
    /// Resource identifier
    pub identifier: String,
    /// Resource metadata
    pub metadata: HashMap<String, String>,
}

/// Cleanup handler trait
#[async_trait]
pub trait CleanupHandler: Send + Sync {
    /// Get the handler name
    fn name(&self) -> &str;

    /// Get the handler description
    fn description(&self) -> Option<&str>;

    /// Get the supported resource types
    fn supported_resource_types(&self) -> Vec<String>;

    /// Clean up a resource
    async fn cleanup_resource(&self, resource: &CleanupResource) -> Result<(), String>;
}

/// File cleanup handler
pub struct FileCleanupHandler {
    /// Handler name
    name: String,
    /// Handler description
    description: Option<String>,
    /// Base directory
    base_dir: Option<PathBuf>,
}

impl FileCleanupHandler {
    /// Create a new file cleanup handler
    pub fn new() -> Self {
        Self {
            name: "file".to_string(),
            description: Some("File cleanup handler".to_string()),
            base_dir: None,
        }
    }

    /// Create a new file cleanup handler with a base directory
    pub fn with_base_dir(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            name: "file".to_string(),
            description: Some("File cleanup handler".to_string()),
            base_dir: Some(base_dir.into()),
        }
    }
}

#[async_trait]
impl CleanupHandler for FileCleanupHandler {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn supported_resource_types(&self) -> Vec<String> {
        vec!["file".to_string(), "directory".to_string()]
    }

    async fn cleanup_resource(&self, resource: &CleanupResource) -> Result<(), String> {
        match resource.resource_type.as_str() {
            "file" => {
                let path = if let Some(base_dir) = &self.base_dir {
                    base_dir.join(&resource.identifier)
                } else {
                    PathBuf::from(&resource.identifier)
                };

                if path.exists() {
                    fs::remove_file(&path)
                        .await
                        .map_err(|e| format!("Failed to remove file: {}", e))?;
                }

                Ok(())
            }
            "directory" => {
                let path = if let Some(base_dir) = &self.base_dir {
                    base_dir.join(&resource.identifier)
                } else {
                    PathBuf::from(&resource.identifier)
                };

                if path.exists() {
                    fs::remove_dir_all(&path)
                        .await
                        .map_err(|e| format!("Failed to remove directory: {}", e))?;
                }

                Ok(())
            }
            _ => Err(format!(
                "Unsupported resource type: {}",
                resource.resource_type
            )),
        }
    }
}

/// Data store cleanup handler
pub struct DataStoreCleanupHandler {
    /// Handler name
    name: String,
    /// Handler description
    description: Option<String>,
    /// Data store
    data_store: Arc<DataStore>,
}

impl DataStoreCleanupHandler {
    /// Create a new data store cleanup handler
    pub fn new(data_store: Arc<DataStore>) -> Self {
        Self {
            name: "data-store".to_string(),
            description: Some("Data store cleanup handler".to_string()),
            data_store,
        }
    }
}

#[async_trait]
impl CleanupHandler for DataStoreCleanupHandler {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn supported_resource_types(&self) -> Vec<String> {
        vec![
            "data-store-key".to_string(),
            "data-store-prefix".to_string(),
        ]
    }

    async fn cleanup_resource(&self, resource: &CleanupResource) -> Result<(), String> {
        match resource.resource_type.as_str() {
            "data-store-key" => {
                self.data_store
                    .remove(&resource.identifier)
                    .await
                    .map_err(|e| format!("Failed to remove data store key: {}", e))?;

                Ok(())
            }
            "data-store-prefix" => {
                let keys = self
                    .data_store
                    .keys()
                    .await
                    .map_err(|e| format!("Failed to get keys: {}", e))?;

                for key in keys {
                    if key.starts_with(&resource.identifier) {
                        self.data_store
                            .remove(&key)
                            .await
                            .map_err(|e| format!("Failed to remove data store key: {}", e))?;
                    }
                }

                Ok(())
            }
            _ => Err(format!(
                "Unsupported resource type: {}",
                resource.resource_type
            )),
        }
    }
}

/// Isolation context cleanup handler
pub struct IsolationCleanupHandler {
    /// Handler name
    name: String,
    /// Handler description
    description: Option<String>,
    /// Isolation manager
    isolation_manager: Arc<IsolationManager>,
}

impl IsolationCleanupHandler {
    /// Create a new isolation context cleanup handler
    pub fn new(isolation_manager: Arc<IsolationManager>) -> Self {
        Self {
            name: "isolation".to_string(),
            description: Some("Isolation context cleanup handler".to_string()),
            isolation_manager,
        }
    }
}

#[async_trait]
impl CleanupHandler for IsolationCleanupHandler {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn supported_resource_types(&self) -> Vec<String> {
        vec!["isolation-context".to_string()]
    }

    async fn cleanup_resource(&self, resource: &CleanupResource) -> Result<(), String> {
        match resource.resource_type.as_str() {
            "isolation-context" => {
                self.isolation_manager
                    .cleanup_context(&resource.identifier)
                    .await
                    .map_err(|e| format!("Failed to clean up isolation context: {}", e))?;

                Ok(())
            }
            _ => Err(format!(
                "Unsupported resource type: {}",
                resource.resource_type
            )),
        }
    }
}

/// Cleanup manager
pub struct CleanupManager {
    /// Manager configuration
    config: CleanupConfig,
    /// Cleanup handlers
    handlers: RwLock<HashMap<String, Arc<dyn CleanupHandler>>>,
    /// Cleanup tasks
    tasks: RwLock<HashMap<String, CleanupTask>>,
    /// Pending tasks (by strategy)
    pending_tasks: RwLock<HashMap<CleanupStrategy, HashSet<String>>>,
}

impl CleanupManager {
    /// Create a new cleanup manager
    pub fn new() -> Self {
        Self {
            config: CleanupConfig::default(),
            handlers: RwLock::new(HashMap::new()),
            tasks: RwLock::new(HashMap::new()),
            pending_tasks: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new cleanup manager with a custom configuration
    pub fn with_config(config: CleanupConfig) -> Self {
        Self {
            config,
            handlers: RwLock::new(HashMap::new()),
            tasks: RwLock::new(HashMap::new()),
            pending_tasks: RwLock::new(HashMap::new()),
        }
    }

    /// Register a cleanup handler
    pub async fn register_handler(
        &self,
        name: impl Into<String>,
        handler: Arc<dyn CleanupHandler>,
    ) {
        let mut handlers = self.handlers.write().await;
        handlers.insert(name.into(), handler);
    }

    /// Get a cleanup handler by name
    pub async fn get_handler(&self, name: &str) -> Option<Arc<dyn CleanupHandler>> {
        let handlers = self.handlers.read().await;
        handlers.get(name).cloned()
    }

    /// Create a cleanup task
    pub async fn create_task(
        &self,
        name: impl Into<String>,
        strategy: Option<CleanupStrategy>,
        scope: Option<CleanupScope>,
        context_id: Option<String>,
    ) -> Result<CleanupTask, String> {
        let now = chrono::Utc::now();
        let task_id = uuid::Uuid::new_v4().to_string();

        let task = CleanupTask {
            id: task_id.clone(),
            name: name.into(),
            description: None,
            strategy: strategy.unwrap_or(self.config.default_strategy),
            scope: scope.unwrap_or(self.config.default_scope),
            context_id,
            resources: Vec::new(),
            created_at: now,
            executed_at: None,
            success: None,
            error: None,
        };

        // Add the task
        let mut tasks = self.tasks.write().await;
        tasks.insert(task_id.clone(), task.clone());

        // Add to pending tasks
        let mut pending = self.pending_tasks.write().await;
        pending
            .entry(task.strategy)
            .or_insert_with(HashSet::new)
            .insert(task_id);

        Ok(task)
    }

    /// Get a cleanup task by ID
    pub async fn get_task(&self, id: &str) -> Result<Option<CleanupTask>, String> {
        let tasks = self.tasks.read().await;
        Ok(tasks.get(id).cloned())
    }

    /// Add a resource to a cleanup task
    pub async fn add_resource(
        &self,
        task_id: &str,
        resource_type: impl Into<String>,
        identifier: impl Into<String>,
    ) -> Result<(), String> {
        let resource = CleanupResource {
            resource_type: resource_type.into(),
            identifier: identifier.into(),
            metadata: HashMap::new(),
        };

        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| format!("Task '{}' not found", task_id))?;

        task.resources.push(resource);

        Ok(())
    }

    /// Add a resource with metadata to a cleanup task
    pub async fn add_resource_with_metadata(
        &self,
        task_id: &str,
        resource_type: impl Into<String>,
        identifier: impl Into<String>,
        metadata: HashMap<String, String>,
    ) -> Result<(), String> {
        let resource = CleanupResource {
            resource_type: resource_type.into(),
            identifier: identifier.into(),
            metadata,
        };

        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| format!("Task '{}' not found", task_id))?;

        task.resources.push(resource);

        Ok(())
    }

    /// Execute a cleanup task
    pub async fn execute_task(&self, id: &str) -> Result<(), String> {
        // Get the task
        let task = {
            let tasks = self.tasks.read().await;
            tasks
                .get(id)
                .cloned()
                .ok_or_else(|| format!("Task '{}' not found", id))?
        };

        // Execute each resource cleanup
        let mut success = true;
        let mut error_message = None;

        for resource in &task.resources {
            // Find a handler for the resource type
            let handler = {
                let handlers = self.handlers.read().await;
                let mut matching_handler = None;

                for handler in handlers.values() {
                    if handler
                        .supported_resource_types()
                        .contains(&resource.resource_type)
                    {
                        matching_handler = Some(handler.clone());
                        break;
                    }
                }

                matching_handler.ok_or_else(|| {
                    format!(
                        "No handler found for resource type '{}'",
                        resource.resource_type
                    )
                })?
            };

            // Clean up the resource
            match handler.cleanup_resource(resource).await {
                Ok(()) => {
                    debug!(
                        "Cleaned up resource '{}' of type '{}'",
                        resource.identifier, resource.resource_type
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to clean up resource '{}' of type '{}': {}",
                        resource.identifier, resource.resource_type, e
                    );
                    success = false;
                    error_message = Some(e);
                }
            }
        }

        // Update the task status
        let now = chrono::Utc::now();
        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(id) {
                task.executed_at = Some(now);
                task.success = Some(success);
                task.error = error_message;
            }
        }

        // Remove from pending tasks
        {
            let mut pending = self.pending_tasks.write().await;
            if let Some(tasks) = pending.get_mut(&task.strategy) {
                tasks.remove(id);
            }
        }

        if success {
            Ok(())
        } else {
            Err(error_message.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }

    /// Execute all pending tasks for a specific strategy
    pub async fn execute_pending_tasks(&self, strategy: CleanupStrategy) -> Result<(), String> {
        // Get the pending task IDs
        let task_ids = {
            let pending = self.pending_tasks.read().await;
            pending.get(&strategy).cloned().unwrap_or_else(HashSet::new)
        };

        // Execute each task
        let mut errors = Vec::new();
        for id in task_ids {
            match self.execute_task(&id).await {
                Ok(()) => {}
                Err(e) => {
                    errors.push(format!("Task '{}': {}", id, e));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "Failed to execute some tasks: {}",
                errors.join(", ")
            ))
        }
    }

    /// Execute all pending immediate tasks
    pub async fn execute_immediate_tasks(&self) -> Result<(), String> {
        self.execute_pending_tasks(CleanupStrategy::Immediate).await
    }

    /// Execute all pending deferred tasks
    pub async fn execute_deferred_tasks(&self) -> Result<(), String> {
        self.execute_pending_tasks(CleanupStrategy::Deferred).await
    }

    /// Get all tasks
    pub async fn get_all_tasks(&self) -> Result<Vec<CleanupTask>, String> {
        let tasks = self.tasks.read().await;
        Ok(tasks.values().cloned().collect())
    }

    /// Get pending tasks for a strategy
    pub async fn get_pending_tasks(
        &self,
        strategy: CleanupStrategy,
    ) -> Result<Vec<CleanupTask>, String> {
        let pending_ids = {
            let pending = self.pending_tasks.read().await;
            pending.get(&strategy).cloned().unwrap_or_else(HashSet::new)
        };

        let tasks = self.tasks.read().await;
        let pending_tasks = pending_ids
            .iter()
            .filter_map(|id| tasks.get(id).cloned())
            .collect();

        Ok(pending_tasks)
    }

    /// Clear all tasks
    pub async fn clear_tasks(&self) -> Result<(), String> {
        let mut tasks = self.tasks.write().await;
        tasks.clear();

        let mut pending = self.pending_tasks.write().await;
        pending.clear();

        Ok(())
    }
}

impl Default for CleanupManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cleanup_manager() {
        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // Create a test file
        let test_file = temp_path.join("test.txt");
        tokio::fs::write(&test_file, b"test data").await.unwrap();

        // Create a cleanup manager
        let manager = CleanupManager::new();

        // Register a file cleanup handler
        let file_handler = Arc::new(FileCleanupHandler::with_base_dir(&temp_path));
        manager.register_handler("file", file_handler).await;

        // Create a cleanup task
        let task = manager
            .create_task("test-cleanup", Some(CleanupStrategy::Immediate), None, None)
            .await
            .unwrap();

        // Add a resource to the task
        manager
            .add_resource(&task.id, "file", "test.txt")
            .await
            .unwrap();

        // Execute the task
        manager.execute_task(&task.id).await.unwrap();

        // Check that the file was removed
        assert!(!test_file.exists());

        // Check that the task was updated
        let updated_task = manager.get_task(&task.id).await.unwrap().unwrap();
        assert!(updated_task.executed_at.is_some());
        assert_eq!(updated_task.success, Some(true));
    }
}
