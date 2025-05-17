//! Chain definition types
//!
//! This module defines the data structures for chain definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Represents a complete chain definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chain {
    // Chain metadata
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,

    // Chain components
    pub steps: HashMap<String, ChainStep>,
    #[serde(default)]
    pub dependencies: Vec<StepDependency>,
    #[serde(default)]
    pub variables: HashMap<String, Variable>,

    // Execution settings
    #[serde(default)]
    pub error_handling: ErrorHandlingStrategy,
    #[serde(default)]
    pub max_parallel_steps: Option<usize>,
    #[serde(with = "duration_serde", default)]
    pub timeout: Option<Duration>,
}

/// Represents a single step in a chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStep {
    pub id: String,
    pub name: String,
    pub description: String,

    // Step type and configuration
    pub step_type: StepType,
    pub role: Role,

    // Input/output configuration
    #[serde(default)]
    pub inputs: Vec<InputMapping>,
    #[serde(default)]
    pub outputs: Vec<OutputMapping>,

    // Execution settings
    #[serde(default)]
    pub condition: Option<Condition>,
    #[serde(default)]
    pub retry_policy: Option<RetryPolicy>,
    #[serde(with = "duration_serde", default)]
    pub timeout: Option<Duration>,
    #[serde(default)]
    pub error_handler: Option<ErrorHandler>,
}

/// Types of steps that can be executed in a chain
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum StepType {
    // LLM inference step
    LLMInference {
        model: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        top_p: Option<f32>,
        #[serde(default)]
        stop_sequences: Vec<String>,
        #[serde(default)]
        additional_params: HashMap<String, serde_json::Value>,
    },

    // Function call step
    FunctionCall {
        function_name: String,
        #[serde(default)]
        arguments: HashMap<String, serde_json::Value>,
    },

    // Tool use step
    ToolUse {
        tool_name: String,
        #[serde(default)]
        arguments: HashMap<String, serde_json::Value>,
    },

    // Conditional branching
    Conditional {
        branches: Vec<ConditionalBranch>,
        default_branch: Option<String>,
    },

    // Parallel execution
    Parallel {
        steps: Vec<String>,
        wait_for_all: bool,
    },

    // Loop execution
    Loop {
        iteration_variable: String,
        max_iterations: Option<u32>,
        steps: Vec<String>,
        break_condition: Option<Condition>,
    },

    // Custom step type (extensibility)
    Custom {
        handler: String,
        #[serde(default)]
        config: HashMap<String, serde_json::Value>,
    },
}

/// Roles that can be assigned to steps
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
    Function,
    Tool,
    Custom(String),
}

impl Default for Role {
    fn default() -> Self {
        Role::Assistant
    }
}

/// Represents a dependency between steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDependency {
    pub dependent_step: String,
    pub dependency_type: DependencyType,
}

/// Types of dependencies between steps
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum DependencyType {
    // Simple dependency (one step depends on another)
    Simple {
        required_step: String,
    },

    // All dependency (one step depends on all of the specified steps)
    All {
        required_steps: Vec<String>,
    },

    // Any dependency (one step depends on any of the specified steps)
    Any {
        required_steps: Vec<String>,
    },

    // Conditional dependency (one step depends on another if a condition is met)
    Conditional {
        required_step: String,
        condition: Condition,
    },
}

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

/// Represents a conditional branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalBranch {
    pub condition: Condition,
    pub target_step: String,
}

/// Represents a retry policy for a step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    #[serde(with = "duration_serde")]
    pub retry_interval: Option<Duration>,
    pub retry_backoff_factor: f32,
    #[serde(default)]
    pub retry_on_error_codes: Vec<String>,
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

/// Module for serializing/deserializing Duration
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match duration {
            Some(duration) => serializer.serialize_u64(duration.as_secs()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = Option::<u64>::deserialize(deserializer)?;
        Ok(seconds.map(Duration::from_secs))
    }
}
