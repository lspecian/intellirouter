//! Chain assertions for the assertion framework.
//!
//! This module provides assertions specific to the chain component of IntelliRouter.

use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::modules::test_harness::assert::core::{
    assert_that, AssertThat, AssertionOutcome, AssertionResult,
};
use crate::modules::test_harness::types::TestHarnessError;

/// Represents a chain node for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainNode {
    /// The node ID.
    pub node_id: String,
    /// The node type.
    pub node_type: String,
    /// The node name.
    pub name: String,
    /// The node description.
    pub description: String,
    /// The node parameters.
    pub parameters: Value,
    /// The node metadata.
    pub metadata: Value,
}

impl ChainNode {
    /// Creates a new chain node.
    pub fn new(node_id: &str, node_type: &str, name: &str) -> Self {
        Self {
            node_id: node_id.to_string(),
            node_type: node_type.to_string(),
            name: name.to_string(),
            description: "".to_string(),
            parameters: Value::Null,
            metadata: Value::Null,
        }
    }

    /// Sets the node description.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Sets the node parameters.
    pub fn with_parameters(mut self, parameters: Value) -> Self {
        self.parameters = parameters;
        self
    }

    /// Sets the node metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Represents a chain for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chain {
    /// The chain ID.
    pub chain_id: String,
    /// The chain name.
    pub name: String,
    /// The chain description.
    pub description: String,
    /// The chain nodes.
    pub nodes: Vec<ChainNode>,
    /// The chain edges.
    pub edges: Vec<(String, String)>,
    /// The chain parameters.
    pub parameters: Value,
    /// The chain metadata.
    pub metadata: Value,
}

impl Chain {
    /// Creates a new chain.
    pub fn new(chain_id: &str, name: &str) -> Self {
        Self {
            chain_id: chain_id.to_string(),
            name: name.to_string(),
            description: "".to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
            parameters: Value::Null,
            metadata: Value::Null,
        }
    }

    /// Sets the chain description.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Adds a node to the chain.
    pub fn with_node(mut self, node: ChainNode) -> Self {
        self.nodes.push(node);
        self
    }

    /// Adds an edge to the chain.
    pub fn with_edge(mut self, from_node_id: &str, to_node_id: &str) -> Self {
        self.edges
            .push((from_node_id.to_string(), to_node_id.to_string()));
        self
    }

    /// Sets the chain parameters.
    pub fn with_parameters(mut self, parameters: Value) -> Self {
        self.parameters = parameters;
        self
    }

    /// Sets the chain metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Gets a node by ID.
    pub fn get_node(&self, node_id: &str) -> Option<&ChainNode> {
        self.nodes.iter().find(|n| n.node_id == node_id)
    }

    /// Gets the outgoing edges from a node.
    pub fn get_outgoing_edges(&self, node_id: &str) -> Vec<&(String, String)> {
        self.edges
            .iter()
            .filter(|(from, _)| from == node_id)
            .collect()
    }

    /// Gets the incoming edges to a node.
    pub fn get_incoming_edges(&self, node_id: &str) -> Vec<&(String, String)> {
        self.edges.iter().filter(|(_, to)| to == node_id).collect()
    }
}

/// Represents a chain execution step for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainExecutionStep {
    /// The step ID.
    pub step_id: String,
    /// The node ID.
    pub node_id: String,
    /// The step input.
    pub input: Value,
    /// The step output.
    pub output: Value,
    /// The step error, if any.
    pub error: Option<String>,
    /// The step start time.
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// The step end time.
    pub end_time: chrono::DateTime<chrono::Utc>,
    /// The step duration.
    pub duration: Duration,
    /// The step metadata.
    pub metadata: Value,
}

impl ChainExecutionStep {
    /// Creates a new chain execution step.
    pub fn new(step_id: &str, node_id: &str) -> Self {
        let now = chrono::Utc::now();
        Self {
            step_id: step_id.to_string(),
            node_id: node_id.to_string(),
            input: Value::Null,
            output: Value::Null,
            error: None,
            start_time: now,
            end_time: now,
            duration: Duration::default(),
            metadata: Value::Null,
        }
    }

    /// Sets the step input.
    pub fn with_input(mut self, input: Value) -> Self {
        self.input = input;
        self
    }

    /// Sets the step output.
    pub fn with_output(mut self, output: Value) -> Self {
        self.output = output;
        self
    }

    /// Sets the step error.
    pub fn with_error(mut self, error: &str) -> Self {
        self.error = Some(error.to_string());
        self
    }

    /// Sets the step start time.
    pub fn with_start_time(mut self, start_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.start_time = start_time;
        self
    }

    /// Sets the step end time.
    pub fn with_end_time(mut self, end_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.end_time = end_time;
        self
    }

    /// Sets the step duration.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the step metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Represents a chain execution for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainExecution {
    /// The execution ID.
    pub execution_id: String,
    /// The chain ID.
    pub chain_id: String,
    /// The execution input.
    pub input: Value,
    /// The execution output.
    pub output: Value,
    /// The execution steps.
    pub steps: Vec<ChainExecutionStep>,
    /// The execution error, if any.
    pub error: Option<String>,
    /// The execution start time.
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// The execution end time.
    pub end_time: chrono::DateTime<chrono::Utc>,
    /// The execution duration.
    pub duration: Duration,
    /// The execution metadata.
    pub metadata: Value,
}

impl ChainExecution {
    /// Creates a new chain execution.
    pub fn new(execution_id: &str, chain_id: &str) -> Self {
        let now = chrono::Utc::now();
        Self {
            execution_id: execution_id.to_string(),
            chain_id: chain_id.to_string(),
            input: Value::Null,
            output: Value::Null,
            steps: Vec::new(),
            error: None,
            start_time: now,
            end_time: now,
            duration: Duration::default(),
            metadata: Value::Null,
        }
    }

    /// Sets the execution input.
    pub fn with_input(mut self, input: Value) -> Self {
        self.input = input;
        self
    }

    /// Sets the execution output.
    pub fn with_output(mut self, output: Value) -> Self {
        self.output = output;
        self
    }

    /// Adds a step to the execution.
    pub fn with_step(mut self, step: ChainExecutionStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Sets the execution error.
    pub fn with_error(mut self, error: &str) -> Self {
        self.error = Some(error.to_string());
        self
    }

    /// Sets the execution start time.
    pub fn with_start_time(mut self, start_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.start_time = start_time;
        self
    }

    /// Sets the execution end time.
    pub fn with_end_time(mut self, end_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.end_time = end_time;
        self
    }

    /// Sets the execution duration.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the execution metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Gets a step by ID.
    pub fn get_step(&self, step_id: &str) -> Option<&ChainExecutionStep> {
        self.steps.iter().find(|s| s.step_id == step_id)
    }

    /// Gets a step by node ID.
    pub fn get_step_by_node_id(&self, node_id: &str) -> Option<&ChainExecutionStep> {
        self.steps.iter().find(|s| s.node_id == node_id)
    }
}

/// Assertions for chain components.
#[derive(Debug, Clone)]
pub struct ChainAssertions;

impl ChainAssertions {
    /// Creates a new chain assertions instance.
    pub fn new() -> Self {
        Self
    }

    /// Asserts that a chain has a specific number of nodes.
    pub fn assert_node_count(&self, chain: &Chain, expected: usize) -> AssertionResult {
        let count = chain.nodes.len();
        if count == expected {
            AssertionResult::new(
                &format!("Node count is {}", expected),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Node count is {}", expected),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Node count does not match expected value",
                    &format!("{}", expected),
                    &format!("{}", count),
                ),
            )
        }
    }

    /// Asserts that a chain has a node with a specific ID.
    pub fn assert_has_node(&self, chain: &Chain, node_id: &str) -> AssertionResult {
        match chain.get_node(node_id) {
            Some(_) => AssertionResult::new(
                &format!("Chain has node '{}'", node_id),
                AssertionOutcome::Passed,
            ),
            None => AssertionResult::new(
                &format!("Chain has node '{}'", node_id),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Chain does not have node '{}'", node_id),
                    &format!("Node '{}'", node_id),
                    "No such node",
                ),
            ),
        }
    }

    /// Asserts that a chain has a node with a specific type.
    pub fn assert_has_node_type(&self, chain: &Chain, node_type: &str) -> AssertionResult {
        let has_node = chain.nodes.iter().any(|n| n.node_type == node_type);
        if has_node {
            AssertionResult::new(
                &format!("Chain has node of type '{}'", node_type),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Chain has node of type '{}'", node_type),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Chain does not have node of type '{}'", node_type),
                    &format!("Node of type '{}'", node_type),
                    "No such node",
                ),
            )
        }
    }

    /// Asserts that a chain has an edge between two nodes.
    pub fn assert_has_edge(
        &self,
        chain: &Chain,
        from_node_id: &str,
        to_node_id: &str,
    ) -> AssertionResult {
        let has_edge = chain
            .edges
            .iter()
            .any(|(from, to)| from == from_node_id && to == to_node_id);
        if has_edge {
            AssertionResult::new(
                &format!("Chain has edge from '{}' to '{}'", from_node_id, to_node_id),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Chain has edge from '{}' to '{}'", from_node_id, to_node_id),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!(
                        "Chain does not have edge from '{}' to '{}'",
                        from_node_id, to_node_id
                    ),
                    &format!("Edge from '{}' to '{}'", from_node_id, to_node_id),
                    "No such edge",
                ),
            )
        }
    }

    /// Asserts that a chain execution was successful (no error).
    pub fn assert_execution_success(&self, execution: &ChainExecution) -> AssertionResult {
        match &execution.error {
            None => AssertionResult::new("Execution is successful", AssertionOutcome::Passed),
            Some(error) => {
                AssertionResult::new("Execution is successful", AssertionOutcome::Failed)
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            "Execution has an error",
                            "No error",
                            error,
                        ),
                    )
            }
        }
    }

    /// Asserts that a chain execution has a specific error.
    pub fn assert_execution_error(
        &self,
        execution: &ChainExecution,
        expected: &str,
    ) -> AssertionResult {
        match &execution.error {
            Some(error) => {
                if error == expected {
                    AssertionResult::new(
                        &format!("Execution has error '{}'", expected),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("Execution has error '{}'", expected),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            "Execution has a different error",
                            expected,
                            error,
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("Execution has error '{}'", expected),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Execution does not have an error",
                    expected,
                    "No error",
                ),
            ),
        }
    }

    /// Asserts that a chain execution has a specific number of steps.
    pub fn assert_step_count(
        &self,
        execution: &ChainExecution,
        expected: usize,
    ) -> AssertionResult {
        let count = execution.steps.len();
        if count == expected {
            AssertionResult::new(
                &format!("Step count is {}", expected),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Step count is {}", expected),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Step count does not match expected value",
                    &format!("{}", expected),
                    &format!("{}", count),
                ),
            )
        }
    }

    /// Asserts that a chain execution has a step with a specific ID.
    pub fn assert_has_step(&self, execution: &ChainExecution, step_id: &str) -> AssertionResult {
        match execution.get_step(step_id) {
            Some(_) => AssertionResult::new(
                &format!("Execution has step '{}'", step_id),
                AssertionOutcome::Passed,
            ),
            None => AssertionResult::new(
                &format!("Execution has step '{}'", step_id),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Execution does not have step '{}'", step_id),
                    &format!("Step '{}'", step_id),
                    "No such step",
                ),
            ),
        }
    }

    /// Asserts that a chain execution has a step for a specific node.
    pub fn assert_has_step_for_node(
        &self,
        execution: &ChainExecution,
        node_id: &str,
    ) -> AssertionResult {
        match execution.get_step_by_node_id(node_id) {
            Some(_) => AssertionResult::new(
                &format!("Execution has step for node '{}'", node_id),
                AssertionOutcome::Passed,
            ),
            None => AssertionResult::new(
                &format!("Execution has step for node '{}'", node_id),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Execution does not have step for node '{}'", node_id),
                    &format!("Step for node '{}'", node_id),
                    "No such step",
                ),
            ),
        }
    }

    /// Asserts that a chain execution step was successful (no error).
    pub fn assert_step_success(
        &self,
        execution: &ChainExecution,
        step_id: &str,
    ) -> AssertionResult {
        match execution.get_step(step_id) {
            Some(step) => match &step.error {
                None => AssertionResult::new(
                    &format!("Step '{}' is successful", step_id),
                    AssertionOutcome::Passed,
                ),
                Some(error) => AssertionResult::new(
                    &format!("Step '{}' is successful", step_id),
                    AssertionOutcome::Failed,
                )
                .with_error(
                    crate::modules::test_harness::assert::core::AssertionError::new(
                        &format!("Step '{}' has an error", step_id),
                        "No error",
                        error,
                    ),
                ),
            },
            None => AssertionResult::new(
                &format!("Step '{}' is successful", step_id),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Execution does not have step '{}'", step_id),
                    &format!("Step '{}'", step_id),
                    "No such step",
                ),
            ),
        }
    }

    /// Asserts that a chain execution step has a specific error.
    pub fn assert_step_error(
        &self,
        execution: &ChainExecution,
        step_id: &str,
        expected: &str,
    ) -> AssertionResult {
        match execution.get_step(step_id) {
            Some(step) => match &step.error {
                Some(error) => {
                    if error == expected {
                        AssertionResult::new(
                            &format!("Step '{}' has error '{}'", step_id, expected),
                            AssertionOutcome::Passed,
                        )
                    } else {
                        AssertionResult::new(
                            &format!("Step '{}' has error '{}'", step_id, expected),
                            AssertionOutcome::Failed,
                        )
                        .with_error(
                            crate::modules::test_harness::assert::core::AssertionError::new(
                                &format!("Step '{}' has a different error", step_id),
                                expected,
                                error,
                            ),
                        )
                    }
                }
                None => AssertionResult::new(
                    &format!("Step '{}' has error '{}'", step_id, expected),
                    AssertionOutcome::Failed,
                )
                .with_error(
                    crate::modules::test_harness::assert::core::AssertionError::new(
                        &format!("Step '{}' does not have an error", step_id),
                        expected,
                        "No error",
                    ),
                ),
            },
            None => AssertionResult::new(
                &format!("Step '{}' has error '{}'", step_id, expected),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Execution does not have step '{}'", step_id),
                    &format!("Step '{}'", step_id),
                    "No such step",
                ),
            ),
        }
    }

    /// Asserts that a chain execution was completed within a specific time.
    pub fn assert_execution_time(
        &self,
        execution: &ChainExecution,
        max_ms: u64,
    ) -> AssertionResult {
        let execution_time_ms = execution.duration.as_millis() as u64;
        if execution_time_ms <= max_ms {
            AssertionResult::new(
                &format!("Execution time <= {} ms", max_ms),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Execution time <= {} ms", max_ms),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Execution time exceeds maximum",
                    &format!("<= {} ms", max_ms),
                    &format!("{} ms", execution_time_ms),
                ),
            )
        }
    }

    /// Asserts that a chain execution step was completed within a specific time.
    pub fn assert_step_time(
        &self,
        execution: &ChainExecution,
        step_id: &str,
        max_ms: u64,
    ) -> AssertionResult {
        match execution.get_step(step_id) {
            Some(step) => {
                let step_time_ms = step.duration.as_millis() as u64;
                if step_time_ms <= max_ms {
                    AssertionResult::new(
                        &format!("Step '{}' time <= {} ms", step_id, max_ms),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("Step '{}' time <= {} ms", step_id, max_ms),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            &format!("Step '{}' time exceeds maximum", step_id),
                            &format!("<= {} ms", max_ms),
                            &format!("{} ms", step_time_ms),
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("Step '{}' time <= {} ms", step_id, max_ms),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Execution does not have step '{}'", step_id),
                    &format!("Step '{}'", step_id),
                    "No such step",
                ),
            ),
        }
    }
}

impl Default for ChainAssertions {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new chain assertions instance.
pub fn create_chain_assertions() -> ChainAssertions {
    ChainAssertions::new()
}

/// Creates a new chain node.
pub fn create_chain_node(node_id: &str, node_type: &str, name: &str) -> ChainNode {
    ChainNode::new(node_id, node_type, name)
}

/// Creates a new chain.
pub fn create_chain(chain_id: &str, name: &str) -> Chain {
    Chain::new(chain_id, name)
}

/// Creates a new chain execution step.
pub fn create_chain_execution_step(step_id: &str, node_id: &str) -> ChainExecutionStep {
    ChainExecutionStep::new(step_id, node_id)
}

/// Creates a new chain execution.
pub fn create_chain_execution(execution_id: &str, chain_id: &str) -> ChainExecution {
    ChainExecution::new(execution_id, chain_id)
}

use std::collections::HashMap;
