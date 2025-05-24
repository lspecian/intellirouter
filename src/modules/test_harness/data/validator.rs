//! Data Validator Module
//!
//! This module provides functionality for validating test data.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use super::schema::{DataSchema, SchemaValidationError};

/// Validation error
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error path
    pub path: String,
    /// Error message
    pub message: String,
    /// Error severity
    pub severity: ValidationSeverity,
}

/// Validation severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Error
    Error,
    /// Warning
    Warning,
    /// Info
    Info,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the validation passed
    pub passed: bool,
    /// Validation errors
    pub errors: Vec<ValidationError>,
    /// Validation warnings
    pub warnings: Vec<ValidationError>,
    /// Validation info
    pub info: Vec<ValidationError>,
    /// Validation metadata
    pub metadata: HashMap<String, String>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new(passed: bool) -> Self {
        Self {
            passed,
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create a successful validation result
    pub fn success() -> Self {
        Self::new(true)
    }

    /// Create a failed validation result
    pub fn failure() -> Self {
        Self::new(false)
    }

    /// Add an error
    pub fn with_error(mut self, path: impl Into<String>, message: impl Into<String>) -> Self {
        self.errors.push(ValidationError {
            path: path.into(),
            message: message.into(),
            severity: ValidationSeverity::Error,
        });
        self.passed = false;
        self
    }

    /// Add a warning
    pub fn with_warning(mut self, path: impl Into<String>, message: impl Into<String>) -> Self {
        self.warnings.push(ValidationError {
            path: path.into(),
            message: message.into(),
            severity: ValidationSeverity::Warning,
        });
        self
    }

    /// Add an info
    pub fn with_info(mut self, path: impl Into<String>, message: impl Into<String>) -> Self {
        self.info.push(ValidationError {
            path: path.into(),
            message: message.into(),
            severity: ValidationSeverity::Info,
        });
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add multiple errors
    pub fn with_errors(mut self, errors: impl IntoIterator<Item = ValidationError>) -> Self {
        let errors: Vec<_> = errors.into_iter().collect();
        if !errors.is_empty() {
            self.passed = false;
            self.errors.extend(errors);
        }
        self
    }

    /// Add multiple warnings
    pub fn with_warnings(mut self, warnings: impl IntoIterator<Item = ValidationError>) -> Self {
        self.warnings.extend(warnings);
        self
    }

    /// Add multiple info
    pub fn with_info_items(mut self, info: impl IntoIterator<Item = ValidationError>) -> Self {
        self.info.extend(info);
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

    /// Merge with another validation result
    pub fn merge(&mut self, other: ValidationResult) {
        self.passed = self.passed && other.passed;
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.info.extend(other.info);
        self.metadata.extend(other.metadata);
    }

    /// Get all validation errors
    pub fn all_errors(&self) -> Vec<&ValidationError> {
        let mut errors = Vec::new();
        errors.extend(self.errors.iter());
        errors.extend(self.warnings.iter());
        errors.extend(self.info.iter());
        errors
    }

    /// Get the error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get the warning count
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Get the info count
    pub fn info_count(&self) -> usize {
        self.info.len()
    }

    /// Get the total error count
    pub fn total_count(&self) -> usize {
        self.error_count() + self.warning_count() + self.info_count()
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Check if there are any info
    pub fn has_info(&self) -> bool {
        !self.info.is_empty()
    }
}

/// Data validator trait
#[async_trait]
pub trait DataValidator: Send + Sync {
    /// Get the validator name
    fn name(&self) -> &str;

    /// Get the validator description
    fn description(&self) -> Option<&str>;

    /// Validate data
    async fn validate(
        &self,
        data: &serde_json::Value,
        schema: Option<&DataSchema>,
    ) -> Result<ValidationResult, String>;
}

/// Schema validator
pub struct SchemaValidator {
    /// Validator name
    name: String,
    /// Validator description
    description: Option<String>,
}

impl SchemaValidator {
    /// Create a new schema validator
    pub fn new() -> Self {
        Self {
            name: "schema".to_string(),
            description: Some("Schema validator".to_string()),
        }
    }

    /// Create a new schema validator with a custom name
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: Some("Schema validator".to_string()),
        }
    }

    /// Create a new schema validator with a custom name and description
    pub fn with_name_and_description(
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
        }
    }
}

#[async_trait]
impl DataValidator for SchemaValidator {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    async fn validate(
        &self,
        data: &serde_json::Value,
        schema: Option<&DataSchema>,
    ) -> Result<ValidationResult, String> {
        // Check if a schema is provided
        let schema = schema.ok_or_else(|| "Schema is required".to_string())?;

        // Validate the data against the schema
        match schema.validate(data) {
            Ok(()) => {
                // Validation passed
                let mut result = ValidationResult::success();
                result
                    .metadata
                    .insert("validator".to_string(), self.name().to_string());
                result
                    .metadata
                    .insert("schema".to_string(), schema.name.clone());
                Ok(result)
            }
            Err(errors) => {
                // Validation failed
                let mut result = ValidationResult::failure();
                result
                    .metadata
                    .insert("validator".to_string(), self.name().to_string());
                result
                    .metadata
                    .insert("schema".to_string(), schema.name.clone());

                // Convert schema validation errors to validation errors
                for error in errors {
                    result.errors.push(ValidationError {
                        path: error.path,
                        message: error.message,
                        severity: ValidationSeverity::Error,
                    });
                }

                Ok(result)
            }
        }
    }
}

/// JSON schema validator
pub struct JsonSchemaValidator {
    /// Validator name
    name: String,
    /// Validator description
    description: Option<String>,
}

impl JsonSchemaValidator {
    /// Create a new JSON schema validator
    pub fn new() -> Self {
        Self {
            name: "json-schema".to_string(),
            description: Some("JSON schema validator".to_string()),
        }
    }

    /// Create a new JSON schema validator with a custom name
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: Some("JSON schema validator".to_string()),
        }
    }

    /// Create a new JSON schema validator with a custom name and description
    pub fn with_name_and_description(
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
        }
    }
}

#[async_trait]
impl DataValidator for JsonSchemaValidator {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    async fn validate(
        &self,
        data: &serde_json::Value,
        schema: Option<&DataSchema>,
    ) -> Result<ValidationResult, String> {
        // Check if a schema is provided
        let _schema = schema.ok_or_else(|| "Schema is required".to_string())?;

        // TODO: Implement JSON schema validation
        // For now, just return a success result
        let mut result = ValidationResult::success();
        result
            .metadata
            .insert("validator".to_string(), self.name().to_string());
        result
            .metadata
            .insert("schema".to_string(), _schema.name.clone());
        result.metadata.insert(
            "note".to_string(),
            "JSON schema validation not implemented yet".to_string(),
        );

        Ok(result)
    }
}

/// Custom validator
pub struct CustomValidator {
    /// Validator name
    name: String,
    /// Validator description
    description: Option<String>,
    /// Validation function
    validation_fn: Box<dyn Fn(&serde_json::Value) -> ValidationResult + Send + Sync>,
}

impl CustomValidator {
    /// Create a new custom validator
    pub fn new(
        name: impl Into<String>,
        validation_fn: impl Fn(&serde_json::Value) -> ValidationResult + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            description: None,
            validation_fn: Box::new(validation_fn),
        }
    }

    /// Create a new custom validator with a description
    pub fn with_description(
        name: impl Into<String>,
        description: impl Into<String>,
        validation_fn: impl Fn(&serde_json::Value) -> ValidationResult + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
            validation_fn: Box::new(validation_fn),
        }
    }
}

#[async_trait]
impl DataValidator for CustomValidator {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    async fn validate(
        &self,
        data: &serde_json::Value,
        _schema: Option<&DataSchema>,
    ) -> Result<ValidationResult, String> {
        // Call the validation function
        let mut result = (self.validation_fn)(data);

        // Add metadata
        result
            .metadata
            .insert("validator".to_string(), self.name().to_string());

        Ok(result)
    }
}

/// Composite validator
pub struct CompositeValidator {
    /// Validator name
    name: String,
    /// Validator description
    description: Option<String>,
    /// Validators
    validators: Vec<Arc<dyn DataValidator>>,
}

impl CompositeValidator {
    /// Create a new composite validator
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: Some("Composite validator".to_string()),
            validators: Vec::new(),
        }
    }

    /// Create a new composite validator with a description
    pub fn with_description(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
            validators: Vec::new(),
        }
    }

    /// Add a validator
    pub fn add_validator(&mut self, validator: Arc<dyn DataValidator>) {
        self.validators.push(validator);
    }

    /// Add multiple validators
    pub fn add_validators(&mut self, validators: impl IntoIterator<Item = Arc<dyn DataValidator>>) {
        self.validators.extend(validators);
    }

    /// Create a new composite validator with validators
    pub fn with_validators(
        name: impl Into<String>,
        validators: impl IntoIterator<Item = Arc<dyn DataValidator>>,
    ) -> Self {
        Self {
            name: name.into(),
            description: Some("Composite validator".to_string()),
            validators: validators.into_iter().collect(),
        }
    }
}

#[async_trait]
impl DataValidator for CompositeValidator {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    async fn validate(
        &self,
        data: &serde_json::Value,
        schema: Option<&DataSchema>,
    ) -> Result<ValidationResult, String> {
        // Create a result
        let mut result = ValidationResult::success();
        result
            .metadata
            .insert("validator".to_string(), self.name().to_string());

        // Validate with each validator
        for validator in &self.validators {
            match validator.validate(data, schema).await {
                Ok(validator_result) => {
                    // Merge the results
                    result.merge(validator_result);
                }
                Err(e) => {
                    // Add an error
                    result = result.with_error(
                        "$",
                        format!("Validator '{}' failed: {}", validator.name(), e),
                    );
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::super::schema::{DataSchema, DataType, FieldDefinition};
    use super::*;

    #[tokio::test]
    async fn test_schema_validator() {
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
                ]
                .into_iter()
                .collect(),
            ),
        );

        // Create a validator
        let validator = SchemaValidator::new();

        // Valid data
        let valid_data = serde_json::json!({
            "name": "John",
            "age": 30
        });
        let result = validator
            .validate(&valid_data, Some(&schema))
            .await
            .unwrap();
        assert!(result.passed);
        assert_eq!(result.error_count(), 0);

        // Invalid data
        let invalid_data = serde_json::json!({
            "age": 30
        });
        let result = validator
            .validate(&invalid_data, Some(&schema))
            .await
            .unwrap();
        assert!(!result.passed);
        assert_eq!(result.error_count(), 1);
    }

    #[tokio::test]
    async fn test_custom_validator() {
        // Create a validator
        let validator = CustomValidator::new("test", |data| {
            let mut result = ValidationResult::success();

            // Check if the data is an object
            if !data.is_object() {
                return result.with_error("$", "Data must be an object");
            }

            // Check if the object has a name field
            if let Some(obj) = data.as_object() {
                if !obj.contains_key("name") {
                    result = result.with_error("$", "Object must have a name field");
                }
            }

            result
        });

        // Valid data
        let valid_data = serde_json::json!({
            "name": "John",
            "age": 30
        });
        let result = validator.validate(&valid_data, None).await.unwrap();
        assert!(result.passed);
        assert_eq!(result.error_count(), 0);

        // Invalid data
        let invalid_data = serde_json::json!({
            "age": 30
        });
        let result = validator.validate(&invalid_data, None).await.unwrap();
        assert!(!result.passed);
        assert_eq!(result.error_count(), 1);
    }

    #[tokio::test]
    async fn test_composite_validator() {
        // Create validators
        let schema_validator = Arc::new(SchemaValidator::new());
        let custom_validator = Arc::new(CustomValidator::new("test", |data| {
            let mut result = ValidationResult::success();

            // Check if the age is positive
            if let Some(age) = data.get("age").and_then(|v| v.as_i64()) {
                if age <= 0 {
                    result = result.with_error("$.age", "Age must be positive");
                }
            }

            result
        }));

        // Create a composite validator
        let mut composite_validator = CompositeValidator::new("composite");
        composite_validator.add_validator(schema_validator);
        composite_validator.add_validator(custom_validator);

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
                ]
                .into_iter()
                .collect(),
            ),
        );

        // Valid data
        let valid_data = serde_json::json!({
            "name": "John",
            "age": 30
        });
        let result = composite_validator
            .validate(&valid_data, Some(&schema))
            .await
            .unwrap();
        assert!(result.passed);
        assert_eq!(result.error_count(), 0);

        // Invalid data (missing name)
        let invalid_data1 = serde_json::json!({
            "age": 30
        });
        let result = composite_validator
            .validate(&invalid_data1, Some(&schema))
            .await
            .unwrap();
        assert!(!result.passed);
        assert_eq!(result.error_count(), 1);

        // Invalid data (negative age)
        let invalid_data2 = serde_json::json!({
            "name": "John",
            "age": -5
        });
        let result = composite_validator
            .validate(&invalid_data2, Some(&schema))
            .await
            .unwrap();
        assert!(!result.passed);
        assert_eq!(result.error_count(), 1);

        // Invalid data (missing name and negative age)
        let invalid_data3 = serde_json::json!({
            "age": -5
        });
        let result = composite_validator
            .validate(&invalid_data3, Some(&schema))
            .await
            .unwrap();
        assert!(!result.passed);
        assert_eq!(result.error_count(), 2);
    }
}
