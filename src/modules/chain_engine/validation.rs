//! Chain validation
//!
//! This module provides functionality for validating chain definitions.

use std::collections::{HashMap, HashSet};

use crate::modules::chain_engine::chain_definition::{Chain, Condition, DependencyType, StepType};
use crate::modules::chain_engine::error::{ChainError, ChainResult};

/// Validates a chain definition
pub fn validate_chain(chain: &Chain) -> ChainResult<()> {
    // Check for duplicate step IDs
    let mut step_ids = HashSet::new();
    for step_id in chain.steps.keys() {
        if !step_ids.insert(step_id) {
            return Err(ChainError::ValidationError(format!(
                "Duplicate step ID: {}",
                step_id
            )));
        }
    }

    // Check for circular dependencies
    check_circular_dependencies(chain)?;

    // Validate step types
    for (step_id, step) in &chain.steps {
        match &step.step_type {
            StepType::LLMInference { .. } => {
                // Validate LLM inference step
                // No specific validation needed at this point
            }
            StepType::FunctionCall { function_name, .. } => {
                // Validate function call step
                if function_name.is_empty() {
                    return Err(ChainError::ValidationError(format!(
                        "Function name cannot be empty in step: {}",
                        step_id
                    )));
                }
            }
            StepType::ToolUse { tool_name, .. } => {
                // Validate tool use step
                if tool_name.is_empty() {
                    return Err(ChainError::ValidationError(format!(
                        "Tool name cannot be empty in step: {}",
                        step_id
                    )));
                }
            }
            StepType::Conditional { branches, .. } => {
                // Validate conditional step
                for branch in branches {
                    if !chain.steps.contains_key(&branch.target_step) {
                        return Err(ChainError::ValidationError(format!(
                            "Branch target step not found: {}",
                            branch.target_step
                        )));
                    }
                }
            }
            StepType::Parallel { steps, .. } => {
                // Validate parallel step
                for parallel_step_id in steps {
                    if !chain.steps.contains_key(parallel_step_id) {
                        return Err(ChainError::ValidationError(format!(
                            "Parallel step not found: {}",
                            parallel_step_id
                        )));
                    }
                }
            }
            StepType::Loop { steps, .. } => {
                // Validate loop step
                for loop_step_id in steps {
                    if !chain.steps.contains_key(loop_step_id) {
                        return Err(ChainError::ValidationError(format!(
                            "Loop step not found: {}",
                            loop_step_id
                        )));
                    }
                }
            }
            StepType::Custom { handler, .. } => {
                // Validate custom step
                if handler.is_empty() {
                    return Err(ChainError::ValidationError(format!(
                        "Handler cannot be empty in step: {}",
                        step_id
                    )));
                }
            }
        }
    }

    // Validate dependencies
    for dependency in &chain.dependencies {
        if !chain.steps.contains_key(&dependency.dependent_step) {
            return Err(ChainError::ValidationError(format!(
                "Dependent step not found: {}",
                dependency.dependent_step
            )));
        }

        match &dependency.dependency_type {
            DependencyType::Simple { required_step } => {
                if !chain.steps.contains_key(required_step) {
                    return Err(ChainError::ValidationError(format!(
                        "Required step not found: {}",
                        required_step
                    )));
                }
            }
            DependencyType::All { required_steps } => {
                for step_id in required_steps {
                    if !chain.steps.contains_key(step_id) {
                        return Err(ChainError::ValidationError(format!(
                            "Required step not found: {}",
                            step_id
                        )));
                    }
                }
            }
            DependencyType::Any { required_steps } => {
                if required_steps.is_empty() {
                    return Err(ChainError::ValidationError(
                        "Any dependency must have at least one required step".to_string(),
                    ));
                }

                for step_id in required_steps {
                    if !chain.steps.contains_key(step_id) {
                        return Err(ChainError::ValidationError(format!(
                            "Required step not found: {}",
                            step_id
                        )));
                    }
                }
            }
            DependencyType::Conditional { required_step, .. } => {
                if !chain.steps.contains_key(required_step) {
                    return Err(ChainError::ValidationError(format!(
                        "Required step not found: {}",
                        required_step
                    )));
                }
            }
        }
    }

    // Validate variables
    for (var_name, variable) in &chain.variables {
        if var_name != &variable.name {
            return Err(ChainError::ValidationError(format!(
                "Variable name mismatch: {} vs {}",
                var_name, variable.name
            )));
        }

        if variable.required && variable.initial_value.is_none() {
            return Err(ChainError::ValidationError(format!(
                "Required variable {} has no initial value",
                var_name
            )));
        }
    }

    Ok(())
}

/// Check for circular dependencies in a chain
fn check_circular_dependencies(chain: &Chain) -> ChainResult<()> {
    // Build dependency graph
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();

    // Add all steps to the graph
    for step_id in chain.steps.keys() {
        graph.insert(step_id.clone(), Vec::new());
    }

    // Add dependencies to the graph
    for dependency in &chain.dependencies {
        let dependent_step = &dependency.dependent_step;

        match &dependency.dependency_type {
            DependencyType::Simple { required_step } => {
                graph
                    .get_mut(dependent_step)
                    .unwrap()
                    .push(required_step.clone());
            }
            DependencyType::All { required_steps } => {
                for step_id in required_steps {
                    graph.get_mut(dependent_step).unwrap().push(step_id.clone());
                }
            }
            DependencyType::Any { required_steps } => {
                for step_id in required_steps {
                    graph.get_mut(dependent_step).unwrap().push(step_id.clone());
                }
            }
            DependencyType::Conditional { required_step, .. } => {
                graph
                    .get_mut(dependent_step)
                    .unwrap()
                    .push(required_step.clone());
            }
        }
    }

    // Check for cycles using DFS
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    for step_id in chain.steps.keys() {
        if is_cyclic(&graph, step_id, &mut visited, &mut rec_stack)? {
            return Err(ChainError::CircularDependency(format!(
                "Circular dependency detected in step: {}",
                step_id
            )));
        }
    }

    Ok(())
}

/// Check if a graph has a cycle using DFS
fn is_cyclic(
    graph: &HashMap<String, Vec<String>>,
    node: &str,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
) -> ChainResult<bool> {
    if !visited.contains(node) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if is_cyclic(graph, neighbor, visited, rec_stack)? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(neighbor) {
                    return Ok(true);
                }
            }
        } else {
            return Err(ChainError::StepNotFound(node.to_string()));
        }
    }

    rec_stack.remove(node);
    Ok(false)
}

/// Validate a condition
pub fn validate_condition(
    condition: &Condition,
    variables: &HashMap<String, serde_json::Value>,
) -> ChainResult<()> {
    match condition {
        Condition::Equals { variable, .. }
        | Condition::Contains { variable, .. }
        | Condition::Regex { variable, .. }
        | Condition::GreaterThan { variable, .. }
        | Condition::LessThan { variable, .. } => {
            if !variables.contains_key(variable) {
                return Err(ChainError::VariableNotFound(variable.clone()));
            }
        }
        Condition::And { conditions } => {
            for cond in conditions {
                validate_condition(cond, variables)?;
            }
        }
        Condition::Or { conditions } => {
            for cond in conditions {
                validate_condition(cond, variables)?;
            }
        }
        Condition::Not { condition } => {
            validate_condition(condition, variables)?;
        }
        Condition::Custom { .. } => {
            // Custom conditions are validated at runtime
        }
    }

    Ok(())
}
