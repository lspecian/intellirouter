//! Validators for assertions
//!
//! This module provides validators for validating values in assertions.

use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the value is valid
    pub is_valid: bool,
    /// Expected value
    pub expected: Option<String>,
    /// Actual value
    pub actual: Option<String>,
    /// Validation details
    pub details: Option<String>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new(is_valid: bool) -> Self {
        Self {
            is_valid,
            expected: None,
            actual: None,
            details: None,
        }
    }

    /// Set the expected value
    pub fn with_expected(mut self, expected: impl fmt::Display) -> Self {
        self.expected = Some(expected.to_string());
        self
    }

    /// Set the actual value
    pub fn with_actual(mut self, actual: impl fmt::Display) -> Self {
        self.actual = Some(actual.to_string());
        self
    }

    /// Set the validation details
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

/// Validation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOptions {
    /// Whether to ignore case when validating strings
    pub ignore_case: bool,
    /// Whether to ignore whitespace when validating strings
    pub ignore_whitespace: bool,
    /// Whether to ignore order when validating arrays
    pub ignore_order: bool,
    /// Whether to ignore extra fields when validating objects
    pub ignore_extra_fields: bool,
    /// Whether to ignore missing fields when validating objects
    pub ignore_missing_fields: bool,
    /// Custom options
    pub custom_options: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for ValidationOptions {
    fn default() -> Self {
        Self {
            ignore_case: false,
            ignore_whitespace: false,
            ignore_order: false,
            ignore_extra_fields: false,
            ignore_missing_fields: false,
            custom_options: std::collections::HashMap::new(),
        }
    }
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error message
    pub message: String,
    /// Error path
    pub path: Option<String>,
    /// Error details
    pub details: Option<String>,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            path: None,
            details: None,
        }
    }

    /// Set the error path
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set the error details
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

/// Validator trait for validating values
pub trait Validator: Send + Sync {
    /// Validate a value
    fn validate(&self, value: &dyn std::any::Any) -> ValidationResult;
}

/// Validate trait for types that can be validated
pub trait Validate {
    /// Validate the value
    fn validate(&self, options: &ValidationOptions) -> ValidationResult;
}

/// JSON validator
pub struct JsonValidator {
    /// JSON schema
    schema: serde_json::Value,
    /// Validation options
    options: ValidationOptions,
}

impl JsonValidator {
    /// Create a new JSON validator
    pub fn new(schema: serde_json::Value) -> Self {
        Self {
            schema,
            options: ValidationOptions::default(),
        }
    }

    /// Create a new JSON validator with options
    pub fn with_options(schema: serde_json::Value, options: ValidationOptions) -> Self {
        Self { schema, options }
    }
}

impl Validator for JsonValidator {
    fn validate(&self, value: &dyn std::any::Any) -> ValidationResult {
        // Try to downcast to JSON value
        if let Some(json_value) = value.downcast_ref::<serde_json::Value>() {
            return self.validate_json(json_value);
        }

        // Try to convert to JSON
        let json_value = match serde_json::to_value(value) {
            Ok(json) => json,
            Err(_) => {
                return ValidationResult::new(false)
                    .with_actual(format!("{:?}", value))
                    .with_expected(format!("JSON value matching schema: {}", self.schema))
                    .with_details("Failed to convert value to JSON".to_string());
            }
        };

        self.validate_json(&json_value)
    }
}

impl JsonValidator {
    /// Validate a JSON value against the schema
    fn validate_json(&self, value: &serde_json::Value) -> ValidationResult {
        // Use jsonschema crate for validation
        let compiled_schema = match jsonschema::JSONSchema::compile(&self.schema) {
            Ok(schema) => schema,
            Err(err) => {
                return ValidationResult::new(false)
                    .with_actual(value.to_string())
                    .with_expected(format!("JSON value matching schema: {}", self.schema))
                    .with_details(format!("Invalid schema: {}", err));
            }
        };

        // Validate the value against the schema
        let validation_result = compiled_schema.validate(value);
        match validation_result {
            Ok(()) => ValidationResult::new(true)
                .with_actual(value.to_string())
                .with_expected(format!("JSON value matching schema: {}", self.schema)),
            Err(errors) => {
                let error_details = errors
                    .map(|err| format!("{}: {}", err.instance_path, err.to_string()))
                    .collect::<Vec<_>>()
                    .join(", ");

                ValidationResult::new(false)
                    .with_actual(value.to_string())
                    .with_expected(format!("JSON value matching schema: {}", self.schema))
                    .with_details(error_details)
            }
        }
    }
}

/// Schema validator
pub struct SchemaValidator {
    /// Schema
    schema: serde_json::Value,
    /// Validation options
    options: ValidationOptions,
}

impl SchemaValidator {
    /// Create a new schema validator
    pub fn new(schema: serde_json::Value) -> Self {
        Self {
            schema,
            options: ValidationOptions::default(),
        }
    }

    /// Create a new schema validator with options
    pub fn with_options(schema: serde_json::Value, options: ValidationOptions) -> Self {
        Self { schema, options }
    }
}

impl Validator for SchemaValidator {
    fn validate(&self, value: &dyn std::any::Any) -> ValidationResult {
        // Try to convert to JSON
        let json_value = match serde_json::to_value(value) {
            Ok(json) => json,
            Err(_) => {
                return ValidationResult::new(false)
                    .with_actual(format!("{:?}", value))
                    .with_expected(format!("Value matching schema: {}", self.schema))
                    .with_details("Failed to convert value to JSON".to_string());
            }
        };

        // Use jsonschema crate for validation
        let compiled_schema = match jsonschema::JSONSchema::compile(&self.schema) {
            Ok(schema) => schema,
            Err(err) => {
                return ValidationResult::new(false)
                    .with_actual(json_value.to_string())
                    .with_expected(format!("Value matching schema: {}", self.schema))
                    .with_details(format!("Invalid schema: {}", err));
            }
        };

        // Validate the value against the schema
        let validation_result = compiled_schema.validate(&json_value);
        match validation_result {
            Ok(()) => ValidationResult::new(true)
                .with_actual(json_value.to_string())
                .with_expected(format!("Value matching schema: {}", self.schema)),
            Err(errors) => {
                let error_details = errors
                    .map(|err| format!("{}: {}", err.instance_path, err.to_string()))
                    .collect::<Vec<_>>()
                    .join(", ");

                ValidationResult::new(false)
                    .with_actual(json_value.to_string())
                    .with_expected(format!("Value matching schema: {}", self.schema))
                    .with_details(error_details)
            }
        }
    }
}

/// Struct validator
pub struct StructValidator<T> {
    /// Validation function
    validate_fn: Box<dyn Fn(&T) -> ValidationResult + Send + Sync>,
}

impl<T: 'static> StructValidator<T> {
    /// Create a new struct validator
    pub fn new(validate_fn: impl Fn(&T) -> ValidationResult + Send + Sync + 'static) -> Self {
        Self {
            validate_fn: Box::new(validate_fn),
        }
    }
}

impl<T: 'static> Validator for StructValidator<T> {
    fn validate(&self, value: &dyn std::any::Any) -> ValidationResult {
        // Try to downcast to the expected type
        if let Some(typed_value) = value.downcast_ref::<T>() {
            (self.validate_fn)(typed_value)
        } else {
            ValidationResult::new(false)
                .with_actual(format!("{:?}", value))
                .with_expected(format!("Value of type {}", std::any::type_name::<T>()))
                .with_details(format!(
                    "Value is not of type {}",
                    std::any::type_name::<T>()
                ))
        }
    }
}
