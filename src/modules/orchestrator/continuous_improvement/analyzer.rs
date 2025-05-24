//! Analyzer Trait
//!
//! This module provides the Analyzer trait for analyzing testing results.

use crate::modules::orchestrator::types::{OrchestratorError, Task, TaskResult};
use crate::modules::orchestrator::workflow::{Workflow, WorkflowResult};

use super::types::AnalysisResult;

/// Analyzer trait for analyzing testing results
pub trait Analyzer: Send + Sync + std::fmt::Debug {
    /// Analyze tasks
    fn analyze_tasks(&self, tasks: &[Task]) -> Result<AnalysisResult, OrchestratorError>;

    /// Analyze workflows
    fn analyze_workflows(
        &self,
        workflows: &[Workflow],
    ) -> Result<AnalysisResult, OrchestratorError>;

    /// Analyze task results
    fn analyze_task_results(
        &self,
        results: &[TaskResult],
    ) -> Result<AnalysisResult, OrchestratorError>;

    /// Analyze workflow results
    fn analyze_workflow_results(
        &self,
        results: &[WorkflowResult],
    ) -> Result<AnalysisResult, OrchestratorError>;
}
