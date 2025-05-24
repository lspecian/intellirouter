//! Coverage Analyzer
//!
//! This module provides the CoverageAnalyzer implementation for analyzing test coverage.

use std::collections::HashMap;

use crate::modules::orchestrator::types::{OrchestratorError, Task, TaskResult, TaskStatus};
use crate::modules::orchestrator::workflow::{Workflow, WorkflowResult};

use super::analyzer::Analyzer;
use super::types::{
    AnalysisFinding, AnalysisRecommendation, AnalysisResult, EstimatedImpact, FindingSeverity,
    ImplementationDifficulty,
};
use crate::modules::orchestrator::reporting::SuggestionPriority;

/// Coverage analyzer
#[derive(Debug, Default)]
pub struct CoverageAnalyzer {
    /// Coverage thresholds
    thresholds: CoverageThresholds,
}

/// Coverage thresholds
#[derive(Debug, Clone)]
pub struct CoverageThresholds {
    /// Low mode coverage threshold
    pub low_mode_coverage_threshold: f64,
    /// Very low mode coverage threshold
    pub very_low_mode_coverage_threshold: f64,
    /// Low feature coverage threshold
    pub low_feature_coverage_threshold: f64,
    /// Very low feature coverage threshold
    pub very_low_feature_coverage_threshold: f64,
}

impl Default for CoverageThresholds {
    fn default() -> Self {
        Self {
            low_mode_coverage_threshold: 0.8,
            very_low_mode_coverage_threshold: 0.5,
            low_feature_coverage_threshold: 0.8,
            very_low_feature_coverage_threshold: 0.5,
        }
    }
}

impl CoverageAnalyzer {
    /// Create a new coverage analyzer
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new coverage analyzer with custom thresholds
    pub fn with_thresholds(thresholds: CoverageThresholds) -> Self {
        Self { thresholds }
    }

    /// Calculate mode coverage
    fn calculate_mode_coverage(&self, tasks: &[Task]) -> HashMap<String, f64> {
        let mut mode_counts: HashMap<String, usize> = HashMap::new();
        let mut mode_totals: HashMap<String, usize> = HashMap::new();

        for task in tasks {
            let mode = format!("{:?}", task.mode);
            *mode_totals.entry(mode.clone()).or_insert(0) += 1;

            if task.status == TaskStatus::Completed {
                *mode_counts.entry(mode.clone()).or_insert(0) += 1;
            }
        }

        let mut mode_coverage = HashMap::new();
        for (mode, total) in mode_totals {
            let count = *mode_counts.get(&mode).unwrap_or(&0);
            let coverage = if total > 0 {
                count as f64 / total as f64
            } else {
                0.0
            };
            mode_coverage.insert(mode, coverage);
        }

        mode_coverage
    }

    /// Calculate feature coverage
    fn calculate_feature_coverage(&self, tasks: &[Task]) -> HashMap<String, f64> {
        let mut feature_counts: HashMap<String, usize> = HashMap::new();
        let mut feature_totals: HashMap<String, usize> = HashMap::new();

        for task in tasks {
            // Extract features from metadata instead of tags
            if let Some(features_str) = task.metadata.get("features") {
                for feature in features_str.split(',').map(|s| s.trim().to_string()) {
                    if !feature.is_empty() {
                        *feature_totals.entry(feature.clone()).or_insert(0) += 1;

                        if task.status == TaskStatus::Completed {
                            *feature_counts.entry(feature.clone()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        let mut feature_coverage = HashMap::new();
        for (feature, total) in feature_totals {
            let count = *feature_counts.get(&feature).unwrap_or(&0);
            let coverage = if total > 0 {
                count as f64 / total as f64
            } else {
                0.0
            };
            feature_coverage.insert(feature, coverage);
        }

        feature_coverage
    }
}

impl Analyzer for CoverageAnalyzer {
    fn analyze_tasks(&self, tasks: &[Task]) -> Result<AnalysisResult, OrchestratorError> {
        let mut result = AnalysisResult::new(
            "coverage-tasks",
            "Coverage Analysis (Tasks)",
            "Analysis of task coverage",
        );

        // Calculate mode coverage
        let mode_coverage = self.calculate_mode_coverage(tasks);

        // Calculate feature coverage
        let feature_coverage = self.calculate_feature_coverage(tasks);

        // Add metrics to result
        for (mode, coverage) in &mode_coverage {
            result = result.with_metric(format!("mode_coverage_{}", mode), *coverage);
        }

        for (feature, coverage) in &feature_coverage {
            result = result.with_metric(format!("feature_coverage_{}", feature), *coverage);
        }

        // Check for low mode coverage
        for (mode, coverage) in &mode_coverage {
            if *coverage < self.thresholds.very_low_mode_coverage_threshold {
                let finding = AnalysisFinding::new(
                    format!("very-low-mode-coverage-{}", mode),
                    format!("Very Low Coverage for Mode '{}'", mode),
                    format!(
                        "The coverage for mode '{}' is very low ({:.1}%)",
                        mode,
                        coverage * 100.0
                    ),
                    FindingSeverity::Critical,
                );
                result = result.with_finding(finding);

                let recommendation = AnalysisRecommendation::new(
                    format!("improve-mode-coverage-{}", mode),
                    format!("Improve Coverage for Mode '{}'", mode),
                    format!("Add more tests for mode '{}'", mode),
                    SuggestionPriority::High,
                    ImplementationDifficulty::Moderate,
                    EstimatedImpact::VeryHigh,
                );
                result = result.with_recommendation(recommendation);
            } else if *coverage < self.thresholds.low_mode_coverage_threshold {
                let finding = AnalysisFinding::new(
                    format!("low-mode-coverage-{}", mode),
                    format!("Low Coverage for Mode '{}'", mode),
                    format!(
                        "The coverage for mode '{}' is low ({:.1}%)",
                        mode,
                        coverage * 100.0
                    ),
                    FindingSeverity::High,
                );
                result = result.with_finding(finding);

                let recommendation = AnalysisRecommendation::new(
                    format!("improve-mode-coverage-{}", mode),
                    format!("Improve Coverage for Mode '{}'", mode),
                    format!("Add more tests for mode '{}'", mode),
                    SuggestionPriority::Medium,
                    ImplementationDifficulty::Moderate,
                    EstimatedImpact::High,
                );
                result = result.with_recommendation(recommendation);
            }
        }

        // Check for low feature coverage
        for (feature, coverage) in &feature_coverage {
            if *coverage < self.thresholds.very_low_feature_coverage_threshold {
                let finding = AnalysisFinding::new(
                    format!("very-low-feature-coverage-{}", feature),
                    format!("Very Low Coverage for Feature '{}'", feature),
                    format!(
                        "The coverage for feature '{}' is very low ({:.1}%)",
                        feature,
                        coverage * 100.0
                    ),
                    FindingSeverity::Critical,
                );
                result = result.with_finding(finding);

                let recommendation = AnalysisRecommendation::new(
                    format!("improve-feature-coverage-{}", feature),
                    format!("Improve Coverage for Feature '{}'", feature),
                    format!("Add more tests for feature '{}'", feature),
                    SuggestionPriority::High,
                    ImplementationDifficulty::Moderate,
                    EstimatedImpact::VeryHigh,
                );
                result = result.with_recommendation(recommendation);
            } else if *coverage < self.thresholds.low_feature_coverage_threshold {
                let finding = AnalysisFinding::new(
                    format!("low-feature-coverage-{}", feature),
                    format!("Low Coverage for Feature '{}'", feature),
                    format!(
                        "The coverage for feature '{}' is low ({:.1}%)",
                        feature,
                        coverage * 100.0
                    ),
                    FindingSeverity::High,
                );
                result = result.with_finding(finding);

                let recommendation = AnalysisRecommendation::new(
                    format!("improve-feature-coverage-{}", feature),
                    format!("Improve Coverage for Feature '{}'", feature),
                    format!("Add more tests for feature '{}'", feature),
                    SuggestionPriority::Medium,
                    ImplementationDifficulty::Moderate,
                    EstimatedImpact::High,
                );
                result = result.with_recommendation(recommendation);
            }
        }

        Ok(result)
    }

    fn analyze_workflows(
        &self,
        workflows: &[Workflow],
    ) -> Result<AnalysisResult, OrchestratorError> {
        let mut result = AnalysisResult::new(
            "coverage-workflows",
            "Coverage Analysis (Workflows)",
            "Analysis of workflow coverage",
        );

        // Calculate metrics
        let total_workflows = workflows.len();
        let completed_workflows = workflows
            .iter()
            .filter(|w| {
                // Check if the workflow is completed based on metadata
                w.metadata.get("status").map_or(false, |s| s == "completed")
            })
            .count();
        let coverage = if total_workflows > 0 {
            completed_workflows as f64 / total_workflows as f64
        } else {
            0.0
        };

        // Add metrics to result
        result = result
            .with_metric("total_workflows", total_workflows as f64)
            .with_metric("completed_workflows", completed_workflows as f64)
            .with_metric("workflow_coverage", coverage);

        // Check for low workflow coverage
        if coverage < self.thresholds.very_low_feature_coverage_threshold {
            let finding = AnalysisFinding::new(
                "very-low-workflow-coverage",
                "Very Low Workflow Coverage",
                format!(
                    "The workflow coverage is very low ({:.1}%)",
                    coverage * 100.0
                ),
                FindingSeverity::Critical,
            );
            result = result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "improve-workflow-coverage",
                "Improve Workflow Coverage",
                "Add more workflow tests",
                SuggestionPriority::High,
                ImplementationDifficulty::Moderate,
                EstimatedImpact::VeryHigh,
            );
            result = result.with_recommendation(recommendation);
        } else if coverage < self.thresholds.low_feature_coverage_threshold {
            let finding = AnalysisFinding::new(
                "low-workflow-coverage",
                "Low Workflow Coverage",
                format!("The workflow coverage is low ({:.1}%)", coverage * 100.0),
                FindingSeverity::High,
            );
            result = result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "improve-workflow-coverage",
                "Improve Workflow Coverage",
                "Add more workflow tests",
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
        let mut result = AnalysisResult::new(
            "coverage-task-results",
            "Coverage Analysis (Task Results)",
            "Analysis of task result coverage",
        );

        // Calculate metrics
        let total_results = results.len();
        let successful_results = results
            .iter()
            .filter(|r| r.status == TaskStatus::Completed)
            .count();
        let coverage = if total_results > 0 {
            successful_results as f64 / total_results as f64
        } else {
            0.0
        };

        // Add metrics to result
        result = result
            .with_metric("total_results", total_results as f64)
            .with_metric("successful_results", successful_results as f64)
            .with_metric("task_result_coverage", coverage);

        // Check for low task result coverage
        if coverage < self.thresholds.very_low_feature_coverage_threshold {
            let finding = AnalysisFinding::new(
                "very-low-task-result-coverage",
                "Very Low Task Result Coverage",
                format!(
                    "The task result coverage is very low ({:.1}%)",
                    coverage * 100.0
                ),
                FindingSeverity::Critical,
            );
            result = result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "improve-task-result-coverage",
                "Improve Task Result Coverage",
                "Investigate and fix failing task results",
                SuggestionPriority::High,
                ImplementationDifficulty::Moderate,
                EstimatedImpact::VeryHigh,
            );
            result = result.with_recommendation(recommendation);
        } else if coverage < self.thresholds.low_feature_coverage_threshold {
            let finding = AnalysisFinding::new(
                "low-task-result-coverage",
                "Low Task Result Coverage",
                format!("The task result coverage is low ({:.1}%)", coverage * 100.0),
                FindingSeverity::High,
            );
            result = result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "improve-task-result-coverage",
                "Improve Task Result Coverage",
                "Investigate and fix failing task results",
                SuggestionPriority::Medium,
                ImplementationDifficulty::Moderate,
                EstimatedImpact::High,
            );
            result = result.with_recommendation(recommendation);
        }

        Ok(result)
    }

    fn analyze_workflow_results(
        &self,
        results: &[WorkflowResult],
    ) -> Result<AnalysisResult, OrchestratorError> {
        let mut result = AnalysisResult::new(
            "coverage-workflow-results",
            "Coverage Analysis (Workflow Results)",
            "Analysis of workflow result coverage",
        );

        // Calculate metrics
        let total_results = results.len();
        let successful_results = results
            .iter()
            .filter(|r| {
                // Check if the workflow result is successful based on completed_tasks vs. failed_tasks
                r.completed_tasks > r.failed_tasks
            })
            .count();
        let coverage = if total_results > 0 {
            successful_results as f64 / total_results as f64
        } else {
            0.0
        };

        // Add metrics to result
        result = result
            .with_metric("total_results", total_results as f64)
            .with_metric("successful_results", successful_results as f64)
            .with_metric("workflow_result_coverage", coverage);

        // Check for low workflow result coverage
        if coverage < self.thresholds.very_low_feature_coverage_threshold {
            let finding = AnalysisFinding::new(
                "very-low-workflow-result-coverage",
                "Very Low Workflow Result Coverage",
                format!(
                    "The workflow result coverage is very low ({:.1}%)",
                    coverage * 100.0
                ),
                FindingSeverity::Critical,
            );
            result = result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "improve-workflow-result-coverage",
                "Improve Workflow Result Coverage",
                "Investigate and fix failing workflow results",
                SuggestionPriority::High,
                ImplementationDifficulty::Moderate,
                EstimatedImpact::VeryHigh,
            );
            result = result.with_recommendation(recommendation);
        } else if coverage < self.thresholds.low_feature_coverage_threshold {
            let finding = AnalysisFinding::new(
                "low-workflow-result-coverage",
                "Low Workflow Result Coverage",
                format!(
                    "The workflow result coverage is low ({:.1}%)",
                    coverage * 100.0
                ),
                FindingSeverity::High,
            );
            result = result.with_finding(finding);

            let recommendation = AnalysisRecommendation::new(
                "improve-workflow-result-coverage",
                "Improve Workflow Result Coverage",
                "Investigate and fix failing workflow results",
                SuggestionPriority::Medium,
                ImplementationDifficulty::Moderate,
                EstimatedImpact::High,
            );
            result = result.with_recommendation(recommendation);
        }

        Ok(result)
    }
}
