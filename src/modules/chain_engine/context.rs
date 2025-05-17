//! Chain execution context
//!
//! This module provides the context and result types for chain execution.

use std::collections::HashMap;
use std::time::Duration;

/// Result of a step execution
#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_id: String,
    pub outputs: HashMap<String, serde_json::Value>,
    pub error: Option<String>,
    pub execution_time: Duration,
}

/// Context for chain execution
#[derive(Debug, Clone)]
pub struct ChainContext {
    pub chain_id: String,
    pub variables: HashMap<String, serde_json::Value>,
    pub step_results: HashMap<String, StepResult>,
    pub inputs: HashMap<String, serde_json::Value>,
    pub outputs: HashMap<String, serde_json::Value>,
}
