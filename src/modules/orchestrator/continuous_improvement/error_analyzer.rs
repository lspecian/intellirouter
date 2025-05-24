//! Error Analyzer
//!
//! This module provides the ErrorAnalyzer implementation for analyzing error patterns.

use std::collections::HashMap;

use crate::modules::orchestrator::types::{OrchestratorError, Task, TaskResult, TaskStatus};
use crate::modules::orchestrator::workflow::{Workflow, WorkflowResult};

use super::analyzer::Analyzer;
use super::types::{
    AnalysisFinding, AnalysisRecommendation, AnalysisResult, EstimatedImpact, FindingSeverity,
    ImplementationDifficulty,
};
use crate::modules::orchestrator::reporting::SuggestionPriority;

/// Error analyzer
#[derive(Debug, Default)]
pub struct ErrorAnalyzer {
    /// Error thresholds
    thresholds: ErrorThresholds,
}

/// Error thresholds
#[derive(Debug, Clone)]
pub struct ErrorThresholds {
    /// High error rate threshold
    pub high_error_rate_threshold: f64,
    /// Very high error rate threshold
    pub very_high_error_rate_threshold: f64,
    /// Common error threshold (count)
    pub common_error_threshold: usize,
    /// Very common error threshold (count)
    pub very_common_error_threshold: usize,
}

impl Default for ErrorThresholds {
    fn default() -> Self {
        Self {
            high_error_rate_threshold: 0.2,
            very_high_error_rate_threshold: 0.5,
            common_error_threshold: 3,
            very_common_error_threshold: 10,
        }
    }
}

impl ErrorAnalyzer {
    /// Create a new error analyzer
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new error analyzer with custom thresholds
    pub fn with_thresholds(thresholds: ErrorThresholds) -> Self {
        Self { thresholds }
    }

    /// Count errors by type
    fn count_errors_by_type(&self, results: &[TaskResult]) -> HashMap<String, usize> {
        let mut error_counts = HashMap::new();

        for result in results {
            if result.status == TaskStatus::Failed {
                // Extract error from message or data
                let error_msg =
                    if result.message.contains("error") || result.message.contains("failed") {
                        result.message.clone()
                    } else if let Some(error) = result.data.get("error") {
                        error.clone()
                    } else {
                        continue;
                    };

                let error_type = self.extract_error_type(&error_msg);
                *error_counts.entry(error_type).or_insert(0) += 1;
            }
        }

        error_counts
    }

    /// Count workflow errors by type
    fn count_workflow_errors_by_type(&self, results: &[WorkflowResult]) -> HashMap<String, usize> {
        let mut error_counts = HashMap::new();

        for result in results {
            if !result.failed_tasks.is_empty() || result.error_message.is_some() {
                // Extract error from error_message
                let error_msg = if let Some(error) = &result.error_message {
                    error
                } else {
                    "Failed tasks"
                };

                let error_type = self.extract_error_type(error_msg);
                *error_counts.entry(error_type).or_insert(0) += 1;
            }
        }

        error_counts
    }

    /// Extract error type from error message
    fn extract_error_type(&self, error: &str) -> String {
        // This is a simple implementation that extracts the first word of the error message
        // In a real implementation, this would use more sophisticated error classification
        error
            .split_whitespace()
            .next()
            .unwrap_or("Unknown")
            .to_string()
    }
}

impl Analyzer for ErrorAnalyzer {
    fn analyze_tasks(&self, tasks: &[Task]) -> Result<AnalysisResult, OrchestratorError> {
        let mut result = AnalysisResult::new(
            "error-tasks",
            "Error Analysis (Tasks)",
            "Analysis of task errors",
        );

        // Calculate metrics
        let total_tasks = tasks.len();
        let failed_tasks = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Failed)
            .count();
        let error_rate = if total_tasks > 0 {
            failed_tasks as f64 / total_tasks as f64
        } else {
            0.0
        };

        // Add metrics to result
        result = result
            .with_metric("total_tasks", total_tasks as f64)
            .with_metric("failed_tasks", failed_tasks as f64)
            .with_metric("error_rate", error_rate);

        // Check for high error rate
        if error_rate > self.thresholds.very_high_error_rate_threshold {
            let finding = AnalysisFinding::new(
                "very-high-task-error-rate",
                "Very High Task Error Rate",
                format!(
                    "The task error rate is very high ({:.1}%)",
                    error_rate * 100.0
                ),
                FindingSeverity::Critical,
            );
            result = result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "reduce-task-errors",
                "Reduce Task Errors",
                "Investigate and fix issues causing tasks to fail",
                SuggestionPriority::High,
                ImplementationDifficulty::Hard,
                EstimatedImpact::VeryHigh,
            );
            result = result.with_recommendation(recommendation);
        } else if error_rate > self.thresholds.high_error_rate_threshold {
            let finding = AnalysisFinding::new(
                "high-task-error-rate",
                "High Task Error Rate",
                format!("The task error rate is high ({:.1}%)", error_rate * 100.0),
                FindingSeverity::High,
            );
            result = result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "reduce-task-errors",
                "Reduce Task Errors",
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
            "error-workflows",
            "Error Analysis (Workflows)",
            "Analysis of workflow errors",
        );

        // Calculate metrics
        let total_workflows = workflows.len();
        let failed_workflows = workflows
            .iter()
            .filter(|w| w.metadata.get("status").map_or(false, |s| s == "failed"))
            .count();
        let error_rate = if total_workflows > 0 {
            failed_workflows as f64 / total_workflows as f64
        } else {
            0.0
        };

        // Add metrics to result
        result = result
            .with_metric("total_workflows", total_workflows as f64)
            .with_metric("failed_workflows", failed_workflows as f64)
            .with_metric("error_rate", error_rate);

        // Check for high error rate
        if error_rate > self.thresholds.very_high_error_rate_threshold {
            let finding = AnalysisFinding::new(
                "very-high-workflow-error-rate",
                "Very High Workflow Error Rate",
                format!(
                    "The workflow error rate is very high ({:.1}%)",
                    error_rate * 100.0
                ),
                FindingSeverity::Critical,
            );
            result = result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "reduce-workflow-errors",
                "Reduce Workflow Errors",
                "Investigate and fix issues causing workflows to fail",
                SuggestionPriority::High,
                ImplementationDifficulty::Hard,
                EstimatedImpact::VeryHigh,
            );
            result = result.with_recommendation(recommendation);
        } else if error_rate > self.thresholds.high_error_rate_threshold {
            let finding = AnalysisFinding::new(
                "high-workflow-error-rate",
                "High Workflow Error Rate",
                format!(
                    "The workflow error rate is high ({:.1}%)",
                    error_rate * 100.0
                ),
                FindingSeverity::High,
            );
            result = result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "reduce-workflow-errors",
                "Reduce Workflow Errors",
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
            "error-task-results",
            "Error Analysis (Task Results)",
            "Analysis of task result errors",
        );

        // Calculate metrics
        let total_results = results.len();
        let failed_results = results
            .iter()
            .filter(|r| r.status == TaskStatus::Failed)
            .count();
        let error_rate = if total_results > 0 {
            failed_results as f64 / total_results as f64
        } else {
            0.0
        };

        // Count errors by type
        let error_counts = self.count_errors_by_type(results);

        // Add metrics to result
        analysis_result = analysis_result
            .with_metric("total_results", total_results as f64)
            .with_metric("failed_results", failed_results as f64)
            .with_metric("error_rate", error_rate);

        for (error_type, count) in &error_counts {
            analysis_result = analysis_result.with_metric(
                format!("error_count_{}", error_type.to_lowercase()),
                *count as f64,
            );
        }

        // Check for high error rate
        if error_rate > self.thresholds.very_high_error_rate_threshold {
            let finding = AnalysisFinding::new(
                "very-high-task-result-error-rate",
                "Very High Task Result Error Rate",
                format!(
                    "The task result error rate is very high ({:.1}%)",
                    error_rate * 100.0
                ),
                FindingSeverity::Critical,
            );
            analysis_result = analysis_result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "reduce-task-result-errors",
                "Reduce Task Result Errors",
                "Investigate and fix issues causing task results to fail",
                SuggestionPriority::High,
                ImplementationDifficulty::Hard,
                EstimatedImpact::VeryHigh,
            );
            analysis_result = analysis_result.with_recommendation(recommendation);
        } else if error_rate > self.thresholds.high_error_rate_threshold {
            let finding = AnalysisFinding::new(
                "high-task-result-error-rate",
                "High Task Result Error Rate",
                format!(
                    "The task result error rate is high ({:.1}%)",
                    error_rate * 100.0
                ),
                FindingSeverity::High,
            );
            analysis_result = analysis_result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "reduce-task-result-errors",
                "Reduce Task Result Errors",
                "Investigate and fix issues causing task results to fail",
                SuggestionPriority::Medium,
                ImplementationDifficulty::Moderate,
                EstimatedImpact::High,
            );
            analysis_result = analysis_result.with_recommendation(recommendation);
        }

        // Check for common errors
        for (error_type, count) in &error_counts {
            if *count >= self.thresholds.very_common_error_threshold {
                let finding = AnalysisFinding::new(
                    format!("very-common-error-{}", error_type.to_lowercase()),
                    format!("Very Common Error: {}", error_type),
                    format!("The error '{}' occurred {} times", error_type, count),
                    FindingSeverity::Critical,
                );
                analysis_result = analysis_result.with_finding(finding);

                let recommendation = AnalysisRecommendation::new(
                    format!("fix-common-error-{}", error_type.to_lowercase()),
                    format!("Fix Common Error: {}", error_type),
                    format!("Investigate and fix the '{}' error", error_type),
                    SuggestionPriority::High,
                    ImplementationDifficulty::Moderate,
                    EstimatedImpact::VeryHigh,
                );
                analysis_result = analysis_result.with_recommendation(recommendation);
            } else if *count >= self.thresholds.common_error_threshold {
                let finding = AnalysisFinding::new(
                    format!("common-error-{}", error_type.to_lowercase()),
                    format!("Common Error: {}", error_type),
                    format!("The error '{}' occurred {} times", error_type, count),
                    FindingSeverity::High,
                );
                analysis_result = analysis_result.with_finding(finding);

                let recommendation = AnalysisRecommendation::new(
                    format!("fix-common-error-{}", error_type.to_lowercase()),
                    format!("Fix Common Error: {}", error_type),
                    format!("Investigate and fix the '{}' error", error_type),
                    SuggestionPriority::Medium,
                    ImplementationDifficulty::Moderate,
                    EstimatedImpact::High,
                );
                analysis_result = analysis_result.with_recommendation(recommendation);
            }
        }

        Ok(analysis_result)
    }

    fn analyze_workflow_results(
        &self,
        results: &[WorkflowResult],
    ) -> Result<AnalysisResult, OrchestratorError> {
        let mut analysis_result = AnalysisResult::new(
            "error-workflow-results",
            "Error Analysis (Workflow Results)",
            "Analysis of workflow result errors",
        );

        // Calculate metrics
        let total_results = results.len();
        let failed_results = results
            .iter()
            .filter(|r| !r.failed_tasks.is_empty() || r.error_message.is_some())
            .count();
        let error_rate = if total_results > 0 {
            failed_results as f64 / total_results as f64
        } else {
            0.0
        };

        // Count errors by type
        let error_counts = self.count_workflow_errors_by_type(results);

        // Add metrics to result
        analysis_result = analysis_result
            .with_metric("total_results", total_results as f64)
            .with_metric("failed_results", failed_results as f64)
            .with_metric("error_rate", error_rate);

        for (error_type, count) in &error_counts {
            analysis_result = analysis_result.with_metric(
                format!("error_count_{}", error_type.to_lowercase()),
                *count as f64,
            );
        }

        // Check for high error rate
        if error_rate > self.thresholds.very_high_error_rate_threshold {
            let finding = AnalysisFinding::new(
                "very-high-workflow-result-error-rate",
                "Very High Workflow Result Error Rate",
                format!(
                    "The workflow result error rate is very high ({:.1}%)",
                    error_rate * 100.0
                ),
                FindingSeverity::Critical,
            );
            analysis_result = analysis_result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "reduce-workflow-result-errors",
                "Reduce Workflow Result Errors",
                "Investigate and fix issues causing workflow results to fail",
                SuggestionPriority::High,
                ImplementationDifficulty::Hard,
                EstimatedImpact::VeryHigh,
            );
            analysis_result = analysis_result.with_recommendation(recommendation);
        } else if error_rate > self.thresholds.high_error_rate_threshold {
            let finding = AnalysisFinding::new(
                "high-workflow-result-error-rate",
                "High Workflow Result Error Rate",
                format!(
                    "The workflow result error rate is high ({:.1}%)",
                    error_rate * 100.0
                ),
                FindingSeverity::High,
            );
            analysis_result = analysis_result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "reduce-workflow-result-errors",
                "Reduce Workflow Result Errors",
                "Investigate and fix issues causing workflow results to fail",
                SuggestionPriority::Medium,
                ImplementationDifficulty::Moderate,
                EstimatedImpact::High,
            );
            analysis_result = analysis_result.with_recommendation(recommendation);
        }

        // Check for common errors
        for (error_type, count) in &error_counts {
            if *count >= self.thresholds.very_common_error_threshold {
                let finding = AnalysisFinding::new(
                    format!("very-common-workflow-error-{}", error_type.to_lowercase()),
                    format!("Very Common Workflow Error: {}", error_type),
                    format!("The error '{}' occurred {} times", error_type, count),
                    FindingSeverity::Critical,
                );
                analysis_result = analysis_result.with_finding(finding);

                let recommendation = AnalysisRecommendation::new(
                    format!("fix-common-workflow-error-{}", error_type.to_lowercase()),
                    format!("Fix Common Workflow Error: {}", error_type),
                    format!("Investigate and fix the '{}' error", error_type),
                    SuggestionPriority::High,
                    ImplementationDifficulty::Moderate,
                    EstimatedImpact::VeryHigh,
                );
                analysis_result = analysis_result.with_recommendation(recommendation);
            } else if *count >= self.thresholds.common_error_threshold {
                let finding = AnalysisFinding::new(
                    format!("common-workflow-error-{}", error_type.to_lowercase()),
                    format!("Common Workflow Error: {}", error_type),
                    format!("The error '{}' occurred {} times", error_type, count),
                    FindingSeverity::High,
                );
                analysis_result = analysis_result.with_finding(finding);

                let recommendation = AnalysisRecommendation::new(
                    format!("fix-common-workflow-error-{}", error_type.to_lowercase()),
                    format!("Fix Common Workflow Error: {}", error_type),
                    format!("Investigate and fix the '{}' error", error_type),
                    SuggestionPriority::Medium,
                    ImplementationDifficulty::Moderate,
                    EstimatedImpact::High,
                );
                analysis_result = analysis_result.with_recommendation(recommendation);
            }
        }

        Ok(analysis_result)
    }
}
