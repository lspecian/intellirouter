//! Tool Use Executor
//!
//! This module provides an executor for tool use steps.

use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Instant;

use crate::modules::chain_engine::context::{ChainContext, StepResult};
use crate::modules::chain_engine::definition::ChainStep;
use crate::modules::chain_engine::error::{ChainError, ChainResult};
use crate::modules::chain_engine::executors::StepExecutor;

/// Tool use step executor
pub struct ToolUseExecutor {
    // This would typically contain a registry of available tools
    // tool_registry: Arc<dyn ToolRegistry>,
}

impl ToolUseExecutor {
    /// Create a new tool use executor
    pub fn new(/* tool_registry: Arc<dyn ToolRegistry> */) -> Self {
        Self {
            // tool_registry,
        }
    }

    /// Resolve arguments for a tool
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
impl StepExecutor for ToolUseExecutor {
    async fn execute_step(
        &self,
        step: &ChainStep,
        context: &ChainContext,
    ) -> ChainResult<StepResult> {
        let start_time = Instant::now();

        // Extract step configuration
        let config = match &step.step_type {
            crate::modules::chain_engine::definition::StepType::ToolUse {
                tool_name,
                arguments,
            } => {
                // Prepare the arguments
                let resolved_args = self.resolve_arguments(arguments, context)?;

                // In a real implementation, we would call the tool registry
                // let result = self.tool_registry.use_tool(
                //     tool_name,
                //     resolved_args,
                // ).await?;

                // Simulate a response for now
                let output = format!(
                    "Tool {} used with arguments: {:?}",
                    tool_name, resolved_args
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
                    "Step type mismatch: expected ToolUse, got {:?}",
                    step.step_type
                )));
            }
        };

        Ok(config)
    }
}
