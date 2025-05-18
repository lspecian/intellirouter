//! Data definitions
//!
//! This file defines the data structures for variables, input/output mappings,
//! and data transformations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a variable in a chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub description: String,
    pub data_type: DataType,
    pub initial_value: Option<serde_json::Value>,
    pub required: bool,
}

/// Data types for variables
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Any,
}

/// Represents an input mapping for a step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMapping {
    pub name: String,
    pub source: DataSource,
    #[serde(default)]
    pub transform: Option<DataTransform>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default_value: Option<serde_json::Value>,
}

/// Represents an output mapping for a step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputMapping {
    pub name: String,
    pub target: DataTarget,
    #[serde(default)]
    pub transform: Option<DataTransform>,
}

/// Data sources for input mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum DataSource {
    // Chain input
    ChainInput {
        input_name: String,
    },

    // Variable
    Variable {
        variable_name: String,
    },

    // Step output
    StepOutput {
        step_id: String,
        output_name: String,
    },

    // Literal value
    Literal {
        value: serde_json::Value,
    },

    // Template (with variable substitution)
    Template {
        template: String,
    },
}

/// Data targets for output mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum DataTarget {
    // Chain output
    ChainOutput { output_name: String },

    // Variable
    Variable { variable_name: String },

    // Step input
    StepInput { step_id: String, input_name: String },
}

/// Data transformations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum DataTransform {
    // Extract a JSON path
    JsonPath {
        path: String,
    },

    // Extract a regex pattern
    Regex {
        pattern: String,
        group: Option<usize>,
    },

    // Apply a template
    Template {
        template: String,
    },

    // Map values
    Map {
        mappings: HashMap<String, serde_json::Value>,
        default: Option<serde_json::Value>,
    },

    // Custom transformation (extensibility)
    Custom {
        handler: String,
        #[serde(default)]
        config: HashMap<String, serde_json::Value>,
    },
}
