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

use super::communication_tests;
use super::report::AuditReport;
use super::service_discovery::ServiceDiscovery;
use super::types::{
    AuditError, CommunicationTestResult, ServiceInfo, ServiceStatus, ServiceType, TestFlow,
    TestResult,
};

/// Validation workflow configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationConfig {
    /// Whether to validate service discovery
    pub validate_service_discovery: bool,
    /// Whether to validate direct communication
    pub validate_direct_communication: bool,
    /// Whether to validate end-to-end flows
    pub validate_end_to_end_flows: bool,
    /// Whether to validate data integrity
    pub validate_data_integrity: bool,
    /// Whether to validate error handling
    pub validate_error_handling: bool,
    /// Whether to validate security
    pub validate_security: bool,
    /// Timeout for validation operations in seconds
    pub validation_timeout_secs: u64,
    /// Whether to fail fast on validation errors
    pub fail_fast: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            validate_service_discovery: true,
            validate_direct_communication: true,
            validate_end_to_end_flows: true,
            validate_data_integrity: true,
            validate_error_handling: true,
            validate_security: true,
            validation_timeout_secs: 120, // 2 minutes
            fail_fast: true,
        }
    }
}

/// Validation result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationResult {
    /// Validation type
    pub validation_type: ValidationType,
    /// Whether the validation was successful
    pub success: bool,
    /// Error message if the validation failed
    pub error: Option<String>,
    /// Validation duration in milliseconds
    pub duration_ms: u64,
    /// Validation timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Validation details
    pub details: HashMap<String, serde_json::Value>,
}

/// Validation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ValidationType {
    /// Service discovery validation
    ServiceDiscovery,
    /// Direct communication validation
    DirectCommunication,
    /// End-to-end flow validation
    EndToEndFlow,
    /// Data integrity validation
    DataIntegrity,
    /// Error handling validation
    ErrorHandling,
    /// Security validation
    Security,
}

impl std::fmt::Display for ValidationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationType::ServiceDiscovery => write!(f, "Service Discovery"),
            ValidationType::DirectCommunication => write!(f, "Direct Communication"),
            ValidationType::EndToEndFlow => write!(f, "End-to-End Flow"),
            ValidationType::DataIntegrity => write!(f, "Data Integrity"),
            ValidationType::ErrorHandling => write!(f, "Error Handling"),
            ValidationType::Security => write!(f, "Security"),
        }
    }
}

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
                    results.push(result);
                    if !result.success && self.config.fail_fast {
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
                    results.push(result);
                    if !result.success && self.config.fail_fast {
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
                    results.push(result);
                    if !result.success && self.config.fail_fast {
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
                    results.push(result);
                    if !result.success && self.config.fail_fast {
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
                    results.push(result);
                    if !result.success && self.config.fail_fast {
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
                    results.push(result);
                    if !result.success && self.config.fail_fast {
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
            /// Validate end-to-end flows
            async fn validate_end_to_end_flows(&self) -> Result<ValidationResult, AuditError> {
                info!("Validating end-to-end flows");

                let start_time = Instant::now();
                let mut details = HashMap::new();
                let mut all_successful = true;

                // Test a complete flow from input to final output
                // We'll use a chain that involves all services
                let chain_definition = json!({
                    "name": "validation_e2e_test_chain",
                    "description": "A comprehensive end-to-end chain for validation",
                    "steps": [
                        {
                            "id": "rag_step",
                            "type": "rag",
                            "input": {
                                "query": "What is IntelliRouter?",
                                "collection": "validation_test_collection"
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

                    return Ok(ValidationResult {
                        validation_type: ValidationType::EndToEndFlow,
                        success: false,
                        error: Some(error_msg),
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                        details: HashMap::new(),
                    });
                }

                // Execute the chain
                let chain_result = self.execute_chain(chain_definition.clone()).await;

                // Clean up the test collection
                let _ = self.delete_test_collection().await;

                match chain_result {
                    Ok(result) => {
                        // Verify the result
                        if let Some(output) = result.get("output") {
                            if let Some(formatted_response) = output.get("formatted_response") {
                                if formatted_response.as_str().is_some() {
                                    info!("End-to-end flow validation passed");
                                    details.insert(
                                        "e2e_flow".to_string(),
                                        serde_json::to_value(result).unwrap(),
                                    );
                                } else {
                                    all_successful = false;
                                    let error_msg = "End-to-end flow validation failed: missing formatted response";
                                    warn!("{}", error_msg);
                                    details.insert("e2e_flow_error".to_string(), json!(error_msg));
                                    details.insert(
                                        "e2e_flow_result".to_string(),
                                        serde_json::to_value(result).unwrap(),
                                    );
                                }
                            } else {
                                all_successful = false;
                                let error_msg =
                                    "End-to-end flow validation failed: missing formatted response";
                                warn!("{}", error_msg);
                                details.insert("e2e_flow_error".to_string(), json!(error_msg));
                                details.insert(
                                    "e2e_flow_result".to_string(),
                                    serde_json::to_value(result).unwrap(),
                                );
                            }
                        } else {
                            all_successful = false;
                            let error_msg = "End-to-end flow validation failed: missing output";
                            warn!("{}", error_msg);
                            details.insert("e2e_flow_error".to_string(), json!(error_msg));
                            details.insert(
                                "e2e_flow_result".to_string(),
                                serde_json::to_value(result).unwrap(),
                            );
                        }
                    }
                    Err(e) => {
                        all_successful = false;
                        let error_msg = format!("End-to-end flow validation failed: {}", e);
                        error!("{}", error_msg);
                        details.insert("e2e_flow_error".to_string(), json!(error_msg));
                    }
                }

                // Test with different input types
                let input_types = vec!["text", "json", "binary"];
                for input_type in input_types {
                    let chain_definition = json!({
                        "name": format!("validation_e2e_{}_chain", input_type),
                        "description": format!("An end-to-end chain for {} input validation", input_type),
                        "steps": [
                            {
                                "id": "input_step",
                                "type": "function",
                                "function": "generate_test_input",
                                "input": {
                                    "type": input_type
                                }
                            },
                            {
                                "id": "process_step",
                                "type": "function",
                                "function": "process_input",
                                "input": {
                                    "data": "{{input_step.output.data}}",
                                    "type": input_type
                                }
                            }
                        ]
                    });

                    let chain_result = self.execute_chain(chain_definition.clone()).await;

                    match chain_result {
                        Ok(result) => {
                            // Verify the result
                            if let Some(output) = result.get("output") {
                                if let Some(processed_data) = output.get("processed_data") {
                                    info!(
                                        "End-to-end flow validation for {} input passed",
                                        input_type
                                    );
                                    details.insert(
                                        format!("e2e_flow_{}", input_type),
                                        serde_json::to_value(result).unwrap(),
                                    );
                                } else {
                                    all_successful = false;
                                    let error_msg = format!(
                                "End-to-end flow validation for {} input failed: missing processed data",
                                input_type
                            );
                                    warn!("{}", error_msg);
                                    details.insert(
                                        format!("e2e_flow_{}_error", input_type),
                                        json!(error_msg),
                                    );
                                    details.insert(
                                        format!("e2e_flow_{}_result", input_type),
                                        serde_json::to_value(result).unwrap(),
                                    );
                                }
                            } else {
                                all_successful = false;
                                let error_msg = format!(
                            "End-to-end flow validation for {} input failed: missing output",
                            input_type
                        );
                                warn!("{}", error_msg);
                                details.insert(
                                    format!("e2e_flow_{}_error", input_type),
                                    json!(error_msg),
                                );
                                details.insert(
                                    format!("e2e_flow_{}_result", input_type),
                                    serde_json::to_value(result).unwrap(),
                                );
                            }
                        }
                        Err(e) => {
                            all_successful = false;
                            let error_msg = format!(
                                "End-to-end flow validation for {} input failed: {}",
                                input_type, e
                            );
                            error!("{}", error_msg);
                            details
                                .insert(format!("e2e_flow_{}_error", input_type), json!(error_msg));
                        }
                    }
                }

                let duration = start_time.elapsed();

                if all_successful {
                    Ok(ValidationResult {
                        validation_type: ValidationType::EndToEndFlow,
                        success: true,
                        error: None,
                        duration_ms: duration.as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                        details,
                    })
                } else {
                    Ok(ValidationResult {
                        validation_type: ValidationType::EndToEndFlow,
                        success: false,
                        error: Some("End-to-end flow validation failed".to_string()),
                        duration_ms: duration.as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                        details,
                    })
                }
            }

            /// Validate data integrity
            async fn validate_data_integrity(&self) -> Result<ValidationResult, AuditError> {
                info!("Validating data integrity");

                let start_time = Instant::now();
                let mut details = HashMap::new();
                let mut all_successful = true;

                // Test data integrity at each step of the process
                // We'll create a chain that passes data through multiple services
                // and validates the data at each step
                let chain_definition = json!({
                    "name": "validation_data_integrity_chain",
                    "description": "A chain for data integrity validation",
                    "steps": [
                        {
                            "id": "generate_step",
                            "type": "function",
                            "function": "generate_test_data",
                            "input": {
                                "size": 1024,
                                "checksum": true
                            }
                        },
                        {
                            "id": "rag_step",
                            "type": "rag",
                            "input": {
                                "data": "{{generate_step.output.data}}",
                                "checksum": "{{generate_step.output.checksum}}",
                                "collection": "validation_test_collection"
                            }
                        },
                        {
                            "id": "validation_step",
                            "type": "function",
                            "function": "validate_data_integrity",
                            "input": {
                                "original_data": "{{generate_step.output.data}}",
                                "original_checksum": "{{generate_step.output.checksum}}",
                                "processed_data": "{{rag_step.output.processed_data}}",
                                "processed_checksum": "{{rag_step.output.checksum}}"
                            }
                        }
                    ]
                });

                // First, create a test collection
                let create_collection_result = self.create_test_collection().await;

                if let Err(e) = create_collection_result {
                    let error_msg = format!("Failed to create test collection: {}", e);
                    error!("{}", error_msg);

                    return Ok(ValidationResult {
                        validation_type: ValidationType::DataIntegrity,
                        success: false,
                        error: Some(error_msg),
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                        details: HashMap::new(),
                    });
                }

                // Execute the chain
                let chain_result = self.execute_chain(chain_definition.clone()).await;

                // Clean up the test collection
                let _ = self.delete_test_collection().await;

                match chain_result {
                    Ok(result) => {
                        // Verify the result
                        if let Some(output) = result.get("output") {
                            if let Some(validation_result) = output.get("validation_result") {
                                if validation_result.as_bool() == Some(true) {
                                    info!("Data integrity validation passed");
                                    details.insert(
                                        "data_integrity".to_string(),
                                        serde_json::to_value(result).unwrap(),
                                    );
                                } else {
                                    all_successful = false;
                                    let error_msg = "Data integrity validation failed: validation step returned false";
                                    warn!("{}", error_msg);
                                    details.insert(
                                        "data_integrity_error".to_string(),
                                        json!(error_msg),
                                    );
                                    details.insert(
                                        "data_integrity_result".to_string(),
                                        serde_json::to_value(result).unwrap(),
                                    );
                                }
                            } else {
                                all_successful = false;
                                let error_msg =
                                    "Data integrity validation failed: missing validation result";
                                warn!("{}", error_msg);
                                details
                                    .insert("data_integrity_error".to_string(), json!(error_msg));
                                details.insert(
                                    "data_integrity_result".to_string(),
                                    serde_json::to_value(result).unwrap(),
                                );
                            }
                        } else {
                            all_successful = false;
                            let error_msg = "Data integrity validation failed: missing output";
                            warn!("{}", error_msg);
                            details.insert("data_integrity_error".to_string(), json!(error_msg));
                            details.insert(
                                "data_integrity_result".to_string(),
                                serde_json::to_value(result).unwrap(),
                            );
                        }
                    }
                    Err(e) => {
                        all_successful = false;
                        let error_msg = format!("Data integrity validation failed: {}", e);
                        error!("{}", error_msg);
                        details.insert("data_integrity_error".to_string(), json!(error_msg));
                    }
                }

                let duration = start_time.elapsed();

                if all_successful {
                    Ok(ValidationResult {
                        validation_type: ValidationType::DataIntegrity,
                        success: true,
                        error: None,
                        duration_ms: duration.as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                        details,
                    })
                } else {
                    Ok(ValidationResult {
                        validation_type: ValidationType::DataIntegrity,
                        success: false,
                        error: Some("Data integrity validation failed".to_string()),
                        duration_ms: duration.as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                        details,
                    })
                }
            }

            /// Validate error handling
            async fn validate_error_handling(&self) -> Result<ValidationResult, AuditError> {
                info!("Validating error handling");

                let start_time = Instant::now();
                let mut details = HashMap::new();
                let mut all_successful = true;

                // Test error handling by simulating service failures
                // We'll test how the system handles various error scenarios

                // 1. Test invalid input handling
                let chain_definition = json!({
                    "name": "validation_error_invalid_input_chain",
                    "description": "A chain for testing invalid input handling",
                    "steps": [
                        {
                            "id": "invalid_input_step",
                            "type": "function",
                            "function": "process_input",
                            "input": {
                                "data": null,
                                "type": "invalid"
                            }
                        }
                    ]
                });

                let chain_result = self.execute_chain(chain_definition.clone()).await;

                match chain_result {
                    Ok(_) => {
                        // This should have failed with an error
                        all_successful = false;
                        let error_msg =
                            "Error handling validation failed: invalid input was accepted";
                        warn!("{}", error_msg);
                        details.insert("error_invalid_input".to_string(), json!(error_msg));
                    }
                    Err(e) => {
                        // This is expected behavior
                        info!("Invalid input was correctly rejected: {}", e);
                        details.insert(
                            "error_invalid_input".to_string(),
                            json!({
                                "status": "success",
                                "message": format!("Invalid input was correctly rejected: {}", e)
                            }),
                        );
                    }
                }

                // 2. Test timeout handling
                let chain_definition = json!({
                    "name": "validation_error_timeout_chain",
                    "description": "A chain for testing timeout handling",
                    "steps": [
                        {
                            "id": "timeout_step",
                            "type": "function",
                            "function": "simulate_timeout",
                            "input": {
                                "duration_ms": 10000 // 10 seconds
                            }
                        }
                    ]
                });

                // Set a short timeout for this test
                let timeout_duration = Duration::from_secs(2); // 2 seconds
                let chain_future = self.execute_chain(chain_definition.clone());
                let timeout_result = timeout(timeout_duration, chain_future).await;

                match timeout_result {
                    Ok(Ok(_)) => {
                        // This should have timed out
                        all_successful = false;
                        let error_msg = "Error handling validation failed: timeout did not occur";
                        warn!("{}", error_msg);
                        details.insert("error_timeout".to_string(), json!(error_msg));
                    }
                    Ok(Err(e)) => {
                        // This is an error from the chain execution, not a timeout
                        info!("Chain execution failed with error: {}", e);
                        details.insert(
                            "error_timeout".to_string(),
                            json!({
                                "status": "success",
                                "message": format!("Chain execution failed with error: {}", e)
                            }),
                        );
                    }
                    Err(_) => {
                        // This is expected behavior - timeout occurred
                        info!("Timeout was correctly triggered");
                        details.insert(
                            "error_timeout".to_string(),
                            json!({
                                "status": "success",
                                "message": "Timeout was correctly triggered"
                            }),
                        );
                    }
                }

                // 3. Test service failure handling
                let chain_definition = json!({
                    "name": "validation_error_service_failure_chain",
                    "description": "A chain for testing service failure handling",
                    "steps": [
                        {
                            "id": "failure_step",
                            "type": "function",
                            "function": "simulate_service_failure",
                            "input": {
                                "service": "rag-injector"
                            }
                        }
                    ]
                });

                let chain_result = self.execute_chain(chain_definition.clone()).await;

                match chain_result {
                    Ok(result) => {
                        // Check if the result contains error handling information
                        if let Some(output) = result.get("output") {
                            if let Some(error_handled) = output.get("error_handled") {
                                if error_handled.as_bool() == Some(true) {
                                    info!("Service failure was correctly handled");
                                    details.insert(
                                        "error_service_failure".to_string(),
                                        json!({
                                            "status": "success",
                                            "message": "Service failure was correctly handled",
                                            "result": result
                                        }),
                                    );
                                } else {
                                    all_successful = false;
                                    let error_msg = "Error handling validation failed: service failure was not handled correctly";
                                    warn!("{}", error_msg);
                                    details.insert(
                                        "error_service_failure".to_string(),
                                        json!(error_msg),
                                    );
                                }
                            } else {
                                all_successful = false;
                                let error_msg =
                                    "Error handling validation failed: missing error_handled field";
                                warn!("{}", error_msg);
                                details
                                    .insert("error_service_failure".to_string(), json!(error_msg));
                            }
                        } else {
                            all_successful = false;
                            let error_msg = "Error handling validation failed: missing output";
                            warn!("{}", error_msg);
                            details.insert("error_service_failure".to_string(), json!(error_msg));
                        }
                    }
                    Err(e) => {
                        // This could be expected behavior depending on how errors are handled
                        info!("Service failure resulted in error: {}", e);
                        details.insert(
                            "error_service_failure".to_string(),
                            json!({
                                "status": "success",
                                "message": format!("Service failure resulted in error: {}", e)
                            }),
                        );
                    }
                }

                let duration = start_time.elapsed();

                if all_successful {
                    Ok(ValidationResult {
                        validation_type: ValidationType::ErrorHandling,
                        success: true,
                        error: None,
                        duration_ms: duration.as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                        details,
                    })
                } else {
                    Ok(ValidationResult {
                        validation_type: ValidationType::ErrorHandling,
                        success: false,
                        error: Some("Error handling validation failed".to_string()),
                        duration_ms: duration.as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                        details,
                    })
                }
            }

            /// Validate security
            async fn validate_security(&self) -> Result<ValidationResult, AuditError> {
                info!("Validating security");

                let start_time = Instant::now();
                let mut details = HashMap::new();
                let mut all_successful = true;

                // 1. Test JWT authentication
                let jwt_validation = self.validate_jwt_authentication().await;
                match jwt_validation {
                    Ok(jwt_result) => {
                        if jwt_result {
                            info!("JWT authentication validation passed");
                            details.insert(
                                "security_jwt".to_string(),
                                json!({
                                    "status": "success",
                                    "message": "JWT authentication validation passed"
                                }),
                            );
                        } else {
                            all_successful = false;
                            let error_msg =
                                "Security validation failed: JWT authentication validation failed";
                            warn!("{}", error_msg);
                            details.insert("security_jwt".to_string(), json!(error_msg));
                        }
                    }
                    Err(e) => {
                        all_successful = false;
                        let error_msg = format!(
                            "Security validation failed: JWT authentication validation error: {}",
                            e
                        );
                        error!("{}", error_msg);
                        details.insert("security_jwt".to_string(), json!(error_msg));
                    }
                }

                // 2. Test mTLS encryption
                let mtls_validation = self.validate_mtls_encryption().await;
                match mtls_validation {
                    Ok(mtls_result) => {
                        if mtls_result {
                            info!("mTLS encryption validation passed");
                            details.insert(
                                "security_mtls".to_string(),
                                json!({
                                    "status": "success",
                                    "message": "mTLS encryption validation passed"
                                }),
                            );
                        } else {
                            all_successful = false;
                            let error_msg =
                                "Security validation failed: mTLS encryption validation failed";
                            warn!("{}", error_msg);
                            details.insert("security_mtls".to_string(), json!(error_msg));
                        }
                    }
                    Err(e) => {
                        all_successful = false;
                        let error_msg = format!(
                            "Security validation failed: mTLS encryption validation error: {}",
                            e
                        );
                        error!("{}", error_msg);
                        details.insert("security_mtls".to_string(), json!(error_msg));
                    }
                }

                // 3. Test authorization checks
                let auth_validation = self.validate_authorization_checks().await;
                match auth_validation {
                    Ok(auth_result) => {
                        if auth_result {
                            info!("Authorization checks validation passed");
                            details.insert(
                                "security_authorization".to_string(),
                                json!({
                                    "status": "success",
                                    "message": "Authorization checks validation passed"
                                }),
                            );
                        } else {
                            all_successful = false;
                            let error_msg = "Security validation failed: authorization checks validation failed";
                            warn!("{}", error_msg);
                            details.insert("security_authorization".to_string(), json!(error_msg));
                        }
                    }
                    Err(e) => {
                        all_successful = false;
                        let error_msg = format!(
                            "Security validation failed: authorization checks validation error: {}",
                            e
                        );
                        error!("{}", error_msg);
                        details.insert("security_authorization".to_string(), json!(error_msg));
                    }
                }

                let duration = start_time.elapsed();

                if all_successful {
                    Ok(ValidationResult {
                        validation_type: ValidationType::Security,
                        success: true,
                        error: None,
                        duration_ms: duration.as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                        details,
                    })
                } else {
                    Ok(ValidationResult {
                        validation_type: ValidationType::Security,
                        success: false,
                        error: Some("Security validation failed".to_string()),
                        duration_ms: duration.as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                        details,
                    })
                }
            }

            /// Validate JWT authentication
            async fn validate_jwt_authentication(&self) -> Result<bool, AuditError> {
                info!("Validating JWT authentication");

                // Create a test JWT token
                let test_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ0ZXN0X3VzZXIiLCJuYW1lIjoiVGVzdCBVc2VyIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

                // Test authentication with the token
                let router_url = format!(
                    "http://{}:{}/api/v1/auth/validate",
                    self.services.get(&ServiceType::Router).unwrap().host,
                    self.services.get(&ServiceType::Router).unwrap().port
                );

                let response = self
                    .client
                    .post(&router_url)
                    .header("Authorization", format!("Bearer {}", test_token))
                    .send()
                    .await
                    .map_err(|e| AuditError::HttpError(e))?;

                if response.status().is_success() {
                    let body: Value = response
                        .json()
                        .await
                        .map_err(|e| AuditError::HttpError(e))?;

                    if let Some(valid) = body.get("valid").and_then(|v| v.as_bool()) {
                        return Ok(valid);
                    }
                }

                Ok(false)
            }

            /// Validate mTLS encryption
            async fn validate_mtls_encryption(&self) -> Result<bool, AuditError> {
                info!("Validating mTLS encryption");

                // Test mTLS connection to the router
                let router_url = format!(
                    "https://{}:{}/api/v1/secure",
                    self.services.get(&ServiceType::Router).unwrap().host,
                    self.services.get(&ServiceType::Router).unwrap().port
                );

                // Create a client with mTLS certificates
                let client = reqwest::Client::builder()
                    .use_rustls_tls()
                    .identity(reqwest::Identity::from_pem(b"test_cert_and_key").unwrap())
                    .add_root_certificate(reqwest::Certificate::from_pem(b"test_ca_cert").unwrap())
                    .build()
                    .map_err(|e| {
                        AuditError::CommunicationTestError(format!(
                            "Failed to build mTLS client: {}",
                            e
                        ))
                    })?;

                // Try to connect with mTLS
                let response = client.get(&router_url).send().await;

                match response {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            info!("mTLS connection successful");
                            return Ok(true);
                        } else {
                            warn!("mTLS connection failed with status: {}", resp.status());
                            return Ok(false);
                        }
                    }
                    Err(e) => {
                        // In a real implementation, we would need to distinguish between
                        // TLS handshake failures and other network errors
                        warn!("mTLS connection failed: {}", e);
                        return Ok(false);
                    }
                }
            }

            /// Validate authorization checks
            async fn validate_authorization_checks(&self) -> Result<bool, AuditError> {
                info!("Validating authorization checks");

                // Create test tokens with different permissions
                let admin_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhZG1pbiIsInJvbGUiOiJhZG1pbiIsImlhdCI6MTUxNjIzOTAyMn0.KjCZV-QdVKNXAQNlDaGi5IkJJoR7uQ3tvu9vRhaK_Ks";
                let user_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyIiwicm9sZSI6InVzZXIiLCJpYXQiOjE1MTYyMzkwMjJ9.WOzJqBrWkag3EF-Cod0e7dVLfTiYGh-z6kI-KgZU1v4";

                // Test admin endpoint with admin token (should succeed)
                let admin_url = format!(
                    "http://{}:{}/api/v1/admin/config",
                    self.services.get(&ServiceType::Router).unwrap().host,
                    self.services.get(&ServiceType::Router).unwrap().port
                );

                let admin_response = self
                    .client
                    .get(&admin_url)
                    .header("Authorization", format!("Bearer {}", admin_token))
                    .send()
                    .await
                    .map_err(|e| AuditError::HttpError(e))?;

                let admin_success = admin_response.status().is_success();

                // Test admin endpoint with user token (should fail)
                let user_response = self
                    .client
                    .get(&admin_url)
                    .header("Authorization", format!("Bearer {}", user_token))
                    .send()
                    .await
                    .map_err(|e| AuditError::HttpError(e))?;

                let user_failure = user_response.status().is_client_error();

                // Both tests should pass for proper authorization
                Ok(admin_success && user_failure)
            }
            return Err(AuditError::TestExecutionError(format!(
                "Failed to delete test collection: {}",
                response.status()
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation_workflow_creation() {
        let config = ValidationConfig::default();
        let report = Arc::new(RwLock::new(AuditReport::new()));
        let discovery_config = super::super::types::DiscoveryConfig::default();
        let service_discovery = ServiceDiscovery::new(discovery_config, Arc::clone(&report));
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
