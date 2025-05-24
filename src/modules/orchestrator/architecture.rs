//! Orchestrator Architecture
//!
//! This module defines the core architecture of the Boomerang orchestrator.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::modules::orchestrator::delegation::TaskDelegator;
use crate::modules::orchestrator::integration::IntegrationFramework;
use crate::modules::orchestrator::reporting::ReportGenerator;
use crate::modules::orchestrator::types::{Mode, OrchestratorError, Task, TaskStatus};
use crate::modules::orchestrator::workflow::WorkflowManager;

/// Configuration for the orchestrator
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Enable debug mode
    pub debug_mode: bool,
    /// Enable verbose logging
    pub verbose_logging: bool,
    /// Maximum concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Timeout for task execution (in seconds)
    pub task_timeout_seconds: u64,
    /// Retry count for failed tasks
    pub retry_count: usize,
    /// Additional configuration options
    pub options: HashMap<String, String>,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            debug_mode: false,
            verbose_logging: false,
            max_concurrent_tasks: 10,
            task_timeout_seconds: 300,
            retry_count: 3,
            options: HashMap::new(),
        }
    }
}

impl OrchestratorConfig {
    /// Create a new orchestrator configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set debug mode
    pub fn with_debug_mode(mut self, debug_mode: bool) -> Self {
        self.debug_mode = debug_mode;
        self
    }

    /// Set verbose logging
    pub fn with_verbose_logging(mut self, verbose_logging: bool) -> Self {
        self.verbose_logging = verbose_logging;
        self
    }

    /// Set maximum concurrent tasks
    pub fn with_max_concurrent_tasks(mut self, max_concurrent_tasks: usize) -> Self {
        self.max_concurrent_tasks = max_concurrent_tasks;
        self
    }

    /// Set task timeout
    pub fn with_task_timeout_seconds(mut self, task_timeout_seconds: u64) -> Self {
        self.task_timeout_seconds = task_timeout_seconds;
        self
    }

    /// Set retry count
    pub fn with_retry_count(mut self, retry_count: usize) -> Self {
        self.retry_count = retry_count;
        self
    }

    /// Add an option
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }
}

/// Core architecture of the Boomerang orchestrator
pub struct OrchestratorArchitecture {
    /// Configuration
    config: OrchestratorConfig,
    /// Task delegator
    delegator: Arc<TaskDelegator>,
    /// Integration framework
    integration: Arc<IntegrationFramework>,
    /// Workflow manager
    workflow: Arc<WorkflowManager>,
    /// Report generator
    reporting: Arc<ReportGenerator>,
    /// Tasks
    tasks: Arc<Mutex<HashMap<String, Task>>>,
}

impl OrchestratorArchitecture {
    /// Create a new orchestrator with default configuration
    pub fn new() -> Self {
        Self::with_config(OrchestratorConfig::default())
    }

    /// Create a new orchestrator with custom configuration
    pub fn with_config(config: OrchestratorConfig) -> Self {
        let delegator = Arc::new(TaskDelegator::new());
        let integration = Arc::new(IntegrationFramework::new());
        let workflow = Arc::new(WorkflowManager::new());
        let reporting = Arc::new(ReportGenerator::new());
        let tasks = Arc::new(Mutex::new(HashMap::new()));

        Self {
            config,
            delegator,
            integration,
            workflow,
            reporting,
            tasks,
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &OrchestratorConfig {
        &self.config
    }

    /// Get the task delegator
    pub fn delegator(&self) -> Arc<TaskDelegator> {
        self.delegator.clone()
    }

    /// Get the integration framework
    pub fn integration(&self) -> Arc<IntegrationFramework> {
        self.integration.clone()
    }

    /// Get the workflow manager
    pub fn workflow(&self) -> Arc<WorkflowManager> {
        self.workflow.clone()
    }

    /// Get the report generator
    pub fn reporting(&self) -> Arc<ReportGenerator> {
        self.reporting.clone()
    }

    /// Add a task
    pub fn add_task(&self, task: Task) -> Result<(), OrchestratorError> {
        let mut tasks = self
            .tasks
            .lock()
            .map_err(|_| OrchestratorError::Other("Failed to acquire lock on tasks".to_string()))?;
        tasks.insert(task.id.clone(), task);
        Ok(())
    }

    /// Get a task
    pub fn get_task(&self, task_id: &str) -> Result<Option<Task>, OrchestratorError> {
        let tasks = self
            .tasks
            .lock()
            .map_err(|_| OrchestratorError::Other("Failed to acquire lock on tasks".to_string()))?;
        Ok(tasks.get(task_id).cloned())
    }

    /// Get all tasks
    pub fn get_all_tasks(&self) -> Result<Vec<Task>, OrchestratorError> {
        let tasks = self
            .tasks
            .lock()
            .map_err(|_| OrchestratorError::Other("Failed to acquire lock on tasks".to_string()))?;
        Ok(tasks.values().cloned().collect())
    }

    /// Update task status
    pub fn update_task_status(
        &self,
        task_id: &str,
        status: TaskStatus,
    ) -> Result<(), OrchestratorError> {
        let mut tasks = self
            .tasks
            .lock()
            .map_err(|_| OrchestratorError::Other("Failed to acquire lock on tasks".to_string()))?;

        if let Some(task) = tasks.get_mut(task_id) {
            task.set_status(status);
            Ok(())
        } else {
            Err(OrchestratorError::Delegation(
                crate::modules::orchestrator::types::DelegationError::TaskNotFound(
                    task_id.to_string(),
                ),
            ))
        }
    }

    /// Delegate a task to a specialized mode
    pub fn delegate_task(&self, task_id: &str, mode: Mode) -> Result<(), OrchestratorError> {
        let task = self.get_task(task_id)?.ok_or_else(|| {
            OrchestratorError::Delegation(
                crate::modules::orchestrator::types::DelegationError::TaskNotFound(
                    task_id.to_string(),
                ),
            )
        })?;

        self.delegator.delegate_task(task, mode)
    }

    /// Process task result
    pub fn process_task_result(
        &self,
        task_id: &str,
        result: crate::modules::orchestrator::types::TaskResult,
    ) -> Result<(), OrchestratorError> {
        let mut tasks = self
            .tasks
            .lock()
            .map_err(|_| OrchestratorError::Other("Failed to acquire lock on tasks".to_string()))?;

        if let Some(task) = tasks.get_mut(task_id) {
            task.set_result(result);
            Ok(())
        } else {
            Err(OrchestratorError::Delegation(
                crate::modules::orchestrator::types::DelegationError::TaskNotFound(
                    task_id.to_string(),
                ),
            ))
        }
    }

    /// Execute workflow
    pub fn execute_workflow(&self, workflow_id: &str) -> Result<(), OrchestratorError> {
        self.workflow.execute_workflow(workflow_id, self)
    }

    /// Generate report
    pub fn generate_report(&self, report_id: &str) -> Result<String, OrchestratorError> {
        self.reporting.generate_report(report_id, self)
    }
}

// Implement OrchestratorReporting trait for OrchestratorArchitecture
impl crate::modules::orchestrator::continuous_improvement::OrchestratorReporting
    for OrchestratorArchitecture
{
    fn get_all_tasks(&self) -> Result<Vec<Task>, OrchestratorError> {
        self.get_all_tasks()
    }
}
