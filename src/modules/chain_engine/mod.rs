//! Chain Engine Module
//!
//! This module handles multi-step orchestration of LLM calls and other operations.
//! It allows for creating complex workflows with multiple steps, conditional branching,
//! parallel execution, and data transformation between steps.

mod condition_evaluator;
mod context;
mod definition;
mod engine;
mod error;
mod executors;
mod validation;

#[cfg(test)]
mod tests;

pub use condition_evaluator::*;
pub use context::*;
pub use definition::*;
pub use engine::*;
pub use error::*;
pub use executors::StepExecutor;
pub use validation::*;

// Note: The following files are now redundant and should be removed in a future cleanup:
// - core.rs (replaced by context.rs, engine.rs, and condition_evaluator.rs)
// - executors.rs (replaced by executors/ directory)
