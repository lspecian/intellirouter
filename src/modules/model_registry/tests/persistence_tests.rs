//! Persistence Tests for Model Registry
//!
//! This module contains tests to verify the persistence layer of the Model Registry.
//! It includes tests for saving, loading, backup rotation, and auto-save functionality.

use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

use crate::modules::model_registry::persistence::{
    create_file_persistent_registry, FileModelRegistryPersistence, ModelRegistryPersistence,
    PersistenceConfig, PersistentModelRegistry,
};
use crate::modules::model_registry::storage::ModelRegistry;
use crate::modules::model_registry::types::{ModelMetadata, ModelStatus, ModelType};

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

#[test]
fn test_persistence_disabled() {
    // Create a temporary directory for the test
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path().to_path_buf();

    // Create a persistence config with persistence disabled
    let config = PersistenceConfig {
        enabled: false,
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

    // Save the registry (should be a no-op)
    persistence.save(&registry).unwrap();

    // Verify the registry file does not exist
    let registry_path = temp_path.join("registry.json");
    assert!(!registry_path.exists());

    // Load the registry (should return an empty registry)
    let loaded_registry = persistence.load().unwrap();

    // Verify the loaded registry is empty
    assert_eq!(loaded_registry.count(), 0);
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

#[tokio::test]
async fn test_persistence_clear() {
    // Create a temporary directory for the test
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path().to_path_buf();

    // Create a persistence config
    let config = PersistenceConfig {
        enabled: true,
        directory: temp_path.clone(),
        auto_save_interval_seconds: 0,
        max_backups: 3,
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

    // Save the registry multiple times to create backups
    for _ in 0..5 {
        persistence.save(&registry).unwrap();
    }

    // Verify files exist
    let registry_path = temp_path.join("registry.json");
    assert!(registry_path.exists());

    for i in 0..3 {
        let backup_path = temp_path.join(format!("registry.backup.{}.json", i));
        assert!(backup_path.exists());
    }

    // Clear the persistence
    persistence.clear().unwrap();

    // Verify files are gone
    assert!(!registry_path.exists());

    for i in 0..3 {
        let backup_path = temp_path.join(format!("registry.backup.{}.json", i));
        assert!(!backup_path.exists());
    }
}

#[tokio::test]
async fn test_persistence_exists() {
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

    // Initially, no registry file exists
    assert!(!persistence.exists());

    // Create a registry and add some models
    let registry = ModelRegistry::new();
    registry
        .register_model(create_test_model("model-1", "openai"))
        .unwrap();

    // Save the registry
    persistence.save(&registry).unwrap();

    // Now the registry file should exist
    assert!(persistence.exists());

    // Clear the persistence
    persistence.clear().unwrap();

    // Now the registry file should not exist again
    assert!(!persistence.exists());
}

#[tokio::test]
async fn test_persistence_drop_save() {
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

    // Create a scope to test drop behavior
    {
        // Create a persistent registry
        let persistent_registry = create_file_persistent_registry(config.clone()).unwrap();
        let registry = persistent_registry.registry();

        // Add some models
        registry
            .register_model(create_test_model("model-1", "openai"))
            .unwrap();
        registry
            .register_model(create_test_model("model-2", "anthropic"))
            .unwrap();

        // Don't explicitly save - let the drop handler do it
    }

    // Create a new persistent registry (after the previous one was dropped)
    let new_persistent_registry = create_file_persistent_registry(config).unwrap();
    let new_registry = new_persistent_registry.registry();

    // Verify the models were saved and loaded
    assert_eq!(new_registry.count(), 2);
    assert!(new_registry.get_model("model-1").is_ok());
    assert!(new_registry.get_model("model-2").is_ok());
}

#[tokio::test]
async fn test_persistence_with_model_updates() {
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
    let persistent_registry = create_file_persistent_registry(config.clone()).unwrap();
    let registry = persistent_registry.registry();

    // Add a model
    let model = create_test_model("model-1", "openai");
    registry.register_model(model).unwrap();

    // Save the registry
    persistent_registry.save().unwrap();

    // Update the model
    let mut model = registry.get_model("model-1").unwrap();
    model.set_status(ModelStatus::Available);
    model.set_description("Updated description".to_string());
    model.add_metadata("key1".to_string(), "value1".to_string());
    registry.update_model(model).unwrap();

    // Save the registry again
    persistent_registry.save().unwrap();

    // Create a new persistent registry (simulating a restart)
    let new_persistent_registry = create_file_persistent_registry(config).unwrap();
    let new_registry = new_persistent_registry.registry();

    // Verify the model was loaded with updates
    let loaded_model = new_registry.get_model("model-1").unwrap();
    assert_eq!(loaded_model.status, ModelStatus::Available);
    assert_eq!(
        loaded_model.description,
        Some("Updated description".to_string())
    );
    assert_eq!(
        loaded_model.additional_metadata.get("key1"),
        Some(&"value1".to_string())
    );
}
