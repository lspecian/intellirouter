//! Enhanced Test Data Management Example
//!
//! This example demonstrates how to use the enhanced IntelliRouter test data management framework.

use intellirouter::modules::test_harness::{
    data::{
        CleanupManager, CleanupScope, CleanupStrategy, DataFactory, DataGenerator, DataLoader,
        DataSchema, DataStore, DataType, DataValidator, DataVersion, FactoryData, FieldDefinition,
        FileCleanupHandler, IsolationContext, IsolationLevel, IsolationManager, JsonDataLoader,
        SchemaValidator, StandardDataFactory, TestDataManager, ValidationResult, VersionManager,
        VersionRepository,
    },
    types::TestHarnessError,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("IntelliRouter Enhanced Test Data Management Example");
    println!("==================================================");

    // Create a temporary directory for test data
    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir.path().to_path_buf();
    println!("Using temporary directory: {}", data_dir.display());

    // Create a test data manager
    let mut data_manager = TestDataManager::new(&data_dir);

    // Register data loaders
    let json_loader = Arc::new(JsonDataLoader::new());
    data_manager.register_loader("json", json_loader.clone());
    println!("Registered JSON data loader");

    // Register data generators and validators
    setup_generators_and_validators(&mut data_manager);

    // Setup data factories
    setup_factories(&mut data_manager);

    // Setup version repositories
    setup_version_repositories(&data_manager).await;

    // Setup cleanup handlers
    setup_cleanup_handlers(&data_manager).await;

    // Create a user schema
    let user_schema = create_user_schema();
    println!("\nCreated user schema: {}", user_schema.name);

    // PART 1: Basic Data Generation and Validation
    println!("\n=== PART 1: Basic Data Generation and Validation ===");

    // Generate test data using a factory
    println!("\nGenerating test data using factory...");
    let factory_config = serde_json::json!({
        "schema": "user",
        "count": 1
    });

    let user: serde_json::Value = data_manager
        .generate_factory_data("standard", &factory_config)
        .await?;

    println!("Generated user: {}", serde_json::to_string_pretty(&user)?);

    // Validate the generated data
    println!("\nValidating generated data...");
    let validation_result = data_manager
        .validate_data("schema", &user, Some(&user_schema))
        .await?;

    if validation_result.passed {
        println!("Validation passed!");
    } else {
        println!("Validation failed!");
        for error in &validation_result.errors {
            println!("  Error at {}: {}", error.path, error.message);
        }
    }

    // PART 2: Data Versioning
    println!("\n=== PART 2: Data Versioning ===");

    // Create a version of the user data
    println!("\nCreating a version of the user data...");
    let version1 = data_manager
        .create_version("user-v1", &user, Some("main"))
        .await?;

    println!("Created version: {} (ID: {})", version1.name, version1.id);

    // Modify the user data
    let mut modified_user = user.clone();
    if let Some(obj) = modified_user.as_object_mut() {
        if let Some(status) = obj.get_mut("status") {
            *status = serde_json::json!("inactive");
        }
    }

    // Create a new version based on the modified data
    println!("\nCreating a new version with modified data...");
    let version2 = data_manager
        .create_version("user-v2", &modified_user, Some("main"))
        .await?;

    println!("Created version: {} (ID: {})", version2.name, version2.id);

    // Get the version history
    println!("\nRetrieving version history...");
    let repository = data_manager
        .version_manager()
        .get_repository("main")
        .await
        .unwrap();

    let history = repository.get_version_history(&version2.id).await?;
    println!("Version history:");
    for version in &history {
        println!("  - {} (ID: {})", version.name, version.id);
    }

    // PART 3: Data Isolation
    println!("\n=== PART 3: Data Isolation ===");

    // Create a test run context
    println!("\nCreating a test run isolation context...");
    let run_context = data_manager
        .create_isolation_context("test-run-1", "run", Some(IsolationLevel::TestRun))
        .await?;

    println!(
        "Created isolation context: {} (ID: {})",
        run_context.name, run_context.id
    );

    // Push the context onto the stack
    data_manager.push_isolation_context(&run_context.id).await?;
    println!("Pushed run context onto the stack");

    // Store data in the run context
    println!("\nStoring data in the run context...");
    data_manager
        .store_isolated_data("run-data", &serde_json::json!({ "key": "run-value" }))
        .await?;

    println!("Stored run data");

    // Create a test case context
    println!("\nCreating a test case isolation context...");
    let case_context = data_manager
        .isolation_manager()
        .create_child_context(&run_context.id, "test-case-1", "case")
        .await?;

    println!(
        "Created isolation context: {} (ID: {})",
        case_context.name, case_context.id
    );

    // Push the case context onto the stack
    data_manager
        .push_isolation_context(&case_context.id)
        .await?;
    println!("Pushed case context onto the stack");

    // Store data in the case context
    println!("\nStoring data in the case context...");
    data_manager
        .store_isolated_data("case-data", &serde_json::json!({ "key": "case-value" }))
        .await?;

    println!("Stored case data");

    // Retrieve data from the case context
    println!("\nRetrieving data from the case context...");
    let case_data: Option<serde_json::Value> =
        data_manager.retrieve_isolated_data("case-data").await?;

    println!("Case data: {}", serde_json::to_string_pretty(&case_data)?);

    // Pop the case context
    println!("\nPopping the case context...");
    data_manager.pop_isolation_context().await?;
    println!("Popped case context");

    // Try to retrieve the case data from the run context (should fail)
    println!("\nTrying to retrieve case data from the run context...");
    let case_data: Option<serde_json::Value> =
        data_manager.retrieve_isolated_data("case-data").await?;

    if let Some(data) = case_data {
        println!("Case data found: {}", serde_json::to_string_pretty(&data)?);
    } else {
        println!("Case data not found (expected)");
    }

    // Retrieve the run data
    println!("\nRetrieving run data from the run context...");
    let run_data: Option<serde_json::Value> =
        data_manager.retrieve_isolated_data("run-data").await?;

    println!("Run data: {}", serde_json::to_string_pretty(&run_data)?);

    // PART 4: Data Cleanup
    println!("\n=== PART 4: Data Cleanup ===");

    // Create a test file
    println!("\nCreating a test file...");
    let test_file = data_dir.join("test.txt");
    tokio::fs::write(&test_file, b"test data").await?;
    println!("Created test file: {}", test_file.display());

    // Create a cleanup task
    println!("\nCreating a cleanup task...");
    let cleanup_task = data_manager
        .create_cleanup_task(
            "test-cleanup",
            Some(CleanupStrategy::Immediate),
            Some(CleanupScope::All),
            Some(run_context.id.clone()),
        )
        .await?;

    println!(
        "Created cleanup task: {} (ID: {})",
        cleanup_task.name, cleanup_task.id
    );

    // Add resources to the cleanup task
    println!("\nAdding resources to the cleanup task...");
    data_manager
        .add_cleanup_resource(&cleanup_task.id, "file", test_file.to_string_lossy())
        .await?;

    println!("Added file resource to cleanup task");

    // Execute the cleanup task
    println!("\nExecuting the cleanup task...");
    data_manager.execute_cleanup_task(&cleanup_task.id).await?;
    println!("Executed cleanup task");

    // Check if the file was removed
    println!("\nChecking if the file was removed...");
    if !test_file.exists() {
        println!("File was successfully removed");
    } else {
        println!("File still exists (unexpected)");
    }

    // Clean up the isolation context
    println!("\nCleaning up the isolation context...");
    data_manager
        .isolation_manager()
        .cleanup_context(&run_context.id)
        .await?;

    println!("Cleaned up isolation context");

    println!("\nEnhanced test data management example completed successfully!");
    Ok(())
}

/// Create a user schema
fn create_user_schema() -> DataSchema {
    DataSchema::new(
        "user",
        DataType::Object(
            [
                (
                    "id".to_string(),
                    FieldDefinition {
                        data_type: DataType::String,
                        required: true,
                        description: Some("User ID".to_string()),
                        ..Default::default()
                    },
                ),
                (
                    "name".to_string(),
                    FieldDefinition {
                        data_type: DataType::String,
                        required: true,
                        description: Some("User name".to_string()),
                        ..Default::default()
                    },
                ),
                (
                    "email".to_string(),
                    FieldDefinition {
                        data_type: DataType::String,
                        required: true,
                        description: Some("User email".to_string()),
                        format: Some("email".to_string()),
                        ..Default::default()
                    },
                ),
                (
                    "age".to_string(),
                    FieldDefinition {
                        data_type: DataType::Integer,
                        required: false,
                        description: Some("User age".to_string()),
                        minimum: Some(18),
                        maximum: Some(120),
                        ..Default::default()
                    },
                ),
                (
                    "roles".to_string(),
                    FieldDefinition {
                        data_type: DataType::Array(Box::new(DataType::String)),
                        required: false,
                        description: Some("User roles".to_string()),
                        ..Default::default()
                    },
                ),
                (
                    "status".to_string(),
                    FieldDefinition {
                        data_type: DataType::Enum(vec![
                            "active".to_string(),
                            "inactive".to_string(),
                            "suspended".to_string(),
                        ]),
                        required: false,
                        description: Some("User status".to_string()),
                        ..Default::default()
                    },
                ),
            ]
            .into_iter()
            .collect(),
        ),
    )
    .with_description("User schema")
    .with_version("1.0.0")
}

/// Setup generators and validators
fn setup_generators_and_validators(data_manager: &mut TestDataManager) {
    // Register data generators
    let faker_generator =
        Arc::new(intellirouter::modules::test_harness::data::FakerDataGenerator::new());
    data_manager.register_generator("faker", faker_generator.clone());
    println!("Registered Faker data generator");

    // Register data validators
    let schema_validator = Arc::new(SchemaValidator::new());
    data_manager.register_validator("schema", schema_validator.clone());
    println!("Registered Schema validator");
}

/// Setup data factories
async fn setup_factories(data_manager: &mut TestDataManager) {
    // Create a standard factory
    let mut standard_factory = StandardDataFactory::new("standard");

    // Register generators with the factory
    let faker_generator =
        Arc::new(intellirouter::modules::test_harness::data::FakerDataGenerator::new());
    standard_factory.register_generator("faker", faker_generator);

    // Register schemas with the factory
    standard_factory.register_schema(create_user_schema());

    // Set defaults
    standard_factory.set_default_generator("faker");
    standard_factory.set_default_schema("user");

    // Register the factory with the data manager
    data_manager.register_factory("standard", Arc::new(standard_factory));
    println!("Registered Standard data factory");
}

/// Setup version repositories
async fn setup_version_repositories(data_manager: &TestDataManager) {
    // Create a version repository
    let repository = Arc::new(VersionRepository::new("main"));

    // Register the repository with the version manager
    data_manager
        .version_manager()
        .register_repository("main", repository)
        .await;

    // Set the default repository
    data_manager
        .version_manager()
        .set_default_repository("main")
        .await;

    println!("Registered version repository: main");
}

/// Setup cleanup handlers
async fn setup_cleanup_handlers(data_manager: &TestDataManager) {
    // Create a file cleanup handler
    let file_handler = Arc::new(FileCleanupHandler::new());

    // Register the handler with the cleanup manager
    data_manager
        .cleanup_manager()
        .register_handler("file", file_handler)
        .await;

    println!("Registered file cleanup handler");
}
