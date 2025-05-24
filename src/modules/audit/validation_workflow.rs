//! Validation Workflow
//!
//! This module implements a comprehensive validation workflow that tests service discovery,
//! direct communication, end-to-end flows, data integrity, and error handling.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::RwLock;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use super::report::AuditReport;
use super::service_discovery::ServiceDiscovery;
use super::types::{AuditError, TestFlow, TestResult};
use super::validation::ValidationConfig;
use intellirouter_test_utils::fixtures::audit::{
    CommunicationTestResult, ServiceInfo, ServiceStatus, ServiceType,
};
use intellirouter_test_utils::helpers::communication;

// Use ValidationResult and ValidationType from validation/types.rs
use super::validation::types::{ValidationResult, ValidationType};

/// Validation Workflow
#[derive(Debug)]
pub struct ValidationWorkflow {
    /// Validation configuration
    config: ValidationConfig,
    /// HTTP client for API requests
    client: Client,
    /// Service discovery validator
    service_discovery: ServiceDiscovery,
    /// Shared audit report
    report: Arc<RwLock<AuditReport>>,
    /// Service information
    services: HashMap<ServiceType, ServiceInfo>,
}

impl ValidationWorkflow {
    /// Create a new validation workflow
    pub fn new(
        config: ValidationConfig,
        service_discovery: ServiceDiscovery,
        report: Arc<RwLock<AuditReport>>,
        services: HashMap<ServiceType, ServiceInfo>,
    ) -> Self {
        Self {
            config,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            service_discovery,
            report,
            services,
        }
    }

    /// Run the complete validation workflow
    pub async fn run_validation(&self) -> Result<Vec<ValidationResult>, AuditError> {
        info!("Starting validation workflow");

        let mut results = Vec::new();

        // Step 1: Service discovery validation
        if self.config.validate_service_discovery {
            info!("Step 1: Validating service discovery");
            match self.validate_service_discovery().await {
                Ok(result) => {
                    let result_clone = result.clone();
                    results.push(result);
                    if !result_clone.success && self.config.fail_fast {
                        return Err(AuditError::ServiceDiscoveryError(
                            "Service discovery validation failed".to_string(),
                        ));
                    }
                }
                Err(e) => {
                    error!("Service discovery validation failed: {}", e);
                    if self.config.fail_fast {
                        return Err(e);
                    }
                }
            }
        }

        // Step 2: Direct communication validation
        if self.config.validate_direct_communication {
            info!("Step 2: Validating direct communication");
            match self.validate_direct_communication().await {
                Ok(result) => {
                    let result_clone = result.clone();
                    results.push(result);
                    if !result_clone.success && self.config.fail_fast {
                        return Err(AuditError::CommunicationTestError(
                            "Direct communication validation failed".to_string(),
                        ));
                    }
                }
                Err(e) => {
                    error!("Direct communication validation failed: {}", e);
                    if self.config.fail_fast {
                        return Err(e);
                    }
                }
            }
        }

        // Step 3: End-to-end flow validation
        if self.config.validate_end_to_end_flows {
            info!("Step 3: Validating end-to-end flows");
            match self.validate_end_to_end_flows().await {
                Ok(result) => {
                    let result_clone = result.clone();
                    results.push(result);
                    if !result_clone.success && self.config.fail_fast {
                        return Err(AuditError::TestExecutionError(
                            "End-to-end flow validation failed".to_string(),
                        ));
                    }
                }
                Err(e) => {
                    error!("End-to-end flow validation failed: {}", e);
                    if self.config.fail_fast {
                        return Err(e);
                    }
                }
            }
        }

        // Step 4: Data integrity validation
        if self.config.validate_data_integrity {
            info!("Step 4: Validating data integrity");
            match self.validate_data_integrity().await {
                Ok(result) => {
                    let result_clone = result.clone();
                    results.push(result);
                    if !result_clone.success && self.config.fail_fast {
                        return Err(AuditError::TestExecutionError(
                            "Data integrity validation failed".to_string(),
                        ));
                    }
                }
                Err(e) => {
                    error!("Data integrity validation failed: {}", e);
                    if self.config.fail_fast {
                        return Err(e);
                    }
                }
            }
        }

        // Step 5: Error handling validation
        if self.config.validate_error_handling {
            info!("Step 5: Validating error handling");
            match self.validate_error_handling().await {
                Ok(result) => {
                    let result_clone = result.clone();
                    results.push(result);
                    if !result_clone.success && self.config.fail_fast {
                        return Err(AuditError::TestExecutionError(
                            "Error handling validation failed".to_string(),
                        ));
                    }
                }
                Err(e) => {
                    error!("Error handling validation failed: {}", e);
                    if self.config.fail_fast {
                        return Err(e);
                    }
                }
            }
        }

        // Step 6: Security validation
        if self.config.validate_security {
            info!("Step 6: Validating security");
            match self.validate_security().await {
                Ok(result) => {
                    let result_clone = result.clone();
                    results.push(result);
                    if !result_clone.success && self.config.fail_fast {
                        return Err(AuditError::TestExecutionError(
                            "Security validation failed".to_string(),
                        ));
                    }
                }
                Err(e) => {
                    error!("Security validation failed: {}", e);
                    if self.config.fail_fast {
                        return Err(e);
                    }
                }
            }
        }

        // Check if all validations were successful
        let all_successful = results.iter().all(|r| r.success);

        if all_successful {
            info!("All validations completed successfully");

            // Update report with validation success
            let mut report = self.report.write().await;
            report.add_success("All validations completed successfully");

            Ok(results)
        } else {
            let error_msg = "Some validations failed";
            error!("{}", error_msg);

            // Update report with validation failure
            let mut report = self.report.write().await;
            report.add_error(format!("Validation error: {}", error_msg));

            Ok(results)
        }
    }

    /// Validate service discovery
    async fn validate_service_discovery(&self) -> Result<ValidationResult, AuditError> {
        info!("Validating service discovery");

        let start_time = Instant::now();

        // Use the service discovery validator to validate service discovery
        let result = self.service_discovery.validate_service_discovery().await;

        let duration = start_time.elapsed();

        match result {
            Ok(()) => {
                info!("Service discovery validation passed");

                Ok(ValidationResult {
                    validation_type: ValidationType::ServiceDiscovery,
                    success: true,
                    error: None,
                    duration_ms: duration.as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                    details: HashMap::new(),
                })
            }
            Err(e) => {
                let error_msg = format!("Service discovery validation failed: {}", e);
                error!("{}", error_msg);

                Ok(ValidationResult {
                    validation_type: ValidationType::ServiceDiscovery,
                    success: false,
                    error: Some(error_msg),
                    duration_ms: duration.as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                    details: HashMap::new(),
                })
            }
        }
    }

    /// Validate direct communication between services
    async fn validate_direct_communication(&self) -> Result<ValidationResult, AuditError> {
        info!("Validating direct communication between services");

        let start_time = Instant::now();

        // Use the service discovery validator to validate communication
        let result = self.service_discovery.validate_communication().await;

        let duration = start_time.elapsed();

        match result {
            Ok(()) => {
                info!("Direct communication validation passed");

                Ok(ValidationResult {
                    validation_type: ValidationType::DirectCommunication,
                    success: true,
                    error: None,
                    duration_ms: duration.as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                    details: HashMap::new(),
                })
            }
            Err(e) => {
                let error_msg = format!("Direct communication validation failed: {}", e);
                error!("{}", error_msg);

                Ok(ValidationResult {
                    validation_type: ValidationType::DirectCommunication,
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
        // In a real implementation, this would call the chain engine service
        // For now, we'll simulate the chain execution

        // Get the chain engine URL
        let chain_engine_url = format!(
            "http://{}:{}/api/v1/chains/execute",
            self.services.get(&ServiceType::ChainEngine).unwrap().host,
            self.services.get(&ServiceType::ChainEngine).unwrap().port
        );

        // Execute the chain
        let response = self
            .client
            .post(&chain_engine_url)
            .json(&chain_definition)
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        if !response.status().is_success() {
            return Err(AuditError::TestExecutionError(format!(
                "Chain execution failed with status: {}",
                response.status()
            )));
        }

        let result: Value = response
            .json()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        Ok(result)
    }

    /// Create a test collection for validation
    async fn create_test_collection(&self) -> Result<(), AuditError> {
        // In a real implementation, this would call the RAG manager service
        // For now, we'll simulate the collection creation

        // Get the RAG manager URL
        let rag_manager_url = format!(
            "http://{}:{}/api/v1/collections",
            self.services.get(&ServiceType::RagManager).unwrap().host,
            self.services.get(&ServiceType::RagManager).unwrap().port
        );

        // Create the collection
        let collection_data = json!({
            "name": "validation_test_collection",
            "description": "A test collection for validation"
        });

        let response = self
            .client
            .post(&rag_manager_url)
            .json(&collection_data)
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        if !response.status().is_success() {
            return Err(AuditError::TestExecutionError(format!(
                "Failed to create test collection: {}",
                response.status()
            )));
        }

        // Add a test document to the collection
        let document_url = format!("{}/validation_test_collection/documents", rag_manager_url);
        let document_data = json!({
            "documents": [
                {
                    "id": "doc1",
                    "text": "IntelliRouter is an advanced AI orchestration system that provides seamless integration between different AI components and services."
                }
            ]
        });

        let response = self
            .client
            .post(&document_url)
            .json(&document_data)
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        if !response.status().is_success() {
            return Err(AuditError::TestExecutionError(format!(
                "Failed to add document to test collection: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Delete a test collection
    async fn delete_test_collection(&self) -> Result<(), AuditError> {
        // In a real implementation, this would call the RAG manager service
        // For now, we'll simulate the collection deletion

        // Get the RAG manager URL
        let rag_manager_url = format!(
            "http://{}:{}/api/v1/collections/validation_test_collection",
            self.services.get(&ServiceType::RagManager).unwrap().host,
            self.services.get(&ServiceType::RagManager).unwrap().port
        );

        // Delete the collection
        let response = self
            .client
            .delete(&rag_manager_url)
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        if !response.status().is_success() {
            return Err(AuditError::TestExecutionError(format!(
                "Failed to delete test collection: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Validate end-to-end flows
    async fn validate_end_to_end_flows(&self) -> Result<ValidationResult, AuditError> {
        info!("Validating end-to-end flows");
        let start_time = Instant::now();
        let mut details = HashMap::new();

        // Define test flows to validate
        let test_flows = vec![
            TestFlow::BasicChainExecution,
            TestFlow::RagIntegration,
            TestFlow::EndToEndFlow,
        ];

        let mut success = true;
        let mut error_msg = None;

        // Execute each test flow
        for flow in test_flows {
            info!("Executing test flow: {}", flow);

            match self.execute_test_flow(flow).await {
                Ok(result) => {
                    if !result.success {
                        success = false;
                        error_msg = result.error.clone();
                        details.insert(
                            format!("flow_{}", flow),
                            json!({
                                "success": false,
                                "error": result.error,
                                "duration_ms": result.duration_ms
                            }),
                        );

                        if self.config.fail_fast {
                            break;
                        }
                    } else {
                        details.insert(
                            format!("flow_{}", flow),
                            json!({
                                "success": true,
                                "duration_ms": result.duration_ms
                            }),
                        );
                    }
                }
                Err(e) => {
                    success = false;
                    error_msg = Some(format!("Failed to execute test flow {}: {}", flow, e));
                    details.insert(
                        format!("flow_{}", flow),
                        json!({
                            "success": false,
                            "error": error_msg.clone(),
                        }),
                    );

                    if self.config.fail_fast {
                        break;
                    }
                }
            }
        }

        let duration = start_time.elapsed();

        if success {
            info!("End-to-end flow validation passed");
        } else {
            error!("End-to-end flow validation failed: {:?}", error_msg);
        }

        Ok(ValidationResult {
            validation_type: ValidationType::EndToEndFlow,
            success,
            error: error_msg,
            duration_ms: duration.as_millis() as u64,
            timestamp: chrono::Utc::now(),
            details,
        })
    }

    /// Execute a specific test flow
    async fn execute_test_flow(&self, flow: TestFlow) -> Result<TestResult, AuditError> {
        match flow {
            TestFlow::BasicChainExecution => self.execute_basic_chain_test().await,
            TestFlow::RagIntegration => self.execute_rag_integration_test().await,
            TestFlow::EndToEndFlow => self.execute_end_to_end_test().await,
            TestFlow::PersonaLayerIntegration => self.execute_persona_layer_test().await,
        }
    }

    /// Execute a basic chain test
    async fn execute_basic_chain_test(&self) -> Result<TestResult, AuditError> {
        let start_time = Instant::now();
        let mut details = HashMap::new();

        // Create a simple chain definition
        let chain_definition = json!({
            "name": "validation_test_chain",
            "description": "A test chain for validation",
            "steps": [
                {
                    "id": "step1",
                    "type": "llm",
                    "model": "mock-model",
                    "prompt": "Generate a short poem about AI"
                }
            ]
        });

        // Execute the chain
        match self.execute_chain(chain_definition).await {
            Ok(result) => {
                details.insert("chain_result".to_string(), result.clone());

                // Verify the result contains expected fields
                if result.get("output").is_some() {
                    Ok(TestResult {
                        test_flow: TestFlow::BasicChainExecution,
                        success: true,
                        error: None,
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                        details,
                    })
                } else {
                    Ok(TestResult {
                        test_flow: TestFlow::BasicChainExecution,
                        success: false,
                        error: Some("Chain execution result missing expected fields".to_string()),
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                        details,
                    })
                }
            }
            Err(e) => {
                details.insert("error".to_string(), json!(e.to_string()));

                Ok(TestResult {
                    test_flow: TestFlow::BasicChainExecution,
                    success: false,
                    error: Some(format!("Chain execution failed: {}", e)),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                    details,
                })
            }
        }
    }

    /// Execute a RAG integration test
    async fn execute_rag_integration_test(&self) -> Result<TestResult, AuditError> {
        let start_time = Instant::now();
        let mut details = HashMap::new();

        // Create a test collection
        match self.create_test_collection().await {
            Ok(()) => {
                details.insert("collection_created".to_string(), json!(true));

                // Create a chain that uses RAG
                let chain_definition = json!({
                    "name": "rag_test_chain",
                    "description": "A test chain for RAG validation",
                    "steps": [
                        {
                            "id": "step1",
                            "type": "retrieval",
                            "collection": "validation_test_collection",
                            "query": "What is IntelliRouter?",
                            "output": "context"
                        },
                        {
                            "id": "step2",
                            "type": "llm",
                            "model": "mock-model",
                            "prompt": "Answer the question using the context: {{context}}. Question: What is IntelliRouter?",
                            "output": "answer"
                        }
                    ]
                });

                // Execute the chain
                match self.execute_chain(chain_definition).await {
                    Ok(result) => {
                        details.insert("chain_result".to_string(), result.clone());

                        // Clean up the test collection
                        if let Err(e) = self.delete_test_collection().await {
                            warn!("Failed to delete test collection: {}", e);
                            details.insert("cleanup_error".to_string(), json!(e.to_string()));
                        } else {
                            details.insert("collection_deleted".to_string(), json!(true));
                        }

                        // Verify the result contains expected fields
                        if result.get("answer").is_some() {
                            Ok(TestResult {
                                test_flow: TestFlow::RagIntegration,
                                success: true,
                                error: None,
                                duration_ms: start_time.elapsed().as_millis() as u64,
                                timestamp: chrono::Utc::now(),
                                details,
                            })
                        } else {
                            Ok(TestResult {
                                test_flow: TestFlow::RagIntegration,
                                success: false,
                                error: Some(
                                    "RAG chain execution result missing expected fields"
                                        .to_string(),
                                ),
                                duration_ms: start_time.elapsed().as_millis() as u64,
                                timestamp: chrono::Utc::now(),
                                details,
                            })
                        }
                    }
                    Err(e) => {
                        // Clean up the test collection
                        if let Err(cleanup_err) = self.delete_test_collection().await {
                            warn!("Failed to delete test collection: {}", cleanup_err);
                            details.insert(
                                "cleanup_error".to_string(),
                                json!(cleanup_err.to_string()),
                            );
                        } else {
                            details.insert("collection_deleted".to_string(), json!(true));
                        }

                        details.insert("error".to_string(), json!(e.to_string()));

                        Ok(TestResult {
                            test_flow: TestFlow::RagIntegration,
                            success: false,
                            error: Some(format!("RAG chain execution failed: {}", e)),
                            duration_ms: start_time.elapsed().as_millis() as u64,
                            timestamp: chrono::Utc::now(),
                            details,
                        })
                    }
                }
            }
            Err(e) => {
                details.insert("error".to_string(), json!(e.to_string()));

                Ok(TestResult {
                    test_flow: TestFlow::RagIntegration,
                    success: false,
                    error: Some(format!("Failed to create test collection: {}", e)),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                    details,
                })
            }
        }
    }

    /// Execute an end-to-end test
    async fn execute_end_to_end_test(&self) -> Result<TestResult, AuditError> {
        let start_time = Instant::now();
        let mut details = HashMap::new();

        // Create a complex chain that uses multiple services
        let chain_definition = json!({
            "name": "end_to_end_test_chain",
            "description": "A test chain for end-to-end validation",
            "steps": [
                {
                    "id": "step1",
                    "type": "llm",
                    "model": "mock-model",
                    "prompt": "Generate a question about AI",
                    "output": "question"
                },
                {
                    "id": "step2",
                    "type": "function",
                    "function": "process_question",
                    "args": {
                        "question": "{{question}}"
                    },
                    "output": "processed_question"
                },
                {
                    "id": "step3",
                    "type": "llm",
                    "model": "mock-model",
                    "prompt": "Answer this question: {{processed_question}}",
                    "output": "answer"
                },
                {
                    "id": "step4",
                    "type": "function",
                    "function": "format_response",
                    "args": {
                        "answer": "{{answer}}"
                    },
                    "output": "formatted_answer"
                }
            ]
        });

        // Execute the chain
        match self.execute_chain(chain_definition).await {
            Ok(result) => {
                details.insert("chain_result".to_string(), result.clone());

                // Verify the result contains expected fields
                if result.get("formatted_answer").is_some() {
                    Ok(TestResult {
                        test_flow: TestFlow::EndToEndFlow,
                        success: true,
                        error: None,
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                        details,
                    })
                } else {
                    Ok(TestResult {
                        test_flow: TestFlow::EndToEndFlow,
                        success: false,
                        error: Some(
                            "End-to-end chain execution result missing expected fields".to_string(),
                        ),
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                        details,
                    })
                }
            }
            Err(e) => {
                details.insert("error".to_string(), json!(e.to_string()));

                Ok(TestResult {
                    test_flow: TestFlow::EndToEndFlow,
                    success: false,
                    error: Some(format!("End-to-end chain execution failed: {}", e)),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                    details,
                })
            }
        }
    }

    /// Execute a persona layer test
    async fn execute_persona_layer_test(&self) -> Result<TestResult, AuditError> {
        let start_time = Instant::now();
        let mut details = HashMap::new();

        // Create a chain that uses the persona layer
        let chain_definition = json!({
            "name": "persona_layer_test_chain",
            "description": "A test chain for persona layer validation",
            "steps": [
                {
                    "id": "step1",
                    "type": "persona",
                    "persona": "expert",
                    "input": "Explain quantum computing",
                    "output": "explanation"
                }
            ]
        });

        // Execute the chain
        match self.execute_chain(chain_definition).await {
            Ok(result) => {
                details.insert("chain_result".to_string(), result.clone());

                // Verify the result contains expected fields
                if result.get("explanation").is_some() {
                    Ok(TestResult {
                        test_flow: TestFlow::PersonaLayerIntegration,
                        success: true,
                        error: None,
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                        details,
                    })
                } else {
                    Ok(TestResult {
                        test_flow: TestFlow::PersonaLayerIntegration,
                        success: false,
                        error: Some(
                            "Persona layer chain execution result missing expected fields"
                                .to_string(),
                        ),
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                        details,
                    })
                }
            }
            Err(e) => {
                details.insert("error".to_string(), json!(e.to_string()));

                Ok(TestResult {
                    test_flow: TestFlow::PersonaLayerIntegration,
                    success: false,
                    error: Some(format!("Persona layer chain execution failed: {}", e)),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                    details,
                })
            }
        }
    }

    /// Validate data integrity
    async fn validate_data_integrity(&self) -> Result<ValidationResult, AuditError> {
        info!("Validating data integrity");
        let start_time = Instant::now();
        let mut details = HashMap::new();
        let mut success = true;
        let mut error_msg = None;

        // Test data integrity across services
        let test_cases = vec![
            self.validate_chain_engine_data_integrity().await,
            self.validate_rag_manager_data_integrity().await,
            self.validate_persona_layer_data_integrity().await,
        ];

        // Process test results
        for (i, result) in test_cases.into_iter().enumerate() {
            match result {
                Ok(test_success) => {
                    details.insert(format!("test_case_{}", i), json!({"success": test_success}));
                    if !test_success {
                        success = false;
                        error_msg = Some(format!("Data integrity test case {} failed", i));

                        if self.config.fail_fast {
                            break;
                        }
                    }
                }
                Err(e) => {
                    success = false;
                    let err_msg = format!("Data integrity test case {} error: {}", i, e);
                    error_msg = Some(err_msg.clone());
                    details.insert(format!("test_case_{}", i), json!({"error": err_msg}));

                    if self.config.fail_fast {
                        break;
                    }
                }
            }
        }

        let duration = start_time.elapsed();

        if success {
            info!("Data integrity validation passed");
        } else {
            error!("Data integrity validation failed: {:?}", error_msg);
        }

        Ok(ValidationResult {
            validation_type: ValidationType::DataIntegrity,
            success,
            error: error_msg,
            duration_ms: duration.as_millis() as u64,
            timestamp: chrono::Utc::now(),
            details,
        })
    }

    /// Validate chain engine data integrity
    async fn validate_chain_engine_data_integrity(&self) -> Result<bool, AuditError> {
        info!("Validating chain engine data integrity");

        // Create a test chain
        let chain_definition = json!({
            "name": "data_integrity_test_chain",
            "description": "A test chain for data integrity validation",
            "steps": [
                {
                    "id": "step1",
                    "type": "llm",
                    "model": "mock-model",
                    "prompt": "Generate a JSON object with the following fields: name, age, city",
                    "output": "json_data"
                }
            ]
        });

        // Execute the chain
        let result = self.execute_chain(chain_definition).await?;

        // Verify the result contains valid JSON data
        if let Some(json_data) = result.get("json_data") {
            if json_data.is_object() {
                // Check if the JSON has the expected fields
                let json_obj = json_data.as_object().unwrap();
                if json_obj.contains_key("name")
                    && json_obj.contains_key("age")
                    && json_obj.contains_key("city")
                {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Validate RAG manager data integrity
    async fn validate_rag_manager_data_integrity(&self) -> Result<bool, AuditError> {
        info!("Validating RAG manager data integrity");

        // Create a test collection
        self.create_test_collection().await?;

        // Get the RAG manager URL
        let rag_manager_url = format!(
            "http://{}:{}/api/v1/collections/validation_test_collection/query",
            self.services.get(&ServiceType::RagManager).unwrap().host,
            self.services.get(&ServiceType::RagManager).unwrap().port
        );

        // Query the collection
        let query_data = json!({
            "query": "IntelliRouter",
            "top_k": 1
        });

        let response = self
            .client
            .post(&rag_manager_url)
            .json(&query_data)
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        // Clean up the test collection
        let _ = self.delete_test_collection().await;

        if !response.status().is_success() {
            return Ok(false);
        }

        // Parse the response
        let result: Value = response
            .json()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        // Verify the result contains the expected data
        if let Some(results) = result.get("results").and_then(|r| r.as_array()) {
            if !results.is_empty() {
                // Check if the result has the expected fields
                let first_result = &results[0];
                if first_result.get("text").is_some() && first_result.get("score").is_some() {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Validate persona layer data integrity
    async fn validate_persona_layer_data_integrity(&self) -> Result<bool, AuditError> {
        info!("Validating persona layer data integrity");

        // Get the persona layer URL
        let persona_layer_url = format!(
            "http://{}:{}/api/v1/personas",
            self.services.get(&ServiceType::PersonaLayer).unwrap().host,
            self.services.get(&ServiceType::PersonaLayer).unwrap().port
        );

        // Get the list of personas
        let response = self
            .client
            .get(&persona_layer_url)
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        if !response.status().is_success() {
            return Ok(false);
        }

        // Parse the response
        let result: Value = response
            .json()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        // Verify the result contains the expected data
        if let Some(personas) = result.get("personas").and_then(|p| p.as_array()) {
            if !personas.is_empty() {
                // Check if each persona has the expected fields
                for persona in personas {
                    if persona.get("id").is_none()
                        || persona.get("name").is_none()
                        || persona.get("description").is_none()
                    {
                        return Ok(false);
                    }
                }
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Validate error handling
    async fn validate_error_handling(&self) -> Result<ValidationResult, AuditError> {
        info!("Validating error handling");
        let start_time = Instant::now();
        let mut details = HashMap::new();
        let mut success = true;
        let mut error_msg = None;

        // Test error handling across services
        let test_cases = vec![
            self.validate_invalid_request_handling().await,
            self.validate_timeout_handling().await,
            self.validate_rate_limit_handling().await,
        ];

        // Process test results
        for (i, result) in test_cases.into_iter().enumerate() {
            match result {
                Ok(test_success) => {
                    details.insert(format!("test_case_{}", i), json!({"success": test_success}));
                    if !test_success {
                        success = false;
                        error_msg = Some(format!("Error handling test case {} failed", i));

                        if self.config.fail_fast {
                            break;
                        }
                    }
                }
                Err(e) => {
                    success = false;
                    let err_msg = format!("Error handling test case {} error: {}", i, e);
                    error_msg = Some(err_msg.clone());
                    details.insert(format!("test_case_{}", i), json!({"error": err_msg}));

                    if self.config.fail_fast {
                        break;
                    }
                }
            }
        }

        let duration = start_time.elapsed();

        if success {
            info!("Error handling validation passed");
        } else {
            error!("Error handling validation failed: {:?}", error_msg);
        }

        Ok(ValidationResult {
            validation_type: ValidationType::ErrorHandling,
            success,
            error: error_msg,
            duration_ms: duration.as_millis() as u64,
            timestamp: chrono::Utc::now(),
            details,
        })
    }

    /// Validate invalid request handling
    async fn validate_invalid_request_handling(&self) -> Result<bool, AuditError> {
        info!("Validating invalid request handling");

        // Get the router URL
        let router_url = format!(
            "http://{}:{}/api/v1/chat/completions",
            self.services.get(&ServiceType::Router).unwrap().host,
            self.services.get(&ServiceType::Router).unwrap().port
        );

        // Send an invalid request (missing required fields)
        let invalid_request = json!({
            // Missing 'model' field
            "messages": [
                {
                    "role": "user",
                    "content": "Hello"
                }
            ]
        });

        let response = self
            .client
            .post(&router_url)
            .json(&invalid_request)
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        // Verify the response is a 400 Bad Request
        if response.status().as_u16() != 400 {
            return Ok(false);
        }

        // Parse the error response
        let error: Value = response
            .json()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        // Verify the error response has the expected structure
        if let Some(error_obj) = error.get("error") {
            if error_obj.get("message").is_some() && error_obj.get("type").is_some() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Validate timeout handling
    async fn validate_timeout_handling(&self) -> Result<bool, AuditError> {
        info!("Validating timeout handling");

        // Create a chain definition that will timeout
        let chain_definition = json!({
            "name": "timeout_test_chain",
            "description": "A test chain for timeout handling validation",
            "steps": [
                {
                    "id": "step1",
                    "type": "function",
                    "function": "sleep",
                    "args": {
                        "duration_ms": 10000 // 10 seconds, should trigger timeout
                    }
                }
            ]
        });

        // Set a short timeout for the request
        let timeout_duration = Duration::from_millis(1000); // 1 second

        // Execute the chain with a timeout
        match timeout(timeout_duration, self.execute_chain(chain_definition)).await {
            Ok(result) => {
                // If the request completed successfully, it should have returned an error
                match result {
                    Ok(_) => Ok(false), // Unexpected success
                    Err(_) => Ok(true), // Expected error
                }
            }
            Err(_) => {
                // Timeout occurred as expected
                Ok(true)
            }
        }
    }

    /// Validate rate limit handling
    async fn validate_rate_limit_handling(&self) -> Result<bool, AuditError> {
        info!("Validating rate limit handling");

        // Get the router URL
        let router_url = format!(
            "http://{}:{}/api/v1/chat/completions",
            self.services.get(&ServiceType::Router).unwrap().host,
            self.services.get(&ServiceType::Router).unwrap().port
        );

        // Create a valid request
        let request = json!({
            "model": "mock-rate-limited-model", // Special model that simulates rate limiting
            "messages": [
                {
                    "role": "user",
                    "content": "Hello"
                }
            ]
        });

        // Send multiple requests in quick succession to trigger rate limiting
        let mut rate_limited = false;
        for _ in 0..10 {
            let response = self
                .client
                .post(&router_url)
                .json(&request)
                .send()
                .await
                .map_err(|e| AuditError::HttpError(e))?;

            // Check if we got a rate limit response (429 Too Many Requests)
            if response.status().as_u16() == 429 {
                rate_limited = true;
                break;
            }
        }

        Ok(rate_limited)
    }

    /// Validate security
    async fn validate_security(&self) -> Result<ValidationResult, AuditError> {
        info!("Validating security");
        let start_time = Instant::now();
        let mut details = HashMap::new();
        let mut success = true;
        let mut error_msg = None;

        // Test security features across services
        let test_cases = vec![
            self.validate_jwt_authentication().await,
            self.validate_mtls_encryption().await,
            self.validate_authorization_checks().await,
        ];

        // Process test results
        for (i, result) in test_cases.into_iter().enumerate() {
            match result {
                Ok(test_success) => {
                    details.insert(format!("test_case_{}", i), json!({"success": test_success}));
                    if !test_success {
                        success = false;
                        error_msg = Some(format!("Security test case {} failed", i));

                        if self.config.fail_fast {
                            break;
                        }
                    }
                }
                Err(e) => {
                    success = false;
                    let err_msg = format!("Security test case {} error: {}", i, e);
                    error_msg = Some(err_msg.clone());
                    details.insert(format!("test_case_{}", i), json!({"error": err_msg}));

                    if self.config.fail_fast {
                        break;
                    }
                }
            }
        }

        let duration = start_time.elapsed();

        if success {
            info!("Security validation passed");
        } else {
            error!("Security validation failed: {:?}", error_msg);
        }

        Ok(ValidationResult {
            validation_type: ValidationType::Security,
            success,
            error: error_msg,
            duration_ms: duration.as_millis() as u64,
            timestamp: chrono::Utc::now(),
            details,
        })
    }

    /// Validate JWT authentication
    async fn validate_jwt_authentication(&self) -> Result<bool, AuditError> {
        info!("Validating JWT authentication");

        // Get the router URL with a protected endpoint
        let router_url = format!(
            "http://{}:{}/api/v1/protected/models",
            self.services.get(&ServiceType::Router).unwrap().host,
            self.services.get(&ServiceType::Router).unwrap().port
        );

        // First try without a token
        let response_without_token = self
            .client
            .get(&router_url)
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        // Should get a 401 Unauthorized
        if response_without_token.status().as_u16() != 401 {
            return Ok(false);
        }

        // Now try with a valid token
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IlRlc3QgVXNlciIsImlhdCI6MTUxNjIzOTAyMn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

        let response_with_token = self
            .client
            .get(&router_url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        // Should get a 200 OK
        if response_with_token.status().as_u16() != 200 {
            return Ok(false);
        }

        // Try with an invalid token
        let invalid_token = "invalid.token.here";

        let response_with_invalid_token = self
            .client
            .get(&router_url)
            .header("Authorization", format!("Bearer {}", invalid_token))
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        // Should get a 401 Unauthorized
        if response_with_invalid_token.status().as_u16() != 401 {
            return Ok(false);
        }

        Ok(true)
    }

    /// Validate MTLS encryption
    async fn validate_mtls_encryption(&self) -> Result<bool, AuditError> {
        info!("Validating MTLS encryption");

        // Get the router URL with a MTLS-protected endpoint
        let router_url = format!(
            "http://{}:{}/api/v1/mtls/status",
            self.services.get(&ServiceType::Router).unwrap().host,
            self.services.get(&ServiceType::Router).unwrap().port
        );

        // Create a client with MTLS certificates
        let client_with_certs = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        // Try with MTLS certificates (simulated by adding a special header)
        let response_with_certs = client_with_certs
            .get(&router_url)
            .header("X-MTLS-Cert", "SIMULATED_CERT")
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        // Should get a 200 OK
        if response_with_certs.status().as_u16() != 200 {
            return Ok(false);
        }

        // Try without MTLS certificates (using the regular client)
        let response_without_certs = self
            .client
            .get(&router_url)
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        // Should get a 403 Forbidden
        if response_without_certs.status().as_u16() != 403 {
            return Ok(false);
        }

        Ok(true)
    }

    /// Validate authorization checks
    async fn validate_authorization_checks(&self) -> Result<bool, AuditError> {
        info!("Validating authorization checks");

        // Get the router URL with a role-protected endpoint
        let router_url = format!(
            "http://{}:{}/api/v1/admin/settings",
            self.services.get(&ServiceType::Router).unwrap().host,
            self.services.get(&ServiceType::Router).unwrap().port
        );

        // Create a token with admin role
        let admin_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkFkbWluIFVzZXIiLCJyb2xlIjoiYWRtaW4iLCJpYXQiOjE1MTYyMzkwMjJ9.KjCZT-DM9W8XJl6FoRX39KD3cGnICVMXZgvqKwU8JHM";

        // Create a token with user role
        let user_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI5ODc2NTQzMjEwIiwibmFtZSI6IlJlZ3VsYXIgVXNlciIsInJvbGUiOiJ1c2VyIiwiaWF0IjoxNTE2MjM5MDIyfQ.QgcMEgE1jvxtXVyGjq-BJQ_NXK_UJmL-hFRWHSU9U-E";

        // Try with admin token
        let response_with_admin = self
            .client
            .get(&router_url)
            .header("Authorization", format!("Bearer {}", admin_token))
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        // Should get a 200 OK
        if response_with_admin.status().as_u16() != 200 {
            return Ok(false);
        }

        // Try with user token
        let response_with_user = self
            .client
            .get(&router_url)
            .header("Authorization", format!("Bearer {}", user_token))
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        // Should get a 403 Forbidden
        if response_with_user.status().as_u16() != 403 {
            return Ok(false);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation_workflow_creation() {
        // Create a validation workflow
        let config = ValidationConfig::default();
        let report = Arc::new(RwLock::new(AuditReport::new()));
        let discovery_config = DiscoveryConfig::default();
        let service_discovery = ServiceDiscovery::new(discovery_config, report.clone());
        let services = HashMap::new();

        let workflow = ValidationWorkflow::new(config, service_discovery, report, services);

        assert!(workflow.config.validate_service_discovery);
        assert!(workflow.config.validate_direct_communication);
        assert!(workflow.config.validate_end_to_end_flows);
        assert!(workflow.config.validate_data_integrity);
        assert!(workflow.config.validate_error_handling);
        assert!(workflow.config.validate_security);
    }
}
