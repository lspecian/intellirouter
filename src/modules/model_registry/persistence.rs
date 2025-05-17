//! Model Registry Persistence
//!
//! This module implements persistence for the Model Registry, allowing
//! model metadata to be saved to and loaded from persistent storage.
//! It provides a trait for different storage backends and a file-based
//! implementation.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::storage::ModelRegistry;
use super::types::{ModelMetadata, RegistryError};

/// Persistence configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    /// Whether persistence is enabled
    pub enabled: bool,
    /// Path to the persistence directory
    pub directory: PathBuf,
    /// Auto-save interval in seconds (0 = manual save only)
    pub auto_save_interval_seconds: u64,
    /// Maximum number of backup files to keep
    pub max_backups: usize,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            directory: PathBuf::from("data/model_registry"),
            auto_save_interval_seconds: 300, // 5 minutes
            max_backups: 3,
        }
    }
}

/// Persistence trait for model registry
pub trait ModelRegistryPersistence: Send + Sync {
    /// Save the registry to persistent storage
    fn save(&self, registry: &ModelRegistry) -> Result<(), RegistryError>;

    /// Load the registry from persistent storage
    fn load(&self) -> Result<ModelRegistry, RegistryError>;

    /// Check if a saved registry exists
    fn exists(&self) -> bool;

    /// Clear the persistent storage
    fn clear(&self) -> Result<(), RegistryError>;
}

/// File-based persistence implementation
pub struct FileModelRegistryPersistence {
    /// Persistence configuration
    config: PersistenceConfig,
}

impl FileModelRegistryPersistence {
    /// Create a new file-based persistence
    pub fn new(config: PersistenceConfig) -> Self {
        Self { config }
    }

    /// Get the path to the registry file
    fn registry_path(&self) -> PathBuf {
        self.config.directory.join("registry.json")
    }

    /// Get the path to a backup file
    fn backup_path(&self, index: usize) -> PathBuf {
        self.config
            .directory
            .join(format!("registry.backup.{}.json", index))
    }

    /// Create the persistence directory if it doesn't exist
    fn ensure_directory_exists(&self) -> Result<(), RegistryError> {
        if !self.config.directory.exists() {
            fs::create_dir_all(&self.config.directory).map_err(|e| {
                RegistryError::StorageError(format!(
                    "Failed to create persistence directory: {}",
                    e
                ))
            })?;
        }
        Ok(())
    }

    /// Rotate backup files
    fn rotate_backups(&self) -> Result<(), RegistryError> {
        if self.config.max_backups == 0 {
            return Ok(());
        }

        // Remove the oldest backup
        let oldest_backup = self.backup_path(self.config.max_backups - 1);
        if oldest_backup.exists() {
            fs::remove_file(&oldest_backup).map_err(|e| {
                RegistryError::StorageError(format!("Failed to remove oldest backup: {}", e))
            })?;
        }

        // Shift other backups
        for i in (0..self.config.max_backups - 1).rev() {
            let current = self.backup_path(i);
            let next = self.backup_path(i + 1);
            if current.exists() {
                fs::rename(&current, &next).map_err(|e| {
                    RegistryError::StorageError(format!("Failed to rotate backup: {}", e))
                })?;
            }
        }

        // Move the current registry to the first backup
        let registry_path = self.registry_path();
        if registry_path.exists() {
            let first_backup = self.backup_path(0);
            fs::copy(&registry_path, &first_backup).map_err(|e| {
                RegistryError::StorageError(format!("Failed to create backup: {}", e))
            })?;
        }

        Ok(())
    }
}

impl ModelRegistryPersistence for FileModelRegistryPersistence {
    fn save(&self, registry: &ModelRegistry) -> Result<(), RegistryError> {
        if !self.config.enabled {
            return Ok(());
        }

        self.ensure_directory_exists()?;
        self.rotate_backups()?;

        // Get all models from the registry
        let models = registry.list_models();

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&models).map_err(|e| {
            RegistryError::StorageError(format!("Failed to serialize registry: {}", e))
        })?;

        // Write to file
        fs::write(self.registry_path(), json).map_err(|e| {
            RegistryError::StorageError(format!("Failed to write registry to file: {}", e))
        })?;

        info!("Registry saved to {}", self.registry_path().display());
        Ok(())
    }

    fn load(&self) -> Result<ModelRegistry, RegistryError> {
        if !self.config.enabled {
            return Ok(ModelRegistry::new());
        }

        let registry_path = self.registry_path();
        if !registry_path.exists() {
            return Ok(ModelRegistry::new());
        }

        // Read from file
        let json = fs::read_to_string(&registry_path).map_err(|e| {
            RegistryError::StorageError(format!("Failed to read registry from file: {}", e))
        })?;

        // Deserialize from JSON
        let models: Vec<ModelMetadata> = serde_json::from_str(&json).map_err(|e| {
            RegistryError::StorageError(format!("Failed to deserialize registry: {}", e))
        })?;

        // Create a new registry and add all models
        let registry = ModelRegistry::new();
        for model in models {
            registry.register_model(model).map_err(|e| {
                RegistryError::StorageError(format!("Failed to register model during load: {}", e))
            })?;
        }

        info!("Registry loaded from {}", registry_path.display());
        Ok(registry)
    }

    fn exists(&self) -> bool {
        if !self.config.enabled {
            return false;
        }

        self.registry_path().exists()
    }

    fn clear(&self) -> Result<(), RegistryError> {
        if !self.config.enabled {
            return Ok(());
        }

        let registry_path = self.registry_path();
        if registry_path.exists() {
            fs::remove_file(&registry_path).map_err(|e| {
                RegistryError::StorageError(format!("Failed to clear registry: {}", e))
            })?;
        }

        // Clear backups
        for i in 0..self.config.max_backups {
            let backup_path = self.backup_path(i);
            if backup_path.exists() {
                fs::remove_file(&backup_path).map_err(|e| {
                    RegistryError::StorageError(format!("Failed to clear backup: {}", e))
                })?;
            }
        }

        info!("Registry cleared");
        Ok(())
    }
}

/// Persistent model registry
pub struct PersistentModelRegistry {
    /// In-memory registry
    registry: Arc<ModelRegistry>,
    /// Persistence implementation
    persistence: Arc<dyn ModelRegistryPersistence>,
    /// Auto-save task handle
    #[allow(dead_code)]
    auto_save_task: Option<tokio::task::JoinHandle<()>>,
}

impl PersistentModelRegistry {
    /// Create a new persistent model registry
    pub fn new(persistence: Arc<dyn ModelRegistryPersistence>) -> Result<Self, RegistryError> {
        // Load the registry from persistence
        let registry = Arc::new(persistence.load()?);

        Ok(Self {
            registry,
            persistence,
            auto_save_task: None,
        })
    }

    /// Create a new persistent model registry with auto-save
    pub fn with_auto_save(
        persistence: Arc<dyn ModelRegistryPersistence>,
        interval_seconds: u64,
    ) -> Result<Self, RegistryError> {
        // Load the registry from persistence
        let registry = Arc::new(persistence.load()?);

        // Create the registry
        let mut persistent_registry = Self {
            registry: registry.clone(),
            persistence: persistence.clone(),
            auto_save_task: None,
        };

        // Start auto-save task if interval is non-zero
        if interval_seconds > 0 {
            persistent_registry.start_auto_save(interval_seconds);
        }

        Ok(persistent_registry)
    }

    /// Start auto-save task
    fn start_auto_save(&mut self, interval_seconds: u64) {
        let registry = self.registry.clone();
        let persistence = self.persistence.clone();

        self.auto_save_task = Some(tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(interval_seconds));

            loop {
                interval.tick().await;
                debug!("Auto-saving registry");

                if let Err(e) = persistence.save(&registry) {
                    error!("Failed to auto-save registry: {}", e);
                }
            }
        }));
    }

    /// Stop auto-save task
    pub fn stop_auto_save(&mut self) {
        if let Some(task) = self.auto_save_task.take() {
            task.abort();
            debug!("Auto-save task stopped");
        }
    }

    /// Save the registry to persistence
    pub fn save(&self) -> Result<(), RegistryError> {
        self.persistence.save(&self.registry)
    }

    /// Get the in-memory registry
    pub fn registry(&self) -> Arc<ModelRegistry> {
        self.registry.clone()
    }
}

impl Drop for PersistentModelRegistry {
    fn drop(&mut self) {
        self.stop_auto_save();

        // Try to save on drop
        if let Err(e) = self.save() {
            error!("Failed to save registry on drop: {}", e);
        }
    }
}

/// Create a file-based persistent model registry
pub fn create_file_persistent_registry(
    config: PersistenceConfig,
) -> Result<PersistentModelRegistry, RegistryError> {
    let persistence = Arc::new(FileModelRegistryPersistence::new(config.clone()));

    if config.auto_save_interval_seconds > 0 {
        PersistentModelRegistry::with_auto_save(persistence, config.auto_save_interval_seconds)
    } else {
        PersistentModelRegistry::new(persistence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::model_registry::types::{ModelMetadata, ModelStatus};
    use tempfile::tempdir;

    /// Helper function to create a test model
    fn create_test_model(id: &str, provider: &str) -> ModelMetadata {
        ModelMetadata::new(
            id.to_string(),
            format!("{} Model", id),
            provider.to_string(),
            "1.0".to_string(),
            format!("https://api.{}.com/v1", provider),
        )
    }

    #[test]
    fn test_file_persistence_save_load() {
        // Create a temporary directory for the test
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // Create a persistence config
        let config = PersistenceConfig {
            enabled: true,
            directory: temp_path.clone(),
            auto_save_interval_seconds: 0,
            max_backups: 1,
        };

        // Create a persistence implementation
        let persistence = FileModelRegistryPersistence::new(config);

        // Create a registry and add some models
        let registry = ModelRegistry::new();
        registry
            .register_model(create_test_model("model-1", "openai"))
            .unwrap();
        registry
            .register_model(create_test_model("model-2", "anthropic"))
            .unwrap();

        // Save the registry
        persistence.save(&registry).unwrap();

        // Verify the registry file exists
        let registry_path = temp_path.join("registry.json");
        assert!(registry_path.exists());

        // Load the registry
        let loaded_registry = persistence.load().unwrap();

        // Verify the loaded registry has the same models
        assert_eq!(loaded_registry.count(), 2);
        assert!(loaded_registry.get_model("model-1").is_ok());
        assert!(loaded_registry.get_model("model-2").is_ok());
    }

    #[test]
    fn test_file_persistence_backup_rotation() {
        // Create a temporary directory for the test
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // Create a persistence config with 3 backups
        let config = PersistenceConfig {
            enabled: true,
            directory: temp_path.clone(),
            auto_save_interval_seconds: 0,
            max_backups: 3,
        };

        // Create a persistence implementation
        let persistence = FileModelRegistryPersistence::new(config);

        // Create a registry and add a model
        let registry = ModelRegistry::new();
        registry
            .register_model(create_test_model("model-1", "openai"))
            .unwrap();

        // Save the registry multiple times to create backups
        for _ in 0..5 {
            persistence.save(&registry).unwrap();
        }

        // Verify the registry file and backups exist
        let registry_path = temp_path.join("registry.json");
        assert!(registry_path.exists());

        let backup0_path = temp_path.join("registry.backup.0.json");
        assert!(backup0_path.exists());

        let backup1_path = temp_path.join("registry.backup.1.json");
        assert!(backup1_path.exists());

        let backup2_path = temp_path.join("registry.backup.2.json");
        assert!(backup2_path.exists());

        // Verify older backups are rotated out
        let backup3_path = temp_path.join("registry.backup.3.json");
        assert!(!backup3_path.exists());
    }

    #[tokio::test]
    async fn test_persistent_registry() {
        // Create a temporary directory for the test
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // Create a persistence config
        let config = PersistenceConfig {
            enabled: true,
            directory: temp_path,
            auto_save_interval_seconds: 0,
            max_backups: 1,
        };

        // Create a persistent registry
        let persistent_registry = create_file_persistent_registry(config).unwrap();
        let registry = persistent_registry.registry();

        // Add some models
        registry
            .register_model(create_test_model("model-1", "openai"))
            .unwrap();
        registry
            .register_model(create_test_model("model-2", "anthropic"))
            .unwrap();

        // Save the registry
        persistent_registry.save().unwrap();

        // Create a new persistent registry (simulating a restart)
        let new_persistent_registry = create_file_persistent_registry(PersistenceConfig {
            enabled: true,
            directory: temp_dir.path().to_path_buf(),
            auto_save_interval_seconds: 0,
            max_backups: 1,
        })
        .unwrap();

        let new_registry = new_persistent_registry.registry();

        // Verify the models were loaded
        assert_eq!(new_registry.count(), 2);
        assert!(new_registry.get_model("model-1").is_ok());
        assert!(new_registry.get_model("model-2").is_ok());
    }

    #[tokio::test]
    async fn test_auto_save() {
        // Create a temporary directory for the test
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // Create a persistence config with auto-save
        let config = PersistenceConfig {
            enabled: true,
            directory: temp_path.clone(),
            auto_save_interval_seconds: 1, // 1 second interval for testing
            max_backups: 1,
        };

        // Create a persistent registry with auto-save
        let persistent_registry = create_file_persistent_registry(config).unwrap();
        let registry = persistent_registry.registry();

        // Add a model
        registry
            .register_model(create_test_model("model-1", "openai"))
            .unwrap();

        // Wait for auto-save to happen
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Verify the registry file exists
        let registry_path = temp_path.join("registry.json");
        assert!(registry_path.exists());

        // Add another model
        registry
            .register_model(create_test_model("model-2", "anthropic"))
            .unwrap();

        // Wait for auto-save to happen again
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Create a new persistent registry (simulating a restart)
        let new_persistent_registry = create_file_persistent_registry(PersistenceConfig {
            enabled: true,
            directory: temp_dir.path().to_path_buf(),
            auto_save_interval_seconds: 0,
            max_backups: 1,
        })
        .unwrap();

        let new_registry = new_persistent_registry.registry();

        // Verify both models were loaded
        assert_eq!(new_registry.count(), 2);
        assert!(new_registry.get_model("model-1").is_ok());
        assert!(new_registry.get_model("model-2").is_ok());
    }
}
