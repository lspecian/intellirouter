//! Data Generator Module
//!
//! This module provides functionality for generating test data.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use fake::{Fake, Faker};
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use super::schema::{DataSchema, DataType, FieldDefinition};

/// Generated data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedData {
    /// Data
    pub data: serde_json::Value,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Data generator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataGeneratorConfig {
    /// Generator name
    pub name: String,
    /// Generator description
    pub description: Option<String>,
    /// Random seed
    pub seed: Option<u64>,
    /// Custom configuration
    pub custom: Option<serde_json::Value>,
}

impl Default for DataGeneratorConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: None,
            seed: None,
            custom: None,
        }
    }
}

/// Data generator trait
#[async_trait]
pub trait DataGenerator: Send + Sync {
    /// Get the generator name
    fn name(&self) -> &str;

    /// Get the generator description
    fn description(&self) -> Option<&str>;

    /// Generate data
    async fn generate(&self, config: &serde_json::Value) -> Result<GeneratedData, String>;

    /// Generate data from a schema
    async fn generate_from_schema(&self, schema: &DataSchema) -> Result<GeneratedData, String>;
}

/// Faker data generator
pub struct FakerDataGenerator {
    /// Generator configuration
    config: DataGeneratorConfig,
    /// Random number generator
    rng: StdRng,
}

impl FakerDataGenerator {
    /// Create a new faker data generator
    pub fn new() -> Self {
        Self {
            config: DataGeneratorConfig {
                name: "faker".to_string(),
                description: Some("Faker data generator".to_string()),
                seed: None,
                custom: None,
            },
            rng: StdRng::from_entropy(),
        }
    }

    /// Create a new faker data generator with a custom configuration
    pub fn with_config(config: DataGeneratorConfig) -> Self {
        let rng = if let Some(seed) = config.seed {
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::from_entropy()
        };

        Self { config, rng }
    }

    /// Generate a value for a field
    fn generate_value(&self, field_def: &FieldDefinition) -> serde_json::Value {
        match &field_def.data_type {
            DataType::String => {
                if let Some(format) = &field_def.format {
                    match format.as_str() {
                        "email" => serde_json::Value::String(Faker.fake::<String>()),
                        "uuid" => serde_json::Value::String(uuid::Uuid::new_v4().to_string()),
                        "date" => serde_json::Value::String(Faker.fake::<String>()),
                        "datetime" => serde_json::Value::String(Faker.fake::<String>()),
                        "time" => serde_json::Value::String(Faker.fake::<String>()),
                        "uri" => serde_json::Value::String(Faker.fake::<String>()),
                        "hostname" => serde_json::Value::String(Faker.fake::<String>()),
                        "ipv4" => serde_json::Value::String(Faker.fake::<String>()),
                        "ipv6" => serde_json::Value::String(Faker.fake::<String>()),
                        _ => serde_json::Value::String(Faker.fake::<String>()),
                    }
                } else if let Some(pattern) = &field_def.pattern {
                    // TODO: Generate string from pattern
                    serde_json::Value::String(Faker.fake::<String>())
                } else {
                    serde_json::Value::String(Faker.fake::<String>())
                }
            }
            DataType::Integer => {
                let min = field_def.minimum.unwrap_or(0);
                let max = field_def.maximum.unwrap_or(100);
                serde_json::Value::Number(serde_json::Number::from((min..=max).fake::<i64>()))
            }
            DataType::Float => {
                let min = field_def.minimum.unwrap_or(0) as f64;
                let max = field_def.maximum.unwrap_or(100) as f64;
                let value: f64 = (min..=max).fake();
                serde_json::json!(value)
            }
            DataType::Boolean => serde_json::Value::Bool(Faker.fake::<bool>()),
            DataType::Array(item_type) => {
                let min_length = field_def.min_length.unwrap_or(0);
                let max_length = field_def.max_length.unwrap_or(10);
                let length = (min_length..=max_length).fake::<usize>();

                let item_field = FieldDefinition {
                    data_type: (**item_type).clone(),
                    ..Default::default()
                };

                let mut items = Vec::with_capacity(length);
                for _ in 0..length {
                    items.push(self.generate_value(&item_field));
                }

                serde_json::Value::Array(items)
            }
            DataType::Object(fields) => {
                let mut obj = serde_json::Map::new();

                for (field_name, field_def) in fields {
                    if field_def.required || Faker.fake::<bool>() {
                        obj.insert(field_name.clone(), self.generate_value(field_def));
                    }
                }

                serde_json::Value::Object(obj)
            }
            DataType::Enum(values) => {
                let index = (0..values.len()).fake::<usize>();
                serde_json::Value::String(values[index].clone())
            }
            DataType::Any => {
                // Generate a random type
                let type_index = (0..5).fake::<usize>();
                match type_index {
                    0 => serde_json::Value::String(Faker.fake::<String>()),
                    1 => serde_json::Value::Number(serde_json::Number::from(Faker.fake::<i64>())),
                    2 => serde_json::Value::Bool(Faker.fake::<bool>()),
                    3 => {
                        let length = (0..10).fake::<usize>();
                        let mut items = Vec::with_capacity(length);
                        for _ in 0..length {
                            items.push(serde_json::Value::String(Faker.fake::<String>()));
                        }
                        serde_json::Value::Array(items)
                    }
                    4 => {
                        let mut obj = serde_json::Map::new();
                        let field_count = (0..5).fake::<usize>();
                        for _ in 0..field_count {
                            obj.insert(
                                Faker.fake::<String>(),
                                serde_json::Value::String(Faker.fake::<String>()),
                            );
                        }
                        serde_json::Value::Object(obj)
                    }
                    _ => serde_json::Value::Null,
                }
            }
            DataType::Null => serde_json::Value::Null,
        }
    }
}

#[async_trait]
impl DataGenerator for FakerDataGenerator {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }

    async fn generate(&self, config: &serde_json::Value) -> Result<GeneratedData, String> {
        // Parse the configuration
        let schema: DataSchema = serde_json::from_value(config.clone())
            .map_err(|e| format!("Failed to parse schema: {}", e))?;

        // Generate data from the schema
        self.generate_from_schema(&schema).await
    }

    async fn generate_from_schema(&self, schema: &DataSchema) -> Result<GeneratedData, String> {
        // Create a field definition from the root type
        let field_def = FieldDefinition {
            data_type: schema.root_type.clone(),
            required: true,
            ..Default::default()
        };

        // Generate the data
        let data = self.generate_value(&field_def);

        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("generator".to_string(), self.name().to_string());
        metadata.insert("schema".to_string(), schema.name.clone());
        metadata.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());

        Ok(GeneratedData { data, metadata })
    }
}

/// Random data generator
pub struct RandomDataGenerator {
    /// Generator configuration
    config: DataGeneratorConfig,
    /// Random number generator
    rng: StdRng,
}

impl RandomDataGenerator {
    /// Create a new random data generator
    pub fn new() -> Self {
        Self {
            config: DataGeneratorConfig {
                name: "random".to_string(),
                description: Some("Random data generator".to_string()),
                seed: None,
                custom: None,
            },
            rng: StdRng::from_entropy(),
        }
    }

    /// Create a new random data generator with a custom configuration
    pub fn with_config(config: DataGeneratorConfig) -> Self {
        let rng = if let Some(seed) = config.seed {
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::from_entropy()
        };

        Self { config, rng }
    }
}

#[async_trait]
impl DataGenerator for RandomDataGenerator {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }

    async fn generate(&self, config: &serde_json::Value) -> Result<GeneratedData, String> {
        // Parse the configuration
        let count = config.get("count").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

        let data_type = config
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("mixed");

        // Generate the data
        let data = match data_type {
            "string" => {
                let mut items = Vec::with_capacity(count);
                for _ in 0..count {
                    items.push(serde_json::Value::String(Faker.fake::<String>()));
                }
                serde_json::Value::Array(items)
            }
            "integer" => {
                let mut items = Vec::with_capacity(count);
                for _ in 0..count {
                    items.push(serde_json::Value::Number(serde_json::Number::from(
                        Faker.fake::<i64>(),
                    )));
                }
                serde_json::Value::Array(items)
            }
            "float" => {
                let mut items = Vec::with_capacity(count);
                for _ in 0..count {
                    let value: f64 = Faker.fake();
                    items.push(serde_json::json!(value));
                }
                serde_json::Value::Array(items)
            }
            "boolean" => {
                let mut items = Vec::with_capacity(count);
                for _ in 0..count {
                    items.push(serde_json::Value::Bool(Faker.fake::<bool>()));
                }
                serde_json::Value::Array(items)
            }
            "object" => {
                let mut items = Vec::with_capacity(count);
                for _ in 0..count {
                    let mut obj = serde_json::Map::new();
                    let field_count = (1..10).fake::<usize>();
                    for _ in 0..field_count {
                        obj.insert(
                            Faker.fake::<String>(),
                            serde_json::Value::String(Faker.fake::<String>()),
                        );
                    }
                    items.push(serde_json::Value::Object(obj));
                }
                serde_json::Value::Array(items)
            }
            "mixed" | _ => {
                let mut items = Vec::with_capacity(count);
                for _ in 0..count {
                    let type_index = (0..5).fake::<usize>();
                    match type_index {
                        0 => items.push(serde_json::Value::String(Faker.fake::<String>())),
                        1 => items.push(serde_json::Value::Number(serde_json::Number::from(
                            Faker.fake::<i64>(),
                        ))),
                        2 => items.push(serde_json::Value::Bool(Faker.fake::<bool>())),
                        3 => {
                            let mut obj = serde_json::Map::new();
                            let field_count = (1..5).fake::<usize>();
                            for _ in 0..field_count {
                                obj.insert(
                                    Faker.fake::<String>(),
                                    serde_json::Value::String(Faker.fake::<String>()),
                                );
                            }
                            items.push(serde_json::Value::Object(obj));
                        }
                        4 => {
                            let array_length = (1..5).fake::<usize>();
                            let mut array = Vec::with_capacity(array_length);
                            for _ in 0..array_length {
                                array.push(serde_json::Value::String(Faker.fake::<String>()));
                            }
                            items.push(serde_json::Value::Array(array));
                        }
                        _ => items.push(serde_json::Value::Null),
                    }
                }
                serde_json::Value::Array(items)
            }
        };

        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("generator".to_string(), self.name().to_string());
        metadata.insert("count".to_string(), count.to_string());
        metadata.insert("type".to_string(), data_type.to_string());
        metadata.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());

        Ok(GeneratedData { data, metadata })
    }

    async fn generate_from_schema(&self, schema: &DataSchema) -> Result<GeneratedData, String> {
        // Create a configuration
        let config = serde_json::json!({
            "count": 1,
            "type": "object"
        });

        // Generate the data
        self.generate(&config).await
    }
}

/// Template data generator
pub struct TemplateDataGenerator {
    /// Generator configuration
    config: DataGeneratorConfig,
    /// Random number generator
    rng: StdRng,
}

impl TemplateDataGenerator {
    /// Create a new template data generator
    pub fn new() -> Self {
        Self {
            config: DataGeneratorConfig {
                name: "template".to_string(),
                description: Some("Template data generator".to_string()),
                seed: None,
                custom: None,
            },
            rng: StdRng::from_entropy(),
        }
    }

    /// Create a new template data generator with a custom configuration
    pub fn with_config(config: DataGeneratorConfig) -> Self {
        let rng = if let Some(seed) = config.seed {
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::from_entropy()
        };

        Self { config, rng }
    }
}

#[async_trait]
impl DataGenerator for TemplateDataGenerator {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }

    async fn generate(&self, config: &serde_json::Value) -> Result<GeneratedData, String> {
        // Parse the configuration
        let template = config
            .get("template")
            .ok_or_else(|| "Template is required".to_string())?;

        let count = config.get("count").and_then(|v| v.as_u64()).unwrap_or(1) as usize;

        // Generate the data
        if count == 1 {
            // Just use the template as is
            let data = template.clone();

            // Create metadata
            let mut metadata = HashMap::new();
            metadata.insert("generator".to_string(), self.name().to_string());
            metadata.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());

            Ok(GeneratedData { data, metadata })
        } else {
            // Create an array of template instances
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                items.push(template.clone());
            }

            // Create metadata
            let mut metadata = HashMap::new();
            metadata.insert("generator".to_string(), self.name().to_string());
            metadata.insert("count".to_string(), count.to_string());
            metadata.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());

            Ok(GeneratedData {
                data: serde_json::Value::Array(items),
                metadata,
            })
        }
    }

    async fn generate_from_schema(&self, _schema: &DataSchema) -> Result<GeneratedData, String> {
        Err("Template data generator does not support generating from schema".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_faker_data_generator() {
        let generator = FakerDataGenerator::new();

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

        // Generate data
        let data = generator.generate_from_schema(&schema).await.unwrap();

        // Check the data
        assert!(data.data.is_object());
        assert!(data.data.get("name").unwrap().is_string());
        assert!(data.data.get("age").unwrap().is_number());
    }
}
