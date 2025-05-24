//! Data Isolation Module
//!
//! This module provides functionality for isolating test data between test runs.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::loader::{DataLoader, LoadedData};
use super::store::{DataStore, StoredData};

/// Data isolation level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationLevel {
    /// No isolation (shared data)
    None,
    /// Test run isolation (data isolated per test run)
    TestRun,
    /// Test case isolation (data isolated per test case)
    TestCase,
    /// Test suite isolation (data isolated per test suite)
    TestSuite,
}

/// Data isolation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationContext {
    /// Context ID
    pub id: String,
    /// Context name
    pub name: String,
    /// Context type
    pub context_type: String,
    /// Parent context ID
    pub parent_id: Option<String>,
    /// Isolation level
    pub isolation_level: IsolationLevel,
    /// Context metadata
    pub metadata: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl IsolationContext {
    /// Create a new isolation context
    pub fn new(
        name: impl Into<String>,
        context_type: impl Into<String>,
        isolation_level: IsolationLevel,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            context_type: context_type.into(),
            parent_id: None,
            isolation_level,
            metadata: HashMap::new(),
            created_at: now,
        }
    }

    /// Create a new isolation context with a specific ID
    pub fn with_id(
        id: impl Into<String>,
        name: impl Into<String>,
        context_type: impl Into<String>,
        isolation_level: IsolationLevel,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: id.into(),
            name: name.into(),
            context_type: context_type.into(),
            parent_id: None,
            isolation_level,
            metadata: HashMap::new(),
            created_at: now,
        }
    }

    /// Set the parent context ID
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add multiple metadata entries
    pub fn with_metadata_entries(
        mut self,
        entries: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (key, value) in entries {
            self.metadata.insert(key.into(), value.into());
        }
        self
    }

    /// Create a child context
    pub fn create_child(&self, name: impl Into<String>, context_type: impl Into<String>) -> Self {
        let mut child = Self::new(name, context_type, self.isolation_level);
        child.parent_id = Some(self.id.clone());

        // Copy relevant metadata
        for (key, value) in &self.metadata {
            if key.starts_with("shared.") {
                child.metadata.insert(key.clone(), value.clone());
            }
        }

        child
    }

    /// Get the context key prefix for data storage
    pub fn key_prefix(&self) -> String {
        match self.isolation_level {
            IsolationLevel::None => "shared".to_string(),
            IsolationLevel::TestRun => format!("run.{}", self.id),
            IsolationLevel::TestCase => format!("case.{}", self.id),
            IsolationLevel::TestSuite => format!("suite.{}", self.id),
        }
    }

    /// Create a data key with the appropriate prefix
    pub fn create_key(&self, key: &str) -> String {
        format!("{}.{}", self.key_prefix(), key)
    }
}

/// Data isolation manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationManagerConfig {
    /// Default isolation level
    pub default_isolation_level: IsolationLevel,
    /// Base directory for isolated data
    pub base_dir: Option<PathBuf>,
    /// Custom configuration
    pub custom: Option<serde_json::Value>,
}

impl Default for IsolationManagerConfig {
    fn default() -> Self {
        Self {
            default_isolation_level: IsolationLevel::TestRun,
            base_dir: None,
            custom: None,
        }
    }
}

/// Data isolation manager
pub struct IsolationManager {
    /// Manager configuration
    config: IsolationManagerConfig,
    /// Active contexts
    contexts: RwLock<HashMap<String, IsolationContext>>,
    /// Current context stack (for nested contexts)
    context_stack: Mutex<Vec<String>>,
    /// Data store
    data_store: Arc<DataStore>,
    /// Data loader
    loader: Option<Arc<dyn DataLoader>>,
}

impl IsolationManager {
    /// Create a new isolation manager
    pub fn new() -> Self {
        Self {
            config: IsolationManagerConfig::default(),
            contexts: RwLock::new(HashMap::new()),
            context_stack: Mutex::new(Vec::new()),
            data_store: Arc::new(DataStore::new()),
            loader: None,
        }
    }

    /// Create a new isolation manager with a custom configuration
    pub fn with_config(config: IsolationManagerConfig) -> Self {
        Self {
            config,
            contexts: RwLock::new(HashMap::new()),
            context_stack: Mutex::new(Vec::new()),
            data_store: Arc::new(DataStore::new()),
            loader: None,
        }
    }

    /// Set the data loader
    pub fn with_loader(mut self, loader: Arc<dyn DataLoader>) -> Self {
        self.loader = Some(loader);
        self
    }

    /// Set the data store
    pub fn with_data_store(mut self, data_store: Arc<DataStore>) -> Self {
        self.data_store = data_store;
        self
    }

    /// Create a new context
    pub async fn create_context(
        &self,
        name: impl Into<String>,
        context_type: impl Into<String>,
        isolation_level: Option<IsolationLevel>,
    ) -> Result<IsolationContext, String> {
        let isolation_level = isolation_level.unwrap_or(self.config.default_isolation_level);
        let context = IsolationContext::new(name, context_type, isolation_level);

        // Add the context to the active contexts
        let mut contexts = self.contexts.write().await;
        contexts.insert(context.id.clone(), context.clone());

        Ok(context)
    }

    /// Create a child context
    pub async fn create_child_context(
        &self,
        parent_id: &str,
        name: impl Into<String>,
        context_type: impl Into<String>,
    ) -> Result<IsolationContext, String> {
        // Get the parent context
        let parent = {
            let contexts = self.contexts.read().await;
            contexts
                .get(parent_id)
                .cloned()
                .ok_or_else(|| format!("Parent context '{}' not found", parent_id))?
        };

        // Create the child context
        let child = parent.create_child(name, context_type);

        // Add the child context to the active contexts
        let mut contexts = self.contexts.write().await;
        contexts.insert(child.id.clone(), child.clone());

        Ok(child)
    }

    /// Get a context by ID
    pub async fn get_context(&self, id: &str) -> Result<Option<IsolationContext>, String> {
        let contexts = self.contexts.read().await;
        Ok(contexts.get(id).cloned())
    }

    /// Push a context onto the stack
    pub async fn push_context(&self, id: &str) -> Result<(), String> {
        // Check if the context exists
        let exists = {
            let contexts = self.contexts.read().await;
            contexts.contains_key(id)
        };

        if !exists {
            return Err(format!("Context '{}' not found", id));
        }

        // Push the context onto the stack
        let mut stack = self.context_stack.lock().await;
        stack.push(id.to_string());

        Ok(())
    }

    /// Pop a context from the stack
    pub async fn pop_context(&self) -> Result<Option<IsolationContext>, String> {
        let mut stack = self.context_stack.lock().await;
        let id = stack.pop();

        if let Some(id) = id {
            let contexts = self.contexts.read().await;
            Ok(contexts.get(&id).cloned())
        } else {
            Ok(None)
        }
    }

    /// Get the current context
    pub async fn current_context(&self) -> Result<Option<IsolationContext>, String> {
        let stack = self.context_stack.lock().await;

        if let Some(id) = stack.last() {
            let contexts = self.contexts.read().await;
            Ok(contexts.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    /// Store data in the current context
    pub async fn store_data(&self, key: &str, data: serde_json::Value) -> Result<(), String> {
        // Get the current context
        let context = self
            .current_context()
            .await?
            .ok_or_else(|| "No active context".to_string())?;

        // Create the prefixed key
        let prefixed_key = context.create_key(key);

        // Store the data
        self.data_store
            .store(prefixed_key, data)
            .await
            .map_err(|e| format!("Failed to store data: {}", e))
    }

    /// Retrieve data from the current context
    pub async fn retrieve_data(&self, key: &str) -> Result<Option<StoredData>, String> {
        // Get the current context
        let context = self
            .current_context()
            .await?
            .ok_or_else(|| "No active context".to_string())?;

        // Create the prefixed key
        let prefixed_key = context.create_key(key);

        // Retrieve the data
        self.data_store
            .retrieve(&prefixed_key)
            .await
            .map_err(|e| format!("Failed to retrieve data: {}", e))
    }

    /// Remove data from the current context
    pub async fn remove_data(&self, key: &str) -> Result<Option<StoredData>, String> {
        // Get the current context
        let context = self
            .current_context()
            .await?
            .ok_or_else(|| "No active context".to_string())?;

        // Create the prefixed key
        let prefixed_key = context.create_key(key);

        // Remove the data
        self.data_store
            .remove(&prefixed_key)
            .await
            .map_err(|e| format!("Failed to remove data: {}", e))
    }

    /// Clear all data for a context
    pub async fn clear_context_data(&self, id: &str) -> Result<(), String> {
        // Get the context
        let context = {
            let contexts = self.contexts.read().await;
            contexts
                .get(id)
                .cloned()
                .ok_or_else(|| format!("Context '{}' not found", id))?
        };

        // Get all keys with the context prefix
        let prefix = context.key_prefix();
        let keys = self
            .data_store
            .keys()
            .await
            .map_err(|e| format!("Failed to get keys: {}", e))?;

        // Remove all keys with the context prefix
        for key in keys {
            if key.starts_with(&prefix) {
                self.data_store
                    .remove(&key)
                    .await
                    .map_err(|e| format!("Failed to remove data: {}", e))?;
            }
        }

        Ok(())
    }

    /// Remove a context and its data
    pub async fn remove_context(&self, id: &str) -> Result<Option<IsolationContext>, String> {
        // Clear the context data
        self.clear_context_data(id).await?;

        // Remove the context
        let mut contexts = self.contexts.write().await;
        let removed = contexts.remove(id);

        // Remove from the stack if present
        let mut stack = self.context_stack.lock().await;
        stack.retain(|ctx_id| ctx_id != id);

        Ok(removed)
    }

    /// Save context data to disk
    pub async fn save_context_data_to_disk(&self, id: &str) -> Result<(), String> {
        if let (Some(base_dir), Some(loader)) = (&self.config.base_dir, &self.loader) {
            // Get the context
            let context = {
                let contexts = self.contexts.read().await;
                contexts
                    .get(id)
                    .cloned()
                    .ok_or_else(|| format!("Context '{}' not found", id))?
            };

            // Create the context directory
            let context_dir = base_dir.join("contexts").join(&context.id);
            if !context_dir.exists() {
                fs::create_dir_all(&context_dir)
                    .await
                    .map_err(|e| format!("Failed to create context directory: {}", e))?;
            }

            // Save the context metadata
            let context_file = context_dir.join("context.json");
            let context_data = LoadedData {
                data: serde_json::to_value(&context).unwrap(),
                metadata: HashMap::new(),
            };

            loader
                .save(&context_file, &context_data)
                .await
                .map_err(|e| format!("Failed to save context metadata: {}", e))?;

            // Get all keys with the context prefix
            let prefix = context.key_prefix();
            let keys = self
                .data_store
                .keys()
                .await
                .map_err(|e| format!("Failed to get keys: {}", e))?;

            // Save all data with the context prefix
            for key in keys {
                if key.starts_with(&prefix) {
                    if let Some(data) = self
                        .data_store
                        .retrieve(&key)
                        .await
                        .map_err(|e| format!("Failed to retrieve data: {}", e))?
                    {
                        // Create a safe filename from the key
                        let safe_key = key.replace(".", "_").replace("/", "_").replace("\\", "_");
                        let data_file = context_dir.join(format!("{}.json", safe_key));

                        // Save the data
                        let loaded_data = LoadedData {
                            data: data.data,
                            metadata: HashMap::new(),
                        };

                        loader
                            .save(&data_file, &loaded_data)
                            .await
                            .map_err(|e| format!("Failed to save data: {}", e))?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Load context data from disk
    pub async fn load_context_data_from_disk(&self, id: &str) -> Result<(), String> {
        if let (Some(base_dir), Some(loader)) = (&self.config.base_dir, &self.loader) {
            let context_dir = base_dir.join("contexts").join(id);
            if !context_dir.exists() {
                return Ok(());
            }

            // Load the context metadata
            let context_file = context_dir.join("context.json");
            if context_file.exists() {
                let context_data = loader
                    .load(&context_file)
                    .await
                    .map_err(|e| format!("Failed to load context metadata: {}", e))?;

                let context: IsolationContext = serde_json::from_value(context_data.data)
                    .map_err(|e| format!("Failed to parse context metadata: {}", e))?;

                // Add the context to the active contexts
                let mut contexts = self.contexts.write().await;
                contexts.insert(context.id.clone(), context);
            }

            // Load all data files
            let mut entries = fs::read_dir(&context_dir)
                .await
                .map_err(|e| format!("Failed to read context directory: {}", e))?;

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| format!("Failed to read directory entry: {}", e))?
            {
                let path = entry.path();
                if path.is_file() && path.file_name().unwrap_or_default() != "context.json" {
                    // Load the data
                    let loaded_data = loader
                        .load(&path)
                        .await
                        .map_err(|e| format!("Failed to load data: {}", e))?;

                    // Get the key from the filename
                    let filename = path.file_stem().unwrap_or_default().to_string_lossy();
                    let key = filename.replace("_", ".");

                    // Store the data
                    self.data_store
                        .store(key, loaded_data.data)
                        .await
                        .map_err(|e| format!("Failed to store data: {}", e))?;
                }
            }
        }

        Ok(())
    }

    /// Clean up all data for a context and its children
    pub async fn cleanup_context(&self, id: &str) -> Result<(), String> {
        // Get the context
        let context = {
            let contexts = self.contexts.read().await;
            contexts
                .get(id)
                .cloned()
                .ok_or_else(|| format!("Context '{}' not found", id))?
        };

        // Find all child contexts
        let child_ids = {
            let contexts = self.contexts.read().await;
            contexts
                .values()
                .filter(|c| c.parent_id.as_ref() == Some(&context.id))
                .map(|c| c.id.clone())
                .collect::<Vec<_>>()
        };

        // Recursively clean up child contexts
        for child_id in child_ids {
            self.cleanup_context(&child_id).await?;
        }

        // Clear the context data
        self.clear_context_data(&context.id).await?;

        // Remove the context
        let mut contexts = self.contexts.write().await;
        contexts.remove(&context.id);

        // Remove from the stack if present
        let mut stack = self.context_stack.lock().await;
        stack.retain(|ctx_id| ctx_id != &context.id);

        // Remove from disk if needed
        if let (Some(base_dir), Some(_)) = (&self.config.base_dir, &self.loader) {
            let context_dir = base_dir.join("contexts").join(&context.id);
            if context_dir.exists() {
                fs::remove_dir_all(&context_dir)
                    .await
                    .map_err(|e| format!("Failed to remove context directory: {}", e))?;
            }
        }

        Ok(())
    }
}

impl Default for IsolationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_isolation_manager() {
        // Create an isolation manager
        let manager = IsolationManager::new();

        // Create a test run context
        let run_context = manager
            .create_context("test-run", "run", Some(IsolationLevel::TestRun))
            .await
            .unwrap();

        // Push the context onto the stack
        manager.push_context(&run_context.id).await.unwrap();

        // Store some data
        manager
            .store_data("key1", serde_json::json!("value1"))
            .await
            .unwrap();

        // Create a test case context
        let case_context = manager
            .create_child_context(&run_context.id, "test-case", "case")
            .await
            .unwrap();

        // Push the case context onto the stack
        manager.push_context(&case_context.id).await.unwrap();

        // Store some data in the case context
        manager
            .store_data("key2", serde_json::json!("value2"))
            .await
            .unwrap();

        // Retrieve the data
        let data2 = manager.retrieve_data("key2").await.unwrap().unwrap();
        assert_eq!(data2.data, serde_json::json!("value2"));

        // Pop the case context
        manager.pop_context().await.unwrap();

        // Try to retrieve the case data from the run context (should fail)
        let result = manager.retrieve_data("key2").await.unwrap();
        assert!(result.is_none());

        // Retrieve the run data
        let data1 = manager.retrieve_data("key1").await.unwrap().unwrap();
        assert_eq!(data1.data, serde_json::json!("value1"));

        // Clean up the run context (should also clean up the case context)
        manager.cleanup_context(&run_context.id).await.unwrap();

        // Check that the contexts are gone
        let run_result = manager.get_context(&run_context.id).await.unwrap();
        assert!(run_result.is_none());

        let case_result = manager.get_context(&case_context.id).await.unwrap();
        assert!(case_result.is_none());
    }
}
