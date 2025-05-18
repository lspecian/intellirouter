//! Step definitions
//!
//! This file defines the step structures and related types for chain execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use super::condition::{Condition, ErrorHandler};
use super::data::{InputMapping, OutputMapping};
use super::utils::duration_serde;

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
