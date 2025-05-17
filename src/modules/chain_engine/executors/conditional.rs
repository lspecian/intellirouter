//! Conditional Executor
//!
//! This module provides an executor for conditional steps.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::modules::chain_engine::chain_definition::{Chain, ChainStep, ConditionalBranch};
use crate::modules::chain_engine::condition_evaluator::ConditionEvaluator;
use crate::modules::chain_engine::context::{ChainContext, StepResult};
use crate::modules::chain_engine::error::{ChainError, ChainResult};
use crate::modules::chain_engine::executors::StepExecutor;

/// Conditional step executor
pub struct ConditionalExecutor {
    condition_evaluator: ConditionEvaluator,
}

impl ConditionalExecutor {
    /// Create a new conditional executor
    pub fn new() -> Self {
        Self {
            condition_evaluator: ConditionEvaluator::new(),
        }
    }
}

#[async_trait]
impl StepExecutor for ConditionalExecutor {
    async fn execute_step(
        &self,
        step: &ChainStep,
        context: &ChainContext,
    ) -> ChainResult<StepResult> {
        let start_time = Instant::now();

        // Extract step configuration
        let config = match &step.step_type {
            crate::modules::chain_engine::chain_definition::StepType::Conditional {
                branches,
                default_branch,
            } => {
                // Evaluate conditions and find the matching branch
                let mut target_step = None;

                for branch in branches {
                    if self
                        .condition_evaluator
                        .evaluate_condition(&branch.condition, context)?
                    {
                        target_step = Some(branch.target_step.clone());
                        break;
                    }
                }

                // If no branch matched, use the default branch
                let target_step =
                    target_step
                        .or_else(|| default_branch.clone())
                        .ok_or_else(|| {
                            ChainError::StepExecutionError(
                                "No matching branch and no default branch specified".to_string(),
                            )
                        })?;

                // Create the result
                let mut outputs = HashMap::new();
                outputs.insert(
                    "target_step".to_string(),
                    serde_json::Value::String(target_step),
                );

                StepResult {
                    step_id: step.id.clone(),
                    outputs,
                    error: None,
                    execution_time: start_time.elapsed(),
                }
            }
            _ => {
                return Err(ChainError::StepExecutionError(format!(
                    "Step type mismatch: expected Conditional, got {:?}",
                    step.step_type
                )));
            }
        };

        Ok(config)
    }
}
