//! Test Executor
//!
//! This module is responsible for executing test flows to verify system integration.
//! It provides functionality to run predefined test flows and validate the results.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::Client;
use serde_json::json;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use super::report::AuditReport;
use super::types::{AuditError, TestConfig, TestFlow, TestResult};

/// Test Executor
#[derive(Debug)]
pub struct TestExecutor {
    /// Test configuration
    config: TestConfig,
    /// HTTP client for API requests
    client: Client,
    /// Shared audit report
    report: Arc<RwLock<AuditReport>>,
}

impl TestExecutor {
    /// Create a new test executor
    pub fn new(config: TestConfig, report: Arc<RwLock<AuditReport>>) -> Self {
        Self {
            config,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            report,
        }
    }

    /// Execute all configured test flows
    pub async fn execute_test_flows(&self) -> Result<(), AuditError> {
        info!("Starting test flow execution");

        let mut results = Vec::new();

        for test_flow in &self.config.test_flows {
            info!("Executing test flow: {}", test_flow);

            let result = match test_flow {
                TestFlow::BasicChainExecution => self.execute_basic_chain_test().await,
                TestFlow::RagIntegration => self.execute_rag_integration_test().await,
                TestFlow::PersonaLayerIntegration => self.execute_persona_layer_test().await,
                TestFlow::EndToEndFlow => self.execute_end_to_end_test().await,
            };

            match result {
                Ok(test_result) => {
                    if test_result.success {
                        info!("Test flow {} completed successfully", test_flow);

                        // Update report with test success
                        let mut report = self.report.write().await;
                        report.add_test_result(test_result.clone());
                        report
                            .add_success(format!("Test flow {} completed successfully", test_flow));
                    } else {
                        let error_msg = format!(
                            "Test flow {} failed: {}",
                            test_flow,
                            test_result.error.as_deref().unwrap_or("Unknown error")
                        );
                        error!("{}", error_msg);

                        // Update report with test failure
                        let mut report = self.report.write().await;
                        report.add_test_result(test_result.clone());
                        report.add_error(format!("Test execution error: {}", error_msg));

                        if self.config.fail_fast {
                            return Err(AuditError::TestExecutionError(error_msg));
                        }
                    }

                    results.push(test_result);
                }
                Err(e) => {
                    let error_msg = format!("Failed to execute test flow {}: {}", test_flow, e);
                    error!("{}", error_msg);

                    // Update report with test error
                    let mut report = self.report.write().await;
                    report.add_error(format!("Test execution error: {}", error_msg));

                    if self.config.fail_fast {
                        return Err(AuditError::TestExecutionError(error_msg));
                    }
                }
            }
        }

        // Check if all tests were successful
        let all_successful = results.iter().all(|r| r.success);

        if all_successful {
            info!("All test flows completed successfully");

            // Update report with test success
            let mut report = self.report.write().await;
            report.add_success("All test flows completed successfully");

            Ok(())
        } else {
            let error_msg = "Some test flows failed";
            error!("{}", error_msg);

            // Update report with test failure
            let mut report = self.report.write().await;
            report.add_error(format!("Test execution error: {}", error_msg));

            Err(AuditError::TestExecutionError(error_msg.to_string()))
        }
    }

    /// Execute a basic chain test
    async fn execute_basic_chain_test(&self) -> Result<TestResult, AuditError> {
        info!("Executing basic chain test");

        let start_time = Instant::now();

        // Define a simple chain for testing
        let chain_definition = json!({
            "name": "audit_test_chain",
            "description": "A simple chain for testing",
            "steps": [
                {
                    "id": "step1",
                    "type": "function",
                    "function": "echo",
                    "input": {
                        "message": "Hello, world!"
                    }
                }
            ]
        });

        // Execute the chain
        let chain_result = self.execute_chain(chain_definition).await;

        let duration = start_time.elapsed();

        match chain_result {
            Ok(result) => {
                // Verify the result
                if let Some(output) = result.get("output") {
                    if let Some(message) = output.get("message") {
                        if message.as_str() == Some("Hello, world!") {
                            info!("Basic chain test passed");

                            return Ok(TestResult {
                                test_flow: TestFlow::BasicChainExecution,
                                success: true,
                                error: None,
                                duration_ms: duration.as_millis() as u64,
                                timestamp: chrono::Utc::now(),
                                details: {
                                    let mut details = HashMap::new();
                                    details.insert(
                                        "result".to_string(),
                                        serde_json::to_value(result).unwrap(),
                                    );
                                    details
                                },
                            });
                        }
                    }
                }

                let error_msg = "Chain execution did not return the expected result";
                warn!("{}", error_msg);

                Ok(TestResult {
                    test_flow: TestFlow::BasicChainExecution,
                    success: false,
                    error: Some(error_msg.to_string()),
                    duration_ms: duration.as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                    details: {
                        let mut details = HashMap::new();
                        details.insert("result".to_string(), serde_json::to_value(result).unwrap());
                        details
                    },
                })
            }
            Err(e) => {
                let error_msg = format!("Failed to execute chain: {}", e);
                error!("{}", error_msg);

                Ok(TestResult {
                    test_flow: TestFlow::BasicChainExecution,
                    success: false,
                    error: Some(error_msg),
                    duration_ms: duration.as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                    details: HashMap::new(),
                })
            }
        }
    }

    /// Execute a RAG integration test
    async fn execute_rag_integration_test(&self) -> Result<TestResult, AuditError> {
        info!("Executing RAG integration test");

        let start_time = Instant::now();

        // Define a RAG chain for testing
        let chain_definition = json!({
            "name": "audit_rag_test_chain",
            "description": "A RAG chain for testing",
            "steps": [
                {
                    "id": "rag_step",
                    "type": "rag",
                    "input": {
                        "query": "What is IntelliRouter?",
                        "collection": "audit_test_collection"
                    }
                },
                {
                    "id": "llm_step",
                    "type": "llm",
                    "input": {
                        "prompt": "Answer the question using the retrieved context: {{rag_step.output.context}}",
                        "question": "What is IntelliRouter?"
                    }
                }
            ]
        });

        // First, create a test collection and add a document
        let create_collection_result = self.create_test_collection().await;

        if let Err(e) = create_collection_result {
            let error_msg = format!("Failed to create test collection: {}", e);
            error!("{}", error_msg);

            return Ok(TestResult {
                test_flow: TestFlow::RagIntegration,
                success: false,
                error: Some(error_msg),
                duration_ms: start_time.elapsed().as_millis() as u64,
                timestamp: chrono::Utc::now(),
                details: HashMap::new(),
            });
        }

        // Execute the chain
        let chain_result = self.execute_chain(chain_definition).await;

        let duration = start_time.elapsed();

        // Clean up the test collection
        let _ = self.delete_test_collection().await;

        match chain_result {
            Ok(result) => {
                // Verify the result
                if let Some(output) = result.get("output") {
                    if let Some(answer) = output.get("answer") {
                        if answer.as_str().is_some() {
                            info!("RAG integration test passed");

                            return Ok(TestResult {
                                test_flow: TestFlow::RagIntegration,
                                success: true,
                                error: None,
                                duration_ms: duration.as_millis() as u64,
                                timestamp: chrono::Utc::now(),
                                details: {
                                    let mut details = HashMap::new();
                                    details.insert(
                                        "result".to_string(),
                                        serde_json::to_value(result).unwrap(),
                                    );
                                    details
                                },
                            });
                        }
                    }
                }

                let error_msg = "RAG chain execution did not return the expected result";
                warn!("{}", error_msg);

                Ok(TestResult {
                    test_flow: TestFlow::RagIntegration,
                    success: false,
                    error: Some(error_msg.to_string()),
                    duration_ms: duration.as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                    details: {
                        let mut details = HashMap::new();
                        details.insert("result".to_string(), serde_json::to_value(result).unwrap());
                        details
                    },
                })
            }
            Err(e) => {
                let error_msg = format!("Failed to execute RAG chain: {}", e);
                error!("{}", error_msg);

                Ok(TestResult {
                    test_flow: TestFlow::RagIntegration,
                    success: false,
                    error: Some(error_msg),
                    duration_ms: duration.as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                    details: HashMap::new(),
                })
            }
        }
    }

    /// Execute a persona layer integration test
    async fn execute_persona_layer_test(&self) -> Result<TestResult, AuditError> {
        info!("Executing persona layer integration test");

        let start_time = Instant::now();

        // Define a persona chain for testing
        let chain_definition = json!({
            "name": "audit_persona_test_chain",
            "description": "A persona chain for testing",
            "steps": [
                {
                    "id": "persona_step",
                    "type": "persona",
                    "persona": "helpful_assistant",
                    "input": {
                        "prompt": "Introduce yourself as an AI assistant."
                    }
                }
            ]
        });

        // Execute the chain
        let chain_result = self.execute_chain(chain_definition).await;

        let duration = start_time.elapsed();

        match chain_result {
            Ok(result) => {
                // Verify the result
                if let Some(output) = result.get("output") {
                    if let Some(response) = output.get("response") {
                        if response.as_str().is_some() {
                            info!("Persona layer integration test passed");

                            return Ok(TestResult {
                                test_flow: TestFlow::PersonaLayerIntegration,
                                success: true,
                                error: None,
                                duration_ms: duration.as_millis() as u64,
                                timestamp: chrono::Utc::now(),
                                details: {
                                    let mut details = HashMap::new();
                                    details.insert(
                                        "result".to_string(),
                                        serde_json::to_value(result).unwrap(),
                                    );
                                    details
                                },
                            });
                        }
                    }
                }

                let error_msg = "Persona chain execution did not return the expected result";
                warn!("{}", error_msg);

                Ok(TestResult {
                    test_flow: TestFlow::PersonaLayerIntegration,
                    success: false,
                    error: Some(error_msg.to_string()),
                    duration_ms: duration.as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                    details: {
                        let mut details = HashMap::new();
                        details.insert("result".to_string(), serde_json::to_value(result).unwrap());
                        details
                    },
                })
            }
            Err(e) => {
                let error_msg = format!("Failed to execute persona chain: {}", e);
                error!("{}", error_msg);

                Ok(TestResult {
                    test_flow: TestFlow::PersonaLayerIntegration,
                    success: false,
                    error: Some(error_msg),
                    duration_ms: duration.as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                    details: HashMap::new(),
                })
            }
        }
    }

    /// Execute an end-to-end test
    async fn execute_end_to_end_test(&self) -> Result<TestResult, AuditError> {
        info!("Executing end-to-end test");

        let start_time = Instant::now();

        // Define a complex chain that uses all components
        let chain_definition = json!({
            "name": "audit_e2e_test_chain",
            "description": "An end-to-end chain for testing",
            "steps": [
                {
                    "id": "rag_step",
                    "type": "rag",
                    "input": {
                        "query": "What is IntelliRouter?",
                        "collection": "audit_test_collection"
                    }
                },
                {
                    "id": "persona_step",
                    "type": "persona",
                    "persona": "helpful_assistant",
                    "input": {
                        "prompt": "Answer the question using the retrieved context: {{rag_step.output.context}}",
                        "question": "What is IntelliRouter?"
                    }
                },
                {
                    "id": "function_step",
                    "type": "function",
                    "function": "format_response",
                    "input": {
                        "response": "{{persona_step.output.response}}",
                        "format": "markdown"
                    }
                }
            ]
        });

        // First, create a test collection and add a document
        let create_collection_result = self.create_test_collection().await;

        if let Err(e) = create_collection_result {
            let error_msg = format!("Failed to create test collection: {}", e);
            error!("{}", error_msg);

            return Ok(TestResult {
                test_flow: TestFlow::EndToEndFlow,
                success: false,
                error: Some(error_msg),
                duration_ms: start_time.elapsed().as_millis() as u64,
                timestamp: chrono::Utc::now(),
                details: HashMap::new(),
            });
        }

        // Execute the chain
        let chain_result = self.execute_chain(chain_definition).await;

        let duration = start_time.elapsed();

        // Clean up the test collection
        let _ = self.delete_test_collection().await;

        match chain_result {
            Ok(result) => {
                // Verify the result
                if let Some(output) = result.get("output") {
                    if let Some(formatted_response) = output.get("formatted_response") {
                        if formatted_response.as_str().is_some() {
                            info!("End-to-end test passed");

                            return Ok(TestResult {
                                test_flow: TestFlow::EndToEndFlow,
                                success: true,
                                error: None,
                                duration_ms: duration.as_millis() as u64,
                                timestamp: chrono::Utc::now(),
                                details: {
                                    let mut details = HashMap::new();
                                    details.insert(
                                        "result".to_string(),
                                        serde_json::to_value(result).unwrap(),
                                    );
                                    details
                                },
                            });
                        }
                    }
                }

                let error_msg = "End-to-end chain execution did not return the expected result";
                warn!("{}", error_msg);

                Ok(TestResult {
                    test_flow: TestFlow::EndToEndFlow,
                    success: false,
                    error: Some(error_msg.to_string()),
                    duration_ms: duration.as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                    details: {
                        let mut details = HashMap::new();
                        details.insert("result".to_string(), serde_json::to_value(result).unwrap());
                        details
                    },
                })
            }
            Err(e) => {
                let error_msg = format!("Failed to execute end-to-end chain: {}", e);
                error!("{}", error_msg);

                Ok(TestResult {
                    test_flow: TestFlow::EndToEndFlow,
                    success: false,
                    error: Some(error_msg),
                    duration_ms: duration.as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                    details: HashMap::new(),
                })
            }
        }
    }

    /// Execute a chain
    async fn execute_chain(
        &self,
        chain_definition: serde_json::Value,
    ) -> Result<serde_json::Value, AuditError> {
        // Send the chain to the Chain Engine service
        let response = self
            .client
            .post("http://orchestrator:8080/v1/chains/execute")
            .json(&chain_definition)
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Chain execution failed with status code: {}",
                response.status()
            );
            return Err(AuditError::TestExecutionError(error_msg));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        Ok(result)
    }

    /// Create a test collection for RAG tests
    async fn create_test_collection(&self) -> Result<(), AuditError> {
        // Create a test collection in the RAG service
        let create_collection_request = json!({
            "name": "audit_test_collection",
            "description": "A test collection for audit tests"
        });

        let response = self
            .client
            .post("http://rag-injector:8080/v1/collections")
            .json(&create_collection_request)
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Failed to create test collection with status code: {}",
                response.status()
            );
            return Err(AuditError::TestExecutionError(error_msg));
        }

        // Add a test document to the collection
        let add_document_request = json!({
            "collection": "audit_test_collection",
            "documents": [
                {
                    "text": "IntelliRouter is a system for routing requests to appropriate LLM backends. It includes components for chain execution, RAG, and persona management.",
                    "metadata": {
                        "source": "audit_test"
                    }
                }
            ]
        });

        let response = self
            .client
            .post("http://rag-injector:8080/v1/documents")
            .json(&add_document_request)
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Failed to add test document with status code: {}",
                response.status()
            );
            return Err(AuditError::TestExecutionError(error_msg));
        }

        Ok(())
    }

    /// Delete the test collection
    async fn delete_test_collection(&self) -> Result<(), AuditError> {
        // Delete the test collection
        let response = self
            .client
            .delete("http://rag-injector:8080/v1/collections/audit_test_collection")
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Failed to delete test collection with status code: {}",
                response.status()
            );
            return Err(AuditError::TestExecutionError(error_msg));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_executor_creation() {
        let config = TestConfig::default();
        let report = Arc::new(RwLock::new(AuditReport::new()));
        let executor = TestExecutor::new(config, report);

        assert_eq!(executor.config.test_flows.len(), 1);
        assert_eq!(executor.config.test_flows[0], TestFlow::BasicChainExecution);
    }
}
