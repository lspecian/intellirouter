//! Data Schema Module
//!
//! This module provides functionality for defining and validating data schemas.

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

/// Data type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    /// String type
    String,
    /// Integer type
    Integer,
    /// Float type
    Float,
    /// Boolean type
    Boolean,
    /// Array type
    Array(Box<DataType>),
    /// Object type
    Object(HashMap<String, FieldDefinition>),
    /// Enum type
    Enum(Vec<String>),
    /// Any type
    Any,
    /// Null type
    Null,
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::String => write!(f, "string"),
            DataType::Integer => write!(f, "integer"),
            DataType::Float => write!(f, "float"),
            DataType::Boolean => write!(f, "boolean"),
            DataType::Array(item_type) => write!(f, "array<{}>", item_type),
            DataType::Object(_) => write!(f, "object"),
            DataType::Enum(values) => write!(f, "enum({})", values.join(", ")),
            DataType::Any => write!(f, "any"),
            DataType::Null => write!(f, "null"),
        }
    }
}

/// Field definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldDefinition {
    /// Field type
    pub data_type: DataType,
    /// Whether the field is required
    pub required: bool,
    /// Field description
    pub description: Option<String>,
    /// Default value
    pub default: Option<serde_json::Value>,
    /// Minimum value (for numeric types)
    pub minimum: Option<i64>,
    /// Maximum value (for numeric types)
    pub maximum: Option<i64>,
    /// Minimum length (for string and array types)
    pub min_length: Option<usize>,
    /// Maximum length (for string and array types)
    pub max_length: Option<usize>,
    /// Pattern (for string types)
    pub pattern: Option<String>,
    /// Format (for string types)
    pub format: Option<String>,
    /// Additional properties (for object types)
    pub additional_properties: bool,
}

impl Default for FieldDefinition {
    fn default() -> Self {
        Self {
            data_type: DataType::Any,
            required: false,
            description: None,
            default: None,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            format: None,
            additional_properties: true,
        }
    }
}

/// Schema validation error
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaValidationError {
    /// Error path
    pub path: String,
    /// Error message
    pub message: String,
    /// Expected type
    pub expected: Option<String>,
    /// Actual type
    pub actual: Option<String>,
}

impl fmt::Display for SchemaValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)?;
        if let (Some(expected), Some(actual)) = (&self.expected, &self.actual) {
            write!(f, " (expected {}, got {})", expected, actual)?;
        }
        Ok(())
    }
}

/// Data schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataSchema {
    /// Schema name
    pub name: String,
    /// Schema description
    pub description: Option<String>,
    /// Schema version
    pub version: Option<String>,
    /// Root type
    pub root_type: DataType,
    /// Additional properties
    pub additional_properties: bool,
}

impl DataSchema {
    /// Create a new data schema
    pub fn new(name: impl Into<String>, root_type: DataType) -> Self {
        Self {
            name: name.into(),
            description: None,
            version: None,
            root_type,
            additional_properties: false,
        }
    }

    /// Set the schema description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the schema version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set whether additional properties are allowed
    pub fn with_additional_properties(mut self, additional_properties: bool) -> Self {
        self.additional_properties = additional_properties;
        self
    }

    /// Validate data against the schema
    pub fn validate(&self, data: &serde_json::Value) -> Result<(), Vec<SchemaValidationError>> {
        let mut errors = Vec::new();
        self.validate_value(data, &self.root_type, "$", &mut errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate a value against a type
    fn validate_value(
        &self,
        value: &serde_json::Value,
        data_type: &DataType,
        path: &str,
        errors: &mut Vec<SchemaValidationError>,
    ) {
        match data_type {
            DataType::String => {
                if !value.is_string() {
                    errors.push(SchemaValidationError {
                        path: path.to_string(),
                        message: "Expected string".to_string(),
                        expected: Some("string".to_string()),
                        actual: Some(Self::get_type_name(value)),
                    });
                }
            }
            DataType::Integer => {
                if !value.is_i64() {
                    errors.push(SchemaValidationError {
                        path: path.to_string(),
                        message: "Expected integer".to_string(),
                        expected: Some("integer".to_string()),
                        actual: Some(Self::get_type_name(value)),
                    });
                }
            }
            DataType::Float => {
                if !value.is_f64() {
                    errors.push(SchemaValidationError {
                        path: path.to_string(),
                        message: "Expected float".to_string(),
                        expected: Some("float".to_string()),
                        actual: Some(Self::get_type_name(value)),
                    });
                }
            }
            DataType::Boolean => {
                if !value.is_boolean() {
                    errors.push(SchemaValidationError {
                        path: path.to_string(),
                        message: "Expected boolean".to_string(),
                        expected: Some("boolean".to_string()),
                        actual: Some(Self::get_type_name(value)),
                    });
                }
            }
            DataType::Array(item_type) => {
                if let Some(array) = value.as_array() {
                    for (i, item) in array.iter().enumerate() {
                        let item_path = format!("{}[{}]", path, i);
                        self.validate_value(item, item_type, &item_path, errors);
                    }
                } else {
                    errors.push(SchemaValidationError {
                        path: path.to_string(),
                        message: "Expected array".to_string(),
                        expected: Some("array".to_string()),
                        actual: Some(Self::get_type_name(value)),
                    });
                }
            }
            DataType::Object(fields) => {
                if let Some(obj) = value.as_object() {
                    // Check required fields
                    for (field_name, field_def) in fields {
                        if field_def.required && !obj.contains_key(field_name) {
                            errors.push(SchemaValidationError {
                                path: format!("{}.{}", path, field_name),
                                message: "Required field is missing".to_string(),
                                expected: Some(format!("{}", field_def.data_type)),
                                actual: Some("missing".to_string()),
                            });
                        }
                    }

                    // Validate fields
                    for (field_name, field_value) in obj {
                        if let Some(field_def) = fields.get(field_name) {
                            let field_path = format!("{}.{}", path, field_name);
                            self.validate_value(
                                field_value,
                                &field_def.data_type,
                                &field_path,
                                errors,
                            );
                        } else if !self.additional_properties {
                            errors.push(SchemaValidationError {
                                path: format!("{}.{}", path, field_name),
                                message: "Additional property not allowed".to_string(),
                                expected: None,
                                actual: None,
                            });
                        }
                    }
                } else {
                    errors.push(SchemaValidationError {
                        path: path.to_string(),
                        message: "Expected object".to_string(),
                        expected: Some("object".to_string()),
                        actual: Some(Self::get_type_name(value)),
                    });
                }
            }
            DataType::Enum(values) => {
                if let Some(s) = value.as_str() {
                    if !values.contains(&s.to_string()) {
                        errors.push(SchemaValidationError {
                            path: path.to_string(),
                            message: format!("Expected one of: {}", values.join(", ")),
                            expected: Some(format!("enum({})", values.join(", "))),
                            actual: Some(s.to_string()),
                        });
                    }
                } else {
                    errors.push(SchemaValidationError {
                        path: path.to_string(),
                        message: "Expected string for enum".to_string(),
                        expected: Some("string".to_string()),
                        actual: Some(Self::get_type_name(value)),
                    });
                }
            }
            DataType::Any => {
                // Any type is always valid
            }
            DataType::Null => {
                if !value.is_null() {
                    errors.push(SchemaValidationError {
                        path: path.to_string(),
                        message: "Expected null".to_string(),
                        expected: Some("null".to_string()),
                        actual: Some(Self::get_type_name(value)),
                    });
                }
            }
        }
    }

    /// Get the type name of a value
    fn get_type_name(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Null => "null".to_string(),
            serde_json::Value::Bool(_) => "boolean".to_string(),
            serde_json::Value::Number(n) => {
                if n.is_i64() {
                    "integer".to_string()
                } else {
                    "float".to_string()
                }
            }
            serde_json::Value::String(_) => "string".to_string(),
            serde_json::Value::Array(_) => "array".to_string(),
            serde_json::Value::Object(_) => "object".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_schema_validation() {
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
                            required: false,
                            ..Default::default()
                        },
                    ),
                    (
                        "tags".to_string(),
                        FieldDefinition {
                            data_type: DataType::Array(Box::new(DataType::String)),
                            required: false,
                            ..Default::default()
                        },
                    ),
                ]
                .into_iter()
                .collect(),
            ),
        );

        // Valid data
        let valid_data = serde_json::json!({
            "name": "John",
            "age": 30,
            "tags": ["a", "b", "c"]
        });
        assert!(schema.validate(&valid_data).is_ok());

        // Missing required field
        let missing_required = serde_json::json!({
            "age": 30,
            "tags": ["a", "b", "c"]
        });
        assert!(schema.validate(&missing_required).is_err());

        // Wrong type
        let wrong_type = serde_json::json!({
            "name": "John",
            "age": "30",
            "tags": ["a", "b", "c"]
        });
        assert!(schema.validate(&wrong_type).is_err());

        // Wrong array item type
        let wrong_array_item = serde_json::json!({
            "name": "John",
            "age": 30,
            "tags": ["a", "b", 3]
        });
        assert!(schema.validate(&wrong_array_item).is_err());
    }
}
