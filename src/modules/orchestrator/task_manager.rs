//! Task Manager
//!
//! This module provides the main task delegation and tracking system that coordinates
//! task execution across different specialized modes.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::modules::orchestrator::delegation::{TaskDelegator, TaskTracker};
use crate::modules::orchestrator::mode_handlers::ModeHandlerFactory;
use crate::modules::orchestrator::types::{
    DelegationError, Mode, OrchestratorError, Task, TaskResult, TaskStatus,
};

/// Task manager configuration
#[derive(Debug, Clone)]
pub struct TaskManagerConfig {
    /// Maximum concurrent tasks per mode
    pub max_concurrent_tasks_per_mode: HashMap<Mode, usize>,
    /// Default mode for tasks without a specified mode
    pub default_mode: Mode,
    /// Enable automatic dependency resolution
    pub auto_resolve_dependencies: bool,
    /// Enable automatic task tracking
    pub auto_track_tasks: bool,
    /// Additional options
    pub options: HashMap<String, String>,
}

impl Default for TaskManagerConfig {
    fn default() -> Self {
        let mut max_concurrent_tasks_per_mode = HashMap::new();
        max_concurrent_tasks_per_mode.insert(Mode::Debug, 5);
        max_concurrent_tasks_per_mode.insert(Mode::Code, 10);
        max_concurrent_tasks_per_mode.insert(Mode::Test, 20);
        max_concurrent_tasks_per_mode.insert(Mode::Boomerang, 1);

        Self {
            max_concurrent_tasks_per_mode,
            default_mode: Mode::Code,
            auto_resolve_dependencies: true,
            auto_track_tasks: true,
            options: HashMap::new(),
        }
    }
}

impl TaskManagerConfig {
    /// Create a new task manager configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum concurrent tasks for a mode
    pub fn with_max_concurrent_tasks(mut self, mode: Mode, max_tasks: usize) -> Self {
        self.max_concurrent_tasks_per_mode.insert(mode, max_tasks);
        self
    }

    /// Set the default mode
    pub fn with_default_mode(mut self, mode: Mode) -> Self {
        self.default_mode = mode;
        self
    }

    /// Set whether to automatically resolve dependencies
    pub fn with_auto_resolve_dependencies(mut self, auto_resolve: bool) -> Self {
        self.auto_resolve_dependencies = auto_resolve;
        self
    }

    /// Set whether to automatically track tasks
    pub fn with_auto_track_tasks(mut self, auto_track: bool) -> Self {
        self.auto_track_tasks = auto_track;
        self
    }

    /// Add an option
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }
}

/// Task manager for delegating and tracking tasks
pub struct TaskManager {
    /// Configuration
    config: TaskManagerConfig,
    /// Task delegator
    delegator: Arc<TaskDelegator>,
    /// Task tracker
    tracker: Arc<TaskTracker>,
    /// Mode handler factory
    handler_factory: ModeHandlerFactory,
    /// Active tasks per mode
    active_tasks_per_mode: Mutex<HashMap<Mode, HashSet<String>>>,
    /// Task dependencies
    task_dependencies: Mutex<HashMap<String, HashSet<String>>>,
    /// Reverse dependencies
    reverse_dependencies: Mutex<HashMap<String, HashSet<String>>>,
}

impl TaskManager {
    /// Create a new task manager
    pub fn new(config: TaskManagerConfig) -> Self {
        let tracker = Arc::new(TaskTracker::new());
        let delegator = Arc::new(TaskDelegator::new());
        let handler_factory = ModeHandlerFactory::new(tracker.clone());

        let this = Self {
            config,
            delegator,
            tracker,
            handler_factory,
            active_tasks_per_mode: Mutex::new(HashMap::new()),
            task_dependencies: Mutex::new(HashMap::new()),
            reverse_dependencies: Mutex::new(HashMap::new()),
        };

        // Register mode handlers
        this.register_mode_handlers();

        this
    }

    /// Register mode handlers
    fn register_mode_handlers(&self) {
        // Register Debug mode handler
        let debug_handler = self.handler_factory.create_handler(Mode::Debug);
        self.delegator
            .register_mode_handler(Mode::Debug, debug_handler)
            .expect("Failed to register Debug mode handler");

        // Register Code mode handler
        let code_handler = self.handler_factory.create_handler(Mode::Code);
        self.delegator
            .register_mode_handler(Mode::Code, code_handler)
            .expect("Failed to register Code mode handler");

        // Register Test mode handler
        let test_handler = self.handler_factory.create_handler(Mode::Test);
        self.delegator
            .register_mode_handler(Mode::Test, test_handler)
            .expect("Failed to register Test mode handler");

        // Register Boomerang mode handler
        let boomerang_handler = self.handler_factory.create_handler(Mode::Boomerang);
        self.delegator
            .register_mode_handler(Mode::Boomerang, boomerang_handler)
            .expect("Failed to register Boomerang mode handler");
    }

    /// Add a task
    pub fn add_task(&self, task: Task) -> Result<(), OrchestratorError> {
        // Store task dependencies
        if !task.dependencies.is_empty() {
            let mut task_dependencies = self.task_dependencies.lock().map_err(|_| {
                OrchestratorError::Other("Failed to acquire lock on task dependencies".to_string())
            })?;

            let mut reverse_dependencies = self.reverse_dependencies.lock().map_err(|_| {
                OrchestratorError::Other(
                    "Failed to acquire lock on reverse dependencies".to_string(),
                )
            })?;

            let deps = task_dependencies
                .entry(task.id.clone())
                .or_insert_with(HashSet::new);
            for dep_id in &task.dependencies {
                deps.insert(dep_id.clone());

                let rev_deps = reverse_dependencies
                    .entry(dep_id.clone())
                    .or_insert_with(HashSet::new);
                rev_deps.insert(task.id.clone());
            }
        }

        // Check if the task can be executed immediately
        if self.config.auto_resolve_dependencies {
            let can_execute = self.can_execute_task(&task)?;

            if can_execute {
                // Delegate the task to the appropriate mode
                self.delegate_task(&task.id, task.mode)?;
            }
        }

        Ok(())
    }

    /// Delegate a task to a specialized mode
    pub fn delegate_task(&self, task_id: &str, mode: Mode) -> Result<(), OrchestratorError> {
        // Check if the mode has capacity
        let mut active_tasks_per_mode = self.active_tasks_per_mode.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on active tasks per mode".to_string())
        })?;

        let active_tasks = active_tasks_per_mode
            .entry(mode)
            .or_insert_with(HashSet::new);
        let max_tasks = self
            .config
            .max_concurrent_tasks_per_mode
            .get(&mode)
            .copied()
            .unwrap_or(10);

        if active_tasks.len() >= max_tasks {
            return Err(OrchestratorError::Delegation(DelegationError::Other(
                format!(
                    "Mode {} has reached maximum concurrent tasks ({})",
                    mode, max_tasks
                ),
            )));
        }

        // Add the task to the active tasks
        active_tasks.insert(task_id.to_string());

        // Delegate the task
        self.delegator
            .delegate_task(Task::new(task_id, "Task", "Task", mode), mode)?;

        Ok(())
    }

    /// Process task result
    pub fn process_task_result(
        &self,
        task_id: &str,
        result: TaskResult,
    ) -> Result<(), OrchestratorError> {
        // Process the result
        self.delegator
            .process_task_result(task_id, result.clone())?;

        // Remove the task from active tasks
        let mut active_tasks_per_mode = self.active_tasks_per_mode.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on active tasks per mode".to_string())
        })?;

        for active_tasks in active_tasks_per_mode.values_mut() {
            active_tasks.remove(task_id);
        }

        // If the task is completed, check if any dependent tasks can be executed
        if result.status == TaskStatus::Completed && self.config.auto_resolve_dependencies {
            let reverse_dependencies = self.reverse_dependencies.lock().map_err(|_| {
                OrchestratorError::Other(
                    "Failed to acquire lock on reverse dependencies".to_string(),
                )
            })?;

            if let Some(rev_deps) = reverse_dependencies.get(task_id) {
                for dep_task_id in rev_deps {
                    // Check if the dependent task can be executed
                    let can_execute = self.can_execute_task_by_id(dep_task_id)?;

                    if can_execute {
                        // Get the task mode
                        let task_mode = Mode::Code; // Default to Code mode if not found

                        // Delegate the task
                        self.delegate_task(dep_task_id, task_mode)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if a task can be executed
    fn can_execute_task(&self, task: &Task) -> Result<bool, OrchestratorError> {
        // If the task has no dependencies, it can be executed
        if task.dependencies.is_empty() {
            return Ok(true);
        }

        // Check if all dependencies are satisfied
        self.delegator
            .are_dependencies_satisfied(&task.dependencies)
    }

    /// Check if a task can be executed by ID
    fn can_execute_task_by_id(&self, task_id: &str) -> Result<bool, OrchestratorError> {
        let task_dependencies = self.task_dependencies.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on task dependencies".to_string())
        })?;

        if let Some(deps) = task_dependencies.get(task_id) {
            // Check if all dependencies are satisfied
            self.delegator
                .are_dependencies_satisfied(&deps.iter().cloned().collect::<Vec<_>>())
        } else {
            // If the task has no dependencies, it can be executed
            Ok(true)
        }
    }

    /// Get the task tracker
    pub fn tracker(&self) -> Arc<TaskTracker> {
        self.tracker.clone()
    }

    /// Get the task delegator
    pub fn delegator(&self) -> Arc<TaskDelegator> {
        self.delegator.clone()
    }
}
