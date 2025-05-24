//! Orchestrator Reporting Trait
//!
//! This module provides the OrchestratorReporting trait for reporting orchestrator state.

use crate::modules::orchestrator::types::{OrchestratorError, Task};

/// Orchestrator reporting trait
pub trait OrchestratorReporting: Send + Sync {
    /// Get all tasks
    fn get_all_tasks(&self) -> Result<Vec<Task>, OrchestratorError>;
}

/// Quality analyzer trait
pub trait QualityAnalyzer: Send + Sync {
    /// Analyze quality
    fn analyze_quality(&self) -> Result<f64, OrchestratorError>;
}
