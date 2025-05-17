//! Step executors
//!
//! This module provides executors for different step types.

use async_trait::async_trait;
use std::sync::Arc;

use crate::modules::chain_engine::chain_definition::ChainStep;
use crate::modules::chain_engine::context::{ChainContext, StepResult};
use crate::modules::chain_engine::error::ChainResult;

/// Interface for step executors
#[async_trait]
pub trait StepExecutor: Send + Sync {
    async fn execute_step(
        &self,
        step: &ChainStep,
        context: &ChainContext,
    ) -> ChainResult<StepResult>;
}

// Re-export specific executors
pub mod conditional;
pub mod custom;
pub mod function;
pub mod llm;
pub mod loop_executor;
pub mod parallel;
pub mod tool;

pub use conditional::ConditionalExecutor;
pub use custom::CustomExecutor;
pub use function::FunctionCallExecutor;
pub use llm::LLMInferenceExecutor;
pub use loop_executor::LoopExecutor;
pub use parallel::ParallelExecutor;
pub use tool::ToolUseExecutor;
