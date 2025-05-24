//! Test Data Management Example
//!
//! This example demonstrates how to use the IntelliRouter test data management framework.

use intellirouter::modules::test_harness::{
    data::{
        DataGenerator, DataLoader, DataSchema, DataStore, DataType, DataValidator,
        FakerDataGenerator, FieldDefinition, JsonDataLoader, SchemaValidator, ValidationResult,
    },
    types::TestHarnessError,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("IntelliRouter Test Data Management Example");
    println!("=========================================");

    // Create a temporary directory for test data
    let temp_dir = tempfile::tempdir()?;
    let data_dir = temp_dir.path().to_path_buf();
    println!("Using temporary directory: {}", data_dir.display());

    // Create a test data manager
    let mut data_manager =
        intellirouter::modules::test_harness::data::TestDataManager::new(&data_dir);

    // Register data loaders
    let json_loader = Arc::new(JsonDataLoader::new());
    data_manager.register_loader("json", json_loader.clone());
    println!("Registered JSON data loader");

    // Register data generators
    let faker_generator = Arc::new(FakerDataGenerator::new());
    data_manager.register_generator("faker", faker_generator.clone());
    println!("Registered Faker data generator");

    // Register data validators
    let schema_validator = Arc::new(SchemaValidator::new());
    data_manager.register_validator("schema", schema_validator.clone());
    println!("Registered Schema validator");

    // Create a data schema
    let schema = DataSchema::new(
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
    .with_version("1.0.0");

    println!("\nCreated user schema:");
    println!("  Name: {}", schema.name);
    println!(
        "  Description: {}",
        schema.description.as_deref().unwrap_or("None")
    );
    println!("  Version: {}", schema.version.as_deref().unwrap_or("None"));

    // Generate test data using the schema
    println!("\nGenerating test data...");
    let generated_data = faker_generator.generate_from_schema(&schema).await?;
    println!(
        "Generated data: {}",
        serde_json::to_string_pretty(&generated_data.data)?
    );

    // Validate the generated data
    println!("\nValidating generated data...");
    let validation_result = schema_validator
        .validate(&generated_data.data, Some(&schema))
        .await?;

    if validation_result.passed {
        println!("Validation passed!");
    } else {
        println!("Validation failed!");
        for error in &validation_result.errors {
            println!("  Error at {}: {}", error.path, error.message);
        }
    }

    // Save the generated data to a file
    let data_file = data_dir.join("user.json");
    println!("\nSaving data to file: {}", data_file.display());
    json_loader.save(&data_file, &generated_data).await?;

    // Load the data from the file
    println!("\nLoading data from file...");
    let loaded_data = json_loader.load(&data_file).await?;
    println!(
        "Loaded data: {}",
        serde_json::to_string_pretty(&loaded_data.data)?
    );

    // Store the data in the data store
    println!("\nStoring data in data store...");
    let data_store = DataStore::new();
    data_store.store("user1", loaded_data.data.clone()).await?;
    println!("Data stored with key: user1");

    // Retrieve the data from the data store
    println!("\nRetrieving data from data store...");
    let retrieved_data = data_store.retrieve("user1").await?;
    if let Some(data) = retrieved_data {
        println!(
            "Retrieved data: {}",
            serde_json::to_string_pretty(&data.data)?
        );
    } else {
        println!("Data not found!");
    }

    // Create invalid data
    println!("\nCreating invalid data...");
    let invalid_data = serde_json::json!({
        "id": "user123",
        "name": "John Doe",
        // Missing required email field
        "age": 17, // Age below minimum
        "roles": ["admin", 123], // Invalid role type
        "status": "unknown", // Invalid status value
    });
    println!(
        "Invalid data: {}",
        serde_json::to_string_pretty(&invalid_data)?
    );

    // Validate the invalid data
    println!("\nValidating invalid data...");
    let validation_result = schema_validator
        .validate(&invalid_data, Some(&schema))
        .await?;

    if validation_result.passed {
        println!("Validation passed!");
    } else {
        println!("Validation failed!");
        for error in &validation_result.errors {
            println!("  Error at {}: {}", error.path, error.message);
        }
    }

    println!("\nTest data management example completed successfully!");
    Ok(())
}
