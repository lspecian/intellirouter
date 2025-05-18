//! Condition definitions
//!
//! This file defines the condition structures and related types for conditional execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Comparison operators for conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonOperator {
    Eq,
    Ne,
    Lt,
    Lte,
    Gt,
    Gte,
    Contains,
    StartsWith,
    EndsWith,
    Matches,
}

/// Represents a condition for conditional execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum Condition {
    // Check if a variable equals a value
    Equals {
        variable: String,
        value: serde_json::Value,
    },

    // Check if a variable contains a value
    Contains {
        variable: String,
        value: serde_json::Value,
    },

    // Check if a variable matches a regex pattern
    Regex {
        variable: String,
        pattern: String,
    },

    // Check if a variable is greater than a value
    GreaterThan {
        variable: String,
        value: serde_json::Value,
    },

    // Check if a variable is less than a value
    LessThan {
        variable: String,
        value: serde_json::Value,
    },

    // General comparison with operator
    Comparison {
        left: String,
        operator: ComparisonOperator,
        right: String,
    },

    // Expression-based condition
    Expression {
        expression: String,
    },

    // Check if all conditions are true
    And {
        conditions: Vec<Condition>,
    },

    // Check if any condition is true
    Or {
        conditions: Vec<Condition>,
    },

    // Negate a condition
    Not {
        condition: Box<Condition>,
    },

    // Custom condition (extensibility)
    Custom {
        evaluator: String,
        #[serde(default)]
        params: HashMap<String, serde_json::Value>,
    },
}

/// Represents an error handler for a step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum ErrorHandler {
    // Continue execution with a default value
    ContinueWithDefault {
        default_value: serde_json::Value,
    },

    // Retry the step with different parameters
    RetryWithDifferentParams {
        params: HashMap<String, serde_json::Value>,
    },

    // Execute a fallback step
    ExecuteFallbackStep {
        step_id: String,
    },

    // Custom error handler (extensibility)
    Custom {
        handler: String,
        #[serde(default)]
        config: HashMap<String, serde_json::Value>,
    },
}

/// Error handling strategies for the chain
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorHandlingStrategy {
    // Stop execution on the first error
    StopOnError,

    // Continue execution and collect errors
    ContinueOnError,

    // Retry the chain with different parameters
    RetryWithDifferentParams {
        max_retries: u32,
        params: HashMap<String, serde_json::Value>,
    },
}

impl Default for ErrorHandlingStrategy {
    fn default() -> Self {
        ErrorHandlingStrategy::StopOnError
    }
}
