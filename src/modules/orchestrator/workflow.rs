//! Workflow Management
//!
//! This module provides functionality for managing workflows across different modes,
//! handling dynamic task dependencies, and coordinating execution flow.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use crate::modules::orchestrator::architecture::OrchestratorArchitecture;
use crate::modules::orchestrator::types::{
    OrchestratorError, TaskStatus, WorkflowError,
};

/// Workflow definition
#[derive(Debug, Clone)]
pub struct Workflow {
    /// Workflow ID
    pub id: String,
    /// Workflow name
    pub name: String,
    /// Workflow description
    pub description: String,
    /// Task IDs in the workflow
    pub task_ids: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Workflow {
    /// Create a new workflow
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            task_ids: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a task to the workflow
    pub fn with_task(mut self, task_id: impl Into<String>) -> Self {
        self.task_ids.push(task_id.into());
        self
    }

    /// Add metadata to the workflow
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Workflow execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowStatus {
    /// Workflow is pending execution
    Pending,
    /// Workflow is currently in progress
    InProgress,
    /// Workflow has been completed successfully
    Completed,
    /// Workflow has failed
    Failed,
    /// Workflow has been cancelled
    Cancelled,
}

/// Workflow execution result
#[derive(Debug, Clone)]
pub struct WorkflowResult {
    /// Workflow ID
    pub workflow_id: String,
    /// Workflow status
    pub status: WorkflowStatus,
    /// Completed tasks
    pub completed_tasks: Vec<String>,
    /// Failed tasks
    pub failed_tasks: Vec<String>,
    /// Pending tasks
    pub pending_tasks: Vec<String>,
    /// Error message (if any)
    pub error_message: Option<String>,
}

impl WorkflowResult {
    /// Create a new workflow result
    pub fn new(workflow_id: impl Into<String>, status: WorkflowStatus) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            status,
            completed_tasks: Vec::new(),
            failed_tasks: Vec::new(),
            pending_tasks: Vec::new(),
            error_message: None,
        }
    }

    /// Add a completed task
    pub fn with_completed_task(mut self, task_id: impl Into<String>) -> Self {
        self.completed_tasks.push(task_id.into());
        self
    }

    /// Add a failed task
    pub fn with_failed_task(mut self, task_id: impl Into<String>) -> Self {
        self.failed_tasks.push(task_id.into());
        self
    }

    /// Add a pending task
    pub fn with_pending_task(mut self, task_id: impl Into<String>) -> Self {
        self.pending_tasks.push(task_id.into());
        self
    }

    /// Set the error message
    pub fn with_error_message(mut self, message: impl Into<String>) -> Self {
        self.error_message = Some(message.into());
        self
    }
}

/// Workflow executor for executing workflows
pub struct WorkflowExecutor {
    /// Workflow
    workflow: Workflow,
    /// Task dependencies
    dependencies: HashMap<String, HashSet<String>>,
    /// Reverse dependencies
    reverse_dependencies: HashMap<String, HashSet<String>>,
    /// Task status
    task_status: HashMap<String, TaskStatus>,
    /// Execution order
    execution_order: Vec<String>,
}

impl WorkflowExecutor {
    /// Create a new workflow executor
    pub fn new(workflow: Workflow) -> Self {
        Self {
            workflow,
            dependencies: HashMap::new(),
            reverse_dependencies: HashMap::new(),
            task_status: HashMap::new(),
            execution_order: Vec::new(),
        }
    }

    /// Add a dependency
    pub fn add_dependency(
        &mut self,
        task_id: &str,
        dependency_id: &str,
    ) -> Result<(), WorkflowError> {
        // Ensure both tasks are in the workflow
        if !self.workflow.task_ids.contains(&task_id.to_string()) {
            return Err(WorkflowError::InvalidWorkflow(format!(
                "Task {} is not in the workflow",
                task_id
            )));
        }

        if !self.workflow.task_ids.contains(&dependency_id.to_string()) {
            return Err(WorkflowError::InvalidWorkflow(format!(
                "Dependency {} is not in the workflow",
                dependency_id
            )));
        }

        // Add the dependency
        self.dependencies
            .entry(task_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(dependency_id.to_string());

        // Add the reverse dependency
        self.reverse_dependencies
            .entry(dependency_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(task_id.to_string());

        Ok(())
    }

    /// Check for dependency cycles
    fn check_for_cycles(&self) -> Result<(), WorkflowError> {
        // Use a depth-first search to check for cycles
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();

        for task_id in &self.workflow.task_ids {
            if !visited.contains(task_id) {
                if self.has_cycle(task_id, &mut visited, &mut stack)? {
                    return Err(WorkflowError::DependencyCycle(format!(
                        "Dependency cycle detected in workflow {}",
                        self.workflow.id
                    )));
                }
            }
        }

        Ok(())
    }

    /// Check if a task has a dependency cycle
    fn has_cycle(
        &self,
        task_id: &str,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
    ) -> Result<bool, WorkflowError> {
        visited.insert(task_id.to_string());
        stack.insert(task_id.to_string());

        if let Some(dependencies) = self.dependencies.get(task_id) {
            for dependency_id in dependencies {
                if !visited.contains(dependency_id) {
                    if self.has_cycle(dependency_id, visited, stack)? {
                        return Ok(true);
                    }
                } else if stack.contains(dependency_id) {
                    return Ok(true);
                }
            }
        }

        stack.remove(task_id);
        Ok(false)
    }

    /// Compute the execution order
    fn compute_execution_order(&mut self) -> Result<(), WorkflowError> {
        // Check for cycles
        self.check_for_cycles()?;

        // Use a topological sort to compute the execution order
        let mut visited = HashSet::new();
        let mut order = Vec::new();

        for task_id in &self.workflow.task_ids {
            if !visited.contains(task_id) {
                self.topological_sort(task_id, &mut visited, &mut order)?;
            }
        }

        // Reverse the order to get the correct execution order
        order.reverse();
        self.execution_order = order;

        Ok(())
    }

    /// Perform a topological sort
    fn topological_sort(
        &self,
        task_id: &str,
        visited: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<(), WorkflowError> {
        visited.insert(task_id.to_string());

        if let Some(dependencies) = self.dependencies.get(task_id) {
            for dependency_id in dependencies {
                if !visited.contains(dependency_id) {
                    self.topological_sort(dependency_id, visited, order)?;
                }
            }
        }

        order.push(task_id.to_string());
        Ok(())
    }

    /// Execute the workflow
    pub fn execute(
        &mut self,
        orchestrator: &OrchestratorArchitecture,
    ) -> Result<WorkflowResult, OrchestratorError> {
        // Compute the execution order
        self.compute_execution_order()
            .map_err(OrchestratorError::Workflow)?;

        // Initialize task status
        for task_id in &self.workflow.task_ids {
            let task = orchestrator.get_task(task_id)?;
            if let Some(task) = task {
                self.task_status.insert(task_id.clone(), task.status);
            } else {
                return Err(OrchestratorError::Workflow(WorkflowError::InvalidWorkflow(
                    format!("Task {} not found", task_id),
                )));
            }
        }

        // Execute tasks in order
        let mut result = WorkflowResult::new(self.workflow.id.clone(), WorkflowStatus::InProgress);

        for task_id in &self.execution_order {
            // Check if all dependencies are satisfied
            let dependencies = self.dependencies.get(task_id).cloned().unwrap_or_default();

            let all_dependencies_satisfied = dependencies
                .iter()
                .all(|dep_id| self.task_status.get(dep_id) == Some(&TaskStatus::Completed));

            if !all_dependencies_satisfied {
                result = result.with_pending_task(task_id.clone());
                continue;
            }

            // Get the task
            let task = orchestrator.get_task(task_id)?;
            if let Some(task) = task {
                // Execute the task
                match task.status {
                    TaskStatus::Completed => {
                        result = result.with_completed_task(task_id.clone());
                    }
                    TaskStatus::Failed => {
                        result = result.with_failed_task(task_id.clone());
                    }
                    TaskStatus::Cancelled => {
                        result = result.with_failed_task(task_id.clone());
                    }
                    _ => {
                        // Delegate the task to the appropriate mode
                        orchestrator.delegate_task(task_id, task.mode)?;
                        result = result.with_pending_task(task_id.clone());
                    }
                }
            } else {
                return Err(OrchestratorError::Workflow(WorkflowError::InvalidWorkflow(
                    format!("Task {} not found", task_id),
                )));
            }
        }

        // Update the workflow status
        if result.failed_tasks.is_empty() && result.pending_tasks.is_empty() {
            result.status = WorkflowStatus::Completed;
        } else if !result.failed_tasks.is_empty() {
            result.status = WorkflowStatus::Failed;
        }

        Ok(result)
    }
}

/// Workflow manager for managing workflows
pub struct WorkflowManager {
    /// Workflows
    workflows: Mutex<HashMap<String, Workflow>>,
    /// Workflow results
    results: Mutex<HashMap<String, WorkflowResult>>,
}

impl WorkflowManager {
    /// Create a new workflow manager
    pub fn new() -> Self {
        Self {
            workflows: Mutex::new(HashMap::new()),
            results: Mutex::new(HashMap::new()),
        }
    }

    /// Register a workflow
    pub fn register_workflow(&self, workflow: Workflow) -> Result<(), OrchestratorError> {
        let mut workflows = self.workflows.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on workflows".to_string())
        })?;

        workflows.insert(workflow.id.clone(), workflow);
        Ok(())
    }

    /// Get a workflow
    pub fn get_workflow(&self, workflow_id: &str) -> Result<Option<Workflow>, OrchestratorError> {
        let workflows = self.workflows.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on workflows".to_string())
        })?;

        Ok(workflows.get(workflow_id).cloned())
    }

    /// Execute a workflow
    pub fn execute_workflow(
        &self,
        workflow_id: &str,
        orchestrator: &OrchestratorArchitecture,
    ) -> Result<(), OrchestratorError> {
        // Get the workflow
        let workflow = self.get_workflow(workflow_id)?.ok_or_else(|| {
            OrchestratorError::Workflow(WorkflowError::InvalidWorkflow(format!(
                "Workflow {} not found",
                workflow_id
            )))
        })?;

        // Create a workflow executor
        let mut executor = WorkflowExecutor::new(workflow.clone());

        // Add dependencies
        for task_id in &workflow.task_ids {
            let task = orchestrator.get_task(task_id)?.ok_or_else(|| {
                OrchestratorError::Workflow(WorkflowError::InvalidWorkflow(format!(
                    "Task {} not found",
                    task_id
                )))
            })?;

            for dependency_id in &task.dependencies {
                executor
                    .add_dependency(task_id, dependency_id)
                    .map_err(OrchestratorError::Workflow)?;
            }
        }

        // Execute the workflow
        let result = executor.execute(orchestrator)?;

        // Store the result
        let mut results = self.results.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on results".to_string())
        })?;

        results.insert(workflow_id.to_string(), result);

        Ok(())
    }

    /// Get a workflow result
    pub fn get_workflow_result(
        &self,
        workflow_id: &str,
    ) -> Result<Option<WorkflowResult>, OrchestratorError> {
        let results = self.results.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on results".to_string())
        })?;

        Ok(results.get(workflow_id).cloned())
    }
}
