//! Performance Analyzer
//!
//! This module provides the PerformanceAnalyzer implementation for analyzing performance metrics.


use crate::modules::orchestrator::types::{OrchestratorError, Task, TaskResult, TaskStatus};
use crate::modules::orchestrator::workflow::{Workflow, WorkflowResult};

use super::analyzer::Analyzer;
use super::types::{
    AnalysisFinding, AnalysisRecommendation, AnalysisResult, EstimatedImpact, FindingSeverity,
    ImplementationDifficulty,
};
use crate::modules::orchestrator::reporting::SuggestionPriority;

/// Performance analyzer
#[derive(Debug, Default)]
pub struct PerformanceAnalyzer {
    /// Performance thresholds
    thresholds: PerformanceThresholds,
}

/// Performance thresholds
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    /// Slow task threshold (ms)
    pub slow_task_threshold_ms: u64,
    /// Very slow task threshold (ms)
    pub very_slow_task_threshold_ms: u64,
    /// Slow workflow threshold (ms)
    pub slow_workflow_threshold_ms: u64,
    /// Very slow workflow threshold (ms)
    pub very_slow_workflow_threshold_ms: u64,
    /// Low completion rate threshold
    pub low_completion_rate_threshold: f64,
    /// Very low completion rate threshold
    pub very_low_completion_rate_threshold: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            slow_task_threshold_ms: 1000,
            very_slow_task_threshold_ms: 5000,
            slow_workflow_threshold_ms: 5000,
            very_slow_workflow_threshold_ms: 20000,
            low_completion_rate_threshold: 0.8,
            very_low_completion_rate_threshold: 0.5,
        }
    }
}

impl PerformanceAnalyzer {
    /// Create a new performance analyzer
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new performance analyzer with custom thresholds
    pub fn with_thresholds(thresholds: PerformanceThresholds) -> Self {
        Self { thresholds }
    }
}

impl Analyzer for PerformanceAnalyzer {
    fn analyze_tasks(&self, tasks: &[Task]) -> Result<AnalysisResult, OrchestratorError> {
        let mut result = AnalysisResult::new(
            "performance-tasks",
            "Performance Analysis (Tasks)",
            "Analysis of task performance",
        );

        // Calculate metrics
        let total_tasks = tasks.len();
        let completed_tasks = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        let failed_tasks = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Failed)
            .count();
        let completion_rate = if total_tasks > 0 {
            completed_tasks as f64 / total_tasks as f64
        } else {
            0.0
        };

        // Add metrics to result
        result = result
            .with_metric("total_tasks", total_tasks as f64)
            .with_metric("completed_tasks", completed_tasks as f64)
            .with_metric("failed_tasks", failed_tasks as f64)
            .with_metric("completion_rate", completion_rate);

        // Check for low completion rate
        if completion_rate < self.thresholds.very_low_completion_rate_threshold {
            let finding = AnalysisFinding::new(
                "very-low-completion-rate",
                "Very Low Task Completion Rate",
                format!(
                    "The task completion rate is very low ({:.1}%)",
                    completion_rate * 100.0
                ),
                FindingSeverity::Critical,
            );
            result = result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "improve-task-reliability",
                "Improve Task Reliability",
                "Investigate and fix issues causing tasks to fail",
                SuggestionPriority::High,
                ImplementationDifficulty::Hard,
                EstimatedImpact::VeryHigh,
            );
            result = result.with_recommendation(recommendation);
        } else if completion_rate < self.thresholds.low_completion_rate_threshold {
            let finding = AnalysisFinding::new(
                "low-completion-rate",
                "Low Task Completion Rate",
                format!(
                    "The task completion rate is low ({:.1}%)",
                    completion_rate * 100.0
                ),
                FindingSeverity::High,
            );
            result = result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "improve-task-reliability",
                "Improve Task Reliability",
                "Investigate and fix issues causing tasks to fail",
                SuggestionPriority::Medium,
                ImplementationDifficulty::Moderate,
                EstimatedImpact::High,
            );
            result = result.with_recommendation(recommendation);
        }

        Ok(result)
    }

    fn analyze_workflows(
        &self,
        workflows: &[Workflow],
    ) -> Result<AnalysisResult, OrchestratorError> {
        let mut result = AnalysisResult::new(
            "performance-workflows",
            "Performance Analysis (Workflows)",
            "Analysis of workflow performance",
        );

        // Calculate metrics
        let total_workflows = workflows.len();
        let completed_workflows = workflows
            .iter()
            .filter(|w| w.metadata.get("status").map_or(false, |s| s == "completed"))
            .count();
        let failed_workflows = workflows
            .iter()
            .filter(|w| w.metadata.get("status").map_or(false, |s| s == "failed"))
            .count();
        let completion_rate = if total_workflows > 0 {
            completed_workflows as f64 / total_workflows as f64
        } else {
            0.0
        };

        // Add metrics to result
        result = result
            .with_metric("total_workflows", total_workflows as f64)
            .with_metric("completed_workflows", completed_workflows as f64)
            .with_metric("failed_workflows", failed_workflows as f64)
            .with_metric("completion_rate", completion_rate);

        // Check for low completion rate
        if completion_rate < self.thresholds.very_low_completion_rate_threshold {
            let finding = AnalysisFinding::new(
                "very-low-workflow-completion-rate",
                "Very Low Workflow Completion Rate",
                format!(
                    "The workflow completion rate is very low ({:.1}%)",
                    completion_rate * 100.0
                ),
                FindingSeverity::Critical,
            );
            result = result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "improve-workflow-reliability",
                "Improve Workflow Reliability",
                "Investigate and fix issues causing workflows to fail",
                SuggestionPriority::High,
                ImplementationDifficulty::Hard,
                EstimatedImpact::VeryHigh,
            );
            result = result.with_recommendation(recommendation);
        } else if completion_rate < self.thresholds.low_completion_rate_threshold {
            let finding = AnalysisFinding::new(
                "low-workflow-completion-rate",
                "Low Workflow Completion Rate",
                format!(
                    "The workflow completion rate is low ({:.1}%)",
                    completion_rate * 100.0
                ),
                FindingSeverity::High,
            );
            result = result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "improve-workflow-reliability",
                "Improve Workflow Reliability",
                "Investigate and fix issues causing workflows to fail",
                SuggestionPriority::Medium,
                ImplementationDifficulty::Moderate,
                EstimatedImpact::High,
            );
            result = result.with_recommendation(recommendation);
        }

        Ok(result)
    }

    fn analyze_task_results(
        &self,
        results: &[TaskResult],
    ) -> Result<AnalysisResult, OrchestratorError> {
        let mut analysis_result = AnalysisResult::new(
            "performance-task-results",
            "Performance Analysis (Task Results)",
            "Analysis of task result performance",
        );

        // Calculate metrics
        let total_results = results.len();
        let successful_results = results
            .iter()
            .filter(|r| r.status == TaskStatus::Completed)
            .count();
        let failed_results = total_results - successful_results;
        let success_rate = if total_results > 0 {
            successful_results as f64 / total_results as f64
        } else {
            0.0
        };

        // Calculate execution time statistics
        let mut execution_times: Vec<u64> = results
            .iter()
            .map(|r| {
                // Extract duration from metadata or use a default value
                r.data
                    .get("duration_ms")
                    .and_then(|d| d.parse::<u64>().ok())
                    .unwrap_or(0)
            })
            .collect();
        execution_times.sort_unstable();

        let avg_execution_time = if !execution_times.is_empty() {
            execution_times.iter().sum::<u64>() as f64 / execution_times.len() as f64
        } else {
            0.0
        };

        let median_execution_time = if !execution_times.is_empty() {
            execution_times[execution_times.len() / 2]
        } else {
            0
        };

        let max_execution_time = execution_times.last().copied().unwrap_or(0);

        // Add metrics to result
        analysis_result = analysis_result
            .with_metric("total_results", total_results as f64)
            .with_metric("successful_results", successful_results as f64)
            .with_metric("failed_results", failed_results as f64)
            .with_metric("success_rate", success_rate)
            .with_metric("avg_execution_time_ms", avg_execution_time)
            .with_metric("median_execution_time_ms", median_execution_time as f64)
            .with_metric("max_execution_time_ms", max_execution_time as f64);

        // Find slow tasks
        let slow_tasks: Vec<&TaskResult> = results
            .iter()
            .filter(|r| {
                let duration = r
                    .data
                    .get("duration_ms")
                    .and_then(|d| d.parse::<u64>().ok())
                    .unwrap_or(0);
                duration > self.thresholds.slow_task_threshold_ms
                    && duration <= self.thresholds.very_slow_task_threshold_ms
            })
            .collect();

        let very_slow_tasks: Vec<&TaskResult> = results
            .iter()
            .filter(|r| {
                let duration = r
                    .data
                    .get("duration_ms")
                    .and_then(|d| d.parse::<u64>().ok())
                    .unwrap_or(0);
                duration > self.thresholds.very_slow_task_threshold_ms
            })
            .collect();

        // Add findings for slow tasks
        if !slow_tasks.is_empty() {
            let finding = AnalysisFinding::new(
                "slow-tasks",
                "Slow Tasks Detected",
                format!(
                    "{} tasks took longer than {}ms to execute",
                    slow_tasks.len(),
                    self.thresholds.slow_task_threshold_ms
                ),
                FindingSeverity::Medium,
            );
            analysis_result = analysis_result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "optimize-slow-tasks",
                "Optimize Slow Tasks",
                "Investigate and optimize slow tasks to improve performance",
                SuggestionPriority::Medium,
                ImplementationDifficulty::Moderate,
                EstimatedImpact::Medium,
            );
            analysis_result = analysis_result.with_recommendation(recommendation);
        }

        // Add findings for very slow tasks
        if !very_slow_tasks.is_empty() {
            let finding = AnalysisFinding::new(
                "very-slow-tasks",
                "Very Slow Tasks Detected",
                format!(
                    "{} tasks took longer than {}ms to execute",
                    very_slow_tasks.len(),
                    self.thresholds.very_slow_task_threshold_ms
                ),
                FindingSeverity::High,
            );
            analysis_result = analysis_result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "optimize-very-slow-tasks",
                "Optimize Very Slow Tasks",
                "Investigate and optimize very slow tasks to improve performance",
                SuggestionPriority::High,
                ImplementationDifficulty::Hard,
                EstimatedImpact::High,
            );
            analysis_result = analysis_result.with_recommendation(recommendation);
        }

        Ok(analysis_result)
    }

    fn analyze_workflow_results(
        &self,
        results: &[WorkflowResult],
    ) -> Result<AnalysisResult, OrchestratorError> {
        let mut analysis_result = AnalysisResult::new(
            "performance-workflow-results",
            "Performance Analysis (Workflow Results)",
            "Analysis of workflow result performance",
        );

        // Calculate metrics
        let total_results = results.len();
        let successful_results = results
            .iter()
            .filter(|r| r.completed_tasks > r.failed_tasks)
            .count();
        let failed_results = total_results - successful_results;
        let success_rate = if total_results > 0 {
            successful_results as f64 / total_results as f64
        } else {
            0.0
        };

        // Calculate execution time statistics
        // Extract duration from metadata or calculate from timestamps
        let mut execution_times: Vec<u64> = results
            .iter()
            .map(|r| {
                // Try to get duration from error_message
                if let Some(error_msg) = &r.error_message {
                    if error_msg.contains("duration:") {
                        if let Some(duration_str) = error_msg.split("duration:").nth(1) {
                            if let Ok(duration) = duration_str.trim().parse::<u64>() {
                                return duration;
                            }
                        }
                    }
                }

                // Default to 0 if no duration information is available
                0
            })
            .collect();
        execution_times.sort_unstable();

        let avg_execution_time = if !execution_times.is_empty() {
            execution_times.iter().sum::<u64>() as f64 / execution_times.len() as f64
        } else {
            0.0
        };

        let median_execution_time = if !execution_times.is_empty() {
            execution_times[execution_times.len() / 2]
        } else {
            0
        };

        let max_execution_time = execution_times.last().copied().unwrap_or(0);

        // Add metrics to result
        analysis_result = analysis_result
            .with_metric("total_results", total_results as f64)
            .with_metric("successful_results", successful_results as f64)
            .with_metric("failed_results", failed_results as f64)
            .with_metric("success_rate", success_rate)
            .with_metric("avg_execution_time_ms", avg_execution_time)
            .with_metric("median_execution_time_ms", median_execution_time as f64)
            .with_metric("max_execution_time_ms", max_execution_time as f64);

        // Find slow workflows
        let slow_workflows: Vec<&WorkflowResult> = results
            .iter()
            .filter(|r| {
                // Extract duration from error_message field
                let duration = if let Some(error_msg) = &r.error_message {
                    if error_msg.contains("duration:") {
                        error_msg
                            .split("duration:")
                            .nth(1)
                            .and_then(|s| s.trim().parse::<u64>().ok())
                            .unwrap_or(0)
                    } else {
                        0
                    }
                } else {
                    0
                };

                duration > self.thresholds.slow_workflow_threshold_ms
                    && duration <= self.thresholds.very_slow_workflow_threshold_ms
            })
            .collect();

        let very_slow_workflows: Vec<&WorkflowResult> = results
            .iter()
            .filter(|r| {
                // Extract duration from error_message field
                let duration = if let Some(error_msg) = &r.error_message {
                    if error_msg.contains("duration:") {
                        error_msg
                            .split("duration:")
                            .nth(1)
                            .and_then(|s| s.trim().parse::<u64>().ok())
                            .unwrap_or(0)
                    } else {
                        0
                    }
                } else {
                    0
                };

                duration > self.thresholds.very_slow_workflow_threshold_ms
            })
            .collect();

        // Add findings for slow workflows
        if !slow_workflows.is_empty() {
            let finding = AnalysisFinding::new(
                "slow-workflows",
                "Slow Workflows Detected",
                format!(
                    "{} workflows took longer than {}ms to execute",
                    slow_workflows.len(),
                    self.thresholds.slow_workflow_threshold_ms
                ),
                FindingSeverity::Medium,
            );
            analysis_result = analysis_result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "optimize-slow-workflows",
                "Optimize Slow Workflows",
                "Investigate and optimize slow workflows to improve performance",
                SuggestionPriority::Medium,
                ImplementationDifficulty::Moderate,
                EstimatedImpact::Medium,
            );
            analysis_result = analysis_result.with_recommendation(recommendation);
        }

        // Add findings for very slow workflows
        if !very_slow_workflows.is_empty() {
            let finding = AnalysisFinding::new(
                "very-slow-workflows",
                "Very Slow Workflows Detected",
                format!(
                    "{} workflows took longer than {}ms to execute",
                    very_slow_workflows.len(),
                    self.thresholds.very_slow_workflow_threshold_ms
                ),
                FindingSeverity::High,
            );
            analysis_result = analysis_result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "optimize-very-slow-workflows",
                "Optimize Very Slow Workflows",
                "Investigate and optimize very slow workflows to improve performance",
                SuggestionPriority::High,
                ImplementationDifficulty::Hard,
                EstimatedImpact::High,
            );
            analysis_result = analysis_result.with_recommendation(recommendation);
        }

        Ok(analysis_result)
    }
}
