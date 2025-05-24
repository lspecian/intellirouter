//! Test Data Management Framework
//!
//! This module provides functionality for managing test data, including loading,
//! generating, validating, and cleaning up test data.

mod cleanup;
mod factory;
mod generator;
mod isolation;
mod loader;
mod schema;
mod store;
mod validator;
mod version;

pub use cleanup::{
    CleanupConfig, CleanupHandler, CleanupManager, CleanupResource, CleanupScope, CleanupStrategy,
    CleanupTask, DataStoreCleanupHandler, FileCleanupHandler, IsolationCleanupHandler,
};
pub use factory::{
    DataFactory, DataFactoryConfig, FactoryData, ScenarioDataFactory, StandardDataFactory,
};
pub use generator::{DataGenerator, DataGeneratorConfig, GeneratedData};
pub use isolation::{IsolationContext, IsolationLevel, IsolationManager, IsolationManagerConfig};
pub use loader::{DataLoader, DataLoaderConfig, LoadedData};
pub use schema::{DataSchema, DataType, FieldDefinition, SchemaValidationError};
pub use store::{DataStore, DataStoreConfig, StoredData};
pub use validator::{DataValidator, ValidationError, ValidationResult};
pub use version::{DataVersion, VersionManager, VersionRepository, VersionRepositoryConfig};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::environment::Environment;
use super::types::TestHarnessError;

/// Test data manager for managing test data
pub struct TestDataManager {
    /// Base directory for test data
    base_dir: PathBuf,
    /// Data loaders
    loaders: HashMap<String, Arc<dyn DataLoader>>,
    /// Data generators
    generators: HashMap<String, Arc<dyn DataGenerator>>,
    /// Data validators
    validators: HashMap<String, Arc<dyn DataValidator>>,
    /// Data factories
    factories: HashMap<String, Arc<dyn DataFactory>>,
    /// Data store
    store: Arc<DataStore>,
    /// Version manager
    version_manager: Arc<VersionManager>,
    /// Isolation manager
    isolation_manager: Arc<IsolationManager>,
    /// Cleanup manager
    cleanup_manager: Arc<CleanupManager>,
    /// Environment
    environment: Option<Arc<dyn Environment>>,
}

impl TestDataManager {
    /// Create a new test data manager
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        let base_dir = base_dir.into();
        let store = Arc::new(DataStore::new());

        // Create managers
        let version_manager = Arc::new(VersionManager::new());
        let isolation_manager = Arc::new(IsolationManager::with_data_store(store.clone()));
        let cleanup_manager = Arc::new(CleanupManager::new());

        Self {
            base_dir,
            loaders: HashMap::new(),
            generators: HashMap::new(),
            validators: HashMap::new(),
            factories: HashMap::new(),
            store,
            version_manager,
            isolation_manager,
            cleanup_manager,
            environment: None,
        }
    }

    /// Create a new test data manager with an environment
    pub fn with_environment(
        base_dir: impl Into<PathBuf>,
        environment: Arc<dyn Environment>,
    ) -> Self {
        let base_dir = base_dir.into();
        let store = Arc::new(DataStore::new());

        // Create managers
        let version_manager = Arc::new(VersionManager::new());
        let isolation_manager = Arc::new(IsolationManager::with_data_store(store.clone()));
        let cleanup_manager = Arc::new(CleanupManager::new());

        Self {
            base_dir,
            loaders: HashMap::new(),
            generators: HashMap::new(),
            validators: HashMap::new(),
            factories: HashMap::new(),
            store,
            version_manager,
            isolation_manager,
            cleanup_manager,
            environment: Some(environment),
        }
    }

    /// Register a data loader
    pub fn register_loader(&mut self, name: impl Into<String>, loader: Arc<dyn DataLoader>) {
        self.loaders.insert(name.into(), loader);
    }

    /// Register a data generator
    pub fn register_generator(
        &mut self,
        name: impl Into<String>,
        generator: Arc<dyn DataGenerator>,
    ) {
        self.generators.insert(name.into(), generator);
    }

    /// Register a data validator
    pub fn register_validator(
        &mut self,
        name: impl Into<String>,
        validator: Arc<dyn DataValidator>,
    ) {
        self.validators.insert(name.into(), validator);
    }

    /// Set the environment
    pub fn set_environment(&mut self, environment: Arc<dyn Environment>) {
        self.environment = Some(environment);
    }

    /// Get the environment
    pub fn environment(&self) -> Option<Arc<dyn Environment>> {
        self.environment.clone()
    }

    /// Get the version manager
    pub fn version_manager(&self) -> Arc<VersionManager> {
        self.version_manager.clone()
    }

    /// Get the isolation manager
    pub fn isolation_manager(&self) -> Arc<IsolationManager> {
        self.isolation_manager.clone()
    }

    /// Get the cleanup manager
    pub fn cleanup_manager(&self) -> Arc<CleanupManager> {
        self.cleanup_manager.clone()
    }

    /// Load data from a file
    pub async fn load_data<T: DeserializeOwned>(
        &self,
        path: impl AsRef<Path>,
        loader_name: Option<&str>,
    ) -> Result<T, TestHarnessError> {
        let path = path.as_ref();
        let full_path = self.base_dir.join(path);

        // Determine the loader to use
        let loader_name = if let Some(name) = loader_name {
            name.to_string()
        } else {
            // Use the file extension as the loader name
            path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("json")
                .to_string()
        };

        // Get the loader
        let loader = self.loaders.get(&loader_name).ok_or_else(|| {
            TestHarnessError::DataError(format!("Loader '{}' not found", loader_name))
        })?;

        // Load the data
        let data = loader
            .load(&full_path)
            .await
            .map_err(|e| TestHarnessError::DataError(format!("Failed to load data: {}", e)))?;

        // Deserialize the data
        let typed_data = serde_json::from_value(data.data).map_err(|e| {
            TestHarnessError::SerializationError(format!("Failed to deserialize data: {}", e))
        })?;

        Ok(typed_data)
    }

    /// Generate data
    pub async fn generate_data<T: DeserializeOwned>(
        &self,
        generator_name: &str,
        config: &serde_json::Value,
    ) -> Result<T, TestHarnessError> {
        // Get the generator
        let generator = self.generators.get(generator_name).ok_or_else(|| {
            TestHarnessError::DataError(format!("Generator '{}' not found", generator_name))
        })?;

        // Generate the data
        let data = generator
            .generate(config)
            .await
            .map_err(|e| TestHarnessError::DataError(format!("Failed to generate data: {}", e)))?;

        // Deserialize the data
        let typed_data = serde_json::from_value(data.data).map_err(|e| {
            TestHarnessError::SerializationError(format!("Failed to deserialize data: {}", e))
        })?;

        Ok(typed_data)
    }

    /// Validate data
    pub async fn validate_data<T: Serialize>(
        &self,
        validator_name: &str,
        data: &T,
        schema: Option<&DataSchema>,
    ) -> Result<ValidationResult, TestHarnessError> {
        // Get the validator
        let validator = self.validators.get(validator_name).ok_or_else(|| {
            TestHarnessError::DataError(format!("Validator '{}' not found", validator_name))
        })?;

        // Serialize the data
        let data_value = serde_json::to_value(data).map_err(|e| {
            TestHarnessError::SerializationError(format!("Failed to serialize data: {}", e))
        })?;

        // Validate the data
        let result = validator
            .validate(&data_value, schema)
            .await
            .map_err(|e| TestHarnessError::DataError(format!("Failed to validate data: {}", e)))?;

        Ok(result)
    }

    /// Store data
    pub async fn store_data<T: Serialize>(
        &self,
        key: impl Into<String>,
        data: &T,
    ) -> Result<(), TestHarnessError> {
        // Serialize the data
        let data_value = serde_json::to_value(data).map_err(|e| {
            TestHarnessError::SerializationError(format!("Failed to serialize data: {}", e))
        })?;

        // Store the data
        self.store
            .store(key.into(), data_value)
            .await
            .map_err(|e| TestHarnessError::DataError(format!("Failed to store data: {}", e)))?;

        Ok(())
    }

    /// Retrieve data
    pub async fn retrieve_data<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, TestHarnessError> {
        // Retrieve the data
        let data =
            self.store.retrieve(key).await.map_err(|e| {
                TestHarnessError::DataError(format!("Failed to retrieve data: {}", e))
            })?;

        // Deserialize the data if it exists
        if let Some(data) = data {
            let typed_data = serde_json::from_value(data.data).map_err(|e| {
                TestHarnessError::SerializationError(format!("Failed to deserialize data: {}", e))
            })?;
            Ok(Some(typed_data))
        } else {
            Ok(None)
        }
    }

    /// Save data to a file
    pub async fn save_data<T: Serialize>(
        &self,
        path: impl AsRef<Path>,
        data: &T,
        loader_name: Option<&str>,
    ) -> Result<(), TestHarnessError> {
        let path = path.as_ref();
        let full_path = self.base_dir.join(path);

        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| TestHarnessError::IoError(e))?;
        }

        // Determine the loader to use
        let loader_name = if let Some(name) = loader_name {
            name.to_string()
        } else {
            // Use the file extension as the loader name
            path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("json")
                .to_string()
        };

        // Get the loader
        let loader = self.loaders.get(&loader_name).ok_or_else(|| {
            TestHarnessError::DataError(format!("Loader '{}' not found", loader_name))
        })?;

        // Serialize the data
        let data_value = serde_json::to_value(data).map_err(|e| {
            TestHarnessError::SerializationError(format!("Failed to serialize data: {}", e))
        })?;

        // Create the loaded data
        let loaded_data = LoadedData {
            data: data_value,
            metadata: HashMap::new(),
        };

        // Save the data
        loader
            .save(&full_path, &loaded_data)
            .await
            .map_err(|e| TestHarnessError::DataError(format!("Failed to save data: {}", e)))?;

        Ok(())
    }

    /// Clear the data store
    pub async fn clear_store(&self) -> Result<(), TestHarnessError> {
        self.store.clear().await.map_err(|e| {
            TestHarnessError::DataError(format!("Failed to clear data store: {}", e))
        })?;
        Ok(())
    }

    /// List available data files
    pub fn list_data_files(
        &self,
        extension: Option<&str>,
    ) -> Result<Vec<PathBuf>, TestHarnessError> {
        let mut files = Vec::new();
        self.list_files_recursive(&self.base_dir, &mut files, extension)?;
        Ok(files)
    }

    /// List files recursively
    fn list_files_recursive(
        &self,
        dir: &Path,
        files: &mut Vec<PathBuf>,
        extension: Option<&str>,
    ) -> Result<(), TestHarnessError> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir).map_err(TestHarnessError::IoError)? {
            let entry = entry.map_err(TestHarnessError::IoError)?;
            let path = entry.path();

            if path.is_dir() {
                self.list_files_recursive(&path, files, extension)?;
            } else if let Some(ext) = extension {
                if path.extension().and_then(|e| e.to_str()) == Some(ext) {
                    files.push(path);
                }
            } else {
                files.push(path);
            }
        }

        Ok(())
    }

    /// Generate data using a factory
    pub async fn generate_factory_data<T: DeserializeOwned>(
        &self,
        factory_name: &str,
        config: &serde_json::Value,
    ) -> Result<T, TestHarnessError> {
        // Get the factory
        let factory = self.factories.get(factory_name).ok_or_else(|| {
            TestHarnessError::DataError(format!("Factory '{}' not found", factory_name))
        })?;

        // Generate the data
        let data = factory
            .generate(config)
            .await
            .map_err(|e| TestHarnessError::DataError(format!("Failed to generate data: {}", e)))?;

        // Deserialize the data
        let typed_data = serde_json::from_value(data.data).map_err(|e| {
            TestHarnessError::SerializationError(format!("Failed to deserialize data: {}", e))
        })?;

        Ok(typed_data)
    }

    /// Create a data version
    pub async fn create_version(
        &self,
        name: impl Into<String>,
        data: &serde_json::Value,
        repository_name: Option<&str>,
    ) -> Result<DataVersion, TestHarnessError> {
        let version = DataVersion::new(name, data.clone());

        // Add the version to the repository
        let repository_name = repository_name.unwrap_or("default");
        let repository = self
            .version_manager
            .get_repository(repository_name)
            .await
            .ok_or_else(|| {
                TestHarnessError::DataError(format!("Repository '{}' not found", repository_name))
            })?;

        repository
            .add_version(version.clone())
            .await
            .map_err(|e| TestHarnessError::DataError(format!("Failed to add version: {}", e)))?;

        Ok(version)
    }

    /// Create an isolation context
    pub async fn create_isolation_context(
        &self,
        name: impl Into<String>,
        context_type: impl Into<String>,
        isolation_level: Option<IsolationLevel>,
    ) -> Result<IsolationContext, TestHarnessError> {
        self.isolation_manager
            .create_context(name, context_type, isolation_level)
            .await
            .map_err(|e| TestHarnessError::DataError(format!("Failed to create context: {}", e)))
    }

    /// Push an isolation context
    pub async fn push_isolation_context(&self, id: &str) -> Result<(), TestHarnessError> {
        self.isolation_manager
            .push_context(id)
            .await
            .map_err(|e| TestHarnessError::DataError(format!("Failed to push context: {}", e)))
    }

    /// Pop an isolation context
    pub async fn pop_isolation_context(
        &self,
    ) -> Result<Option<IsolationContext>, TestHarnessError> {
        self.isolation_manager
            .pop_context()
            .await
            .map_err(|e| TestHarnessError::DataError(format!("Failed to pop context: {}", e)))
    }

    /// Store data in the current isolation context
    pub async fn store_isolated_data(
        &self,
        key: &str,
        data: &serde_json::Value,
    ) -> Result<(), TestHarnessError> {
        self.isolation_manager
            .store_data(key, data.clone())
            .await
            .map_err(|e| TestHarnessError::DataError(format!("Failed to store data: {}", e)))
    }

    /// Retrieve data from the current isolation context
    pub async fn retrieve_isolated_data<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, TestHarnessError> {
        let data = self
            .isolation_manager
            .retrieve_data(key)
            .await
            .map_err(|e| TestHarnessError::DataError(format!("Failed to retrieve data: {}", e)))?;

        if let Some(data) = data {
            let typed_data = serde_json::from_value(data.data).map_err(|e| {
                TestHarnessError::SerializationError(format!("Failed to deserialize data: {}", e))
            })?;
            Ok(Some(typed_data))
        } else {
            Ok(None)
        }
    }

    /// Create a cleanup task
    pub async fn create_cleanup_task(
        &self,
        name: impl Into<String>,
        strategy: Option<CleanupStrategy>,
        scope: Option<CleanupScope>,
        context_id: Option<String>,
    ) -> Result<CleanupTask, TestHarnessError> {
        self.cleanup_manager
            .create_task(name, strategy, scope, context_id)
            .await
            .map_err(|e| {
                TestHarnessError::DataError(format!("Failed to create cleanup task: {}", e))
            })
    }

    /// Add a resource to a cleanup task
    pub async fn add_cleanup_resource(
        &self,
        task_id: &str,
        resource_type: impl Into<String>,
        identifier: impl Into<String>,
    ) -> Result<(), TestHarnessError> {
        self.cleanup_manager
            .add_resource(task_id, resource_type, identifier)
            .await
            .map_err(|e| {
                TestHarnessError::DataError(format!("Failed to add cleanup resource: {}", e))
            })
    }

    /// Execute a cleanup task
    pub async fn execute_cleanup_task(&self, id: &str) -> Result<(), TestHarnessError> {
        self.cleanup_manager.execute_task(id).await.map_err(|e| {
            TestHarnessError::DataError(format!("Failed to execute cleanup task: {}", e))
        })
    }

    /// Execute all immediate cleanup tasks
    pub async fn execute_immediate_cleanup_tasks(&self) -> Result<(), TestHarnessError> {
        self.cleanup_manager
            .execute_immediate_tasks()
            .await
            .map_err(|e| {
                TestHarnessError::DataError(format!(
                    "Failed to execute immediate cleanup tasks: {}",
                    e
                ))
            })
    }

    /// Execute all deferred cleanup tasks
    pub async fn execute_deferred_cleanup_tasks(&self) -> Result<(), TestHarnessError> {
        self.cleanup_manager
            .execute_deferred_tasks()
            .await
            .map_err(|e| {
                TestHarnessError::DataError(format!(
                    "Failed to execute deferred cleanup tasks: {}",
                    e
                ))
            })
    }
}

impl Default for TestDataManager {
    fn default() -> Self {
        Self::new("test_data")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_test_data_manager() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = TestDataManager::new(temp_dir.path());

        // Register a JSON loader
        let json_loader = Arc::new(super::loader::JsonDataLoader::new());
        manager.register_loader("json", json_loader.clone());

        // Register a faker generator
        let faker_generator = Arc::new(super::generator::FakerDataGenerator::new());
        manager.register_generator("faker", faker_generator.clone());

        // Register a schema validator
        let schema_validator = Arc::new(super::validator::SchemaValidator::new());
        manager.register_validator("schema", schema_validator.clone());

        // Create a schema
        let schema = DataSchema::new(
            "test",
            DataType::Object(
                [
                    (
                        "name".to_string(),
                        FieldDefinition {
                            data_type: DataType::String,
                            required: true,
                            ..Default::default()
                        },
                    ),
                    (
                        "value".to_string(),
                        FieldDefinition {
                            data_type: DataType::Integer,
                            required: true,
                            ..Default::default()
                        },
                    ),
                ]
                .into_iter()
                .collect(),
            ),
        );

        // Test data generation
        let config = serde_json::json!({
            "schema": {
                "name": "test",
                "root_type": {
                    "Object": {
                        "name": {
                            "data_type": { "String": null },
                            "required": true,
                            "description": null,
                            "default": null,
                            "minimum": null,
                            "maximum": null,
                            "min_length": null,
                            "max_length": null,
                            "pattern": null,
                            "format": null,
                            "additional_properties": true
                        },
                        "value": {
                            "data_type": { "Integer": null },
                            "required": true,
                            "description": null,
                            "default": null,
                            "minimum": null,
                            "maximum": null,
                            "min_length": null,
                            "max_length": null,
                            "pattern": null,
                            "format": null,
                            "additional_properties": true
                        }
                    }
                }
            }
        });

        let data: serde_json::Value = manager.generate_data("faker", &config).await.unwrap();
        assert!(data.is_object());
        assert!(data.get("name").is_some());
        assert!(data.get("value").is_some());

        // Test data validation
        let validation_result = manager
            .validate_data("schema", &data, Some(&schema))
            .await
            .unwrap();
        assert!(validation_result.passed);

        // Test data storage and retrieval
        manager.store_data("test-key", &data).await.unwrap();
        let retrieved: serde_json::Value =
            manager.retrieve_data("test-key").await.unwrap().unwrap();
        assert_eq!(retrieved, data);

        // Test isolation context
        let context = manager
            .create_isolation_context("test-context", "test", Some(IsolationLevel::TestRun))
            .await
            .unwrap();

        manager.push_isolation_context(&context.id).await.unwrap();

        manager
            .store_isolated_data("isolated-key", &data)
            .await
            .unwrap();

        let isolated_data: serde_json::Value = manager
            .retrieve_isolated_data("isolated-key")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(isolated_data, data);

        // Test cleanup
        let test_file = temp_dir.path().join("test.txt");
        tokio::fs::write(&test_file, b"test data").await.unwrap();

        // Register a file cleanup handler
        let file_handler = Arc::new(FileCleanupHandler::with_base_dir(temp_dir.path()));
        manager
            .cleanup_manager()
            .register_handler("file", file_handler)
            .await;

        let cleanup_task = manager
            .create_cleanup_task("test-cleanup", Some(CleanupStrategy::Immediate), None, None)
            .await
            .unwrap();

        manager
            .add_cleanup_resource(&cleanup_task.id, "file", "test.txt")
            .await
            .unwrap();

        manager
            .execute_cleanup_task(&cleanup_task.id)
            .await
            .unwrap();

        assert!(!test_file.exists());
    }
}
