//! Function Call Executor
//!
//! This module provides an executor for function call steps.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::modules::chain_engine::chain_definition::ChainStep;
use crate::modules::chain_engine::context::{ChainContext, StepResult};
use crate::modules::chain_engine::error::{ChainError, ChainResult};
use crate::modules::chain_engine::executors::StepExecutor;

/// Function call step executor
pub struct FunctionCallExecutor {
    // This would typically contain a registry of available functions
    // function_registry: Arc<dyn FunctionRegistry>,
}

impl FunctionCallExecutor {
    /// Create a new function call executor
    pub fn new(/* function_registry: Arc<dyn FunctionRegistry> */) -> Self {
        Self {
            // function_registry,
        }
    }

    /// Resolve arguments for a function
    fn resolve_arguments(
        &self,
        arguments: &HashMap<String, serde_json::Value>,
        context: &ChainContext,
    ) -> ChainResult<HashMap<String, serde_json::Value>> {
        let mut resolved_args = HashMap::new();

        for (name, value) in arguments {
            let resolved_value = match value {
                serde_json::Value::String(s) if s.starts_with("{{") && s.ends_with("}}") => {
                    // Variable reference
                    let var_name = s[2..s.len() - 2].trim();
                    context.variables.get(var_name).cloned().ok_or_else(|| {
                        ChainError::VariableNotFound(format!("Variable not found: {}", var_name))
                    })?
                }
                _ => value.clone(),
            };

            resolved_args.insert(name.clone(), resolved_value);
        }

        Ok(resolved_args)
    }
}

#[async_trait]
impl StepExecutor for FunctionCallExecutor {
    async fn execute_step(
        &self,
        step: &ChainStep,
        context: &ChainContext,
    ) -> ChainResult<StepResult> {
        let start_time = Instant::now();

        // Extract step configuration
        let config = match &step.step_type {
            crate::modules::chain_engine::chain_definition::StepType::FunctionCall {
                function_name,
                arguments,
            } => {
                // Prepare the arguments
                let resolved_args = self.resolve_arguments(arguments, context)?;

                // In a real implementation, we would call the function registry
                // let result = self.function_registry.call_function(
                //     function_name,
                //     resolved_args,
                // ).await?;

                // Simulate a response for now
                let output = format!(
                    "Function {} called with arguments: {:?}",
                    function_name, resolved_args
                );

                // Create the result
                let mut outputs = HashMap::new();
                outputs.insert("output".to_string(), serde_json::Value::String(output));

                StepResult {
                    step_id: step.id.clone(),
                    outputs,
                    error: None,
                    execution_time: start_time.elapsed(),
                }
            }
            _ => {
                return Err(ChainError::StepExecutionError(format!(
                    "Step type mismatch: expected FunctionCall, got {:?}",
                    step.step_type
                )));
            }
        };

        Ok(config)
    }
}
