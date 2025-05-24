//! Parallel Executor
//!
//! This module provides an executor for parallel steps.

use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Instant;

use crate::modules::chain_engine::context::{ChainContext, StepResult};
use crate::modules::chain_engine::definition::ChainStep;
use crate::modules::chain_engine::error::{ChainError, ChainResult};
use crate::modules::chain_engine::executors::StepExecutor;

/// Parallel step executor
pub struct ParallelExecutor {
    // This would typically contain a reference to the chain engine
    // chain_engine: Arc<ChainEngine>,
}

impl ParallelExecutor {
    /// Create a new parallel executor
    pub fn new(/* chain_engine: Arc<ChainEngine> */) -> Self {
        Self {
            // chain_engine,
        }
    }
}

#[async_trait]
impl StepExecutor for ParallelExecutor {
    async fn execute_step(
        &self,
        step: &ChainStep,
        _context: &ChainContext,
    ) -> ChainResult<StepResult> {
        let start_time = Instant::now();

        // Extract step configuration
        let config = match &step.step_type {
            crate::modules::chain_engine::definition::StepType::Parallel {
                steps,
                wait_for_all,
            } => {
                // In a real implementation, we would execute the steps in parallel
                // let mut handles = Vec::new();
                // for step_id in steps {
                //     let target_step = chain.steps.get(step_id).ok_or_else(|| {
                //         ChainError::StepNotFound(format!("Step not found: {}", step_id))
                //     })?;
                //     let context_clone = context.clone();
                //     let target_step_clone = target_step.clone();
                //     let chain_engine_clone = self.chain_engine.clone();
                //
                //     let handle = tokio::spawn(async move {
                //         match &target_step_clone.step_type {
                //             StepType::LLMInference { .. } => {
                //                 chain_engine_clone.execute_llm_inference_step(&target_step_clone, context_clone).await
                //             }
                //             // ... other step types
                //             _ => {
                //                 Err(ChainError::StepExecutionError(format!(
                //                     "Unsupported step type in parallel execution: {:?}",
                //                     target_step_clone.step_type
                //                 )))
                //             }
                //         }
                //     });
                //
                //     handles.push(handle);
                // }
                //
                // // Wait for all steps to complete or for the first success/failure
                // let results = if *wait_for_all {
                //     try_join_all(handles).await?
                // } else {
                //     // Wait for the first step to complete
                //     let mut results = Vec::new();
                //     for handle in handles {
                //         match handle.await {
                //             Ok(result) => {
                //                 results.push(result);
                //                 break;
                //             }
                //             Err(e) => {
                //                 return Err(ChainError::StepExecutionError(format!(
                //                     "Error in parallel execution: {}",
                //                     e
                //                 )));
                //             }
                //         }
                //     }
                //     results
                // };

                // Simulate parallel execution for now
                let output = format!(
                    "Parallel execution of steps: {:?}, wait_for_all: {}",
                    steps, wait_for_all
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
                    "Step type mismatch: expected Parallel, got {:?}",
                    step.step_type
                )));
            }
        };

        Ok(config)
    }
}
