//! Chain Engine IPC interface
//!
//! This module provides trait-based abstractions for the Chain Engine service,
//! ensuring a clear separation between interface and transport logic.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use crate::modules::ipc::{IpcError, IpcResult};

/// Represents a step in a chain
#[derive(Debug, Clone)]
pub struct ChainStep {
    pub id: String,
    pub description: String,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub input_template: String,
    pub output_format: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub parameters: HashMap<String, String>,
}

/// Represents a chain configuration
#[derive(Debug, Clone)]
pub struct Chain {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<ChainStep>,
    pub version: VersionInfo,
    pub metadata: HashMap<String, String>,
}

/// Represents version information
#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

/// Represents the result of executing a step
#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_id: String,
    pub status: Status,
    pub input: String,
    pub output: String,
    pub error: Option<ErrorDetails>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub tokens: u32,
    pub model: String,
    pub metadata: HashMap<String, String>,
}

/// Represents the status of an operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Success,
    Error,
    InProgress,
    Timeout,
    Cancelled,
}

/// Represents error details
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    pub details: HashMap<String, String>,
    pub stack_trace: Option<String>,
}

/// Represents the type of chain execution event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    StepStarted,
    StepCompleted,
    StepFailed,
    ChainCompleted,
    ChainFailed,
    TokenGenerated,
}

/// Represents a chain execution event
#[derive(Debug, Clone)]
pub enum ChainExecutionEventData {
    StepStarted {
        step_id: String,
        step_index: u32,
        input: String,
    },
    StepCompleted {
        step_id: String,
        step_index: u32,
        output: String,
        tokens: u32,
    },
    StepFailed {
        step_id: String,
        step_index: u32,
        error: ErrorDetails,
    },
    ChainCompleted {
        output: String,
        total_tokens: u32,
        execution_time_ms: u64,
    },
    ChainFailed {
        error: ErrorDetails,
        execution_time_ms: u64,
    },
    TokenGenerated {
        step_id: String,
        token: String,
    },
}

/// Represents a chain execution event
#[derive(Debug, Clone)]
pub struct ChainExecutionEvent {
    pub event_type: EventType,
    pub execution_id: String,
    pub timestamp: DateTime<Utc>,
    pub data: ChainExecutionEventData,
}

/// Client interface for the Chain Engine service
#[async_trait]
pub trait ChainEngineClient: Send + Sync {
    /// Execute a chain with the given input
    async fn execute_chain(
        &self,
        chain_id: Option<String>,
        chain: Option<Chain>,
        input: String,
        variables: HashMap<String, String>,
        stream: bool,
        timeout_seconds: Option<u32>,
    ) -> IpcResult<ChainExecutionResponse>;

    /// Get the status of a chain execution
    async fn get_chain_status(&self, execution_id: &str) -> IpcResult<ChainStatusResponse>;

    /// Cancel a running chain execution
    async fn cancel_chain_execution(&self, execution_id: &str) -> IpcResult<CancelChainResponse>;

    /// Stream the results of a chain execution
    async fn stream_chain_execution(
        &self,
        chain_id: Option<String>,
        chain: Option<Chain>,
        input: String,
        variables: HashMap<String, String>,
        timeout_seconds: Option<u32>,
    ) -> IpcResult<Pin<Box<dyn Stream<Item = Result<ChainExecutionEvent, tonic::Status>> + Send>>>;
}

/// Server interface for the Chain Engine service
#[async_trait]
pub trait ChainEngineService: Send + Sync {
    /// Execute a chain with the given input
    async fn execute_chain(
        &self,
        chain_id: Option<String>,
        chain: Option<Chain>,
        input: String,
        variables: HashMap<String, String>,
        stream: bool,
        timeout_seconds: Option<u32>,
    ) -> IpcResult<ChainExecutionResponse>;

    /// Get the status of a chain execution
    async fn get_chain_status(&self, execution_id: &str) -> IpcResult<ChainStatusResponse>;

    /// Cancel a running chain execution
    async fn cancel_chain_execution(&self, execution_id: &str) -> IpcResult<CancelChainResponse>;

    /// Stream the results of a chain execution
    async fn stream_chain_execution(
        &self,
        chain_id: Option<String>,
        chain: Option<Chain>,
        input: String,
        variables: HashMap<String, String>,
        timeout_seconds: Option<u32>,
    ) -> IpcResult<Pin<Box<dyn Stream<Item = Result<ChainExecutionEvent, tonic::Status>> + Send>>>;
}

/// Represents a chain execution response
#[derive(Debug, Clone)]
pub struct ChainExecutionResponse {
    pub execution_id: String,
    pub status: Status,
    pub output: String,
    pub error: Option<ErrorDetails>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub step_results: Vec<StepResult>,
    pub total_tokens: u32,
    pub metadata: HashMap<String, String>,
}

/// Represents a chain status response
#[derive(Debug, Clone)]
pub struct ChainStatusResponse {
    pub execution_id: String,
    pub status: Status,
    pub current_step_id: Option<String>,
    pub completed_steps: u32,
    pub total_steps: u32,
    pub error: Option<ErrorDetails>,
    pub start_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

/// Represents a cancel chain response
#[derive(Debug, Clone)]
pub struct CancelChainResponse {
    pub execution_id: String,
    pub success: bool,
    pub error: Option<ErrorDetails>,
}

/// gRPC implementation of the Chain Engine client
pub struct GrpcChainEngineClient {
    // This would contain the generated gRPC client
    // client: chain_engine_client::ChainEngineClient<tonic::transport::Channel>,
}

impl GrpcChainEngineClient {
    /// Create a new gRPC Chain Engine client
    pub async fn new(addr: &str) -> Result<Self, tonic::transport::Error> {
        // This would create the gRPC client
        // let client = chain_engine_client::ChainEngineClient::connect(addr).await?;
        Ok(Self {
            // client,
        })
    }
}

#[async_trait]
impl ChainEngineClient for GrpcChainEngineClient {
    async fn execute_chain(
        &self,
        _chain_id: Option<String>,
        _chain: Option<Chain>,
        _input: String,
        _variables: HashMap<String, String>,
        _stream: bool,
        _timeout_seconds: Option<u32>,
    ) -> IpcResult<ChainExecutionResponse> {
        // Stub implementation for now
        Ok(ChainExecutionResponse {
            execution_id: "stub-execution-id".to_string(),
            status: Status::Success,
            output: "This is a stub chain execution response.".to_string(),
            error: None,
            start_time: chrono::Utc::now(),
            end_time: chrono::Utc::now(),
            step_results: vec![StepResult {
                step_id: "stub-step-1".to_string(),
                status: Status::Success,
                input: "Input for step 1".to_string(),
                output: "Output from step 1".to_string(),
                error: None,
                start_time: chrono::Utc::now(),
                end_time: chrono::Utc::now(),
                tokens: 10,
                model: "stub-model".to_string(),
                metadata: HashMap::new(),
            }],
            total_tokens: 10,
            metadata: HashMap::new(),
        })
    }

    async fn get_chain_status(&self, _execution_id: &str) -> IpcResult<ChainStatusResponse> {
        // Stub implementation for now
        Ok(ChainStatusResponse {
            execution_id: "stub-execution-id".to_string(),
            status: Status::Success,
            current_step_id: Some("stub-step-1".to_string()),
            completed_steps: 1,
            total_steps: 3,
            error: None,
            start_time: chrono::Utc::now(),
            update_time: chrono::Utc::now(),
        })
    }

    async fn cancel_chain_execution(&self, _execution_id: &str) -> IpcResult<CancelChainResponse> {
        // Stub implementation for now
        Ok(CancelChainResponse {
            execution_id: "stub-execution-id".to_string(),
            success: true,
            error: None,
        })
    }

    async fn stream_chain_execution(
        &self,
        _chain_id: Option<String>,
        _chain: Option<Chain>,
        _input: String,
        _variables: HashMap<String, String>,
        _timeout_seconds: Option<u32>,
    ) -> IpcResult<Pin<Box<dyn Stream<Item = Result<ChainExecutionEvent, tonic::Status>> + Send>>>
    {
        // Stub implementation for now
        let events = vec![
            Ok(ChainExecutionEvent {
                event_type: EventType::StepStarted,
                execution_id: "stub-execution-id".to_string(),
                timestamp: chrono::Utc::now(),
                data: ChainExecutionEventData::StepStarted {
                    step_id: "stub-step-1".to_string(),
                    step_index: 0,
                    input: "Input for step 1".to_string(),
                },
            }),
            Ok(ChainExecutionEvent {
                event_type: EventType::StepCompleted,
                execution_id: "stub-execution-id".to_string(),
                timestamp: chrono::Utc::now(),
                data: ChainExecutionEventData::StepCompleted {
                    step_id: "stub-step-1".to_string(),
                    step_index: 0,
                    output: "Output from step 1".to_string(),
                    tokens: 10,
                },
            }),
            Ok(ChainExecutionEvent {
                event_type: EventType::ChainCompleted,
                execution_id: "stub-execution-id".to_string(),
                timestamp: chrono::Utc::now(),
                data: ChainExecutionEventData::ChainCompleted {
                    output: "Final chain output".to_string(),
                    total_tokens: 10,
                    execution_time_ms: 100,
                },
            }),
        ];

        let stream = futures::stream::iter(events);
        Ok(Box::pin(stream))
    }
}

/// Mock implementation of the Chain Engine client for testing
#[cfg(test)]
pub struct MockChainEngineClient {
    executions: HashMap<String, ChainExecutionResponse>,
}

#[cfg(test)]
impl MockChainEngineClient {
    /// Create a new mock Chain Engine client
    pub fn new() -> Self {
        Self {
            executions: HashMap::new(),
        }
    }

    /// Add an execution to the mock client
    pub fn add_execution(&mut self, execution: ChainExecutionResponse) {
        self.executions
            .insert(execution.execution_id.clone(), execution);
    }
}

#[cfg(test)]
#[async_trait]
impl ChainEngineClient for MockChainEngineClient {
    async fn execute_chain(
        &self,
        _chain_id: Option<String>,
        _chain: Option<Chain>,
        input: String,
        _variables: HashMap<String, String>,
        _stream: bool,
        _timeout_seconds: Option<u32>,
    ) -> IpcResult<ChainExecutionResponse> {
        // Create a mock execution response
        let execution_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let response = ChainExecutionResponse {
            execution_id,
            status: Status::Success,
            output: format!("Processed: {}", input),
            error: None,
            start_time: now,
            end_time: now,
            step_results: Vec::new(),
            total_tokens: 100,
            metadata: HashMap::new(),
        };

        Ok(response)
    }

    async fn get_chain_status(&self, execution_id: &str) -> IpcResult<ChainStatusResponse> {
        if let Some(execution) = self.executions.get(execution_id) {
            Ok(ChainStatusResponse {
                execution_id: execution_id.to_string(),
                status: execution.status,
                current_step_id: None,
                completed_steps: execution.step_results.len() as u32,
                total_steps: execution.step_results.len() as u32,
                error: execution.error.clone(),
                start_time: execution.start_time,
                update_time: Utc::now(),
            })
        } else {
            Err(IpcError::NotFound(format!(
                "Execution not found: {}",
                execution_id
            )))
        }
    }

    async fn cancel_chain_execution(&self, execution_id: &str) -> IpcResult<CancelChainResponse> {
        if self.executions.contains_key(execution_id) {
            Ok(CancelChainResponse {
                execution_id: execution_id.to_string(),
                success: true,
                error: None,
            })
        } else {
            Err(IpcError::NotFound(format!(
                "Execution not found: {}",
                execution_id
            )))
        }
    }

    async fn stream_chain_execution(
        &self,
        _chain_id: Option<String>,
        _chain: Option<Chain>,
        _input: String,
        _variables: HashMap<String, String>,
        _timeout_seconds: Option<u32>,
    ) -> IpcResult<Pin<Box<dyn Stream<Item = Result<ChainExecutionEvent, tonic::Status>> + Send>>>
    {
        // This is a simplified mock implementation
        let stream = futures::stream::empty();
        Ok(Box::pin(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_chain_engine_client() {
        let mut client = MockChainEngineClient::new();

        // Test execute_chain
        let response = client
            .execute_chain(
                Some("test-chain".to_string()),
                None,
                "Test input".to_string(),
                HashMap::new(),
                false,
                None,
            )
            .await
            .unwrap();

        assert_eq!(response.status, Status::Success);
        assert_eq!(response.output, "Processed: Test input");

        // Add the execution to the mock client
        client.add_execution(response.clone());

        // Test get_chain_status
        let status = client
            .get_chain_status(&response.execution_id)
            .await
            .unwrap();
        assert_eq!(status.status, Status::Success);

        // Test cancel_chain_execution
        let cancel = client
            .cancel_chain_execution(&response.execution_id)
            .await
            .unwrap();
        assert!(cancel.success);

        // Test get_chain_status with non-existent ID
        let result = client.get_chain_status("non-existent").await;
        assert!(result.is_err());
    }
}
