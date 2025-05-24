//! Data Store Module
//!
//! This module provides functionality for storing and retrieving test data.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Stored data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredData {
    /// Data
    pub data: serde_json::Value,
    /// Metadata
    pub metadata: HashMap<String, String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Data store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStoreConfig {
    /// Store name
    pub name: String,
    /// Store description
    pub description: Option<String>,
    /// Maximum number of entries
    pub max_entries: Option<usize>,
    /// Time to live in seconds
    pub ttl: Option<u64>,
    /// Custom configuration
    pub custom: Option<serde_json::Value>,
}

impl Default for DataStoreConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: None,
            max_entries: None,
            ttl: None,
            custom: None,
        }
    }
}

/// Data store
pub struct DataStore {
    /// Store configuration
    config: DataStoreConfig,
    /// Store data
    data: RwLock<HashMap<String, StoredData>>,
}

impl DataStore {
    /// Create a new data store
    pub fn new() -> Self {
        Self {
            config: DataStoreConfig::default(),
            data: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new data store with a custom configuration
    pub fn with_config(config: DataStoreConfig) -> Self {
        Self {
            config,
            data: RwLock::new(HashMap::new()),
        }
    }

    /// Store data
    pub async fn store(
        &self,
        key: impl Into<String>,
        data: serde_json::Value,
    ) -> Result<(), String> {
        let key = key.into();
        let now = chrono::Utc::now();

        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("store".to_string(), self.config.name.clone());
        metadata.insert("timestamp".to_string(), now.to_rfc3339());

        // Create stored data
        let stored_data = StoredData {
            data,
            metadata,
            timestamp: now,
        };

        // Store the data
        let mut store = self.data.write().await;
        store.insert(key, stored_data);

        // Check if we need to enforce max entries
        if let Some(max_entries) = self.config.max_entries {
            if store.len() > max_entries {
                // Remove the oldest entries
                let mut entries: Vec<_> = store.iter().collect();
                entries.sort_by(|(_, a), (_, b)| a.timestamp.cmp(&b.timestamp));
                let to_remove = entries.len() - max_entries;
                for (key, _) in entries.iter().take(to_remove) {
                    store.remove(*key);
                }
            }
        }

        // Check if we need to enforce TTL
        if let Some(ttl) = self.config.ttl {
            let ttl_duration = chrono::Duration::seconds(ttl as i64);
            let cutoff = now - ttl_duration;

            // Remove expired entries
            store.retain(|_, v| v.timestamp > cutoff);
        }

        Ok(())
    }

    /// Retrieve data
    pub async fn retrieve(&self, key: &str) -> Result<Option<StoredData>, String> {
        let store = self.data.read().await;
        Ok(store.get(key).cloned())
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> Result<bool, String> {
        let store = self.data.read().await;
        Ok(store.contains_key(key))
    }

    /// Remove data
    pub async fn remove(&self, key: &str) -> Result<Option<StoredData>, String> {
        let mut store = self.data.write().await;
        Ok(store.remove(key))
    }

    /// Clear the store
    pub async fn clear(&self) -> Result<(), String> {
        let mut store = self.data.write().await;
        store.clear();
        Ok(())
    }

    /// Get all keys
    pub async fn keys(&self) -> Result<Vec<String>, String> {
        let store = self.data.read().await;
        Ok(store.keys().cloned().collect())
    }

    /// Get the number of entries
    pub async fn len(&self) -> Result<usize, String> {
        let store = self.data.read().await;
        Ok(store.len())
    }

    /// Check if the store is empty
    pub async fn is_empty(&self) -> Result<bool, String> {
        let store = self.data.read().await;
        Ok(store.is_empty())
    }

    /// Get all entries
    pub async fn entries(&self) -> Result<HashMap<String, StoredData>, String> {
        let store = self.data.read().await;
        Ok(store.clone())
    }

    /// Get entries with a prefix
    pub async fn entries_with_prefix(
        &self,
        prefix: &str,
    ) -> Result<HashMap<String, StoredData>, String> {
        let store = self.data.read().await;
        let mut result = HashMap::new();
        for (key, value) in store.iter() {
            if key.starts_with(prefix) {
                result.insert(key.clone(), value.clone());
            }
        }
        Ok(result)
    }

    /// Get entries with a suffix
    pub async fn entries_with_suffix(
        &self,
        suffix: &str,
    ) -> Result<HashMap<String, StoredData>, String> {
        let store = self.data.read().await;
        let mut result = HashMap::new();
        for (key, value) in store.iter() {
            if key.ends_with(suffix) {
                result.insert(key.clone(), value.clone());
            }
        }
        Ok(result)
    }

    /// Get entries with a pattern
    pub async fn entries_with_pattern(
        &self,
        pattern: &str,
    ) -> Result<HashMap<String, StoredData>, String> {
        let store = self.data.read().await;
        let regex = regex::Regex::new(pattern).map_err(|e| format!("Invalid regex: {}", e))?;
        let mut result = HashMap::new();
        for (key, value) in store.iter() {
            if regex.is_match(key) {
                result.insert(key.clone(), value.clone());
            }
        }
        Ok(result)
    }

    /// Get entries with a filter
    pub async fn entries_with_filter<F>(
        &self,
        filter: F,
    ) -> Result<HashMap<String, StoredData>, String>
    where
        F: Fn(&str, &StoredData) -> bool,
    {
        let store = self.data.read().await;
        let mut result = HashMap::new();
        for (key, value) in store.iter() {
            if filter(key, value) {
                result.insert(key.clone(), value.clone());
            }
        }
        Ok(result)
    }

    /// Get the configuration
    pub fn config(&self) -> &DataStoreConfig {
        &self.config
    }
}

impl Default for DataStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Persistent data store
pub struct PersistentDataStore {
    /// Store configuration
    config: DataStoreConfig,
    /// Store data
    data: RwLock<HashMap<String, StoredData>>,
    /// Store file
    file: Option<std::path::PathBuf>,
}

impl PersistentDataStore {
    /// Create a new persistent data store
    pub fn new(file: impl Into<std::path::PathBuf>) -> Self {
        Self {
            config: DataStoreConfig::default(),
            data: RwLock::new(HashMap::new()),
            file: Some(file.into()),
        }
    }

    /// Create a new persistent data store with a custom configuration
    pub fn with_config(file: impl Into<std::path::PathBuf>, config: DataStoreConfig) -> Self {
        Self {
            config,
            data: RwLock::new(HashMap::new()),
            file: Some(file.into()),
        }
    }

    /// Load the store from a file
    pub async fn load(&self) -> Result<(), String> {
        if let Some(file) = &self.file {
            // Check if the file exists
            if !file.exists() {
                return Ok(());
            }

            // Read the file
            let content = tokio::fs::read_to_string(file)
                .await
                .map_err(|e| format!("Failed to read file: {}", e))?;

            // Parse the JSON
            let data: HashMap<String, StoredData> = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;

            // Update the store
            let mut store = self.data.write().await;
            *store = data;
        }

        Ok(())
    }

    /// Save the store to a file
    pub async fn save(&self) -> Result<(), String> {
        if let Some(file) = &self.file {
            // Create parent directories if they don't exist
            if let Some(parent) = file.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|e| format!("Failed to create directories: {}", e))?;
            }

            // Get the store data
            let store = self.data.read().await;

            // Serialize the data
            let content = serde_json::to_string_pretty(&*store)
                .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

            // Write the file
            tokio::fs::write(file, content)
                .await
                .map_err(|e| format!("Failed to write file: {}", e))?;
        }

        Ok(())
    }

    /// Store data
    pub async fn store(
        &self,
        key: impl Into<String>,
        data: serde_json::Value,
    ) -> Result<(), String> {
        let key = key.into();
        let now = chrono::Utc::now();

        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("store".to_string(), self.config.name.clone());
        metadata.insert("timestamp".to_string(), now.to_rfc3339());

        // Create stored data
        let stored_data = StoredData {
            data,
            metadata,
            timestamp: now,
        };

        // Store the data
        let mut store = self.data.write().await;
        store.insert(key, stored_data);

        // Check if we need to enforce max entries
        if let Some(max_entries) = self.config.max_entries {
            if store.len() > max_entries {
                // Remove the oldest entries
                let mut entries: Vec<_> = store.iter().collect();
                entries.sort_by(|(_, a), (_, b)| a.timestamp.cmp(&b.timestamp));
                let to_remove = entries.len() - max_entries;
                for (key, _) in entries.iter().take(to_remove) {
                    store.remove(*key);
                }
            }
        }

        // Check if we need to enforce TTL
        if let Some(ttl) = self.config.ttl {
            let ttl_duration = chrono::Duration::seconds(ttl as i64);
            let cutoff = now - ttl_duration;

            // Remove expired entries
            store.retain(|_, v| v.timestamp > cutoff);
        }

        // Save the store
        drop(store);
        self.save().await?;

        Ok(())
    }

    /// Retrieve data
    pub async fn retrieve(&self, key: &str) -> Result<Option<StoredData>, String> {
        let store = self.data.read().await;
        Ok(store.get(key).cloned())
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> Result<bool, String> {
        let store = self.data.read().await;
        Ok(store.contains_key(key))
    }

    /// Remove data
    pub async fn remove(&self, key: &str) -> Result<Option<StoredData>, String> {
        let mut store = self.data.write().await;
        let result = store.remove(key);

        // Save the store
        drop(store);
        self.save().await?;

        Ok(result)
    }

    /// Clear the store
    pub async fn clear(&self) -> Result<(), String> {
        let mut store = self.data.write().await;
        store.clear();

        // Save the store
        drop(store);
        self.save().await?;

        Ok(())
    }

    /// Get all keys
    pub async fn keys(&self) -> Result<Vec<String>, String> {
        let store = self.data.read().await;
        Ok(store.keys().cloned().collect())
    }

    /// Get the number of entries
    pub async fn len(&self) -> Result<usize, String> {
        let store = self.data.read().await;
        Ok(store.len())
    }

    /// Check if the store is empty
    pub async fn is_empty(&self) -> Result<bool, String> {
        let store = self.data.read().await;
        Ok(store.is_empty())
    }

    /// Get all entries
    pub async fn entries(&self) -> Result<HashMap<String, StoredData>, String> {
        let store = self.data.read().await;
        Ok(store.clone())
    }

    /// Get the configuration
    pub fn config(&self) -> &DataStoreConfig {
        &self.config
    }

    /// Get the file
    pub fn file(&self) -> Option<&std::path::PathBuf> {
        self.file.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_data_store() {
        let store = DataStore::new();

        // Store data
        store
            .store("key1", serde_json::json!("value1"))
            .await
            .unwrap();
        store.store("key2", serde_json::json!(42)).await.unwrap();

        // Retrieve data
        let value1 = store.retrieve("key1").await.unwrap().unwrap();
        let value2 = store.retrieve("key2").await.unwrap().unwrap();

        assert_eq!(value1.data, serde_json::json!("value1"));
        assert_eq!(value2.data, serde_json::json!(42));

        // Check if keys exist
        assert!(store.exists("key1").await.unwrap());
        assert!(store.exists("key2").await.unwrap());
        assert!(!store.exists("key3").await.unwrap());

        // Get all keys
        let keys = store.keys().await.unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));

        // Remove data
        let removed = store.remove("key1").await.unwrap().unwrap();
        assert_eq!(removed.data, serde_json::json!("value1"));
        assert!(!store.exists("key1").await.unwrap());

        // Clear the store
        store.clear().await.unwrap();
        assert!(store.is_empty().await.unwrap());
    }

    #[tokio::test]
    async fn test_persistent_data_store() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("store.json");
        let store = PersistentDataStore::new(&file_path);

        // Store data
        store
            .store("key1", serde_json::json!("value1"))
            .await
            .unwrap();
        store.store("key2", serde_json::json!(42)).await.unwrap();

        // Create a new store and load the data
        let store2 = PersistentDataStore::new(&file_path);
        store2.load().await.unwrap();

        // Check if the data was loaded
        let value1 = store2.retrieve("key1").await.unwrap().unwrap();
        let value2 = store2.retrieve("key2").await.unwrap().unwrap();

        assert_eq!(value1.data, serde_json::json!("value1"));
        assert_eq!(value2.data, serde_json::json!(42));
    }
}
