//! Data Version Module
//!
//! This module provides functionality for versioning and sharing test data.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::loader::{DataLoader, LoadedData};
use super::store::{DataStore, StoredData};

/// Data version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataVersion {
    /// Version ID
    pub id: String,
    /// Version name
    pub name: String,
    /// Version description
    pub description: Option<String>,
    /// Version tags
    pub tags: Vec<String>,
    /// Version metadata
    pub metadata: HashMap<String, String>,
    /// Version timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Version data
    pub data: serde_json::Value,
    /// Parent version ID
    pub parent_id: Option<String>,
    /// Child version IDs
    pub child_ids: Vec<String>,
}

impl DataVersion {
    /// Create a new data version
    pub fn new(name: impl Into<String>, data: serde_json::Value) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
            timestamp: now,
            data,
            parent_id: None,
            child_ids: Vec::new(),
        }
    }

    /// Create a new data version with a specific ID
    pub fn with_id(
        id: impl Into<String>,
        name: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
            timestamp: now,
            data,
            parent_id: None,
            child_ids: Vec::new(),
        }
    }

    /// Set the version description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for tag in tags {
            self.tags.push(tag.into());
        }
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

    /// Set the parent version ID
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    /// Add a child version ID
    pub fn with_child(mut self, child_id: impl Into<String>) -> Self {
        self.child_ids.push(child_id.into());
        self
    }

    /// Add multiple child version IDs
    pub fn with_children(mut self, child_ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for child_id in child_ids {
            self.child_ids.push(child_id.into());
        }
        self
    }

    /// Create a new version based on this version
    pub fn create_child(&self, name: impl Into<String>, data: serde_json::Value) -> Self {
        let mut child = Self::new(name, data);
        child.parent_id = Some(self.id.clone());

        // Copy relevant metadata
        for (key, value) in &self.metadata {
            if key.starts_with("shared.") {
                child.metadata.insert(key.clone(), value.clone());
            }
        }

        child
    }

    /// Check if this version has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }

    /// Check if this version has a specific metadata key
    pub fn has_metadata(&self, key: &str) -> bool {
        self.metadata.contains_key(key)
    }

    /// Get a metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// Data version repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRepositoryConfig {
    /// Repository name
    pub name: String,
    /// Repository description
    pub description: Option<String>,
    /// Base directory for storing versions
    pub base_dir: Option<PathBuf>,
    /// Maximum number of versions to keep
    pub max_versions: Option<usize>,
    /// Custom configuration
    pub custom: Option<serde_json::Value>,
}

impl Default for VersionRepositoryConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: None,
            base_dir: None,
            max_versions: None,
            custom: None,
        }
    }
}

/// Data version repository
pub struct VersionRepository {
    /// Repository configuration
    config: VersionRepositoryConfig,
    /// Versions
    versions: RwLock<HashMap<String, DataVersion>>,
    /// Data loader
    loader: Option<Arc<dyn DataLoader>>,
}

impl VersionRepository {
    /// Create a new version repository
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            config: VersionRepositoryConfig {
                name: name.into(),
                ..Default::default()
            },
            versions: RwLock::new(HashMap::new()),
            loader: None,
        }
    }

    /// Create a new version repository with a custom configuration
    pub fn with_config(config: VersionRepositoryConfig) -> Self {
        Self {
            config,
            versions: RwLock::new(HashMap::new()),
            loader: None,
        }
    }

    /// Set the data loader
    pub fn with_loader(mut self, loader: Arc<dyn DataLoader>) -> Self {
        self.loader = Some(loader);
        self
    }

    /// Get the repository name
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Get the repository description
    pub fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }

    /// Add a version
    pub async fn add_version(&self, version: DataVersion) -> Result<(), String> {
        let mut versions = self.versions.write().await;

        // Update parent's child list if needed
        if let Some(parent_id) = &version.parent_id {
            if let Some(parent) = versions.get_mut(parent_id) {
                if !parent.child_ids.contains(&version.id) {
                    parent.child_ids.push(version.id.clone());
                }
            }
        }

        // Add the version
        versions.insert(version.id.clone(), version);

        // Check if we need to enforce max versions
        if let Some(max_versions) = self.config.max_versions {
            if versions.len() > max_versions {
                // Remove the oldest versions without children
                let mut version_ages: Vec<_> = versions
                    .iter()
                    .filter(|(_, v)| v.child_ids.is_empty())
                    .map(|(id, v)| (id.clone(), v.timestamp))
                    .collect();

                version_ages.sort_by(|(_, a), (_, b)| a.cmp(b));

                let to_remove = version_ages.len() - max_versions;
                for (id, _) in version_ages.iter().take(to_remove) {
                    versions.remove(id);
                }
            }
        }

        // Save to disk if we have a base directory and loader
        if let (Some(base_dir), Some(loader)) = (&self.config.base_dir, &self.loader) {
            let version_dir = base_dir.join("versions");
            let version_file = version_dir.join(format!("{}.json", version.id));

            // Create the directory if it doesn't exist
            if !version_dir.exists() {
                fs::create_dir_all(&version_dir)
                    .await
                    .map_err(|e| format!("Failed to create version directory: {}", e))?;
            }

            // Save the version
            let loaded_data = LoadedData {
                data: serde_json::to_value(&version).unwrap(),
                metadata: HashMap::new(),
            };

            loader
                .save(&version_file, &loaded_data)
                .await
                .map_err(|e| format!("Failed to save version: {}", e))?;
        }

        Ok(())
    }

    /// Get a version by ID
    pub async fn get_version(&self, id: &str) -> Result<Option<DataVersion>, String> {
        let versions = self.versions.read().await;
        Ok(versions.get(id).cloned())
    }

    /// Get versions by tag
    pub async fn get_versions_by_tag(&self, tag: &str) -> Result<Vec<DataVersion>, String> {
        let versions = self.versions.read().await;
        let matching_versions = versions
            .values()
            .filter(|v| v.has_tag(tag))
            .cloned()
            .collect();
        Ok(matching_versions)
    }

    /// Get versions by metadata
    pub async fn get_versions_by_metadata(
        &self,
        key: &str,
        value: Option<&str>,
    ) -> Result<Vec<DataVersion>, String> {
        let versions = self.versions.read().await;
        let matching_versions = versions
            .values()
            .filter(|v| {
                if let Some(value) = value {
                    v.get_metadata(key) == Some(&value.to_string())
                } else {
                    v.has_metadata(key)
                }
            })
            .cloned()
            .collect();
        Ok(matching_versions)
    }

    /// Get all versions
    pub async fn get_all_versions(&self) -> Result<Vec<DataVersion>, String> {
        let versions = self.versions.read().await;
        let all_versions = versions.values().cloned().collect();
        Ok(all_versions)
    }

    /// Get the version history (ancestors) for a version
    pub async fn get_version_history(&self, id: &str) -> Result<Vec<DataVersion>, String> {
        let versions = self.versions.read().await;
        let mut history = Vec::new();
        let mut current_id = id.to_string();

        while let Some(version) = versions.get(&current_id) {
            history.push(version.clone());
            if let Some(parent_id) = &version.parent_id {
                current_id = parent_id.clone();
            } else {
                break;
            }
        }

        Ok(history)
    }

    /// Get the version descendants for a version
    pub async fn get_version_descendants(&self, id: &str) -> Result<Vec<DataVersion>, String> {
        let versions = self.versions.read().await;
        let mut descendants = Vec::new();
        let mut to_process = vec![id.to_string()];

        while let Some(current_id) = to_process.pop() {
            if let Some(version) = versions.get(&current_id) {
                for child_id in &version.child_ids {
                    if let Some(child) = versions.get(child_id) {
                        descendants.push(child.clone());
                        to_process.push(child_id.clone());
                    }
                }
            }
        }

        Ok(descendants)
    }

    /// Remove a version
    pub async fn remove_version(&self, id: &str) -> Result<Option<DataVersion>, String> {
        let mut versions = self.versions.write().await;

        // Check if the version has children
        if let Some(version) = versions.get(id) {
            if !version.child_ids.is_empty() {
                return Err(format!(
                    "Cannot remove version '{}' because it has children",
                    id
                ));
            }
        }

        // Remove the version
        let removed = versions.remove(id);

        // Update parent's child list if needed
        if let Some(version) = &removed {
            if let Some(parent_id) = &version.parent_id {
                if let Some(parent) = versions.get_mut(parent_id) {
                    parent.child_ids.retain(|child_id| child_id != id);
                }
            }
        }

        // Remove from disk if we have a base directory
        if let (Some(base_dir), Some(_)) = (&self.config.base_dir, &self.loader) {
            let version_file = base_dir.join("versions").join(format!("{}.json", id));
            if version_file.exists() {
                fs::remove_file(&version_file)
                    .await
                    .map_err(|e| format!("Failed to remove version file: {}", e))?;
            }
        }

        Ok(removed)
    }

    /// Clear all versions
    pub async fn clear(&self) -> Result<(), String> {
        let mut versions = self.versions.write().await;
        versions.clear();

        // Remove from disk if we have a base directory
        if let (Some(base_dir), Some(_)) = (&self.config.base_dir, &self.loader) {
            let version_dir = base_dir.join("versions");
            if version_dir.exists() {
                fs::remove_dir_all(&version_dir)
                    .await
                    .map_err(|e| format!("Failed to remove version directory: {}", e))?;
            }
        }

        Ok(())
    }

    /// Load versions from disk
    pub async fn load_from_disk(&self) -> Result<(), String> {
        if let (Some(base_dir), Some(loader)) = (&self.config.base_dir, &self.loader) {
            let version_dir = base_dir.join("versions");
            if !version_dir.exists() {
                return Ok(());
            }

            // Read the directory
            let mut entries = fs::read_dir(&version_dir)
                .await
                .map_err(|e| format!("Failed to read version directory: {}", e))?;

            // Load each version file
            let mut versions = self.versions.write().await;
            versions.clear();

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| format!("Failed to read directory entry: {}", e))?
            {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
                    // Load the version
                    let loaded_data = loader
                        .load(&path)
                        .await
                        .map_err(|e| format!("Failed to load version: {}", e))?;

                    // Parse the version
                    let version: DataVersion = serde_json::from_value(loaded_data.data)
                        .map_err(|e| format!("Failed to parse version: {}", e))?;

                    // Add the version
                    versions.insert(version.id.clone(), version);
                }
            }
        }

        Ok(())
    }

    /// Save all versions to disk
    pub async fn save_to_disk(&self) -> Result<(), String> {
        if let (Some(base_dir), Some(loader)) = (&self.config.base_dir, &self.loader) {
            let version_dir = base_dir.join("versions");

            // Create the directory if it doesn't exist
            if !version_dir.exists() {
                fs::create_dir_all(&version_dir)
                    .await
                    .map_err(|e| format!("Failed to create version directory: {}", e))?;
            }

            // Save each version
            let versions = self.versions.read().await;
            for version in versions.values() {
                let version_file = version_dir.join(format!("{}.json", version.id));

                // Save the version
                let loaded_data = LoadedData {
                    data: serde_json::to_value(version).unwrap(),
                    metadata: HashMap::new(),
                };

                loader
                    .save(&version_file, &loaded_data)
                    .await
                    .map_err(|e| format!("Failed to save version: {}", e))?;
            }
        }

        Ok(())
    }

    /// Share a version with another repository
    pub async fn share_version(
        &self,
        id: &str,
        target_repository: &VersionRepository,
    ) -> Result<(), String> {
        // Get the version
        let version = self
            .get_version(id)
            .await?
            .ok_or_else(|| format!("Version '{}' not found", id))?;

        // Add the version to the target repository
        target_repository.add_version(version).await?;

        Ok(())
    }
}

/// Data version manager
pub struct VersionManager {
    /// Repositories
    repositories: RwLock<HashMap<String, Arc<VersionRepository>>>,
    /// Default repository
    default_repository: RwLock<Option<String>>,
}

impl VersionManager {
    /// Create a new version manager
    pub fn new() -> Self {
        Self {
            repositories: RwLock::new(HashMap::new()),
            default_repository: RwLock::new(None),
        }
    }

    /// Register a repository
    pub async fn register_repository(
        &self,
        name: impl Into<String>,
        repository: Arc<VersionRepository>,
    ) {
        let mut repositories = self.repositories.write().await;
        repositories.insert(name.into(), repository);
    }

    /// Get a repository by name
    pub async fn get_repository(&self, name: &str) -> Option<Arc<VersionRepository>> {
        let repositories = self.repositories.read().await;
        repositories.get(name).cloned()
    }

    /// Set the default repository
    pub async fn set_default_repository(&self, name: impl Into<String>) {
        let mut default = self.default_repository.write().await;
        *default = Some(name.into());
    }

    /// Get the default repository
    pub async fn default_repository(&self) -> Option<Arc<VersionRepository>> {
        let default = self.default_repository.read().await;
        if let Some(name) = &*default {
            let repositories = self.repositories.read().await;
            repositories.get(name).cloned()
        } else {
            None
        }
    }

    /// Get all repository names
    pub async fn repository_names(&self) -> Vec<String> {
        let repositories = self.repositories.read().await;
        repositories.keys().cloned().collect()
    }

    /// Add a version to a repository
    pub async fn add_version(
        &self,
        repository_name: &str,
        version: DataVersion,
    ) -> Result<(), String> {
        let repository = self
            .get_repository(repository_name)
            .await
            .ok_or_else(|| format!("Repository '{}' not found", repository_name))?;

        repository.add_version(version).await
    }

    /// Get a version from a repository
    pub async fn get_version(
        &self,
        repository_name: &str,
        id: &str,
    ) -> Result<Option<DataVersion>, String> {
        let repository = self
            .get_repository(repository_name)
            .await
            .ok_or_else(|| format!("Repository '{}' not found", repository_name))?;

        repository.get_version(id).await
    }

    /// Share a version between repositories
    pub async fn share_version(
        &self,
        source_repository: &str,
        target_repository: &str,
        id: &str,
    ) -> Result<(), String> {
        let source = self
            .get_repository(source_repository)
            .await
            .ok_or_else(|| format!("Source repository '{}' not found", source_repository))?;

        let target = self
            .get_repository(target_repository)
            .await
            .ok_or_else(|| format!("Target repository '{}' not found", target_repository))?;

        source.share_version(id, &target).await
    }
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::loader::JsonDataLoader;
    use super::*;

    #[tokio::test]
    async fn test_version_repository() {
        // Create a repository
        let repository = VersionRepository::new("test-repo");

        // Create versions
        let v1 = DataVersion::new("v1", serde_json::json!({"data": "version 1"}));
        let v1_id = v1.id.clone();

        let v2 = DataVersion::new("v2", serde_json::json!({"data": "version 2"}))
            .with_parent(v1_id.clone());
        let v2_id = v2.id.clone();

        // Add versions
        repository.add_version(v1).await.unwrap();
        repository.add_version(v2).await.unwrap();

        // Get versions
        let v1 = repository.get_version(&v1_id).await.unwrap().unwrap();
        let v2 = repository.get_version(&v2_id).await.unwrap().unwrap();

        // Check relationships
        assert_eq!(v1.child_ids, vec![v2_id.clone()]);
        assert_eq!(v2.parent_id, Some(v1_id.clone()));

        // Get history
        let history = repository.get_version_history(&v2_id).await.unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].id, v2_id);
        assert_eq!(history[1].id, v1_id);

        // Get descendants
        let descendants = repository.get_version_descendants(&v1_id).await.unwrap();
        assert_eq!(descendants.len(), 1);
        assert_eq!(descendants[0].id, v2_id);
    }
}
