//! Continuous Improvement Types
//!
//! This module provides type definitions for the continuous improvement system.

use std::collections::HashMap;
use std::time::Instant;

use crate::modules::orchestrator::reporting::{
    ImprovementSuggestion, SuggestionPriority, SuggestionStatus,
};

/// Analysis result
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Analysis ID
    pub id: String,
    /// Analysis name
    pub name: String,
    /// Analysis description
    pub description: String,
    /// Analysis timestamp
    pub timestamp: Instant,
    /// Analysis metrics
    pub metrics: HashMap<String, f64>,
    /// Analysis findings
    pub findings: Vec<AnalysisFinding>,
    /// Analysis recommendations
    pub recommendations: Vec<AnalysisRecommendation>,
}

impl AnalysisResult {
    /// Create a new analysis result
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            timestamp: Instant::now(),
            metrics: HashMap::new(),
            findings: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    /// Add a metric
    pub fn with_metric(mut self, key: impl Into<String>, value: f64) -> Self {
        self.metrics.insert(key.into(), value);
        self
    }

    /// Add a finding
    pub fn with_finding(mut self, finding: AnalysisFinding) -> Self {
        self.findings.push(finding);
        self
    }

    /// Add a recommendation
    pub fn with_recommendation(mut self, recommendation: AnalysisRecommendation) -> Self {
        self.recommendations.push(recommendation);
        self
    }
}

/// Analysis finding
#[derive(Debug, Clone)]
pub struct AnalysisFinding {
    /// Finding ID
    pub id: String,
    /// Finding title
    pub title: String,
    /// Finding description
    pub description: String,
    /// Finding severity
    pub severity: FindingSeverity,
    /// Related tasks
    pub related_tasks: Vec<String>,
    /// Related workflows
    pub related_workflows: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl AnalysisFinding {
    /// Create a new analysis finding
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        severity: FindingSeverity,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            severity,
            related_tasks: Vec::new(),
            related_workflows: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a related task
    pub fn with_related_task(mut self, task_id: impl Into<String>) -> Self {
        self.related_tasks.push(task_id.into());
        self
    }

    /// Add a related workflow
    pub fn with_related_workflow(mut self, workflow_id: impl Into<String>) -> Self {
        self.related_workflows.push(workflow_id.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Finding severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FindingSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Analysis recommendation
#[derive(Debug, Clone)]
pub struct AnalysisRecommendation {
    /// Recommendation ID
    pub id: String,
    /// Recommendation title
    pub title: String,
    /// Recommendation description
    pub description: String,
    /// Recommendation priority
    pub priority: SuggestionPriority,
    /// Implementation difficulty
    pub difficulty: ImplementationDifficulty,
    /// Estimated impact
    pub impact: EstimatedImpact,
    /// Related tasks
    pub related_tasks: Vec<String>,
    /// Related workflows
    pub related_workflows: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl AnalysisRecommendation {
    /// Create a new analysis recommendation
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        priority: SuggestionPriority,
        difficulty: ImplementationDifficulty,
        impact: EstimatedImpact,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            priority,
            difficulty,
            impact,
            related_tasks: Vec::new(),
            related_workflows: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a related task
    pub fn with_related_task(mut self, task_id: impl Into<String>) -> Self {
        self.related_tasks.push(task_id.into());
        self
    }

    /// Add a related workflow
    pub fn with_related_workflow(mut self, workflow_id: impl Into<String>) -> Self {
        self.related_workflows.push(workflow_id.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Convert to an improvement suggestion
    pub fn to_improvement_suggestion(&self) -> ImprovementSuggestion {
        ImprovementSuggestion {
            id: self.id.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            affected_tasks: self.related_tasks.clone(),
            affected_workflows: self.related_workflows.clone(),
            priority: self.priority,
            status: SuggestionStatus::Pending,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

/// Implementation difficulty
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImplementationDifficulty {
    /// Easy implementation
    Easy,
    /// Moderate implementation
    Moderate,
    /// Hard implementation
    Hard,
    /// Very hard implementation
    VeryHard,
}

/// Estimated impact
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EstimatedImpact {
    /// Low impact
    Low,
    /// Medium impact
    Medium,
    /// High impact
    High,
    /// Very high impact
    VeryHigh,
}
