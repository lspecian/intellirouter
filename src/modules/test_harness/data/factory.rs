//! Data Factory Module
//!
//! This module provides functionality for creating and managing test data factories.
//! Data factories are responsible for generating test data for specific test scenarios.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::generator::{DataGenerator, GeneratedData};
use super::schema::{DataSchema, DataType, FieldDefinition};
use super::store::{DataStore, StoredData};

/// Data factory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFactoryConfig {
    /// Factory name
    pub name: String,
    /// Factory description
    pub description: Option<String>,
    /// Default generator
    pub default_generator: Option<String>,
    /// Default schema
    pub default_schema: Option<String>,
    /// Custom configuration
    pub custom: Option<serde_json::Value>,
}

impl Default for DataFactoryConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: None,
            default_generator: None,
            default_schema: None,
            custom: None,
        }
    }
}

/// Factory-generated data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryData {
    /// Data
    pub data: serde_json::Value,
    /// Metadata
    pub metadata: HashMap<String, String>,
    /// Version
    pub version: String,
    /// Factory name
    pub factory_name: String,
    /// Generator name
    pub generator_name: String,
    /// Schema name
    pub schema_name: Option<String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Data factory trait
#[async_trait]
pub trait DataFactory: Send + Sync {
    /// Get the factory name
    fn name(&self) -> &str;

    /// Get the factory description
    fn description(&self) -> Option<&str>;

    /// Generate data
    async fn generate(&self, config: &serde_json::Value) -> Result<FactoryData, String>;

    /// Generate data with a specific generator
    async fn generate_with_generator(
        &self,
        generator_name: &str,
        config: &serde_json::Value,
    ) -> Result<FactoryData, String>;

    /// Generate data with a specific schema
    async fn generate_with_schema(
        &self,
        schema: &DataSchema,
        generator_name: Option<&str>,
    ) -> Result<FactoryData, String>;

    /// Get the available generators
    fn available_generators(&self) -> Vec<String>;

    /// Get the available schemas
    fn available_schemas(&self) -> Vec<String>;
}

/// Standard data factory
pub struct StandardDataFactory {
    /// Factory configuration
    config: DataFactoryConfig,
    /// Generators
    generators: HashMap<String, Arc<dyn DataGenerator>>,
    /// Schemas
    schemas: HashMap<String, DataSchema>,
    /// Data store for caching generated data
    data_store: Arc<DataStore>,
}

impl StandardDataFactory {
    /// Create a new standard data factory
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            config: DataFactoryConfig {
                name: name.into(),
                ..Default::default()
            },
            generators: HashMap::new(),
            schemas: HashMap::new(),
            data_store: Arc::new(DataStore::new()),
        }
    }

    /// Create a new standard data factory with a custom configuration
    pub fn with_config(config: DataFactoryConfig) -> Self {
        Self {
            config,
            generators: HashMap::new(),
            schemas: HashMap::new(),
            data_store: Arc::new(DataStore::new()),
        }
    }

    /// Register a generator
    pub fn register_generator(
        &mut self,
        name: impl Into<String>,
        generator: Arc<dyn DataGenerator>,
    ) {
        self.generators.insert(name.into(), generator);
    }

    /// Register a schema
    pub fn register_schema(&mut self, schema: DataSchema) {
        self.schemas.insert(schema.name.clone(), schema);
    }

    /// Get a generator by name
    pub fn get_generator(&self, name: &str) -> Option<Arc<dyn DataGenerator>> {
        self.generators.get(name).cloned()
    }

    /// Get a schema by name
    pub fn get_schema(&self, name: &str) -> Option<&DataSchema> {
        self.schemas.get(name)
    }

    /// Get the default generator
    pub fn default_generator(&self) -> Option<Arc<dyn DataGenerator>> {
        self.config
            .default_generator
            .as_ref()
            .and_then(|name| self.get_generator(name))
    }

    /// Get the default schema
    pub fn default_schema(&self) -> Option<&DataSchema> {
        self.config
            .default_schema
            .as_ref()
            .and_then(|name| self.get_schema(name))
    }

    /// Set the default generator
    pub fn set_default_generator(&mut self, name: impl Into<String>) {
        self.config.default_generator = Some(name.into());
    }

    /// Set the default schema
    pub fn set_default_schema(&mut self, name: impl Into<String>) {
        self.config.default_schema = Some(name.into());
    }

    /// Create a factory data object
    fn create_factory_data(
        &self,
        data: serde_json::Value,
        generator_name: &str,
        schema_name: Option<&str>,
    ) -> FactoryData {
        let now = chrono::Utc::now();
        let version = Uuid::new_v4().to_string();

        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("factory".to_string(), self.name().to_string());
        metadata.insert("generator".to_string(), generator_name.to_string());
        if let Some(schema_name) = schema_name {
            metadata.insert("schema".to_string(), schema_name.to_string());
        }
        metadata.insert("version".to_string(), version.clone());
        metadata.insert("timestamp".to_string(), now.to_rfc3339());

        FactoryData {
            data,
            metadata,
            version,
            factory_name: self.name().to_string(),
            generator_name: generator_name.to_string(),
            schema_name: schema_name.map(|s| s.to_string()),
            timestamp: now,
        }
    }
}

#[async_trait]
impl DataFactory for StandardDataFactory {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }

    async fn generate(&self, config: &serde_json::Value) -> Result<FactoryData, String> {
        // Check if we have a generator specified in the config
        let generator_name = config
            .get("generator")
            .and_then(|v| v.as_str())
            .or_else(|| self.config.default_generator.as_deref())
            .ok_or_else(|| "No generator specified".to_string())?;

        // Check if we have a schema specified in the config
        let schema_name = config
            .get("schema")
            .and_then(|v| v.as_str())
            .or_else(|| self.config.default_schema.as_deref());

        // Generate the data
        self.generate_with_generator(generator_name, config).await
    }

    async fn generate_with_generator(
        &self,
        generator_name: &str,
        config: &serde_json::Value,
    ) -> Result<FactoryData, String> {
        // Get the generator
        let generator = self
            .get_generator(generator_name)
            .ok_or_else(|| format!("Generator '{}' not found", generator_name))?;

        // Generate the data
        let generated_data = generator.generate(config).await?;

        // Create the factory data
        let schema_name = config.get("schema").and_then(|v| v.as_str());
        let factory_data =
            self.create_factory_data(generated_data.data, generator_name, schema_name);

        // Store the data in the cache
        self.data_store
            .store(
                factory_data.version.clone(),
                serde_json::to_value(&factory_data).unwrap(),
            )
            .await
            .map_err(|e| format!("Failed to store factory data: {}", e))?;

        Ok(factory_data)
    }

    async fn generate_with_schema(
        &self,
        schema: &DataSchema,
        generator_name: Option<&str>,
    ) -> Result<FactoryData, String> {
        // Get the generator
        let generator_name = generator_name
            .or_else(|| self.config.default_generator.as_deref())
            .ok_or_else(|| "No generator specified".to_string())?;

        let generator = self
            .get_generator(generator_name)
            .ok_or_else(|| format!("Generator '{}' not found", generator_name))?;

        // Generate the data
        let generated_data = generator.generate_from_schema(schema).await?;

        // Create the factory data
        let factory_data =
            self.create_factory_data(generated_data.data, generator_name, Some(&schema.name));

        // Store the data in the cache
        self.data_store
            .store(
                factory_data.version.clone(),
                serde_json::to_value(&factory_data).unwrap(),
            )
            .await
            .map_err(|e| format!("Failed to store factory data: {}", e))?;

        Ok(factory_data)
    }

    fn available_generators(&self) -> Vec<String> {
        self.generators.keys().cloned().collect()
    }

    fn available_schemas(&self) -> Vec<String> {
        self.schemas.keys().cloned().collect()
    }
}

/// Scenario data factory
pub struct ScenarioDataFactory {
    /// Factory configuration
    config: DataFactoryConfig,
    /// Base factory
    base_factory: Arc<dyn DataFactory>,
    /// Scenario configurations
    scenarios: HashMap<String, serde_json::Value>,
}

impl ScenarioDataFactory {
    /// Create a new scenario data factory
    pub fn new(name: impl Into<String>, base_factory: Arc<dyn DataFactory>) -> Self {
        Self {
            config: DataFactoryConfig {
                name: name.into(),
                ..Default::default()
            },
            base_factory,
            scenarios: HashMap::new(),
        }
    }

    /// Create a new scenario data factory with a custom configuration
    pub fn with_config(config: DataFactoryConfig, base_factory: Arc<dyn DataFactory>) -> Self {
        Self {
            config,
            base_factory,
            scenarios: HashMap::new(),
        }
    }

    /// Register a scenario
    pub fn register_scenario(&mut self, name: impl Into<String>, config: serde_json::Value) {
        self.scenarios.insert(name.into(), config);
    }

    /// Get a scenario by name
    pub fn get_scenario(&self, name: &str) -> Option<&serde_json::Value> {
        self.scenarios.get(name)
    }

    /// Get all scenario names
    pub fn scenario_names(&self) -> Vec<String> {
        self.scenarios.keys().cloned().collect()
    }
}

#[async_trait]
impl DataFactory for ScenarioDataFactory {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }

    async fn generate(&self, config: &serde_json::Value) -> Result<FactoryData, String> {
        // Check if we have a scenario specified in the config
        let scenario_name = config
            .get("scenario")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "No scenario specified".to_string())?;

        // Get the scenario configuration
        let scenario_config = self
            .get_scenario(scenario_name)
            .ok_or_else(|| format!("Scenario '{}' not found", scenario_name))?;

        // Merge the scenario configuration with the provided configuration
        let mut merged_config = scenario_config.clone();
        if let Some(obj) = merged_config.as_object_mut() {
            if let Some(provided_obj) = config.as_object() {
                for (key, value) in provided_obj {
                    if key != "scenario" {
                        obj.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        // Generate the data using the base factory
        let mut factory_data = self.base_factory.generate(&merged_config).await?;

        // Add scenario information to the metadata
        factory_data
            .metadata
            .insert("scenario".to_string(), scenario_name.to_string());

        Ok(factory_data)
    }

    async fn generate_with_generator(
        &self,
        generator_name: &str,
        config: &serde_json::Value,
    ) -> Result<FactoryData, String> {
        // Check if we have a scenario specified in the config
        let scenario_name = config
            .get("scenario")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "No scenario specified".to_string())?;

        // Get the scenario configuration
        let scenario_config = self
            .get_scenario(scenario_name)
            .ok_or_else(|| format!("Scenario '{}' not found", scenario_name))?;

        // Merge the scenario configuration with the provided configuration
        let mut merged_config = scenario_config.clone();
        if let Some(obj) = merged_config.as_object_mut() {
            if let Some(provided_obj) = config.as_object() {
                for (key, value) in provided_obj {
                    if key != "scenario" {
                        obj.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        // Generate the data using the base factory
        let mut factory_data = self
            .base_factory
            .generate_with_generator(generator_name, &merged_config)
            .await?;

        // Add scenario information to the metadata
        factory_data
            .metadata
            .insert("scenario".to_string(), scenario_name.to_string());

        Ok(factory_data)
    }

    async fn generate_with_schema(
        &self,
        schema: &DataSchema,
        generator_name: Option<&str>,
    ) -> Result<FactoryData, String> {
        // Generate the data using the base factory
        let factory_data = self
            .base_factory
            .generate_with_schema(schema, generator_name)
            .await?;

        Ok(factory_data)
    }

    fn available_generators(&self) -> Vec<String> {
        self.base_factory.available_generators()
    }

    fn available_schemas(&self) -> Vec<String> {
        self.base_factory.available_schemas()
    }
}

#[cfg(test)]
mod tests {
    use super::super::generator::FakerDataGenerator;
    use super::*;

    #[tokio::test]
    async fn test_standard_data_factory() {
        // Create a factory
        let mut factory = StandardDataFactory::new("test-factory");

        // Register a generator
        let generator = Arc::new(FakerDataGenerator::new());
        factory.register_generator("faker", generator);

        // Register a schema
        let schema = DataSchema::new(
            "user",
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
                        "age".to_string(),
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
        factory.register_schema(schema);

        // Set defaults
        factory.set_default_generator("faker");
        factory.set_default_schema("user");

        // Generate data
        let config = serde_json::json!({});
        let factory_data = factory.generate(&config).await.unwrap();

        // Check the data
        assert_eq!(factory_data.factory_name, "test-factory");
        assert_eq!(factory_data.generator_name, "faker");
        assert_eq!(factory_data.schema_name, Some("user".to_string()));
        assert!(factory_data.data.is_object());
        assert!(factory_data.data.get("name").is_some());
        assert!(factory_data.data.get("age").is_some());
    }
}
