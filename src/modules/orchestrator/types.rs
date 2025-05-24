//! Orchestrator Types
//!
//! This module defines the core data structures for the Boomerang orchestration functionality.

use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

/// Specialized mode for handling different aspects of testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Debug mode for diagnosing and resolving issues
    Debug,
    /// Code mode for implementing testing frameworks and infrastructure
    Code,
    /// Test mode for developing and executing test cases
    Test,
    /// Boomerang mode for orchestrating across other modes
    Boomerang,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Debug => write!(f, "Debug"),
            Mode::Code => write!(f, "Code"),
            Mode::Test => write!(f, "Test"),
            Mode::Boomerang => write!(f, "Boomerang"),
        }
    }
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    /// Task is pending execution
    Pending,
    /// Task is currently in progress
    InProgress,
    /// Task has been completed successfully
    Completed,
    /// Task has failed
    Failed,
    /// Task has been cancelled
    Cancelled,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "Pending"),
            TaskStatus::InProgress => write!(f, "In Progress"),
            TaskStatus::Completed => write!(f, "Completed"),
            TaskStatus::Failed => write!(f, "Failed"),
            TaskStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Task result
#[derive(Debug, Clone)]
pub struct TaskResult {
    /// Task ID
    pub task_id: String,
    /// Task status
    pub status: TaskStatus,
    /// Result message
    pub message: String,
    /// Additional data
    pub data: HashMap<String, String>,
}

impl TaskResult {
    /// Create a new task result
    pub fn new(task_id: impl Into<String>, status: TaskStatus, message: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            status,
            message: message.into(),
            data: HashMap::new(),
        }
    }

    /// Add data to the task result
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }
}

/// Task to be delegated to a specialized mode
#[derive(Debug, Clone)]
pub struct Task {
    /// Task ID
    pub id: String,
    /// Task title
    pub title: String,
    /// Task description
    pub description: String,
    /// Task mode
    pub mode: Mode,
    /// Task status
    pub status: TaskStatus,
    /// Task dependencies
    pub dependencies: Vec<String>,
    /// Task result
    pub result: Option<TaskResult>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Task {
    /// Create a new task
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        mode: Mode,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            mode,
            status: TaskStatus::Pending,
            dependencies: Vec::new(),
            result: None,
            metadata: HashMap::new(),
        }
    }

    /// Add a dependency to the task
    pub fn with_dependency(mut self, dependency_id: impl Into<String>) -> Self {
        self.dependencies.push(dependency_id.into());
        self
    }

    /// Add metadata to the task
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set the task status
    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
    }

    /// Set the task result
    pub fn set_result(&mut self, result: TaskResult) {
        self.status = result.status;
        self.result = Some(result);
    }
}

/// Delegation error
#[derive(Debug, Error)]
pub enum DelegationError {
    /// Invalid mode
    #[error("Invalid mode: {0}")]
    InvalidMode(String),
    /// Task not found
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    /// Dependency not satisfied
    #[error("Dependency not satisfied: {0}")]
    DependencyNotSatisfied(String),
    /// Mode not available
    #[error("Mode not available: {0}")]
    ModeNotAvailable(String),
    /// Other error
    #[error("Delegation error: {0}")]
    Other(String),
}

/// Integration error
#[derive(Debug, Error)]
pub enum IntegrationError {
    /// Invalid result format
    #[error("Invalid result format: {0}")]
    InvalidResultFormat(String),
    /// Result not found
    #[error("Result not found: {0}")]
    ResultNotFound(String),
    /// Aggregation failed
    #[error("Aggregation failed: {0}")]
    AggregationFailed(String),
    /// Other error
    #[error("Integration error: {0}")]
    Other(String),
}

/// Workflow error
#[derive(Debug, Error)]
pub enum WorkflowError {
    /// Invalid workflow
    #[error("Invalid workflow: {0}")]
    InvalidWorkflow(String),
    /// Workflow execution failed
    #[error("Workflow execution failed: {0}")]
    ExecutionFailed(String),
    /// Dependency cycle detected
    #[error("Dependency cycle detected: {0}")]
    DependencyCycle(String),
    /// Other error
    #[error("Workflow error: {0}")]
    Other(String),
}

/// Reporting error
#[derive(Debug, Error)]
pub enum ReportingError {
    /// Invalid report format
    #[error("Invalid report format: {0}")]
    InvalidReportFormat(String),
    /// Report generation failed
    #[error("Report generation failed: {0}")]
    GenerationFailed(String),
    /// Report generation error
    #[error("Report generation error: {0}")]
    ReportGenerationError(String),
    /// Other error
    #[error("Reporting error: {0}")]
    Other(String),
}

/// Orchestrator error
#[derive(Debug, Error)]
pub enum OrchestratorError {
    /// Delegation error
    #[error("Delegation error: {0}")]
    Delegation(#[from] DelegationError),
    /// Integration error
    #[error("Integration error: {0}")]
    Integration(#[from] IntegrationError),
    /// Workflow error
    #[error("Workflow error: {0}")]
    Workflow(#[from] WorkflowError),
    /// Reporting error
    #[error("Reporting error: {0}")]
    Reporting(#[from] ReportingError),
    /// Other error
    #[error("Orchestrator error: {0}")]
    Other(String),
}
