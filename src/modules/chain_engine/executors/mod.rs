//! Step executors
//!
//! This module provides executors for different step types.

use async_trait::async_trait;

use crate::modules::chain_engine::context::{ChainContext, StepResult};
use crate::modules::chain_engine::definition::ChainStep;
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

