//! Task Delegation and Tracking
//!
//! This module provides functionality for delegating tasks to specialized modes
//! and tracking their execution.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::modules::orchestrator::types::{
    DelegationError, Mode, OrchestratorError, Task, TaskResult, TaskStatus,
};

/// Task delegation event
#[derive(Debug, Clone)]
pub struct DelegationEvent {
    /// Task ID
    pub task_id: String,
    /// Mode
    pub mode: Mode,
    /// Timestamp
    pub timestamp: Instant,
    /// Status
    pub status: TaskStatus,
    /// Message
    pub message: String,
}

impl DelegationEvent {
    /// Create a new delegation event
    pub fn new(
        task_id: impl Into<String>,
        mode: Mode,
        status: TaskStatus,
        message: impl Into<String>,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            mode,
            timestamp: Instant::now(),
            status,
            message: message.into(),
        }
    }
}

/// Task tracker for monitoring task execution
pub struct TaskTracker {
    /// Task events
    events: Mutex<Vec<DelegationEvent>>,
    /// Task status
    status: Mutex<HashMap<String, TaskStatus>>,
    /// Task start times
    start_times: Mutex<HashMap<String, Instant>>,
}

impl TaskTracker {
    /// Create a new task tracker
    pub fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
            status: Mutex::new(HashMap::new()),
            start_times: Mutex::new(HashMap::new()),
        }
    }

    /// Record a delegation event
    pub fn record_event(&self, event: DelegationEvent) -> Result<(), OrchestratorError> {
        let mut events = self.events.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on events".to_string())
        })?;

        let mut status = self.status.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on status".to_string())
        })?;

        events.push(event.clone());
        status.insert(event.task_id.clone(), event.status);

        if event.status == TaskStatus::InProgress {
            let mut start_times = self.start_times.lock().map_err(|_| {
                OrchestratorError::Other("Failed to acquire lock on start times".to_string())
            })?;

            start_times.insert(event.task_id.clone(), event.timestamp);
        }

        Ok(())
    }

    /// Get the status of a task
    pub fn get_task_status(&self, task_id: &str) -> Result<Option<TaskStatus>, OrchestratorError> {
        let status = self.status.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on status".to_string())
        })?;

        Ok(status.get(task_id).copied())
    }

    /// Get the events for a task
    pub fn get_task_events(
        &self,
        task_id: &str,
    ) -> Result<Vec<DelegationEvent>, OrchestratorError> {
        let events = self.events.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on events".to_string())
        })?;

        Ok(events
            .iter()
            .filter(|event| event.task_id == task_id)
            .cloned()
            .collect())
    }

    /// Get the duration of a task
    pub fn get_task_duration(&self, task_id: &str) -> Result<Option<Duration>, OrchestratorError> {
        let start_times = self.start_times.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on start times".to_string())
        })?;

        let status = self.status.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on status".to_string())
        })?;

        if let (Some(start_time), Some(task_status)) =
            (start_times.get(task_id), status.get(task_id))
        {
            if *task_status == TaskStatus::Completed
                || *task_status == TaskStatus::Failed
                || *task_status == TaskStatus::Cancelled
            {
                // For completed tasks, calculate duration from start to now
                Ok(Some(start_time.elapsed()))
            } else if *task_status == TaskStatus::InProgress {
                // For in-progress tasks, calculate current duration
                Ok(Some(start_time.elapsed()))
            } else {
                // For pending tasks, return None
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Get all events
    pub fn get_all_events(&self) -> Result<Vec<DelegationEvent>, OrchestratorError> {
        let events = self.events.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on events".to_string())
        })?;

        Ok(events.clone())
    }
}

/// Task delegator for delegating tasks to specialized modes
pub struct TaskDelegator {
    /// Task tracker
    tracker: Arc<TaskTracker>,
    /// Mode handlers
    mode_handlers: Mutex<HashMap<Mode, Box<dyn ModeHandler + Send + Sync>>>,
}

/// Mode handler trait for handling tasks in a specific mode
pub trait ModeHandler: Send + Sync {
    /// Handle a task
    fn handle_task(&self, task: Task) -> Result<(), DelegationError>;
}

impl TaskDelegator {
    /// Create a new task delegator
    pub fn new() -> Self {
        Self {
            tracker: Arc::new(TaskTracker::new()),
            mode_handlers: Mutex::new(HashMap::new()),
        }
    }

    /// Get the task tracker
    pub fn tracker(&self) -> Arc<TaskTracker> {
        self.tracker.clone()
    }

    /// Register a mode handler
    pub fn register_mode_handler(
        &self,
        mode: Mode,
        handler: Box<dyn ModeHandler + Send + Sync>,
    ) -> Result<(), OrchestratorError> {
        let mut mode_handlers = self.mode_handlers.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on mode handlers".to_string())
        })?;

        mode_handlers.insert(mode, handler);
        Ok(())
    }

    /// Delegate a task to a specialized mode
    pub fn delegate_task(&self, task: Task, mode: Mode) -> Result<(), OrchestratorError> {
        // Record delegation event
        self.tracker.record_event(DelegationEvent::new(
            task.id.clone(),
            mode,
            TaskStatus::InProgress,
            format!("Delegating task to {} mode", mode),
        ))?;

        // Get the mode handler
        let mode_handlers = self.mode_handlers.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on mode handlers".to_string())
        })?;

        let handler = mode_handlers.get(&mode).ok_or_else(|| {
            OrchestratorError::Delegation(DelegationError::ModeNotAvailable(format!("{}", mode)))
        })?;

        // Handle the task
        handler
            .handle_task(task)
            .map_err(OrchestratorError::Delegation)?;

        Ok(())
    }

    /// Process task result
    pub fn process_task_result(
        &self,
        task_id: &str,
        result: TaskResult,
    ) -> Result<(), OrchestratorError> {
        // Record result event
        self.tracker.record_event(DelegationEvent::new(
            task_id,
            Mode::Boomerang, // Result processing is done by Boomerang
            result.status,
            result.message.clone(),
        ))?;

        Ok(())
    }

    /// Check if all dependencies are satisfied
    pub fn are_dependencies_satisfied(
        &self,
        dependencies: &[String],
    ) -> Result<bool, OrchestratorError> {
        for dependency_id in dependencies {
            let status = self.tracker.get_task_status(dependency_id)?;

            if status != Some(TaskStatus::Completed) {
                return Ok(false);
            }
        }

        Ok(true)
    }
}
