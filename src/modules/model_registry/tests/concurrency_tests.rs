//! Concurrency Tests for Model Registry
//!
//! This module contains tests to verify the thread safety of the Model Registry.
//! It includes tests for concurrent registration, updates, reads, and writes.

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::Barrier;
    use tokio::time;

    use crate::modules::model_registry::api::ModelRegistryApi;
    use crate::modules::model_registry::persistence::{
        create_file_persistent_registry, FileModelRegistryPersistence, ModelRegistryPersistence,
        PersistenceConfig, PersistentModelRegistry,
    };
    use crate::modules::model_registry::storage::ModelRegistry;
    use crate::modules::model_registry::types::{ModelMetadata, ModelStatus, ModelType};
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

    #[tokio::test]
    async fn test_concurrent_registration() {
        let api = Arc::new(ModelRegistryApi::new());
        let num_tasks = 100;
        let barrier = Arc::new(Barrier::new(num_tasks));

        // Create tasks that will all register models concurrently
        let mut handles = Vec::with_capacity(num_tasks);

        for i in 0..num_tasks {
            let api_clone = api.clone();
            let barrier_clone = barrier.clone();
            let model_id = format!("model-{}", i);

            let handle = tokio::spawn(async move {
                // Wait for all tasks to be ready
                barrier_clone.wait().await;

                // Register a model
                let model = create_test_model(&model_id, "test");
                api_clone.register_model(model.clone()).unwrap();

                // Verify the model was registered
                let retrieved = api_clone.get_model(&model_id).unwrap();
                assert_eq!(retrieved.id, model_id);
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all models were registered
        assert_eq!(api.count(), num_tasks);
    }

    #[tokio::test]
    async fn test_concurrent_updates() {
        let api = Arc::new(ModelRegistryApi::new());
        let model_id = "concurrent-update-model";

        // Register a model
        let model = create_test_model(model_id, "test");
        api.register_model(model).unwrap();

        let num_tasks = 100;
        let barrier = Arc::new(Barrier::new(num_tasks));

        // Create tasks that will all update the model concurrently
        let mut handles = Vec::with_capacity(num_tasks);

        for i in 0..num_tasks {
            let api_clone = api.clone();
            let barrier_clone = barrier.clone();

            let handle = tokio::spawn(async move {
                // Wait for all tasks to be ready
                barrier_clone.wait().await;

                // Get the model
                let mut model = api_clone.get_model(model_id).unwrap();

                // Update the model
                model.add_metadata(format!("key-{}", i), format!("value-{}", i));
                api_clone.update_model(model).unwrap();
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify the model was updated
        let updated_model = api.get_model(model_id).unwrap();
        assert_eq!(updated_model.additional_metadata.len(), num_tasks);
    }

    #[tokio::test]
    async fn test_concurrent_reads_and_writes() {
        let api = Arc::new(ModelRegistryApi::new());
        let num_models = 100;

        // Register some models
        for i in 0..num_models {
            let model = create_test_model(&format!("model-{}", i), "test");
            api.register_model(model).unwrap();
        }

        let num_tasks = 100;
        let barrier = Arc::new(Barrier::new(num_tasks * 2)); // For both readers and writers

        // Create reader tasks
        let mut reader_handles = Vec::with_capacity(num_tasks);

        for _ in 0..num_tasks {
            let api_clone = api.clone();
            let barrier_clone = barrier.clone();

            let handle = tokio::spawn(async move {
                // Wait for all tasks to be ready
                barrier_clone.wait().await;

                // Read all models
                let models = api_clone.list_models();
                assert_eq!(models.len(), num_models);

                // Read individual models
                for i in 0..num_models {
                    let model_id = format!("model-{}", i);
                    let model = api_clone.get_model(&model_id).unwrap();
                    assert_eq!(model.id, model_id);
                }
            });

            reader_handles.push(handle);
        }

        // Create writer tasks
        let mut writer_handles = Vec::with_capacity(num_tasks);

        for i in 0..num_tasks {
            let api_clone = api.clone();
            let barrier_clone = barrier.clone();

            let handle = tokio::spawn(async move {
                // Wait for all tasks to be ready
                barrier_clone.wait().await;

                // Update models
                for j in 0..10 {
                    let model_id = format!("model-{}", (i * 10 + j) % num_models);
                    let mut model = api_clone.get_model(&model_id).unwrap();
                    model.set_status(ModelStatus::Available);
                    api_clone.update_model(model).unwrap();
                }
            });

            writer_handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in reader_handles {
            handle.await.unwrap();
        }

        for handle in writer_handles {
            handle.await.unwrap();
        }

        // Verify all models are still there
        assert_eq!(api.count(), num_models);
    }

    #[tokio::test]
    async fn test_concurrent_filters() {
        let api = Arc::new(ModelRegistryApi::new());
        let num_models = 100;

        // Register some models with different providers
        for i in 0..num_models {
            let provider = if i % 2 == 0 { "openai" } else { "anthropic" };
            let model = create_test_model(&format!("model-{}", i), provider);
            api.register_model(model).unwrap();
        }

        let num_tasks = 100;
        let barrier = Arc::new(Barrier::new(num_tasks));

        // Create tasks that will all filter models concurrently
        let mut handles = Vec::with_capacity(num_tasks);

        for i in 0..num_tasks {
            let api_clone = api.clone();
            let barrier_clone = barrier.clone();

            let handle = tokio::spawn(async move {
                // Wait for all tasks to be ready
                barrier_clone.wait().await;

                // Filter models
                let provider = if i % 2 == 0 { "openai" } else { "anthropic" };
                let models = api_clone.find_by_provider(provider);
                assert_eq!(models.len(), num_models / 2);
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_concurrent_persistence() {
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

        // Create a persistent registry
        let persistent_registry = create_file_persistent_registry(config.clone()).unwrap();
        let registry = persistent_registry.registry();

        let num_tasks = 50;
        let barrier = Arc::new(Barrier::new(num_tasks));

        // Create tasks that will all register models concurrently
        let mut handles = Vec::with_capacity(num_tasks);

        for i in 0..num_tasks {
            let registry_clone = registry.clone();
            let barrier_clone = barrier.clone();
            let model_id = format!("model-{}", i);

            let handle = tokio::spawn(async move {
                // Wait for all tasks to be ready
                barrier_clone.wait().await;

                // Register a model
                let model = create_test_model(&model_id, "test");
                registry_clone.register_model(model).unwrap();
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Save the registry
        persistent_registry.save().unwrap();

        // Create a new persistent registry (simulating a restart)
        let new_persistent_registry = create_file_persistent_registry(config).unwrap();
        let new_registry = new_persistent_registry.registry();

        // Verify all models were loaded
        assert_eq!(new_registry.count(), num_tasks);
        for i in 0..num_tasks {
            let model_id = format!("model-{}", i);
            assert!(new_registry.get_model(&model_id).is_ok());
        }
    }

    #[tokio::test]
    async fn test_concurrent_auto_save() {
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
        let persistent_registry = create_file_persistent_registry(config.clone()).unwrap();
        let registry = persistent_registry.registry();

        let num_tasks = 50;
        let barrier = Arc::new(Barrier::new(num_tasks));

        // Create tasks that will all register and update models concurrently
        let mut handles = Vec::with_capacity(num_tasks);

        for i in 0..num_tasks {
            let registry_clone = registry.clone();
            let barrier_clone = barrier.clone();
            let model_id = format!("model-{}", i);

            let handle = tokio::spawn(async move {
                // Wait for all tasks to be ready
                barrier_clone.wait().await;

                // Register a model
                let model = create_test_model(&model_id, "test");
                registry_clone.register_model(model).unwrap();

                // Wait a bit
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

                // Update the model
                let mut model = registry_clone.get_model(&model_id).unwrap();
                model.set_status(ModelStatus::Available);
                registry_clone.update_model(model).unwrap();
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Wait for auto-save to happen
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

        // Verify all models were loaded with their updated status
        assert_eq!(new_registry.count(), num_tasks);
        for i in 0..num_tasks {
            let model_id = format!("model-{}", i);
            let model = new_registry.get_model(&model_id).unwrap();
            assert_eq!(model.status, ModelStatus::Available);
        }
    }

    #[tokio::test]
    async fn test_concurrent_mixed_operations() {
        let api = Arc::new(ModelRegistryApi::new());
        let num_tasks = 100;
        let barrier = Arc::new(Barrier::new(num_tasks * 3)); // For registration, update, and removal tasks

        // Create registration tasks
        let mut registration_handles = Vec::with_capacity(num_tasks);

        for i in 0..num_tasks {
            let api_clone = api.clone();
            let barrier_clone = barrier.clone();
            let model_id = format!("model-{}", i);

            let handle = tokio::spawn(async move {
                // Wait for all tasks to be ready
                barrier_clone.wait().await;

                // Register a model
                let model = create_test_model(&model_id, "test");
                let _ = api_clone.register_model(model); // Ignore errors (might already exist)
            });

            registration_handles.push(handle);
        }

        // Create update tasks
        let mut update_handles = Vec::with_capacity(num_tasks);

        for i in 0..num_tasks {
            let api_clone = api.clone();
            let barrier_clone = barrier.clone();
            let model_id = format!("model-{}", i % (num_tasks / 2)); // Update half the models

            let handle = tokio::spawn(async move {
                // Wait for all tasks to be ready
                barrier_clone.wait().await;

                // Try to update the model (it might not exist yet or might have been removed)
                if let Ok(mut model) = api_clone.get_model(&model_id) {
                    model.add_metadata(format!("key-{}", i), format!("value-{}", i));
                    let _ = api_clone.update_model(model); // Ignore errors
                }
            });

            update_handles.push(handle);
        }

        // Create removal tasks
        let mut removal_handles = Vec::with_capacity(num_tasks);

        for i in 0..num_tasks {
            let api_clone = api.clone();
            let barrier_clone = barrier.clone();
            let model_id = format!("model-{}", i % (num_tasks / 4)); // Remove quarter of the models

            let handle = tokio::spawn(async move {
                // Wait for all tasks to be ready
                barrier_clone.wait().await;

                // Try to remove the model (it might not exist or might have been removed already)
                let _ = api_clone.remove_model(&model_id); // Ignore errors
            });

            removal_handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in registration_handles {
            handle.await.unwrap();
        }

        for handle in update_handles {
            handle.await.unwrap();
        }

        for handle in removal_handles {
            handle.await.unwrap();
        }

        // Verify the registry is in a consistent state
        // We can't assert exact counts due to the concurrent nature of the test
        let models = api.list_models();
        for model in models {
            // Verify each model can be retrieved
            let retrieved = api.get_model(&model.id).unwrap();
            assert_eq!(retrieved.id, model.id);
        }
    }
}
