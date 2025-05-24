//! Integration Framework
//!
//! This module provides functionality for integrating results from different specialized modes
//! and facilitating communication between them.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::modules::orchestrator::types::{
    IntegrationError, Mode, OrchestratorError, TaskResult,
};

/// Result adapter for converting between different result formats
pub trait ResultAdapter: Send + Sync {
    /// Convert a result from a specialized mode to a common format
    fn adapt_result(&self, mode: Mode, result: &str) -> Result<TaskResult, IntegrationError>;
}

/// Default result adapter
pub struct DefaultResultAdapter;

impl ResultAdapter for DefaultResultAdapter {
    fn adapt_result(&self, mode: Mode, result: &str) -> Result<TaskResult, IntegrationError> {
        // Parse the result string as JSON
        let result_value: serde_json::Value = serde_json::from_str(result).map_err(|e| {
            IntegrationError::InvalidResultFormat(format!("Failed to parse result as JSON: {}", e))
        })?;

        // Extract the task ID
        let task_id = result_value
            .get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                IntegrationError::InvalidResultFormat("Missing task_id field".to_string())
            })?
            .to_string();

        // Extract the status
        let status_str = result_value
            .get("status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                IntegrationError::InvalidResultFormat("Missing status field".to_string())
            })?;

        let status = match status_str {
            "pending" => crate::modules::orchestrator::types::TaskStatus::Pending,
            "in_progress" => crate::modules::orchestrator::types::TaskStatus::InProgress,
            "completed" => crate::modules::orchestrator::types::TaskStatus::Completed,
            "failed" => crate::modules::orchestrator::types::TaskStatus::Failed,
            "cancelled" => crate::modules::orchestrator::types::TaskStatus::Cancelled,
            _ => {
                return Err(IntegrationError::InvalidResultFormat(format!(
                    "Invalid status: {}",
                    status_str
                )))
            }
        };

        // Extract the message
        let message = result_value
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Extract additional data
        let mut data = HashMap::new();
        if let Some(data_obj) = result_value.get("data").and_then(|v| v.as_object()) {
            for (key, value) in data_obj {
                if let Some(value_str) = value.as_str() {
                    data.insert(key.clone(), value_str.to_string());
                }
            }
        }

        // Create the task result
        let mut task_result = TaskResult::new(task_id, status, message);
        for (key, value) in data {
            task_result = task_result.with_data(key, value);
        }

        Ok(task_result)
    }
}

/// Result aggregator for combining results from different modes
pub struct ResultAggregator {
    /// Results
    results: Mutex<HashMap<String, Vec<TaskResult>>>,
}

impl ResultAggregator {
    /// Create a new result aggregator
    pub fn new() -> Self {
        Self {
            results: Mutex::new(HashMap::new()),
        }
    }

    /// Add a result
    pub fn add_result(&self, task_id: &str, result: TaskResult) -> Result<(), OrchestratorError> {
        let mut results = self.results.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on results".to_string())
        })?;

        let task_results = results.entry(task_id.to_string()).or_insert_with(Vec::new);
        task_results.push(result);

        Ok(())
    }

    /// Get results for a task
    pub fn get_results(&self, task_id: &str) -> Result<Vec<TaskResult>, OrchestratorError> {
        let results = self.results.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on results".to_string())
        })?;

        Ok(results.get(task_id).cloned().unwrap_or_default())
    }

    /// Aggregate results for a task
    pub fn aggregate_results(&self, task_id: &str) -> Result<TaskResult, OrchestratorError> {
        let results = self.get_results(task_id)?;

        if results.is_empty() {
            return Err(OrchestratorError::Integration(
                IntegrationError::ResultNotFound(task_id.to_string()),
            ));
        }

        // Determine the overall status
        let status = if results
            .iter()
            .any(|r| r.status == crate::modules::orchestrator::types::TaskStatus::Failed)
        {
            crate::modules::orchestrator::types::TaskStatus::Failed
        } else if results
            .iter()
            .any(|r| r.status == crate::modules::orchestrator::types::TaskStatus::InProgress)
        {
            crate::modules::orchestrator::types::TaskStatus::InProgress
        } else if results
            .iter()
            .all(|r| r.status == crate::modules::orchestrator::types::TaskStatus::Completed)
        {
            crate::modules::orchestrator::types::TaskStatus::Completed
        } else {
            crate::modules::orchestrator::types::TaskStatus::Pending
        };

        // Combine messages
        let message = results
            .iter()
            .map(|r| r.message.clone())
            .collect::<Vec<_>>()
            .join("\n");

        // Combine data
        let mut data = HashMap::new();
        for result in &results {
            for (key, value) in &result.data {
                data.insert(key.clone(), value.clone());
            }
        }

        // Create the aggregated result
        let mut aggregated_result = TaskResult::new(task_id, status, message);
        for (key, value) in data {
            aggregated_result = aggregated_result.with_data(key, value);
        }

        Ok(aggregated_result)
    }
}

/// Integration framework for facilitating communication between modes
pub struct IntegrationFramework {
    /// Result adapters
    adapters: Mutex<HashMap<Mode, Box<dyn ResultAdapter + Send + Sync>>>,
    /// Result aggregator
    aggregator: Arc<ResultAggregator>,
}

impl IntegrationFramework {
    /// Create a new integration framework
    pub fn new() -> Self {
        let mut adapters = HashMap::new();
        adapters.insert(
            Mode::Debug,
            Box::new(DefaultResultAdapter) as Box<dyn ResultAdapter + Send + Sync>,
        );
        adapters.insert(
            Mode::Code,
            Box::new(DefaultResultAdapter) as Box<dyn ResultAdapter + Send + Sync>,
        );
        adapters.insert(
            Mode::Test,
            Box::new(DefaultResultAdapter) as Box<dyn ResultAdapter + Send + Sync>,
        );
        adapters.insert(
            Mode::Boomerang,
            Box::new(DefaultResultAdapter) as Box<dyn ResultAdapter + Send + Sync>,
        );

        Self {
            adapters: Mutex::new(adapters),
            aggregator: Arc::new(ResultAggregator::new()),
        }
    }

    /// Get the result aggregator
    pub fn aggregator(&self) -> Arc<ResultAggregator> {
        self.aggregator.clone()
    }

    /// Register a result adapter
    pub fn register_adapter(
        &self,
        mode: Mode,
        adapter: Box<dyn ResultAdapter + Send + Sync>,
    ) -> Result<(), OrchestratorError> {
        let mut adapters = self.adapters.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on adapters".to_string())
        })?;

        adapters.insert(mode, adapter);
        Ok(())
    }

    /// Process a result from a specialized mode
    pub fn process_result(
        &self,
        mode: Mode,
        task_id: &str,
        result: &str,
    ) -> Result<TaskResult, OrchestratorError> {
        // Get the adapter for the mode
        let adapters = self.adapters.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on adapters".to_string())
        })?;

        let adapter = adapters.get(&mode).ok_or_else(|| {
            OrchestratorError::Integration(IntegrationError::Other(format!(
                "No adapter registered for mode: {}",
                mode
            )))
        })?;

        // Adapt the result
        let task_result = adapter
            .adapt_result(mode, result)
            .map_err(OrchestratorError::Integration)?;

        // Add the result to the aggregator
        self.aggregator.add_result(task_id, task_result.clone())?;

        Ok(task_result)
    }

    /// Aggregate results for a task
    pub fn aggregate_results(&self, task_id: &str) -> Result<TaskResult, OrchestratorError> {
        self.aggregator.aggregate_results(task_id)
    }
}
