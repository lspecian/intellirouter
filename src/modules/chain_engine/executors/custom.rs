//! Custom Executor
//!
//! This module provides an executor for custom steps.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::modules::chain_engine::chain_definition::ChainStep;
use crate::modules::chain_engine::context::{ChainContext, StepResult};
use crate::modules::chain_engine::error::{ChainError, ChainResult};
use crate::modules::chain_engine::executors::StepExecutor;

/// Custom step executor
pub struct CustomExecutor {
    // This would typically contain a registry of custom handlers
    // handler_registry: Arc<dyn HandlerRegistry>,
}

impl CustomExecutor {
    /// Create a new custom executor
    pub fn new(/* handler_registry: Arc<dyn HandlerRegistry> */) -> Self {
        Self {
            // handler_registry,
        }
    }
}

#[async_trait]
impl StepExecutor for CustomExecutor {
    async fn execute_step(
        &self,
        step: &ChainStep,
        context: &ChainContext,
    ) -> ChainResult<StepResult> {
        let start_time = Instant::now();

        // Extract step configuration
        let config = match &step.step_type {
            crate::modules::chain_engine::chain_definition::StepType::Custom {
                handler,
                config,
            } => {
                // In a real implementation, we would call the handler registry
                // let result = self.handler_registry.call_handler(
                //     handler,
                //     config.clone(),
                //     context,
                // ).await?;

                // Simulate a response for now
                let output = format!(
                    "Custom handler {} called with config: {:?}",
                    handler, config
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
                    "Step type mismatch: expected Custom, got {:?}",
                    step.step_type
                )));
            }
        };

        Ok(config)
    }
}
