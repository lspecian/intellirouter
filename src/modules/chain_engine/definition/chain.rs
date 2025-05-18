//! Chain structure
//!
//! This file defines the main Chain structure and its components.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use super::condition::ErrorHandlingStrategy;
use super::data::Variable;
use super::step::{ChainStep, StepDependency};
use super::utils::duration_serde;

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
